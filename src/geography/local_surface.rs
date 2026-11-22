//! Bounded projection of macro-geography and river channels into one local area.
//!
//! The macro lattice and local tile grid have independent resolutions. The
//! location footprint is sampled at a bounded set of points for broad terrain,
//! while deterministic channel geometry is evaluated at tile centres so narrow
//! rivers are not lost between coarse samples.

use super::{
    climate::{self, Biome},
    river_geometry::{RiverGeometryCache, RiverPosition},
    rivers::RiverWidthClass,
    terrain, waterbodies, MacroCoordinate,
};

const MAX_SAMPLES_PER_AXIS: usize = 17;
const MAX_LOCAL_MAP_DIMENSION: u16 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceWaterFeature {
    Land,
    Ocean,
    /// Non-ocean water that blocks local movement, including lakes and rivers.
    Lake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSurfaceSample {
    biome: Biome,
    water: SurfaceWaterFeature,
    mountain_barrier: bool,
    river_width: Option<RiverWidthClass>,
}

impl LocalSurfaceSample {
    pub const fn biome(self) -> Biome {
        self.biome
    }

    pub const fn water(self) -> SurfaceWaterFeature {
        self.water
    }

    pub const fn is_mountain_barrier(self) -> bool {
        self.mountain_barrier
    }

    /// Relative channel width when this tile lies inside a generated river.
    pub const fn river_width(self) -> Option<RiverWidthClass> {
        self.river_width
    }

    pub const fn is_river(self) -> bool {
        self.river_width.is_some()
    }
}

/// A broad macro sample grid plus a tile-resolution river footprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSurfaceMap {
    tile_width: u16,
    tile_height: u16,
    sample_width: u16,
    sample_height: u16,
    samples: Vec<LocalSurfaceSample>,
    river_tiles: Vec<Option<RiverWidthClass>>,
}

impl LocalSurfaceMap {
    /// Sample a location footprint for the requested local-map dimensions.
    ///
    /// Broad terrain uses at most 17 × 17 macro positions. River geometry is
    /// then checked at local tile centres against cached channel segments. This
    /// second pass is also bounded: at most 289 distinct coarse query cells and
    /// nine candidate source cells per query. Macro samples are never treated
    /// as individual playable tiles or physical distance units.
    pub fn sample_location(
        world_seed: u64,
        ruleset_version: u32,
        anchor: MacroCoordinate,
        footprint_radius_samples: u16,
        tile_width: u16,
        tile_height: u16,
    ) -> Option<Self> {
        if anchor.stratum != 0
            || tile_width == 0
            || tile_height == 0
            || tile_width > MAX_LOCAL_MAP_DIMENSION
            || tile_height > MAX_LOCAL_MAP_DIMENSION
        {
            return None;
        }

        let radius_x = footprint_radius_samples;
        let radius_y = if radius_x == 0 {
            0
        } else {
            let scaled = (u64::from(radius_x) * u64::from(tile_height) + u64::from(tile_width) / 2)
                / u64::from(tile_width);
            scaled.clamp(1, u64::from(u16::MAX)) as u16
        };
        let x_coordinates = sample_axis(anchor.x, radius_x)?;
        let y_coordinates = sample_axis(anchor.y, radius_y)?;
        let sample_width = u16::try_from(x_coordinates.len()).ok()?;
        let sample_height = u16::try_from(y_coordinates.len()).ok()?;
        let mut samples = Vec::with_capacity(x_coordinates.len() * y_coordinates.len());

        for y in &y_coordinates {
            for x in &x_coordinates {
                let coordinate = MacroCoordinate::new(*x, *y);
                let terrain_sample =
                    terrain::sample_surface(world_seed, ruleset_version, coordinate)?;
                let climate_sample =
                    climate::sample_surface(world_seed, ruleset_version, coordinate)?;
                let water = if !terrain_sample.is_land() {
                    SurfaceWaterFeature::Ocean
                } else if waterbodies::sample_surface(world_seed, ruleset_version, coordinate)
                    .is_some()
                {
                    SurfaceWaterFeature::Lake
                } else {
                    SurfaceWaterFeature::Land
                };

                samples.push(LocalSurfaceSample {
                    biome: climate_sample.biome(),
                    water,
                    mountain_barrier: terrain_sample.is_mountain_barrier(),
                    river_width: None,
                });
            }
        }

        let tile_count = usize::from(tile_width) * usize::from(tile_height);
        let mut result = Self {
            tile_width,
            tile_height,
            sample_width,
            sample_height,
            samples,
            river_tiles: vec![None; tile_count],
        };

        let mut river_cache = RiverGeometryCache::default();
        for tile_y in 0..tile_height {
            for tile_x in 0..tile_width {
                let Some(base_sample) = result.macro_sample_at_tile(tile_x, tile_y) else {
                    continue;
                };
                if base_sample.water != SurfaceWaterFeature::Land {
                    continue;
                }

                let position = tile_center_position(
                    anchor,
                    radius_x,
                    radius_y,
                    tile_x,
                    tile_y,
                    tile_width,
                    tile_height,
                )?;
                let segments = river_cache.segments_near(world_seed, ruleset_version, position);
                let width = segments
                    .iter()
                    .filter(|segment| segment.contains_position(position))
                    .map(|segment| segment.width_class())
                    .max_by_key(|width| width_rank(*width));
                if let Some(width) = width {
                    let index = usize::from(tile_y) * usize::from(tile_width) + usize::from(tile_x);
                    result.river_tiles[index] = Some(width);
                }
            }
        }

        Some(result)
    }

    pub const fn dimensions(&self) -> (u16, u16) {
        (self.tile_width, self.tile_height)
    }

    /// Dimensions of the bounded underlying macro-sample grid.
    pub const fn sample_dimensions(&self) -> (u16, u16) {
        (self.sample_width, self.sample_height)
    }

    /// Return the broad terrain and any local river channel covering one tile.
    pub fn sample_at_tile(&self, tile_x: u16, tile_y: u16) -> Option<LocalSurfaceSample> {
        let mut sample = self.macro_sample_at_tile(tile_x, tile_y)?;
        let index = usize::from(tile_y) * usize::from(self.tile_width) + usize::from(tile_x);
        if let Some(width) = self.river_tiles.get(index).copied().flatten() {
            sample.water = SurfaceWaterFeature::Lake;
            sample.river_width = Some(width);
        }
        Some(sample)
    }

    fn macro_sample_at_tile(&self, tile_x: u16, tile_y: u16) -> Option<LocalSurfaceSample> {
        if tile_x >= self.tile_width || tile_y >= self.tile_height {
            return None;
        }

        let sample_x = (((usize::from(tile_x) * 2 + 1) * usize::from(self.sample_width))
            / (usize::from(self.tile_width) * 2))
            .min(usize::from(self.sample_width.saturating_sub(1)));
        let sample_y = (((usize::from(tile_y) * 2 + 1) * usize::from(self.sample_height))
            / (usize::from(self.tile_height) * 2))
            .min(usize::from(self.sample_height.saturating_sub(1)));
        self.samples
            .get(sample_y * usize::from(self.sample_width) + sample_x)
            .copied()
    }
}

fn tile_center_position(
    anchor: MacroCoordinate,
    radius_x: u16,
    radius_y: u16,
    tile_x: u16,
    tile_y: u16,
    tile_width: u16,
    tile_height: u16,
) -> Option<RiverPosition> {
    use super::river_geometry::RIVER_POSITION_SCALE;

    let x_offset_numerator = i128::from(tile_x) * 2 + 1 - i128::from(tile_width);
    let y_offset_numerator = i128::from(tile_y) * 2 + 1 - i128::from(tile_height);
    let x_offset = x_offset_numerator * i128::from(radius_x) * i128::from(RIVER_POSITION_SCALE)
        / i128::from(tile_width);
    let y_offset = y_offset_numerator * i128::from(radius_y) * i128::from(RIVER_POSITION_SCALE)
        / i128::from(tile_height);
    Some(RiverPosition {
        x_q16: i64::try_from(i128::from(anchor.x) * i128::from(RIVER_POSITION_SCALE) + x_offset)
            .ok()?,
        y_q16: i64::try_from(i128::from(anchor.y) * i128::from(RIVER_POSITION_SCALE) + y_offset)
            .ok()?,
    })
}

fn width_rank(width: RiverWidthClass) -> u8 {
    match width {
        RiverWidthClass::Narrow => 0,
        RiverWidthClass::Medium => 1,
        RiverWidthClass::Wide => 2,
        RiverWidthClass::VeryWide => 3,
    }
}

fn sample_axis(center: i64, radius: u16) -> Option<Vec<i64>> {
    let radius = i64::from(radius);
    let count = (usize::try_from(radius)
        .ok()?
        .saturating_mul(2)
        .saturating_add(1))
    .min(MAX_SAMPLES_PER_AXIS);
    if count == 1 {
        return Some(vec![center]);
    }

    let span = radius.checked_mul(2)?;
    (0..count)
        .map(|index| {
            let offset =
                -radius + span * i64::try_from(index).ok()? / i64::try_from(count - 1).ok()?;
            center.checked_add(offset)
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/local_surface.rs"]
mod tests;

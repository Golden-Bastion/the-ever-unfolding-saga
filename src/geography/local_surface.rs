//! Bounded projection of macro-geography into one local exploration area.
//!
//! The macro lattice and local tile grid have independent resolutions. This
//! module samples the location's persisted footprint at a bounded set of points
//! and resamples those points across the local map; it never assumes one macro
//! sample corresponds to one playable tile.

use super::{
    climate::{self, Biome},
    terrain, waterbodies, MacroCoordinate,
};

const MAX_SAMPLES_PER_AXIS: usize = 17;
const MAX_LOCAL_MAP_DIMENSION: u16 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceWaterFeature {
    Land,
    Ocean,
    Lake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSurfaceSample {
    biome: Biome,
    water: SurfaceWaterFeature,
    mountain_barrier: bool,
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
}

/// A small macro-sample grid resampled over a local map's tile dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSurfaceMap {
    tile_width: u16,
    tile_height: u16,
    sample_width: u16,
    sample_height: u16,
    samples: Vec<LocalSurfaceSample>,
}

impl LocalSurfaceMap {
    /// Sample a location footprint for the requested local-map dimensions.
    ///
    /// The horizontal footprint uses the persisted radius. The vertical radius
    /// is scaled by the map aspect ratio so macro sampling remains isotropic
    /// without introducing a physical tile-to-macro-unit conversion. At most
    /// 17 × 17 macro positions are evaluated, regardless of footprint size.
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
                });
            }
        }

        Some(Self {
            tile_width,
            tile_height,
            sample_width,
            sample_height,
            samples,
        })
    }

    pub const fn dimensions(&self) -> (u16, u16) {
        (self.tile_width, self.tile_height)
    }

    /// Dimensions of the bounded underlying macro-sample grid.
    pub const fn sample_dimensions(&self) -> (u16, u16) {
        (self.sample_width, self.sample_height)
    }

    /// Return the macro-derived sample covering one local tile.
    ///
    /// Nearest-cell resampling is intentional for this first presentation
    /// stage: it keeps water/land boundaries discrete and avoids inventing
    /// interpolated coastlines or river geometry.
    pub fn sample_at_tile(&self, tile_x: u16, tile_y: u16) -> Option<LocalSurfaceSample> {
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

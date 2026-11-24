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

/// The side where a macro route leaves a location's rectangular surface footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalSurfaceEdgeSide {
    North,
    East,
    South,
    West,
}

/// Exact normalized position along a local-map edge, in the inclusive Q16 range 0..=65535.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSurfaceEdge {
    pub side: LocalSurfaceEdgeSide,
    pub offset_q16: u16,
}

/// A tile coordinate in a projected local surface map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSurfaceTile {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSurfaceSample {
    biome: Biome,
    water: SurfaceWaterFeature,
    elevation_q16: i32,
    mountain_ridge_strength_q16: u32,
    moisture_q16: u32,
    shoreline_mask: u8,
    shoreline_strength_q16: u16,
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

    pub const fn elevation_q16(self) -> i32 {
        self.elevation_q16
    }

    pub const fn mountain_ridge_strength_q16(self) -> u32 {
        self.mountain_ridge_strength_q16
    }

    pub const fn moisture_q16(self) -> u32 {
        self.moisture_q16
    }

    /// Direction bits are shared with geography::coast::direction.
    pub const fn shoreline_mask(self) -> u8 {
        self.shoreline_mask
    }

    pub const fn is_shoreline(self) -> bool {
        self.shoreline_mask != 0
    }

    /// Smooth presentation influence derived from nearby shoreline samples.
    ///
    /// Unlike the directional adjacency mask, this value is bilinearly
    /// interpolated across the bounded local projection grid.
    pub const fn shoreline_strength_q16(self) -> u16 {
        self.shoreline_strength_q16
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
    anchor: MacroCoordinate,
    radius_x: u16,
    radius_y: u16,
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
                let shoreline_sample =
                    super::coast::sample_surface(world_seed, ruleset_version, coordinate)?;
                let water = if !terrain_sample.is_land() {
                    SurfaceWaterFeature::Ocean
                } else if waterbodies::sample_surface(world_seed, ruleset_version, coordinate)
                    .is_some()
                {
                    SurfaceWaterFeature::Lake
                } else {
                    SurfaceWaterFeature::Land
                };

                let shoreline_mask = shoreline_sample.opposite_terrain_mask();
                samples.push(LocalSurfaceSample {
                    biome: climate_sample.biome(),
                    water,
                    elevation_q16: terrain_sample.elevation_q16(),
                    mountain_ridge_strength_q16: terrain_sample.mountain_ridge_strength_q16(),
                    moisture_q16: climate_sample.moisture_q16(),
                    shoreline_mask,
                    shoreline_strength_q16: if shoreline_mask == 0 { 0 } else { u16::MAX },
                    mountain_barrier: terrain_sample.is_mountain_barrier(),
                    river_width: None,
                });
            }
        }

        let tile_count = usize::from(tile_width) * usize::from(tile_height);
        let mut result = Self {
            anchor,
            radius_x,
            radius_y,
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

    /// Project the first route segment leaving this location onto its actual footprint edge.
    ///
    /// Route paths are stored from endpoint A to endpoint B. The supplied endpoint
    /// must match this surface map's persisted location anchor; legacy or malformed
    /// paths safely fall back to their older compass-heading placement.
    pub fn route_edge(
        &self,
        path: &[MacroCoordinate],
        from_anchor_a: bool,
    ) -> Option<LocalSurfaceEdge> {
        if self.radius_x == 0 || self.radius_y == 0 {
            return None;
        }
        let endpoint = if from_anchor_a {
            path.first().copied()?
        } else {
            path.last().copied()?
        };
        if endpoint != self.anchor {
            return None;
        }
        let next = if from_anchor_a {
            path.iter().copied().find(|point| *point != self.anchor)?
        } else {
            path.iter()
                .rev()
                .copied()
                .find(|point| *point != self.anchor)?
        };
        let dx = i128::from(next.x) - i128::from(self.anchor.x);
        let dy = i128::from(next.y) - i128::from(self.anchor.y);
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        if abs_dx == 0 && abs_dy == 0 {
            return None;
        }

        let radius_x = i128::from(self.radius_x);
        let radius_y = i128::from(self.radius_y);
        // Compare ray parameters without division. Exact corner hits resolve to
        // the east/west side, matching the existing compass-heading tie-break.
        let hits_vertical_edge = radius_x * abs_dy <= radius_y * abs_dx;
        if hits_vertical_edge {
            Some(LocalSurfaceEdge {
                side: if dx > 0 {
                    LocalSurfaceEdgeSide::East
                } else {
                    LocalSurfaceEdgeSide::West
                },
                offset_q16: normalized_edge_offset_q16(dy * radius_x, abs_dx, self.radius_y)?,
            })
        } else {
            Some(LocalSurfaceEdge {
                side: if dy > 0 {
                    LocalSurfaceEdgeSide::South
                } else {
                    LocalSurfaceEdgeSide::North
                },
                offset_q16: normalized_edge_offset_q16(dx * radius_y, abs_dy, self.radius_x)?,
            })
        }
    }

    /// Convert an exact global Q16 geography position to the local tile containing it.
    /// Positions outside the footprint are rejected rather than clamped to an edge.
    pub fn tile_for_position(&self, position: RiverPosition) -> Option<LocalSurfaceTile> {
        Some(LocalSurfaceTile {
            x: coordinate_to_tile(
                position.x_q16,
                self.anchor.x,
                self.radius_x,
                self.tile_width,
            )?,
            y: coordinate_to_tile(
                position.y_q16,
                self.anchor.y,
                self.radius_y,
                self.tile_height,
            )?,
        })
    }

    /// Dimensions of the bounded underlying macro-sample grid.
    pub const fn sample_dimensions(&self) -> (u16, u16) {
        (self.sample_width, self.sample_height)
    }

    /// Return the broad terrain and any local river channel covering one tile.
    ///
    /// Continuous environmental fields are bilinearly interpolated from the
    /// bounded macro grid. Biome, water classification, directional shoreline
    /// mask, mountain-barrier flag, and river width remain nearest-sample data.
    pub fn sample_at_tile(&self, tile_x: u16, tile_y: u16) -> Option<LocalSurfaceSample> {
        let mut sample = self.macro_sample_at_tile(tile_x, tile_y)?;
        let (x0, x1, fraction_x) =
            sample_grid_position_q16(tile_x, self.tile_width, self.sample_width)?;
        let (y0, y1, fraction_y) =
            sample_grid_position_q16(tile_y, self.tile_height, self.sample_height)?;
        let top_left = self.grid_sample(x0, y0)?;
        let top_right = self.grid_sample(x1, y0)?;
        let bottom_left = self.grid_sample(x0, y1)?;
        let bottom_right = self.grid_sample(x1, y1)?;

        sample.elevation_q16 = bilinear_interpolate_q16(
            i64::from(top_left.elevation_q16),
            i64::from(top_right.elevation_q16),
            i64::from(bottom_left.elevation_q16),
            i64::from(bottom_right.elevation_q16),
            fraction_x,
            fraction_y,
        ) as i32;
        sample.mountain_ridge_strength_q16 = bilinear_interpolate_q16(
            i64::from(top_left.mountain_ridge_strength_q16),
            i64::from(top_right.mountain_ridge_strength_q16),
            i64::from(bottom_left.mountain_ridge_strength_q16),
            i64::from(bottom_right.mountain_ridge_strength_q16),
            fraction_x,
            fraction_y,
        )
        .clamp(0, i64::from(u32::MAX)) as u32;
        sample.moisture_q16 = bilinear_interpolate_q16(
            i64::from(top_left.moisture_q16),
            i64::from(top_right.moisture_q16),
            i64::from(bottom_left.moisture_q16),
            i64::from(bottom_right.moisture_q16),
            fraction_x,
            fraction_y,
        )
        .clamp(0, i64::from(u32::MAX)) as u32;
        sample.shoreline_strength_q16 = bilinear_interpolate_q16(
            i64::from(top_left.shoreline_strength_q16),
            i64::from(top_right.shoreline_strength_q16),
            i64::from(bottom_left.shoreline_strength_q16),
            i64::from(bottom_right.shoreline_strength_q16),
            fraction_x,
            fraction_y,
        )
        .clamp(0, i64::from(u16::MAX)) as u16;

        let index = usize::from(tile_y) * usize::from(self.tile_width) + usize::from(tile_x);
        if let Some(width) = self.river_tiles.get(index).copied().flatten() {
            sample.water = SurfaceWaterFeature::Lake;
            sample.river_width = Some(width);
        }
        Some(sample)
    }

    fn grid_sample(&self, x: usize, y: usize) -> Option<LocalSurfaceSample> {
        self.samples
            .get(y.checked_mul(usize::from(self.sample_width))?.checked_add(x)?)
            .copied()
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

fn normalized_edge_offset_q16(
    offset_numerator: i128,
    step_denominator: i128,
    radius: u16,
) -> Option<u16> {
    if step_denominator <= 0 || radius == 0 {
        return None;
    }
    let radius = i128::from(radius);
    let denominator = step_denominator.checked_mul(radius)?.checked_mul(2)?;
    let numerator = offset_numerator.checked_add(step_denominator.checked_mul(radius)?)?;
    let scaled = numerator.checked_mul(i128::from(u16::MAX))? / denominator;
    u16::try_from(scaled.clamp(0, i128::from(u16::MAX))).ok()
}

fn coordinate_to_tile(position_q16: i64, anchor: i64, radius: u16, dimension: u16) -> Option<u16> {
    if dimension == 0 {
        return None;
    }
    let scale = i128::from(super::river_geometry::RIVER_POSITION_SCALE);
    let center = i128::from(anchor).checked_mul(scale)?;
    let radius_q16 = i128::from(radius).checked_mul(scale)?;
    if radius == 0 {
        return (i128::from(position_q16) == center).then_some(dimension / 2);
    }
    let minimum = center.checked_sub(radius_q16)?;
    let maximum = center.checked_add(radius_q16)?;
    let position = i128::from(position_q16);
    if position < minimum || position > maximum {
        return None;
    }
    let span = radius_q16.checked_mul(2)?;
    let tile = position
        .checked_sub(minimum)?
        .checked_mul(i128::from(dimension))?
        / span;
    u16::try_from(tile.clamp(0, i128::from(dimension - 1))).ok()
}

/// Map a local tile center to the continuous macro-sample grid in Q16.
fn sample_grid_position_q16(
    tile: u16,
    tile_dimension: u16,
    sample_dimension: u16,
) -> Option<(usize, usize, i64)> {
    if tile >= tile_dimension || tile_dimension == 0 || sample_dimension == 0 {
        return None;
    }
    if sample_dimension == 1 {
        return Some((0, 0, 0));
    }

    // The location anchor remains aligned with the center tile on even-sized
    // maps, matching the existing nearest-sample projection contract.
    let position_q16 = i64::from(tile)
        .checked_mul(i64::from(sample_dimension - 1))?
        .checked_mul(2 * (1_i64 << 16))?
        / i64::from(tile_dimension);
    let maximum = i64::from(sample_dimension - 1) * (1_i64 << 16);
    let position_q16 = position_q16.clamp(0, maximum);
    let lower = usize::try_from(position_q16 >> 16).ok()?;
    let upper = (lower + 1).min(usize::from(sample_dimension - 1));
    Some((lower, upper, position_q16 & 0xFFFF))
}

fn interpolate_q16(from: i64, to: i64, fraction: i64) -> i64 {
    from + (to - from) * fraction / (1_i64 << 16)
}

fn bilinear_interpolate_q16(
    top_left: i64,
    top_right: i64,
    bottom_left: i64,
    bottom_right: i64,
    fraction_x: i64,
    fraction_y: i64,
) -> i64 {
    let top = interpolate_q16(top_left, top_right, fraction_x);
    let bottom = interpolate_q16(bottom_left, bottom_right, fraction_x);
    interpolate_q16(top, bottom, fraction_y)
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

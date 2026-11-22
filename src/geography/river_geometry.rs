//! Deterministic local channel geometry derived from the macro watershed graph.
//!
//! A channel is a bounded curved strip between adjacent coarse watershed-cell
//! centres. Adjacent segments share the exact same endpoint, so the channel
//! remains connected across cells and region boundaries regardless of generation
//! order. Widths are presentational macro-sample fractions, not physical metres
//! or simulated discharge.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    rivers::{self, RiverFlowDirection, RiverId, RiverWidthClass},
    terrain, waterbodies, watershed, MacroCoordinate,
};

/// One macro sample expressed in fixed-point Q16 coordinates.
pub const RIVER_POSITION_SCALE: i64 = 1 << 16;
const CURVE_SUBDIVISIONS: i64 = 12;
const MAX_NEARBY_RIVER_SEGMENTS: usize = 9;
const MAX_CACHED_QUERY_CELLS: usize = 289;
const MAX_CACHED_SOURCE_CELLS: usize = 361;

/// A sub-sample position on the shared macro-geography lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RiverPosition {
    pub x_q16: i64,
    pub y_q16: i64,
}

impl RiverPosition {
    pub fn from_macro_coordinate(coordinate: MacroCoordinate) -> Option<Self> {
        if coordinate.stratum != 0 {
            return None;
        }
        Some(Self {
            x_q16: coordinate.x.checked_mul(RIVER_POSITION_SCALE)?,
            y_q16: coordinate.y.checked_mul(RIVER_POSITION_SCALE)?,
        })
    }

    /// Round this fixed-point point to the nearest macro sample.
    pub fn rounded_macro_coordinate(self) -> Option<MacroCoordinate> {
        Some(MacroCoordinate::new(
            round_q16(self.x_q16)?,
            round_q16(self.y_q16)?,
        ))
    }

    pub fn midpoint(self, other: Self) -> Option<Self> {
        Some(Self {
            x_q16: i64::try_from((i128::from(self.x_q16) + i128::from(other.x_q16)) / 2).ok()?,
            y_q16: i64::try_from((i128::from(self.y_q16) + i128::from(other.y_q16)) / 2).ok()?,
        })
    }

    pub fn offset(self, dx_q16: i64, dy_q16: i64) -> Option<Self> {
        Some(Self {
            x_q16: self.x_q16.checked_add(dx_q16)?,
            y_q16: self.y_q16.checked_add(dy_q16)?,
        })
    }
}

/// Exact bank-entry and bank-exit points where an overland route intersects a
/// generated river channel. Points are stored in route travel order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiverBankIntersection {
    pub entry: RiverPosition,
    pub exit: RiverPosition,
}

impl RiverBankIntersection {
    pub fn midpoint(self) -> Option<RiverPosition> {
        self.entry.midpoint(self.exit)
    }
}

/// One continuous local-scale river channel between adjacent watershed cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverChannelSegment {
    river_id: RiverId,
    width: RiverWidthClass,
    flow_direction: RiverFlowDirection,
    half_width_q16: i64,
    centerline: Vec<RiverPosition>,
}

impl RiverChannelSegment {
    pub const fn river_id(&self) -> RiverId {
        self.river_id
    }

    pub const fn width_class(&self) -> RiverWidthClass {
        self.width
    }

    pub const fn flow_direction(&self) -> RiverFlowDirection {
        self.flow_direction
    }

    pub fn centerline(&self) -> &[RiverPosition] {
        &self.centerline
    }

    /// Whether a global local-map tile centre falls inside this channel band.
    pub fn contains_position(&self, position: RiverPosition) -> bool {
        self.centerline
            .windows(2)
            .any(|pair| point_within_segment(position, pair[0], pair[1], self.half_width_q16))
    }

    /// Whether a coordinate lies within a requested distance of the channel
    /// edge. Used by location placement to evaluate actual nearby freshwater
    /// rather than treating an entire coarse watershed cell as river.
    pub fn is_near(&self, position: RiverPosition, distance_q16: i64) -> bool {
        let max_distance = watershed::WATERSHED_CELL_SAMPLES * RIVER_POSITION_SCALE;
        let radius = self
            .half_width_q16
            .saturating_add(distance_q16.clamp(0, max_distance));
        self.centerline
            .windows(2)
            .any(|pair| point_within_segment(position, pair[0], pair[1], radius))
    }

    /// Resolve both banks where a route segment crosses this channel.
    ///
    /// The returned points are intersections with the two parallel bank
    /// boundaries, not a guessed coarse-cell centre. A tangent or partial
    /// overlap that touches only one bank is not reported as a completed
    /// crossing.
    pub fn intersect_route(
        &self,
        route_start: RiverPosition,
        route_end: RiverPosition,
    ) -> Option<RiverBankIntersection> {
        if route_start == route_end {
            return None;
        }

        let mut intersections = Vec::new();
        for pair in self.centerline.windows(2) {
            collect_bank_intersections(
                pair[0],
                pair[1],
                route_start,
                route_end,
                self.half_width_q16,
                &mut intersections,
            );
        }
        intersections.sort_by(|left, right| {
            compare_fraction(
                left.numerator,
                left.denominator,
                right.numerator,
                right.denominator,
            )
        });

        // Adjacent tessellation pieces can report the same bank at their shared
        // vertex. Collapse those duplicates before selecting the outer pair.
        intersections.dedup_by(|left, right| {
            if left.side != right.side {
                return false;
            }
            let dx = i128::from(left.position.x_q16) - i128::from(right.position.x_q16);
            let dy = i128::from(left.position.y_q16) - i128::from(right.position.y_q16);
            dx * dx + dy * dy <= 1_048_576
        });

        let first = intersections.first()?;
        let last = intersections
            .iter()
            .rev()
            .find(|item| item.side != first.side)?;
        if first.position == last.position {
            return None;
        }

        Some(RiverBankIntersection {
            entry: first.position,
            exit: last.position,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct RouteIntersection {
    numerator: i128,
    denominator: i128,
    position: RiverPosition,
    side: i8,
}

/// Cache neighbouring channel segments while projecting a location or
/// classifying a route. Each lookup examines at most nine coarse source cells.
#[derive(Debug, Default)]
pub struct RiverGeometryCache {
    nearby_by_cell: HashMap<MacroCoordinate, Vec<RiverChannelSegment>>,
    segment_by_source: HashMap<MacroCoordinate, Option<RiverChannelSegment>>,
}

impl RiverGeometryCache {
    pub fn segments_near(
        &mut self,
        world_seed: u64,
        ruleset_version: u32,
        position: RiverPosition,
    ) -> &[RiverChannelSegment] {
        let coarse_x = position
            .x_q16
            .div_euclid(RIVER_POSITION_SCALE)
            .div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let coarse_y = position
            .y_q16
            .div_euclid(RIVER_POSITION_SCALE)
            .div_euclid(watershed::WATERSHED_CELL_SAMPLES);
        let Some(query_cell) = watershed::cell_center(coarse_x, coarse_y) else {
            return &[];
        };

        if !self.nearby_by_cell.contains_key(&query_cell) {
            if self.nearby_by_cell.len() >= MAX_CACHED_QUERY_CELLS {
                return &[];
            }

            let mut segments = Vec::new();
            for offset_y in -1_i64..=1 {
                for offset_x in -1_i64..=1 {
                    let Some(source_x) = coarse_x.checked_add(offset_x) else {
                        continue;
                    };
                    let Some(source_y) = coarse_y.checked_add(offset_y) else {
                        continue;
                    };
                    let Some(source) = watershed::cell_center(source_x, source_y) else {
                        continue;
                    };
                    let segment = if let Some(cached) = self.segment_by_source.get(&source) {
                        cached.clone()
                    } else if self.segment_by_source.len() < MAX_CACHED_SOURCE_CELLS {
                        let generated = channel_segment(world_seed, ruleset_version, source);
                        self.segment_by_source.insert(source, generated.clone());
                        generated
                    } else {
                        None
                    };
                    if let Some(segment) = segment {
                        segments.push(segment);
                        if segments.len() == MAX_NEARBY_RIVER_SEGMENTS {
                            break;
                        }
                    }
                }
                if segments.len() == MAX_NEARBY_RIVER_SEGMENTS {
                    break;
                }
            }

            self.nearby_by_cell.insert(query_cell, segments);
        }

        self.nearby_by_cell
            .get(&query_cell)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn channel_segment(
    world_seed: u64,
    ruleset_version: u32,
    source: MacroCoordinate,
) -> Option<RiverChannelSegment> {
    if !terrain::sample_surface(world_seed, ruleset_version, source)?.is_land()
        || waterbodies::sample_surface(world_seed, ruleset_version, source).is_some()
    {
        return None;
    }

    let river = rivers::sample_surface(world_seed, ruleset_version, source)?;
    if !river.is_channel() {
        return None;
    }

    let drainage = watershed::sample_surface(world_seed, ruleset_version, source)?;
    let downstream = drainage.coarse_downstream()?;
    let river_id = river.network_id(world_seed, ruleset_version)?;
    let flow_direction = river.flow_direction(world_seed, ruleset_version)?;
    let width = river.width_class();
    let centerline = curved_centerline(world_seed, ruleset_version, source, downstream)?;

    Some(RiverChannelSegment {
        river_id,
        width,
        flow_direction,
        half_width_q16: half_width_q16(width),
        centerline,
    })
}

/// Generate a shared-endpoint quadratic curve using only stable coordinate
/// inputs. Its bend is bounded to a small fraction of one coarse watershed cell.
fn curved_centerline(
    world_seed: u64,
    ruleset_version: u32,
    source: MacroCoordinate,
    downstream: MacroCoordinate,
) -> Option<Vec<RiverPosition>> {
    let start = RiverPosition::from_macro_coordinate(source)?;
    let end = RiverPosition::from_macro_coordinate(downstream)?;
    let dx = downstream.x.checked_sub(source.x)?;
    let dy = downstream.y.checked_sub(source.y)?;
    let length_squared = i128::from(dx) * i128::from(dx) + i128::from(dy) * i128::from(dy);
    let length = i64::try_from(integer_sqrt(u128::try_from(length_squared).ok()?)).ok()?;
    if length == 0 {
        return None;
    }

    let hash = super::coordinate_seed(world_seed, ruleset_version, source);
    let base_offset = watershed::WATERSHED_CELL_SAMPLES / 12;
    let variation = (hash % u64::try_from(watershed::WATERSHED_CELL_SAMPLES / 24).ok()?) as i64;
    let bend_samples = base_offset + variation;
    let bend_scale = i128::from(bend_samples) * i128::from(RIVER_POSITION_SCALE);
    let perpendicular_x = -i128::from(dy) * bend_scale / i128::from(length);
    let perpendicular_y = i128::from(dx) * bend_scale / i128::from(length);
    let midpoint_x = (i128::from(start.x_q16) + i128::from(end.x_q16)) / 2;
    let midpoint_y = (i128::from(start.y_q16) + i128::from(end.y_q16)) / 2;
    let positive_bend = RiverPosition {
        x_q16: i64::try_from(midpoint_x + perpendicular_x).ok()?,
        y_q16: i64::try_from(midpoint_y + perpendicular_y).ok()?,
    };
    let negative_bend = RiverPosition {
        x_q16: i64::try_from(midpoint_x - perpendicular_x).ok()?,
        y_q16: i64::try_from(midpoint_y - perpendicular_y).ok()?,
    };

    // Prefer the lower sampled terrain side for the bend. A coordinate hash
    // breaks ties, so flat terrain remains deterministic without arbitrary
    // generation-order state.
    let elevation_at = |position: RiverPosition| {
        let coordinate = position.rounded_macro_coordinate()?;
        terrain::sample_surface(world_seed, ruleset_version, coordinate)
            .map(|sample| sample.elevation_q16())
    };
    let positive_elevation = elevation_at(positive_bend);
    let negative_elevation = elevation_at(negative_bend);
    let bend_positive = match (positive_elevation, negative_elevation) {
        (Some(positive), Some(negative)) if positive != negative => positive < negative,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        _ => hash & 1 == 0,
    };
    let control = if bend_positive {
        positive_bend
    } else {
        negative_bend
    };

    let subdivisions = i128::from(CURVE_SUBDIVISIONS);
    let denominator = subdivisions * subdivisions;
    let mut points = Vec::with_capacity(CURVE_SUBDIVISIONS as usize + 1);
    for step in 0..=CURVE_SUBDIVISIONS {
        let t = i128::from(step);
        let remaining = subdivisions - t;
        let start_weight = remaining * remaining;
        let control_weight = 2 * remaining * t;
        let end_weight = t * t;
        let x = (i128::from(start.x_q16) * start_weight
            + i128::from(control.x_q16) * control_weight
            + i128::from(end.x_q16) * end_weight)
            / denominator;
        let y = (i128::from(start.y_q16) * start_weight
            + i128::from(control.y_q16) * control_weight
            + i128::from(end.y_q16) * end_weight)
            / denominator;
        points.push(RiverPosition {
            x_q16: i64::try_from(x).ok()?,
            y_q16: i64::try_from(y).ok()?,
        });
    }

    // Retain the exact coarse centres as endpoints; independently generated
    // neighbouring segments therefore meet without accumulated interpolation error.
    if let Some(first) = points.first_mut() {
        *first = start;
    }
    if let Some(last) = points.last_mut() {
        *last = end;
    }
    Some(points)
}

fn half_width_q16(width: RiverWidthClass) -> i64 {
    let quarter = RIVER_POSITION_SCALE / 4;
    match width {
        RiverWidthClass::Narrow => quarter,
        RiverWidthClass::Medium => quarter * 2,
        RiverWidthClass::Wide => quarter * 4,
        RiverWidthClass::VeryWide => quarter * 8,
    }
}

fn point_within_segment(
    point: RiverPosition,
    start: RiverPosition,
    end: RiverPosition,
    radius_q16: i64,
) -> bool {
    let vx = i128::from(end.x_q16) - i128::from(start.x_q16);
    let vy = i128::from(end.y_q16) - i128::from(start.y_q16);
    let wx = i128::from(point.x_q16) - i128::from(start.x_q16);
    let wy = i128::from(point.y_q16) - i128::from(start.y_q16);
    let length_squared = vx * vx + vy * vy;
    let radius_squared = i128::from(radius_q16) * i128::from(radius_q16);
    if length_squared == 0 {
        return wx * wx + wy * wy <= radius_squared;
    }

    let projection = wx * vx + wy * vy;
    if projection <= 0 {
        return wx * wx + wy * wy <= radius_squared;
    }
    if projection >= length_squared {
        let dx = i128::from(point.x_q16) - i128::from(end.x_q16);
        let dy = i128::from(point.y_q16) - i128::from(end.y_q16);
        return dx * dx + dy * dy <= radius_squared;
    }

    let cross = vx * wy - vy * wx;
    cross * cross <= radius_squared * length_squared
}

fn collect_bank_intersections(
    channel_start: RiverPosition,
    channel_end: RiverPosition,
    route_start: RiverPosition,
    route_end: RiverPosition,
    half_width_q16: i64,
    intersections: &mut Vec<RouteIntersection>,
) {
    let vx = i128::from(channel_end.x_q16) - i128::from(channel_start.x_q16);
    let vy = i128::from(channel_end.y_q16) - i128::from(channel_start.y_q16);
    let length_squared = vx * vx + vy * vy;
    if length_squared == 0 {
        return;
    }
    let length = integer_sqrt(length_squared as u128) as i128;
    let threshold = i128::from(half_width_q16) * length;
    let start_dx = i128::from(route_start.x_q16) - i128::from(channel_start.x_q16);
    let start_dy = i128::from(route_start.y_q16) - i128::from(channel_start.y_q16);
    let end_dx = i128::from(route_end.x_q16) - i128::from(channel_start.x_q16);
    let end_dy = i128::from(route_end.y_q16) - i128::from(channel_start.y_q16);
    let start_side = vx * start_dy - vy * start_dx;
    let end_side = vx * end_dy - vy * end_dx;
    let side_delta = end_side - start_side;
    if side_delta == 0 {
        return;
    }

    for (boundary, side) in [(-threshold, -1_i8), (threshold, 1_i8)] {
        let numerator = boundary - start_side;
        let denominator = side_delta;
        let (numerator, denominator) = if denominator < 0 {
            (-numerator, -denominator)
        } else {
            (numerator, denominator)
        };
        if numerator < 0 || numerator > denominator {
            continue;
        }

        let route_dx = i128::from(route_end.x_q16) - i128::from(route_start.x_q16);
        let route_dy = i128::from(route_end.y_q16) - i128::from(route_start.y_q16);
        let point_x = i128::from(route_start.x_q16) + route_dx * numerator / denominator;
        let point_y = i128::from(route_start.y_q16) + route_dy * numerator / denominator;
        let along = (point_x - i128::from(channel_start.x_q16)) * vx
            + (point_y - i128::from(channel_start.y_q16)) * vy;
        if along < 0 || along > length_squared {
            continue;
        }

        let Ok(x_q16) = i64::try_from(point_x) else {
            continue;
        };
        let Ok(y_q16) = i64::try_from(point_y) else {
            continue;
        };
        intersections.push(RouteIntersection {
            numerator,
            denominator,
            position: RiverPosition { x_q16, y_q16 },
            side,
        });
    }
}

fn compare_fraction(a_num: i128, a_den: i128, b_num: i128, b_den: i128) -> std::cmp::Ordering {
    (a_num * b_den).cmp(&(b_num * a_den))
}

fn round_q16(value: i64) -> Option<i64> {
    let half = RIVER_POSITION_SCALE / 2;
    let rounded = if value >= 0 {
        i128::from(value) + i128::from(half)
    } else {
        i128::from(value) - i128::from(half)
    };
    i64::try_from(rounded / i128::from(RIVER_POSITION_SCALE)).ok()
}

fn integer_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let mut estimate = value;
    let mut next = estimate / 2 + 1;
    while next < estimate {
        estimate = next;
        next = (estimate + value / estimate) / 2;
    }
    estimate
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_channel() -> RiverChannelSegment {
        let point = |x: i64, y: i64| RiverPosition {
            x_q16: x * RIVER_POSITION_SCALE,
            y_q16: y * RIVER_POSITION_SCALE,
        };
        RiverChannelSegment {
            river_id: RiverId {
                outlet_coordinate: MacroCoordinate::new(0, 0),
                outlet_kind: rivers::RiverOutletKind::Ocean,
            },
            width: RiverWidthClass::Narrow,
            flow_direction: RiverFlowDirection::East,
            half_width_q16: RIVER_POSITION_SCALE,
            centerline: vec![point(0, 0), point(10, 0)],
        }
    }

    #[test]
    fn channel_footprint_is_a_bounded_strip_not_the_whole_watershed_cell() {
        let channel = straight_channel();
        let inside = RiverPosition {
            x_q16: 5 * RIVER_POSITION_SCALE,
            y_q16: RIVER_POSITION_SCALE,
        };
        let outside = RiverPosition {
            x_q16: 5 * RIVER_POSITION_SCALE,
            y_q16: RIVER_POSITION_SCALE * 2,
        };
        assert!(channel.contains_position(inside));
        assert!(!channel.contains_position(outside));
        assert!(channel.is_near(outside, RIVER_POSITION_SCALE));
        assert!(!channel.is_near(outside, RIVER_POSITION_SCALE / 2));
    }

    #[test]
    fn route_intersection_returns_both_sub_sample_bank_boundaries() {
        let channel = straight_channel();
        let start = RiverPosition {
            x_q16: 5 * RIVER_POSITION_SCALE,
            y_q16: -4 * RIVER_POSITION_SCALE,
        };
        let end = RiverPosition {
            x_q16: 5 * RIVER_POSITION_SCALE,
            y_q16: 4 * RIVER_POSITION_SCALE,
        };
        let banks = channel.intersect_route(start, end).unwrap();

        assert_eq!(
            banks.entry,
            RiverPosition {
                x_q16: 5 * RIVER_POSITION_SCALE,
                y_q16: -RIVER_POSITION_SCALE,
            }
        );
        assert_eq!(
            banks.exit,
            RiverPosition {
                x_q16: 5 * RIVER_POSITION_SCALE,
                y_q16: RIVER_POSITION_SCALE,
            }
        );
    }

    #[test]
    fn deterministic_curves_keep_exact_shared_macro_cell_endpoints() {
        let source = watershed::cell_center(0, 0).unwrap();
        let middle = watershed::cell_center(1, 0).unwrap();
        let destination = watershed::cell_center(2, 0).unwrap();
        let first = curved_centerline(991, 2, source, middle).unwrap();
        let first_again = curved_centerline(991, 2, source, middle).unwrap();
        let second = curved_centerline(991, 2, middle, destination).unwrap();

        assert_eq!(first, first_again);
        assert_eq!(
            first.first(),
            RiverPosition::from_macro_coordinate(source).as_ref()
        );
        assert_eq!(
            first.last(),
            RiverPosition::from_macro_coordinate(middle).as_ref()
        );
        assert_eq!(first.last(), second.first());
    }
}

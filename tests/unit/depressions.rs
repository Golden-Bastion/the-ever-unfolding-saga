use super::*;
use crate::geography::{RegionCoordinate, GEOGRAPHY_RULESET_VERSION};

fn find_negative_closed_sink(seed: u64) -> MacroCoordinate {
    for coarse_y in (-40..=-1).rev() {
        for coarse_x in (-40..=-1).rev() {
            let coordinate = watershed::cell_center(coarse_x, coarse_y).unwrap();
            let terrain_sample =
                terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
            if terrain_sample.is_land()
                && watershed::downstream_cell(
                    seed,
                    GEOGRAPHY_RULESET_VERSION,
                    coarse_x,
                    coarse_y,
                    terrain_sample.elevation_q16(),
                )
                .is_none()
            {
                return coordinate;
            }
        }
    }

    panic!("expected a closed inland sink in the deterministic search area");
}

#[test]
fn depression_sampling_resolves_closed_sinks_deterministically() {
    let seed = 8_211;
    let sink = find_negative_closed_sink(seed);
    let nearby = MacroCoordinate::new(sink.x + 1, sink.y + 1);

    let first = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, nearby).unwrap();
    let repeated = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, sink).unwrap();

    assert_eq!(first, repeated);
    assert_eq!(first.outlet_cell(), sink);
    assert!(first.spill_elevation_q16() > first.floor_elevation_q16());
    assert_eq!(
        first.depth_q16(),
        first.spill_elevation_q16() - first.floor_elevation_q16()
    );
    assert!(first.basin_cell_count() > 0);
    assert!(first.basin_cell_count() <= WINDOW_CELL_COUNT as u16);
}

#[test]
fn depression_sampling_rejects_underground_strata() {
    assert!(sample_surface(
        8_211,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
    )
    .is_none());
}

#[test]
fn depression_sampling_is_independent_of_region_chunk_ownership() {
    let seed = 8_211;
    let sink = find_negative_closed_sink(seed);
    let (region, local_x, local_y) = sink.region_and_local();
    let reconstructed = RegionCoordinate::new(region.x, region.y)
        .global_sample(local_x, local_y)
        .unwrap();

    assert_eq!(sink, reconstructed);
    assert_eq!(
        sample_surface(seed, GEOGRAPHY_RULESET_VERSION, sink),
        sample_surface(seed, GEOGRAPHY_RULESET_VERSION, reconstructed),
    );
}

#[test]
fn minimax_spill_uses_the_lowest_escape_saddle() {
    let mut elevations = [12_000_i32; WINDOW_CELL_COUNT];

    for y in 7..=9 {
        for x in 7..=9 {
            elevations[window_index(x, y)] = 1_000;
        }
    }
    elevations[CENTER_INDEX] = 200;

    // A lower saddle corridor exits through the north edge.
    for y in 0..=6 {
        elevations[window_index(8, y)] = 5_000;
    }

    assert_eq!(minimax_spill_elevation(&elevations), Some(5_000));
    assert_eq!(count_basin_cells(&elevations, 5_000), 9);
}

#[test]
fn basin_footprint_excludes_disconnected_low_terrain() {
    let mut elevations = [12_000_i32; WINDOW_CELL_COUNT];

    for y in 7..=9 {
        for x in 7..=9 {
            elevations[window_index(x, y)] = 1_000;
        }
    }
    elevations[CENTER_INDEX] = 200;
    elevations[window_index(2, 2)] = 100;

    let spill = minimax_spill_elevation(&elevations).unwrap();
    assert_eq!(spill, 12_000);
    assert_eq!(count_basin_cells(&elevations, spill), 9);
}

#[test]
fn a_flat_window_has_no_depression_depth_or_basin_footprint() {
    let elevations = [4_000_i32; WINDOW_CELL_COUNT];

    let spill = minimax_spill_elevation(&elevations).unwrap();
    assert_eq!(spill, elevations[CENTER_INDEX]);
    assert_eq!(count_basin_cells(&elevations, spill), 0);
}

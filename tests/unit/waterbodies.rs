use super::*;
use crate::geography::{RegionCoordinate, GEOGRAPHY_RULESET_VERSION};

fn find_lake_cell(seed: u64) -> Option<MacroCoordinate> {
    for coarse_y in (-40..=-1).rev() {
        for coarse_x in (-40..=-1).rev() {
            let coordinate = watershed::cell_center(coarse_x, coarse_y)?;
            let elevation = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)?;
            if !elevation.is_land()
                || watershed::downstream_cell(seed, GEOGRAPHY_RULESET_VERSION, coarse_x, coarse_y, elevation.elevation_q16()).is_some()
            {
                continue;
            }
            if sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).is_some() {
                return Some(coordinate);
            }
        }
    }
    None
}

#[test]
fn lake_sampling_is_deterministic_and_uses_stable_basin_identity() {
    let (seed, coordinate) = (1..=24)
        .find_map(|seed| find_lake_cell(seed).map(|coordinate| (seed, coordinate)))
        .expect("expected at least one wet closed basin in the deterministic search area");

    let first = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    assert_eq!(first, repeated);
    assert!(first.depth_q16() >= 0);
    assert!(first.basin_cell_count() >= MIN_LAKE_BASIN_CELLS);
    assert!(first.precipitation_potential_q16() > 0);

    let nearby = MacroCoordinate::new(
        coordinate.x + 1,
        coordinate.y + 1,
    );
    if let Some(adjacent) = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, nearby) {
        assert_eq!(first.id(seed, GEOGRAPHY_RULESET_VERSION), adjacent.id(seed, GEOGRAPHY_RULESET_VERSION));
    }
}

#[test]
fn lake_sampling_rejects_underground_strata() {
    assert!(sample_surface(
        8_211,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
    )
    .is_none());
}

#[test]
fn lake_identity_is_independent_of_region_chunk_ownership() {
    let (seed, coordinate) = (1..=24)
        .find_map(|seed| find_lake_cell(seed).map(|coordinate| (seed, coordinate)))
        .expect("expected a lake in the deterministic search area");
    let sample = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let (region, local_x, local_y) = coordinate.region_and_local();
    let reconstructed = RegionCoordinate::new(region.x, region.y)
        .global_sample(local_x, local_y)
        .unwrap();

    assert_eq!(
        sample.id(seed, GEOGRAPHY_RULESET_VERSION),
        sample_surface(seed, GEOGRAPHY_RULESET_VERSION, reconstructed)
            .unwrap()
            .id(seed, GEOGRAPHY_RULESET_VERSION)
    );
}

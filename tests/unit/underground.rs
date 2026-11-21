use super::*;
use crate::geography::{
    MacroCoordinate, RegionCoordinate, GEOGRAPHY_RULESET_VERSION,
};

fn chamber_center(cell_x: i64, cell_y: i64, stratum: i32) -> MacroCoordinate {
    MacroCoordinate::in_stratum(
        cell_x * NETWORK_CELL_SAMPLES + NETWORK_CELL_SAMPLES / 2,
        cell_y * NETWORK_CELL_SAMPLES + NETWORK_CELL_SAMPLES / 2,
        stratum,
    )
}

#[test]
fn underground_sampling_rejects_surface_and_positive_strata() {
    for stratum in [0, 1, i32::MAX] {
        assert!(sample_underground(
            42,
            GEOGRAPHY_RULESET_VERSION,
            MacroCoordinate::in_stratum(0, 0, stratum),
        )
        .is_none());
    }
}

#[test]
fn cave_nodes_are_deterministic_and_seeded_by_stratum() {
    let coordinates = [
        MacroCoordinate::in_stratum(-640, -384, -1),
        MacroCoordinate::in_stratum(-32, 32, -1),
        MacroCoordinate::in_stratum(32, 32, -1),
        MacroCoordinate::in_stratum(288, 160, -2),
        MacroCoordinate::in_stratum(1_024, -768, -3),
    ];
    let first = coordinates
        .iter()
        .map(|coordinate| sample_underground(42, GEOGRAPHY_RULESET_VERSION, *coordinate))
        .collect::<Vec<_>>();
    let repeated = coordinates
        .iter()
        .map(|coordinate| sample_underground(42, GEOGRAPHY_RULESET_VERSION, *coordinate))
        .collect::<Vec<_>>();
    assert_eq!(first, repeated);

    let other_seed = coordinates
        .iter()
        .map(|coordinate| sample_underground(43, GEOGRAPHY_RULESET_VERSION, *coordinate))
        .collect::<Vec<_>>();
    assert_ne!(first, other_seed);

    let shallow = coordinates
        .iter()
        .map(|coordinate| {
            sample_underground(
                42,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(coordinate.x, coordinate.y, -1),
            )
        })
        .collect::<Vec<_>>();
    let deep = coordinates
        .iter()
        .map(|coordinate| {
            sample_underground(
                42,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(coordinate.x, coordinate.y, -2),
            )
        })
        .collect::<Vec<_>>();
    assert_ne!(shallow, deep);
}

#[test]
fn underground_field_contains_both_solid_rock_and_cave_space() {
    let mut chambers = 0;
    let mut solid = 0;
    for cell_y in -24..24 {
        for cell_x in -24..24 {
            let sample = sample_underground(
                7_321,
                GEOGRAPHY_RULESET_VERSION,
                chamber_center(cell_x, cell_y, -1),
            )
            .unwrap();
            match sample.kind() {
                UndergroundKind::Chamber => chambers += 1,
                UndergroundKind::SolidRock => solid += 1,
                UndergroundKind::Passage => panic!("cell center should be a chamber or solid rock"),
            }
        }
    }
    assert!(chambers > 0, "expected generated underground chambers");
    assert!(solid > 0, "solid rock must remain between cave chambers");
}

#[test]
fn connected_chambers_share_reciprocal_passages() {
    let mut found_connection = false;
    'search: for seed in 1..=32 {
        for cell_y in -12..12 {
            for cell_x in -12..12 {
                let center = chamber_center(cell_x, cell_y, -1);
                let Some(first) = sample_underground(seed, GEOGRAPHY_RULESET_VERSION, center)
                else {
                    continue;
                };
                if first.kind() != UndergroundKind::Chamber
                    || first.connections() & direction::EAST == 0
                {
                    continue;
                }

                let neighbor = chamber_center(cell_x + 1, cell_y, -1);
                let second =
                    sample_underground(seed, GEOGRAPHY_RULESET_VERSION, neighbor).unwrap();
                assert_eq!(second.kind(), UndergroundKind::Chamber);
                assert_ne!(second.connections() & direction::WEST, 0);
                assert_eq!(
                    first.network_cell(),
                    Some(center),
                    "chamber identity must be its stable global center"
                );
                assert_eq!(second.network_cell(), Some(neighbor));

                let passage = MacroCoordinate::in_stratum(
                    center.x + NETWORK_CELL_SAMPLES / 2,
                    center.y,
                    -1,
                );
                let crossing =
                    sample_underground(seed, GEOGRAPHY_RULESET_VERSION, passage).unwrap();
                assert_eq!(crossing.kind(), UndergroundKind::Passage);
                assert!(crossing.is_open());
                assert_eq!(crossing.connections() & direction::EAST, 0);
                found_connection = true;
                break 'search;
            }
        }
    }
    assert!(found_connection, "expected a connected cave-network edge");
}

#[test]
fn cave_passages_remain_continuous_across_region_boundaries() {
    let mut found_crossing = false;
    for seed in 1..=32 {
        for cell_y in -16..16 {
            let y = cell_y * NETWORK_CELL_SAMPLES + NETWORK_CELL_SAMPLES / 2;
            let west = sample_underground(
                seed,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(255, y, -2),
            )
            .unwrap();
            let east = sample_underground(
                seed,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(256, y, -2),
            )
            .unwrap();
            if west.kind() != UndergroundKind::Passage {
                continue;
            }
            assert_eq!(east.kind(), UndergroundKind::Passage);
            assert_ne!(west.connections() & direction::EAST, 0);
            assert_ne!(east.connections() & direction::WEST, 0);
            found_crossing = true;
            break;
        }
        if found_crossing {
            break;
        }
    }
    assert!(
        found_crossing,
        "expected a connected cave passage crossing a region boundary"
    );
}

#[test]
fn cave_sampling_is_stable_across_region_ownership_and_negative_coordinates() {
    let coordinate = MacroCoordinate::in_stratum(-1, -65, -3);
    let (region, local_x, local_y) = coordinate.region_and_local();
    let reconstructed = RegionCoordinate::in_stratum(region.x, region.y, region.stratum)
        .global_sample(local_x, local_y)
        .unwrap();

    assert_eq!(coordinate, reconstructed);
    assert_eq!(
        sample_underground(812, GEOGRAPHY_RULESET_VERSION, coordinate),
        sample_underground(812, GEOGRAPHY_RULESET_VERSION, reconstructed)
    );
}

#[test]
fn underground_sampling_handles_the_deepest_stratum_without_overflow() {
    assert!(sample_underground(
        42,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(i64::MAX, i64::MIN, i32::MIN),
    )
    .is_some());
}

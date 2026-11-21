use super::*;

#[test]
fn region_local_samples_map_to_global_coordinates() {
    let region = RegionCoordinate::new(2, -3);

    assert_eq!(
        region.global_sample(0, 0),
        Some(MacroCoordinate::new(512, -768))
    );
    assert_eq!(
        region.global_sample(255, 255),
        Some(MacroCoordinate::new(767, -513))
    );
    assert_eq!(region.global_sample(256, 0), None);
    assert_eq!(region.global_sample(0, 256), None);
}

#[test]
fn global_coordinates_partition_correctly_on_positive_and_negative_axes() {
    let cases: [(MacroCoordinate, RegionCoordinate, u16, u16); 6] = [
        (MacroCoordinate::new(0, 0), RegionCoordinate::ORIGIN, 0, 0),
        (
            MacroCoordinate::new(255, 255),
            RegionCoordinate::ORIGIN,
            255,
            255,
        ),
        (
            MacroCoordinate::new(256, 0),
            RegionCoordinate::new(1, 0),
            0,
            0,
        ),
        (
            MacroCoordinate::new(-1, -1),
            RegionCoordinate::new(-1, -1),
            255,
            255,
        ),
        (
            MacroCoordinate::new(-256, -257),
            RegionCoordinate::new(-1, -2),
            0,
            255,
        ),
        (
            MacroCoordinate::new(-257, 256),
            RegionCoordinate::new(-2, 1),
            255,
            0,
        ),
    ];

    for (global, expected_region, expected_x, expected_y) in cases {
        assert_eq!(
            global.region_and_local(),
            (expected_region, expected_x, expected_y)
        );
    }
}

#[test]
fn adjacent_region_chunks_share_one_global_coordinate_lattice() {
    let west = RegionCoordinate::new(0, 0);
    let east = RegionCoordinate::new(1, 0);

    assert_eq!(
        west.global_sample(255, 73).unwrap().x + 1,
        east.global_sample(0, 73).unwrap().x
    );
    assert_eq!(
        west.global_sample(255, 73).unwrap().y,
        east.global_sample(0, 73).unwrap().y
    );

    let negative = RegionCoordinate::new(-1, 0);
    assert_eq!(negative.global_sample(255, 73).unwrap().x + 1, 0);
}

#[test]
fn global_sample_conversion_rejects_coordinate_overflow() {
    let region = RegionCoordinate::new(i64::MAX, i64::MIN);
    assert_eq!(region.global_sample(0, 0), None);
}

#[test]
fn coordinate_seed_is_stable_and_independent_of_sampling_order() {
    let coordinates = [
        MacroCoordinate::new(-256, -1),
        MacroCoordinate::new(0, 0),
        MacroCoordinate::new(12, 98),
        MacroCoordinate::new(256, 255),
    ];
    let forward = coordinates
        .iter()
        .map(|coordinate| {
            (
                *coordinate,
                coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, *coordinate),
            )
        })
        .collect::<Vec<_>>();
    let reverse = coordinates
        .iter()
        .rev()
        .map(|coordinate| {
            (
                *coordinate,
                coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, *coordinate),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn coordinate_seed_changes_with_world_ruleset_or_global_position() {
    let coordinate = MacroCoordinate::new(-7, 19);
    let baseline = coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, coordinate);

    assert_ne!(
        baseline,
        coordinate_seed(43, GEOGRAPHY_RULESET_VERSION, coordinate)
    );
    assert_ne!(
        baseline,
        coordinate_seed(42, GEOGRAPHY_RULESET_VERSION + 1, coordinate)
    );
    assert_ne!(
        baseline,
        coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, MacroCoordinate::new(-6, 19))
    );
}

#[test]
fn underground_strata_keep_horizontal_coordinates_and_change_sample_keys() {
    let cavern = RegionCoordinate::in_stratum(3, -2, -4);
    let sample = cavern
        .global_sample(17, 92)
        .expect("cavern sample should resolve");

    assert_eq!(sample, MacroCoordinate::in_stratum(785, -420, -4));
    assert_eq!(sample.region_and_local(), (cavern, 17, 92));

    let surface = MacroCoordinate::new(785, -420);
    assert_ne!(
        coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, sample),
        coordinate_seed(42, GEOGRAPHY_RULESET_VERSION, surface)
    );
}

#[test]
fn surface_elevation_is_deterministic_for_seed_ruleset_and_coordinate() {
    let coordinate = MacroCoordinate::new(-1_337, 2_048);
    let first = terrain::sample_surface(42, GEOGRAPHY_RULESET_VERSION, coordinate)
        .expect("surface coordinates should be sampled");
    let repeated = terrain::sample_surface(42, GEOGRAPHY_RULESET_VERSION, coordinate)
        .expect("surface coordinates should be sampled");

    assert_eq!(first, repeated);
    assert_eq!(first.elevation_q16(), repeated.elevation_q16());
    assert!((-65_536..=65_536).contains(&first.elevation_q16()));
}

#[test]
fn surface_elevation_changes_with_world_seed_and_ruleset() {
    let coordinate = MacroCoordinate::new(1_024, -2_048);
    let baseline = terrain::sample_surface(42, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();

    assert_ne!(
        baseline,
        terrain::sample_surface(43, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap()
    );
    assert_ne!(
        baseline,
        terrain::sample_surface(42, GEOGRAPHY_RULESET_VERSION + 1, coordinate).unwrap()
    );
}

#[test]
fn surface_sampler_rejects_underground_strata() {
    assert!(terrain::sample_surface(
        42,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1)
    )
    .is_none());
}

#[test]
fn continental_field_contains_both_land_and_ocean_samples() {
    let mut land_samples = 0usize;
    let mut ocean_samples = 0usize;

    for y in (-32_768..32_768).step_by(1_024) {
        for x in (-32_768..32_768).step_by(1_024) {
            let sample = terrain::sample_surface(
                7_321,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::new(x, y),
            )
            .unwrap();
            if sample.is_land() {
                land_samples += 1;
            } else {
                ocean_samples += 1;
            }
        }
    }

    assert!(land_samples > 0, "the generated field should contain land");
    assert!(
        ocean_samples > 0,
        "the generated field should contain ocean"
    );
    assert!(land_samples > 1_024, "land should occupy a meaningful area");
    assert!(
        ocean_samples > 1_024,
        "ocean should occupy a meaningful area"
    );
}

#[test]
fn elevation_remains_smooth_across_positive_and_negative_region_boundaries() {
    let east_edge = RegionCoordinate::new(0, 0)
        .global_sample(255, 73)
        .expect("east-edge sample should resolve");
    let next_region = RegionCoordinate::new(1, 0)
        .global_sample(0, 73)
        .expect("next-region sample should resolve");
    let west_edge = RegionCoordinate::new(-1, 0)
        .global_sample(255, 73)
        .expect("west-edge sample should resolve");
    let origin_edge = RegionCoordinate::new(0, 0)
        .global_sample(0, 73)
        .expect("origin-edge sample should resolve");

    let east_value = terrain::sample_surface(99, GEOGRAPHY_RULESET_VERSION, east_edge)
        .unwrap()
        .elevation_q16();
    let next_value = terrain::sample_surface(99, GEOGRAPHY_RULESET_VERSION, next_region)
        .unwrap()
        .elevation_q16();
    let west_value = terrain::sample_surface(99, GEOGRAPHY_RULESET_VERSION, west_edge)
        .unwrap()
        .elevation_q16();
    let origin_value = terrain::sample_surface(99, GEOGRAPHY_RULESET_VERSION, origin_edge)
        .unwrap()
        .elevation_q16();

    assert_eq!(east_edge.x + 1, next_region.x);
    assert_eq!(west_edge.x + 1, origin_edge.x);
    assert!((east_value - next_value).abs() < 4_096);
    assert!((west_value - origin_value).abs() < 4_096);
}

#[test]
fn terrain_samples_do_not_depend_on_region_sampling_order() {
    let coordinates = [
        MacroCoordinate::new(-256, -256),
        MacroCoordinate::new(0, 0),
        MacroCoordinate::new(256, 256),
        MacroCoordinate::new(4_096, -2_048),
    ];
    let forward = coordinates
        .iter()
        .map(|coordinate| {
            terrain::sample_surface(123, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();
    let reverse = coordinates
        .iter()
        .rev()
        .map(|coordinate| {
            terrain::sample_surface(123, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());
}
#[test]
fn surface_climate_is_deterministic_and_varies_across_coordinates() {
    let coordinate = MacroCoordinate::new(-1_337, 2_048);
    let first = climate::sample_surface(42, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = climate::sample_surface(42, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    assert_eq!(first, repeated);
    assert!((-65_536..=65_536).contains(&first.temperature_q16()));
    assert!(first.moisture_q16() <= 65_536);
    assert!(first.precipitation_q16() <= 65_536);

    let distant = climate::sample_surface(
        42,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::new(8_192, -6_144),
    )
    .unwrap();
    assert_ne!(first, distant);

    assert_ne!(
        first,
        climate::sample_surface(43, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap()
    );
    assert_ne!(
        first,
        climate::sample_surface(42, GEOGRAPHY_RULESET_VERSION + 1, coordinate).unwrap()
    );
}

#[test]
fn climate_sampler_rejects_underground_strata() {
    assert!(climate::sample_surface(
        42,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -2),
    )
    .is_none());
}

#[test]
fn climate_sampling_is_independent_of_region_order_and_negative_coordinates() {
    let coordinates = [
        MacroCoordinate::new(-257, -1),
        MacroCoordinate::new(-256, 255),
        MacroCoordinate::new(255, 256),
        MacroCoordinate::new(256, -257),
    ];
    let forward = coordinates
        .iter()
        .map(|coordinate| {
            climate::sample_surface(731, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();
    let reverse = coordinates
        .iter()
        .rev()
        .map(|coordinate| {
            climate::sample_surface(731, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn climate_biomes_have_multiple_distinct_surface_environments() {
    use std::collections::HashSet;

    let mut biomes = HashSet::new();
    for y in (-16_384..16_384).step_by(1_024) {
        for x in (-16_384..16_384).step_by(1_024) {
            let sample = climate::sample_surface(
                7_321,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::new(x, y),
            )
            .unwrap();
            biomes.insert(sample.biome());
        }
    }

    assert!(
        biomes.len() >= 4,
        "expected varied environments, got {biomes:?}"
    );
}

#[test]
fn coastal_biome_classification_remains_continuous_across_region_edges() {
    // Search several chunk boundaries on both axes. A coastline is not
    // guaranteed to intersect any one arbitrarily selected region edge.
    for boundary in (-4_096..=4_096).step_by(256) {
        for offset in (-8_192..8_192).step_by(16) {
            let west = MacroCoordinate::new(boundary - 1, offset);
            let east = MacroCoordinate::new(boundary, offset);
            let north = MacroCoordinate::new(offset, boundary - 1);
            let south = MacroCoordinate::new(offset, boundary);

            for (first, second) in [(west, east), (north, south)] {
                let first_terrain =
                    terrain::sample_surface(55, GEOGRAPHY_RULESET_VERSION, first).unwrap();
                let second_terrain =
                    terrain::sample_surface(55, GEOGRAPHY_RULESET_VERSION, second).unwrap();

                if first_terrain.is_land() != second_terrain.is_land() {
                    let land_coordinate = if first_terrain.is_land() {
                        first
                    } else {
                        second
                    };
                    let sample =
                        climate::sample_surface(55, GEOGRAPHY_RULESET_VERSION, land_coordinate)
                            .unwrap();

                    assert_eq!(
                        sample.biome(),
                        climate::Biome::Coast,
                        "land immediately beside ocean across a region edge must be coastal: {land_coordinate:?}"
                    );
                    return;
                }
            }
        }
    }

    panic!("expected the generated coastline to cross at least one searched region edge");
}

#[test]
fn shoreline_sampling_is_deterministic_and_rejects_underground_strata() {
    let coordinate = MacroCoordinate::new(-513, 1_025);
    let first = coast::sample_surface(82, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = coast::sample_surface(82, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();

    assert_eq!(first, repeated);
    assert_eq!(first.is_shoreline(), first.opposite_terrain_mask() != 0);
    assert!(coast::sample_surface(
        82,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(coordinate.x, coordinate.y, -1),
    )
    .is_none());
}

#[test]
fn shoreline_masks_identify_opposite_terrain_in_each_direction() {
    let mut found_boundary = None;
    for y in [-4_096, -2_048, -1_024, -256, 0, 256, 1_024, 2_048, 4_096] {
        for x in (-8_192..8_192).step_by(8) {
            let first = MacroCoordinate::new(x, y);
            let second = MacroCoordinate::new(x + 1, y);
            let first_terrain =
                terrain::sample_surface(1_903, GEOGRAPHY_RULESET_VERSION, first).unwrap();
            let second_terrain =
                terrain::sample_surface(1_903, GEOGRAPHY_RULESET_VERSION, second).unwrap();

            if first_terrain.is_land() != second_terrain.is_land() {
                found_boundary = Some((first, second));
                break;
            }
        }
        if found_boundary.is_some() {
            break;
        }
    }

    let (first, second) = found_boundary.expect("expected an adjacent land/ocean pair");
    let first_shore = coast::sample_surface(1_903, GEOGRAPHY_RULESET_VERSION, first).unwrap();
    let second_shore = coast::sample_surface(1_903, GEOGRAPHY_RULESET_VERSION, second).unwrap();

    assert!(first_shore.is_shoreline());
    assert!(second_shore.is_shoreline());
    assert_ne!(first_shore.is_coastal_land(), second_shore.is_coastal_land());
    assert_ne!(first_shore.is_coastal_water(), second_shore.is_coastal_water());
}

#[test]
fn shoreline_data_is_continuous_across_region_chunk_boundaries() {
    use coast::direction;

    for boundary in (-4_096..=4_096).step_by(256) {
        for offset in (-8_192..8_192).step_by(16) {
            let pairs = [
                (
                    MacroCoordinate::new(boundary - 1, offset),
                    MacroCoordinate::new(boundary, offset),
                    direction::EAST,
                    direction::WEST,
                ),
                (
                    MacroCoordinate::new(offset, boundary - 1),
                    MacroCoordinate::new(offset, boundary),
                    direction::SOUTH,
                    direction::NORTH,
                ),
            ];

            for (first, second, first_direction, second_direction) in pairs {
                let first_terrain =
                    terrain::sample_surface(55, GEOGRAPHY_RULESET_VERSION, first).unwrap();
                let second_terrain =
                    terrain::sample_surface(55, GEOGRAPHY_RULESET_VERSION, second).unwrap();

                if first_terrain.is_land() == second_terrain.is_land() {
                    continue;
                }

                let first_shore =
                    coast::sample_surface(55, GEOGRAPHY_RULESET_VERSION, first).unwrap();
                let second_shore =
                    coast::sample_surface(55, GEOGRAPHY_RULESET_VERSION, second).unwrap();

                assert!(first_shore.is_shoreline());
                assert!(second_shore.is_shoreline());
                assert!(first_shore.has_opposite_neighbor(first_direction));
                assert!(second_shore.has_opposite_neighbor(second_direction));
                return;
            }
        }
    }

    panic!("expected a coastline to cross one of the searched region boundaries");
}

#[test]
fn shoreline_sampling_is_independent_of_coordinate_visit_order() {
    let coordinates = [
        MacroCoordinate::new(-257, -1),
        MacroCoordinate::new(-256, 255),
        MacroCoordinate::new(255, 256),
        MacroCoordinate::new(256, -257),
    ];
    let forward = coordinates
        .iter()
        .map(|coordinate| {
            coast::sample_surface(731, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();
    let reverse = coordinates
        .iter()
        .rev()
        .map(|coordinate| {
            coast::sample_surface(731, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());
}

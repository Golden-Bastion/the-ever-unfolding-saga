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
    assert_ne!(
        first_shore.is_coastal_land(),
        second_shore.is_coastal_land()
    );
    assert_ne!(
        first_shore.is_coastal_water(),
        second_shore.is_coastal_water()
    );
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

#[test]
fn watershed_sampling_is_deterministic_and_rejects_underground() {
    let coordinate = MacroCoordinate::new(-4_097, 8_193);
    let first = watershed::sample_surface(1_337, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = watershed::sample_surface(1_337, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();

    assert_eq!(first, repeated);
    assert_eq!(
        first
            .coarse_cell()
            .x
            .rem_euclid(watershed::WATERSHED_CELL_SAMPLES),
        watershed::WATERSHED_CELL_SAMPLES / 2
    );
    assert_eq!(
        first
            .coarse_cell()
            .y
            .rem_euclid(watershed::WATERSHED_CELL_SAMPLES),
        watershed::WATERSHED_CELL_SAMPLES / 2
    );
    assert!(watershed::sample_surface(
        1_337,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(coordinate.x, coordinate.y, -1),
    )
    .is_none());
    assert!(watershed::trace_to_outlet(
        1_337,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(coordinate.x, coordinate.y, -1),
    )
    .is_none());
}

#[test]
fn watershed_coarse_links_are_strictly_downhill_and_region_independent() {
    let coordinates = [
        MacroCoordinate::new(-4_097, -1),
        MacroCoordinate::new(-1, 4_095),
        MacroCoordinate::new(4_095, 4_096),
        MacroCoordinate::new(8_192, -8_193),
    ];
    let forward = coordinates
        .iter()
        .map(|coordinate| {
            watershed::sample_surface(8_201, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();
    let reverse = coordinates
        .iter()
        .rev()
        .map(|coordinate| {
            watershed::sample_surface(8_201, GEOGRAPHY_RULESET_VERSION, *coordinate).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());

    for sample in forward {
        if let Some(downstream) = sample.coarse_downstream() {
            let downstream_elevation =
                terrain::sample_surface(8_201, GEOGRAPHY_RULESET_VERSION, downstream)
                    .unwrap()
                    .elevation_q16();
            assert!(downstream_elevation < sample.elevation_q16());
        }
    }
}

#[test]
fn watershed_local_d8_directions_stay_inside_their_coarse_cell() {
    let mut found_direction = false;
    for y in (-8_192..8_192).step_by(257) {
        for x in (-8_192..8_192).step_by(257) {
            let coordinate = MacroCoordinate::new(x, y);
            let sample =
                watershed::sample_surface(4_771, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
            if let Some(direction) = sample.local_direction() {
                let (dx, dy) = direction.offset();
                let neighbor = MacroCoordinate::new(x + dx, y + dy);
                if neighbor.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES)
                    != coordinate.x.div_euclid(watershed::WATERSHED_CELL_SAMPLES)
                    || neighbor.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES)
                        != coordinate.y.div_euclid(watershed::WATERSHED_CELL_SAMPLES)
                {
                    panic!("local D8 direction crossed a coarse watershed-cell boundary");
                }
                found_direction = true;
                break;
            }
        }
        if found_direction {
            break;
        }
    }
    assert!(
        found_direction,
        "expected at least one local downhill direction"
    );
}

#[test]
fn watershed_outlet_traces_are_bounded_and_deterministic() {
    let coordinate = MacroCoordinate::new(-12_345, 6_789);
    let first = watershed::trace_to_outlet(912, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = watershed::trace_to_outlet(912, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    assert_eq!(first, repeated);
    assert!(usize::from(first.steps()) <= watershed::MAX_OUTLET_TRACE_STEPS);
    if first.kind() == watershed::OutletKind::SearchLimit {
        assert!(!first.is_terminal());
        assert_eq!(
            usize::from(first.steps()),
            watershed::MAX_OUTLET_TRACE_STEPS
        );
    } else {
        assert!(first.is_terminal());
    }
}

#[test]
fn watershed_outlet_kinds_match_their_terminal_cells() {
    let coordinate = MacroCoordinate::new(31_337, -18_721);
    let outlet = watershed::trace_to_outlet(5_119, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let sample =
        watershed::sample_surface(5_119, GEOGRAPHY_RULESET_VERSION, outlet.coordinate()).unwrap();

    match outlet.kind() {
        watershed::OutletKind::Ocean => assert!(sample.is_ocean()),
        watershed::OutletKind::ClosedBasin => {
            assert!(!sample.is_ocean());
            assert!(sample.coarse_downstream().is_none());
        }
        watershed::OutletKind::SearchLimit => assert!(!outlet.is_terminal()),
    }
}

#[test]
fn watershed_sampling_is_independent_of_region_chunk_ownership() {
    let coordinate = MacroCoordinate::new(-257, 511);
    let (region, local_x, local_y) = coordinate.region_and_local();
    let reconstructed = region.global_sample(local_x, local_y).unwrap();

    assert_eq!(reconstructed, coordinate);
    assert_eq!(
        watershed::sample_surface(1_701, GEOGRAPHY_RULESET_VERSION, coordinate),
        watershed::sample_surface(1_701, GEOGRAPHY_RULESET_VERSION, reconstructed),
    );
}

#[test]
fn mountain_ranges_are_deterministic_and_change_surface_elevation() {
    let mut found_range = false;
    let mut found_non_mountain_land = false;
    for y in (-32_768..32_768).step_by(512) {
        for x in (-32_768..32_768).step_by(512) {
            let coordinate = MacroCoordinate::new(x, y);
            let mountain =
                mountains::sample_surface(7_321, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
            let repeated =
                mountains::sample_surface(7_321, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
            assert_eq!(mountain, repeated);
            assert!(mountain.ridge_strength_q16() <= 65_536);
            assert!(mountain.uplift_q16() >= 0);

            let base =
                terrain::base_surface_elevation_q16(7_321, GEOGRAPHY_RULESET_VERSION, coordinate)
                    .unwrap();
            let terrain =
                terrain::sample_surface(7_321, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
            assert_eq!(
                terrain.mountain_ridge_strength_q16(),
                mountain.ridge_strength_q16()
            );
            assert_eq!(terrain.is_mountain_barrier(), mountain.is_mountain());

            if base > 0 && mountain.is_mountain() {
                found_range = true;
                assert!(mountain.uplift_q16() > 0);
                assert!(terrain.elevation_q16() > base);
            } else if base > 0 {
                found_non_mountain_land = true;
                assert!(terrain.elevation_q16() >= base);
            } else {
                assert_eq!(mountain.uplift_q16(), 0);
                assert_eq!(terrain.elevation_q16(), base);
            }
        }
    }
    assert!(
        found_range,
        "expected mountain ranges in the sampled continental field"
    );
    assert!(
        found_non_mountain_land,
        "mountain ranges must not cover all land"
    );
}

#[test]
fn mountain_ranges_preserve_legacy_ruleset_elevation() {
    let coordinate = MacroCoordinate::new(-1_337, 2_048);
    let base = terrain::base_surface_elevation_q16(42, 1, coordinate).unwrap();
    let sampled = terrain::sample_surface(42, 1, coordinate).unwrap();
    assert_eq!(sampled.elevation_q16(), base);
    assert_eq!(
        mountains::sample_surface(42, 1, coordinate)
            .unwrap()
            .uplift_q16(),
        0
    );
}

#[test]
fn mountain_field_is_continuous_across_region_chunk_boundaries() {
    let mut found_land_pair = false;
    let left_x = RegionCoordinate::new(-1, 0)
        .global_sample(255, 0)
        .unwrap()
        .x;
    let right_x = RegionCoordinate::new(0, 0).global_sample(0, 0).unwrap().x;
    for y in (-8_192..8_192).step_by(64) {
        let left = MacroCoordinate::new(left_x, y);
        let right = MacroCoordinate::new(right_x, y);
        let left_base =
            terrain::base_surface_elevation_q16(8_201, GEOGRAPHY_RULESET_VERSION, left).unwrap();
        let right_base =
            terrain::base_surface_elevation_q16(8_201, GEOGRAPHY_RULESET_VERSION, right).unwrap();
        if left_base <= 0 || right_base <= 0 {
            continue;
        }

        let first = mountains::sample_surface(8_201, GEOGRAPHY_RULESET_VERSION, left).unwrap();
        let second = mountains::sample_surface(8_201, GEOGRAPHY_RULESET_VERSION, right).unwrap();
        assert_eq!(left.x + 1, right.x);
        assert!(
            (first.ridge_strength_q16() as i32 - second.ridge_strength_q16() as i32).abs() < 4_096
        );
        assert!((first.uplift_q16() - second.uplift_q16()).abs() < 2_048);
        found_land_pair = true;
        break;
    }
    assert!(
        found_land_pair,
        "expected adjacent land samples across the region boundary"
    );
}

#[test]
fn legacy_surface_samples_have_no_mountain_barrier_signal() {
    let coordinate = MacroCoordinate::new(-1_337, 2_048);
    let terrain = terrain::sample_surface(42, 1, coordinate).unwrap();

    assert_eq!(terrain.mountain_ridge_strength_q16(), 0);
    assert!(!terrain.is_mountain_barrier());
}

#[test]
fn surface_terrain_barrier_sampling_rejects_underground_strata() {
    assert!(terrain::sample_surface(
        42,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
    )
    .is_none());
}


#[test]
fn neighboring_surface_chunks_are_continuous_and_sampling_order_independent() {
    let seed = 8_211;
    let boundaries = [0_i64, 256_i64];
    let offsets = [-1_i64, 0, 255];
    let mut coordinates = Vec::new();
    let mut horizontal_pairs = Vec::new();
    let mut vertical_pairs = Vec::new();

    for boundary in boundaries {
        for offset in offsets {
            let west = MacroCoordinate::new(boundary - 1, offset);
            let east = MacroCoordinate::new(boundary, offset);
            let north = MacroCoordinate::new(offset, boundary - 1);
            let south = MacroCoordinate::new(offset, boundary);

            horizontal_pairs.push((west, east));
            vertical_pairs.push((north, south));
            coordinates.extend([west, east, north, south]);
        }
    }

    let sample_in_order = |ordered: &[MacroCoordinate]| {
        ordered
            .iter()
            .map(|coordinate| {
                (
                    *coordinate,
                    terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate),
                    mountains::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate),
                    climate::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate),
                    coast::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate),
                    watershed::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate),
                    hydrology::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate),
                )
            })
            .collect::<Vec<_>>()
    };

    // The same border strips are evaluated as if neighboring chunks were
    // visited in opposite orders. Every derived field must remain identical.
    let forward = sample_in_order(&coordinates);
    let reversed_coordinates = coordinates.iter().rev().copied().collect::<Vec<_>>();
    let reverse = sample_in_order(&reversed_coordinates);
    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());

    // Region ownership is only a partition of the global lattice. Converting
    // each border sample through its owning chunk must not change its identity.
    for coordinate in &coordinates {
        let (region, local_x, local_y) = coordinate.region_and_local();
        assert_eq!(
            region.global_sample(local_x, local_y),
            Some(*coordinate),
            "region-local conversion changed {coordinate:?}"
        );
    }

    // Both sides of every seam must agree about whether the shared edge is a
    // shoreline. This catches chunk-local coastline masks that disagree.
    for (first, second) in horizontal_pairs {
        let first_shore = coast::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, first).unwrap();
        let second_shore = coast::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, second).unwrap();
        assert_eq!(
            first_shore.has_opposite_neighbor(coast::direction::EAST),
            second_shore.has_opposite_neighbor(coast::direction::WEST),
            "east/west shoreline masks disagree across {first:?} and {second:?}"
        );
    }
    for (first, second) in vertical_pairs {
        let first_shore = coast::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, first).unwrap();
        let second_shore = coast::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, second).unwrap();
        assert_eq!(
            first_shore.has_opposite_neighbor(coast::direction::SOUTH),
            second_shore.has_opposite_neighbor(coast::direction::NORTH),
            "north/south shoreline masks disagree across {first:?} and {second:?}"
        );
    }
}

use super::*;
use crate::geography::{RegionCoordinate, GEOGRAPHY_RULESET_VERSION};

#[test]
fn river_sampling_is_deterministic_and_bounded() {
    let coordinate = MacroCoordinate::new(-2_048, 6_144);
    let first = sample_surface(8_211, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = sample_surface(8_211, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();

    assert_eq!(first, repeated);
    assert!(usize::from(first.accumulation().contributing_cells()) <= MAX_CONTRIBUTING_CELLS);
    assert!(
        first.accumulation().precipitation_potential_sum_q16()
            <= (MAX_CONTRIBUTING_CELLS as u64) * (1 << 16)
    );
    assert!(
        !first.is_channel()
            || first.accumulation().contributing_cells() >= MIN_RIVER_CONTRIBUTING_CELLS
    );
}

#[test]
fn upstream_cells_are_included_in_their_downstream_accumulation() {
    let seed = 8_211;
    let mut verified = false;

    for coarse_y in -8..=8 {
        for coarse_x in -8..=8 {
            let Some(coordinate) = watershed::cell_center(coarse_x, coarse_y) else {
                continue;
            };
            let sample =
                watershed::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
            let Some(downstream) = sample.coarse_downstream() else {
                continue;
            };

            if !terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, downstream)
                .unwrap()
                .is_land()
            {
                continue;
            }

            let downstream_flow =
                sample_surface(seed, GEOGRAPHY_RULESET_VERSION, downstream).unwrap();
            assert!(downstream_flow.accumulation().contributing_cells() >= 2);

            let upstream_precipitation = u64::from(
                climate::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                    .unwrap()
                    .precipitation_q16(),
            );
            let downstream_precipitation = u64::from(
                climate::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, downstream)
                    .unwrap()
                    .precipitation_q16(),
            );
            assert!(
                downstream_flow
                    .accumulation()
                    .precipitation_potential_sum_q16()
                    >= upstream_precipitation + downstream_precipitation
            );
            verified = true;
            break;
        }

        if verified {
            break;
        }
    }

    assert!(
        verified,
        "expected a land watershed cell with a downstream link"
    );
}

#[test]
fn ocean_samples_have_no_river_flow_or_precipitation_sum() {
    let seed = 8_211;
    let mut ocean_coordinate = None;

    'search: for coarse_y in -32..=32 {
        for coarse_x in -32..=32 {
            let Some(coordinate) = watershed::cell_center(coarse_x, coarse_y) else {
                continue;
            };
            if terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                .is_some_and(|sample| !sample.is_land())
            {
                ocean_coordinate = Some(coordinate);
                break 'search;
            }
        }
    }

    let coordinate = ocean_coordinate.expect("expected ocean in the deterministic search area");
    let sample = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();

    assert_eq!(sample.accumulation().contributing_cells(), 0);
    assert_eq!(sample.accumulation().precipitation_potential_sum_q16(), 0);
    assert!(!sample.is_channel());
}

#[test]
fn river_sampler_rejects_underground_strata() {
    assert!(sample_surface(
        8_211,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
    )
    .is_none());
}

#[test]
fn river_sampler_handles_extreme_signed_coordinates() {
    for coordinate in [
        MacroCoordinate::new(i64::MIN, i64::MIN),
        MacroCoordinate::new(i64::MAX, i64::MAX),
    ] {
        assert!(
            sample_surface(8_211, GEOGRAPHY_RULESET_VERSION, coordinate).is_some(),
            "expected the outermost representable coordinates to map to valid coarse cells"
        );
    }
}

#[test]
fn river_channel_threshold_uses_contributing_area() {
    let below = RiverSample {
        coarse_cell: MacroCoordinate::ORIGIN,
        accumulation: FlowAccumulation {
            contributing_cells: MIN_RIVER_CONTRIBUTING_CELLS - 1,
            precipitation_potential_sum_q16: 0,
            truncated: false,
        },
    };
    let at = RiverSample {
        coarse_cell: MacroCoordinate::ORIGIN,
        accumulation: FlowAccumulation {
            contributing_cells: MIN_RIVER_CONTRIBUTING_CELLS,
            precipitation_potential_sum_q16: 0,
            truncated: false,
        },
    };

    assert!(!below.is_channel());
    assert!(at.is_channel());
}

#[test]
fn truncated_accumulation_is_explicit_and_preserves_confirmed_channels() {
    let sample = RiverSample {
        coarse_cell: MacroCoordinate::ORIGIN,
        accumulation: FlowAccumulation {
            contributing_cells: MAX_CONTRIBUTING_CELLS as u16,
            precipitation_potential_sum_q16: (MAX_CONTRIBUTING_CELLS as u64) * (1 << 16),
            truncated: true,
        },
    };

    assert!(sample.accumulation().is_truncated());
    assert!(!sample.accumulation().is_complete());
    assert!(sample.is_channel());
    assert_eq!(sample.width_class(), RiverWidthClass::VeryWide);
}

#[test]
fn river_width_classes_follow_contributing_area() {
    let sample = |cells| RiverSample {
        coarse_cell: MacroCoordinate::ORIGIN,
        accumulation: FlowAccumulation {
            contributing_cells: cells,
            precipitation_potential_sum_q16: 0,
            truncated: false,
        },
    };

    assert_eq!(sample(8).width_class(), RiverWidthClass::Narrow);
    assert_eq!(sample(15).width_class(), RiverWidthClass::Narrow);
    assert_eq!(sample(16).width_class(), RiverWidthClass::Medium);
    assert_eq!(sample(31).width_class(), RiverWidthClass::Medium);
    assert_eq!(sample(32).width_class(), RiverWidthClass::Wide);
    assert_eq!(sample(63).width_class(), RiverWidthClass::Wide);
    assert_eq!(sample(64).width_class(), RiverWidthClass::VeryWide);
    assert!(RiverWidthClass::Narrow.is_fordable());
    assert!(RiverWidthClass::Medium.is_fordable());
    assert!(!RiverWidthClass::Wide.is_fordable());
    assert!(!RiverWidthClass::VeryWide.is_fordable());
}

#[test]
fn river_network_traces_downhill_courses_with_stable_outlet_identity() {
    let fixture = [8_211, 42, 1_701, 7_321, 12_345, 1, 2, 3]
        .into_iter()
        .find_map(|seed| {
            for coarse_y in -48..=48 {
                for coarse_x in -48..=48 {
                    let Some(coordinate) = watershed::cell_center(coarse_x, coarse_y) else {
                        continue;
                    };
                    let Some(channel) = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                    else {
                        continue;
                    };
                    if !channel.is_channel() {
                        continue;
                    }
                    let Some(network) = trace_network(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                    else {
                        continue;
                    };
                    if network.path.len() >= 2
                        && network.outlet.kind != RiverOutletKind::SearchLimit
                    {
                        return Some((seed, network));
                    }
                }
            }
            None
        })
        .expect("expected a multi-cell river course draining to a bounded outlet");

    let (seed, network) = fixture;
    assert!(network.path.len() <= MAX_RIVER_NETWORK_STEPS);
    assert_eq!(
        sample_surface(seed, GEOGRAPHY_RULESET_VERSION, network.path[0].coordinate)
            .and_then(|sample| sample.network_id(seed, GEOGRAPHY_RULESET_VERSION)),
        Some(network.id),
    );
    assert_eq!(
        trace_network(seed, GEOGRAPHY_RULESET_VERSION, network.path[0].coordinate),
        Some(network.clone())
    );

    for pair in network.path.windows(2) {
        assert_eq!(pair[0].downstream, Some(pair[1].coordinate));
        assert!(pair[0].flow_direction.is_some());
        let current = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, pair[0].coordinate)
            .unwrap()
            .elevation_q16();
        let next = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, pair[1].coordinate)
            .unwrap()
            .elevation_q16();
        assert!(next < current, "river courses must follow downhill links");
    }

    let downstream_trace =
        trace_network(seed, GEOGRAPHY_RULESET_VERSION, network.path[1].coordinate)
            .expect("a downstream channel point should still belong to the same river");
    assert_eq!(
        downstream_trace.path[0].coordinate,
        network.path[1].coordinate
    );

    // Search-limited watershed identities are explicitly provisional: a trace
    // started farther downstream may reach a different bounded continuation.
    // Once the watershed identity reaches a real outlet, every point on the
    // same course must resolve to that shared outlet identity.
    if network.id.outlet_kind != RiverOutletKind::SearchLimit {
        assert_eq!(downstream_trace.id, network.id);
    }
}

#[test]
fn river_sampling_is_independent_of_region_chunk_ownership() {
    let coordinate = MacroCoordinate::new(-257, 511);
    let (region, local_x, local_y) = coordinate.region_and_local();
    let reconstructed = RegionCoordinate::new(region.x, region.y)
        .global_sample(local_x, local_y)
        .unwrap();

    assert_eq!(coordinate, reconstructed);
    assert_eq!(
        sample_surface(1_701, GEOGRAPHY_RULESET_VERSION, coordinate),
        sample_surface(1_701, GEOGRAPHY_RULESET_VERSION, reconstructed),
    );
}

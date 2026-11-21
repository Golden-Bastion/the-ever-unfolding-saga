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
            truncated: false,
        },
    };
    let at = RiverSample {
        coarse_cell: MacroCoordinate::ORIGIN,
        accumulation: FlowAccumulation {
            contributing_cells: MIN_RIVER_CONTRIBUTING_CELLS,
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
            truncated: true,
        },
    };

    assert!(sample.accumulation().is_truncated());
    assert!(!sample.accumulation().is_complete());
    assert!(sample.is_channel());
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

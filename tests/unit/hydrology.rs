use super::*;
use crate::geography::{MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

fn find_lake_cell(seed: u64) -> Option<MacroCoordinate> {
    for coarse_y in (-40..=-1).rev() {
        for coarse_x in (-40..=-1).rev() {
            let coordinate = watershed::cell_center(coarse_x, coarse_y)?;
            let elevation = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)?;
            if !elevation.is_land()
                || watershed::downstream_cell(
                    seed,
                    GEOGRAPHY_RULESET_VERSION,
                    coarse_x,
                    coarse_y,
                    elevation.elevation_q16(),
                )
                .is_some()
            {
                continue;
            }
            if waterbodies::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).is_some() {
                return Some(coordinate);
            }
        }
    }
    None
}

#[test]
fn ocean_samples_are_terminal_surface_water() {
    let seed = 8_211;
    let ocean = (-16_384..16_384)
        .step_by(256)
        .flat_map(|y| {
            (-16_384..16_384)
                .step_by(256)
                .map(move |x| MacroCoordinate::new(x, y))
        })
        .find(|coordinate| {
            terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, *coordinate)
                .is_some_and(|sample| !sample.is_land())
        })
        .expect("expected ocean terrain in the deterministic search area");

    let sample = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, ocean).unwrap();
    assert_eq!(sample.kind(), SurfaceWaterKind::Ocean);
    assert_eq!(sample.downstream_cell(), None);
    assert_eq!(sample.downstream(), None);
}

#[test]
fn inland_lakes_are_terminal_nodes_with_stable_outlet_identity() {
    let (seed, coordinate) = (1..=24)
        .find_map(|seed| find_lake_cell(seed).map(|coordinate| (seed, coordinate)))
        .expect("expected a generated inland lake in the deterministic search area");

    let first = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    assert_eq!(first, repeated);
    assert!(matches!(first.kind(), SurfaceWaterKind::Lake(_)));
    assert_eq!(first.downstream_cell(), None);
    assert_eq!(first.downstream(), None);
}

#[test]
fn river_channels_expose_a_deterministic_downstream_connection() {
    let seed = 8_211;
    // Confirmed channels need at least eight contributing coarse cells. Search
    // a continental-scale area so this test does not depend on a river
    // happening to cross a small, arbitrary 33 × 33-cell window.
    let river = (-64..=64)
        .flat_map(|y| (-64..=64).map(move |x| (x, y)))
        .filter_map(|(x, y)| watershed::cell_center(x, y))
        .find_map(|coordinate| {
            let flow = rivers::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)?;
            if !flow.is_channel() {
                return None;
            }
            sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                .filter(|sample| matches!(sample.kind(), SurfaceWaterKind::River(_)))
        })
        .expect("expected a classified river channel in the deterministic search area");

    let repeated = sample_surface(seed, GEOGRAPHY_RULESET_VERSION, river.coordinate()).unwrap();
    assert_eq!(river, repeated);
    assert!(river.downstream().is_some());

    if let Some(next) = river.downstream_cell() {
        let current_elevation =
            terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, river.coordinate())
                .unwrap()
                .elevation_q16();
        let next_elevation = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, next)
            .unwrap()
            .elevation_q16();
        assert!(next_elevation < current_elevation);

        match river.downstream().unwrap() {
            FlowDestination::Ocean => {
                assert!(
                    !terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, next)
                        .unwrap()
                        .is_land()
                )
            }
            FlowDestination::Lake { outlet_cell } => {
                let lake = waterbodies::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, next)
                    .expect("lake destination must be covered by an inland lake");
                assert_eq!(lake.outlet_cell(), outlet_cell);
            }
            FlowDestination::River { coarse_cell }
            | FlowDestination::Land { coarse_cell }
            | FlowDestination::Inconclusive { coarse_cell } => {
                assert_eq!(coarse_cell, next);
            }
            FlowDestination::ClosedBasin => {
                panic!("a linked river cell cannot terminate at a closed basin")
            }
        }
    } else {
        assert_eq!(river.downstream(), Some(FlowDestination::ClosedBasin));
    }
}

#[test]
fn surface_hydrology_rejects_underground_strata() {
    assert!(sample_surface(
        8_211,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
    )
    .is_none());
}

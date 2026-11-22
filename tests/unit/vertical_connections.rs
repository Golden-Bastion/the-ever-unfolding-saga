use super::*;
use crate::geography::{MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

#[test]
fn surface_entrances_are_land_to_open_cave_and_deterministic() {
    let mut found = None;
    'search: for cell_y in -24..24 {
        for cell_x in -24..24 {
            if let Some(connection) = connection_for_cell(
                7_321, GEOGRAPHY_RULESET_VERSION, cell_x, cell_y, 0,
            ) {
                found = Some(connection);
                break 'search;
            }
        }
    }
    let entrance = found.expect("expected a surface cave entrance in the search area");
    assert_eq!(entrance.kind, VerticalConnectionKind::SurfaceEntrance);
    assert_eq!(entrance.upper.stratum, 0);
    assert_eq!(entrance.lower.stratum, -1);
    assert!(terrain::sample_surface(7_321, GEOGRAPHY_RULESET_VERSION, entrance.upper)
        .unwrap().is_land());
    assert!(underground::sample_underground(7_321, GEOGRAPHY_RULESET_VERSION, entrance.lower)
        .unwrap().is_open());
    assert_eq!(
        Some(entrance),
        connection_for_cell(7_321, GEOGRAPHY_RULESET_VERSION,
            entrance.upper.x.div_euclid(underground::NETWORK_CELL_SAMPLES),
            entrance.upper.y.div_euclid(underground::NETWORK_CELL_SAMPLES), 0)
    );
}

#[test]
fn inter_stratum_connections_join_adjacent_open_cave_layers() {
    let mut found = None;
    'search: for cell_y in -64..64 {
        for cell_x in -64..64 {
            if let Some(connection) = connection_for_cell(
                42, GEOGRAPHY_RULESET_VERSION, cell_x, cell_y, -1,
            ) {
                found = Some(connection);
                break 'search;
            }
        }
    }
    let shaft = found.expect("expected an inter-stratum cave connection in the search area");
    assert_eq!(shaft.kind, VerticalConnectionKind::InterStratum);
    assert_eq!(shaft.upper.stratum, -1);
    assert_eq!(shaft.lower.stratum, -2);
    assert!(underground::sample_underground(42, GEOGRAPHY_RULESET_VERSION, shaft.upper)
        .unwrap().is_open());
    assert!(underground::sample_underground(42, GEOGRAPHY_RULESET_VERSION, shaft.lower)
        .unwrap().is_open());
}

#[test]
fn vertical_links_are_stable_and_never_skip_strata() {
    for upper_stratum in [0, -1, -2, -8] {
        if let Some(connection) = connection_for_cell(
            99, GEOGRAPHY_RULESET_VERSION, -3, 7, upper_stratum,
        ) {
            assert_eq!(connection.upper.stratum, upper_stratum);
            assert_eq!(connection.lower.stratum, upper_stratum - 1);
            assert_eq!(
                connection.id,
                connection_for_cell(99, GEOGRAPHY_RULESET_VERSION, -3, 7, upper_stratum)
                    .unwrap().id
            );
        }
    }
    assert!(connection_for_cell(1, GEOGRAPHY_RULESET_VERSION, 0, 0, 1).is_none());
    assert!(connection_for_cell(1, GEOGRAPHY_RULESET_VERSION, 0, 0, i32::MIN).is_none());
}

#[test]
fn nearby_queries_are_bounded_and_order_independent() {
    let origin = MacroCoordinate::in_stratum(-1, -65, -1);
    let first = connections_near(7_321, GEOGRAPHY_RULESET_VERSION, origin, u8::MAX);
    let second = connections_near(7_321, GEOGRAPHY_RULESET_VERSION, origin, 2);
    assert_eq!(first, second);
    assert!(first.len() <= 25);
    assert!(connections_near(7_321, GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::new(0, 0), 1).iter()
        .all(|connection| connection.upper.stratum == 0));
}

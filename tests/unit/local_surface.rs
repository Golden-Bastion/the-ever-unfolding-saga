use super::*;

use crate::geography::river_geometry::RiverPosition;
use crate::geography::{climate, MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

#[test]
fn location_projection_is_deterministic_and_keeps_macro_and_tile_resolutions_separate() {
    let seed = 41;
    let anchor = MacroCoordinate::new(1_237, -842);
    let first =
        LocalSurfaceMap::sample_location(seed, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48)
            .unwrap();
    let second =
        LocalSurfaceMap::sample_location(seed, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48)
            .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.dimensions(), (64, 48));
    assert_eq!(first.sample_dimensions(), (9, 7));

    let center = first.sample_at_tile(32, 24).unwrap();
    assert_eq!(
        center.biome(),
        climate::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, anchor)
            .unwrap()
            .biome()
    );
    assert!(first.sample_at_tile(64, 24).is_none());
}

#[test]
fn route_edge_projection_uses_the_exact_aspect_corrected_footprint_boundary() {
    use crate::geography::local_surface::LocalSurfaceEdgeSide;

    let anchor = MacroCoordinate::new(1_237, -842);
    let surface =
        LocalSurfaceMap::sample_location(41, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48).unwrap();

    let east = surface
        .route_edge(
            &[anchor, MacroCoordinate::new(anchor.x + 32, anchor.y + 8)],
            true,
        )
        .unwrap();
    assert_eq!(east.side, LocalSurfaceEdgeSide::East);
    assert_eq!(east.offset_q16, 43_690);

    // A 64×48 map is wider than it is tall, so this diagonal leaves the south edge first.
    let south = surface
        .route_edge(
            &[anchor, MacroCoordinate::new(anchor.x + 32, anchor.y + 32)],
            true,
        )
        .unwrap();
    assert_eq!(south.side, LocalSurfaceEdgeSide::South);
    assert_eq!(south.offset_q16, 57_343);

    // At endpoint B, the first segment is followed in reverse travel direction.
    let west = surface
        .route_edge(
            &[MacroCoordinate::new(anchor.x - 32, anchor.y - 8), anchor],
            false,
        )
        .unwrap();
    assert_eq!(west.side, LocalSurfaceEdgeSide::West);
    assert_eq!(west.offset_q16, 21_845);

    assert!(surface
        .route_edge(
            &[MacroCoordinate::new(anchor.x - 32, anchor.y), anchor],
            true,
        )
        .is_none());
}

#[test]
fn exact_macro_positions_project_to_local_tiles_and_reject_outside_points() {
    let anchor = MacroCoordinate::new(-17, 23);
    let surface =
        LocalSurfaceMap::sample_location(41, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48).unwrap();
    let center = RiverPosition {
        x_q16: anchor.x * crate::geography::river_geometry::RIVER_POSITION_SCALE,
        y_q16: anchor.y * crate::geography::river_geometry::RIVER_POSITION_SCALE,
    };
    assert_eq!(
        surface.tile_for_position(center),
        Some(crate::geography::local_surface::LocalSurfaceTile { x: 32, y: 24 })
    );
    assert!(surface
        .tile_for_position(RiverPosition {
            x_q16: center.x_q16 + 5 * crate::geography::river_geometry::RIVER_POSITION_SCALE,
            y_q16: center.y_q16,
        })
        .is_none());
}

#[test]
fn generated_river_channels_occupy_only_their_local_tile_footprint() {
    use crate::geography::hydrology::{self, SurfaceWaterKind};
    use crate::geography::watershed;

    let fixture = [8_211, 42, 1_701, 7_321, 12_345, 1, 2, 3]
        .into_iter()
        .find_map(|seed| {
            for coarse_y in -48..=48 {
                for coarse_x in -48..=48 {
                    let coordinate = watershed::cell_center(coarse_x, coarse_y)?;
                    let Some(sample) =
                        hydrology::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, coordinate)
                    else {
                        continue;
                    };
                    if let SurfaceWaterKind::River(river) = sample.kind() {
                        if river.network_id(seed, GEOGRAPHY_RULESET_VERSION).is_some()
                            && river
                                .flow_direction(seed, GEOGRAPHY_RULESET_VERSION)
                                .is_some()
                        {
                            return Some((seed, coordinate));
                        }
                    }
                }
            }
            None
        })
        .expect("expected a deterministic river suitable for local projection");

    let (seed, anchor) = fixture;
    let surface =
        LocalSurfaceMap::sample_location(seed, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48)
            .unwrap();

    let center = surface.sample_at_tile(32, 24).unwrap();
    assert!(center.is_river());
    assert!(matches!(center.water(), SurfaceWaterFeature::Lake));
    assert!(center.river_width().is_some());

    let river_tiles = (0..48)
        .flat_map(|y| (0..64).map(move |x| (x, y)))
        .filter(|(x, y)| {
            surface
                .sample_at_tile(*x, *y)
                .is_some_and(|sample| sample.is_river())
        })
        .count();
    assert!(river_tiles > 0);
    assert!(river_tiles < 64 * 48);
}

#[test]
fn macro_sample_axis_is_evenly_spaced_and_bounded() {
    let samples = sample_axis(-17, 4).unwrap();
    assert_eq!(samples.len(), 9);
    assert_eq!(samples.first(), Some(&-21));
    assert_eq!(samples.get(4), Some(&-17));
    assert_eq!(samples.last(), Some(&-13));

    let capped = sample_axis(10, 100).unwrap();
    assert_eq!(capped.len(), 17);
    assert_eq!(capped.first(), Some(&-90));
    assert_eq!(capped.last(), Some(&110));
}

#[test]
fn projection_rejects_underground_and_invalid_local_dimensions() {
    assert!(LocalSurfaceMap::sample_location(
        1,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::in_stratum(0, 0, -1),
        4,
        64,
        48,
    )
    .is_none());
    assert!(LocalSurfaceMap::sample_location(
        1,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::ORIGIN,
        4,
        0,
        48,
    )
    .is_none());
    assert!(LocalSurfaceMap::sample_location(
        1,
        GEOGRAPHY_RULESET_VERSION,
        MacroCoordinate::ORIGIN,
        4,
        129,
        48,
    )
    .is_none());
}

#[test]
fn sample_axis_rejects_coordinate_overflow() {
    assert!(sample_axis(i64::MAX, 1).is_none());
    assert!(sample_axis(i64::MIN, 1).is_none());
}

#[test]
fn surface_samples_retain_elevation_shoreline_and_moisture_for_presentation() {
    use crate::geography::{coast, terrain};

    let seed = 41;
    let anchor = MacroCoordinate::new(1_237, -842);
    let surface =
        LocalSurfaceMap::sample_location(seed, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48)
            .unwrap();
    let sample = surface.grid_sample(4, 3).unwrap();
    let terrain = terrain::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, anchor).unwrap();
    let climate = climate::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, anchor).unwrap();
    let shoreline = coast::sample_surface(seed, GEOGRAPHY_RULESET_VERSION, anchor).unwrap();

    assert_eq!(sample.elevation_q16(), terrain.elevation_q16());
    assert_eq!(
        sample.mountain_ridge_strength_q16(),
        terrain.mountain_ridge_strength_q16()
    );
    assert_eq!(sample.moisture_q16(), climate.moisture_q16());
    assert_eq!(sample.shoreline_mask(), shoreline.opposite_terrain_mask());
    assert_eq!(sample.is_shoreline(), shoreline.is_shoreline());
    assert_eq!(
        sample.shoreline_strength_q16(),
        if shoreline.is_shoreline() {
            u16::MAX
        } else {
            0
        }
    );
}

#[test]
fn continuous_surface_fields_are_bilinearly_interpolated_but_categories_remain_nearest() {
    let sample = |biome, elevation, ridge, moisture, shoreline| LocalSurfaceSample {
        biome,
        water: SurfaceWaterFeature::Land,
        elevation_q16: elevation,
        mountain_ridge_strength_q16: ridge,
        moisture_q16: moisture,
        shoreline_mask: if shoreline == 0 { 0 } else { 1 },
        shoreline_strength_q16: shoreline,
        mountain_barrier: false,
        river_width: None,
    };
    let surface = LocalSurfaceMap {
        anchor: MacroCoordinate::ORIGIN,
        radius_x: 1,
        radius_y: 1,
        tile_width: 4,
        tile_height: 4,
        sample_width: 2,
        sample_height: 2,
        samples: vec![
            sample(Biome::Grassland, 0, 0, 0, 0),
            sample(Biome::Desert, 100, 1_000, 10_000, u16::MAX),
            sample(Biome::Tundra, 200, 2_000, 20_000, u16::MAX),
            sample(Biome::Alpine, 300, 3_000, 30_000, 0),
        ],
        river_tiles: vec![None; 16],
    };

    // Tile (1, 1) lies one quarter of the way between the four sample centers.
    let projected = surface.sample_at_tile(1, 1).unwrap();
    assert_eq!(projected.elevation_q16(), 75);
    assert_eq!(projected.mountain_ridge_strength_q16(), 750);
    assert_eq!(projected.moisture_q16(), 7_500);
    assert_eq!(projected.shoreline_strength_q16(), 24_575);

    // Discrete world classifications and the directional mask still come
    // from the nearest authoritative macro sample.
    assert_eq!(projected.biome(), Biome::Grassland);
    assert_eq!(projected.water(), SurfaceWaterFeature::Land);
    assert_eq!(projected.shoreline_mask(), 0);
    assert!(!projected.is_shoreline());
}

#[test]
fn interpolation_is_symmetric_around_the_location_anchor() {
    let seed = 41;
    let anchor = MacroCoordinate::new(1_237, -842);
    let surface =
        LocalSurfaceMap::sample_location(seed, GEOGRAPHY_RULESET_VERSION, anchor, 4, 64, 48)
            .unwrap();
    let anchor_sample = surface.grid_sample(4, 3).unwrap();
    let surrounding = [
        surface.sample_at_tile(31, 23).unwrap(),
        surface.sample_at_tile(32, 23).unwrap(),
        surface.sample_at_tile(31, 24).unwrap(),
        surface.sample_at_tile(32, 24).unwrap(),
    ];

    let average_elevation = surrounding
        .iter()
        .map(|sample| i64::from(sample.elevation_q16()))
        .sum::<i64>()
        / 4;
    let average_ridge = surrounding
        .iter()
        .map(|sample| i64::from(sample.mountain_ridge_strength_q16()))
        .sum::<i64>()
        / 4;
    let average_moisture = surrounding
        .iter()
        .map(|sample| i64::from(sample.moisture_q16()))
        .sum::<i64>()
        / 4;

    assert!((average_elevation - i64::from(anchor_sample.elevation_q16())).abs() <= 2);
    assert!((average_ridge - i64::from(anchor_sample.mountain_ridge_strength_q16())).abs() <= 2);
    assert!((average_moisture - i64::from(anchor_sample.moisture_q16())).abs() <= 2);
}

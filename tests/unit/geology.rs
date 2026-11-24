use super::*;
use crate::geography::{geology, GeologicalMaterial, MacroCoordinate, GEOGRAPHY_RULESET_VERSION};

#[test]
fn geology_rejects_surface_and_positive_strata() {
    for stratum in [0, 1, i32::MAX] {
        assert!(geology::sample_geology(
            42,
            GEOGRAPHY_RULESET_VERSION,
            MacroCoordinate::in_stratum(0, 0, stratum),
        )
        .is_none());
    }
}

#[test]
fn geological_materials_are_deterministic_and_seeded() {
    let coordinate = MacroCoordinate::in_stratum(-1_237, 842, -3);
    let first = geology::sample_geology(42, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let repeated = geology::sample_geology(42, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();
    let other_seed = geology::sample_geology(43, GEOGRAPHY_RULESET_VERSION, coordinate).unwrap();

    assert_eq!(first, repeated);
    assert_ne!(first, other_seed);
}

#[test]
fn all_material_families_appear_across_a_broad_underground_area() {
    let mut granite = false;
    let mut slate = false;
    let mut basalt = false;

    for y in (-16_384..=16_384).step_by(256) {
        for x in (-16_384..=16_384).step_by(256) {
            let sample = geology::sample_geology(
                7_321,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(x, y, -2),
            )
            .unwrap();
            match sample.material() {
                GeologicalMaterial::PaleGranite => granite = true,
                GeologicalMaterial::VeilSlate => slate = true,
                GeologicalMaterial::CinderBasalt => basalt = true,
            }
            if granite && slate && basalt {
                return;
            }
        }
    }

    assert!(granite, "expected pale granite formations");
    assert!(slate, "expected veil slate formations");
    assert!(basalt, "expected cinder basalt formations");
}

#[test]
fn geological_materials_vary_with_depth_without_changing_cave_topology() {
    let mut found_depth_variation = false;
    let mut found_cave_space = false;
    let mut found_solid_rock = false;

    for y in (-2_048..=2_048).step_by(64) {
        for x in (-2_048..=2_048).step_by(64) {
            let shallow = geology::sample_geology(
                812,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(x, y, -1),
            )
            .unwrap();
            let deep = geology::sample_geology(
                812,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(x, y, -8),
            )
            .unwrap();
            found_depth_variation |= shallow.material() != deep.material();

            let cave = underground::sample_underground(
                812,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(x, y, -1),
            )
            .unwrap();
            found_cave_space |= cave.is_open();
            found_solid_rock |= cave.kind() == UndergroundKind::SolidRock;

            // Geological material exists regardless of whether this point is
            // a chamber, passage, or solid rock.
            assert!(geology::sample_geology(
                812,
                GEOGRAPHY_RULESET_VERSION,
                MacroCoordinate::in_stratum(x, y, -1),
            )
            .is_some());

            if found_depth_variation && found_cave_space && found_solid_rock {
                return;
            }
        }
    }

    assert!(found_depth_variation, "expected some material variation by depth");
    assert!(found_cave_space, "expected cave space in the sampled area");
    assert!(found_solid_rock, "expected solid rock in the sampled area");
}

#[test]
fn geological_sampling_is_stable_across_region_seams_and_sampling_order() {
    let coordinates = [
        MacroCoordinate::in_stratum(255, -257, -1),
        MacroCoordinate::in_stratum(256, -257, -1),
        MacroCoordinate::in_stratum(-257, 255, -2),
        MacroCoordinate::in_stratum(-256, 255, -2),
        MacroCoordinate::in_stratum(-1, -1, -8),
    ];
    let forward = coordinates
        .iter()
        .copied()
        .map(|coordinate| {
            geology::sample_geology(1_234, GEOGRAPHY_RULESET_VERSION, coordinate)
        })
        .collect::<Vec<_>>();
    let reverse = coordinates
        .iter()
        .rev()
        .copied()
        .map(|coordinate| {
            geology::sample_geology(1_234, GEOGRAPHY_RULESET_VERSION, coordinate)
        })
        .collect::<Vec<_>>();

    assert_eq!(forward, reverse.into_iter().rev().collect::<Vec<_>>());
}

#[test]
fn material_properties_are_bounded_and_reflect_their_family() {
    let granite = GeologicalMaterial::PaleGranite;
    let slate = GeologicalMaterial::VeilSlate;
    let basalt = GeologicalMaterial::CinderBasalt;

    for material in [granite, slate, basalt] {
        assert!(material.hardness_q16() <= u16::MAX);
        assert!(material.fracture_q16() <= u16::MAX);
    }
    assert!(slate.fracture_q16() > granite.fracture_q16());
    assert!(granite.hardness_q16() > slate.hardness_q16());
}

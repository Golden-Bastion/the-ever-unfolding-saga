//! Isolated Bevy 0.20 3D scene used as an alternate application entry point.
//!
//! The prototype owns its camera, scene, materials, and demo motion. It reuses
//! the mod-aware asset resolver but does not create or mutate gameplay state.

use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImagePlugin;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

const MAP_SIZE: u16 = 8;
const ATLAS_COLUMNS: usize = 6;
const ATLAS_ROWS: usize = 8;
const WALK_FRAME_DURATION_SECONDS: f32 = 0.12;
const PATH_CYCLE_SECONDS: f32 = 8.0;
const CAMERA_HEIGHT: f32 = 12.0;

#[derive(Component)]
struct AnimatedActor {
    frames: Vec<Handle<Mesh>>,
    start: Vec2,
    end: Vec2,
    phase: f32,
}

#[derive(Component)]
struct ShadowToggleLight;

pub(crate) fn run() {
    // Android supplies its native surface through GameActivity. The desktop
    // prototype uses a fixed landscape window; both share the same renderer.
    let default_plugins = DefaultPlugins.set(ImagePlugin::default_nearest());
    #[cfg(not(target_os = "android"))]
    let default_plugins = default_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "TEUS - Spatial 3D Renderer Prototype".to_string(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    });

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.03, 0.04)))
        .add_plugins(default_plugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_actors, toggle_shadow_maps))
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_position = Vec3::new(10.0, 14.0, 10.0);
    let sprite_rotation = Quat::from_rotation_y(PI * 0.25);

    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: CAMERA_HEIGHT,
            },
            ..OrthographicProjection::default_3d()
        }),
        Msaa::Off,
        Transform::from_translation(camera_position).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Keep the lighting test intentionally small: one directional light owns
    // shadow-map cost, and L toggles its shadow maps for a direct comparison.
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        ShadowToggleLight,
        Transform::from_xyz(8.0, 14.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let floor_image = load_spatial_image(&mut images, "spatial/terrain_floor.png");
    let ash_image = load_spatial_image(&mut images, "spatial/terrain_ash.png");
    let rubble_image = load_spatial_image(&mut images, "spatial/terrain_rubble.png");
    let wall_top_image = load_spatial_image(&mut images, "spatial/terrain_wall_top.png");
    let wall_left_image = load_spatial_image(&mut images, "spatial/wall_face_left.png");
    let wall_right_image = load_spatial_image(&mut images, "spatial/wall_face_right.png");
    let table_image = load_spatial_image(&mut images, "spatial/prop_table.png");
    let player_sheet = load_spatial_image(&mut images, "spatial/player_oathbound_animated.png");

    let floor_material = materials.add(masked_material(floor_image));
    let ash_material = materials.add(masked_material(ash_image));
    let rubble_material = materials.add(masked_material(rubble_image));
    let wall_top_material = materials.add(masked_material(wall_top_image));
    let wall_left_material = materials.add(masked_material(wall_left_image));
    let wall_right_material = materials.add(masked_material(wall_right_image));
    let table_material = materials.add(masked_material(table_image));
    let player_material = materials.add(masked_material(player_sheet));

    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.105, 0.11),
        perceptual_roughness: 1.0,
        ..default()
    });
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(18.0, 18.0));
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.08, 0.0),
    ));

    let tile_mesh = meshes.add(Plane3d::default().mesh().size(1.08, 1.08));
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let material = if (x + y * 3) % 11 == 0 {
                rubble_material.clone()
            } else if (x * 2 + y) % 7 == 0 {
                ash_material.clone()
            } else {
                floor_material.clone()
            };
            commands.spawn((
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(
                    grid_to_world(Vec2::new(f32::from(x) + 0.5, f32::from(y) + 0.5))
                        + Vec3::Y * 0.005,
                ),
            ));
        }
    }

    // A wall is represented as a horizontal top and separate vertical face
    // pieces. The depth buffer, not hand-authored sprite Z values, resolves
    // which piece appears in front of an actor.
    let wall_ground = grid_to_world(Vec2::new(5.5, 5.5));
    let wall_top_mesh = meshes.add(Plane3d::default().mesh().size(1.9, 1.4));
    commands.spawn((
        Mesh3d(wall_top_mesh),
        MeshMaterial3d(wall_top_material),
        Transform::from_translation(wall_ground + Vec3::Y * 1.25),
    ));
    let wall_face_mesh = meshes.add(vertical_quad(Vec2::new(1.0, 1.25), full_texture_uvs()));
    commands.spawn((
        Mesh3d(wall_face_mesh.clone()),
        MeshMaterial3d(wall_left_material),
        Transform::from_translation(wall_ground + Vec3::new(0.0, 0.0, 0.42)),
    ));
    commands.spawn((
        Mesh3d(wall_face_mesh),
        MeshMaterial3d(wall_right_material),
        Transform::from_translation(wall_ground + Vec3::new(0.42, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_y(PI * 0.5)),
    ));

    // The table is a deliberately wide sprite over a smaller conceptual
    // footprint. Actors move through its ground position from both sides.
    let table_mesh = meshes.add(vertical_quad(Vec2::new(2.2, 1.45), full_texture_uvs()));
    commands.spawn((
        Mesh3d(table_mesh),
        MeshMaterial3d(table_material),
        Transform::from_translation(grid_to_world(Vec2::new(3.9, 3.9)))
            .with_rotation(sprite_rotation),
    ));

    // A simple canopy/trunk proxy exercises actual 3D geometry and shadows
    // alongside the existing sprite assets; it is not a production game asset.
    let tree_ground = grid_to_world(Vec2::new(6.2, 3.0));
    let trunk_mesh = meshes.add(Cuboid::new(0.22, 1.3, 0.22));
    let trunk_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.23, 0.13, 0.08),
        perceptual_roughness: 1.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(trunk_mesh),
        MeshMaterial3d(trunk_material),
        Transform::from_translation(tree_ground + Vec3::Y * 0.65),
    ));
    let canopy_mesh = meshes.add(Sphere::new(0.88));
    let canopy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.22, 0.13),
        perceptual_roughness: 1.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(canopy_mesh),
        MeshMaterial3d(canopy_material),
        Transform::from_translation(tree_ground + Vec3::Y * 1.75)
            .with_scale(Vec3::new(1.35, 1.05, 1.15)),
    ));

    let atlas_frames = build_walk_frames(&mut meshes, Vec2::new(0.82, 1.22));
    commands.spawn((
        Mesh3d(atlas_frames.east[0].clone()),
        MeshMaterial3d(player_material.clone()),
        AnimatedActor {
            frames: atlas_frames.east,
            start: Vec2::new(1.0, 1.0),
            end: Vec2::new(7.0, 7.0),
            phase: 0.0,
        },
        Transform::from_translation(grid_to_world(Vec2::new(1.0, 1.0)))
            .with_rotation(sprite_rotation),
    ));
    commands.spawn((
        Mesh3d(atlas_frames.west[0].clone()),
        MeshMaterial3d(player_material),
        AnimatedActor {
            frames: atlas_frames.west,
            start: Vec2::new(7.0, 1.0),
            end: Vec2::new(1.0, 7.0),
            phase: 0.37,
        },
        Transform::from_translation(grid_to_world(Vec2::new(7.0, 1.0)))
            .with_rotation(sprite_rotation),
    ));

    info!(
        "Spatial 3D prototype: two animated actors cross the map; L toggles directional-light shadow maps."
    );
}

fn load_spatial_image(images: &mut Assets<Image>, reference: &str) -> Handle<Image> {
    crate::asset_loading::load_image_from_data(images, reference)
        .unwrap_or_else(|error| panic!("Could not load spatial prototype image {reference}: {error}"))
}

fn masked_material(texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    }
}

fn build_walk_frames(meshes: &mut Assets<Mesh>, size: Vec2) -> DirectionalWalkFrames {
    let east = (0..ATLAS_COLUMNS)
        .map(|column| {
            meshes.add(vertical_quad(
                size,
                atlas_frame_uvs(3, column, ATLAS_COLUMNS, ATLAS_ROWS),
            ))
        })
        .collect();
    let west = (0..ATLAS_COLUMNS)
        .map(|column| {
            meshes.add(vertical_quad(
                size,
                atlas_frame_uvs(7, column, ATLAS_COLUMNS, ATLAS_ROWS),
            ))
        })
        .collect();
    DirectionalWalkFrames { east, west }
}

struct DirectionalWalkFrames {
    east: Vec<Handle<Mesh>>,
    west: Vec<Handle<Mesh>>,
}

fn vertical_quad(size: Vec2, uvs: [[f32; 2]; 4]) -> Mesh {
    let half_width = size.x * 0.5;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-half_width, 0.0, 0.0],
            [half_width, 0.0, 0.0],
            [half_width, size.y, 0.0],
            [-half_width, size.y, 0.0],
        ],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.to_vec());
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

fn full_texture_uvs() -> [[f32; 2]; 4] {
    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
}

/// UVs follow Bevy's top-left texture origin. The mesh origin is at the feet,
/// so every atlas frame keeps the same ground-contact anchor.
fn atlas_frame_uvs(row: usize, column: usize, columns: usize, rows: usize) -> [[f32; 2]; 4] {
    let u0 = column as f32 / columns as f32;
    let u1 = (column + 1) as f32 / columns as f32;
    let v0 = row as f32 / rows as f32;
    let v1 = (row + 1) as f32 / rows as f32;
    [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
}

fn grid_to_world(grid_position: Vec2) -> Vec3 {
    let centered = grid_position - Vec2::splat(f32::from(MAP_SIZE) * 0.5);
    Vec3::new(centered.x, 0.0, centered.y)
}

fn animate_actors(
    time: Res<Time>,
    mut actors: Query<(&AnimatedActor, &mut Mesh3d, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();
    for (actor, mut mesh, mut transform) in &mut actors {
        let path_progress = ((elapsed / PATH_CYCLE_SECONDS) + actor.phase).rem_euclid(1.0);
        let grid_position = actor.start.lerp(actor.end, path_progress);
        transform.translation = grid_to_world(grid_position);

        let animation_frame =
            ((elapsed / WALK_FRAME_DURATION_SECONDS).floor() as usize) % actor.frames.len();
        mesh.0 = actor.frames[animation_frame].clone();
    }
}

fn toggle_shadow_maps(
    keys: Res<ButtonInput<KeyCode>>,
    mut lights: Query<&mut DirectionalLight, With<ShadowToggleLight>>,
) {
    if !keys.just_pressed(KeyCode::KeyL) {
        return;
    }
    for mut light in &mut lights {
        light.shadow_maps_enabled = !light.shadow_maps_enabled;
        info!(
            "Directional-light shadow maps: {}",
            light.shadow_maps_enabled
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_frame_uvs_select_the_expected_top_left_cell() {
        let uvs = atlas_frame_uvs(0, 0, ATLAS_COLUMNS, ATLAS_ROWS);
        assert_eq!(uvs[3], [0.0, 0.0]);
        assert_eq!(uvs[2], [1.0 / 6.0, 0.0]);
        assert_eq!(uvs[0], [0.0, 1.0 / 8.0]);
    }

    #[test]
    fn atlas_frame_uvs_select_the_last_cell_without_exceeding_the_sheet() {
        let uvs = atlas_frame_uvs(7, 5, ATLAS_COLUMNS, ATLAS_ROWS);
        assert_eq!(uvs[3], [5.0 / 6.0, 7.0 / 8.0]);
        assert_eq!(uvs[2], [1.0, 7.0 / 8.0]);
        assert_eq!(uvs[0], [5.0 / 6.0, 1.0]);
    }

    #[test]
    fn grid_projection_preserves_orthogonal_unit_gameplay_axes() {
        let origin = grid_to_world(Vec2::ZERO);
        let x_axis = grid_to_world(Vec2::X) - origin;
        let y_axis = grid_to_world(Vec2::Y) - origin;
        assert!(x_axis.dot(y_axis).abs() < 0.0001);
        assert!((x_axis.length() - 1.0).abs() < 0.0001);
        assert!((y_axis.length() - 1.0).abs() < 0.0001);
    }
}

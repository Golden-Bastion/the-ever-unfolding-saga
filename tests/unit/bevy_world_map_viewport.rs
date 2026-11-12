use super::*;

fn assert_vec2_approx_eq(actual: Vec2, expected: Vec2) {
    assert!(
        actual.abs_diff_eq(expected, 0.001),
        "expected {expected:?}, got {actual:?}"
    );
}

#[test]
fn world_map_fit_zoom_preserves_margin_and_fits_both_axes() {
    assert!(
        (world_map_fit_zoom(Vec2::new(2000.0, 1000.0), Vec2::new(1000.0, 700.0))
            - 0.46)
            .abs()
            < 0.001
    );
}

#[test]
fn world_map_initial_view_centers_the_current_location() {
    let pan = world_map_initial_pan(
        Vec2::new(1200.0, 800.0),
        Vec2::new(600.0, 400.0),
        Vec2::new(300.0, 200.0),
        1.0,
    );
    assert_vec2_approx_eq(pan, Vec2::new(300.0, 200.0));
}

#[test]
fn world_map_pan_is_clamped_independently_on_both_axes() {
    let pan = world_map_clamp_pan(
        Vec2::new(500.0, -500.0),
        1.0,
        Vec2::new(1000.0, 800.0),
        Vec2::new(600.0, 400.0),
    );
    assert_vec2_approx_eq(pan, Vec2::new(200.0, -200.0));

    let centered_pan = world_map_clamp_pan(
        Vec2::new(80.0, -80.0),
        0.5,
        Vec2::new(1000.0, 800.0),
        Vec2::new(600.0, 400.0),
    );
    assert_vec2_approx_eq(centered_pan, Vec2::ZERO);
}

#[test]
fn pointer_zoom_keeps_the_map_point_under_the_pointer_stationary() {
    let pan = Vec2::new(30.0, -25.0);
    let old_zoom = 1.25;
    let new_zoom = 1.75;
    let anchor = Vec2::new(120.0, -60.0);
    let map_point = (anchor - pan) / old_zoom;

    let new_pan = world_map_zoom_pan(pan, old_zoom, new_zoom, anchor, anchor);
    assert_vec2_approx_eq(new_pan + map_point * new_zoom, anchor);
}

#[test]
fn pinch_zoom_follows_a_moving_midpoint_without_losing_its_map_point() {
    let pan = Vec2::new(-40.0, 15.0);
    let old_zoom = 0.9;
    let new_zoom = 1.35;
    let previous_midpoint = Vec2::new(-100.0, 30.0);
    let current_midpoint = Vec2::new(-80.0, 10.0);
    let map_point = (previous_midpoint - pan) / old_zoom;

    let new_pan = world_map_zoom_pan(
        pan,
        old_zoom,
        new_zoom,
        previous_midpoint,
        current_midpoint,
    );
    assert_vec2_approx_eq(new_pan + map_point * new_zoom, current_midpoint);
}

#[test]
fn viewport_resize_recalculates_reset_position_without_saving_view_state() {
    let canvas_size = Vec2::new(1600.0, 900.0);
    let current_location_center = Vec2::new(300.0, 450.0);
    let zoom = 1.0;
    let old_default = world_map_initial_pan(
        canvas_size,
        Vec2::new(900.0, 600.0),
        current_location_center,
        zoom,
    );
    let new_default = world_map_initial_pan(
        canvas_size,
        Vec2::new(600.0, 900.0),
        current_location_center,
        zoom,
    );

    assert_ne!(old_default, new_default);
    assert_vec2_approx_eq(
        world_map_clamp_pan(
            new_default,
            zoom,
            canvas_size,
            Vec2::new(600.0, 900.0),
        ),
        new_default,
    );
}

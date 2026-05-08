use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;

use crate::core::camera::projection_scale;
use crate::features::player::components::{Player, Velocity};
use crate::features::world::solid::Solid;

const FORWARD_SPEED_PX: f32 = 60.0;
const GRAVITY_PX: f32 = 120.0;
const JUMP_SPEED_PX: f32 = 72.0;
const GROUND_EPSILON: f32 = 0.05;

fn sprite_size(sprite: &Sprite) -> Vec2 {
    sprite.custom_size.unwrap_or(Vec2::splat(32.0))
}

fn overlaps_x(center_a: f32, half_a: f32, center_b: f32, half_b: f32) -> bool {
    (center_a - center_b).abs() <= half_a + half_b
}

fn solid_bounds<'a, I>(solids: I) -> Vec<(f32, f32, f32)>
where
    I: IntoIterator<Item = (&'a Transform, &'a Sprite)>,
{
    solids
        .into_iter()
        .map(|(solid_transform, solid_sprite)| {
            let solid_size = sprite_size(solid_sprite);
            let solid_half_w = solid_size.x * 0.5;
            let solid_top = solid_transform.translation.y + solid_size.y * 0.5;
            (solid_transform.translation.x, solid_top, solid_half_w)
        })
        .collect()
}

fn player_bounds(transform: &Transform, sprite: &Sprite) -> (f32, f32, f32, f32) {
    let player_size = sprite_size(sprite);
    let player_half_w = player_size.x * 0.5;
    let player_half_h = player_size.y * 0.5;
    let player_bottom = transform.translation.y - player_half_h;
    (
        transform.translation.x,
        player_bottom,
        player_half_w,
        player_half_h,
    )
}

fn is_grounded(
    player_x: f32,
    player_bottom: f32,
    player_half_w: f32,
    solids: &[(f32, f32, f32)],
) -> bool {
    for (solid_x, solid_top, solid_half_w) in solids.iter().copied() {
        if overlaps_x(player_x, player_half_w, solid_x, solid_half_w)
            && (player_bottom - solid_top).abs() <= GROUND_EPSILON
        {
            return true;
        }
    }

    false
}

fn landing_top(
    next_x: f32,
    previous_bottom: f32,
    next_bottom: f32,
    player_half_w: f32,
    solids: &[(f32, f32, f32)],
) -> Option<f32> {
    let mut landing_top: Option<f32> = None;

    for (solid_x, solid_top, solid_half_w) in solids.iter().copied() {
        if overlaps_x(next_x, player_half_w, solid_x, solid_half_w)
            && previous_bottom >= solid_top - GROUND_EPSILON
            && next_bottom <= solid_top
        {
            landing_top = Some(landing_top.map_or(solid_top, |top| top.max(solid_top)));
        }
    }

    landing_top
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    mut params: ParamSet<(
        Query<(&mut Transform, &Sprite, &mut Velocity), With<Player>>,
        Query<(&Transform, &Sprite), (With<Solid>, Without<Player>)>,
    )>,
) {
    let dt = time.delta_secs();
    let world_units_per_render_pixel =
        projection_scale(camera_projection.into_inner(), 1.0).max(f32::EPSILON);
    let forward_speed = FORWARD_SPEED_PX * world_units_per_render_pixel;
    let gravity = GRAVITY_PX * world_units_per_render_pixel;
    let jump_speed = JUMP_SPEED_PX * world_units_per_render_pixel;
    let solids = solid_bounds(params.p1().iter());

    for (mut transform, sprite, mut velocity) in &mut params.p0() {
        let (player_x, player_bottom, player_half_w, player_half_h) =
            player_bounds(&transform, sprite);

        let grounded = is_grounded(player_x, player_bottom, player_half_w, &solids);
        if grounded && (keyboard.just_pressed(KeyCode::ArrowUp)) {
            velocity.0.y = jump_speed;
        } else if grounded && velocity.0.y < 0.0 {
            velocity.0.y = 0.0;
        }

        velocity.0.x = forward_speed;
        velocity.0.y -= gravity * dt;

        let previous_position = transform.translation;
        let mut next_position = previous_position + velocity.0.extend(0.0) * dt;

        if velocity.0.y <= 0.0 {
            let previous_bottom = previous_position.y - player_half_h;
            let next_bottom = next_position.y - player_half_h;
            if let Some(top) = landing_top(
                next_position.x,
                previous_bottom,
                next_bottom,
                player_half_w,
                &solids,
            ) {
                next_position.y = top + player_half_h;
                velocity.0.y = 0.0;
            }
        }

        transform.translation = next_position;
    }
}

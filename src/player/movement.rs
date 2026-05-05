// gravity, collision, jumping, velocity
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;

use crate::player::cube::{Player, Velocity};
use crate::world::solid::Solid;

pub struct MovementPlugin;

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

fn is_grounded(
    player_transform: &Transform,
    player_sprite: &Sprite,
    solids: &[(f32, f32, f32)],
) -> bool {
    let player_size = sprite_size(player_sprite);
    let player_half_w = player_size.x * 0.5;
    let player_bottom = player_transform.translation.y - player_size.y * 0.5;

    for (solid_x, solid_top, solid_half_w) in solids.iter().copied() {
        if overlaps_x(
            player_transform.translation.x,
            player_half_w,
            solid_x,
            solid_half_w,
        ) && (player_bottom - solid_top).abs() <= GROUND_EPSILON
        {
            return true;
        }
    }

    false
}

fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    mut params: ParamSet<(
        Query<(&mut Transform, &Sprite, &mut Velocity), With<Player>>,
        Query<(&Transform, &Sprite), (With<Solid>, Without<Player>)>,
    )>,
) {
    let dt = time.delta_secs();
    let world_units_per_render_pixel = match camera_projection.into_inner() {
        Projection::Orthographic(ortho) => ortho.scale.max(f32::EPSILON),
        _ => 1.0,
    };
    let forward_speed = FORWARD_SPEED_PX * world_units_per_render_pixel;
    let gravity = GRAVITY_PX * world_units_per_render_pixel;
    let jump_speed = JUMP_SPEED_PX * world_units_per_render_pixel;
    let solids: Vec<(f32, f32, f32)> = params
        .p1()
        .iter()
        .map(|(solid_transform, solid_sprite)| {
            let solid_size = sprite_size(solid_sprite);
            let solid_half_w = solid_size.x * 0.5;
            let solid_top = solid_transform.translation.y + solid_size.y * 0.5;
            (solid_transform.translation.x, solid_top, solid_half_w)
        })
        .collect();

    for (mut transform, sprite, mut velocity) in &mut params.p0() {
        let player_size = sprite_size(sprite);
        let player_half_w = player_size.x * 0.5;
        let player_half_h = player_size.y * 0.5;

        let grounded = is_grounded(&transform, sprite, &solids);
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
            let mut landing_top: Option<f32> = None;

            for (solid_x, solid_top, solid_half_w) in solids.iter().copied() {
                if overlaps_x(next_position.x, player_half_w, solid_x, solid_half_w)
                    && previous_bottom >= solid_top - GROUND_EPSILON
                    && next_bottom <= solid_top
                {
                    landing_top = Some(landing_top.map_or(solid_top, |top| top.max(solid_top)));
                }
            }

            if let Some(top) = landing_top {
                next_position.y = top + player_half_h;
                velocity.0.y = 0.0;
            }
        }

        transform.translation = next_position;
    }
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player);
    }
}

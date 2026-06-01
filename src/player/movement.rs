use crate::config::Config;
use crate::core::camera::world_units_per_pixel;
use crate::core::collision::{
    GROUND_EPSILON, bounds_at, bounds_from_sprite, overlaps_x, overlaps_y,
};
use crate::input::{InputState, just_pressed};
use crate::level::components::Solid;
use crate::level::loading::CurrentWorld;
use crate::player::components::Player;
use crate::player::queries::PlayerQuery;

use bevy::math::bounding::BoundingVolume;

use bevy::math::bounding::Aabb2d;
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;

use ratatui::crossterm::event::KeyCode;
use std::f32::consts::PI;

type SolidSprites<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Sprite), (With<Solid>, Without<Player>)>;

const AIR_SPIN_RADIANS_PER_SECOND: f32 = 8.0;
const VIEWPORT_FLOOR_THICKNESS: f32 = 16.0;

fn viewport_floor(
    camera_transform: &Transform,
    projection: &Projection,
    ratatui_camera: &RatatuiCamera,
) -> Aabb2d {
    let scale = world_units_per_pixel(projection);
    let viewport_size = ratatui_camera.dimensions.as_vec2() * scale;

    let top = camera_transform.translation.y + viewport_size.y * 0.5;

    Aabb2d::new(
        Vec2::new(
            camera_transform.translation.x,
            top - VIEWPORT_FLOOR_THICKNESS * 0.5,
        ),
        Vec2::new(viewport_size.x, VIEWPORT_FLOOR_THICKNESS * 0.5),
    )
}

fn grounded(player: Aabb2d, solids: &[Aabb2d], gravity_dir: f32) -> bool {
    solids.iter().any(|solid| {
        overlaps_x(player, *solid)
            && ((gravity_dir > 0.0 && (player.min.y - solid.max.y).abs() <= GROUND_EPSILON)
                || (gravity_dir < 0.0 && (player.max.y - solid.min.y).abs() <= GROUND_EPSILON))
    })
}

/// Determines if the player just collided horizontally, taking into account gravity direction.
fn resolve_horizontal(
    position: &mut Vec3,
    velocity: &mut Vec2,
    bounds: Aabb2d,
    solids: &[Aabb2d],
    dt: f32,
) {
    if velocity.x <= 0.0 {
        return;
    }

    let next_x = position.x + velocity.x * dt;

    for solid in solids {
        let moved = bounds_at(bounds, Vec2::new(next_x, position.y));

        let hit = bounds.max.x <= solid.min.x + GROUND_EPSILON
            && moved.max.x >= solid.min.x
            && overlaps_y(moved, *solid);

        if hit {
            position.x = solid.min.x - bounds.half_size().x;
            velocity.x = 0.0;
            return;
        }
    }

    position.x = next_x;
}

/// Determines if the player just landed, taking into account gravity direction.
fn resolve_vertical(
    position: &mut Vec3,
    velocity: &mut Vec2,
    bounds: Aabb2d,
    solids: &[Aabb2d],
    gravity_dir: f32,
    dt: f32,
) -> bool {
    let next_y = position.y + velocity.y * dt;

    for solid in solids {
        let moved = bounds_at(bounds, Vec2::new(position.x, next_y));

        let floor_hit = if gravity_dir > 0.0 {
            velocity.y <= 0.0
                && overlaps_x(moved, *solid)
                && bounds.min.y >= solid.max.y - GROUND_EPSILON
                && moved.min.y <= solid.max.y
        } else {
            velocity.y >= 0.0
                && overlaps_x(moved, *solid)
                && bounds.max.y <= solid.min.y + GROUND_EPSILON
                && moved.max.y >= solid.min.y
        };

        if floor_hit {
            position.y = if gravity_dir > 0.0 {
                solid.max.y + bounds.half_size().y
            } else {
                solid.min.y - bounds.half_size().y
            };

            velocity.y = 0.0;

            return true;
        }

        let ceiling_hit = if gravity_dir > 0.0 {
            velocity.y > 0.0
                && overlaps_x(moved, *solid)
                && bounds.max.y <= solid.min.y + GROUND_EPSILON
                && moved.max.y >= solid.min.y
        } else {
            velocity.y < 0.0
                && overlaps_x(moved, *solid)
                && bounds.min.y >= solid.max.y - GROUND_EPSILON
                && moved.min.y <= solid.max.y
        };

        if ceiling_hit {
            position.y = if gravity_dir > 0.0 {
                solid.min.y - bounds.half_size().y
            } else {
                solid.max.y + bounds.half_size().y
            };

            velocity.y = 0.0;

            return false;
        }
    }

    position.y = next_y;

    false
}

fn grounded_rotation(gravity_dir: f32) -> Quat {
    Quat::from_rotation_z(if gravity_dir > 0.0 { 0.0 } else { PI })
}

pub fn move_player(
    config: Res<Config>,
    time: Res<Time>,
    input_state: Res<InputState>,
    current_world: Res<CurrentWorld>,
    camera: Single<
        (&Transform, &Projection, &RatatuiCamera),
        (With<RatatuiCamera>, Without<Player>),
    >,
    queries: (SolidSprites, PlayerQuery),
) {
    // Scale to frame time
    let dt = time.delta_secs();

    let (camera_transform, projection, ratatui_camera) = camera.into_inner();

    let scale = world_units_per_pixel(projection);

    let forward_speed = current_world
        .definition
        .as_ref()
        .map(|world| world.scroll_speed_px)
        .unwrap_or(config.player.forward_speed_px)
        * scale;

    let gravity = config.player.gravity_px * scale;
    let jump_speed = config.player.jump_speed_px * scale;

    let (solid_sprites, player) = queries;

    let mut solids = solid_sprites
        .iter()
        .map(|(transform, sprite)| bounds_from_sprite(transform, sprite))
        .collect::<Vec<_>>();

    solids.push(viewport_floor(camera_transform, projection, ratatui_camera));

    let wants_jump = just_pressed(&input_state, KeyCode::Up);

    let (mut transform, sprite, mut velocity, player) = player.into_inner();

    let gravity_dir = player.gravity_dir;

    let mut position = transform.translation;

    velocity.0.x = forward_speed;
    velocity.0.y -= gravity * gravity_dir * dt;

    let bounds = bounds_from_sprite(&Transform::from_translation(position), sprite);

    let is_grounded = grounded(bounds, &solids, gravity_dir);

    if is_grounded && wants_jump {
        velocity.0.y = jump_speed * gravity_dir;
    }

    resolve_horizontal(&mut position, &mut velocity.0, bounds, &solids, dt);

    let landed = resolve_vertical(
        &mut position,
        &mut velocity.0,
        bounds,
        &solids,
        gravity_dir,
        dt,
    );

    transform.translation = position;

    if landed || grounded(bounds_at(bounds, position.xy()), &solids, gravity_dir) {
        transform.rotation = grounded_rotation(gravity_dir);
    } else {
        transform.rotate_z(AIR_SPIN_RADIANS_PER_SECOND * gravity_dir * dt);
    }
}

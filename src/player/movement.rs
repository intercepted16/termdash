use crate::config::Config;
use crate::core::camera::world_units_per_pixel;
use crate::core::collision::{GROUND_EPSILON, overlaps_y};
use crate::core::collision::{bounds_at, bounds_from_sprite, overlaps_x};
use crate::player::components::Player;
use crate::player::jump_pressed;
use crate::player::queries::Players;
use crate::world::components::Solid;
use crate::world::loading::CurrentWorld;
use bevy::math::bounding::{Aabb2d, BoundingVolume};
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui_camera::RatatuiCamera;

type SolidSprites<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Sprite), (With<Solid>, Without<Player>)>;

const AIR_SPIN_RADIANS_PER_SECOND: f32 = 8.0;

fn solid_bounds<'a, I>(solids: I) -> Vec<Aabb2d>
where
    I: IntoIterator<Item = (&'a Transform, &'a Sprite)>,
{
    solids
        .into_iter()
        .map(|(transform, sprite)| bounds_from_sprite(transform, sprite))
        .collect()
}
fn collision_surface(
    solids: &[Aabb2d],
    matches: impl Fn(Aabb2d) -> bool,
    surface: impl Fn(Aabb2d) -> f32,
    nearest: fn(f32, f32) -> f32,
) -> Option<f32> {
    solids
        .iter()
        .copied()
        .filter(|solid| matches(*solid))
        .map(surface)
        .reduce(nearest)
}
fn player_on_ground(player: Aabb2d, solids: &[Aabb2d]) -> bool {
    solids.iter().copied().any(|solid| {
        overlaps_x(player, solid) && (player.min.y - solid.max.y).abs() <= GROUND_EPSILON
    })
}
pub fn move_player(
    config: Res<Config>,
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    current_world: Res<CurrentWorld>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    queries: (Players, SolidSprites),
) {
    let (mut players, solid_sprites) = queries;
    let dt = time.delta_secs();
    let world_units_per_pixel = world_units_per_pixel(camera_projection.into_inner());
    let forward_speed_px = current_world
        .definition
        .as_ref()
        .map(|world| world.scroll_speed_px)
        .unwrap_or(config.player.forward_speed_px);
    let forward_speed = forward_speed_px * world_units_per_pixel;
    let gravity = config.player.gravity_px * world_units_per_pixel;
    let jump_speed = config.player.jump_speed_px * world_units_per_pixel;
    let solids = solid_bounds(solid_sprites.iter());
    let wants_jump = jump_pressed(&mut keys);
    for (mut transform, sprite, mut velocity) in &mut players {
        let player = bounds_from_sprite(&transform, sprite);
        let grounded = player_on_ground(player, &solids);
        if grounded && wants_jump {
            velocity.0.y = jump_speed;
        } else if grounded && velocity.0.y < 0.0 {
            velocity.0.y = 0.0;
        }
        velocity.0.x = forward_speed;
        velocity.0.y -= gravity * dt;
        let previous_position = transform.translation;
        let mut next_position = previous_position + velocity.0.extend(0.0) * dt;
        if velocity.0.x > 0.0
            && let Some(edge) = collision_surface(
                &solids,
                |solid| {
                    let next_player =
                        bounds_at(player, Vec2::new(next_position.x, previous_position.y));
                    player.max.x <= solid.min.x + GROUND_EPSILON
                        && next_player.max.x >= solid.min.x
                        && overlaps_y(next_player, solid)
                },
                |bounds| bounds.min.x,
                f32::min,
            )
        {
            next_position.x = edge - player.half_size().x;
            velocity.0.x = 0.0;
        }
        if velocity.0.y <= 0.0 {
            if let Some(top) = collision_surface(
                &solids,
                |solid| {
                    let next_player = bounds_at(player, next_position.xy());
                    overlaps_x(next_player, solid)
                        && player.min.y >= solid.max.y - GROUND_EPSILON
                        && next_player.min.y <= solid.max.y
                },
                |bounds| bounds.max.y,
                f32::max,
            ) {
                next_position.y = top + player.half_size().y;
                velocity.0.y = 0.0;
            }
        } else if let Some(bottom) = collision_surface(
            &solids,
            |solid| {
                let next_player = bounds_at(player, next_position.xy());
                overlaps_x(next_player, solid)
                    && player.max.y <= solid.min.y + GROUND_EPSILON
                    && next_player.max.y >= solid.min.y
            },
            |bounds| bounds.min.y,
            f32::min,
        ) {
            next_position.y = bottom - player.half_size().y;
            velocity.0.y = 0.0;
        }
        transform.translation = next_position;
        if player_on_ground(bounds_from_sprite(&transform, sprite), &solids) {
            transform.rotation = Quat::IDENTITY;
        } else {
            transform.rotate_z(AIR_SPIN_RADIANS_PER_SECOND * time.delta_secs());
        }
    }
}

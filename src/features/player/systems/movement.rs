use crate::core::camera::projection_scale;
use crate::features::player::components::Player;
use crate::features::player::systems::hazard::PlayerDeathState;
use crate::features::player::systems::physics::{bounds_at, bounds_from_sprite, overlaps_x};
use crate::features::player::systems::queries::Players;
use crate::features::world::components::Solid;
use crate::features::world::loading::CurrentWorld;
use bevy::math::bounding::{Aabb2d, BoundingVolume};
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui_camera::RatatuiCamera;
use ratatui::crossterm::event::{KeyCode as TerminalKeyCode, KeyEventKind};
const FORWARD_SPEED_PX: f32 = 100.0;
const GRAVITY_PX: f32 = 300.0;
const JUMP_SPEED_PX: f32 = 120.0;
const GROUND_EPSILON: f32 = 0.05;
const AIR_SPIN_RADIANS_PER_SECOND: f32 = 8.0;
type SolidSprites<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Sprite), (With<Solid>, Without<Player>)>;
fn overlaps_y(a: Aabb2d, b: Aabb2d) -> bool {
    a.min.y < b.max.y - GROUND_EPSILON && a.max.y > b.min.y + GROUND_EPSILON
}
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
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    death_state: Res<PlayerDeathState>,
    current_world: Res<CurrentWorld>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    mut params: ParamSet<(Players, SolidSprites)>,
) {
    if death_state.is_active() {
        keys.clear();
        return;
    }
    let dt = time.delta_secs();
    let world_units_per_render_pixel =
        projection_scale(camera_projection.into_inner(), 1.0).max(f32::EPSILON);
    let forward_speed_px = current_world
        .definition
        .as_ref()
        .map(|world| world.scroll_speed_px)
        .unwrap_or(FORWARD_SPEED_PX);
    let forward_speed = forward_speed_px * world_units_per_render_pixel;
    let gravity = GRAVITY_PX * world_units_per_render_pixel;
    let jump_speed = JUMP_SPEED_PX * world_units_per_render_pixel;
    let solids = solid_bounds(params.p1().iter());
    let jump_pressed = keys.read().any(|key| {
        matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            && key.code == TerminalKeyCode::Up
    });
    for (mut transform, sprite, mut velocity) in &mut params.p0() {
        let player = bounds_from_sprite(&transform, sprite);
        let grounded = player_on_ground(player, &solids);
        if grounded && jump_pressed {
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

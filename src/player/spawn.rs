use bevy::prelude::*;

use crate::player::cube::{NeedsGroundStart, Player};
use crate::world::ground::Ground;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, start_player_on_ground);
    }
}

// find the pos of the highest ground under the player, and move the player there
fn start_player_on_ground(
    mut commands: Commands,
    ground: Query<(&Transform, &Sprite), (With<Ground>, Without<Player>)>,
    mut player: Query<
        (Entity, &mut Transform, &Sprite),
        (With<Player>, With<NeedsGroundStart>, Without<Ground>),
    >,
) {
    for (entity, mut player_transform, player_sprite) in &mut player {
        let player_height = player_sprite.custom_size.unwrap_or(Vec2::splat(32.0)).y;
        let player_x = player_transform.translation.x;
        let player_half_width = player_sprite.custom_size.unwrap_or(Vec2::splat(32.0)).x * 0.5;

        let mut best_ground_top = None;
        for (ground_transform, ground_sprite) in &ground {
            let ground_size = ground_sprite.custom_size.unwrap_or(Vec2::splat(32.0));
            let ground_half_width = ground_size.x * 0.5;
            let overlaps_x = (player_x - ground_transform.translation.x).abs()
                <= ground_half_width + player_half_width;

            if overlaps_x {
                let ground_top = ground_transform.translation.y + ground_size.y * 0.5;
                best_ground_top =
                    Some(best_ground_top.map_or(ground_top, |y: f32| y.max(ground_top)));
            }
        }

        if let Some(ground_top) = best_ground_top {
            player_transform.translation.y = ground_top + player_height * 0.5;
            commands.entity(entity).remove::<NeedsGroundStart>();
        }
    }
}

use crate::core::camera::world_units_per_pixel;
use crate::core::collision::bounds_from_sprite;
use crate::gameplay::death::PlayerDeathState;
use crate::player::components::Player;
use crate::player::jump_pressed;
use crate::player::queries::Players;
use crate::world::components::JumpOrb;
use bevy::math::bounding::{BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui_camera::RatatuiCamera;

type JumpOrbs<'w, 's> = Query<'w, 's, (&'static Transform, &'static JumpOrb), Without<Player>>;

pub fn activate_jump_orbs(
    mut keys: MessageReader<KeyMessage>,
    death_state: Res<PlayerDeathState>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    mut players: Players,
    orbs: JumpOrbs,
) {
    if death_state.is_active() {
        return;
    }

    if !jump_pressed(&mut keys) {
        return;
    }

    let world_units_per_pixel = world_units_per_pixel(camera_projection.into_inner());

    for (player_transform, player_sprite, mut velocity) in &mut players {
        let player = bounds_from_sprite(&player_transform, player_sprite);

        for (orb_transform, orb) in &orbs {
            if player.intersects(&BoundingCircle::new(
                orb_transform.translation.xy(),
                orb.radius,
            )) {
                velocity.0.y = velocity.0.y.max(orb.strength_px * world_units_per_pixel);
            }
        }
    }
}

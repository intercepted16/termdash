use crate::core::camera::world_units_per_pixel;
use crate::core::collision::bounds_from_sprite;
use crate::gameplay::death::{KillPlayerEvent, completion_percent};
use crate::player::components::Player;
use crate::player::jump_pressed;
use crate::player::queries::Players;
use crate::world::components::{PlayerTrigger, TriggerActivation, TriggerEffect, TriggerShape};
use crate::world::loading::CurrentWorld;
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui_camera::RatatuiCamera;

type PlayerTriggers<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static PlayerTrigger), Without<Player>>;

pub fn apply_player_triggers(
    current_world: Res<CurrentWorld>,
    mut keys: MessageReader<KeyMessage>,
    mut deaths: MessageWriter<KillPlayerEvent>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    queries: (Players, PlayerTriggers),
) {
    let (mut players, triggers) = queries;

    let Some(world) = current_world.definition.as_ref() else {
        return;
    };

    let jump_pressed = jump_pressed(&mut keys);
    let world_units_per_pixel = world_units_per_pixel(camera_projection.into_inner());

    for (player_transform, player_sprite, mut velocity) in &mut players {
        let player = bounds_from_sprite(&player_transform, player_sprite);

        for (trigger_transform, trigger) in &triggers {
            if !trigger.activation.is_active(jump_pressed) {
                continue;
            }

            if trigger.shape.intersects_player(player, trigger_transform) {
                match trigger.effect {
                    TriggerEffect::SetMinVerticalSpeedPx(speed_px) => {
                        velocity.0.y = velocity.0.y.max(speed_px * world_units_per_pixel);
                    }
                    TriggerEffect::KillPlayer => {
                        deaths.write(KillPlayerEvent {
                            percent: completion_percent(player_transform.translation.x, world),
                        });
                    }
                }
            }
        }
    }
}

impl TriggerActivation {
    fn is_active(self, jump_pressed: bool) -> bool {
        match self {
            Self::Touch => true,
            Self::JumpPressed => jump_pressed,
        }
    }
}

impl TriggerShape {
    fn intersects_player(self, player: Aabb2d, trigger_transform: &Transform) -> bool {
        let center = trigger_transform.translation.xy();

        match self {
            Self::Circle { radius } => player.intersects(&BoundingCircle::new(center, radius)),
            Self::Rect { half_size } => player.intersects(&Aabb2d::new(center, half_size)),
        }
    }
}

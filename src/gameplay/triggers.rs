use crate::core::camera::world_units_per_pixel;
use crate::core::collision::bounds_from_sprite;
use crate::gameplay::death::{KillPlayerEvent, completion_percent};
use crate::input::{InputState, just_pressed};
use crate::player::components::Player;
use crate::player::queries::PlayerQuery;
use crate::world::components::{PlayerTrigger, TriggerActivation, TriggerEffect, TriggerShape};
use crate::world::loading::CurrentWorld;
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;
use std::collections::HashSet;

type PlayerTriggers<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform, &'static PlayerTrigger), Without<Player>>;

#[derive(Resource, Default)]
pub struct TriggerState {
    active: HashSet<Entity>,
}

impl TriggerState {
    pub fn clear(&mut self) {
        self.active.clear();
    }
}

pub fn just_entered(trigger_state: &mut TriggerState, entity: Entity, active: bool) -> bool {
    if active {
        trigger_state.active.insert(entity)
    } else {
        trigger_state.active.remove(&entity);
        false
    }
}

pub fn apply_player_triggers(
    current_world: Res<CurrentWorld>,
    mut deaths: MessageWriter<KillPlayerEvent>,
    input: Res<InputState>,
    mut trigger_state: ResMut<TriggerState>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    queries: (PlayerQuery, PlayerTriggers),
) {
    let (player, triggers) = queries;

    let Some(world) = current_world.definition.as_ref() else {
        return;
    };

    let jump_pressed = just_pressed(&input, TerminalKeyCode::Up);
    let world_units_per_pixel = world_units_per_pixel(camera_projection.into_inner());

    let (player_transform, player_sprite, mut velocity, mut player) = player.into_inner();
    let player_bounds = bounds_from_sprite(&player_transform, player_sprite);

    for (trigger_entity, trigger_transform, trigger) in &triggers {
        if !trigger.activation.is_active(jump_pressed) {
            continue;
        }

        let intersects_player = trigger
            .shape
            .intersects_player(player_bounds, trigger_transform);

        let just_entered = just_entered(
            &mut trigger_state,
            trigger_entity,
            intersects_player && trigger.effect.enters_once(),
        );

        if !intersects_player {
            continue;
        }

        match trigger.effect {
            TriggerEffect::SetMinVerticalSpeedPx(speed_px) => {
                let mut local_velocity_y = velocity.0.y * player.gravity_dir;
                local_velocity_y = local_velocity_y.max(speed_px * world_units_per_pixel);
                velocity.0.y = local_velocity_y * player.gravity_dir;
            }
            TriggerEffect::KillPlayer => {
                deaths.write(KillPlayerEvent {
                    percent: completion_percent(player_transform.translation.x, world),
                });
            }
            TriggerEffect::FlipGravity => {
                if !just_entered {
                    continue;
                }

                player.gravity_dir *= -1.0;
            }
        }
    }
}

impl TriggerEffect {
    fn enters_once(self) -> bool {
        matches!(self, Self::FlipGravity)
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

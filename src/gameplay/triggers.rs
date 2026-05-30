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

fn intersects(shape: &TriggerShape, player: Aabb2d, transform: &Transform) -> bool {
    let center = transform.translation.xy();

    match shape {
        TriggerShape::Circle { radius } => player.intersects(&BoundingCircle::new(center, *radius)),

        TriggerShape::Rect { half_size } => player.intersects(&Aabb2d::new(center, *half_size)),
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

    for (entity, transform, trigger) in &triggers {
        let activation_allowed = match trigger.activation {
            TriggerActivation::Touch => true,
            TriggerActivation::JumpPressed => jump_pressed,
        };

        if !activation_allowed {
            continue;
        }

        let intersects = intersects(&trigger.shape, player_bounds, transform);

        if !intersects {
            trigger_state.active.remove(&entity);
            continue;
        }

        let just_entered = trigger_state.active.insert(entity);

        match trigger.effect {
            TriggerEffect::SetMinVerticalSpeedPx(speed_px) => {
                let current = velocity.0.y * player.gravity_dir;
                let minimum = speed_px * world_units_per_pixel;

                velocity.0.y = current.max(minimum) * player.gravity_dir;
            }

            TriggerEffect::KillPlayer => {
                deaths.write(KillPlayerEvent {
                    percent: completion_percent(player_transform.translation.x, world),
                });
            }

            TriggerEffect::FlipGravity => {
                if just_entered {
                    player.gravity_dir *= -1.0;
                }
            }
        }
    }
}

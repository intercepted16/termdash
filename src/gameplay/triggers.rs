use crate::core::camera::world_units_per_pixel;
use crate::core::collision::bounds_from_sprite;
use crate::gameplay::death::{KillPlayerEvent, completion_percent};
use crate::input::{InputState, just_pressed};
use crate::level::loading::CurrentWorld;
use crate::level::model::ObjectShape;
use crate::player::components::Player;
use crate::player::queries::PlayerQuery;
use bevy::math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume};
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Clone, Copy, Component, Debug)]
pub struct PlayerTrigger {
    pub activation: TriggerActivation,
    pub shape: ObjectShape,
    pub effect: TriggerEffect,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TriggerActivation {
    Touch,
    JumpPressed,
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerEffect {
    SetMinVerticalSpeedPx { speed_px: f32 },
    KillPlayer,
    FlipGravity,
}

type PlayerTriggers<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform, &'static PlayerTrigger), Without<Player>>;

#[derive(Resource, Default)]
pub struct TriggerState(pub HashSet<Entity>);

impl ObjectShape {
    fn intersects(self, player: Aabb2d, transform: &Transform) -> bool {
        let center = transform.translation.xy();
        match self {
            ObjectShape::Circle { radius } => {
                player.intersects(&BoundingCircle::new(center, radius))
            }
            ObjectShape::Rect { size } => player.intersects(&Aabb2d::new(center, size / 2.0)),
            // TODO: maybe use real triangles, although, for geometry dash, maybe this is OK?
            ObjectShape::Triangle { size } => player.intersects(&Aabb2d::new(center, size / 2.0)),
        }
    }
}

pub fn apply_player_triggers(
    current_world: Res<CurrentWorld>,
    mut deaths: MessageWriter<KillPlayerEvent>,
    input: Res<InputState>,
    mut state: ResMut<TriggerState>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    (player, triggers): (PlayerQuery, PlayerTriggers),
) {
    let Some(world) = current_world.definition.as_ref() else {
        return;
    };

    let jump_pressed = just_pressed(&input, TerminalKeyCode::Up);
    let world_units_per_pixel = world_units_per_pixel(camera_projection.into_inner());
    let (player_transform, player_sprite, mut velocity, mut player) = player.into_inner();
    let player_bounds = bounds_from_sprite(&player_transform, player_sprite);

    for (entity, transform, trigger) in &triggers {
        let active = match trigger.activation {
            TriggerActivation::Touch => true,
            TriggerActivation::JumpPressed => jump_pressed,
        };

        if !active || !trigger.shape.intersects(player_bounds, transform) {
            state.0.remove(&entity);
            continue;
        }

        let just_entered = state.0.insert(entity);

        match trigger.effect {
            TriggerEffect::SetMinVerticalSpeedPx { speed_px } => {
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

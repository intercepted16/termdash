use crate::core::camera::level_units_per_pixel;
use crate::gameplay::death::{KillPlayer, completion_percent};
use crate::input::InputState;
use crate::level::load::CurrentLevel;
use crate::player::components::Player;
use crate::player::queries::PlayerQuery;
use avian2d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Clone, Copy, Component, Debug)]
pub struct PlayerTrigger {
    pub activation: TriggerActivation,
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

type PlayerTriggers<'w, 's> = Query<'w, 's, &'static PlayerTrigger, Without<Player>>;

#[derive(Resource, Default)]
pub struct TriggerState(pub HashSet<Entity>);

pub fn apply_player_triggers(
    current_world: Res<CurrentLevel>,
    mut deaths: MessageWriter<KillPlayer>,
    input: Res<InputState>,
    mut state: ResMut<TriggerState>,
    camera_projection: Single<&Projection, With<RatatuiCamera>>,
    mut spatial_query: SpatialQuery,
    (player, triggers): (PlayerQuery, PlayerTriggers),
) {
    let Some(world) = current_world.0.as_ref() else {
        return;
    };

    let jump_pressed = input.just_pressed(TerminalKeyCode::Up);
    let world_units_per_pixel = level_units_per_pixel(camera_projection.into_inner());
    let (player_entity, player_transform, player_collider, mut velocity, mut player) =
        player.into_inner();
    let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

    spatial_query.update_pipeline();

    let hit_entities = spatial_query
        .shape_intersections(
            player_collider,
            player_transform.translation.xy(),
            player_transform.rotation.to_euler(EulerRot::XYZ).2,
            &filter,
        )
        .into_iter()
        .filter(|entity| triggers.contains(*entity))
        .collect::<HashSet<_>>();

    state.0.retain(|entity| hit_entities.contains(entity));

    for entity in hit_entities {
        let Ok(trigger) = triggers.get(entity) else {
            continue;
        };

        let active = match trigger.activation {
            TriggerActivation::Touch => true,
            TriggerActivation::JumpPressed => jump_pressed,
        };

        if !active {
            state.0.remove(&entity);
            continue;
        }

        let just_entered = state.0.insert(entity);

        match trigger.effect {
            TriggerEffect::SetMinVerticalSpeedPx { speed_px } => {
                let away_from_gravity = -player.gravity_dir.as_vec2();
                let current = velocity.0.dot(away_from_gravity);
                let minimum = speed_px * world_units_per_pixel;
                velocity.0 += away_from_gravity * (current.max(minimum) - current);
            }
            TriggerEffect::KillPlayer => {
                deaths.write(KillPlayer {
                    percent: completion_percent(player_transform.translation.x, world),
                });
            }
            TriggerEffect::FlipGravity => {
                if just_entered {
                    player.gravity_dir = Dir2::new(-player.gravity_dir.as_vec2())
                        .expect("gravity direction must stay normalized");
                }
            }
        }
    }
}

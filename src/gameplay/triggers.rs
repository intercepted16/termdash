use crate::config::Config;
use crate::gameplay::death::KillPlayer;
use crate::input::InputState;
use crate::level::load::CurrentLevel;
use crate::player::components::Player;
use crate::player::queries::PlayerQuery;
use avian2d::collision::collider::contact_query;
use avian2d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use level_data_macros::level_data;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[level_data(Component)]
pub struct PlayerTrigger {
    pub activation: TriggerActivation,
    pub effect: TriggerEffect,
}

#[level_data(PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerActivation {
    Touch,
    TouchOnSide,
    JumpPressed,
}

#[level_data(PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerEffect {
    SetMinVerticalSpeedPx { speed_px: f32 },
    KillPlayer,
    FlipGravity,
    MultiplyScrollSpeed { multiplier: f32 },
}

type PlayerTriggers<'w, 's> = Query<
    'w,
    's,
    (
        &'static PlayerTrigger,
        &'static Transform,
        &'static avian2d::prelude::Collider,
    ),
    Without<Player>,
>;

#[derive(Resource, Default)]
pub struct TriggerState(pub HashSet<Entity>);

const SIDE_CONTACT_MIN_ACROSS_GRAVITY: f32 = 0.25;

fn trigger_rotation(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::XYZ).2
}

fn touch_on_side(
    player_collider: &avian2d::prelude::Collider,
    player_transform: &Transform,
    player: &Player,
    trigger_collider: &avian2d::prelude::Collider,
    trigger_transform: &Transform,
) -> bool {
    let Ok(Some(contact)) = contact_query::contact(
        player_collider,
        player_transform.translation.xy(),
        trigger_rotation(player_transform),
        trigger_collider,
        trigger_transform.translation.xy(),
        trigger_rotation(trigger_transform),
        0.0,
    ) else {
        return false;
    };

    let across_gravity = player.gravity_dir.perp();
    let normal = player_transform.rotation * contact.local_normal1.extend(0.0);

    normal.xy().dot(across_gravity).abs() > SIDE_CONTACT_MIN_ACROSS_GRAVITY
}

pub fn apply_player_triggers(
    mut deaths: MessageWriter<KillPlayer>,
    input: Res<InputState>,
    mut state: ResMut<TriggerState>,
    mut spatial_query: SpatialQuery,
    (player, triggers): (PlayerQuery, PlayerTriggers),
    mut current_level: ResMut<CurrentLevel>,
    config: Res<Config>,
) {
    let jump_pressed = input.just_pressed(TerminalKeyCode::Up);
    let world_units_per_pixel = config.camera.zoom;
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
        let Ok((trigger, trigger_transform, trigger_collider)) = triggers.get(entity) else {
            continue;
        };

        let active = match trigger.activation {
            TriggerActivation::Touch => true,
            TriggerActivation::JumpPressed => jump_pressed,
            TriggerActivation::TouchOnSide => touch_on_side(
                player_collider,
                &player_transform,
                &player,
                trigger_collider,
                trigger_transform,
            ),
        };

        if !active {
            state.0.remove(&entity);
            continue;
        }

        // Only run on first collision on an entity, otherwise... well.. its buggy asf
        let just_entered = state.0.insert(entity);
        if !just_entered {
            continue;
        }

        match trigger.effect {
            TriggerEffect::SetMinVerticalSpeedPx { speed_px } => {
                let away_from_gravity = -player.gravity_dir.as_vec2();
                let current = velocity.0.dot(away_from_gravity);
                let minimum = speed_px * world_units_per_pixel;
                velocity.0 += away_from_gravity * (current.max(minimum) - current);
            }
            TriggerEffect::KillPlayer => {
                deaths.write(KillPlayer);
            }
            TriggerEffect::FlipGravity => {
                player.gravity_dir = -player.gravity_dir;
            }
            TriggerEffect::MultiplyScrollSpeed { multiplier } => {
                current_level.level.as_mut().unwrap().scroll_speed_px *= multiplier;
            }
        }
    }
}

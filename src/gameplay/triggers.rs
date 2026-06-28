use crate::config::Config;
use crate::gameplay::death::KillPlayer;
use crate::input::InputState;
use crate::player::components::Player;
use crate::player::components::Velocity;
use avian2d::collision::collider::contact_query;
use avian2d::prelude::{Collider, SpatialQuery, SpatialQueryFilter};
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
    SetMinVerticalSpeedPx {
        speed_px: f32,
    },
    KillPlayer,
    FlipGravity {
        #[serde(default)]
        min_vertical_speed_px: Option<f32>,
    },
    MultiplyScrollSpeed {
        multiplier: f32,
    },
    SetScrollSpeedMultiplier {
        multiplier: f32,
    },
    SetGravityUp,
    SetGravityDown,
    SetPlayerScale {
        scale: f32,
    },
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

type TriggerPlayerQuery<'w, 's> = Single<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static Collider,
        &'static mut Sprite,
        &'static mut Velocity,
        &'static mut Player,
    ),
    With<Player>,
>;

#[derive(Resource, Default)]
pub struct TriggerState(pub HashSet<Entity>);

const SIDE_CONTACT_MIN_ACROSS_GRAVITY: f32 = 0.25;

fn trigger_rotation(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::XYZ).2
}

fn touch_on_side(
    player_collider: &Collider,
    player_transform: &Transform,
    player: &Player,
    trigger_collider: &Collider,
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

fn trigger_active(
    trigger: &PlayerTrigger,
    jump_pressed: bool,
    player: (&Collider, &Transform, &Player),
    trigger_body: (&Collider, &Transform),
) -> bool {
    match trigger.activation {
        TriggerActivation::Touch => true,
        TriggerActivation::JumpPressed => jump_pressed,
        TriggerActivation::TouchOnSide => {
            let (player_collider, player_transform, player) = player;
            let (trigger_collider, trigger_transform) = trigger_body;
            touch_on_side(
                player_collider,
                player_transform,
                player,
                trigger_collider,
                trigger_transform,
            )
        }
    }
}

impl TriggerEffect {
    fn apply(
        &self,
        target: (&mut Player, &mut Velocity, &mut Transform, &mut Sprite),
        level_units_per_pixel: f32,
        deaths: &mut MessageWriter<KillPlayer>,
    ) -> Option<Collider> {
        let (player, velocity, transform, sprite) = target;

        match self {
            Self::SetMinVerticalSpeedPx { speed_px } => {
                player.add_vertical_speed(velocity, *speed_px, level_units_per_pixel);
            }
            Self::KillPlayer => {
                deaths.write(KillPlayer);
            }
            Self::FlipGravity {
                min_vertical_speed_px,
            } => {
                player.gravity_dir = -player.gravity_dir;
                if let Some(speed_px) = min_vertical_speed_px {
                    player.add_vertical_speed(velocity, *speed_px, level_units_per_pixel);
                }
            }
            Self::MultiplyScrollSpeed { multiplier } => {
                player.scroll_speed_multiplier *= *multiplier;
            }
            Self::SetScrollSpeedMultiplier { multiplier } => {
                player.scroll_speed_multiplier = *multiplier;
            }
            Self::SetGravityUp => player.gravity_dir = Dir2::Y,
            Self::SetGravityDown => player.gravity_dir = Dir2::NEG_Y,
            Self::SetPlayerScale { scale } => {
                return player.set_size_scale(transform, sprite, *scale);
            }
        }

        None
    }
}

pub fn apply_player_triggers(
    side_effects: (Commands, MessageWriter<KillPlayer>),
    input: Res<InputState>,
    mut state: ResMut<TriggerState>,
    mut spatial_query: SpatialQuery,
    (player, triggers): (TriggerPlayerQuery, PlayerTriggers),
    config: Res<Config>,
) {
    let (mut commands, mut deaths) = side_effects;
    let jump_pressed = input.just_pressed(TerminalKeyCode::Up);
    let level_units_per_pixel = config.camera.zoom;

    let (player_entity, mut transform, collider, mut sprite, mut velocity, mut player) =
        player.into_inner();
    let filter = SpatialQueryFilter::from_excluded_entities([player_entity]);

    spatial_query.update_pipeline();

    let hit_entities = spatial_query
        .shape_intersections(
            collider,
            transform.translation.xy(),
            trigger_rotation(&transform),
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

        if !trigger_active(
            trigger,
            jump_pressed,
            (collider, &transform, &player),
            (trigger_collider, trigger_transform),
        ) {
            state.0.remove(&entity);
            continue;
        }

        // Only run once while the player remains inside the same trigger.
        if !state.0.insert(entity) {
            continue;
        }

        let pending_collider = trigger.effect.apply(
            (&mut player, &mut velocity, &mut transform, &mut sprite),
            level_units_per_pixel,
            &mut deaths,
        );
        if let Some(collider) = pending_collider {
            commands.entity(player_entity).insert(collider);
        }
    }
}

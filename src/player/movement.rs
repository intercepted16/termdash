//! Handles X and Y forward movement, side collisions (and deaths due to this); which is applicable to all objects.
//! Does not handle death for objects that are 'hazards'; i.e., they kill on any collision.
//! That is handled in triggers.rs.
use crate::config::Config;
use crate::gameplay::death::KillPlayer;
use crate::input::InputState;
use crate::level::load::CurrentLevel;
use crate::level::model::{KillPlayerOnSide, Level, Solid};
use crate::player::queries::PlayerQuery;

use avian2d::prelude::{Collider, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use crossterm::event::KeyCode as TerminalKeyCode;

use std::f32::consts::PI;

type MovementQueries<'w, 's> = (
    Query<'w, 's, (), With<Solid>>,
    Query<'w, 's, (), With<KillPlayerOnSide>>,
    PlayerQuery<'w, 's>,
);

const AIR_SPIN_RADIANS_PER_SECOND: f32 = 8.0;
const CONTACT_EPSILON: f32 = 0.05;
const GROUND_PROBE_DISTANCE: f32 = 2.0;
const GROUNDED_GRACE_SECONDS: f32 = 0.1;
const JUMP_BUFFER_SECONDS: f32 = 0.1;
const BLOCKING_NORMAL_DOT: f32 = 0.5;
const MAX_SHAPE_HITS: u32 = 8;

fn rotation_z(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::XYZ).2
}

fn solid_filter(player_entity: Entity) -> SpatialQueryFilter {
    SpatialQueryFilter::from_excluded_entities([player_entity])
}

fn move_and_collide(
    spatial_query: &SpatialQuery,
    solids: &Query<(), With<Solid>>,
    player_entity: Entity,
    collider: &Collider,
    transform: &mut Transform,
    delta: Vec2,
    ignore_origin_penetration: bool,
) -> Option<Entity> {
    let direction = Dir2::new(delta).ok()?;

    let config = ShapeCastConfig {
        ignore_origin_penetration,
        ..ShapeCastConfig::from_max_distance(delta.length())
    };

    let blocking_normal = -direction.as_vec2();

    let hit = spatial_query
        .shape_hits(
            collider,
            transform.translation.xy(),
            rotation_z(transform),
            direction,
            MAX_SHAPE_HITS,
            &config,
            &solid_filter(player_entity),
        )
        .into_iter()
        .find(|hit| {
            solids.contains(hit.entity) && hit.normal1.dot(blocking_normal) >= BLOCKING_NORMAL_DOT
        });

    let travel = hit
        .as_ref()
        .map(|hit| (hit.distance - CONTACT_EPSILON).max(0.0))
        .unwrap_or(delta.length());

    transform.translation += (direction.as_vec2() * travel).extend(0.0);

    hit.map(|hit| hit.entity)
}

fn touching_ground(
    spatial_query: &SpatialQuery,
    solids: &Query<(), With<Solid>>,
    player_entity: Entity,
    collider: &Collider,
    transform: &Transform,
    gravity_dir: Dir2,
) -> bool {
    let supporting_normal = -gravity_dir.as_vec2();

    spatial_query
        .shape_hits(
            collider,
            transform.translation.xy(),
            rotation_z(transform),
            gravity_dir,
            MAX_SHAPE_HITS,
            &ShapeCastConfig::from_max_distance(GROUND_PROBE_DISTANCE),
            &solid_filter(player_entity),
        )
        .into_iter()
        .any(|hit| {
            solids.contains(hit.entity) && hit.normal1.dot(supporting_normal) >= BLOCKING_NORMAL_DOT
        })
}

fn fell_out_of_world(transform: &Transform, level: &Level) -> bool {
    let bottom = level.ground.y - level.height;
    let top = level.ground.y + level.height;

    transform.translation.y < bottom || transform.translation.y > top
}

pub fn move_player(
    levels: Res<crate::level::registry::Levels>,
    resources: (Res<Config>, Res<Time>, Res<InputState>, Res<CurrentLevel>),
    mut deaths: MessageWriter<KillPlayer>,
    spatial_query: SpatialQuery,
    queries: MovementQueries,
) {
    let (config, time, input, current_level) = resources;
    let (solids, side_kill_solids, player) = queries;
    let dt = time.delta_secs();

    let Some(level) = current_level.get_from(&levels) else {
        return;
    };

    let (player_entity, mut transform, collider, mut velocity, mut player) = player.into_inner();

    let zoom = config.camera.zoom;
    let forward_speed = level.scroll_speed_px * player.scroll_speed_multiplier * zoom;
    let gravity = config.player.gravity_px * zoom;
    let jump_speed = config.player.jump_speed_px * zoom;

    let gravity_dir = player.gravity_dir;
    let gravity_axis = gravity_dir.as_vec2();

    velocity.x = forward_speed;
    velocity.0 += gravity_axis * gravity * dt;

    let grounded = touching_ground(
        &spatial_query,
        &solids,
        player_entity,
        collider,
        &transform,
        gravity_dir,
    );

    player.jump_buffer = if input.just_pressed(TerminalKeyCode::Up) {
        JUMP_BUFFER_SECONDS
    } else {
        (player.jump_buffer - dt).max(0.0)
    };

    player.grounded_grace = if grounded {
        GROUNDED_GRACE_SECONDS
    } else {
        (player.grounded_grace - dt).max(0.0)
    };

    if player.jump_buffer > 0.0 && player.grounded_grace > 0.0 {
        velocity.0 = -gravity_axis * jump_speed;
        velocity.x = forward_speed;
        player.jump_buffer = 0.0;
        player.grounded_grace = 0.0;
    }

    if let Some(entity) = move_and_collide(
        &spatial_query,
        &solids,
        player_entity,
        collider,
        &mut transform,
        Vec2::X * velocity.x * dt,
        true,
    ) {
        if side_kill_solids.contains(entity) {
            deaths.write(KillPlayer);
        }

        velocity.x = 0.0;
    }

    let gravity_speed = velocity.0.dot(gravity_axis);
    let moving_with_gravity = gravity_speed >= 0.0;

    let gravity_hit = move_and_collide(
        &spatial_query,
        &solids,
        player_entity,
        collider,
        &mut transform,
        gravity_axis * gravity_speed * dt,
        gravity_speed < 0.0,
    );

    if gravity_hit.is_some() {
        velocity.0 -= gravity_axis * gravity_speed;
    }

    let landed = gravity_hit.is_some() && moving_with_gravity;

    if landed && player.jump_buffer > 0.0 {
        velocity.0 = -gravity_axis * jump_speed;
        velocity.x = forward_speed;
        player.jump_buffer = 0.0;
        player.grounded_grace = 0.0;
    }

    let grounded = landed
        || touching_ground(
            &spatial_query,
            &solids,
            player_entity,
            collider,
            &transform,
            gravity_dir,
        );

    transform.rotation = if grounded {
        Quat::from_rotation_z(if gravity_dir.y < 0.0 { 0.0 } else { PI })
    } else {
        transform.rotation
            * Quat::from_rotation_z(AIR_SPIN_RADIANS_PER_SECOND * -gravity_dir.y * dt)
    };

    if fell_out_of_world(&transform, level) {
        deaths.write(KillPlayer);
    }
}

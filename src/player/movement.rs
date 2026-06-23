use crate::config::Config;
use crate::gameplay::death::KillPlayer;
use crate::input::InputState;
use crate::level::load::CurrentLevel;
use crate::level::model::{KillPlayerOnSide, Level, Solid};
use crate::player::queries::PlayerQuery;

use avian2d::prelude::{Collider, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use ratatui::crossterm::event::KeyCode;
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
const BLOCKING_NORMAL_DOT: f32 = 0.5;

fn rotation_z(transform: &Transform) -> f32 {
    transform.rotation.to_euler(EulerRot::XYZ).2
}

fn solid_filter(player_entity: Entity) -> SpatialQueryFilter {
    SpatialQueryFilter::from_excluded_entities([player_entity])
}

fn hit(
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
    let filter = solid_filter(player_entity);
    let opposing_normal = -direction.as_vec2();

    let hit = spatial_query
        .shape_hits(
            collider,
            transform.translation.xy(),
            rotation_z(transform),
            direction,
            8,
            &config,
            &filter,
        )
        .into_iter()
        .find(|hit| {
            solids.contains(hit.entity) && hit.normal1.dot(opposing_normal) >= BLOCKING_NORMAL_DOT
        });

    let travel = hit
        .as_ref()
        .map(|hit| (hit.distance - CONTACT_EPSILON).max(0.0))
        .unwrap_or(delta.length());

    transform.translation.x += direction.x * travel;
    transform.translation.y += direction.y * travel;

    hit.map(|hit| hit.entity)
}

fn grounded(
    spatial_query: &SpatialQuery,
    solids: &Query<(), With<Solid>>,
    player_entity: Entity,
    collider: &Collider,
    position: Vec2,
    rotation: f32,
    gravity_dir: Dir2,
) -> bool {
    let config = ShapeCastConfig::from_max_distance(GROUND_PROBE_DISTANCE);
    let filter = solid_filter(player_entity);
    let supporting_normal = -gravity_dir.as_vec2();

    spatial_query
        .shape_hits(
            collider,
            position,
            rotation,
            gravity_dir,
            8,
            &config,
            &filter,
        )
        .into_iter()
        .any(|hit| {
            solids.contains(hit.entity) && hit.normal1.dot(supporting_normal) >= BLOCKING_NORMAL_DOT
        })
}

fn fell_out_of_world(transform: &Transform, world: &Level) -> bool {
    let world_bottom = world.ground.y - world.size.y;
    let world_top = world.ground.y + world.size.y;

    transform.translation.y < world_bottom || transform.translation.y > world_top
}

pub fn move_player(
    resources: (Res<Config>, Res<Time>, Res<InputState>, Res<CurrentLevel>),
    mut deaths: MessageWriter<KillPlayer>,
    spatial_query: SpatialQuery,
    queries: MovementQueries,
) {
    let (config, time, input_state, current_level) = resources;
    let (solids, side_kill_solids, player) = queries;
    let dt = time.delta_secs();
    let level = current_level.level.as_ref().unwrap();

    let forward_speed = level.scroll_speed_px * config.camera.zoom;

    let gravity = config.player.gravity_px * config.camera.zoom;
    let jump_speed = config.player.jump_speed_px * config.camera.zoom;

    let wants_jump = input_state.just_pressed(KeyCode::Up);

    let (player_entity, mut transform, collider, mut velocity, mut player) = player.into_inner();

    let gravity_dir = player.gravity_dir;
    let gravity_velocity = gravity_dir.as_vec2() * gravity;

    velocity.x = forward_speed;
    velocity.0 += gravity_velocity * dt;

    let is_grounded = grounded(
        &spatial_query,
        &solids,
        player_entity,
        collider,
        transform.translation.xy(),
        rotation_z(&transform),
        gravity_dir,
    );

    if is_grounded {
        player.grounded_grace = GROUNDED_GRACE_SECONDS;
    } else {
        player.grounded_grace = (player.grounded_grace - dt).max(0.0);
    }

    if wants_jump && player.grounded_grace > 0.0 {
        velocity.0 = -gravity_dir.as_vec2() * jump_speed;
        velocity.x = forward_speed;
        player.grounded_grace = 0.0;
    }

    if let Some(entity) = hit(
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

    let vertical_delta = Vec2::Y * velocity.y * dt;
    let speed_with_gravity = velocity.0.dot(gravity_dir.as_vec2());
    let moving_with_gravity = speed_with_gravity >= 0.0;
    let moving_away_from_ground = speed_with_gravity < 0.0;
    let vertical_hit = hit(
        &spatial_query,
        &solids,
        player_entity,
        collider,
        &mut transform,
        vertical_delta,
        moving_away_from_ground,
    );

    if vertical_hit.is_some() {
        velocity.y = 0.0;
    }

    let landed = vertical_hit.is_some() && moving_with_gravity;

    if landed
        || grounded(
            &spatial_query,
            &solids,
            player_entity,
            collider,
            transform.translation.xy(),
            rotation_z(&transform),
            gravity_dir,
        )
    {
        transform.rotation = Quat::from_rotation_z(if gravity_dir.y < 0.0 { 0.0 } else { PI })
    } else {
        transform.rotate_z(AIR_SPIN_RADIANS_PER_SECOND * -gravity_dir.y * dt);
    }

    if fell_out_of_world(&transform, level) {
        deaths.write(KillPlayer);
    }
}

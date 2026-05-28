use crate::AppState;
use crate::config::Config;
use crate::core::collision::bounds_from_sprite;
use crate::player::components::{Player, Velocity};
use crate::player::queries::Players;
use crate::world::components::HazardBox;
use crate::world::loading::{CurrentWorld, despawn_music, spawn_music};
use crate::world::model::Level;
use crate::world::queries::MusicEntities;
use bevy::ecs::system::SystemParam;
use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;

type Hazards<'w, 's> = Query<'w, 's, (&'static Transform, &'static HazardBox), Without<Player>>;

#[derive(Resource)]
pub struct DeathPause {
    pub timer: Timer,
    pub percent: u8,
}

impl DeathPause {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            percent: 0,
        }
    }
}

#[derive(SystemParam)]
pub struct HazardParams<'w, 's> {
    players: Players<'w, 's>,
    hazards: Hazards<'w, 's>,
    music: MusicEntities<'w, 's>,
}
fn reset_player(transform: &mut Transform, velocity: &mut Velocity, spawn: Vec2) {
    transform.translation = spawn.extend(0.0);
    transform.rotation = Quat::IDENTITY;
    velocity.0 = Vec2::ZERO;
}
pub fn completion_percent(player_x: f32, world: &Level) -> u8 {
    let start_x = world.player.spawn.x;
    let distance = (world.size.x - start_x).max(f32::EPSILON);
    (((player_x - start_x) / distance) * 100.0)
        .clamp(0.0, 100.0)
        .round() as u8
}
fn detect_player_death(world: &Level, players: &mut Players, hazards: &Hazards) -> Option<u8> {
    let world_bottom = world.ground.y - world.size.y;
    for (transform, sprite, _) in players.iter() {
        let player_bounds = bounds_from_sprite(transform, sprite);
        let hit_hazard = hazards.iter().any(|(transform, area)| {
            player_bounds.intersects(&Aabb2d::new(transform.translation.xy(), area.half_size))
        });
        let fell_out_of_world = transform.translation.y < world_bottom;
        if hit_hazard || fell_out_of_world {
            return Some(completion_percent(transform.translation.x, world));
        }
    }
    None
}
fn start_death_pause(
    percent: u8,
    config: &Config,
    next_state: &mut NextState<AppState>,
    pause: &mut DeathPause,
    players: &mut Players,
    commands: &mut Commands,
    music: &MusicEntities,
) {
    for (_, _, mut velocity) in players.iter_mut() {
        velocity.0 = Vec2::ZERO;
    }
    despawn_music(commands, music);
    *pause = DeathPause {
        timer: Timer::from_seconds(config.player.death_pause_seconds, TimerMode::Once),
        percent,
    };
    next_state.set(AppState::Dead);
}

pub fn begin_death_pause(
    config: Res<Config>,
    current_world: Res<CurrentWorld>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    mut hazards: HazardParams,
) {
    let Some(world) = current_world.definition.as_ref() else {
        return;
    };
    let Some(percent) = detect_player_death(world, &mut hazards.players, &hazards.hazards) else {
        return;
    };
    start_death_pause(
        percent,
        &config,
        &mut next_state,
        &mut pause,
        &mut hazards.players,
        &mut commands,
        &hazards.music,
    );
}

type DeathResources<'w> = (
    Res<'w, Config>,
    Res<'w, Time>,
    Res<'w, CurrentWorld>,
    Res<'w, AssetServer>,
);

pub fn tick_death_pause(
    resources: DeathResources,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    mut players: Players,
) {
    let (config, time, current_world, asset_server) = resources;

    let Some(world) = current_world.definition.as_ref() else {
        return;
    };
    pause.timer.tick(time.delta());
    if !pause.timer.is_finished() {
        return;
    }
    let spawn = world.player.spawn;
    for (mut transform, _, mut velocity) in players.iter_mut() {
        reset_player(&mut transform, &mut velocity, spawn);
    }
    spawn_music(&mut commands, &asset_server, world, &config);
    next_state.set(AppState::Playing);
}

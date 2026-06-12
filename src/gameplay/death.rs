use crate::AppState;
use crate::config::Config;
use crate::level::load::{CurrentLevel, despawn_music, spawn_music};
use crate::level::model::Level;
use crate::level::queries::MusicEntities;
use crate::player::components::{Player, Velocity};
use crate::player::queries::PlayerQuery;
use bevy::prelude::*;

#[derive(Resource)]
pub struct DeathPause {
    pub timer: Timer,
    pub percent: u8,
}

#[derive(Message)]
pub struct KillPlayer;

impl DeathPause {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            percent: 0,
        }
    }
}

fn reset_player(
    player: &mut Player,
    transform: &mut Transform,
    velocity: &mut Velocity,
    spawn: Vec2,
) {
    player.gravity_dir = Dir2::NEG_Y;
    player.grounded_grace = 0.0;
    transform.translation = spawn.extend(0.0);
    transform.rotation = Quat::IDENTITY;
    velocity.0 = Vec2::ZERO;
}
fn completion_percent(player_x: f32, world: &Level) -> u8 {
    let start_x = world.player.spawn.x;
    let distance = (world.size.x - start_x).max(f32::EPSILON);
    (((player_x - start_x) / distance) * 100.0)
        .clamp(0.0, 100.0)
        .round() as u8
}

type BeginDeathResources<'w> = (Res<'w, Config>, Res<'w, CurrentLevel>);

pub fn begin_death_pause(
    resources: BeginDeathResources,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    player: PlayerQuery,
    music: MusicEntities,
    mut deaths: MessageReader<KillPlayer>,
) {
    if deaths.read().next().is_none() {
        return;
    }
    let (config, current_level) = resources;
    let Some(world) = current_level.0.as_ref() else {
        return;
    };
    let (_, transform, _, mut velocity, _) = player.into_inner();
    let percent = completion_percent(transform.translation.x, world);

    velocity.0 = Vec2::ZERO;
    despawn_music(&mut commands, &music);
    *pause = DeathPause {
        timer: Timer::from_seconds(config.player.death_pause_seconds, TimerMode::Once),
        percent,
    };
    next_state.set(AppState::Dead);
}

type DeathResources<'w> = (
    Res<'w, Config>,
    Res<'w, Time>,
    Res<'w, CurrentLevel>,
    Res<'w, AssetServer>,
);

pub fn tick_death_pause(
    resources: DeathResources,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    player: PlayerQuery,
) {
    let (config, time, current_level, asset_server) = resources;

    let Some(world) = current_level.0.as_ref() else {
        return;
    };
    pause.timer.tick(time.delta());
    if !pause.timer.is_finished() {
        return;
    }
    let spawn = world.player.spawn;
    let (_, mut transform, _, mut velocity, mut player) = player.into_inner();

    reset_player(&mut player, &mut transform, &mut velocity, spawn);
    spawn_music(&mut commands, &config, &asset_server, world);
    next_state.set(AppState::Playing);
}

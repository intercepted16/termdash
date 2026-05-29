use crate::AppState;
use crate::config::Config;
use crate::player::components::{Player, Velocity};
use crate::player::queries::PlayerQuery;
use crate::world::loading::{CurrentWorld, despawn_music, spawn_music};
use crate::world::model::Level;
use crate::world::queries::MusicEntities;
use bevy::prelude::*;

#[derive(Resource)]
pub struct DeathPause {
    pub timer: Timer,
    pub percent: u8,
}

#[derive(Message)]
pub struct KillPlayerEvent {
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

pub fn fell_out_of_world(transform: &Transform, world: &Level) -> bool {
    let world_bottom = world.ground.y - world.size.y;
    transform.translation.y < world_bottom
}

pub fn emit_out_of_world_deaths(
    current_world: Res<CurrentWorld>,
    players: Query<&Transform, With<Player>>,
    mut deaths: MessageWriter<KillPlayerEvent>,
) {
    let Some(world) = current_world.definition.as_ref() else {
        return;
    };

    for transform in &players {
        if fell_out_of_world(transform, world) {
            deaths.write(KillPlayerEvent {
                percent: completion_percent(transform.translation.x, world),
            });
        }
    }
}

fn start_death_pause(
    percent: u8,
    config: &Config,
    next_state: &mut NextState<AppState>,
    pause: &mut DeathPause,
    player: PlayerQuery,
    commands: &mut Commands,
    music: &MusicEntities,
) {
    let (_, _, mut velocity, _) = player.into_inner();
    velocity.0 = Vec2::ZERO;
    despawn_music(commands, music);
    *pause = DeathPause {
        timer: Timer::from_seconds(config.player.death_pause_seconds, TimerMode::Once),
        percent,
    };
    next_state.set(AppState::Dead);
}

pub fn begin_death_pause(
    config: Res<Config>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    player: PlayerQuery,
    music: MusicEntities,
    mut deaths: MessageReader<KillPlayerEvent>,
) {
    let Some(death) = deaths.read().next() else {
        return;
    };

    start_death_pause(
        death.percent,
        &config,
        &mut next_state,
        &mut pause,
        player,
        &mut commands,
        &music,
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
    player: PlayerQuery,
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
    let (mut transform, _, mut velocity, _) = player.into_inner();

    reset_player(&mut transform, &mut velocity, spawn);
    spawn_music(&mut commands, &asset_server, world, &config);
    next_state.set(AppState::Playing);
}

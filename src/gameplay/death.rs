use crate::AppState;
use crate::config::Config;
use crate::gameplay::RunStats;
use crate::level::load::{CurrentLevel, LoadLevelEvent, despawn_music};
use crate::level::queries::MusicEntities;
use crate::player::queries::PlayerQuery;
use bevy::prelude::*;

#[derive(Resource)]
pub struct DeathPause {
    pub timer: Timer,
}

#[derive(Message)]
pub struct KillPlayer;

impl DeathPause {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

pub fn begin(
    resources: (Res<Config>, ResMut<RunStats>),
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

    let (config, mut stats) = resources;
    stats.attempts += 1;

    let (_, _, _, mut velocity, _) = player.into_inner();

    velocity.0 = Vec2::ZERO;
    despawn_music(&mut commands, &music);

    *pause = DeathPause::new(config.player.death_pause_seconds);

    next_state.set(AppState::Dead);
}

pub fn tick(
    time: Res<Time>,
    current_level: Res<CurrentLevel>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    mut load_events: MessageWriter<LoadLevelEvent>,
    mut stats: ResMut<RunStats>,
) {
    pause.timer.tick(time.delta());

    if !pause.timer.is_finished() {
        return;
    }

    let Some(index) = current_level.0 else {
        return;
    };

    stats.time = 0.0;

    load_events.write(LoadLevelEvent { index });
    next_state.set(AppState::Playing);
}

pub fn finish_pause(mut pause: ResMut<DeathPause>) {
    let duration = pause.timer.duration();
    pause.timer.set_elapsed(duration);
}

use crate::AppState;
use crate::config::Config;
use crate::level::load::{CurrentLevel, LoadWorldEvent, despawn_music};
use crate::level::model::Level;
use crate::level::queries::MusicEntities;
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

fn completion_percent(player_x: f32, level: &Level) -> u8 {
    let start_x = level.player.spawn.x;
    let distance = (level.size.x - start_x).max(f32::EPSILON);

    (((player_x - start_x) / distance) * 100.0)
        .clamp(0.0, 100.0)
        .round() as u8
}

pub fn begin(
    levels: Res<crate::level::registry::Levels>,
    resources: (Res<Config>, Res<CurrentLevel>),
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    player: PlayerQuery,
    music: MusicEntities,
    mut deaths: MessageReader<KillPlayer>,
) {
    if deaths.read().count() == 0 {
        return;
    }

    let (config, current_level) = resources;

    let Some(level) = current_level.get_from(&levels) else {
        return;
    };

    let (_, transform, _, mut velocity, _) = player.into_inner();

    let percent = completion_percent(transform.translation.x, level);

    velocity.0 = Vec2::ZERO;
    despawn_music(&mut commands, &music);

    *pause = DeathPause {
        timer: Timer::from_seconds(config.player.death_pause_seconds, TimerMode::Once),
        percent,
    };

    next_state.set(AppState::Dead);
}

pub fn tick(
    time: Res<Time>,
    current_level: Res<CurrentLevel>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<DeathPause>,
    mut load_events: MessageWriter<LoadWorldEvent>,
) {
    pause.timer.tick(time.delta());

    if !pause.timer.is_finished() {
        return;
    }

    if pause.percent >= 100 {
        next_state.set(AppState::MainMenu);
        return;
    }

    let Some(index) = current_level.0 else {
        return;
    };

    load_events.write(LoadWorldEvent { index });
    next_state.set(AppState::Playing);
}

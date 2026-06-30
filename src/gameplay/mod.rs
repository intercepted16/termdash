pub mod death;
pub mod triggers;

use crate::AppState;
use crate::config::Config;
use crate::gameplay::death::{DeathPause, KillPlayer, begin, tick};
use crate::gameplay::triggers::{TriggerState, apply_player_triggers};
use crate::level::load::CurrentLevel;
use crate::level::model::LevelEntity;
use crate::level::registry::Levels;
use crate::paths::GamePaths;
use crate::player::components::Player;
use crate::player::move_player;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct RunStats {
    pub percent: u8,
    pub attempts: u32,
}

fn update_run_stats(
    player: Query<&Transform, With<Player>>,
    current_level: Res<CurrentLevel>,
    levels: Res<Levels>,
    mut stats: ResMut<RunStats>,
) {
    let Ok(transform) = player.single() else {
        return;
    };

    let Some(level) = current_level.get_from(levels.as_ref()) else {
        return;
    };

    let player_x = transform.translation.x;
    let start_x = level.player.spawn.x;
    let end_x = level.end_x();
    stats.percent = (((player_x - start_x) / (end_x - start_x)) * 100.0).clamp(0.0, 100.0) as u8;
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // TODO: use preexisting config resource
        let paths = GamePaths::init().expect("paths should load");
        app.insert_resource(DeathPause::new(
            Config::load(&paths)
                .expect("config should load")
                .player
                .death_pause_seconds,
        ))
        .init_resource::<TriggerState>()
        .init_resource::<RunStats>()
        .add_message::<KillPlayer>()
        .add_systems(Update, update_run_stats)
        .add_systems(Update, handle_victory.run_if(in_state(AppState::Playing)));
        for state in [AppState::MainMenu, AppState::Victory] {
            app.add_systems(OnEnter(state), cleanup_gameplay);
        }
        app.add_systems(
            Update,
            (apply_player_triggers, begin)
                .chain()
                .after(move_player)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, tick.run_if(in_state(AppState::Dead)));
    }
}

fn handle_victory(stats: Res<RunStats>, mut next_state: ResMut<NextState<AppState>>) {
    if stats.percent >= 100 {
        next_state.set(AppState::Victory);
    }
}

fn cleanup_gameplay(
    _levels: Res<crate::level::registry::Levels>,
    mut commands: Commands,
    mut current_level: ResMut<CurrentLevel>,
    mut trigger_state: ResMut<TriggerState>,
    mut stats: ResMut<RunStats>,
    entities: Query<Entity, With<LevelEntity>>,
) {
    *stats = RunStats::default();
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    current_level.0 = None;
    trigger_state.0.clear();
}

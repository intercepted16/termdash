pub mod death;
pub mod triggers;

use crate::AppState;
use crate::config::Config;
use crate::gameplay::death::{DeathPause, KillPlayer, begin, tick};
use crate::gameplay::triggers::{TriggerState, apply_player_triggers};
use crate::level::load::CurrentLevel;
use crate::level::model::LevelEntity;
use crate::paths::GamePaths;
use crate::player::move_player;
use bevy::prelude::*;

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
        .add_message::<KillPlayer>()
        .add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
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
fn cleanup_gameplay(
    _levels: Res<crate::level::registry::Levels>,
    mut commands: Commands,
    mut current_level: ResMut<CurrentLevel>,
    mut trigger_state: ResMut<TriggerState>,
    entities: Query<Entity, With<LevelEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    current_level.0 = None;
    trigger_state.0.clear();
}

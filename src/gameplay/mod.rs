pub mod death;
pub mod triggers;

use crate::AppState;
use crate::config::Config;
use crate::gameplay::death::{DeathPause, KillPlayer, begin_death_pause, tick_death_pause};
use crate::gameplay::triggers::{TriggerState, apply_player_triggers};
use crate::level::load::CurrentLevel;
use crate::level::model::LevelEntity;
use crate::player::move_player;
use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // TODO: use preexisting config resource
        app.insert_resource(DeathPause::new(Config::load().player.death_pause_seconds))
            .init_resource::<TriggerState>()
            .add_message::<KillPlayer>()
            .add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
        app.add_systems(
            Update,
            (apply_player_triggers, begin_death_pause)
                .chain()
                .after(move_player)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, tick_death_pause.run_if(in_state(AppState::Dead)));
    }
}
fn cleanup_gameplay(
    mut commands: Commands,
    mut current_level: ResMut<CurrentLevel>,
    mut trigger_state: ResMut<TriggerState>,
    entities: Query<Entity, With<LevelEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    current_level.index = None;
    current_level.level = None;
    trigger_state.0.clear();
}

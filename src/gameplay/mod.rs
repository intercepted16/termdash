pub mod death;
pub mod triggers;

use crate::AppState;
use crate::config::Config;
use crate::gameplay::death::{
    DeathPause, KillPlayerEvent, begin_death_pause, emit_out_of_world_deaths, tick_death_pause,
};
use crate::gameplay::triggers::{TriggerState, apply_player_triggers};
use crate::level::components::WorldEntity;
use crate::level::loading::CurrentWorld;
use crate::player::move_player;
use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // TODO: use preexisting config resource
        app.insert_resource(DeathPause::new(Config::load().player.death_pause_seconds))
            .init_resource::<TriggerState>()
            .add_message::<KillPlayerEvent>()
            .add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
        app.add_systems(
            Update,
            (
                apply_player_triggers,
                emit_out_of_world_deaths,
                begin_death_pause,
            )
                .chain()
                .after(move_player)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, tick_death_pause.run_if(in_state(AppState::Dead)));
    }
}
fn cleanup_gameplay(
    mut commands: Commands,
    mut current_world: ResMut<CurrentWorld>,
    mut trigger_state: ResMut<TriggerState>,
    entities: Query<Entity, With<WorldEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    current_world.definition = None;
    trigger_state.0.clear();
}

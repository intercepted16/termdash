pub mod death;
mod orbs;

use crate::AppState;
use crate::config::Config;
use crate::gameplay::death::{DeathPause, begin_death_pause, tick_death_pause};
use crate::gameplay::orbs::activate_jump_orbs;
use crate::player::move_player;
use crate::world::components::WorldEntity;
use crate::world::loading::CurrentWorld;
use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        // TODO: use preexisting config resource
        app.insert_resource(DeathPause::new(Config::load().player.death_pause_seconds))
            .add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
        app.add_systems(
            Update,
            (
                activate_jump_orbs.after(move_player),
                begin_death_pause.after(activate_jump_orbs),
            )
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, tick_death_pause.run_if(in_state(AppState::Dead)));
    }
}
fn cleanup_gameplay(
    mut commands: Commands,
    mut current_world: ResMut<CurrentWorld>,
    entities: Query<Entity, With<WorldEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    current_world.definition = None;
}

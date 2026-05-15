pub mod death;

use crate::core::app_state::AppState;
use crate::features::gameplay::death::handle_death;
use crate::features::world::components::WorldEntity;
use crate::features::world::loading::CurrentWorld;
use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
        app.add_systems(Update, handle_death.run_if(in_state(AppState::Playing)));
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

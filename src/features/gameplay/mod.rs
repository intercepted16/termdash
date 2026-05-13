use crate::core::app_state::AppState;
use crate::features::world::components::WorldEntity;
use crate::features::world::loading::CurrentWorld;
use bevy::prelude::*;
pub struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
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

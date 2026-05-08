use bevy::prelude::*;

use crate::core::app_state::AppState;
use crate::features::player::PlayerPlugin;
use crate::features::player::components::Player;
use crate::features::world::WorldPlugin;
use crate::features::world::components::Ground;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WorldPlugin, PlayerPlugin))
            .add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
    }
}

fn cleanup_gameplay(
    mut commands: Commands,
    entities: Query<Entity, Or<(With<Player>, With<Ground>)>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

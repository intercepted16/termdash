pub mod death;
mod orbs;

use crate::AppState;
use crate::gameplay::death::{PlayerDeathState, handle_death};
use crate::gameplay::orbs::activate_jump_orbs;
use crate::player::move_player;
use crate::world::components::WorldEntity;
use crate::world::loading::CurrentWorld;
use bevy::prelude::*;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDeathState>()
            .add_systems(OnEnter(AppState::MainMenu), cleanup_gameplay);
        app.add_systems(
            Update,
            (activate_jump_orbs.after(move_player), handle_death)
                .run_if(in_state(AppState::Playing)),
        );
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

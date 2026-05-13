pub mod components;
mod systems;
use crate::core::app_state::AppState;
use crate::features::player::systems::hazard::handle_hazards;
use crate::features::player::systems::movement::move_player;
use bevy::prelude::*;
pub use systems::hazard::PlayerDeathState;
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDeathState>().add_systems(
            Update,
            (move_player, handle_hazards)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

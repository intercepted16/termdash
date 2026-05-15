pub mod components;
pub mod queries;
mod systems;
use crate::core::app_state::AppState;
use crate::features::player::systems::movement::move_player;
use bevy::prelude::*;

pub use crate::features::gameplay::death::PlayerDeathState;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDeathState>()
            .add_systems(Update, move_player.run_if(in_state(AppState::Playing)));
    }
}

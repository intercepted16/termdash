pub mod components;
mod movement;
pub mod queries;
use crate::AppState;
use bevy::prelude::*;

pub use crate::player::movement::move_player;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player.run_if(in_state(AppState::Playing)));
    }
}

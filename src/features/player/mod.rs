pub mod components;
mod systems;

use bevy::prelude::*;

use crate::core::app_state::AppState;
use crate::features::player::systems::move_player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player.run_if(in_state(AppState::Playing)));
    }
}

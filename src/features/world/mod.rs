pub mod components;
pub mod solid;

use bevy::prelude::*;

use crate::core::app_state::AppState;
use crate::features::world::components::spawn_ground;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Playing), spawn_ground);
    }
}

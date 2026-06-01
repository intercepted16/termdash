pub mod components;
pub mod loading;
pub mod model;
mod objects;
pub mod queries;
pub mod registry;
pub mod visualizer;
use crate::world::loading::{CurrentWorld, LoadWorldEvent, load_world};
use crate::world::registry::Levels;
use crate::world::visualizer::update_audio_visualizer;
use bevy::prelude::*;
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentWorld>()
            .insert_resource(Levels::load().expect("failed to load worlds"))
            .add_message::<LoadWorldEvent>()
            .add_systems(Update, (load_world, update_audio_visualizer));
    }
}

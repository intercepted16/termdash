pub mod components;
pub mod loading;
pub mod model;
pub mod queries;
pub mod registry;
pub mod visualizer;
use crate::level::loading::{CurrentWorld, LoadWorldEvent, load_world};
use crate::level::model::Prefabs;
use crate::level::registry::Levels;
use crate::level::visualizer::update_audio_visualizer;
use bevy::prelude::*;
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentWorld>()
            .insert_resource(Levels::load().expect("failed to load worlds"))
            .insert_resource(Prefabs::load().expect("failed to load object prefabs"))
            .add_message::<LoadWorldEvent>()
            .add_systems(Update, (load_world, update_audio_visualizer));
    }
}

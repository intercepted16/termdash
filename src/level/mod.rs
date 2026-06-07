pub mod components;
pub mod load;
pub mod model;
pub mod queries;
pub mod registry;
pub mod visualizer;
use crate::level::load::{CurrentLevel, LoadWorldEvent, load_level};
use crate::level::model::Prefabs;
use crate::level::registry::Levels;
use crate::level::visualizer::update_audio_visualizer;
use bevy::prelude::*;
pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .insert_resource(Levels::load().expect("failed to load worlds"))
            .insert_resource(Prefabs::load())
            .add_message::<LoadWorldEvent>()
            .add_systems(Update, (load_level, update_audio_visualizer));
    }
}

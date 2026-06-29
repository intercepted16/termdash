pub mod load;
pub mod model;
pub mod queries;
pub mod registry;
pub mod visualizer;
use crate::AppState;
use crate::level::load::{CurrentLevel, LoadLevelEvent, animate_objects, load_level};
use crate::level::model::Prefabs;
use crate::level::registry::Levels;
use crate::level::visualizer::update_audio_visualizer;
use crate::paths::GamePaths;
use crate::player::move_player;
use bevy::prelude::*;
pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let prefabs = {
            let paths = app.world().resource::<GamePaths>();
            Prefabs::load(paths)
        };
        let levels = {
            let paths = app.world().resource::<GamePaths>();
            Levels::load(paths).unwrap_or_else(|err| panic!("failed to load worlds: {}", err))
        };

        app.init_resource::<CurrentLevel>()
            .insert_resource(levels)
            .insert_resource(prefabs)
            .add_message::<LoadLevelEvent>()
            .add_systems(
                Update,
                (
                    load_level.before(move_player),
                    update_audio_visualizer.run_if(in_state(AppState::Playing)),
                ),
            )
            .add_systems(Update, animate_objects.run_if(in_state(AppState::Playing)));
    }
}

pub mod components;
pub mod loading;
pub mod model;
pub mod queries;
pub mod registry;
use crate::features::world::loading::{CurrentWorld, LoadWorldEvent, load_world};
use crate::features::world::registry::WorldRegistry;
use bevy::prelude::*;
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldRegistry>()
            .init_resource::<CurrentWorld>()
            .add_message::<LoadWorldEvent>()
            .add_systems(Update, load_world);
    }
}

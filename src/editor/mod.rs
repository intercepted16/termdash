pub mod inspector;
pub mod model;
pub mod refresh;
pub mod selection;
pub mod window;

use avian2d::prelude::ColliderConstructor;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use model::{EditorState, EditorWindowPass};

use crate::level::model::register_level_data_types;
use crate::state::AppState;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), DefaultInspectorConfigPlugin))
            .init_resource::<EditorState>()
            .add_message::<refresh::RefreshLevelEvent>()
            .register_type::<ColliderConstructor>()
            .add_systems(Startup, window::disable_primary_egui_context)
            .add_systems(OnEnter(AppState::Editing), window::open_editor_window)
            .add_systems(OnExit(AppState::Editing), window::close_editor_window)
            .add_systems(Update, window::handle_window_close)
            .add_systems(Update, refresh::refresh_level)
            .add_systems(EditorWindowPass, inspector::show_editor);

        register_level_data_types(app);
    }
}

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

#[derive(Component)]
pub struct EditorWindow;

#[derive(Component)]
pub struct EditorCamera;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EditorWindowPass;

#[derive(Resource)]
pub struct EditorState {
    pub selected_object: Option<usize>,
    pub status: String,
    pub dirty: bool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            selected_object: None,
            status: "editor closed".to_string(),
            dirty: false,
        }
    }
}

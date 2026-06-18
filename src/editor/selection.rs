use crate::{
    editor::model::EditorState,
    level::model::{Level, LevelObject},
};
use bevy::prelude::*;

const DEFAULT_PREFAB: &str = "spike";

pub fn clamp_selection(editor: &mut EditorState, level: &Level) {
    if editor
        .selected_object
        .is_some_and(|index| index >= level.objects.len())
    {
        editor.selected_object = None;
    }
}

pub fn select_nearest(editor: &mut EditorState, level: &Level, position: Vec2) {
    editor.selected_object = level
        .objects
        .iter()
        .enumerate()
        .map(|(index, object)| (index, object.position.distance(position)))
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index);
}

pub fn push_default_object(level: &mut Level, position: Vec2) -> usize {
    let index = level.objects.len();
    level.objects.push(LevelObject {
        prefab: Some(DEFAULT_PREFAB.to_string()),
        position,
        ..default()
    });
    index
}

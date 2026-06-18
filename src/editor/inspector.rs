use super::{
    model::{EditorCamera, EditorState},
    refresh::RefreshLevelEvent,
    save::save_current_level,
    selection::{clamp_selection, push_default_object, select_nearest},
};
use crate::{
    level::{load::CurrentLevel, registry::Levels},
    player::components::Player,
    state::AppState,
};
use bevy::{ecs::reflect::AppTypeRegistry, ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContext, egui};
use bevy_inspector_egui::reflect_inspector;

pub fn show_editor(
    mut egui_context: Single<&mut EguiContext, With<EditorCamera>>,
    mut editor_ui: EditorUiParams,
) {
    let ctx = egui_context.get_mut();
    let save_requested = ctx.input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
            egui::Modifiers::CTRL,
            egui::Key::S,
        ))
    });

    if editor_ui.current_level.level.is_none() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("No Level Loaded");
            ui.label("Enter a level before opening the editor.");
        });
        return;
    }

    let mut save_clicked = save_requested;
    let mut close_clicked = false;
    let mut changed = false;

    {
        let level = editor_ui
            .current_level
            .level
            .as_mut()
            .expect("level checked above");

        clamp_selection(&mut editor_ui.editor, level);

        egui::TopBottomPanel::top("editor_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    save_clicked = true;
                }

                if ui.button("Test").clicked() {
                    close_clicked = true;
                }

                let marker = if editor_ui.editor.dirty { "*" } else { "" };
                ui.label(format!("{marker}{}", editor_ui.editor.status));
            });
        });

        egui::SidePanel::left("object_selection")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Objects");

                if ui.button("Select nearest to player").clicked()
                    && let Ok(transform) = editor_ui.player.single()
                {
                    select_nearest(&mut editor_ui.editor, level, transform.translation.xy());
                }

                if ui.button("Add object at player").clicked() {
                    let position = editor_ui
                        .player
                        .single()
                        .map(|transform| transform.translation.xy())
                        .unwrap_or(level.player.spawn);
                    editor_ui.editor.selected_object = Some(push_default_object(level, position));
                    changed = true;
                }

                if ui.button("Delete selected").clicked()
                    && let Some(index) = editor_ui.editor.selected_object
                    && index < level.objects.len()
                {
                    level.objects.remove(index);
                    editor_ui.editor.selected_object = None;
                    changed = true;
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, object) in level.objects.iter().enumerate() {
                        let prefab = object.prefab.as_deref().unwrap_or("custom");
                        let selected = editor_ui.editor.selected_object == Some(index);
                        let label = format!(
                            "#{index} {prefab} ({:.0}, {:.0})",
                            object.position.x, object.position.y
                        );

                        if ui.selectable_label(selected, label).clicked() {
                            editor_ui.editor.selected_object = Some(index);
                        }
                    }
                });
            });

        egui::SidePanel::right("selected_object")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Object Inspector");

                let Some(index) = editor_ui.editor.selected_object else {
                    ui.label("No object selected.");
                    return;
                };

                let Some(object) = level.objects.get_mut(index) else {
                    ui.label("Selected object no longer exists.");
                    return;
                };

                let registry = editor_ui.type_registry.read();
                changed |= reflect_inspector::ui_for_value(object, ui, &registry);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Level Data");
            egui::ScrollArea::both().show(ui, |ui| {
                let registry = editor_ui.type_registry.read();
                changed |= reflect_inspector::ui_for_value(level, ui, &registry);
            });
        });

        if changed {
            clamp_selection(&mut editor_ui.editor, level);
            editor_ui.editor.dirty = true;
            editor_ui.editor.status = "edited level data".to_string();
            editor_ui.refresh_events.write(RefreshLevelEvent);
        }
    }

    if save_clicked {
        save(
            &mut editor_ui.editor,
            &editor_ui.current_level,
            &mut editor_ui.levels,
        );
    }

    if close_clicked {
        editor_ui.next_state.set(AppState::Playing);
    }
}

#[derive(SystemParam)]
pub struct EditorUiParams<'w, 's> {
    editor: ResMut<'w, EditorState>,
    current_level: ResMut<'w, CurrentLevel>,
    levels: ResMut<'w, Levels>,
    type_registry: Res<'w, AppTypeRegistry>,
    player: Query<'w, 's, &'static Transform, With<Player>>,
    refresh_events: MessageWriter<'w, RefreshLevelEvent>,
    next_state: ResMut<'w, NextState<AppState>>,
}

fn save(editor: &mut EditorState, current_level: &CurrentLevel, levels: &mut Levels) {
    match save_current_level(current_level, levels) {
        Ok(path) => {
            editor.dirty = false;
            editor.status = format!("saved {}", path.display());
        }
        Err(err) => {
            editor.status = format!("save failed: {err}");
        }
    }
}

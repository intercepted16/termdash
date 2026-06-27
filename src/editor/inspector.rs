use super::{
    model::{EditorCamera, EditorState},
    refresh::RefreshLevelEvent,
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
use std::{ops::DerefMut, path::PathBuf};

pub fn show_editor(
    mut egui_context: Single<&mut EguiContext, With<EditorCamera>>,
    mut params: EditorUiParams,
) {
    let ctx = egui_context.get_mut();
    let save_requested = ctx.input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
            egui::Modifiers::CTRL,
            egui::Key::S,
        ))
    });

    if params.current_level.0.is_none() {
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
        let level = params
            .current_level
            .get_from_mut(params.levels.deref_mut())
            .expect("level checked above");

        clamp_selection(&mut params.editor, level);

        egui::TopBottomPanel::top("editor_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    save_clicked = true;
                }

                if ui.button("Test").clicked() {
                    close_clicked = true;
                }

                let marker = if params.editor.dirty { "*" } else { "" };
                ui.label(format!("{marker}{}", params.editor.status));
            });
        });

        egui::SidePanel::left("object_selection")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Objects");

                if ui.button("Select nearest to player").clicked()
                    && let Ok(transform) = params.player.single()
                {
                    select_nearest(&mut params.editor, level, transform.translation.xy());
                }

                if ui.button("Add object at player").clicked() {
                    let position = params
                        .player
                        .single()
                        .map(|transform| transform.translation.xy())
                        .unwrap_or(level.player.spawn);
                    params.editor.selected_object = Some(push_default_object(level, position));
                    changed = true;
                }

                if ui.button("Delete selected").clicked()
                    && let Some(index) = params.editor.selected_object
                    && index < level.objects.len()
                {
                    level.objects.remove(index);
                    params.editor.selected_object = None;
                    changed = true;
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, object) in level.objects.iter().enumerate() {
                        let prefab = object.prefab.as_deref().unwrap_or("custom");
                        let selected = params.editor.selected_object == Some(index);
                        let label = format!(
                            "#{index} {prefab} ({:.0}, {:.0})",
                            object.position.x, object.position.y
                        );

                        if ui.selectable_label(selected, label).clicked() {
                            params.editor.selected_object = Some(index);
                        }
                    }
                });
            });

        egui::SidePanel::right("selected_object")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Object Inspector");

                let Some(index) = params.editor.selected_object else {
                    ui.label("No object selected.");
                    return;
                };

                let Some(object) = level.objects.get_mut(index) else {
                    ui.label("Selected object no longer exists.");
                    return;
                };

                let registry = params.type_registry.read();
                changed |= reflect_inspector::ui_for_value(object, ui, &registry);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Level Data");
            egui::ScrollArea::both().show(ui, |ui| {
                let registry = params.type_registry.read();
                changed |= reflect_inspector::ui_for_value(level, ui, &registry);
            });
        });

        if changed {
            clamp_selection(&mut params.editor, level);
            params.editor.dirty = true;
            params.editor.status = "edited level data".to_string();
            params.editor.refresh_pending = true;
        }

        if params.editor.refresh_pending && !ctx.wants_keyboard_input() {
            params.editor.refresh_pending = false;
            params.refresh_events.write(RefreshLevelEvent);
        }
    }

    if save_clicked {
        save(
            &mut params.editor,
            &params.current_level,
            &mut params.levels,
        );
    }

    if close_clicked {
        params.next_state.set(AppState::Playing);
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
    let path: Result<PathBuf, String> = (|| {
        let index = current_level.0.ok_or("no level is loaded")?;
        let level = current_level.get_from(levels).ok_or("no current level")?;

        levels.save(level.clone(), index)
    })();

    match path {
        Ok(path) => {
            editor.dirty = false;
            editor.status = format!("saved {}", path.display());
        }
        Err(err) => {
            editor.status = format!("save failed: {err}");
        }
    }
}

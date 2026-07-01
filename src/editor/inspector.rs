use super::{
    model::{EditorCamera, EditorState, EditorWindow, History},
    refresh::RefreshLevelEvent,
    selection::{clamp_selection, push_default_object, select_nearest},
};
use crate::{
    level::{load::CurrentLevel, model::Level, registry::Levels},
    player::components::Player,
    state::AppState,
};
use bevy::window::{WindowFocused, WindowMoved};
use bevy::{ecs::reflect::AppTypeRegistry, ecs::system::SystemParam, prelude::*};
use bevy_egui::{EguiContext, egui};
use bevy_inspector_egui::reflect_inspector;
use std::path::PathBuf;

const FOCUS_TEST_GRACE_SECS: f32 = 1.0;

pub fn show_editor(
    mut egui_context: Single<&mut EguiContext, With<EditorCamera>>,
    mut params: EditorUiParams,
) {
    let ctx = egui_context.get_mut();
    let save_requested = consume_ctrl_shortcut(ctx, egui::Key::S);
    let undo_requested = consume_ctrl_shortcut(ctx, egui::Key::Z);
    let redo_requested = consume_ctrl_shortcut(ctx, egui::Key::Y);

    let mut save_clicked = save_requested;
    let focus_test_requested = update_focus_test_timer(&mut params);
    let mut close_clicked = focus_test_requested;
    let mut changed = false;
    let current_level_index = params.current_level.0;

    {
        let level = params
            .current_level
            .get_from_mut(&mut params.levels)
            .expect("should be a level at this point");

        if params.editor.history_level != current_level_index {
            params.editor.history.reset(level);
            params.editor.history_level = current_level_index;
            params.editor.dirty = false;
        }

        if undo_requested || redo_requested {
            finish_pending_edit(&mut params.editor, level, &mut params.refresh_events);
        }

        if undo_requested {
            apply_history(
                &mut params.editor,
                level,
                &mut params.refresh_events,
                History::undo,
                "undid level edit",
            );
        } else if redo_requested {
            apply_history(
                &mut params.editor,
                level,
                &mut params.refresh_events,
                History::redo,
                "redid level edit",
            );
        }

        clamp_selection(&mut params.editor, level);

        egui::TopBottomPanel::top("editor_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                save_clicked |= ui.button("Save").clicked();
                close_clicked |= ui.button("Test").clicked();

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

        let editing_interactively =
            ctx.wants_keyboard_input() || ctx.input(|input| input.pointer.any_down());

        if !editing_interactively || save_clicked || close_clicked {
            finish_pending_edit(&mut params.editor, level, &mut params.refresh_events);
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

fn consume_ctrl_shortcut(ctx: &egui::Context, key: egui::Key) -> bool {
    ctx.input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, key))
    })
}

#[derive(SystemParam)]
pub struct EditorUiParams<'w, 's> {
    editor: ResMut<'w, EditorState>,
    current_level: ResMut<'w, CurrentLevel>,
    levels: ResMut<'w, Levels>,
    type_registry: Res<'w, AppTypeRegistry>,
    player: Query<'w, 's, &'static Transform, With<Player>>,
    editor_windows: Query<'w, 's, Entity, With<EditorWindow>>,
    window_focus: MessageReader<'w, 's, WindowFocused>,
    window_moved: MessageReader<'w, 's, WindowMoved>,
    time: Res<'w, Time>,
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

fn update_focus_test_timer(params: &mut EditorUiParams) -> bool {
    if params
        .window_moved
        .read()
        .any(|event| params.editor_windows.contains(event.window))
    {
        params.editor.focus_test_timer = None;
        return false;
    }

    let editor_focused = params
        .window_focus
        .read()
        .filter(|event| params.editor_windows.contains(event.window))
        .last()
        .map(|event| event.focused);

    match editor_focused {
        Some(false) => {
            params.editor.focus_test_timer =
                Some(Timer::from_seconds(FOCUS_TEST_GRACE_SECS, TimerMode::Once));
        }
        Some(true) => {
            params.editor.focus_test_timer = None;
        }
        None => {}
    }

    let Some(timer) = params.editor.focus_test_timer.as_mut() else {
        return false;
    };

    timer.tick(params.time.delta());
    if !timer.is_finished() {
        return false;
    }

    params.editor.focus_test_timer = None;
    true
}

fn apply_history(
    editor: &mut EditorState,
    level: &mut Level,
    refresh_events: &mut MessageWriter<RefreshLevelEvent>,
    action: fn(&mut History, &mut Level) -> Result<(), &'static str>,
    status: &'static str,
) {
    match action(&mut editor.history, level) {
        Ok(()) => {
            clamp_selection(editor, level);
            editor.dirty = true;
            editor.status = status.to_string();
            editor.refresh_pending = false;
            refresh_events.write(RefreshLevelEvent);
        }
        Err(err) => {
            editor.status = err.to_string();
        }
    }
}

fn finish_pending_edit(
    editor: &mut EditorState,
    level: &Level,
    refresh_events: &mut MessageWriter<RefreshLevelEvent>,
) {
    if !editor.refresh_pending {
        return;
    }

    editor.history.push(level);
    editor.refresh_pending = false;
    refresh_events.write(RefreshLevelEvent);
}

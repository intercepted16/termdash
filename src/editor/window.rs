use super::model::{EditorCamera, EditorWindow, EditorWindowPass};
use crate::state::AppState;
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    window::{PresentMode, WindowCloseRequested, WindowFocused, WindowRef, WindowResolution},
};
use bevy_egui::{EguiGlobalSettings, EguiMultipassSchedule};

pub fn disable_primary_egui_context(mut settings: ResMut<EguiGlobalSettings>) {
    settings.auto_create_primary_context = false;
}

pub fn open_editor_window(mut commands: Commands, windows: Query<Entity, With<EditorWindow>>) {
    if !windows.is_empty() {
        return;
    }

    let window = commands
        .spawn((
            Window {
                title: "Term Dash Editor".to_string(),
                resolution: WindowResolution::new(960, 720),
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            EditorWindow,
        ))
        .id();

    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Window(WindowRef::Entity(window)),
            ..default()
        },
        EguiMultipassSchedule::new(EditorWindowPass),
        EditorCamera,
        RenderLayers::none(),
    ));
}

pub fn close_editor_window(
    mut commands: Commands,
    windows: Query<Entity, With<EditorWindow>>,
    cameras: Query<Entity, With<EditorCamera>>,
) {
    for entity in &cameras {
        commands.entity(entity).despawn();
    }

    for entity in &windows {
        commands.entity(entity).despawn();
    }
}

pub fn handle_window_close(
    mut commands: Commands,
    windows: Query<Entity, With<EditorWindow>>,
    cameras: Query<Entity, With<EditorCamera>>,
    mut closed: MessageReader<WindowCloseRequested>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in closed.read() {
        if windows.contains(event.window) {
            for entity in &cameras {
                commands.entity(entity).despawn();
            }

            commands.entity(event.window).despawn();
            next_state.set(AppState::Playing);
        }
    }
}

pub fn handle_focus_change(
    windows: Query<Entity, With<EditorWindow>>,
    state: Res<State<AppState>>,
    mut window_focus: MessageReader<WindowFocused>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if window_focus
        .read()
        .any(|event| event.focused && windows.contains(event.window))
        && matches!(
            state.get(),
            AppState::Playing | AppState::Paused | AppState::Dead
        )
    {
        next_state.set(AppState::Editing);
    }
}

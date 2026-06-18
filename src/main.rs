mod config;
mod core;
mod editor;
mod gameplay;
mod input;
mod level;
mod player;
mod state;
mod ui;

#[macro_use]
mod macros;

use avian2d::prelude::{Gravity, PhysicsPlugins};
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::{UpdateMode, WinitPlugin, WinitSettings};
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use std::sync::OnceLock;
use std::time::Duration;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::Config;
use crate::core::camera::CameraPlugin;
use crate::editor::EditorPlugin;
use crate::gameplay::GameplayPlugin;
use crate::input::InputPlugin;
use crate::level::LevelPlugin;
use crate::player::PlayerPlugin;
use crate::state::{AppState, AppStatePlugin, EditorAvailability};
use crate::ui::UiPlugin;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn setup_logging(path: String) {
    let (writer, guard) = non_blocking(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap(),
    );

    LOG_GUARD.set(guard).unwrap();

    fmt()
        .with_writer(writer)
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();
}

fn main() {
    let config = Config::load();

    setup_logging(config.game.logfile.clone());

    let frame_wait = Duration::from_secs_f64(1.0 / config.game.fps.max(1.0));
    let graphics_enabled = config.game.graphics;

    let mut app = App::new();

    if graphics_enabled {
        app.add_plugins((
            DefaultPlugins
                .build()
                .set(ImagePlugin::default_linear())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .disable::<bevy::log::LogPlugin>(),
            EditorPlugin,
        ))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(frame_wait),
            unfocused_mode: UpdateMode::reactive(frame_wait),
        });
    } else {
        app.add_plugins((
            ScheduleRunnerPlugin::run_loop(frame_wait),
            DefaultPlugins
                .build()
                .set(ImagePlugin::default_linear())
                .disable::<WinitPlugin>()
                .disable::<bevy::log::LogPlugin>(),
        ));
    }

    app.add_plugins((
        RatatuiPlugins::default(),
        RatatuiCameraPlugin,
        PhysicsPlugins::default(),
        CameraPlugin,
        LevelPlugin,
        UiPlugin,
        PlayerPlugin,
        GameplayPlugin,
        InputPlugin,
        AppStatePlugin,
    ))
    .insert_resource(Gravity::ZERO)
    .insert_resource(EditorAvailability {
        graphical: graphics_enabled,
    })
    .insert_resource(config)
    .init_state::<AppState>()
    .run();
}

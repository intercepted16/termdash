mod config;
mod core;
mod editor;
mod gameplay;
mod input;
mod level;
mod paths;
mod player;
mod state;
mod ui;

#[macro_use]
mod macros;

use avian2d::prelude::{Gravity, PhysicsPlugins};
use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::{ExitCondition, WindowPlugin};
use bevy::winit::{UpdateMode, WinitPlugin, WinitSettings};
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::Config;
use crate::core::camera::CameraPlugin;
use crate::editor::EditorPlugin;
use crate::gameplay::GameplayPlugin;
use crate::input::InputPlugin;
use crate::level::LevelPlugin;
use crate::paths::GamePaths;
use crate::player::PlayerPlugin;
use crate::state::{AppState, AppStatePlugin};
use crate::ui::UiPlugin;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn setup_logging(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!("failed to create log directory {}: {err}", parent.display())
        });
    }

    let (writer, guard) = non_blocking(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap_or_else(|err| panic!("failed to open log file {}: {err}", path.display())),
    );

    LOG_GUARD.set(guard).unwrap();

    fmt()
        .with_writer(writer)
        .with_env_filter(EnvFilter::new("warn,termdash=debug"))
        .init();
}

fn main() {
    let paths = GamePaths::init().expect("paths should be returned");
    let config = Config::load(&paths).expect("config should load");

    setup_logging(&paths.data_file(&config.game.logfile));

    let frame_wait = Duration::from_secs_f64(1.0 / config.game.fps.max(1.0));
    let graphics = config.game.graphics;
    let asset_file_path = paths.data_dir.to_string_lossy().into_owned();

    let mut app = App::new();
    let default_plugins = DefaultPlugins
        .build()
        .set(ImagePlugin::default_linear())
        .set(AssetPlugin {
            file_path: asset_file_path,
            ..default()
        })
        .disable::<bevy::log::LogPlugin>();

    if graphics {
        app.add_plugins((
            default_plugins.set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            }),
            EditorPlugin,
        ))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(frame_wait),
            unfocused_mode: UpdateMode::reactive(frame_wait),
        });
    } else {
        warn!("not running in a graphical environment, editor will be disabled");
        app.add_plugins((
            ScheduleRunnerPlugin::run_loop(frame_wait),
            default_plugins.disable::<WinitPlugin>(),
        ));
    }

    app.insert_resource(paths)
        .insert_resource(config)
        .insert_resource(Gravity::ZERO)
        .init_state::<AppState>();

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
    .run();
}

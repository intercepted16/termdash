mod config;
mod core;
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
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;

use std::time::Duration;

use tracing_appender::non_blocking;
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::Config;
use crate::core::camera::CameraPlugin;
use crate::gameplay::GameplayPlugin;
use crate::input::InputPlugin;
use crate::level::LevelPlugin;
use crate::ui::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::state::AppState;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;

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

    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / config.game.fps)),
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<bevy::log::LogPlugin>(),
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
            PhysicsPlugins::default(),
            CameraPlugin,
            LevelPlugin,
            MenuPlugin,
            PlayerPlugin,
            GameplayPlugin,
            InputPlugin,
        ))
        .insert_resource(Gravity::ZERO)
        .insert_resource(config)
        .init_state::<AppState>()
        .run();
}

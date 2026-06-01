mod config;
mod core;
mod gameplay;
mod input;
mod level;
mod menu;
mod player;
mod state;

use crate::config::Config;
use crate::core::camera::CameraPlugin;
use crate::gameplay::GameplayPlugin;
use crate::level::WorldPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use std::time::Duration;

pub use state::AppState;

use crate::input::InputPlugin;

fn main() {
    let config = Config::load();
    let frame_duration = Duration::from_secs_f64(1.0 / config.game.fps);

    let mut app = App::new();

    app.add_plugins((
        ScheduleRunnerPlugin::run_loop(frame_duration),
        DefaultPlugins
            .build()
            .disable::<WinitPlugin>()
            .disable::<LogPlugin>(),
        RatatuiPlugins::default(),
        RatatuiCameraPlugin,
        CameraPlugin,
        WorldPlugin,
        MenuPlugin,
        PlayerPlugin,
        GameplayPlugin,
        InputPlugin,
    ))
    .insert_resource(config)
    .init_state::<AppState>()
    .run();
}

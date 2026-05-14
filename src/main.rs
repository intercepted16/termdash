mod config;
mod core;
mod features;
use crate::config::load_config;
use crate::core::{CameraPlugin, StatePlugin};
use crate::features::gameplay::GameplayPlugin;
use crate::features::menu::{MenuPlugin, MenuUiPlugin};
use crate::features::player::PlayerPlugin;
use crate::features::world::WorldPlugin;
use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use std::time::Duration;
fn main() {
    let config = load_config();
    let frame_duration = Duration::from_secs_f64(1.0 / config.game.fps);

    let mut app = App::new();
    app.insert_resource(config)
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(frame_duration),
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
            StatePlugin,
            CameraPlugin,
            WorldPlugin,
            MenuPlugin,
            PlayerPlugin,
            GameplayPlugin,
            MenuUiPlugin,
        ))
        .run();
}

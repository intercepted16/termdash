mod core;
mod features;

use crate::core::constants::FPS;
use crate::core::{CameraPlugin, StatePlugin};
use crate::features::gameplay::GameplayPlugin;
use crate::features::menu::{MenuPlugin, MenuUiPlugin};
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / FPS)),
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            RatatuiPlugins {
                enable_input_forwarding: true,
                ..default()
            },
            RatatuiCameraPlugin,
            StatePlugin,
            CameraPlugin,
            MenuPlugin,
            GameplayPlugin,
            MenuUiPlugin,
        ))
        .run();
}

mod core;
mod features;
use crate::core::{CameraPlugin, StatePlugin, constants::FPS};
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
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / FPS)),
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

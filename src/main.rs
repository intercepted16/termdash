mod config;
mod core;
mod gameplay;
mod input;
mod level;
mod menu;
mod player;
mod state;

#[macro_use]
mod macros;

use avian2d::prelude::{Gravity, PhysicsPlugins};
use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use std::time::Duration;

use crate::config::Config;
use crate::core::camera::CameraPlugin;
use crate::gameplay::GameplayPlugin;
use crate::input::InputPlugin;
use crate::level::LevelPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::state::AppState;

fn main() {
    let config = Config::load();

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

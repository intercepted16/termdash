mod camera;
mod constants;
mod player;
mod world;

use std::time::Duration;
use crate::constants::FPS;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;

use crate::camera::CameraPlugin;
use crate::player::cube::PlayerPlugin;
use crate::player::movement::MovementPlugin;
use crate::world::ground::GroundPlugin;

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / FPS)), // 60 fps
            CameraPlugin,
            GroundPlugin,
            PlayerPlugin,
            MovementPlugin,
        ))
        .run();
}

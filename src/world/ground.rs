use bevy::prelude::*;

use crate::constants::GROUND_HEIGHT;
use crate::world::solid::SolidBundle;

#[derive(Component)]
pub struct Ground;

pub const GROUND_PADDING: f32 = 40.0;

fn spawn_ground_strip(commands: &mut Commands, y: f32) {
    commands.spawn((
        Ground,
        SolidBundle::new(
            Vec3::new(0.0, y, 0.0),
            Vec2::new(1.0, GROUND_HEIGHT),
            Color::linear_rgb(0.2, 0.8, 0.2),
        ),
    ));
}

pub fn spawn_ground(mut commands: Commands) {
    spawn_ground_strip(&mut commands, 0.0);
}

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ground);
    }
}

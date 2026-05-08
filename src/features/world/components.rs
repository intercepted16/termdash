use bevy::prelude::*;

use crate::core::constants::{GROUND_HEIGHT, GROUND_Y};
use crate::features::world::solid::SolidBundle;

#[derive(Component)]
pub struct Ground;

pub const GROUND_PADDING: f32 = 40.0;

pub fn spawn_ground(mut commands: Commands, ground: Query<Entity, With<Ground>>) {
    if !ground.is_empty() {
        return;
    }

    commands.spawn((
        Ground,
        SolidBundle::new(
            Vec3::new(0.0, GROUND_Y, 0.0),
            Vec2::new(1.0, GROUND_HEIGHT),
            Color::linear_rgb(0.2, 0.8, 0.2),
        ),
    ));
}

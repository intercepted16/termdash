// cubeman (main character)
use crate::constants::{CUBE_SIZE, GROUND_HEIGHT, GROUND_Y};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity(pub Vec2);

fn setup(mut commands: Commands) {
    commands.spawn(make_player());
}

fn make_player() -> impl Bundle {
    (
        Player,
        player_transform(),
        player_sprite(),
        player_motion(),
    )
}

fn player_transform() -> Transform {
    Transform::from_xyz(0.0, ground_top() + CUBE_SIZE * 0.5, 0.0)
}

fn ground_top() -> f32 {
    GROUND_Y + GROUND_HEIGHT * 0.5
}

fn player_sprite() -> Sprite {
    Sprite {
        color: Color::WHITE,
        custom_size: Some(Vec2::new(CUBE_SIZE, CUBE_SIZE)),
        ..default()
    }
}

fn player_motion() -> Velocity {
    Velocity(Vec2::ZERO)
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

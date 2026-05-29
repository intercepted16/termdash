use crate::world::model::PlayerDef;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub gravity_dir: f32,
}

impl Player {
    pub fn bundle(def: &PlayerDef) -> impl Bundle {
        (
            Player::default(),
            Transform::from_translation(def.spawn.extend(0.0)),
            Sprite::from_color(def.color, def.size),
            Velocity(Vec2::ZERO),
        )
    }
}

impl Default for Player {
    fn default() -> Self {
        Self { gravity_dir: 1.0 }
    }
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

use crate::{level::model::PlayerDef, newtype};
use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub gravity_dir: Dir2,
    pub grounded_grace: f32,
}

impl Player {
    pub fn bundle(def: &PlayerDef) -> impl Bundle {
        (
            Player::default(),
            RigidBody::Kinematic,
            Collider::rectangle(def.size.x, def.size.y),
            Transform::from_translation(def.spawn.extend(0.0)),
            Sprite::from_color(def.color, def.size),
            Velocity(Vec2::ZERO),
        )
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            gravity_dir: Dir2::NEG_Y,
            grounded_grace: 0.0,
        }
    }
}

newtype! {
    #[derive(Component)]
    pub struct Velocity(pub Vec2);
}

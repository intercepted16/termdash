use crate::world::model::PlayerDef;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity(pub Vec2);

impl Player {
    pub fn bundle(def: &PlayerDef) -> impl Bundle {
        (
            Player,
            Transform::from_translation(def.spawn.extend(0.0)),
            Sprite::from_color(def.color, def.size),
            Velocity(Vec2::ZERO),
        )
    }
}

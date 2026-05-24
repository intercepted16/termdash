use crate::world::model::PlayerDefinition;
use bevy::prelude::*;
#[derive(Component)]
pub struct Player;
#[derive(Component)]
pub struct Velocity(pub Vec2);
pub fn make_player(player: &PlayerDefinition) -> impl Bundle {
    (
        Player,
        Transform::from_translation(player.spawn.as_vec2().extend(0.0)),
        Sprite::from_color(player.color.as_color(), player.size.as_vec2()),
        Velocity(Vec2::ZERO),
    )
}

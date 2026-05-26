use crate::world::model::{Ground, GroundSegment};
use bevy::prelude::*;

macro_rules! components {
    ($($component:ident),* $(,)?) => {
        $(#[derive(Component)] pub struct $component;)*
    };
}
components!(WorldEntity, Solid, WorldMusic, AudioVisualizerBar, JumpOrb);

#[derive(Component)]
pub struct HazardBox {
    pub half_size: Vec2,
}

pub fn make_ground_segment(ground: &Ground, segment: &GroundSegment) -> impl Bundle {
    (
        WorldEntity,
        make_solid_sprite(
            Vec3::new(segment.start_x + segment.width * 0.5, ground.y, 0.0),
            Vec2::new(segment.width, ground.height),
            ground.color,
        ),
    )
}
pub fn make_solid_sprite(position: Vec3, size: Vec2, color: Color) -> impl Bundle {
    (
        Solid,
        Transform::from_translation(position),
        Sprite::from_color(color, size),
    )
}

use crate::world::model::{GroundDefinition, GroundSegmentDefinition, WorldDefinition};
use bevy::prelude::*;
macro_rules! marker_components {
    ($($component:ident),* $(,)?) => {
        $(#[derive(Component)] pub struct $component;)*
    };
}
marker_components!(WorldEntity, Solid, WorldMusic, AudioVisualizerBar);

#[derive(Component)]
pub struct HazardBox {
    pub half_size: Vec2,
}

#[derive(Component)]
pub struct JumpOrb {
    pub radius: f32,
    pub strength_px: f32,
}

pub fn make_ground_segment(
    ground: &GroundDefinition,
    segment: &GroundSegmentDefinition,
) -> impl Bundle {
    (
        WorldEntity,
        make_solid_sprite(
            Vec3::new(segment.start_x + segment.width * 0.5, ground.y, 0.0),
            Vec2::new(segment.width, ground.height),
            ground.color,
        ),
    )
}
pub fn default_ground_segment(world: &WorldDefinition) -> GroundSegmentDefinition {
    GroundSegmentDefinition {
        start_x: 0.0,
        width: world.size.x,
    }
}
pub fn make_solid_sprite(position: Vec3, size: Vec2, color: Color) -> impl Bundle {
    (
        Solid,
        Transform::from_translation(position),
        Sprite::from_color(color, size),
    )
}

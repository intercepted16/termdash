use crate::features::world::model::{GroundDefinition, GroundSegmentDefinition, WorldDefinition};
use bevy::prelude::*;
macro_rules! marker_components {
    ($($component:ident),* $(,)?) => {
        $(#[derive(Component)] pub struct $component;)*
    };
}
marker_components!(
    WorldEntity,
    Ground,
    Obstacle,
    Spike,
    Hazard,
    Solid,
    WorldMusic,
    AudioVisualizerBar
);
pub fn make_ground_segment(
    ground: &GroundDefinition,
    segment: &GroundSegmentDefinition,
) -> impl Bundle {
    (
        WorldEntity,
        Ground,
        solid_sprite(
            Vec3::new(segment.start_x + segment.width * 0.5, ground.y, 0.0),
            Vec2::new(segment.width, ground.height),
            ground.color.as_color(),
        ),
    )
}
pub fn default_ground_segment(world: &WorldDefinition) -> GroundSegmentDefinition {
    GroundSegmentDefinition {
        start_x: 0.0,
        width: world.size.x,
    }
}
fn solid_sprite(position: Vec3, size: Vec2, color: Color) -> impl Bundle {
    (
        Solid,
        Transform::from_translation(position),
        Sprite::from_color(color, size),
    )
}

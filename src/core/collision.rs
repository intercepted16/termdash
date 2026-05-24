use bevy::math::bounding::{Aabb2d, BoundingVolume};
use bevy::prelude::*;

pub const GROUND_EPSILON: f32 = 0.05;

pub fn bounds_from_sprite(transform: &Transform, sprite: &Sprite) -> Aabb2d {
    Aabb2d::new(
        transform.translation.xy(),
        sprite.custom_size.unwrap() * 0.5,
    )
}
pub fn bounds_at(bounds: Aabb2d, center: Vec2) -> Aabb2d {
    Aabb2d::new(center, bounds.half_size())
}

pub fn overlaps_x(a: Aabb2d, b: Aabb2d) -> bool {
    (a.center().x - b.center().x).abs() <= a.half_size().x + b.half_size().x
}

pub fn overlaps_y(a: Aabb2d, b: Aabb2d) -> bool {
    a.min.y < b.max.y - GROUND_EPSILON && a.max.y > b.min.y + GROUND_EPSILON
}

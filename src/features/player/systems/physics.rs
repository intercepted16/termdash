use bevy::math::bounding::{Aabb2d, BoundingVolume};
use bevy::prelude::*;
const DEFAULT_SPRITE_SIZE: f32 = 32.0;
pub fn bounds_from_sprite(transform: &Transform, sprite: &Sprite) -> Aabb2d {
    Aabb2d::new(
        transform.translation.xy(),
        sprite
            .custom_size
            .unwrap_or(Vec2::splat(DEFAULT_SPRITE_SIZE))
            * 0.5,
    )
}
pub fn bounds_at(bounds: Aabb2d, center: Vec2) -> Aabb2d {
    Aabb2d::new(center, bounds.half_size())
}
pub fn overlaps_x(a: Aabb2d, b: Aabb2d) -> bool {
    (a.center().x - b.center().x).abs() <= a.half_size().x + b.half_size().x
}
pub fn intersects(a: Aabb2d, b: Aabb2d) -> bool {
    overlaps_x(a, b) && (a.center().y - b.center().y).abs() <= a.half_size().y + b.half_size().y
}

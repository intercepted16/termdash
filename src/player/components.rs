use crate::{level::model::PlayerDef, newtype};
use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub gravity_dir: Dir2,
    pub grounded_grace: f32,
    pub jump_buffer: f32,
    pub base_size: Vec2,
    pub size_scale: f32,
    pub scroll_speed_multiplier: f32,
}

impl Player {
    pub fn bundle(def: &PlayerDef) -> impl Bundle {
        (
            Player {
                base_size: def.size,
                ..default()
            },
            RigidBody::Kinematic,
            Collider::rectangle(def.size.x, def.size.y),
            Transform::from_translation(def.spawn.extend(0.0)),
            Sprite::from_color(def.color, def.size),
            Velocity(Vec2::ZERO),
        )
    }

    pub fn size(&self) -> Vec2 {
        self.base_size * self.size_scale
    }

    pub fn add_vertical_speed(
        &self,
        velocity: &mut Velocity,
        speed_px: f32,
        level_units_per_pixel: f32,
    ) {
        let away_from_gravity = -self.gravity_dir.as_vec2();
        let current = velocity.0.dot(away_from_gravity);
        let minimum = speed_px * level_units_per_pixel;

        velocity.0 += away_from_gravity * (current.max(minimum) - current);
    }

    pub fn set_size_scale(
        &mut self,
        transform: &mut Transform,
        sprite: &mut Sprite,
        scale: f32,
    ) -> Option<Collider> {
        let scale = scale.clamp(0.5, 1.0);
        let old_size = self.size();

        if (self.size_scale - scale).abs() <= f32::EPSILON {
            return None;
        }

        self.size_scale = scale;
        let new_size = self.size();

        // Keep the active floor/ceiling contact visually stable when changing size.
        transform.translation +=
            (self.gravity_dir.as_vec2() * (old_size.y - new_size.y) * 0.5).extend(0.0);
        sprite.custom_size = Some(new_size);

        Some(Collider::rectangle(new_size.x, new_size.y))
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            gravity_dir: Dir2::NEG_Y,
            grounded_grace: 0.0,
            jump_buffer: 0.0,
            base_size: Vec2::new(32.0, 32.0),
            size_scale: 1.0,
            scroll_speed_multiplier: 1.0,
        }
    }
}

newtype! {
    #[derive(Component)]
    pub struct Velocity(pub Vec2);
}

use crate::gameplay::triggers::{TriggerActivation, TriggerEffect};
use crate::{components, newtype};
use avian2d::collision::collider::ColliderConstructor;
use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use level_data_macros::level_data;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct LevelDataRegistration(pub fn(&mut App));

inventory::collect!(LevelDataRegistration);

pub fn register_level_data_types(app: &mut App) {
    for registration in inventory::iter::<LevelDataRegistration> {
        registration.0(app);
    }
}

#[level_data(Default)]
pub struct Level {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub height: f32,
    pub scroll_speed_px: f32,
    pub player: PlayerDef,
    pub ground: Ground,
    #[serde(default)]
    pub objects: Vec<LevelObject>,
    pub music_path: Option<String>,
    pub audio_visualizer: Option<AudioVisualizer>,
}

impl Level {
    pub fn end_x(&self) -> f32 {
        self.ground
            .segments
            .iter()
            .map(|s| s.start_x + s.width)
            .fold(0.0, f32::max)
    }
}

#[level_data(Default)]
pub struct PlayerDef {
    pub spawn: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[level_data(Default)]
pub struct Ground {
    pub y: f32,
    pub height: f32,
    pub color: Color,
    pub segments: Vec<GroundSegment>,
}

#[level_data(Default)]
pub struct AudioVisualizer {
    #[serde(default)]
    pub bar_count: usize,
}

#[level_data]
pub struct GroundSegment {
    pub start_x: f32,
    pub width: f32,
}

impl GroundSegment {
    pub fn make(&self, ground: &Ground) -> impl Bundle {
        (
            LevelEntity,
            Solid,
            RigidBody::Static,
            Collider::rectangle(self.width, ground.height),
            Transform::from_translation(Vec3::new(self.start_x + self.width * 0.5, ground.y, 0.0)),
            Sprite::from_color(ground.color, Vec2::new(self.width, ground.height)),
        )
    }
}

newtype! {
#[derive(Resource)]
pub struct Prefabs(pub HashMap<String, Prefab>);
}

fn default_scale() -> f32 {
    1.0
}

fn is_default_scale(scale: &f32) -> bool {
    *scale == default_scale()
}

#[level_data(Default)]
pub struct LevelObject {
    pub prefab: Option<String>,
    pub position: Vec2,
    #[serde(default = "default_scale", skip_serializing_if = "is_default_scale")]
    pub scale: f32,
    pub color: Option<Color>,
    pub visual: Option<Visual>,
    pub collider: Option<ColliderConstructor>,
    pub behavior: Option<ObjectBehavior>,
}

#[derive(Deserialize)]
pub struct Prefab {
    #[serde(default)]
    pub color: Option<Color>,
    pub visual: Visual,
    pub collider: Option<ColliderConstructor>,
    pub behavior: Option<ObjectBehavior>,
}

newtype! {
#[level_data]
pub struct ObjectShape(pub ColliderConstructor);
}

#[level_data(PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectBehavior {
    Solid,
    Trigger {
        activation: TriggerActivation,
        effect: TriggerEffect,
    },
}

#[level_data]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Visual {
    Shape {
        shape: ObjectShape,
        #[serde(default)]
        animations: Vec<ObjectAnimation>,
    },
    Sprite {
        path: String,
        #[serde(default)]
        animations: Vec<ObjectAnimation>,
    },
}

#[derive(Clone, Debug)]
pub struct ResolvedObject {
    pub color: Option<Color>,
    pub visual: Visual,
    pub collider: ColliderConstructor,
    pub behavior: ObjectBehavior,
}

#[level_data]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectAnimation {
    Spin { degrees_per_second: f32 },
}

components!(
    LevelEntity,
    Solid,
    KillPlayerOnSide,
    LevelMusic,
    AudioVisualizerBar,
);

newtype! {
#[derive(Component, Clone)]
pub struct ObjectAnimator(pub Vec<ObjectAnimation>);
}

use crate::gameplay::triggers::{TriggerActivation, TriggerEffect};
use crate::{components, newtype};
use avian2d::collision::collider::ColliderConstructor;
use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Level {
    pub name: String,
    pub description: String,
    pub size: Vec2,
    pub scroll_speed_px: f32,
    pub player: PlayerDef,
    pub ground: Ground,
    #[serde(default)]
    pub objects: Vec<LevelObject>,
    pub music_path: Option<String>,
    #[serde(default)]
    pub audio_visualizer: Option<AudioVisualizer>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerDef {
    pub spawn: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Ground {
    pub y: f32,
    pub height: f32,
    pub color: Color,
    pub segments: Vec<GroundSegment>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AudioVisualizer {
    #[serde(default)]
    pub bar_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct LevelObject {
    pub position: Vec2,
    #[serde(default = "default_scale")]
    pub scale: f32, // scale size relative to the defined prefab size
    pub color: Option<Color>,
    #[serde(default)]
    pub prefab: Option<String>,
    pub visual: Option<Visual>,
    pub collider: Option<ColliderConstructor>,
    pub behavior: Option<ObjectBehavior>,
}

#[derive(Deserialize)]
pub struct Prefab {
    pub visual: Visual,
    pub collider: Option<ColliderConstructor>,
    pub behavior: Option<ObjectBehavior>,
}

newtype! {
#[derive(Deserialize, Clone, Debug)]
pub struct ObjectShape(pub ColliderConstructor);
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObjectBehavior {
    Solid,
    Trigger {
        activation: TriggerActivation,
        effect: TriggerEffect,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Visual {
    Shape { shape: ObjectShape },
    Sprite { path: String },
    Scene { path: String },
}

#[derive(Clone, Debug)]
pub struct ResolvedObject {
    pub visual: Visual,
    pub collider: ColliderConstructor,
    pub behavior: ObjectBehavior,
}

components!(
    LevelEntity,
    Solid,
    KillPlayerOnSide,
    LevelMusic,
    AudioVisualizerBar
);

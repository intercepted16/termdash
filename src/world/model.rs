use bevy::color::Color;
use bevy::prelude::*;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize)]
pub struct Level {
    pub name: String,
    pub description: String,
    pub size: Vec2,
    pub scroll_speed_px: f32,
    pub player: PlayerDef,
    pub ground: Ground,
    #[serde(default)]
    pub objects: Vec<WorldObject>,
    pub music_path: Option<String>, // relative to assets/
    #[serde(default)]
    pub audio_visualizer: Option<AudioVisualizer>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerDef {
    pub spawn: Vec2,
    pub size: Vec2,
    pub color: Color,
}

fn require_elements<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let vec = Vec::<T>::deserialize(deserializer)?;
    if vec.is_empty() {
        return Err(D::Error::custom("cannot be empty"));
    }
    Ok(vec)
}
#[derive(Clone, Debug, Deserialize)]
pub struct Ground {
    pub y: f32,
    pub height: f32,
    pub color: Color,
    #[serde(deserialize_with = "require_elements")]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorldObject {
    Solid(Solid),
    Spike(Spike),
    JumpOrb(JumpOrbDef),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Solid {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Spike {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Debug, Deserialize, Component)]
pub struct JumpOrbDef {
    pub position: Vec2,
    pub radius: f32,
    pub color: Color,
    pub strength_px: f32,
}

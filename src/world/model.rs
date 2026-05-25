use bevy::color::Color;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct WorldDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size: Vec2,
    pub scroll_speed_px: f32,
    pub player: PlayerDefinition,
    pub ground: GroundDefinition,
    #[serde(default)]
    pub objects: Vec<WorldObjectDefinition>,
    pub music_path: Option<String>, // relative to assets/
    #[serde(default)]
    pub audio_visualizer: Option<AudioVisualizerDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayerDefinition {
    pub spawn: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroundDefinition {
    pub y: f32,
    pub height: f32,
    pub color: Color,
    #[serde(default)]
    pub segments: Vec<GroundSegmentDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AudioVisualizerDefinition {
    #[serde(default)]
    pub bar_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroundSegmentDefinition {
    pub start_x: f32,
    pub width: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorldObjectDefinition {
    Solid(SolidObjectDefinition),
    Spike(SpikeObjectDefinition),
    JumpOrb(JumpOrbObjectDefinition),
}

#[derive(Clone, Debug, Deserialize)]
pub struct SolidObjectDefinition {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpikeObjectDefinition {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JumpOrbObjectDefinition {
    pub position: Vec2,
    pub radius: f32,
    pub color: Color,
    pub strength_px: f32,
}

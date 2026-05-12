use bevy::prelude::*;
use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
pub struct WorldDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size: Vec2Def,
    pub scroll_speed_px: f32,
    pub player: PlayerDefinition,
    pub ground: GroundDefinition,
    #[serde(default)]
    pub objects: Vec<WorldObjectDefinition>,
    pub music_path: Option<String>, // relative to assets/
}
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}
impl Vec2Def {
    pub fn as_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ColorDef {
    #[serde(default)]
    pub r: f32,
    #[serde(default)]
    pub g: f32,
    #[serde(default)]
    pub b: f32,
    #[serde(default = "opaque")]
    pub a: f32,
}
fn opaque() -> f32 {
    1.0
}
impl ColorDef {
    pub fn as_color(self) -> Color {
        Color::linear_rgba(self.r, self.g, self.b, self.a)
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct PlayerDefinition {
    pub spawn: Vec2Def,
    pub size: Vec2Def,
    pub color: ColorDef,
}
#[derive(Clone, Debug, Deserialize)]
pub struct GroundDefinition {
    pub y: f32,
    pub height: f32,
    pub color: ColorDef,
    #[serde(default)]
    pub segments: Vec<GroundSegmentDefinition>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct GroundSegmentDefinition {
    pub start_x: f32,
    pub width: f32,
}
#[derive(Clone, Debug, Deserialize)]
pub struct WorldObjectDefinition {
    pub kind: WorldObjectKind,
    pub position: Vec2Def,
    pub size: Vec2Def,
    pub color: ColorDef,
}
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldObjectKind {
    Solid,
    Spike,
}

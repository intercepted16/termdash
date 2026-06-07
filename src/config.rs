use bevy::prelude::Resource;
use serde::Deserialize;

#[derive(Debug, Deserialize, Resource)]
pub struct Config {
    pub game: GameConfig,
    pub camera: CameraConfig,
    pub player: PlayerConfig,
    pub visualizer: VisualizerConfig,
}

impl Config {
    pub fn load() -> Self {
        toml::from_str(include_str!("../assets/config.toml"))
            .expect("failed to parse assets/config.toml")
    }
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub fps: f64,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub zoom: f32,
    pub bottom_margin_fraction: f32,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub gravity_px: f32,
    pub jump_speed_px: f32,
    pub death_pause_seconds: f32,
}

#[derive(Debug, Deserialize)]
pub struct VisualizerConfig {
    pub enabled: bool,
}

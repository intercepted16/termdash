use std::fs;

use bevy::prelude::Resource;
use serde::Deserialize;

use crate::paths::GamePaths;

#[derive(Debug, Deserialize, Resource)]
pub struct Config {
    pub game: GameConfig,
    pub camera: CameraConfig,
    pub player: PlayerConfig,
    pub visualizer: VisualizerConfig,
}

impl Config {
    pub fn load(paths: &GamePaths) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(&paths.config("config.toml"))?;
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub fps: f64,
    pub logfile: String, // relative to working dir
    pub graphics: bool,
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

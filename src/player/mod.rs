pub mod components;
mod movement;
pub mod queries;
use crate::AppState;
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use ratatui::crossterm::event::{KeyCode as TerminalKeyCode, KeyEventKind};

pub use crate::player::movement::move_player;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player.run_if(in_state(AppState::Playing)));
    }
}

pub fn jump_pressed(keys: &mut MessageReader<KeyMessage>) -> bool {
    keys.read().any(|key| {
        matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            && key.code == TerminalKeyCode::Up
    })
}

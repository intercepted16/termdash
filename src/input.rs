// Input helpers
use bevy::prelude::*;
use bevy::prelude::{MessageReader, ResMut, Resource};
use bevy_ratatui::event::KeyMessage;
use ratatui::crossterm::event::{KeyCode as TerminalKeyCode, KeyEventKind};
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct InputState {
    held: HashSet<TerminalKeyCode>,
    pressed: HashSet<TerminalKeyCode>,
}

pub fn update_input_state(mut keys: MessageReader<KeyMessage>, mut input: ResMut<InputState>) {
    input.pressed.clear();

    for key in keys.read() {
        match key.kind {
            KeyEventKind::Press => {
                input.held.insert(key.code);
                input.pressed.insert(key.code);
            }
            KeyEventKind::Release => {
                input.held.remove(&key.code);
            }
            _ => {}
        }
    }
}

pub fn just_pressed(input: &InputState, key: TerminalKeyCode) -> bool {
    input.pressed.contains(&key)
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(PreUpdate, update_input_state);
    }
}

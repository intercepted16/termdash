use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use ratatui::crossterm::event::{KeyCode as TerminalKeyCode, KeyEventKind};
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct InputState {
    pressed: HashSet<TerminalKeyCode>,
}

impl InputState {
    pub fn just_pressed(&self, key: TerminalKeyCode) -> bool {
        self.pressed.contains(&key)
    }
}

fn update(mut keys: MessageReader<KeyMessage>, mut input: ResMut<InputState>) {
    input.pressed.clear();

    for key in keys.read() {
        if key.kind == KeyEventKind::Press {
            input.pressed.insert(key.code);
        }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .add_systems(PreUpdate, update);
    }
}

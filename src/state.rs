use bevy::prelude::*;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;

use crate::input::InputState;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    Dead,
    Editing,
}

#[derive(Resource, Default)]
pub struct RuntimeFeatures {
    pub graphics: bool,
}

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, app_state_input);
    }
}

fn app_state_input(
    input: Res<InputState>,
    editor: Res<RuntimeFeatures>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    match state.get() {
        AppState::Playing => {
            if input.just_pressed(TerminalKeyCode::Esc) {
                next.set(AppState::Paused);
            }

            if input.just_pressed(TerminalKeyCode::Char('e')) && editor.graphics {
                next.set(AppState::Editing);
            }
        }

        AppState::Paused => {
            if input.just_pressed(TerminalKeyCode::Esc) {
                next.set(AppState::Playing);
            }

            if input.just_pressed(TerminalKeyCode::Enter) {
                next.set(AppState::MainMenu);
            }

            if input.just_pressed(TerminalKeyCode::Char('e')) && editor.graphics {
                next.set(AppState::Editing);
            }
        }

        AppState::Editing => {
            if input.just_pressed(TerminalKeyCode::Char('e')) {
                next.set(AppState::Playing);
            }
        }

        _ => {}
    }
}

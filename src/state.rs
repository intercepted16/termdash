use bevy::prelude::*;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;

use crate::config::Config;
use crate::input::InputState;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    Dead,
    DeathPaused,
    Editing,
    Victory,
}

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, app_state_input);
    }
}

fn app_state_input(
    input: Res<InputState>,
    config: Res<Config>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let esc = input.just_pressed(TerminalKeyCode::Esc);
    let enter = input.just_pressed(TerminalKeyCode::Enter);
    let edit = input.just_pressed(TerminalKeyCode::Char('e'));

    let target = match state.get() {
        AppState::Playing if edit && config.game.graphics => Some(AppState::Editing),
        AppState::Playing if esc => Some(AppState::Paused),
        AppState::Paused if edit && config.game.graphics => Some(AppState::Editing),
        AppState::Paused if enter => Some(AppState::MainMenu),
        AppState::Paused if esc => Some(AppState::Playing),
        AppState::Dead if esc => Some(AppState::DeathPaused),
        AppState::DeathPaused if enter => Some(AppState::MainMenu),
        AppState::DeathPaused if esc => Some(AppState::Dead),
        AppState::Editing if edit => Some(AppState::Playing),
        AppState::Victory if enter => Some(AppState::MainMenu),
        _ => None,
    };

    if let Some(target) = target {
        next.set(target);
    }
}

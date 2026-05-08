use bevy::prelude::*;

use crate::core::app_state::AppState;
use crate::features::menu::resources::MenuState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(OnEnter(AppState::MainMenu), clear_keyboard_input)
            .add_systems(OnEnter(AppState::Playing), clear_keyboard_input)
            .add_systems(OnEnter(AppState::Paused), clear_keyboard_input)
            .add_systems(Update, main_menu_input.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, pause_input.run_if(in_state(AppState::Playing)))
            .add_systems(Update, paused_menu_input.run_if(in_state(AppState::Paused)));
    }
}

fn clear_keyboard_input(mut keyboard: ResMut<ButtonInput<KeyCode>>) {
    keyboard.clear();
}

fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        menu.select_previous();
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        menu.select_next();
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

fn pause_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Paused);
    }
}

fn paused_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Playing);
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::MainMenu);
    }
}

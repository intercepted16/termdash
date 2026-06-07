use crate::AppState;
use crate::input::InputState;
use crate::level::components::LevelMusic;
use crate::level::load::LoadWorldEvent;
use crate::level::registry::Levels;
use crate::menu::resources::MenuState;
use crate::menu::ui::render;
use bevy::prelude::*;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(Update, main_menu_input.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, pause_input.run_if(in_state(AppState::Playing)))
            .add_systems(Update, paused_menu_input.run_if(in_state(AppState::Paused)))
            .add_systems(OnEnter(AppState::Paused), music_playing::<true>)
            .add_systems(
                OnTransition {
                    exited: AppState::Paused,
                    entered: AppState::Playing,
                },
                music_playing::<false>,
            )
            .add_systems(PostUpdate, render);
    }
}

fn main_menu_input(
    input: Res<InputState>,
    mut menu: ResMut<MenuState>,
    level_registry: Res<Levels>,
    mut load_world_events: MessageWriter<LoadWorldEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if input.just_pressed(TerminalKeyCode::Up) {
        menu.previous();
    }

    if input.just_pressed(TerminalKeyCode::Down) {
        menu.next(level_registry.len());
    }

    if input.just_pressed(TerminalKeyCode::Enter) {
        load_world_events.write(LoadWorldEvent {
            index: menu.selected_world,
        });

        next_state.set(AppState::Playing);
    }
}

fn pause_input(input: Res<InputState>, mut next_state: ResMut<NextState<AppState>>) {
    if input.just_pressed(TerminalKeyCode::Esc) {
        next_state.set(AppState::Paused);
    }
}

fn paused_menu_input(input: Res<InputState>, mut next_state: ResMut<NextState<AppState>>) {
    if input.just_pressed(TerminalKeyCode::Esc) {
        next_state.set(AppState::Playing);
    }

    if input.just_pressed(TerminalKeyCode::Enter) {
        next_state.set(AppState::MainMenu);
    }
}

fn music_playing<const PAUSED: bool>(music: Query<&AudioSink, With<LevelMusic>>) {
    for sink in &music {
        if PAUSED {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

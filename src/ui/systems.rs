use crate::AppState;
use crate::input::InputState;
use crate::level::load::LoadLevelEvent;
use crate::level::model::LevelMusic;
use crate::level::registry::Levels;
use crate::ui::model::LevelMenu;
use crate::ui::render;
use bevy::prelude::*;
use ratatui::crossterm::event::KeyCode as TerminalKeyCode;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelMenu>()
            .add_systems(Update, main_menu_input.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnEnter(AppState::Paused), music_playing::<true>)
            .add_systems(OnEnter(AppState::Editing), music_playing::<true>)
            .add_systems(
                OnTransition {
                    exited: AppState::Paused,
                    entered: AppState::Playing,
                },
                music_playing::<false>,
            )
            .add_systems(
                OnTransition {
                    exited: AppState::Editing,
                    entered: AppState::Playing,
                },
                music_playing::<false>,
            )
            .add_systems(PostUpdate, render);
    }
}

fn main_menu_input(
    input: Res<InputState>,
    mut menu: ResMut<LevelMenu>,
    mut levels: ResMut<Levels>,
    mut load_level_events: MessageWriter<LoadLevelEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if menu.confirm_delete {
        if input.just_pressed(TerminalKeyCode::Enter) {
            if let Err(err) = levels.remove(menu.selected) {
                error!("could not delete level {}: {err}", menu.selected);
            } else {
                menu.selected = menu.selected.min(levels.len().saturating_sub(1));
            }

            menu.confirm_delete = false;
        } else if input.just_pressed(TerminalKeyCode::Esc) {
            menu.confirm_delete = false;
        }

        return;
    }

    if input.just_pressed(TerminalKeyCode::Up) {
        menu.previous();
    }

    if input.just_pressed(TerminalKeyCode::Down) {
        menu.next(levels.len());
    }

    if input.just_pressed(TerminalKeyCode::Char('+')) {
        let index = match levels.save_new() {
            Ok(index) => index,
            Err(err) => {
                error!("could not create a new level: {err}");
                return;
            }
        };

        menu.selected = index;
        load_level_events.write(LoadLevelEvent { index });
        next_state.set(AppState::Editing);
        return;
    }

    if input.just_pressed(TerminalKeyCode::Char('-')) && !levels.is_empty() {
        menu.confirm_delete = true;
    }

    if input.just_pressed(TerminalKeyCode::Enter) {
        load_level_events.write(LoadLevelEvent {
            index: menu.selected,
        });
        next_state.set(AppState::Playing);
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

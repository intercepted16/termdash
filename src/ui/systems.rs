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
            .add_message::<NewLevelEvent>()
            .add_message::<DeleteLevelEvent>()
            .add_systems(
                Update,
                (main_menu_input, new_level, delete_level)
                    .chain()
                    .run_if(in_state(AppState::MainMenu)),
            )
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

#[derive(Message)]
struct NewLevelEvent;

#[derive(Message)]
struct DeleteLevelEvent {
    index: usize,
}

fn main_menu_input(
    input: Res<InputState>,
    mut menu: ResMut<LevelMenu>,
    levels: Res<Levels>,
    mut new_level_events: MessageWriter<NewLevelEvent>,
    mut delete_level_events: MessageWriter<DeleteLevelEvent>,
    mut load_level_events: MessageWriter<LoadLevelEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if menu.confirm_delete {
        if input.just_pressed(TerminalKeyCode::Enter) {
            delete_level_events.write(DeleteLevelEvent {
                index: menu.selected,
            });
            menu.confirm_delete = false;
        }

        if input.just_pressed(TerminalKeyCode::Esc) {
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
        new_level_events.write(NewLevelEvent);
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

fn new_level(
    mut events: MessageReader<NewLevelEvent>,
    mut levels: ResMut<Levels>,
    mut menu: ResMut<LevelMenu>,
    mut load_level_events: MessageWriter<LoadLevelEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ in events.read() {
        let index = match levels.save_new() {
            Ok(index) => index,
            Err(err) => {
                error!("could not create a new level: {err}");
                continue;
            }
        };

        menu.selected = index;
        load_level_events.write(LoadLevelEvent { index });
        next_state.set(AppState::Editing);
    }
}

fn delete_level(
    mut events: MessageReader<DeleteLevelEvent>,
    mut levels: ResMut<Levels>,
    mut menu: ResMut<LevelMenu>,
) {
    for event in events.read() {
        if let Err(err) = levels.remove(event.index) {
            error!("could not delete level {}: {err}", event.index);
            continue;
        }

        menu.selected = menu.selected.min(levels.len().saturating_sub(1));
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

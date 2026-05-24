use crate::AppState;
use crate::config::Config;
use crate::menu::resources::MenuState;
use crate::world::components::WorldMusic;
use crate::world::loading::LoadWorldEvent;
use crate::world::registry::WorldRegistry;
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use ratatui::crossterm::event::{KeyCode as TerminalKeyCode, KeyEventKind};
pub struct MenuPlugin;
#[derive(Resource, Default)]
struct MenuKeyLatch {
    esc_cooldown: f32,
    enter_cooldown: f32,
}
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<MenuKeyLatch>()
            .add_systems(Update, main_menu_input.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, pause_input.run_if(in_state(AppState::Playing)))
            .add_systems(Update, paused_menu_input.run_if(in_state(AppState::Paused)))
            .add_systems(OnEnter(AppState::Paused), set_world_music_paused::<true>)
            .add_systems(
                OnTransition {
                    exited: AppState::Paused,
                    entered: AppState::Playing,
                },
                set_world_music_paused::<false>,
            );
    }
}
fn tick_cooldown(value: &mut f32, dt: f32) {
    *value = (*value - dt).max(0.0);
}
fn consume_cooldown(value: &mut f32, reset_seconds: f32) -> bool {
    if *value != 0.0 {
        return false;
    }
    *value = reset_seconds;
    true
}
fn read_key_presses(keys: &mut MessageReader<KeyMessage>) -> Vec<TerminalKeyCode> {
    keys.read()
        .filter(|key| matches!(key.kind, KeyEventKind::Press))
        .map(|key| key.code)
        .collect()
}
fn main_menu_input(
    timing: (Res<Config>, Res<Time>),
    mut keys: MessageReader<KeyMessage>,
    mut key_latch: ResMut<MenuKeyLatch>,
    mut menu: ResMut<MenuState>,
    world_registry: Res<WorldRegistry>,
    mut load_world_events: MessageWriter<LoadWorldEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let (config, time) = timing;

    tick_cooldown(&mut key_latch.enter_cooldown, time.delta_secs());

    let pressed = read_key_presses(&mut keys);

    if pressed.contains(&TerminalKeyCode::Up) {
        menu.select_previous();
    }
    if pressed.contains(&TerminalKeyCode::Down) {
        menu.select_next(world_registry.worlds.len());
    }

    if pressed.contains(&TerminalKeyCode::Enter)
        && consume_cooldown(
            &mut key_latch.enter_cooldown,
            config.menu.key_cooldown_seconds,
        )
    {
        load_world_events.write(LoadWorldEvent {
            index: menu.selected_world,
        });
        next_state.set(AppState::Playing);
    }
}
fn pause_input(
    config: Res<Config>,
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    mut key_latch: ResMut<MenuKeyLatch>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    tick_cooldown(&mut key_latch.esc_cooldown, time.delta_secs());
    if read_key_presses(&mut keys).contains(&TerminalKeyCode::Esc)
        && consume_cooldown(
            &mut key_latch.esc_cooldown,
            config.menu.key_cooldown_seconds,
        )
    {
        next_state.set(AppState::Paused);
    }
}
fn paused_menu_input(
    config: Res<Config>,
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    mut key_latch: ResMut<MenuKeyLatch>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    tick_cooldown(&mut key_latch.esc_cooldown, time.delta_secs());
    tick_cooldown(&mut key_latch.enter_cooldown, time.delta_secs());
    let pressed = read_key_presses(&mut keys);
    if pressed.contains(&TerminalKeyCode::Esc)
        && consume_cooldown(
            &mut key_latch.esc_cooldown,
            config.menu.key_cooldown_seconds,
        )
    {
        next_state.set(AppState::Playing);
    }
    if pressed.contains(&TerminalKeyCode::Enter)
        && consume_cooldown(
            &mut key_latch.enter_cooldown,
            config.menu.key_cooldown_seconds,
        )
    {
        next_state.set(AppState::MainMenu);
    }
}
fn set_world_music_paused<const PAUSED: bool>(music: Query<&AudioSink, With<WorldMusic>>) {
    for sink in &music {
        if PAUSED {
            sink.pause();
        } else {
            sink.play();
        }
    }
}

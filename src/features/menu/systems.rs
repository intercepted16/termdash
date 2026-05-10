use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use ratatui::crossterm::event::{KeyCode as TerminalKeyCode, KeyEventKind};

use crate::core::app_state::AppState;
use crate::features::menu::resources::MenuState;
use crate::features::world::loading::LoadWorldEvent;
use crate::features::world::registry::WorldRegistry;

pub struct MenuPlugin;

const KEY_COOLDOWN_SECONDS: f32 = 0.2;

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
            .add_systems(Update, paused_menu_input.run_if(in_state(AppState::Paused)));
    }
}

fn tick_cooldown(value: &mut f32, dt: f32) {
    *value = (*value - dt).max(0.0);
}

fn consume_cooldown(value: &mut f32) -> bool {
    if *value != 0.0 {
        return false;
    }

    *value = KEY_COOLDOWN_SECONDS;
    true
}

fn read_key_presses(keys: &mut MessageReader<KeyMessage>) -> Vec<TerminalKeyCode> {
    keys.read()
        .filter(|key| matches!(key.kind, KeyEventKind::Press))
        .map(|key| key.code)
        .collect()
}

fn main_menu_input(
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    mut menu: ResMut<MenuState>,
    mut key_latch: ResMut<MenuKeyLatch>,
    world_registry: Res<WorldRegistry>,
    mut load_world_events: MessageWriter<LoadWorldEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    tick_cooldown(&mut key_latch.enter_cooldown, time.delta_secs());
    let pressed = read_key_presses(&mut keys);

    if pressed.contains(&TerminalKeyCode::Up) {
        menu.select_previous();
    }

    if pressed.contains(&TerminalKeyCode::Down) {
        menu.select_next(world_registry.worlds.len());
    }

    if pressed.contains(&TerminalKeyCode::Enter) && consume_cooldown(&mut key_latch.enter_cooldown)
    {
        load_world_events.write(LoadWorldEvent {
            index: menu.selected_world,
        });
        next_state.set(AppState::Playing);
    }
}

fn pause_input(
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    mut key_latch: ResMut<MenuKeyLatch>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    tick_cooldown(&mut key_latch.esc_cooldown, time.delta_secs());
    if read_key_presses(&mut keys).contains(&TerminalKeyCode::Esc)
        && consume_cooldown(&mut key_latch.esc_cooldown)
    {
        next_state.set(AppState::Paused);
    }
}

fn paused_menu_input(
    time: Res<Time>,
    mut keys: MessageReader<KeyMessage>,
    mut key_latch: ResMut<MenuKeyLatch>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    tick_cooldown(&mut key_latch.esc_cooldown, time.delta_secs());
    tick_cooldown(&mut key_latch.enter_cooldown, time.delta_secs());
    let pressed = read_key_presses(&mut keys);

    if pressed.contains(&TerminalKeyCode::Esc) && consume_cooldown(&mut key_latch.esc_cooldown) {
        next_state.set(AppState::Playing);
    }

    if pressed.contains(&TerminalKeyCode::Enter) && consume_cooldown(&mut key_latch.enter_cooldown)
    {
        next_state.set(AppState::MainMenu);
    }
}

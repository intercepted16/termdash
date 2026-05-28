use crate::gameplay::death::DeathPause;
use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui_camera::RatatuiCameraWidget;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget};
use std::rc::Rc;

use crate::AppState;
use crate::menu::resources::MenuState;
use crate::world::registry::LevelRegistry;

pub struct MenuUiPlugin;

impl Plugin for MenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            render_main_menu.run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(PostUpdate, render_game.run_if(in_state(AppState::Playing)))
        .add_systems(
            PostUpdate,
            render_dead_game.run_if(in_state(AppState::Dead)),
        )
        .add_systems(
            PostUpdate,
            render_paused_game.run_if(in_state(AppState::Paused)),
        );
    }
}

fn render_main_menu(
    mut ratatui: ResMut<RatatuiContext>,
    menu: Res<MenuState>,
    world_registry: Res<LevelRegistry>,
) {
    let _ = ratatui.draw(|frame| {
        let layout = menu_layout(frame.area());

        frame.render_widget(
            Paragraph::new(title_text(layout[0].width as usize))
                .alignment(Alignment::Center)
                .style(highlight_style()),
            layout[0],
        );
        frame.render_stateful_widget(
            world_list_widget(&world_registry),
            layout[1],
            &mut list_state(&menu, &world_registry),
        );
        frame.render_widget(details_widget(&menu, &world_registry), layout[2]);
        frame.render_widget(
            help_widget("Up/Down select  |  Enter play  |  Esc menu during run"),
            layout[3],
        );
    });
}

fn render_game(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
) {
    draw_camera(&mut ratatui, &mut camera_widget, |_| {});
}

fn render_dead_game(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
    pause: Res<DeathPause>,
) {
    draw_camera(&mut ratatui, &mut camera_widget, |frame| {
        render_progress(frame, pause.percent);
    });
}

fn render_paused_game(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
) {
    draw_camera(&mut ratatui, &mut camera_widget, |frame| {
        let area = centered_rect(42, 9, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(paused_widget(), area);
    });
}

fn draw_camera(
    ratatui: &mut RatatuiContext,
    camera_widget: &mut RatatuiCameraWidget,
    overlay: impl FnOnce(&mut Frame<'_>),
) {
    let _ = ratatui.draw(|frame| {
        camera_widget.render(frame.area(), frame.buffer_mut());
        overlay(frame);
    });
}

fn menu_layout(area: Rect) -> Rc<[Rect]> {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);
    let content = centered_rect(76, area.height.saturating_sub(2), area);
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical[0].height),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(content)
}

fn title_text(width: usize) -> Text<'static> {
    const BANNER: &str = " _____                     ____            _
|_   _|__ _ __ _ __ ___   |  _ \\  __ _ ___| |__
  | |/ _ \\ '__| '_ ` _ \\  | | | |/ _` / __| '_ \\
  | |  __/ |  | | | | | | | |_| | (_| \\__ \\ | | |
  |_|\\___|_|  |_| |_| |_| |____/ \\__,_|___/_| |_|";

    Text::from(if width < 34 {
        vec![Line::from("Term Dash")]
    } else {
        BANNER.lines().map(Line::from).collect()
    })
}

fn world_list_widget(world_registry: &LevelRegistry) -> List<'_> {
    List::new(
        world_registry
            .worlds
            .iter()
            .map(|world| {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", base_style()),
                    Span::styled(world.name.as_str(), base_style()),
                ]))
            })
            .collect::<Vec<_>>(),
    )
    .block(panel_block(" Worlds "))
    .highlight_style(highlight_style())
    .highlight_symbol("> ")
}

fn list_state(menu: &MenuState, world_registry: &LevelRegistry) -> ListState {
    let mut state = ListState::default();
    if !world_registry.worlds.is_empty() {
        state.select(Some(menu.selected_world));
    }
    state
}

fn details_widget<'a>(menu: &MenuState, world_registry: &'a LevelRegistry) -> Paragraph<'a> {
    let description = world_registry
        .selected(menu.selected_world)
        .map(|world| world.description.as_str())
        .unwrap_or("No worlds available.");

    Paragraph::new(description)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(base_style())
        .block(panel_block(" Details "))
}

fn help_widget(text: &'static str) -> Paragraph<'static> {
    Paragraph::new(text).alignment(Alignment::Center).style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM),
    )
}

fn paused_widget() -> Paragraph<'static> {
    Paragraph::new(vec![
        Line::styled("Paused", highlight_style()),
        Line::raw(""),
        Line::styled("Esc resume", base_style()),
        Line::styled("Enter main menu", base_style()),
    ])
    .alignment(Alignment::Center)
    .block(panel_block(" Menu "))
}

fn render_progress(frame: &mut Frame<'_>, percent: u8) {
    let lines = big_percent_text(percent);
    let width = lines.iter().map(|line| line.width()).max().unwrap_or(0) as u16;
    let height = lines.len() as u16;
    let area = centered_rect(width, height, frame.area());

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(highlight_style()),
        area,
    );
}

fn big_percent_text(percent: u8) -> Vec<Line<'static>> {
    const DIGITS: [[&str; 5]; 10] = [
        ["███", "█ █", "█ █", "█ █", "███"],
        [" ██", "  █", "  █", "  █", " ███"],
        ["███", "  █", "███", "█  ", "███"],
        ["███", "  █", "███", "  █", "███"],
        ["█ █", "█ █", "███", "  █", "  █"],
        ["███", "█  ", "███", "  █", "███"],
        ["███", "█  ", "███", "█ █", "███"],
        ["███", "  █", "  █", "  █", "  █"],
        ["███", "█ █", "███", "█ █", "███"],
        ["███", "█ █", "███", "  █", "███"],
    ];
    const PERCENT: [&str; 5] = ["█   █", "   █ ", "  █  ", " █   ", "█   █"];

    let glyphs = percent
        .to_string()
        .bytes()
        .map(|digit| DIGITS[(digit - b'0') as usize])
        .chain([PERCENT])
        .collect::<Vec<_>>();

    (0..5)
        .map(|row| {
            Line::from(
                glyphs
                    .iter()
                    .map(|glyph| glyph[row])
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
        .collect()
}

fn panel_block(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
}

fn base_style() -> Style {
    Style::default().fg(Color::White)
}

fn highlight_style() -> Style {
    base_style().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

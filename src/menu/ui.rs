use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui_camera::RatatuiCameraWidget;
use ratatui::{
    layout::{Alignment::Center, Constraint::Length, Direction::Vertical, Layout, Rect},
    style::{Color::*, Modifier, Style},
    text::{Line, Text},
    widgets::*,
};

use crate::{
    AppState, gameplay::death::DeathPause, menu::resources::MenuState,
    world::registry::LevelRegistry,
};

// Base, highlight and border styles
const BASE: Style = Style::new().fg(White);
const HI: Style = Style::new().fg(Cyan).add_modifier(Modifier::BOLD);
const BORDER: Style = Style::new().fg(Green);

const BANNER: &str = r#" _____                     ____            _
|_   _|__ _ __ _ __ ___   |  _ \  __ _ ___| |__
  | |/ _ \ '__| '_ ` _ \  | | | |/ _` / __| '_ \\
  | |  __/ |  | | | | | | | |_| | (_| \__ \ | | |
  |_|\___|_|  |_| |_| |_| |____/ \__,_|___/_| |_|"#;

pub fn render(
    mut tui: ResMut<RatatuiContext>,
    state: Res<State<AppState>>,
    menu: Option<Res<MenuState>>,
    worlds: Option<Res<LevelRegistry>>,
    pause: Option<Res<DeathPause>>,
    mut camera: Option<Single<&mut RatatuiCameraWidget>>,
) {
    let _ = tui.draw(|f| {
        let block = |t| {
            Block::default()
                .title(t)
                .borders(Borders::ALL)
                .border_style(BORDER)
        };

        let center = |w: u16, h: u16, r: Rect| Rect {
            x: r.x + r.width.saturating_sub(w.min(r.width)) / 2,
            y: r.y + r.height.saturating_sub(h.min(r.height)) / 2,
            width: w.min(r.width),
            height: h.min(r.height),
        };

        let mut render_camera = || {
            Widget::render(&mut ***(camera.as_mut().unwrap()), f.area(), f.buffer_mut());
        };

        match state.get() {
            AppState::MainMenu => {
                let menu = menu.unwrap();
                let worlds = worlds.unwrap();

                let area = f.area();
                let center_area = center(76, area.height.saturating_sub(2), area);
                let rects = Layout::default()
                    .direction(Vertical)
                    .constraints([Length(6), Length(10), Length(5), Length(3)])
                    .split(center_area);
                let [title, list, details, help] = [rects[0], rects[1], rects[2], rects[3]];

                f.render_widget(
                    Paragraph::new(Text::from(if title.width < 34 {
                        vec![Line::from("Term Dash")]
                    } else {
                        BANNER.lines().map(Line::from).collect()
                    }))
                    .alignment(Center)
                    .style(HI),
                    title,
                );
                f.render_stateful_widget(
                    List::new(
                        worlds
                            .worlds
                            .iter()
                            .map(|w| ListItem::new(Line::styled(format!("  {}", w.name), BASE))),
                    )
                    .block(block(" Worlds "))
                    .highlight_style(HI)
                    .highlight_symbol("> "),
                    list,
                    &mut {
                        let mut s = ListState::default();
                        s.select((!worlds.worlds.is_empty()).then_some(menu.selected_world));
                        s
                    },
                );

                f.render_widget(
                    Paragraph::new(
                        worlds
                            .selected(menu.selected_world)
                            .map(|w| w.description.as_str())
                            .unwrap_or("No worlds available."),
                    )
                    .wrap(Wrap { trim: true })
                    .style(BASE)
                    .block(block(" Details ")),
                    details,
                );

                f.render_widget(
                    Paragraph::new("Up/Down select  |  Enter play  |  Esc menu during run")
                        .alignment(Center)
                        .style(Style::new().fg(DarkGray).add_modifier(Modifier::DIM)),
                    help,
                );
            }

            AppState::Playing => {
                render_camera();
            }

            AppState::Paused => {
                render_camera();

                let area = center(42, 9, f.area());

                f.render_widget(Clear, area);
                f.render_widget(
                    Paragraph::new(vec![
                        Line::styled("Paused", HI),
                        Line::raw(""),
                        Line::styled("Esc: resume", BASE),
                        Line::styled("Enter: main menu", BASE),
                    ])
                    .alignment(Center)
                    .block(block(" Menu ")),
                    area,
                );
            }

            AppState::Dead => {
                const FONT: [&str; 11] = [
                    "███|█ █|█ █|█ █|███",
                    " ██|  █|  █|  █| ███",
                    "███|  █|███|█  |███",
                    "███|  █|███|  █|███",
                    "█ █|█ █|███|  █|  █",
                    "███|█  |███|  █|███",
                    "███|█  |███|█ █|███",
                    "███|  █|  █|  █|  █",
                    "███|█ █|███|█ █|███",
                    "███|█ █|███|  █|███",
                    "█   █|   █ |  █  | █   |█   █",
                ];

                let glyphs = pause
                    .unwrap()
                    .percent
                    .to_string()
                    .bytes()
                    .map(|d| FONT[(d - b'0') as usize])
                    .chain([FONT[10]])
                    .map(|g| g.split('|').collect::<Vec<_>>())
                    .collect::<Vec<_>>();

                let lines = (0..5)
                    .map(|r| Line::from(glyphs.iter().map(|g| g[r]).collect::<Vec<_>>().join(" ")))
                    .collect::<Vec<_>>();

                let area = center(
                    lines.iter().map(Line::width).max().unwrap_or(0) as u16,
                    5,
                    f.area(),
                );

                f.render_widget(Paragraph::new(lines).alignment(Center).style(HI), area);
            }
        }
    });
}

pub mod model;
mod systems;
use crate::{core::camera::render_camera, gameplay::RunStats};
pub use systems::UiPlugin;

use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui_camera::{RatatuiCamera, RatatuiCameraWidget};
use ratatui::{
    buffer::Buffer,
    layout::{
        Alignment, Alignment::Center, Constraint, Constraint::Length, Direction::Vertical, Layout,
        Rect,
    },
    style::{Color::*, Modifier, Style},
    text::{Line, Text},
    widgets::*,
};
use tui_big_text::{BigText, PixelSize};

use crate::{AppState, level::registry::Levels, state::RuntimeFeatures, ui::model::MenuState};

const BASE: Style = Style::new().fg(White);
pub(crate) const HI: Style = Style::new().fg(Cyan).add_modifier(Modifier::BOLD);
const BORDER: Style = Style::new().fg(Green);

const BANNER: &str = r#" _____                     ____            _
|_   _|__ _ __ _ __ ___   |  _ \  __ _ ___| |__
  | |/ _ \ '__| '_ ` _ \  | | | |/ _` / __| '_ \\
  | |  __/ |  | | | | | | | |_| | (_| \__ \ | | |
  |_|\___|_|  |_| |_| |_| |____/ \__,_|___/_| |_|"#;

pub fn render(
    mut tui: ResMut<RatatuiContext>,
    state: Res<State<AppState>>,
    editor: Res<RuntimeFeatures>,
    menu: Option<Res<MenuState>>,
    levels: Res<Levels>,
    mut camera: Query<(&mut RatatuiCameraWidget, &mut RatatuiCamera)>,
    stats: Res<RunStats>,
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

        match state.get() {
            AppState::MainMenu => {
                let menu = menu.unwrap();

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
                        levels
                            .iter()
                            .map(|w| ListItem::new(Line::styled(format!("  {}", w.name), BASE))),
                    )
                    .block(block(" Levels "))
                    .highlight_style(HI)
                    .highlight_symbol("> "),
                    list,
                    &mut {
                        let mut s = ListState::default();
                        s.select((!levels.is_empty()).then_some(menu.0));
                        s
                    },
                );

                f.render_widget(
                    Paragraph::new(
                        levels
                            .get(menu.0)
                            .map_or("No levels found.", |level| level.description.as_str()),
                    )
                    .wrap(Wrap { trim: true })
                    .style(BASE)
                    .block(block(" Details ")),
                    details,
                );

                f.render_widget(
                    Paragraph::new("Up/Down select  |  Enter play  |  + create  |  - delete")
                        .alignment(Center)
                        .style(Style::new().fg(DarkGray).add_modifier(Modifier::DIM)),
                    help,
                );
            }

            AppState::Playing => {
                let area = f.area();

                render_camera(&mut camera, area, f.buffer_mut());

                let text = format!("> Attempt {}: {}%", stats.attempts + 1, stats.percent);
                let width = text.len() as u16;

                f.render_widget(
                    Paragraph::new(text).alignment(Alignment::Right).style(HI),
                    Rect {
                        x: f.area().right() - width - 1,
                        y: 0,
                        width,
                        height: 1,
                    },
                );
            }

            AppState::Paused => {
                render_camera(&mut camera, f.area(), f.buffer_mut());

                let area = center(42, 9, f.area());
                let mut lines = vec![
                    Line::styled("Paused", HI),
                    Line::raw(""),
                    Line::styled("Esc: resume", BASE),
                    Line::styled("Enter: main menu", BASE),
                ];
                if editor.graphics {
                    lines.insert(3, Line::styled("E: editor", BASE));
                }

                f.render_widget(Clear, area);
                f.render_widget(
                    Paragraph::new(lines)
                        .alignment(Center)
                        .block(block(" Menu ")),
                    area,
                );
            }
            AppState::Dead => {
                render_camera(&mut camera, f.area(), f.buffer_mut());

                let [_, logo_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(8),
                    Constraint::Fill(1),
                ])
                .areas(f.area());

                let text = format!("{}%", stats.percent);
                let widget = BigText::builder()
                    .lines(vec![Line::from(text)])
                    .pixel_size(PixelSize::Full)
                    .style(HI)
                    .centered()
                    .build();

                let mut text_buffer = Buffer::empty(logo_area);
                widget.render(logo_area, &mut text_buffer);

                // Strip blank symbols; we want only the text
                for y in logo_area.top()..logo_area.bottom() {
                    for x in logo_area.left()..logo_area.right() {
                        let source = &text_buffer[(x, y)];
                        if source.symbol() == " " {
                            continue;
                        }

                        f.buffer_mut()[(x, y)]
                            .set_symbol(source.symbol())
                            .set_fg(source.fg)
                            .modifier = source.modifier;
                    }
                }
            }

            AppState::Editing => {
                render_camera(&mut camera, f.area(), f.buffer_mut());
            }
        }
    });
}

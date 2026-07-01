mod helpers;
pub mod model;
mod systems;
mod widgets;
use crate::{core::camera::render_camera, gameplay::RunStats};
use ratatui::prelude::Stylize;
pub use systems::UiPlugin;

use crate::ui::helpers::center;
use crate::ui::widgets::modal::Modal;
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

use crate::{AppState, level::registry::Levels, state::RuntimeFeatures, ui::model::LevelMenu};

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
    menu: Option<Res<LevelMenu>>,
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

        let app_state = state.get();

        match app_state {
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
                        s.select((!levels.is_empty()).then_some(menu.selected));
                        s
                    },
                );

                f.render_widget(
                    Paragraph::new(levels.get(menu.selected).map_or(
                        "no levels found, although, this should never be reached",
                        |level| level.description.as_str(),
                    ))
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

                if menu.confirm_delete {
                    f.render_widget(
                        Modal {
                            title: Line::styled("Delete level?", Style::new().red().bold()),
                            lines: vec![
                                Line::from(""),
                                Line::from("This cannot be undone.").red(),
                                Line::from(""),
                                Line::from("[Enter] Delete"),
                                Line::from("[Esc] Cancel"),
                            ],
                        },
                        f.area(),
                    );
                }
            }

            AppState::Playing => {
                let area = f.area();

                render_camera(&mut camera, area, f.buffer_mut());

                let text = format!(
                    "> Attempt {}: {}%, {:.1}s",
                    stats.attempts + 1,
                    stats.percent,
                    stats.time
                );
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

            AppState::Paused | AppState::DeathPaused => {
                render_camera(&mut camera, f.area(), f.buffer_mut());

                let mut lines = vec![
                    Line::styled("Paused", HI),
                    Line::raw(""),
                    Line::from("[Esc] resume"),
                    Line::from("[Enter] main menu"),
                ];
                if *app_state == AppState::Paused && editor.graphics {
                    lines.insert(3, Line::styled("[E] editor", BASE));
                }
                f.render_widget(
                    Modal {
                        title: Line::from("Menu"),
                        lines,
                    },
                    f.area(),
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

            AppState::Victory => {
                render_camera(&mut camera, f.area(), f.buffer_mut());
                f.render_widget(
                    Modal {
                        title: Line::styled("Victory", Style::new().green()),
                        lines: vec![
                            Line::from(""),
                            Line::from("You won!").green().bold(),
                            Line::from(format!("Attempts: {}", stats.attempts + 1)),
                            Line::from(format!("Time (s): {:.2}", stats.time)),
                            Line::from("[Enter] Return to menu"),
                        ],
                    },
                    f.area(),
                );
            }
        }
    });
}

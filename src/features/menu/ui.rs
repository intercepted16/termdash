use bevy::prelude::*;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui_camera::RatatuiCameraWidget;
use figlet_rs::FIGlet;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget};

use crate::core::app_state::AppState;
use crate::features::menu::resources::MenuState;
use crate::features::menu::ui_helpers::{base_style, border_style, highlight_style, muted_style};

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
            render_paused_game.run_if(in_state(AppState::Paused)),
        );
    }
}

fn render_main_menu(mut ratatui: ResMut<RatatuiContext>, menu: Res<MenuState>) {
    let _ = ratatui.draw(|frame| {
        let area = frame.area();
        let layout = menu_layout(area);

        frame.render_widget(title_widget(layout.header.width), layout.header);
        frame.render_stateful_widget(
            world_list_widget(&menu),
            layout.list,
            &mut list_state(&menu),
        );
        frame.render_widget(details_widget(&menu), layout.details);
        frame.render_widget(
            help_widget("Up/Down select  |  Enter play  |  Esc menu during run"),
            layout.footer,
        );
    });
}

fn render_game(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
) {
    let _ = ratatui.draw(|frame| {
        camera_widget.render(frame.area(), frame.buffer_mut());
    });
}

fn render_paused_game(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
) {
    let _ = ratatui.draw(|frame| {
        camera_widget.render(frame.area(), frame.buffer_mut());

        let area = centered_rect(42, 9, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("Paused", highlight_style())),
                Line::raw(""),
                Line::from(Span::styled("Esc resume", base_style())),
                Line::from(Span::styled("Enter main menu", base_style())),
            ])
            .alignment(Alignment::Center)
            .block(panel_block(" Menu ")),
            area,
        );
    });
}

struct MenuLayout {
    header: Rect,
    list: Rect,
    details: Rect,
    footer: Rect,
}

fn menu_layout(area: Rect) -> MenuLayout {
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

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical[0].height),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(content);

    MenuLayout {
        header: rows[0],
        list: rows[1],
        details: rows[2],
        footer: rows[3],
    }
}

fn title_widget(width: u16) -> Paragraph<'static> {
    Paragraph::new(title_text(width as usize))
        .alignment(Alignment::Center)
        .style(highlight_style())
}

fn title_text(width: usize) -> Text<'static> {
    if width < 34 {
        return Text::from(Line::from("Term Dash"));
    }

    let Some(font) = FIGlet::standard().ok() else {
        return Text::from(Line::from("Term Dash"));
    };
    let Some(figure) = font.convert("Term Dash") else {
        return Text::from(Line::from("Term Dash"));
    };

    Text::from(
        figure
            .as_str()
            .lines()
            .map(|line| Line::from(line.trim_end().to_string()))
            .collect::<Vec<_>>(),
    )
}

fn world_list_widget(menu: &MenuState) -> List<'_> {
    let items = menu
        .worlds
        .iter()
        .map(|world| {
            ListItem::new(Line::from(vec![
                Span::styled("  ", base_style()),
                Span::styled(world.name.as_str(), base_style()),
            ]))
        })
        .collect::<Vec<_>>();

    List::new(items)
        .block(panel_block(" Worlds "))
        .highlight_style(highlight_style())
        .highlight_symbol("> ")
}

fn list_state(menu: &MenuState) -> ListState {
    let mut state = ListState::default();
    if !menu.worlds.is_empty() {
        state.select(Some(menu.selected_world));
    }
    state
}

fn details_widget(menu: &MenuState) -> Paragraph<'_> {
    let description = menu
        .selected_world()
        .map(|world| world.description.as_str())
        .unwrap_or("No worlds available.");

    Paragraph::new(description)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(base_style())
        .block(panel_block(" Details "))
}

fn help_widget(text: &'static str) -> Paragraph<'static> {
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(muted_style().add_modifier(Modifier::DIM))
}

fn panel_block(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style())
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

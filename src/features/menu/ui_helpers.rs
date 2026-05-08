use ratatui::style::{Color, Modifier, Style};

pub const HIGHLIGHT: Color = Color::Cyan;
pub const BORDER: Color = Color::Green;
pub const TEXT: Color = Color::White;
pub const MUTED: Color = Color::DarkGray;

pub fn base_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn highlight_style() -> Style {
    Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
}

pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER)
}

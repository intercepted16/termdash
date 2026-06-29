use crate::ui::{BASE, BORDER, helpers::center};
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Widget},
};

pub struct Modal<'a> {
    pub title: Line<'a>,
    pub lines: Vec<Line<'a>>,
}

impl Widget for Modal<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = center(42, 9, area);

        // pad it so that it looks cleaner
        self.title.spans.insert(0, Span::raw(" "));
        self.title.spans.push(Span::raw(" "));

        // style default lines with base default
        let default_style = Style::default();
        for line in &mut self.lines {
            if line.style == default_style {
                line.style = BASE;
            }
        }

        Clear.render(area, buf);

        Paragraph::new(self.lines)
            .alignment(Alignment::Center)
            .block(Block::bordered().title(self.title).border_style(BORDER))
            .render(area, buf);
    }
}

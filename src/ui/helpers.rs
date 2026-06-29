//! General helpers for UI.
use ratatui::layout::Rect;

pub fn center(w: u16, h: u16, r: Rect) -> Rect {
    Rect {
        x: r.x + r.width.saturating_sub(w.min(r.width)) / 2,
        y: r.y + r.height.saturating_sub(h.min(r.height)) / 2,
        width: w.min(r.width),
        height: h.min(r.height),
    }
}

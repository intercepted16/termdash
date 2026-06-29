#[derive(bevy::prelude::Resource, Default)]
pub struct LevelMenu {
    pub selected: usize,
    pub confirm_delete: bool,
}

impl LevelMenu {
    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn next(&mut self, level_count: usize) {
        self.selected = (self.selected + 1).min(level_count.saturating_sub(1));
    }
}

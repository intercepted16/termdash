#[derive(bevy::prelude::Resource, Default)]
pub struct MenuState {
    pub selected_world: usize,
}

impl MenuState {
    pub fn select_previous(&mut self) {
        self.selected_world = self.selected_world.saturating_sub(1);
    }

    pub fn select_next(&mut self, world_count: usize) {
        if world_count > 0 {
            self.selected_world = (self.selected_world + 1).min(world_count - 1);
        }
    }
}

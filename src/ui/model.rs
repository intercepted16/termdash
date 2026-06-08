use crate::newtype;

newtype! {
#[derive(bevy::prelude::Resource, Default)]
pub struct MenuState(pub usize);
}

impl MenuState {
    pub fn previous(&mut self) {
        self.0 = self.saturating_sub(1);
    }
    pub fn next(&mut self, world_count: usize) {
        if world_count > 0 {
            self.0 = (self.0 + 1).min(world_count - 1);
        }
    }
}

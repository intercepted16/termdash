use bevy::prelude::Resource;

#[derive(Resource)]
pub struct MenuState {
    pub selected_world: usize,
    pub worlds: Vec<WorldOption>,
}

#[derive(Clone)]
pub struct WorldOption {
    pub name: String,
    pub description: String,
}

impl MenuState {
    pub fn selected_world(&self) -> Option<&WorldOption> {
        self.worlds.get(self.selected_world)
    }

    pub fn select_previous(&mut self) {
        self.selected_world = self.selected_world.saturating_sub(1);
    }

    pub fn select_next(&mut self) {
        if self.selected_world + 1 < self.worlds.len() {
            self.selected_world += 1;
        }
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected_world: 0,
            worlds: vec![
                WorldOption {
                    name: "Silly sausage".to_string(),
                    description: "your first world!".to_string(),
                },
                WorldOption {
                    name: "Tyrus Tester".to_string(),
                    description: "for testing tyrus".to_string(),
                },
            ],
        }
    }
}

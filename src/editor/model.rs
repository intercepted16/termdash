use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::level::model::Level;

#[derive(Component)]
pub struct EditorWindow;

#[derive(Component)]
pub struct EditorCamera;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EditorWindowPass;

pub struct History {
    pub limit: usize,
    pub stack: Vec<Level>,
    pub position: usize,
}

impl History {
    pub fn reset(&mut self, level: &Level) {
        self.stack.clear();
        self.position = 0;
        self.push(level);
    }

    pub fn undo(&mut self, level: &mut Level) -> Result<(), &'static str> {
        if self.position == 0 {
            return Err("there are no previous levels");
        }

        self.position -= 1;
        *level = self.stack[self.position].clone();
        Ok(())
    }

    pub fn redo(&mut self, level: &mut Level) -> Result<(), &'static str> {
        if self.position + 1 >= self.stack.len() {
            return Err("there are no redo levels");
        }

        self.position += 1;
        *level = self.stack[self.position].clone();
        Ok(())
    }

    pub fn push(&mut self, level: &Level) {
        self.stack.truncate(self.position + 1);
        self.stack.push(level.clone());

        if self.stack.len() > self.limit {
            self.stack.remove(0);
        }

        self.position = self.stack.len() - 1;
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            limit: 100,
            stack: Vec::new(),
            position: 0,
        }
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub selected_object: Option<usize>,
    pub status: String,
    pub dirty: bool,
    pub refresh_pending: bool,
    pub history: History,
    pub history_level: Option<usize>,
    pub focus_test_timer: Option<Timer>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            selected_object: None,
            status: "editor closed".to_string(),
            dirty: false,
            refresh_pending: false,
            history: History::default(),
            history_level: None,
            focus_test_timer: None,
        }
    }
}

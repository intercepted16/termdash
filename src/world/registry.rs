use crate::world::model::Level;

use bevy::prelude::Resource;
use std::fs;
use std::path::Path;

#[derive(Resource)]
pub struct LevelRegistry {
    pub worlds: Vec<Level>,
}

impl LevelRegistry {
    pub fn load() -> Result<Self, String> {
        Ok(Self {
            worlds: load_levels()?,
        })
    }

    pub fn selected(&self, index: usize) -> Option<&Level> {
        self.worlds.get(index)
    }
}

fn load_levels() -> Result<Vec<Level>, String> {
    let world_dir = Path::new("assets/worlds");
    if !world_dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(world_dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let contents =
                fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
            serde_json::from_str::<Level>(&contents)
                .map_err(|err| format!("{}: {err}", path.display()))
        })
        .collect()
}

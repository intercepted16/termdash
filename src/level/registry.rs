use crate::level::model::Level;
use crate::paths::GamePaths;

use bevy::prelude::Resource;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Resource, Default)]
pub struct Levels {
    levels: Vec<Level>,
    paths: Vec<PathBuf>,
    dir: PathBuf,
}

impl Deref for Levels {
    type Target = Vec<Level>;

    fn deref(&self) -> &Self::Target {
        &self.levels
    }
}

impl DerefMut for Levels {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.levels
    }
}

impl Levels {
    pub fn load(game_paths: &GamePaths) -> Result<Self, String> {
        let dir = game_paths.asset("worlds");

        fs::create_dir_all(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;

        let mut paths = fs::read_dir(&dir)
            .map_err(|err| format!("{}: {err}", dir.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>();
        paths.sort();

        let levels = paths
            .iter()
            .map(|path| {
                let contents =
                    fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;

                serde_json::from_str::<Level>(&contents)
                    .map_err(|err| format!("{}: {err}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { levels, paths, dir })
    }

    pub fn remove(&mut self, index: usize) -> Result<(), String> {
        let path = self.paths.get(index).ok_or("missing level path")?;

        fs::remove_file(path).map_err(|err| format!("{}: {err}", path.display()))?;

        self.levels.remove(index);
        self.paths.remove(index);

        Ok(())
    }

    /// Update and save a pre-existing level.
    pub fn save(&mut self, level: Level, index: usize) -> Result<PathBuf, String> {
        self.levels[index] = level.clone();
        let path = self.paths.get(index).ok_or("missing level path")?;
        let json = serde_json::to_string_pretty(&level).map_err(|err| err.to_string())?;
        fs::write(path, json).map_err(|err| format!("{}: {err}", path.display()))?;
        Ok(path.clone())
    }

    /// Register and save a new level to disk, returning it's index or an error.
    pub fn save_new(&mut self) -> Result<usize, String> {
        let level = Level::new();
        let id = level.id.as_deref().unwrap_or("untitled");
        let path = self.dir.join(format!("{}.json", id));

        let json = serde_json::to_string_pretty(&level).map_err(|err| err.to_string())?;
        fs::write(&path, json).map_err(|err| format!("{}: {err}", path.display()))?;

        self.levels.push(level);
        self.paths.push(path);

        Ok(self.levels.len() - 1)
    }
}

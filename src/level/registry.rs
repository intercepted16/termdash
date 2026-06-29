use crate::level::model::Level;
use crate::paths::GamePaths;

use bevy::prelude::Resource;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Load all levels from `worlds/`, given `GamePaths`. Ignores files beginning with '_'.
    pub fn load(game_paths: &GamePaths) -> Result<Self, String> {
        let dir = game_paths.asset("worlds");

        fs::create_dir_all(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;

        let mut paths = fs::read_dir(&dir)
            .map_err(|err| format!("{}: {err}", dir.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .filter(|path| {
                path.file_name()
                    .is_none_or(|name| !name.to_string_lossy().starts_with('_'))
            })
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

        if levels.is_empty() {
            return Err("no levels were found".to_string());
        }

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

    /// Register and save a new default level to disk, returning it's index or an error.
    pub fn save_new(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(self.dir.join("_default_level.json"))?;
        let mut level: Level = serde_json::from_str(&json)?;

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        level.id = Some(format!("{}_{}", level.id.as_deref().unwrap(), stamp));
        let path = self
            .dir
            .join(format!("{}.json", level.id.as_deref().unwrap()));

        let json = serde_json::to_string_pretty(&level).map_err(|err| err.to_string())?;
        fs::write(&path, json).map_err(|err| format!("{}: {err}", path.display()))?;

        self.levels.push(level);
        self.paths.push(path);

        Ok(self.levels.len() - 1)
    }
}

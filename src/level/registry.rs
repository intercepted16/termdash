use crate::level::model::Level;
use crate::paths::GamePaths;

use bevy::prelude::Resource;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Resource, Default)]
pub struct Levels {
    levels: Vec<Level>,
    paths: Vec<PathBuf>,
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
    /// Get all the level paths from the levels directory; or None if there was an issue or 0 were found.
    pub fn paths(paths: &GamePaths) -> Option<Vec<PathBuf>> {
        let mut paths = fs::read_dir(paths.asset("worlds"))
            .ok()?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect::<Vec<_>>();

        paths.sort();
        if paths.is_empty() {
            return None;
        }
        Some(paths)
    }

    pub fn path(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(PathBuf::as_path)
    }

    pub fn load(game_paths: &GamePaths) -> Result<Self, String> {
        let paths = Levels::paths(game_paths).ok_or("no levels found")?;

        let levels = paths
            .iter()
            .map(|path| {
                let contents =
                    fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;

                serde_json::from_str::<Level>(&contents)
                    .map_err(|err| format!("{}: {err}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { levels, paths })
    }
}

use crate::level::model::Level;
use crate::newtype;

use bevy::prelude::Resource;
use std::fs;
use std::path::Path;

newtype! {
#[derive(Resource, Default)]
pub struct Levels(pub Vec<Level>);
}

impl Levels {
    pub fn load() -> Result<Self, String> {
        let world_dir = Path::new("assets/worlds");

        if !world_dir.exists() {
            return Err("level directory does not exist".to_string());
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

        let levels = paths
            .into_iter()
            .map(|path| {
                let contents = fs::read_to_string(&path)
                    .map_err(|err| format!("{}: {err}", path.display()))?;

                serde_json::from_str::<Level>(&contents)
                    .map_err(|err| format!("{}: {err}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if levels.is_empty() {
            return Err("there are no levels".to_string());
        }

        Ok(Self(levels))
    }
}

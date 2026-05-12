use super::model::*;
use bevy::prelude::*;
use std::fs;
use std::path::Path;
#[derive(Resource)]
pub struct WorldRegistry {
    pub worlds: Vec<WorldDefinition>,
}
impl WorldRegistry {
    pub fn selected(&self, index: usize) -> Option<&WorldDefinition> {
        self.worlds.get(index)
    }
}
impl Default for WorldRegistry {
    fn default() -> Self {
        let worlds = load_worlds_from_assets().unwrap();
        if worlds.is_empty() {
            warn!("no worlds found in assets/worlds");
        }
        Self { worlds }
    }
}
fn load_worlds_from_assets() -> Result<Vec<WorldDefinition>, String> {
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
            serde_json::from_str::<WorldDefinition>(&contents)
                .map_err(|err| format!("{}: {err}", path.display()))
        })
        .collect()
}

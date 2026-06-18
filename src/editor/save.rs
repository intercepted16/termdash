use crate::level::{load::CurrentLevel, registry::Levels};
use std::{fs, path::PathBuf};

pub fn save_current_level(
    current_level: &CurrentLevel,
    levels: &mut Levels,
) -> Result<PathBuf, String> {
    let index = current_level
        .index
        .ok_or_else(|| "no level is loaded".to_string())?;
    let level = current_level
        .level
        .as_ref()
        .ok_or_else(|| "no level data is loaded".to_string())?;
    let path = current_level
        .path
        .clone()
        .or_else(|| levels.path(index).map(PathBuf::from))
        .ok_or_else(|| "could not find level path".to_string())?;
    let json = serde_json::to_string_pretty(level).map_err(|err| err.to_string())?;

    fs::write(&path, json).map_err(|err| format!("{}: {err}", path.display()))?;

    if let Some(slot) = levels.get_mut(index) {
        *slot = level.clone();
    }

    Ok(path)
}

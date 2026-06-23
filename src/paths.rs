use bevy::ecs::resource::Resource;
use bevy::prelude::info;
use directories::ProjectDirs;
use include_dir::{Dir, include_dir};
use std::{
    fs,
    path::{Path, PathBuf},
};

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[derive(Resource)]
pub struct GamePaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl GamePaths {
    /// Initializes game paths and extracts embedded assets in release builds.
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        if cfg!(debug_assertions) {
            let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
            info!("using assets dir: {} in debug mode", assets_dir.display());
            return Ok(Self {
                data_dir: assets_dir.clone(),
                config_dir: assets_dir.clone().join("config"),
            });
        }
        let dirs = ProjectDirs::from("com", "termdash", "termdash")
            .ok_or("could not find project dirs")?;

        let config_dir = dirs.config_dir().to_path_buf();
        let data_dir = dirs.data_dir().to_path_buf();

        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&data_dir)?;

        extract_assets(&ASSETS, &data_dir, &config_dir)?;

        Ok(Self {
            data_dir,
            config_dir,
        })
    }

    pub fn asset(&self, path: impl AsRef<Path>) -> PathBuf {
        self.data_dir.join(path)
    }

    pub fn data_file(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.data_dir.join(path)
        }
    }

    pub fn config(&self, path: impl AsRef<Path>) -> PathBuf {
        self.config_dir.join(path)
    }
}

fn extract_assets(
    dir: &Dir<'_>,
    data_dir: &Path,
    config_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for file in dir.files() {
        let rel_path = file.path();

        let target = if let Ok(stripped) = rel_path.strip_prefix("config") {
            config_dir.join(stripped)
        } else {
            data_dir.join(rel_path)
        };

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        if !target.exists() {
            fs::write(target, file.contents())?;
        }
    }

    for child in dir.dirs() {
        extract_assets(child, data_dir, config_dir)?;
    }

    Ok(())
}

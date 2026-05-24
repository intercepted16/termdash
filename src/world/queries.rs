use crate::world::components::WorldMusic;
use bevy::prelude::*;

pub type MusicEntities<'w, 's> = Query<'w, 's, Entity, With<WorldMusic>>;

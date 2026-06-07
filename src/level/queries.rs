use crate::level::components::LevelMusic;
use bevy::prelude::*;

pub type MusicEntities<'w, 's> = Query<'w, 's, Entity, With<LevelMusic>>;

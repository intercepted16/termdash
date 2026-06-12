use crate::level::model::LevelMusic;
use bevy::prelude::*;

pub type MusicEntities<'w, 's> = Query<'w, 's, Entity, With<LevelMusic>>;

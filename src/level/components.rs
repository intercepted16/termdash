use bevy::prelude::*;

macro_rules! components {
    ($($component:ident),* $(,)?) => {
        $(#[derive(Component)] pub struct $component;)*
    };
}
components!(LevelEntity, Solid, LevelMusic, AudioVisualizerBar);

use bevy::prelude::*;

macro_rules! components {
    ($($component:ident),* $(,)?) => {
        $(#[derive(Component)] pub struct $component;)*
    };
}
components!(WorldEntity, Solid, WorldMusic, AudioVisualizerBar);

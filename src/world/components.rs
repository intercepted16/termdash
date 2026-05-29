use bevy::prelude::*;

macro_rules! components {
    ($($component:ident),* $(,)?) => {
        $(#[derive(Component)] pub struct $component;)*
    };
}
components!(WorldEntity, Solid, WorldMusic, AudioVisualizerBar);

#[derive(Clone, Copy, Component, Debug)]
pub struct PlayerTrigger {
    pub activation: TriggerActivation,
    pub shape: TriggerShape,
    pub effect: TriggerEffect,
}

#[derive(Clone, Copy, Debug)]
pub enum TriggerActivation {
    Touch,
    JumpPressed,
}

#[derive(Clone, Copy, Debug)]
pub enum TriggerShape {
    Circle { radius: f32 },
    Rect { half_size: Vec2 },
}

#[derive(Clone, Copy, Debug)]
pub enum TriggerEffect {
    SetMinVerticalSpeedPx(f32),
    KillPlayer,
    FlipGravity,
}

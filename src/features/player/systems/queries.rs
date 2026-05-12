use crate::features::player::components::{Player, Velocity};
use bevy::prelude::*;
pub type Players<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static Sprite,
        &'static mut Velocity,
    ),
    With<Player>,
>;

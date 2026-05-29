use crate::player::components::{Player, Velocity};
use bevy::prelude::*;

pub type PlayerQuery<'w, 's> = Single<
    'w,
    's,
    (
        &'static mut Transform,
        &'static Sprite,
        &'static mut Velocity,
        &'static mut Player,
    ),
>;

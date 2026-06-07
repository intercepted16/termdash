use crate::player::components::{Player, Velocity};
use avian2d::prelude::Collider;
use bevy::prelude::*;

pub type PlayerQuery<'w, 's> = Single<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static Collider,
        &'static mut Velocity,
        &'static mut Player,
    ),
>;

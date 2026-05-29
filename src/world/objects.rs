use crate::world::components::*;
use crate::world::model::{GravityPortalDef, JumpOrbDef, JumpPadDef, SolidDef, Spike, WorldObject};
use bevy::prelude::*;

pub struct ShapeAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<ColorMaterial>,
}

impl WorldObject {
    pub fn spawn(&self, commands: &mut Commands, assets: ShapeAssets<'_>) {
        let mut entity = commands.spawn(WorldEntity);

        match self {
            Self::Solid(object) => {
                entity.insert(object.bundle());
            }
            Self::Spike(object) => {
                entity.insert(object.bundle(assets));
            }
            Self::JumpOrb(object) => {
                entity.insert(object.bundle(assets));
            }
            Self::JumpPad(object) => {
                entity.insert(object.bundle());
            }
            Self::GravityPortal(object) => {
                entity.insert(object.bundle());
            }
        }
    }
}

impl SolidDef {
    fn bundle(&self) -> impl Bundle {
        (
            Solid,
            Transform::from_translation(self.position.extend(0.0)),
            Sprite::from_color(self.color, self.size),
        )
    }
}

impl Spike {
    fn bundle(&self, assets: ShapeAssets<'_>) -> impl Bundle {
        let size = self.size;

        (
            PlayerTrigger {
                activation: TriggerActivation::Touch,
                shape: TriggerShape::Rect {
                    half_size: size * 0.5,
                },
                effect: TriggerEffect::KillPlayer,
            },
            Mesh2d(assets.meshes.add(triangle(size))),
            MeshMaterial2d(assets.materials.add(self.color)),
            Transform::from_translation(self.position.extend(0.0)),
        )
    }
}

impl JumpOrbDef {
    fn bundle(&self, assets: ShapeAssets<'_>) -> impl Bundle {
        (
            PlayerTrigger {
                activation: TriggerActivation::JumpPressed,
                shape: TriggerShape::Circle {
                    radius: self.radius,
                },
                effect: TriggerEffect::SetMinVerticalSpeedPx(self.strength_px),
            },
            Mesh2d(assets.meshes.add(Circle::new(self.radius))),
            MeshMaterial2d(assets.materials.add(self.color)),
            Transform::from_translation(self.position.extend(0.0)),
        )
    }
}

impl JumpPadDef {
    fn bundle(&self) -> impl Bundle {
        (
            PlayerTrigger {
                activation: TriggerActivation::Touch,
                shape: TriggerShape::Rect {
                    half_size: self.size * 0.5,
                },
                effect: TriggerEffect::SetMinVerticalSpeedPx(self.strength_px),
            },
            Transform::from_translation(self.position.extend(0.0)),
            Sprite::from_color(self.color, self.size),
        )
    }
}

impl GravityPortalDef {
    fn bundle(&self) -> impl Bundle {
        (
            PlayerTrigger {
                activation: TriggerActivation::Touch,
                shape: TriggerShape::Rect {
                    half_size: Vec2::splat(16.0),
                },
                effect: TriggerEffect::FlipGravity,
            },
            Transform::from_translation(self.position.extend(0.0)),
            Sprite::from_color(self.color, Vec2::splat(32.0)),
        )
    }
}

fn triangle(size: Vec2) -> Triangle2d {
    Triangle2d::new(
        Vec2::new(0.0, size.y * 0.5),
        Vec2::new(size.x * -0.5, size.y * -0.5),
        Vec2::new(size.x * 0.5, size.y * -0.5),
    )
}

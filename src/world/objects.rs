use crate::world::components::*;
use crate::world::model::{
    JumpOrbObjectDefinition, SolidObjectDefinition, SpikeObjectDefinition, WorldObjectDefinition,
};
use bevy::prelude::*;

pub struct ShapeAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<ColorMaterial>,
}

impl WorldObjectDefinition {
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
        }
    }
}

impl SolidObjectDefinition {
    fn bundle(&self) -> impl Bundle {
        make_solid_sprite(
            self.position.as_vec2().extend(0.0),
            self.size.as_vec2(),
            self.color.as_color(),
        )
    }
}

impl SpikeObjectDefinition {
    fn bundle(&self, assets: ShapeAssets<'_>) -> impl Bundle {
        let size = self.size.as_vec2();

        (
            HazardBox {
                half_size: size * 0.5,
            },
            Mesh2d(assets.meshes.add(triangle(size))),
            MeshMaterial2d(assets.materials.add(self.color.as_color())),
            Transform::from_translation(self.position.as_vec2().extend(0.0)),
        )
    }
}

impl JumpOrbObjectDefinition {
    fn bundle(&self, assets: ShapeAssets<'_>) -> impl Bundle {
        (
            JumpOrb {
                radius: self.radius,
                strength_px: self.strength_px,
            },
            Mesh2d(assets.meshes.add(Circle::new(self.radius))),
            MeshMaterial2d(assets.materials.add(self.color.as_color())),
            Transform::from_translation(self.position.as_vec2().extend(0.0)),
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

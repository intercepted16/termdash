use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Solid;

#[derive(Bundle, Default)]
pub struct SolidBundle {
    pub solid: Solid,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl SolidBundle {
    pub fn new(position: Vec3, size: Vec2, color: Color) -> (Self, Sprite) {
        (
            Self {
                solid: Solid,
                transform: Transform::from_translation(position),
                global_transform: GlobalTransform::default(),
                visibility: Visibility::default(),
                inherited_visibility: InheritedVisibility::default(),
                view_visibility: ViewVisibility::default(),
            },
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
        )
    }
}

use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;

use crate::core::app_state::AppState;
use crate::core::constants::*;
use crate::features::player::components::Player;
use crate::features::world::loading::CurrentWorld;

pub struct CameraPlugin;

type CameraQuery<'w, 's> = Single<
    'w,
    's,
    (
        &'static mut Transform,
        &'static Projection,
        &'static RatatuiCamera,
    ),
    (With<RatatuiCamera>, Without<Player>),
>;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera).add_systems(
            PostUpdate,
            follow_player.run_if(in_state(AppState::Playing)),
        );
    }
}

pub fn projection_scale(projection: &Projection, fallback: f32) -> f32 {
    match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => fallback,
    }
}

pub fn follow_player(
    player: Single<&Transform, With<Player>>,
    current_world: Res<CurrentWorld>,
    camera: CameraQuery,
) {
    let (mut camera_transform, projection, ratatui_camera) = camera.into_inner();
    let scale = projection_scale(projection, CAMERA_ZOOM);
    let world_height = ratatui_camera.dimensions.y as f32 * scale;
    let ground_bottom = current_world
        .definition
        .as_ref()
        .map(|world| world.ground.y - world.ground.height * 0.5)
        .unwrap_or(GROUND_Y - GROUND_HEIGHT * 0.5);
    let bottom_margin = world_height * CAMERA_BOTTOM_MARGIN_FRACTION;

    camera_transform.translation.x = player.translation.x;
    camera_transform.translation.y = ground_bottom + world_height * 0.5 - bottom_margin;
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: CAMERA_ZOOM,
            ..OrthographicProjection::default_2d()
        }),
        RatatuiCamera::default(),
    ));
}

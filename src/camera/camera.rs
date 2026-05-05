// manage viewport, camera following, rendering, etc.
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraWidget;

use crate::constants::*;
use crate::player::cube::Player;
use crate::world::ground::{GROUND_PADDING, Ground};
use ratatui::widgets::Widget;
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        ratatui_config(app);
        app.add_systems(Startup, setup_scene_system);
        app.add_systems(
            PostUpdate,
            (sync_ground_to_camera, follow_player, draw_scene_system).chain(),
        );
    }
}

// a RatatuiCameraWidget component will be available in your camera entity.
fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
) {
    let _ = ratatui.draw(|frame| {
        camera_widget.render(frame.area(), frame.buffer_mut());
    });
}

// like the original; we move w/ the player
fn follow_player(
    player: Single<&Transform, With<Player>>,
    camera: Single<
        (&mut Transform, &Projection, &RatatuiCamera),
        (With<RatatuiCamera>, Without<Player>),
    >,
) {
    let (mut camera_transform, projection, ratatui_camera) = camera.into_inner();
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => CAMERA_ZOOM,
    };
    let world_height = ratatui_camera.dimensions.y as f32 * scale;
    let ground_bottom = GROUND_Y - GROUND_HEIGHT * 0.5;
    let bottom_margin = world_height * CAMERA_BOTTOM_MARGIN_FRACTION;

    camera_transform.translation.x = player.translation.x;
    camera_transform.translation.y = ground_bottom + world_height * 0.5 - bottom_margin;
}

fn setup_scene_system(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: CAMERA_ZOOM,
            ..OrthographicProjection::default_2d()
        }),
        RatatuiCamera::default(),
    ));
}

pub fn sync_ground_to_camera(
    player_transform: Single<&Transform, (With<Player>, Without<Ground>)>,
    camera: Single<
        (&Projection, &RatatuiCamera),
        (With<RatatuiCamera>, Without<Player>, Without<Ground>),
    >,
    ground: Single<(&mut Transform, &mut Sprite), With<Ground>>,
) {
    let (projection, ratatui_camera) = camera.into_inner();
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => CAMERA_ZOOM,
    };
    let world_width = ratatui_camera.dimensions.x as f32 * scale;
    let player_x = player_transform.translation.x;

    // keep x/2 behind the player and x + x/2 in front, where x is viewport width.
    let keep_behind_x = player_x - world_width * 0.5;
    let spawn_ahead_x = player_x + world_width * 1.5;
    let strip_left_x = keep_behind_x - GROUND_PADDING;
    let strip_right_x = spawn_ahead_x + GROUND_PADDING;

    let strip_width = strip_right_x - strip_left_x;
    let strip_center_x = (strip_left_x + strip_right_x) * 0.5;

    let (mut ground_transform, mut ground_sprite) = ground.into_inner();
    ground_transform.translation.x = strip_center_x;
    ground_transform.translation.y = GROUND_Y;
    ground_sprite.custom_size = Some(Vec2::new(strip_width, GROUND_HEIGHT));
}

fn ratatui_config(app: &mut App) {
    app.add_plugins((
        DefaultPlugins
            .build()
            .disable::<WinitPlugin>()
            .disable::<LogPlugin>(),
        RatatuiPlugins {
            enable_input_forwarding: true,
            ..default()
        },
        RatatuiCameraPlugin,
    ));
}

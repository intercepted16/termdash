use crate::AppState;
use crate::config::Config;
use crate::level::load::CurrentLevel;
use crate::player::components::Player;
use bevy::prelude::*;
use bevy_ratatui_camera::{RatatuiCamera, RatatuiCameraWidget};
use ratatui::layout::Rect as RatatuiRect;
use ratatui::prelude::Buffer;
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
pub fn projection_scale_or(projection: &Projection, fallback: f32) -> f32 {
    match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => fallback,
    }
}
pub fn follow_player(
    config: Res<Config>,
    player: Single<&Transform, With<Player>>,
    current_level: Res<CurrentLevel>,
    camera: CameraQuery,
) {
    let (mut camera_transform, projection, ratatui_camera) = camera.into_inner();
    let scale = projection_scale_or(projection, config.camera.zoom);
    let world_height = ratatui_camera.dimensions.y as f32 * scale;
    let ground_bottom = current_level
        .level
        .as_ref()
        .map(|level| level.ground.y - level.ground.height * 0.5)
        .unwrap();
    let bottom_margin = world_height * config.camera.bottom_margin_fraction;
    camera_transform.translation.x = player.translation.x;
    camera_transform.translation.y = ground_bottom + world_height * 0.5 - bottom_margin;
}

fn setup_camera(mut commands: Commands, config: Res<Config>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: config.camera.zoom,
            ..OrthographicProjection::default_2d()
        }),
        RatatuiCamera::default(),
    ));
}

/// Render the game buffer; applies special effects
pub fn render_camera(
    camera: &mut Query<(&mut RatatuiCameraWidget, &mut RatatuiCamera)>,
    area: RatatuiRect,
    buffer: &mut Buffer,
) {
    let Ok((widget, mut ratatui_camera)) = camera.single_mut() else {
        return;
    };

    let dimensions = UVec2::new(
        (area.width as u32 * 2).max(1),
        (area.height as u32 * 4).max(1),
    );

    if ratatui_camera.autoresize || ratatui_camera.dimensions != dimensions {
        ratatui_camera.autoresize = false;
        ratatui_camera.dimensions = dimensions;
    }

    render_braille_camera(&widget, area, buffer);
}

fn render_braille_camera(widget: &RatatuiCameraWidget, area: RatatuiRect, buffer: &mut Buffer) {
    let image = widget.camera_image.to_rgba8();
    let expected_width = area.width as u32 * 2;
    let expected_height = area.height as u32 * 4;

    if image.width() < expected_width || image.height() < expected_height {
        return;
    }

    for y in 0..area.height {
        for x in 0..area.width {
            let mut mask = 0u8;
            let mut red = 0u32;
            let mut green = 0u32;
            let mut blue = 0u32;
            let mut samples = 0u32;

            for sub_y in 0..4 {
                for sub_x in 0..2 {
                    let pixel = image
                        .get_pixel(x as u32 * 2 + sub_x, y as u32 * 4 + sub_y)
                        .0;

                    if !is_visible(pixel) {
                        continue;
                    }

                    add_pixel(&mut mask, sub_x, sub_y);
                    red += pixel[0] as u32;
                    green += pixel[1] as u32;
                    blue += pixel[2] as u32;
                    samples += 1;
                }
            }

            let Some(cell) = buffer.cell_mut((area.x + x, area.y + y)) else {
                continue;
            };

            if samples == 0 {
                cell.set_char(' ');
                continue;
            }

            let character = char::from_u32(0x2800 + mask as u32).unwrap_or(' ');
            cell.set_char(character).set_fg(ratatui::style::Color::Rgb(
                boost_channel((red / samples) as u8),
                boost_channel((green / samples) as u8),
                boost_channel((blue / samples) as u8),
            ));
        }
    }
}

/// Map x, y pixel position to braille unicode.
fn braille_dot(x: u32, y: u32) -> u8 {
    match (x, y) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (0, 3) => 0x40,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (1, 3) => 0x80,
        _ => 0,
    }
}

/// Boost contrast at midpoint for visibility.
fn boost_channel(value: u8) -> u8 {
    let v = value as f32 / 255.0;
    let v = ((v - 0.5) * 1.45 + 0.5) * 1.25;
    (v.clamp(0.0, 1.0) * 255.0) as u8
}

/// Check if a pixel is visible; ignore dull colors
fn is_visible(pixel: [u8; 4]) -> bool {
    if pixel[3] == 0 {
        return false;
    }

    let r = pixel[0] as i16;
    let g = pixel[1] as i16;
    let b = pixel[2] as i16;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    let luminance = r + g + b;

    luminance > 90 || chroma > 35
}

/// Illuminate neighboring pixels when adding a pixel; so that it appears a 'thick pixel'.
fn add_pixel(mask: &mut u8, x: u32, y: u32) {
    *mask |= braille_dot(x, y);

    if y > 0 {
        *mask |= braille_dot(x, y - 1);
    }
    if y < 3 {
        *mask |= braille_dot(x, y + 1);
    }

    if x > 0 {
        *mask |= braille_dot(x - 1, y);
    }
    if x < 1 {
        *mask |= braille_dot(x + 1, y);
    }
}

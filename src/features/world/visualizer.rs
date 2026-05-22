// Procedural music visualizer synchronized to audio playback time.
///
/// Each bar combines several time-based wave components:
/// - low-frequency motion for large swells
/// - mid-frequency motion for rhythmic variation
/// - high-frequency motion for shimmer/detail
///
/// The oscillators are spatially offset per-bar to create coherent traveling
/// wave patterns across the screen. The combined signal is clamped and shaped
/// nonlinearly to exaggerate peaks and suppress weaker movement.
///
/// Bar height, brightness, opacity, and hue animation are all derived from the
/// same pulse value.
///
/// In summary, everything is fake;
/// two completely different audio samples at the same
/// playback time would appear the same.
use crate::config::Config;
use crate::core::camera::projection_scale;
use crate::world::components::{AudioVisualizerBar, WorldEntity, WorldMusic};
use crate::world::model::WorldDefinition;
use bevy::audio::AudioSinkPlayback;
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;

const MIN_BAR_HEIGHT: f32 = 4.0;
const VISUALIZER_Z: f32 = -20.0;

#[derive(Component)]
pub struct AudioVisualizer;

#[derive(Component)]
pub struct AudioVisualizerBarState {
    index: usize,
    bar_count: usize,
    base_y: f32,
    max_height: f32,
    phase: f32,
}

pub fn spawn_audio_visualizer(
    commands: &mut Commands,
    world: &WorldDefinition,
    _music_path: &str,
    config: &Config,
) {
    if !config.visualizer.enabled {
        return;
    }

    let Some(visualizer) = world.audio_visualizer.as_ref() else {
        return;
    };

    let bar_count = visualizer.bar_count.clamp(16, 160);
    let base_y = world.ground.y + world.ground.height * 0.5;
    let max_height = (world.size.y * 0.46).max(MIN_BAR_HEIGHT);

    for index in 0..bar_count {
        let phase = index as f32 * 0.47;

        commands.spawn((
            WorldEntity,
            WorldMusic,
            AudioVisualizerBar,
            AudioVisualizer,
            AudioVisualizerBarState {
                index,
                bar_count,
                base_y,
                max_height,
                phase,
            },
            Transform::from_translation(Vec3::new(
                world.player.spawn.x,
                base_y + MIN_BAR_HEIGHT * 0.5,
                VISUALIZER_Z,
            )),
            Sprite::from_color(
                visualizer_color(0.0, 0.0, index, bar_count),
                Vec2::new(8.0, MIN_BAR_HEIGHT),
            ),
        ));
    }
}

type VisualizerCamera<'w, 's> = Single<
    'w,
    's,
    (
        &'static Transform,
        &'static Projection,
        &'static RatatuiCamera,
    ),
    (With<RatatuiCamera>, Without<AudioVisualizerBar>),
>;

pub fn update_audio_visualizer(
    config: Res<Config>,
    music: Query<&AudioSink, With<WorldMusic>>,
    camera: VisualizerCamera,
    mut bars: Query<
        (
            &AudioVisualizer,
            &AudioVisualizerBarState,
            &mut Transform,
            &mut Sprite,
        ),
        Without<RatatuiCamera>,
    >,
) {
    let Some(seconds) = music
        .iter()
        .next()
        .map(|sink| sink.position().as_secs_f32())
    else {
        return;
    };

    let (camera_transform, projection, ratatui_camera) = camera.into_inner();
    let scale = projection_scale(projection, config.camera.zoom);
    let viewport_width = ratatui_camera.dimensions.x as f32 * scale;
    let left_x = camera_transform.translation.x - viewport_width * 0.5;

    for (_, bar, mut transform, mut sprite) in &mut bars {
        let spacing = viewport_width / bar.bar_count as f32;
        let width = (spacing * 0.64).max(3.0);

        let x = bar.index as f32 / bar.bar_count as f32;

        let low = (seconds * 3.2 + bar.phase).sin() * 0.5;
        let mid = (seconds * 2.3 + x * 9.0).sin() * 0.3;
        let high = (seconds * 3.2 + x * 21.0).sin() * 0.2;

        let pulse = (low + mid + high + 0.5).clamp(0.0, 1.0).powf(1.6);
        let height =
            (MIN_BAR_HEIGHT + pulse * bar.max_height * 0.52).clamp(MIN_BAR_HEIGHT, bar.max_height);

        sprite.custom_size = Some(Vec2::new(width, height));
        sprite.color = visualizer_color(pulse, seconds, bar.index, bar.bar_count);

        transform.translation.x = left_x + spacing * (bar.index as f32 + 0.5);
        transform.translation.y = bar.base_y + height * 0.5;
    }
}

fn visualizer_color(pulse: f32, seconds: f32, index: usize, bar_count: usize) -> Color {
    let gradient = index as f32 / bar_count.max(1) as f32;

    let hue = (220.0 + gradient * 42.0 + seconds * 7.0).rem_euclid(360.0);
    let saturation = 0.70;
    let brightness = 0.55 + pulse * 0.18;
    let alpha = 0.45 + pulse * 0.18;

    Color::hsva(hue, saturation, brightness, alpha)
}

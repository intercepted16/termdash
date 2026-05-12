/// Visualize music played in worlds.
/// The music `.ogg` is decoded once into an 'envelope'.
/// Each frame stores bass, mid, treble and volume.
///
/// Every tick, based on the music's intensity, a
/// colour and height is determined and shown.
/// There is smoothing so that it does not flicker and change constantly due to minor
/// changes.
///
/// As to not distract the player, the constants below have been tuned by trial-and-error.
///
/// To be honest, I don't understand the maths here to it's full; this is more of a
/// set-and-forget ordeal.
use crate::core::camera::projection_scale;
use crate::core::constants::CAMERA_ZOOM;
use crate::features::world::components::{AudioVisualizerBar, WorldEntity, WorldMusic};
use crate::features::world::model::WorldDefinition;
use bevy::audio::AudioSinkPlayback;
use bevy::prelude::*;
use bevy_ratatui_camera::RatatuiCamera;
use lewton::inside_ogg::OggStreamReader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

const ENVELOPE_FRAME_SECONDS: f32 = 0.080;
const MIN_BAR_HEIGHT: f32 = 4.0;
const VISUALIZER_Z: f32 = -20.0;
const BAR_TIME_SPREAD_SECONDS: f32 = 0.024;
const BASS_CUTOFF_HZ: f32 = 120.0;
const MID_CUTOFF_HZ: f32 = 1_500.0;
const FEATURE_SMOOTHING: f32 = 0.070;

#[derive(Component)]
pub struct AudioVisualizer {
    envelope: Arc<AudioEnvelope>,
}

#[derive(Component)]
pub struct AudioVisualizerBarState {
    index: usize,
    bar_count: usize,
    base_y: f32,
    max_height: f32,
    phase: f32,
}

#[derive(Clone)]
struct AudioEnvelope {
    frames: Vec<AudioFrame>,
    frame_seconds: f32,
    duration_seconds: f32,
}

impl AudioEnvelope {
    fn frame_at(&self, seconds: f32) -> AudioFrame {
        if self.frames.is_empty() || self.duration_seconds <= 0.0 {
            return AudioFrame::default();
        }

        let wrapped_seconds = seconds.rem_euclid(self.duration_seconds);
        let position = wrapped_seconds / self.frame_seconds;
        let index = position.floor() as usize;
        let next_index = (index + 1) % self.frames.len();
        let blend = position.fract();

        self.frames[index].lerp(self.frames[next_index], blend)
    }
}

#[derive(Clone, Copy, Default)]
struct AudioFrame {
    bass: f32,
    mids: f32,
    treble: f32,
    volume: f32,
}

impl AudioFrame {
    fn clamp(self) -> Self {
        Self {
            bass: self.bass.clamp(0.0, 1.0),
            mids: self.mids.clamp(0.0, 1.0),
            treble: self.treble.clamp(0.0, 1.0),
            volume: self.volume.clamp(0.0, 1.0),
        }
    }

    fn lerp(self, target: Self, factor: f32) -> Self {
        Self {
            bass: self.bass.lerp(target.bass, factor),
            mids: self.mids.lerp(target.mids, factor),
            treble: self.treble.lerp(target.treble, factor),
            volume: self.volume.lerp(target.volume, factor),
        }
        .clamp()
    }
}

pub fn spawn_audio_visualizer(commands: &mut Commands, world: &WorldDefinition, music_path: &str) {
    let Some(config) = world.audio_visualizer.as_ref() else {
        return;
    };

    let Some(envelope) = analyze_ogg_envelope(music_path) else {
        warn!("could not analyze audio visualizer envelope for {music_path}");
        return;
    };

    let envelope = Arc::new(envelope);
    let bar_count = config.bar_count.clamp(16, 160);
    let base_y = world.ground.y + world.ground.height * 0.5;
    let max_height = (world.size.y * 0.46).max(MIN_BAR_HEIGHT);

    for index in 0..bar_count {
        let phase = index as f32 * 0.47;

        commands.spawn((
            WorldEntity,
            WorldMusic,
            AudioVisualizerBar,
            AudioVisualizer {
                envelope: envelope.clone(),
            },
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
                visualizer_color(AudioFrame::default(), 0.0, index, bar_count),
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

// Update bar position and color from the current music time
pub fn update_audio_visualizer(
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
    let Some(position_seconds) = music
        .iter()
        .next()
        .map(|sink| sink.position().as_secs_f32())
    else {
        return;
    };

    let (camera_transform, projection, ratatui_camera) = camera.into_inner();
    let scale = projection_scale(projection, CAMERA_ZOOM);
    let viewport_width = ratatui_camera.dimensions.x as f32 * scale;
    let left_x = camera_transform.translation.x - viewport_width * 0.5;

    for (visualizer, bar, mut transform, mut sprite) in &mut bars {
        let bar_spacing = viewport_width / bar.bar_count as f32;
        let bar_width = (bar_spacing * 0.64).max(3.0);

        let sample_time = position_seconds + bar.index as f32 * BAR_TIME_SPREAD_SECONDS;
        let features = visualizer.envelope.frame_at(sample_time);

        let wave = ((position_seconds * 3.2 + bar.phase).sin() * 0.5 + 0.5) * 0.08;
        let pulse = (features.volume * 0.30 + features.bass * 0.70).clamp(0.0, 1.0);

        let height = (MIN_BAR_HEIGHT + (pulse.powf(1.55) + wave) * bar.max_height * 0.72)
            .clamp(MIN_BAR_HEIGHT, bar.max_height);

        sprite.custom_size = Some(Vec2::new(bar_width, height));
        sprite.color = visualizer_color(features, position_seconds, bar.index, bar.bar_count);

        transform.translation.x = left_x + bar_spacing * (bar.index as f32 + 0.5);
        transform.translation.y = bar.base_y + height * 0.5;
    }
}

// populate the envelope with the initial info
fn analyze_ogg_envelope(asset_path: &str) -> Option<AudioEnvelope> {
    let path = Path::new("assets").join(asset_path);
    let file = File::open(path).ok()?;
    let mut reader = OggStreamReader::new(BufReader::new(file)).ok()?;

    let sample_rate = reader.ident_hdr.audio_sample_rate as f32;
    let channels = usize::from(reader.ident_hdr.audio_channels).max(1);
    let frame_samples = (sample_rate * ENVELOPE_FRAME_SECONDS).round() as usize;
    let mut analyzer = AudioFeatureAnalyzer::new(sample_rate);

    let mut frames = Vec::new();
    let mut sums = AudioFrameSums::default();
    let mut samples = 0usize;
    let mut total_samples = 0usize;
    let mut smoothed = AudioFrame::default();

    while let Some(packet) = reader.read_dec_packet_itl().ok()? {
        for sample_frame in packet.chunks(channels) {
            let normalized = sample_frame
                .iter()
                .map(|sample| *sample as f32 / i16::MAX as f32)
                .sum::<f32>()
                / sample_frame.len() as f32;

            let bands = analyzer.analyze(normalized);

            sums.add(bands);
            samples += 1;
            total_samples += 1;

            if samples >= frame_samples {
                smoothed = smoothed.lerp(sums.to_frame(samples), FEATURE_SMOOTHING);
                frames.push(smoothed);

                sums = AudioFrameSums::default();
                samples = 0;
            }
        }
    }

    if samples > 0 {
        smoothed = smoothed.lerp(sums.to_frame(samples), FEATURE_SMOOTHING);
        frames.push(smoothed);
    }

    let duration_seconds = total_samples as f32 / sample_rate;

    (!frames.is_empty()).then_some(AudioEnvelope {
        frames,
        frame_seconds: ENVELOPE_FRAME_SECONDS,
        duration_seconds,
    })
}

struct AudioFeatureAnalyzer {
    bass: OnePoleLowPass,
    mids: OnePoleLowPass,
}

impl AudioFeatureAnalyzer {
    fn new(sample_rate: f32) -> Self {
        Self {
            bass: OnePoleLowPass::new(BASS_CUTOFF_HZ, sample_rate),
            mids: OnePoleLowPass::new(MID_CUTOFF_HZ, sample_rate),
        }
    }

    fn analyze(&mut self, sample: f32) -> AudioFrame {
        let low = self.bass.next(sample);
        let low_mid = self.mids.next(sample);
        let mid = low_mid - low;
        let high = sample - low_mid;

        AudioFrame {
            bass: low * low,
            mids: mid * mid,
            treble: high * high,
            volume: sample * sample,
        }
    }
}

struct OnePoleLowPass {
    alpha: f32,
    value: f32,
}

impl OnePoleLowPass {
    fn new(cutoff_hz: f32, sample_rate: f32) -> Self {
        let dt = 1.0 / sample_rate;
        let rc = 1.0 / (std::f32::consts::TAU * cutoff_hz);

        Self {
            alpha: dt / (rc + dt),
            value: 0.0,
        }
    }

    fn next(&mut self, sample: f32) -> f32 {
        self.value += self.alpha * (sample - self.value);
        self.value
    }
}

#[derive(Default)]
struct AudioFrameSums {
    bass: f32,
    mids: f32,
    treble: f32,
    volume: f32,
}

impl AudioFrameSums {
    fn add(&mut self, frame: AudioFrame) {
        self.bass += frame.bass;
        self.mids += frame.mids;
        self.treble += frame.treble;
        self.volume += frame.volume;
    }

    fn to_frame(&self, sample_count: usize) -> AudioFrame {
        AudioFrame {
            bass: rms_to_visual_level(self.bass, sample_count),
            mids: rms_to_visual_level(self.mids, sample_count),
            treble: rms_to_visual_level(self.treble, sample_count),
            volume: rms_to_visual_level(self.volume, sample_count),
        }
    }
}

fn rms_to_visual_level(sum_squared: f32, sample_count: usize) -> f32 {
    let rms = (sum_squared / sample_count.max(1) as f32).sqrt();
    let decibels = 20.0 * rms.max(0.000_01).log10();

    ((decibels + 48.0) / 48.0).clamp(0.0, 1.0)
}

fn visualizer_color(
    features: AudioFrame,
    seconds: f32,
    index: usize,
    bar_count: usize,
) -> Color {
    let intensity = (features.volume * 0.35 + features.bass * 0.65).clamp(0.0, 1.0);
    let gradient = index as f32 / bar_count.max(1) as f32;

    let base_hue = 220.0;
    let gradient_hue = gradient * 42.0;
    let audio_hue = features.mids * 32.0;
    let time_hue = seconds * 7.0;

    let hue = (base_hue + gradient_hue + audio_hue + time_hue).rem_euclid(360.0);

    let saturation = (0.62 + features.treble * 0.18).clamp(0.0, 0.85);
    let brightness = (0.46 + intensity * 0.30).clamp(0.0, 0.82);
    let alpha = (0.42 + intensity * 0.26).clamp(0.0, 0.72);

    Color::hsva(hue, saturation, brightness, alpha)
}

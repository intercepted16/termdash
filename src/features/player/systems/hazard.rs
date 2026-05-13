use crate::features::player::components::{Player, Velocity};
use crate::features::player::systems::physics::{bounds_from_sprite, intersects};
use crate::features::player::systems::queries::Players;
use crate::features::world::components::Hazard;
use crate::features::world::loading::{CurrentWorld, despawn_music, spawn_music};
use crate::features::world::model::WorldDefinition;
use crate::features::world::queries::MusicEntities;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
const DEATH_PAUSE_SECONDS: f32 = 3.0;
type Hazards<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Sprite), (With<Hazard>, Without<Player>)>;
struct DeathPause {
    timer: Timer,
    percent: u8,
}
#[derive(Resource, Default)]
pub struct PlayerDeathState {
    pause: Option<DeathPause>,
}
impl PlayerDeathState {
    pub fn is_active(&self) -> bool {
        self.pause.is_some()
    }
    pub fn percent(&self) -> Option<u8> {
        self.pause.as_ref().map(|pause| pause.percent)
    }
}
#[derive(SystemParam)]
pub struct HazardParams<'w, 's> {
    players: Players<'w, 's>,
    hazards: Hazards<'w, 's>,
    music: MusicEntities<'w, 's>,
}
fn reset_player(transform: &mut Transform, velocity: &mut Velocity, spawn: Vec2) {
    transform.translation = spawn.extend(0.0);
    transform.rotation = Quat::IDENTITY;
    velocity.0 = Vec2::ZERO;
}
pub fn completion_percent(player_x: f32, world: &WorldDefinition) -> u8 {
    let start_x = world.player.spawn.x;
    let distance = (world.size.x - start_x).max(f32::EPSILON);
    (((player_x - start_x) / distance) * 100.0)
        .clamp(0.0, 100.0)
        .round() as u8
}
fn detect_player_death(
    world: &WorldDefinition,
    players: &mut Players,
    hazards: &Hazards,
) -> Option<u8> {
    let world_bottom = world.ground.y - world.size.y;
    let hazard_bounds = hazards
        .iter()
        .map(|(transform, sprite)| bounds_from_sprite(transform, sprite))
        .collect::<Vec<_>>();
    for (transform, sprite, _) in players.iter() {
        let player_bounds = bounds_from_sprite(transform, sprite);
        let hit_hazard = hazard_bounds
            .iter()
            .copied()
            .any(|hazard| intersects(player_bounds, hazard));
        let fell_out_of_world = transform.translation.y < world_bottom;
        if hit_hazard || fell_out_of_world {
            return Some(completion_percent(transform.translation.x, world));
        }
    }
    None
}
fn start_death_pause(
    percent: u8,
    death_state: &mut PlayerDeathState,
    players: &mut Players,
    commands: &mut Commands,
    music: &MusicEntities,
) {
    for (_, _, mut velocity) in players.iter_mut() {
        velocity.0 = Vec2::ZERO;
    }
    despawn_music(commands, music);
    death_state.pause = Some(DeathPause {
        timer: Timer::from_seconds(DEATH_PAUSE_SECONDS, TimerMode::Once),
        percent,
    });
}
fn tick_death_pause(
    time: &Time,
    world: &WorldDefinition,
    death_state: &mut PlayerDeathState,
    players: &mut Players,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    let Some(pause) = death_state.pause.as_mut() else {
        return;
    };
    pause.timer.tick(time.delta());
    if !pause.timer.is_finished() {
        return;
    }
    let spawn = world.player.spawn.as_vec2();
    for (mut transform, _, mut velocity) in players.iter_mut() {
        reset_player(&mut transform, &mut velocity, spawn);
    }
    spawn_music(commands, asset_server, world);
    death_state.pause = None;
}
pub fn handle_hazards(
    time: Res<Time>,
    current_world: Res<CurrentWorld>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut death_state: ResMut<PlayerDeathState>,
    mut hazards: HazardParams,
) {
    let Some(world) = current_world.definition.as_ref() else {
        return;
    };
    if death_state.is_active() {
        tick_death_pause(
            &time,
            world,
            &mut death_state,
            &mut hazards.players,
            &mut commands,
            &asset_server,
        );
        return;
    }
    let Some(percent) = detect_player_death(world, &mut hazards.players, &hazards.hazards) else {
        return;
    };
    start_death_pause(
        percent,
        &mut death_state,
        &mut hazards.players,
        &mut commands,
        &hazards.music,
    );
}

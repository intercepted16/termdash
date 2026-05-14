use crate::config::Config;
use crate::features::player::components::make_player;
use crate::features::world::components::*;
use crate::features::world::model::{WorldDefinition, WorldObjectKind};
use crate::features::world::queries::MusicEntities;
use crate::features::world::registry::WorldRegistry;
use crate::features::world::visualizer::spawn_audio_visualizer;
use bevy::prelude::*;
#[derive(Resource, Default)]
pub struct CurrentWorld {
    pub definition: Option<WorldDefinition>,
}
#[derive(Message)]
pub struct LoadWorldEvent {
    pub index: usize,
}
pub fn load_world(
    resources: (Res<Config>, Res<AssetServer>),
    mut commands: Commands,
    mut events: MessageReader<LoadWorldEvent>,
    mut current_world: ResMut<CurrentWorld>,
    registry: Res<WorldRegistry>,
    world_entities: Query<Entity, With<WorldEntity>>,
    music_entities: MusicEntities,
) {
    let (config, asset_server) = resources;

    for event in events.read() {
        let Some(world) = registry.selected(event.index) else {
            continue;
        };
        debug!("loading world {}", world.id);
        despawn_music(&mut commands, &music_entities);
        for entity in &world_entities {
            commands.entity(entity).despawn();
        }
        let default_segment = default_ground_segment(world);
        for segment in world
            .ground
            .segments
            .iter()
            .chain(world.ground.segments.is_empty().then_some(&default_segment))
        {
            commands.spawn(make_ground_segment(&world.ground, segment));
        }
        for object in &world.objects {
            let mut entity = commands.spawn((
                WorldEntity,
                Obstacle,
                Transform::from_translation(object.position.as_vec2().extend(0.0)),
                Sprite::from_color(object.color.as_color(), object.size.as_vec2()),
            ));
            match object.kind {
                WorldObjectKind::Solid => entity.insert(Solid),
                WorldObjectKind::Spike => entity.insert((Spike, Hazard)),
            };
        }
        spawn_music(&mut commands, &asset_server, world, &config);
        commands.spawn((WorldEntity, make_player(&world.player)));
        current_world.definition = Some(world.clone());
    }
}
pub fn despawn_music(commands: &mut Commands, music: &MusicEntities) {
    for entity in music.iter() {
        commands.entity(entity).despawn();
    }
}
pub fn spawn_music(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world: &WorldDefinition,
    config: &Config,
) {
    if let Some(path) = &world.music_path {
        commands.spawn((
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::LOOP,
            WorldMusic,
        ));
        spawn_audio_visualizer(commands, world, path, config);
    }
}

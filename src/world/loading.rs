use crate::config::Config;
use crate::player::components::Player;
use crate::world::components::*;
use crate::world::model::Level;
use crate::world::objects::ShapeAssets;
use crate::world::queries::MusicEntities;
use crate::world::registry::Levels;
use crate::world::visualizer::spawn_audio_visualizer;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CurrentWorld {
    pub definition: Option<Level>,
}

#[derive(Message)]
pub struct LoadWorldEvent {
    pub index: usize,
}

pub fn load_world(
    resources: (Res<Config>, Res<AssetServer>, Res<Levels>),
    render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut commands: Commands,
    mut events: MessageReader<LoadWorldEvent>,
    mut current_world: ResMut<CurrentWorld>,
    world_entities: Query<Entity, With<WorldEntity>>,
    music_entities: MusicEntities,
) {
    let (config, asset_server, registry) = resources;
    let (mut meshes, mut materials) = render_assets;

    for event in events.read() {
        let Some(world) = registry.0.get(event.index) else {
            continue;
        };

        despawn_music(&mut commands, &music_entities);
        for entity in &world_entities {
            commands.entity(entity).despawn();
        }

        for segment in world.ground.segments.iter() {
            commands.spawn(segment.make(&world.ground));
        }

        for object in &world.objects {
            object.spawn(
                &mut commands,
                ShapeAssets {
                    meshes: &mut meshes,
                    materials: &mut materials,
                },
            );
        }

        spawn_music(&mut commands, &asset_server, world, &config);
        commands.spawn((WorldEntity, Player::bundle(&world.player)));
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
    world: &Level,
    config: &Config,
) {
    let Some(path) = &world.music_path else {
        return;
    };
    commands.spawn((
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::LOOP,
        WorldMusic,
    ));

    spawn_audio_visualizer(commands, world, config);
}

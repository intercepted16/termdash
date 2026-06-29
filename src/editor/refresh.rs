use crate::{
    config::Config,
    level::{
        load::{CurrentLevel, despawn_music, spawn_music},
        model::{LevelEntity, LevelMusic, Prefabs},
        queries::MusicEntities,
        registry::Levels,
    },
    player::components::Player,
};
use avian2d::prelude::Collider;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(Message)]
pub struct RefreshLevelEvent;

pub fn refresh_level(
    resources: (Res<Config>, Res<AssetServer>, Res<Prefabs>, Res<Levels>),
    render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut commands: Commands,
    mut events: MessageReader<RefreshLevelEvent>,
    mut world: RefreshWorld,
) {
    if events.read().next().is_none() {
        return;
    }

    let (config, asset_server, prefabs, levels) = resources;
    let (mut meshes, mut materials) = render_assets;
    let Some(level) = world.current_level.get_from(&levels) else {
        return;
    };

    despawn_music(&mut commands, &world.music_entities);

    for entity in &world.authored_entities {
        commands.entity(entity).despawn_children();
        commands.entity(entity).despawn();
    }

    for segment in &level.ground.segments {
        commands.spawn(segment.make(&level.ground));
    }

    for (index, object) in level.objects.iter().enumerate() {
        object.spawn(
            index,
            &mut commands,
            (&mut meshes, &mut materials),
            &prefabs,
            &asset_server,
        );
    }

    spawn_music(&mut commands, &config, &asset_server, level, true);
    refresh_player(&mut commands, level, &mut world.player);
}

#[derive(SystemParam)]
pub struct RefreshWorld<'w, 's> {
    current_level: Res<'w, CurrentLevel>,
    authored_entities: AuthoredEntities<'w, 's>,
    player: Query<'w, 's, (Entity, &'static mut Sprite), With<Player>>,
    music_entities: MusicEntities<'w, 's>,
}

type AuthoredEntities<'w, 's> =
    Query<'w, 's, Entity, (With<LevelEntity>, Without<Player>, Without<LevelMusic>)>;

fn refresh_player(
    commands: &mut Commands,
    level: &crate::level::model::Level,
    player: &mut Query<(Entity, &mut Sprite), With<Player>>,
) {
    let Ok((entity, mut sprite)) = player.single_mut() else {
        return;
    };

    sprite.color = level.player.color;
    sprite.custom_size = Some(level.player.size);
    commands.entity(entity).insert(Collider::rectangle(
        level.player.size.x,
        level.player.size.y,
    ));
}

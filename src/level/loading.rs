use crate::config::Config;
use crate::gameplay::triggers::PlayerTrigger;
use crate::level::components::*;
use crate::level::components::{Solid, WorldEntity};
use crate::level::model::{
    Level, LevelObject, ObjectBehavior, ObjectShape, Prefabs, ResolvedObject,
};
use crate::level::queries::MusicEntities;
use crate::level::registry::Levels;
use crate::level::visualizer::spawn_audio_visualizer;
use crate::player::components::Player;
use bevy::prelude::*;
use std::fs;

#[derive(Resource, Default)]
pub struct CurrentWorld {
    pub definition: Option<Level>,
}

#[derive(Message)]
pub struct LoadWorldEvent {
    pub index: usize,
}

impl Prefabs {
    pub fn load() -> Result<Self, String> {
        // TODO: move all 'assets/' references to a constant
        let path = "assets/prefabs.json";
        let raw = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        serde_json::from_str(&raw)
            .map(Self)
            .map_err(|e| format!("{path}: {e}"))
    }
}

impl ObjectShape {
    fn insert(
        self,
        entity: &mut EntityCommands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        color: Color,
    ) {
        match self {
            ObjectShape::Rect { size } => {
                entity.insert(Sprite::from_color(color, size));
            }
            ObjectShape::Circle { radius } => {
                entity.insert((
                    Mesh2d(meshes.add(Circle::new(radius))),
                    MeshMaterial2d(materials.add(color)),
                ));
            }
            ObjectShape::Triangle { size } => {
                let mesh = Triangle2d::new(
                    Vec2::new(0.0, size.y * 0.5),
                    Vec2::new(size.x * -0.5, size.y * -0.5),
                    Vec2::new(size.x * 0.5, size.y * -0.5),
                );
                entity.insert((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(materials.add(color)),
                ));
            }
        }
    }
}

impl ObjectBehavior {
    fn insert(self, entity: &mut EntityCommands, shape: ObjectShape) {
        match self {
            ObjectBehavior::Solid => {
                entity.insert(Solid);
            }
            ObjectBehavior::Trigger { activation, effect } => {
                entity.insert(PlayerTrigger {
                    activation,
                    shape,
                    effect,
                });
            }
        }
    }
}

impl LevelObject {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        (meshes, materials): (&mut Assets<Mesh>, &mut Assets<ColorMaterial>),
        prefabs: &Prefabs,
    ) -> Result<(), String> {
        let resolved = self.resolve(prefabs)?;
        let mut entity = commands.spawn((
            WorldEntity,
            Transform::from_translation(self.position.extend(0.0)),
        ));
        resolved
            .shape
            .insert(&mut entity, meshes, materials, self.color);
        resolved.behavior.insert(&mut entity, resolved.shape);
        Ok(())
    }

    fn resolve(&self, prefabs: &Prefabs) -> Result<ResolvedObject, String> {
        let prefab = self
            .prefab
            .as_ref()
            .map(|name| {
                prefabs
                    .0
                    .get(name)
                    .ok_or_else(|| format!("unknown object prefab '{name}'"))
            })
            .transpose()?;

        macro_rules! get {
            ($field:ident) => {
                self.$field
                    .or(prefab.as_ref().map(|p| p.$field))
                    .ok_or_else(|| format!("object missing '{}' field", stringify!($field)))
            };
        }

        Ok(ResolvedObject {
            shape: get!(shape)?,
            behavior: get!(behavior)?,
        })
    }
}

pub fn load_world(
    resources: (Res<Config>, Res<AssetServer>, Res<Levels>, Res<Prefabs>),
    render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut commands: Commands,
    mut events: MessageReader<LoadWorldEvent>,
    mut current_world: ResMut<CurrentWorld>,
    world_entities: Query<Entity, With<WorldEntity>>,
    music_entities: MusicEntities,
) {
    let (config, asset_server, registry, prefabs) = resources;
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
            if let Err(err) = object.spawn(&mut commands, (&mut meshes, &mut materials), &prefabs) {
                warn!("failed to spawn world object: {err}");
            }
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

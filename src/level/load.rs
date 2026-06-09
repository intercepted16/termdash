use crate::config::Config;
use crate::gameplay::triggers::PlayerTrigger;
use crate::level::components::{LevelEntity, LevelMusic, Solid};
use crate::level::model::{
    ColliderDef, Level, LevelObject, ObjectBehavior, ObjectShape, Prefabs, ResolvedObject, Visual,
};
use crate::level::queries::MusicEntities;
use crate::level::registry::Levels;
use crate::newtype;
use crate::player::components::Player;
use avian2d::prelude::{Collider, RigidBody, Sensor};
use bevy::prelude::*;
use bevy::scene::SceneRoot;
use std::fs;

newtype! {
#[derive(Resource, Default)]
pub struct CurrentLevel(pub Option<Level>);
}

#[derive(Message)]
pub struct LoadWorldEvent {
    pub index: usize,
}

impl Prefabs {
    pub fn load() -> Self {
        let raw = fs::read_to_string("assets/prefabs.json").expect("failed to read prefabs.json");
        Prefabs(serde_json::from_str(&raw).expect("malformed prefabs JSON"))
    }
}

impl ColliderDef {
    fn into_collider(self) -> Collider {
        match self {
            ColliderDef::Rect { size } => Collider::rectangle(size.x, size.y),
            ColliderDef::Circle { radius } => Collider::circle(radius),
            ColliderDef::Triangle { size } => Collider::triangle(
                Vec2::new(0.0, size.y * 0.5),
                Vec2::new(size.x * -0.5, size.y * -0.5),
                Vec2::new(size.x * 0.5, size.y * -0.5),
            ),
        }
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
    fn insert(self, entity: &mut EntityCommands) {
        match self {
            ObjectBehavior::Solid => {
                entity.insert((Solid, RigidBody::Static));
            }
            ObjectBehavior::Trigger { activation, effect } => {
                entity.insert((
                    RigidBody::Static,
                    Sensor,
                    PlayerTrigger { activation, effect },
                ));
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
        asset_server: &AssetServer,
    ) {
        let resolved = self.resolve(prefabs);

        let mut entity = commands.spawn((
            LevelEntity,
            resolved.collider.into_collider(),
            Transform::from_translation(self.position.extend(0.0))
                .with_scale(Vec3::splat(self.scale)),
        ));

        resolved.behavior.insert(&mut entity);

        resolved
            .visual
            .spawn(&mut entity, meshes, materials, self.color, asset_server);
    }
    fn resolve(&self, prefabs: &Prefabs) -> ResolvedObject {
        let prefab = self
            .prefab
            .as_ref()
            .map(|name| {
                prefabs
                    .get(name)
                    .unwrap_or_else(|| panic!("Unknown prefab: {name}"))
            })
            .unwrap_or_else(|| panic!("couldn't get prefab"));

        let visual = self.visual.clone().unwrap_or_else(|| prefab.visual.clone());

        let collider = self
            .collider
            .or(prefab.collider)
            .or(match &visual {
                Visual::Shape { shape } => Some(*shape),
                _ => None,
            })
            .expect("non-shape objects should have a collider");

        let behavior = self.behavior.or(prefab.behavior).expect("missing behavior");

        ResolvedObject {
            visual,
            collider,
            behavior,
        }
    }
}

impl Visual {
    pub fn spawn(
        self,
        entity: &mut EntityCommands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        color: Option<Color>,
        asset_server: &AssetServer,
    ) {
        match self {
            Visual::Shape { shape } => {
                shape.insert(entity, meshes, materials, color.unwrap_or(Color::WHITE))
            }
            Visual::Sprite { path } => {
                let mut sprite = Sprite::from_image(asset_server.load(path));
                if let Some(color) = color {
                    sprite.color = color
                };
                entity.insert(sprite);
            }

            Visual::Scene { path } => {
                entity.insert(SceneRoot(asset_server.load(path)));
            }
        }
    }
}

pub fn load_level(
    resources: (Res<Config>, Res<AssetServer>, Res<Levels>, Res<Prefabs>),
    render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut commands: Commands,
    mut events: MessageReader<LoadWorldEvent>,
    mut current_level: ResMut<CurrentLevel>,
    world_entities: Query<Entity, With<LevelEntity>>,
    music_entities: MusicEntities,
) {
    let (config, asset_server, registry, prefabs) = resources;
    let (mut meshes, mut materials) = render_assets;

    for event in events.read() {
        let Some(level) = registry.0.get(event.index) else {
            continue;
        };

        despawn_music(&mut commands, &music_entities);

        for entity in &world_entities {
            commands.entity(entity).despawn();
        }

        for segment in &level.ground.segments {
            commands.spawn(segment.make(&level.ground));
        }

        for object in &level.objects {
            object.spawn(
                &mut commands,
                (&mut meshes, &mut materials),
                &prefabs,
                &asset_server,
            );
        }

        spawn_music(&mut commands, &config, &asset_server, level);
        commands.spawn((LevelEntity, Player::bundle(&level.player)));

        current_level.0 = Some(level.clone());
    }
}

pub fn despawn_music(commands: &mut Commands, music: &MusicEntities) {
    for entity in music.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_music(
    commands: &mut Commands,
    config: &Config,
    asset_server: &AssetServer,
    level: &Level,
) {
    if let Some(path) = &level.music_path {
        commands.spawn((
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::LOOP,
            LevelMusic,
        ));
        if let Some(visualizer) = level.audio_visualizer.as_ref()
            && config.visualizer.enabled
        {
            for bundle in visualizer.bundles(level) {
                commands.spawn(bundle);
            }
        }
    }
}

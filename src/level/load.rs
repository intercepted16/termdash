use crate::config::Config;
use crate::gameplay::triggers::PlayerTrigger;
use crate::level::model::{KillPlayerOnSide, LevelEntity, LevelMusic, Solid};
use crate::level::model::{
    Level, LevelObject, ObjectAnimation, ObjectAnimator, ObjectBehavior, ObjectShape, Prefabs,
    ResolvedObject, Visual,
};
use crate::level::queries::MusicEntities;
use crate::level::registry::Levels;
use crate::player::components::Player;
use avian2d::prelude::{ColliderConstructor, RigidBody, Sensor};
use bevy::prelude::*;
use std::fs;

#[derive(Resource, Default)]
pub struct CurrentLevel {
    pub index: Option<usize>,
    pub level: Option<Level>,
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

impl ObjectShape {
    fn insert(
        self,
        entity: &mut EntityCommands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        color: Color,
    ) {
        match self.0 {
            ColliderConstructor::Rectangle { x_length, y_length } => {
                let size = Vec2::new(x_length, y_length);
                entity.insert(Sprite::from_color(color, size));
            }
            ColliderConstructor::Circle { radius } => {
                entity.insert((
                    Mesh2d(meshes.add(Circle::new(radius))),
                    MeshMaterial2d(materials.add(color)),
                ));
            }
            ColliderConstructor::Triangle { a, b, c } => {
                let mesh = Triangle2d::new(a, b, c);
                entity.insert((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(materials.add(color)),
                ));
            }
            s => {
                panic!("{s:?} shape is not supported")
            }
        }
    }
}

impl ObjectBehavior {
    fn insert(self, entity: &mut EntityCommands) {
        match self {
            ObjectBehavior::Solid => {
                entity.insert((Solid, KillPlayerOnSide, RigidBody::Static));
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
            resolved.collider,
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
            .as_ref()
            .or(prefab.collider.as_ref())
            .or(match &visual {
                Visual::Shape { shape, .. } => Some(shape),
                _ => None,
            })
            .expect("non-shape objects should have a collider");

        let behavior = self.behavior.or(prefab.behavior).expect("missing behavior");

        ResolvedObject {
            visual: visual.clone(),
            behavior,
            collider: collider.clone(),
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
            Visual::Shape { shape, animations } => {
                shape.insert(entity, meshes, materials, color.unwrap_or(Color::WHITE));
                ObjectAnimation::insert_all(entity, animations);
            }
            Visual::Sprite { path, animations } => {
                let mut sprite = Sprite::from_image(asset_server.load(path));
                if let Some(color) = color {
                    sprite.color = color
                };
                entity.insert(sprite);
                ObjectAnimation::insert_all(entity, animations);
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

        current_level.index = Some(event.index);
        current_level.level = Some(level.clone());
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

impl ObjectAnimation {
    pub fn insert_all(entity: &mut EntityCommands, animations: Vec<Self>) {
        if !animations.is_empty() {
            entity.insert(ObjectAnimator(animations));
        }
    }

    pub fn animate(&self, transform: &mut Transform, time: &Time) {
        match self {
            ObjectAnimation::Spin { degrees_per_second } => {
                transform.rotate_z(degrees_per_second.to_radians() * time.delta_secs());
            }
        }
    }
}

pub fn animate_objects(time: Res<Time>, mut query: Query<(&ObjectAnimator, &mut Transform)>) {
    for (animator, mut transform) in &mut query {
        for animation in animator.0.clone() {
            animation.animate(&mut transform, &time);
        }
    }
}

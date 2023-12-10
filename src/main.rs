use aabb_picking_backend::AabbPickingBackend;
use bevy::prelude::*;

use bevy_ecs_ldtk::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_rapier2d::prelude::*;

mod aabb_picking_backend;
mod audio;
mod components;
mod crash;
mod edit;
mod lost;
mod menu;
mod play;
mod won;

fn main() {
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Worst Physics Engine Ever".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
            LdtkPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            DefaultPickingPlugins,
            AabbPickingBackend,
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..Default::default()
        })
        .insert_resource(LdtkSettings {
            ..Default::default()
        })
        .add_plugins((
            won::WonPlugin,
            lost::LostPlugin,
            edit::EditPlugin,
            play::PlayPlugin,
            menu::MenuPlugin,
            crash::CrashPlugin,
            audio::AudioPlugin,
        ))
        .add_systems(Startup, setup)
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_int_cell::<components::LadderBundle>(2)
        .register_ldtk_int_cell::<components::WallBundle>(3)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::MobBundle>("Mob")
        .register_ldtk_entity::<components::ChestBundle>("Chest")
        .register_ldtk_entity::<components::PumpkinsBundle>("Pumpkins")
        .add_state::<GameMode>()
        .add_state::<GameKind>()
        .run();
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum GameKind {
    #[cfg_attr(not(feature = "debug"), default)]
    Platformer,
    #[cfg_attr(feature = "debug", default)]
    Puzzle,
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum GameMode {
    #[default]
    Menu,
    Edit,
    Play,
    Won,
    Lost,
    Crash,
}

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const DISABLED_BUTTON: Color = Color::rgb(0.1, 0.1, 0.1);

#[derive(Resource)]
pub struct FontHandle(Handle<Font>);

#[derive(Resource)]
pub struct LdtkHandle(Handle<LdtkProject>);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    let world = asset_server.load("Typical_2D_platformer_example.ldtk");
    commands.insert_resource(LdtkHandle(world));

    let font = asset_server.load("PublicPixel-z84yD.ttf");
    commands.insert_resource(FontHandle(font));

    #[cfg(not(feature = "debug"))]
    let levels = vec![usize::MAX; LEVELS.len()];
    #[cfg(feature = "debug")]
    let levels = {
        use rand::Rng;
        let mut levels = vec![];
        let mut rng = rand::thread_rng();
        for i in 0..LEVELS.len() {
            levels.push(rng.gen_range(0..3));
        }
        levels
    };

    commands.insert_resource(Progression { levels });
}

#[derive(Resource, Clone)]
pub struct LevelInfo {
    pub start_colliders: [GridCoords; 2],
    pub thresholds: [usize; 3],
    pub max_colliders: usize,
}

const LEVELS: [LevelInfo; 3] = [
    LevelInfo {
        start_colliders: [GridCoords { x: 5, y: 5 }, GridCoords { x: 30, y: 5 }],
        thresholds: [5, 8, 10],
        max_colliders: 20,
    },
    LevelInfo {
        start_colliders: [GridCoords { x: 5, y: 5 }, GridCoords { x: 30, y: 5 }],
        thresholds: [5, 8, 10],
        max_colliders: 20,
    },
    LevelInfo {
        start_colliders: [GridCoords { x: 1, y: 15 }, GridCoords { x: 34, y: 1 }],
        thresholds: [7, 10, 13],
        max_colliders: 20,
    },
];

#[derive(Resource)]
struct CurrentLevel(usize);

#[derive(Resource)]
pub struct Progression {
    pub levels: Vec<usize>,
}

// This example shows off a more in-depth implementation of a game with `bevy_ecs_ldtk`.
// Please run with `--release`.

use aabb_picking_backend::AabbPickingBackend;
use bevy::{prelude::*, window::WindowResolution};

use bevy_ecs_ldtk::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_rapier2d::prelude::*;

mod aabb_picking_backend;
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
                        resolution: WindowResolution::new(1152.0, 640.0),
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
    #[default]
    Platformer,
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
}

#[derive(Resource)]
pub struct LevelInfo {
    pub start_colliders: Vec<GridCoords>,
    pub max_colliders: usize,
}

fn level_1() -> (LevelInfo, LevelSelection) {
    return (
        LevelInfo {
            start_colliders: vec![GridCoords::new(5, 5), GridCoords::new(30, 5)],
            max_colliders: 15,
        },
        LevelSelection::Uid(0),
    );
}

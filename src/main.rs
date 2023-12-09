// This example shows off a more in-depth implementation of a game with `bevy_ecs_ldtk`.
// Please run with `--release`.

use aabb_picking_backend::AabbPickingBackend;
use bevy::{prelude::*, window::WindowResolution};

use bevy_ecs_ldtk::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_rapier2d::prelude::*;
use edit::EnabledColliders;

mod aabb_picking_backend;
mod components;
mod edit;
mod lost;
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
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .add_plugins((
            LdtkPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            DefaultPickingPlugins,
            AabbPickingBackend,
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, 0.0),
            ..Default::default()
        })
        .insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            ..Default::default()
        })
        .insert_resource(EnabledColliders::default())
        .add_plugins((
            won::WonPlugin,
            lost::LostPlugin,
            edit::EditPlugin,
            play::PlayPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            set_default_font.run_if(resource_exists::<FontHandle>()),
        )
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_int_cell::<components::LadderBundle>(2)
        .register_ldtk_int_cell::<components::WallBundle>(3)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::MobBundle>("Mob")
        .register_ldtk_entity::<components::ChestBundle>("Chest")
        .register_ldtk_entity::<components::PumpkinsBundle>("Pumpkins")
        .add_state::<GameMode>()
        .run();
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum GameMode {
    #[default]
    Edit,
    Play,
    Won,
    Lost,
}

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Resource)]
pub struct FontHandle(Handle<Font>);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    let ldtk_handle = asset_server.load("Typical_2D_platformer_example.ldtk");
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });

    let font = asset_server.load("PublicPixel-z84yD.ttf");
    commands.insert_resource(FontHandle(font));
}

pub fn set_default_font(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    font_handle: Res<FontHandle>,
) {
    if let Some(font) = fonts.remove(&font_handle.0) {
        fonts.insert(TextStyle::default().font, font);
        commands.remove_resource::<FontHandle>();
    }
}

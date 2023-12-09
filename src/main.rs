// This example shows off a more in-depth implementation of a game with `bevy_ecs_ldtk`.
// Please run with `--release`.

use aabb_picking_backend::AabbPickingBackend;
use bevy::{prelude::*, window::WindowResolution};

use bevy_ecs_ldtk::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_rapier2d::prelude::*;
use systems::EnabledColliders;

mod aabb_picking_backend;
mod components;
mod lost;
mod systems;
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
        .add_plugins((won::WonPlugin, lost::LostPlugin))
        .add_systems(Startup, systems::setup)
        .add_systems(
            Update,
            systems::set_default_font.run_if(resource_exists::<systems::FontHandle>()),
        )
        .add_systems(
            Update,
            (
                systems::ignore_gravity_if_climbing,
                systems::detect_collision_with_environment,
                systems::movement,
                systems::patrol,
                systems::ground_detection,
                systems::update_on_ground,
                systems::check_lost_condition,
            )
                .run_if(in_state(GameMode::Play)),
        )
        .add_systems(Update, systems::camera_fit_inside_current_level)
        .add_systems(Update, systems::update_level_selection)
        .add_systems(Update, systems::spawn_ground_sensor)
        .add_systems(
            Update,
            systems::spawn_complete_wall_collision.run_if(in_state(GameMode::Play)),
        )
        .add_systems(
            Update,
            (
                systems::spawn_wall_aabb,
                systems::set_color_based_on_enabled,
            )
                .run_if(in_state(GameMode::Edit)),
        )
        .add_systems(OnEnter(GameMode::Edit), systems::setup_edit_mode)
        .add_systems(OnEnter(GameMode::Play), systems::setup_play_mode)
        .add_systems(Update, systems::restart_level)
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

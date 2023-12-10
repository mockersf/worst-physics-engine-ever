use bevy::{math::Vec3A, prelude::*, render::primitives::Aabb, utils::HashSet};
use bevy_ecs_ldtk::{assets::LdtkProject, GridCoords, Respawn};
use bevy_mod_picking::{
    events::{Click, Out, Over, Pointer},
    prelude::On,
    PickableBundle,
};
use bevy_rapier2d::plugin::RapierConfiguration;
use rand::seq::IteratorRandom;

use crate::{
    audio::AudioEvent, components::Wall, CurrentLevel, FontHandle, GameMode, HOVERED_BUTTON,
    LEVELS, NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR,
};

pub struct EditPlugin;

impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_wall_aabb,
                set_color_based_on_enabled,
                update_collider_count,
                button_system,
                crate::play::camera_fit_inside_current_level,
            )
                .run_if(in_state(GameMode::Edit)),
        )
        .add_systems(OnEnter(GameMode::Edit), setup_edit_mode)
        .add_systems(OnExit(GameMode::Edit), exit_mode);
    }
}

fn exit_mode(mut commands: Commands, query: Query<Entity, With<OnEditMode>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct OnEditMode;

fn setup_edit_mode(
    mut commands: Commands,
    world_query: Query<Entity, With<Handle<LdtkProject>>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    colliders: Res<EnabledColliders>,
    font: Res<FontHandle>,
    level: Res<CurrentLevel>,
) {
    let button_style = Style {
        width: Val::Px(150.0),
        height: Val::Px(50.0),
        margin: UiRect::bottom(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(5.0)),
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 20.0,
        color: TEXT_COLOR,
        font: font.0.clone(),
    };

    for world_entity in &world_query {
        commands.entity(world_entity).insert(Respawn);
    }
    rapier_config.gravity = Vec2::new(0.0, 0.0);

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            ..default()
        })
        .insert(OnEditMode)
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: NORMAL_BUTTON.into(),
                        border_color: BorderColor(HOVERED_BUTTON),
                        ..default()
                    },
                    ButtonAction::Play,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Play", button_text_style.clone()));
                });

            parent.spawn(
                TextBundle::from_sections([
                    TextSection {
                        value: colliders.coords.len().to_string(),
                        style: TextStyle {
                            font_size: 20.,
                            color: TEXT_COLOR,
                            font: font.0.clone(),
                        },
                    },
                    TextSection {
                        value: format!(" colliders (max {})", LEVELS[level.0].max_colliders),
                        style: TextStyle {
                            font_size: 20.,
                            color: TEXT_COLOR,
                            font: font.0.clone(),
                        },
                    },
                ])
                .with_style(Style {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                }),
            );

            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: NORMAL_BUTTON.into(),
                        border_color: BorderColor(HOVERED_BUTTON),
                        ..default()
                    },
                    ButtonAction::Reset,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Reset", button_text_style.clone()));
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: button_style.clone(),
                        background_color: NORMAL_BUTTON.into(),
                        border_color: BorderColor(HOVERED_BUTTON),
                        ..default()
                    },
                    ButtonAction::Random,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Random",
                        button_text_style.clone(),
                    ));
                });
        });
}

#[derive(Component)]
enum ButtonAction {
    Play,
    Reset,
    Random,
}

#[derive(Component)]
struct ColliderStatus {
    enabled: bool,
}

#[derive(Resource, Default)]
pub struct EnabledColliders {
    pub coords: HashSet<GridCoords>,
}

fn spawn_wall_aabb(
    mut commands: Commands,
    wall_query: Query<(Entity, &GridCoords), Added<Wall>>,
    enabled: Res<EnabledColliders>,
) {
    wall_query.for_each(|(entity, gridcoords)| {
        commands.entity(entity).insert((
            Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(8., 8., 0.) * 0.95,
            },
            AabbGizmo {
                color: Some(if enabled.coords.contains(gridcoords) {
                    Color::GREEN
                } else {
                    Color::GRAY
                }),
            },
            PickableBundle::default(),
            ColliderStatus {
                enabled: enabled.coords.contains(gridcoords),
            },
            On::<Pointer<Out>>::target_component_mut::<AabbGizmo>(|_, gizmo| {
                let color = gizmo.color.unwrap();
                let mut color = color.as_hsla();
                color.set_l(0.5);
                gizmo.color = Some(color.as_rgba());
            }),
            On::<Pointer<Over>>::target_component_mut::<AabbGizmo>(|_, gizmo| {
                let color = gizmo.color.unwrap();
                let mut color = color.as_hsla();
                color.set_l(0.9);
                gizmo.color = Some(color.as_rgba());
            }),
            On::<Pointer<Click>>::target_component_mut::<ColliderStatus>(|_, collider| {
                collider.enabled = !collider.enabled;
            }),
        ));
    });
}

fn set_color_based_on_enabled(
    mut query: Query<(&mut ColliderStatus, &mut AabbGizmo, &GridCoords)>,
    mut enabled: ResMut<EnabledColliders>,
    level: Res<CurrentLevel>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for (mut collider_status, mut gizmo, gridcoords) in &mut query {
        if collider_status.is_changed() && !collider_status.is_added() {
            if collider_status.enabled {
                if enabled.coords.len() >= LEVELS[level.0].max_colliders {
                    collider_status.enabled = false;
                    audio_events.send(AudioEvent::FailedCollider);
                } else {
                    debug!("{:?}", gridcoords);
                    gizmo.color = Some(Color::GREEN);
                    enabled.coords.insert(*gridcoords);
                    audio_events.send(AudioEvent::AddCollider);
                }
            } else {
                gizmo.color = Some(Color::GRAY);
                enabled.coords.remove(gridcoords);
                audio_events.send(AudioEvent::RemoveCollider);
            }
        }
    }
}

fn update_collider_count(colliders: Res<EnabledColliders>, mut text: Query<&mut Text>) {
    if colliders.is_changed() {
        for mut text in &mut text {
            if text.sections.len() == 2 {
                text.sections[0].value = colliders.coords.len().to_string();
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameMode>>,
    mut enabled: ResMut<EnabledColliders>,
    world_query: Query<Entity, With<Handle<LdtkProject>>>,
    wall_query: Query<&GridCoords, With<Wall>>,
    level: Res<CurrentLevel>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => {
                audio_events.send(AudioEvent::Click);
                match button {
                    ButtonAction::Play => next_state.set(GameMode::Play),
                    ButtonAction::Reset => {
                        enabled.coords.clear();
                        for coord in &LEVELS[level.0].start_colliders {
                            enabled.coords.insert(*coord);
                        }
                        for world_entity in &world_query {
                            commands.entity(world_entity).insert(Respawn);
                        }
                    }
                    ButtonAction::Random => {
                        enabled.coords.clear();
                        for coord in &LEVELS[level.0].start_colliders {
                            enabled.coords.insert(*coord);
                        }

                        let mut rng = rand::thread_rng();
                        while enabled.coords.len() < LEVELS[level.0].max_colliders {
                            let sample = wall_query.iter().choose(&mut rng).unwrap();
                            enabled.coords.insert(*sample);
                        }

                        for world_entity in &world_query {
                            commands.entity(world_entity).insert(Respawn);
                        }
                    }
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

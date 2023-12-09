use bevy::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;

use crate::{
    FontHandle, GameKind, GameMode, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR,
};

pub struct CrashPlugin;

impl Plugin for CrashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            crash
                .run_if(in_state(GameMode::Play))
                .run_if(in_state(GameKind::Platformer)),
        )
        .add_systems(OnEnter(GameMode::Crash), setup)
        .add_systems(OnExit(GameMode::Crash), exit_screen)
        .add_systems(Update, button_system.run_if(in_state(GameMode::Crash)));
    }
}

fn crash(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut acc: Local<f32>,
    mut next_state: ResMut<NextState<GameMode>>,
    mut next_kind: ResMut<NextState<GameKind>>,
) {
    if input.pressed(KeyCode::A) || input.pressed(KeyCode::D) {
        *acc += time.delta_seconds();
    }
    if input.just_pressed(KeyCode::Space) {
        *acc += 0.5;
    }
    if *acc > 1.0 {
        next_state.set(GameMode::Crash);
        next_kind.set(GameKind::Puzzle);
    }
}

#[derive(Component)]
struct OnCrashScreen;

fn exit_screen(mut commands: Commands, query: Query<Entity, With<OnCrashScreen>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(
    mut commands: Commands,
    font: Res<FontHandle>,
    world_query: Query<Entity, With<Handle<LdtkProject>>>,
) {
    commands.entity(world_query.single()).despawn_recursive();
    commands.insert_resource(ClearColor(Color::BLUE));

    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(70.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(5.0)),
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 30.0,
        color: TEXT_COLOR,
        font: font.0.clone(),
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.3, 0.3, 0.3, 0.5).into(),
                ..default()
            },
            OnCrashScreen,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "Crashed!",
                    TextStyle {
                        font_size: 60.0,
                        color: TEXT_COLOR,
                        font: font.0.clone(),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );
            parent.spawn(
                TextBundle::from_section(
                    "Wow, that's a lot of collider entities!",
                    TextStyle {
                        font_size: 25.0,
                        color: TEXT_COLOR,
                        font: font.0.clone(),
                    },
                )
                .with_text_alignment(TextAlignment::Left)
                .with_style(Style {
                    margin: UiRect::left(Val::Px(50.0)),
                    ..default()
                }),

            );
            parent.spawn(
                TextBundle::from_section(
                    "Let's be honest, I'm not that good of a physics engine...\nHelp me on this, you'll select where you actually want a collider, and I'll see what I can do.",
                    TextStyle {
                        font_size: 25.0,
                        color: TEXT_COLOR,
                        font: font.0.clone(),
                    },
                )
                .with_text_alignment(TextAlignment::Left)
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                border_color: BorderColor(HOVERED_BUTTON),
                                ..default()
                            },
                            ButtonAction::Menu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Ok...",
                                button_text_style.clone(),
                            ));
                        });
                });
        });
}

#[derive(Component)]
enum ButtonAction {
    Menu,
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => {
                match button {
                    ButtonAction::Menu => next_state.set(GameMode::Menu),
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

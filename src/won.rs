use bevy::prelude::*;
use bevy_ecs_ldtk::assets::LdtkProject;

use crate::{
    edit::EnabledColliders, CurrentLevel, FontHandle, GameMode, Progression, HOVERED_BUTTON,
    LEVELS, NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR,
};

pub struct WonPlugin;

impl Plugin for WonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::Won), setup)
            .add_systems(OnExit(GameMode::Won), exit_screen)
            .add_systems(Update, button_system.run_if(in_state(GameMode::Won)));
    }
}

#[derive(Component)]
struct OnWonScreen;

fn exit_screen(mut commands: Commands, query: Query<Entity, With<OnWonScreen>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(
    mut commands: Commands,
    colliders: Res<EnabledColliders>,
    font: Res<FontHandle>,
    mut progression: ResMut<Progression>,
    level: Res<CurrentLevel>,
    asset_server: Res<AssetServer>,
) {
    progression.levels[level.0] = LEVELS[level.0]
        .thresholds
        .binary_search(&colliders.coords.len())
        .map_or_else(|err| err, |ok| ok)
        .min(progression.levels[level.0]);

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
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgba(0.3, 0.3, 0.3, 0.5).into(),
                ..default()
            },
            OnWonScreen,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "You won!",
                    TextStyle {
                        font_size: 80.0,
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
                    format!("with {} colliders", colliders.coords.len()),
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        font: font.0.clone(),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );

            let percent = (colliders.coords.len() as f32 - LEVELS[level.0].max_colliders as f32)
                .abs()
                / (LEVELS[level.0].max_colliders as f32 - LEVELS[level.0].thresholds[0] as f32);
            let gold =
                (LEVELS[level.0].thresholds[0] as f32 - LEVELS[level.0].max_colliders as f32).abs()
                    / (LEVELS[level.0].max_colliders as f32 - LEVELS[level.0].thresholds[0] as f32);
            let silver =
                (LEVELS[level.0].thresholds[1] as f32 - LEVELS[level.0].max_colliders as f32).abs()
                    / (LEVELS[level.0].max_colliders as f32 - LEVELS[level.0].thresholds[0] as f32);
            let bronze =
                (LEVELS[level.0].thresholds[2] as f32 - LEVELS[level.0].max_colliders as f32).abs()
                    / (LEVELS[level.0].max_colliders as f32 - LEVELS[level.0].thresholds[0] as f32);
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        width: Val::Px(500.0),
                        height: Val::Px(50.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(percent * 100.0),
                            height: Val::Px(30.0),
                            ..default()
                        },
                        background_color: Color::rgb(0.0, 1.0, 0.0).into(),
                        ..default()
                    });
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent((1.0 - percent) * 100.0),
                            height: Val::Px(30.0),
                            ..default()
                        },
                        background_color: Color::rgb(0.0, 0.0, 0.0).into(),
                        ..default()
                    });
                    parent.spawn(ImageBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(bronze * 100.0 - 2.5),
                            top: Val::Px(-15.0),
                            width: Val::Px(25.0),
                            height: Val::Px(25.0),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load("starBronze.png")),
                        background_color: BackgroundColor(if progression.levels[level.0] < 3 {
                            Color::WHITE
                        } else {
                            Color::GRAY
                        }),
                        ..default()
                    });
                    parent.spawn(ImageBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(silver * 100.0 - 2.5),
                            top: Val::Px(-15.0),
                            width: Val::Px(25.0),
                            height: Val::Px(25.0),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load("starSilver.png")),
                        background_color: BackgroundColor(if progression.levels[level.0] < 2 {
                            Color::WHITE
                        } else {
                            Color::GRAY
                        }),
                        ..default()
                    });
                    parent.spawn(ImageBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(gold * 100.0 - 2.5),
                            top: Val::Px(-15.0),
                            width: Val::Px(25.0),
                            height: Val::Px(25.0),
                            ..default()
                        },
                        image: UiImage::new(asset_server.load("starGold.png")),
                        background_color: BackgroundColor(if progression.levels[level.0] < 1 {
                            Color::WHITE
                        } else {
                            Color::GRAY
                        }),
                        ..default()
                    });
                });

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
                            parent
                                .spawn(TextBundle::from_section("Menu", button_text_style.clone()));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                border_color: BorderColor(HOVERED_BUTTON),
                                ..default()
                            },
                            ButtonAction::Retry,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Retry",
                                button_text_style.clone(),
                            ));
                        });
                });
        });
}

#[derive(Component)]
enum ButtonAction {
    Menu,
    Retry,
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameMode>>,
    world_query: Query<Entity, With<Handle<LdtkProject>>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => {
                match button {
                    ButtonAction::Retry => next_state.set(GameMode::Edit),
                    ButtonAction::Menu => {
                        commands.entity(world_query.single()).despawn_recursive();
                        next_state.set(GameMode::Menu);
                    }
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

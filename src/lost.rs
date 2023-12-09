use bevy::prelude::*;

use crate::GameMode;

pub struct LostPlugin;

impl Plugin for LostPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::Lost), setup)
            .add_systems(OnExit(GameMode::Lost), exit_screen)
            .add_systems(Update, button_system.run_if(in_state(GameMode::Lost)));
    }
}

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct OnLostScreen;

fn exit_screen(mut commands: Commands, query: Query<Entity, With<OnLostScreen>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(70.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 30.0,
        color: TEXT_COLOR,
        ..default()
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
            OnLostScreen,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "You lost...",
                    TextStyle {
                        font_size: 60.0,
                        color: TEXT_COLOR,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );
            // parent.spawn(
            //     TextBundle::from_section(
            //         format!("with {} colliders", colliders.coords.len()),
            //         TextStyle {
            //             font_size: 40.0,
            //             color: TEXT_COLOR,
            //             ..default()
            //         },
            //     )
            //     .with_style(Style {
            //         margin: UiRect::all(Val::Px(50.0)),
            //         ..default()
            //     }),
            // );
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
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            ButtonAction::Edit,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_section("Edit", button_text_style.clone()));
                        });
                });
        });
}

#[derive(Component)]
enum ButtonAction {
    Menu,
    Retry,
    Edit,
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
                    ButtonAction::Edit => next_state.set(GameMode::Edit),
                    ButtonAction::Retry => next_state.set(GameMode::Play),
                    ButtonAction::Menu => (),
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

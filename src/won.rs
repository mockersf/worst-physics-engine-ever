use bevy::prelude::*;

use crate::{
    edit::EnabledColliders, FontHandle, GameMode, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON,
    TEXT_COLOR,
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

fn setup(mut commands: Commands, colliders: Res<EnabledColliders>, font: Res<FontHandle>) {
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
                    ButtonAction::Retry => next_state.set(GameMode::Edit),
                    ButtonAction::Menu => (),
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkWorldBundle, LevelSelection};

use crate::{
    edit::EnabledColliders, FontHandle, GameMode, LdtkHandle, HOVERED_BUTTON, NORMAL_BUTTON,
    PRESSED_BUTTON, TEXT_COLOR,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::Menu), setup)
            .add_systems(OnExit(GameMode::Menu), exit_screen)
            .add_systems(Update, button_system.run_if(in_state(GameMode::Menu)));
    }
}

#[derive(Component)]
struct OnMenuScreen;

fn exit_screen(mut commands: Commands, query: Query<Entity, With<OnMenuScreen>>) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands, font: Res<FontHandle>) {
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
            OnMenuScreen,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "Worst Physics Engine Ever",
                    TextStyle {
                        font_size: 70.0,
                        color: TEXT_COLOR,
                        font: font.0.clone(),
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                })
                .with_text_alignment(TextAlignment::Center),
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
                            ButtonAction::Start,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Start",
                                button_text_style.clone(),
                            ));
                        });
                });
        });
}

#[derive(Component)]
enum ButtonAction {
    Start,
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameMode>>,
    world: Res<LdtkHandle>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => {
                match button {
                    ButtonAction::Start => {
                        next_state.set(GameMode::Edit);
                        commands.insert_resource(LevelSelection::Uid(0));
                        commands.spawn(LdtkWorldBundle {
                            ldtk_handle: world.0.clone(),
                            ..Default::default()
                        });
                        commands.insert_resource(EnabledColliders::default());
                    }
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

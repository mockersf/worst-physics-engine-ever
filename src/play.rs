use crate::{
    components::*, edit::EnabledColliders, FontHandle, GameKind, GameMode, HOVERED_BUTTON,
    NORMAL_BUTTON, PRESSED_BUTTON, TEXT_COLOR,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use std::collections::{HashMap, HashSet};

use bevy_rapier2d::prelude::*;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                ignore_gravity_if_climbing,
                detect_collision_with_environment,
                movement,
                patrol,
                ground_detection,
                update_on_ground,
                check_lost_condition,
                spawn_complete_wall_collision,
                update_level_selection,
                spawn_ground_sensor,
                button_system,
                camera_fit_inside_current_level,
                spawn_complete_wall_collision,
            )
                .run_if(in_state(GameMode::Play)),
        )
        .add_systems(OnEnter(GameMode::Play), setup_play_mode)
        .add_systems(OnExit(GameMode::Play), exit_mode)
        .add_systems(Update, freeze.run_if(not(in_state(GameMode::Play))));
    }
}

fn exit_mode(
    mut commands: Commands,
    query: Query<Entity, With<OnPlayMode>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut player: Query<&mut Velocity, With<Player>>,
) {
    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    rapier_config.gravity = Vec2::new(0.0, 0.0);
    player.single_mut().linvel = Vec2::ZERO;
}

fn freeze(mut player: Query<&mut Velocity, With<Player>>) {
    if let Ok(mut player) = player.get_single_mut() {
        if player.is_changed() {
            player.linvel = Vec2::ZERO;
        }
    }
}

fn movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<
        (
            &mut Velocity,
            &mut Climber,
            &GroundDetection,
            &mut TextureAtlasSprite,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (mut velocity, mut climber, ground_detection, mut atlas) in &mut query {
        let right = if input.pressed(KeyCode::D) { 1. } else { 0. };
        let left = if input.pressed(KeyCode::A) { 1. } else { 0. };

        velocity.linvel.x = (right - left) * 200.;

        if velocity.linvel.x != 0.0 {
            atlas.index = ((time.elapsed_seconds() * 15.0).floor() as usize) % 6 + 7;
        }
        if velocity.linvel.x < 0.0 {
            atlas.flip_x = true;
        }
        if velocity.linvel.x > 0.0 {
            atlas.flip_x = false;
        }
        if velocity.linvel.x == 0.0 {
            atlas.index = ((time.elapsed_seconds() * 5.0).floor() as usize) % 4;
        }

        if climber.intersecting_climbables.is_empty() {
            climber.climbing = false;
        } else if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::S) {
            climber.climbing = true;
        }

        if climber.climbing {
            let up = if input.pressed(KeyCode::W) { 1. } else { 0. };
            let down = if input.pressed(KeyCode::S) { 1. } else { 0. };

            velocity.linvel.y = (up - down) * 200.;
            if velocity.linvel.y != 0.0 {
                atlas.index = ((time.elapsed_seconds() * 5.0).floor() as usize) % 4 + 14;
            }
        } else if velocity.linvel.y > 10.0 {
            atlas.index = 35;
        } else if velocity.linvel.y < -10.0 {
            atlas.index = 36;
        }

        if input.just_pressed(KeyCode::Space) && (ground_detection.on_ground || climber.climbing) {
            velocity.linvel.y = 500.;
            climber.climbing = false;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_complete_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &LevelIid)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    enabled: Res<EnabledColliders>,
    game_kind: Res<State<GameKind>>,
) {
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            if matches!(game_kind.get(), GameKind::Platformer)
                || enabled.coords.contains(&grid_coords)
            {
                level_to_wall_locations
                    .entry(grandparent.get())
                    .or_default()
                    .insert(grid_coords);
            }
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_iid)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(ldtk_projects.single())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_standalone()
                    .get_loaded_level_by_iid(&level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}

fn detect_collision_with_environment(
    mut climbers: Query<&mut Climber>,
    climbables: Query<Entity, With<Climbable>>,
    mut collisions: EventReader<CollisionEvent>,
    player: Query<&Player>,
    chests: Query<&Chest>,
    mut next_state: ResMut<NextState<GameMode>>,
) {
    for collision in collisions.read() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b, _) => {
                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_a), climbables.get(*collider_b))
                {
                    climber.intersecting_climbables.insert(climbable);
                }
                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_b), climbables.get(*collider_a))
                {
                    climber.intersecting_climbables.insert(climbable);
                };
                if (player.contains(*collider_a) && chests.contains(*collider_b))
                    || (player.contains(*collider_b) && chests.contains(*collider_a))
                {
                    next_state.set(GameMode::Won);
                }
            }
            CollisionEvent::Stopped(collider_a, collider_b, _) => {
                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_a), climbables.get(*collider_b))
                {
                    climber.intersecting_climbables.remove(&climbable);
                }

                if let (Ok(mut climber), Ok(climbable)) =
                    (climbers.get_mut(*collider_b), climbables.get(*collider_a))
                {
                    climber.intersecting_climbables.remove(&climbable);
                }
            }
        }
    }
}

fn ignore_gravity_if_climbing(mut query: Query<(&Climber, &mut GravityScale), Changed<Climber>>) {
    for (climber, mut gravity_scale) in &mut query {
        if climber.climbing {
            gravity_scale.0 = 0.0;
        } else {
            gravity_scale.0 = 1.0;
        }
    }
}

fn patrol(mut query: Query<(&mut Transform, &mut Velocity, &mut Patrol)>) {
    for (mut transform, mut velocity, mut patrol) in &mut query {
        if patrol.points.len() <= 1 {
            continue;
        }

        let mut new_velocity =
            (patrol.points[patrol.index] - transform.translation.truncate()).normalize() * 75.;

        if new_velocity.dot(velocity.linvel) < 0. {
            if patrol.index == 0 {
                patrol.forward = true;
            } else if patrol.index == patrol.points.len() - 1 {
                patrol.forward = false;
            }

            transform.translation.x = patrol.points[patrol.index].x;
            transform.translation.y = patrol.points[patrol.index].y;

            if patrol.forward {
                patrol.index += 1;
            } else {
                patrol.index -= 1;
            }

            new_velocity =
                (patrol.points[patrol.index] - transform.translation.truncate()).normalize() * 75.;
        }

        velocity.linvel = new_velocity;
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &LevelIid), (Without<OrthographicProjection>, Without<Player>)>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    window: Query<&Window>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let window_aspect_ratio = window
            .get_single()
            .map(|w| w.width() / w.height())
            .unwrap_or(1.);
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

        for (level_transform, level_iid) in &level_query {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_projects.single())
                .expect("Project should be loaded if level has spawned");

            let level = ldtk_project
                .get_raw_level_by_iid(&level_iid.to_string())
                .expect("Spawned level should exist in LDtk project");

            if level_selection.is_match(&LevelIndices::default(), level) {
                let level_ratio = level.px_wid as f32 / level.px_hei as f32;
                orthographic_projection.viewport_origin = Vec2::ZERO;
                if level_ratio > window_aspect_ratio {
                    // level is wider than the screen
                    let height = (level.px_hei as f32 / 9.).round() * 9.;
                    let width = height * window_aspect_ratio;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed {
                            width,
                            height: height - 4.0,
                        };
                    camera_transform.translation.x =
                        (player_translation.x - level_transform.translation.x - width / 2.)
                            .clamp(0., level.px_wid as f32 - width);
                    camera_transform.translation.y = 0.;
                } else {
                    // level is taller than the screen
                    let width = (level.px_wid as f32 / 16.).round() * 16.;
                    let height = width / window_aspect_ratio;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };
                    camera_transform.translation.y =
                        (player_translation.y - level_transform.translation.y - height / 2.)
                            .clamp(0., level.px_hei as f32 - height);
                    camera_transform.translation.x = 0.;
                }

                camera_transform.translation.x += level_transform.translation.x;
                camera_transform.translation.y += level_transform.translation.y;
            }
        }
    }
}

fn update_level_selection(
    level_query: Query<(&LevelIid, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_projects: Query<&Handle<LdtkProject>>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for (level_iid, level_transform) in &level_query {
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single())
            .expect("Project should be loaded if level has spawned");

        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        let level_bounds = Rect {
            min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
            max: Vec2::new(
                level_transform.translation.x + level.px_wid as f32,
                level_transform.translation.y + level.px_hei as f32,
            ),
        };

        for player_transform in &player_query {
            if player_transform.translation.x < level_bounds.max.x
                && player_transform.translation.x > level_bounds.min.x
                && player_transform.translation.y < level_bounds.max.y
                && player_transform.translation.y > level_bounds.min.y
                && !level_selection.is_match(&LevelIndices::default(), level)
            {
                *level_selection = LevelSelection::iid(level.iid.clone());
            }
        }
    }
}

fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Collider), Added<GroundDetection>>,
) {
    for (entity, shape) in &detect_ground_for {
        if let Some(cuboid) = shape.as_cuboid() {
            let Vec2 {
                x: half_extents_x,
                y: half_extents_y,
            } = cuboid.half_extents();

            let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);

            let sensor_translation = Vec3::new(0., -half_extents_y, 0.);

            commands.entity(entity).with_children(|builder| {
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(detector_shape)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation))
                    .insert(GlobalTransform::default())
                    .insert(GroundSensor {
                        ground_detection_entity: entity,
                        intersecting_ground_entities: HashSet::new(),
                    });
            });
        }
    }
}

fn ground_detection(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<With<Collider>, Without<Sensor>>,
) {
    for collision_event in collisions.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.insert(*e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.insert(*e2);
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.remove(e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.remove(e2);
                    }
                }
            }
        }
    }
}

fn update_on_ground(
    mut ground_detectors: Query<&mut GroundDetection>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    for sensor in &ground_sensors {
        if let Ok(mut ground_detection) = ground_detectors.get_mut(sensor.ground_detection_entity) {
            ground_detection.on_ground = !sensor.intersecting_ground_entities.is_empty();
        }
    }
}

#[derive(Component)]
struct OnPlayMode;

fn setup_play_mode(
    mut commands: Commands,
    world_query: Query<Entity, With<Handle<LdtkProject>>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    colliders: Res<EnabledColliders>,
    font: Res<FontHandle>,
    game_kind: Res<State<GameKind>>,
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
    rapier_config.gravity = Vec2::new(0.0, -2000.0);

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
        .insert(OnPlayMode)
        .with_children(|parent| {
            if matches!(game_kind.get(), GameKind::Puzzle) {
                parent
                    .spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: NORMAL_BUTTON.into(),
                            border_color: BorderColor(HOVERED_BUTTON),
                            ..default()
                        },
                        ButtonAction::Edit,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section("Edit", button_text_style.clone()));
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
                            value: " colliders".to_string(),
                            style: TextStyle {
                                font_size: 20.,
                                color: TEXT_COLOR,
                                font: font.0.clone(),
                            },
                        },
                    ])
                    .with_style(Style {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    }),
                );
            }
            parent.spawn(TextBundle::from_sections([
                TextSection {
                    value: "60.0".to_string(),
                    style: TextStyle {
                        font_size: 20.,
                        color: Color::GREEN,
                        font: font.0.clone(),
                    },
                },
                TextSection {
                    value: "s".to_string(),
                    style: TextStyle {
                        font_size: 20.,
                        color: TEXT_COLOR,
                        font: font.0.clone(),
                    },
                },
            ]));
        });

    commands.insert_resource(Playthrough {
        timer: Timer::from_seconds(60.0, TimerMode::Once),
        lost_chest: false,
        lost_player: false,
    });
}

#[derive(Resource)]
pub struct Playthrough {
    pub timer: Timer,
    pub lost_player: bool,
    pub lost_chest: bool,
}

#[derive(Component)]
enum ButtonAction {
    Edit,
}

fn check_lost_condition(
    mut next: ResMut<NextState<GameMode>>,
    chest: Query<&Transform, With<Chest>>,
    player: Query<&Transform, With<Player>>,
    respawn: Query<&Respawn>,
    time: Res<Time>,
    mut playthrough: ResMut<Playthrough>,
    mut text: Query<&mut Text>,
) {
    if respawn.iter().count() > 0 {
        return;
    }
    let transform = chest.single();
    if transform.translation.y < -500. {
        next.set(GameMode::Lost);
        playthrough.lost_chest = true;
    }
    let transform = player.single();
    if transform.translation.y < -500. {
        next.set(GameMode::Lost);
        playthrough.lost_player = true;
    }
    if playthrough.timer.tick(time.delta()).just_finished() {
        next.set(GameMode::Lost);
    }
    for mut text in &mut text {
        if text.sections[0].style.color != TEXT_COLOR {
            text.sections[0].value = format!("{:.1}", playthrough.timer.remaining_secs());
            if playthrough.timer.remaining_secs() < 20.0 {
                text.sections[0].style.color = Color::RED;
            }
        }
    }
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
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

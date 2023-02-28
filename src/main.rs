use std::ptr::null_mut;
use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::DefaultPlugins;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};
use bevy::time::FixedTimestep;
use bevy::window::PresentMode;
use rand::Rng;
use crate::Direction::{Down, Left, Right, Up};

const TIME_STEP: f64 = 1. / 9.;
const WINDOW_WIDTH: f32 = 700.;
const WINDOW_HEIGHT: f32 = 700.;
const GRID_SIZE: i32 = 28;
const GRID_SQUARE_SIZE: f32 = (WINDOW_WIDTH / GRID_SIZE as f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Snake Game".to_string(),
                width: WINDOW_WIDTH,
                height: WINDOW_HEIGHT,
                present_mode: PresentMode::AutoVsync,
                resizable: false,
                ..default()
            },
            ..default()
        }))
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(handle_movement_input)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP))
                .with_system(handle_movement)
                .with_system(handle_eat_food.after(handle_movement))
                .with_system(check_for_death.after(handle_movement))
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(2.))
                .with_system(spawn_food)
        )
        .add_startup_system(setup)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
        )
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .run();
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction
}

#[derive(Component)]
struct SnakeSegment;


#[derive(Component)]
struct Food;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    println!("Setting up...");

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::MIDNIGHT_BLUE,
                ..default()
            },
            transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
            ..default()
        },
        SnakeHead {
            direction: Up
        },
        Position {
            x: 0,
            y: 0
        }
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                ..default()
            },
            transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
            ..default()
        },
        SnakeSegment,
        Position {
            x: 0,
            y: -1
        }
    ));
}

fn spawn_food(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                ..default()
            },
            transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
            ..default()
        },
        Food,
        Position {
            x: rand::thread_rng().gen_range(0..GRID_SIZE),
            y: rand::thread_rng().gen_range(0..GRID_SIZE)
        }
    ));
}

fn handle_movement_input(keys: Res<Input<KeyCode>>, mut query: Query<&mut SnakeHead>) {
    let mut head = query.iter_mut().next().unwrap();

    if keys.pressed(KeyCode::W) && head.direction != Down {
        head.direction = Up;
    } else if keys.pressed(KeyCode::S) && head.direction != Up {
        head.direction = Down;
    } else if keys.pressed(KeyCode::A) && head.direction != Right {
        head.direction = Left;
    } else if keys.pressed(KeyCode::D) && head.direction != Left {
        head.direction = Right;
    }
}

fn handle_movement(mut query: Query<(&mut SnakeHead, &mut Position), (With<SnakeHead>, Without<SnakeSegment>)>, mut segment_query: Query<(&mut Position), (With<SnakeSegment>, Without<SnakeHead>)>) {
    let tuple = query.iter_mut().next().unwrap();
    let head = tuple.0;
    let mut pos = tuple.1;

    let prev_transform = pos.clone();

    match head.direction {
        Up => {
            pos.y += 1;
        }
        Down => {
            pos.y -= 1;
        }
        Left => {
            pos.x -= 1;
        }
        Right => {
            pos.x += 1;
        }
    }

    let mut prev_translation = prev_transform;
    for mut segment in segment_query.iter_mut() {
        let prev = segment.clone();
        segment.x = prev_translation.x;
        segment.y = prev_translation.y;

        prev_translation = prev;
    }
}

fn handle_eat_food(mut commands: Commands, head_query: Query<&Position, With<SnakeHead>>, food_query: Query<(Entity, &Position), With<Food>>) {
    let head_pos = head_query.single();

    for food in food_query.iter() {
        if head_pos.x == food.1.x && head_pos.y == food.1.y {
            commands.entity(food.0).despawn();
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN,
                        ..default()
                    },
                    transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
                    ..default()
                },
                SnakeSegment,
                Position {
                    x: -1,
                    y: -1
                }
            ));
        }
    }
}

fn check_for_death(mut commands: Commands, entity_query: Query<Entity, Without<Camera2d>>, head_query: Query<&Position, With<SnakeHead>>, segments_query: Query<&Position, With<SnakeSegment>>) {
    let head = head_query.single();
    for segment in segments_query.iter() {
        if head.x == segment.x && head.y == segment.y {
            for entity in entity_query.iter() {
                commands.entity(entity).despawn();
            }

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::MIDNIGHT_BLUE,
                        ..default()
                    },
                    transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
                    ..default()
                },
                SnakeHead {
                    direction: Up
                },
                Position {
                    x: 0,
                    y: 0
                }
            ));

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::GREEN,
                        ..default()
                    },
                    transform: Transform::default().with_scale(Vec3::splat(GRID_SQUARE_SIZE)),
                    ..default()
                },
                SnakeSegment,
                Position {
                    x: 0,
                    y: -1
                }
            ));
        }
    }
}

fn grid_coord_to_coords(coord: i32) -> f32 {
    coord as f32 * GRID_SQUARE_SIZE
}

fn coords_to_grid_coord(coord: f32) -> i32 {
    (coord / GRID_SQUARE_SIZE) as i32
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, GRID_SIZE as f32),
            convert(pos.y as f32, window.height() as f32, GRID_SIZE as f32),
            0.0,
        );
    }
}

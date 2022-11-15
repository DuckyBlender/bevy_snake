// TODO: Fix the bug where the snake can go through itself when clicking the opposite direction in a specific way

// Don't show the console window
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::time::FixedTimestep;
use rand::Rng;

const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

const ARENA_HEIGHT: u32 = 12;
const ARENA_WIDTH: u32 = 12;

const MAX_FOOD_COUNT: u32 = 5;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

struct SpawnFoodEvent;
struct GameOverEvent;
struct GrowthEvent;

#[derive(Default, Resource)]
struct LastTailPosition(Option<Position>);

#[derive(Component)]
struct SnakeSegment;

#[derive(Default, Deref, DerefMut, Resource)]
struct SnakeSegments(Vec<Entity>);

#[derive(Component)]
struct Food;

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_scoreboard(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/PixeloidSans.ttf"),
                    font_size: 20.0,
                    color: Color::rgb(1., 1., 1.),
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/PixeloidSans.ttf"),
                font_size: 20.0,
                color: Color::rgb(0.5, 1.0, 1.0),
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    );
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>, mut foodevent_writer: EventWriter<SpawnFoodEvent>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(SnakeHead {
                direction: Direction::Up,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
    for _ in 0..MAX_FOOD_COUNT {
        foodevent_writer.send(SpawnFoodEvent);
    }
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.75))
        .id()
}

fn snake_movement(
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
    segments: ResMut<SnakeSegments>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_writer.send(GameOverEvent);
        }
        if segment_positions.contains(&head_pos) {
            game_over_writer.send(GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
    }
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };
        if dir != head.direction.opposite() {
            head.direction = dir;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    foodevent_writer: EventWriter<SpawnFoodEvent>
) {
    // If the game is over, remove all entities and spawn a new snake
    if reader.iter().next().is_some() {
        //println!("Game Over!");
        // Play the death sound
        let death = asset_server.load("sounds/die.ogg");
        audio.play(death);
        // Remove all entities
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn_recursive();
        }
        scoreboard.score = 0;
        spawn_snake(commands, segments_res, foodevent_writer);
    }
}

#[allow(clippy::too_many_arguments)]
fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut foodevent_writer: EventWriter<SpawnFoodEvent>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
                foodevent_writer.send(SpawnFoodEvent);
                let eat = asset_server.load("sounds/eat.ogg");
                audio.play(eat);
                scoreboard.score += 1;
            }
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.iter().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = match windows.get_primary() {
        Some(window) => window,
        None => return,
    };
    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = match windows.get_primary() {
        Some(window) => window,
        None => return,
    };
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn food_spawner(
    mut commands: Commands,
    segments: Res<SnakeSegments>,
    mut positions: Query<&Position>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    mut foodevent_listener: EventReader<SpawnFoodEvent>,
) {
    // Check for food spawn event
    // Loop through all spawn event counts
    for _ in foodevent_listener.iter() {
        let food_count: u32 = food_positions.iter().count() as u32;
        if food_count >= MAX_FOOD_COUNT {
            //println!("Already {} food on the board", food_count);
            return;
        }
        let mut rng = rand::thread_rng();
        // Loop until the spawned food isn't already occupied.
        loop {
            // Generate two random numbers between 0 and ARENA_WIDTH and ARENA_HEIGHT
            let x_pos = rng.gen_range(0..ARENA_WIDTH) as i32;
            let y_pos = rng.gen_range(0..ARENA_HEIGHT) as i32;
            // Check if the position is already occupied by a snake segment
            let mut segment_positions = segments.iter().map(|e| *positions.get_mut(*e).unwrap());
            if segment_positions.any(|p| p.x == x_pos && p.y == y_pos) {
                //println!("Tried spawning food on snake! ({}, {}) Trying again...", x_pos, y_pos);
                continue;
            }
            //println!("Spawning food at {}, {}", x_pos, y_pos);
            //println!("Segments: {:?}", segment_positions);
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: FOOD_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(Food)
                .insert(Position { x: x_pos, y: y_pos })
                .insert(Size::square(0.8));
            break;
        }
    }
}

fn main() {
    App::new()
        // Background color
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        // Window
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Snake!".to_string(),
                width: 500.0,
                height: 500.0,
                resizable: false,
                ..default()
            },
            ..default()
        }))
        // Initialize scoreboard
        .insert_resource(Scoreboard { score: 0 })
        // Setup different systems
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_startup_system(setup_scoreboard)
        // Setup snake segments
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_event::<GrowthEvent>()
        .add_event::<SpawnFoodEvent>()
        // Setup movement
        .add_system(snake_movement_input.before(snake_movement))
        // Setup game over event
        .add_event::<GameOverEvent>()
        
        // Setup timestep for snake stuff
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0 / 6.0))
                .with_system(snake_movement)
                .with_system(snake_eating.after(snake_movement))
                .with_system(snake_growth.after(snake_eating)),
        )
        // Setup game over after the snake has moved
        .add_system(check_game_over.after(snake_movement))
        // Setup food spawner
        .add_system(food_spawner)
        // Setup scaling
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        // Setup scoreboard
        .add_system(update_scoreboard)
        // Close on escape
        .add_system(bevy::window::close_on_esc)
        // Run
        .run();
}

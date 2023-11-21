//! A simple implementation of space invaders.

use bevy::{
    prelude::*,
    sprite::collide_aabb::collide,
    sprite::MaterialMesh2dBundle
};
use rand::Rng;
use crate::constants::*;
use crate::components::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(LivesCounter { count: 3 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_resource::<AlienSpawnTimer>()
        .add_event::<CollisionEvent>()
        .add_state::<GameState>()
        .add_systems(OnEnter(GameState::MainMenu), setup)
        .add_systems(
            FixedUpdate,
            start_game
            .run_if(in_state(GameState::MainMenu))
        )
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_spaceship,
                fire_laser,
                check_for_collisions,
                tick_alien_spawn_timer,
                spawn_alien,
            )
                .chain()
                .run_if(in_state(GameState::InGame))
        )
        .add_systems(Update, (update_scoreboard, update_lives_remaining).run_if(in_state(GameState::InGame)))
        .add_systems(OnEnter(GameState::GameOver), display_game_over)
        .add_systems(Update, bevy::window::close_on_esc) // apply to all states
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
}

impl States for GameState {}

// Track the game's score
#[derive(Resource)]
struct Scoreboard {
    score: i16,
}

// Track the number of lives remaining
#[derive(Resource)]
struct LivesCounter {
    count: u16,
}

#[derive(Resource)]
struct AlienSpawnTimer {
    timer: Timer,
}

impl Default for AlienSpawnTimer {
    fn default() -> Self {
        AlienSpawnTimer { 
            timer: Timer::from_seconds(ALIEN_SPAWN_TIME, TimerMode::Repeating),
        }
    }
}

// Add the game's entities
fn setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Spaceship
    let spaceship_y = BOTTOM_WALL + GAP_BETWEEN_SPACESHIP_AND_FLOOR;
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, spaceship_y, 0.0),
                scale: SPACESHIP_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: SPACESHIP_COLOR,
                ..default()
            },
            ..default()
        },
        Spaceship,
        Collider,
    ));

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
            TextSection::new(
                "  Lives remaining: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    );

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // Instructions
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Welcome to Space Invaders.
                \nUse the arrow keys to control your spaceship.
                \nUse the spacebar to fire lasers at aliens. 
                \nYou will gain points for any aliens you hit and lose points for any aliens which get past you.
                \nYou will lose a life for any aliens which hit you.
                \nPress esc to leave the game at any point.
                \nPress enter to start.",
                TextStyle {
                    font_size: INSTRUCTIONS_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },),
        Instructions,
    ));
}

// Use keyboard input to move the spaceship
fn move_spaceship(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Spaceship>>,
    time: Res<Time>,
) {
    let mut spaceship_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    // Calculate the new horizontal spaceship position based on player input
    let new_spaceship_position =
        spaceship_transform.translation.x + direction * SPACESHIP_SPEED * time.delta_seconds();

    // Update the spaceship position,
    // making sure it doesn't cause the spaceship to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + SPACESHIP_SIZE.x / 2.0 + SPACESHIP_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - SPACESHIP_SIZE.x / 2.0 - SPACESHIP_PADDING;

    spaceship_transform.translation.x = new_spaceship_position.clamp(left_bound, right_bound);
}

// If return is pressed, remove the instructions and change the game state to in-game
fn start_game(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut query: Query<Entity, With<Instructions>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        let instructions = query.single_mut();
        commands.entity(instructions).despawn();
        next_state.set(GameState::InGame);
    }
}

fn fire_laser(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Spaceship>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut spaceship_transform = query.single_mut().translation;
    spaceship_transform.y = spaceship_transform.y + SPACESHIP_SIZE.y;
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(LASER_COLOR)),
                transform: Transform::from_translation(spaceship_transform).with_scale(LASER_SIZE),
                ..default()
            },
            Laser,
            Velocity(INITIAL_LASER_DIRECTION.normalize() * LASER_SPEED),
        ));
    }
}

// Apply velocity to any entity with the Velocity component
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn update_lives_remaining(lives_counter: Res<LivesCounter>, mut query: Query<&mut Text>, mut next_state: ResMut<NextState<GameState>>) {
    let lives_remaining = lives_counter.count;
    // Check whether the game should end
    if lives_remaining < 1 {
        next_state.set(GameState::GameOver);
    }
    let mut text = query.single_mut();
    text.sections[3].value = lives_remaining.to_string();
}

// Increment timer for spawning aliens
fn tick_alien_spawn_timer(
    mut alien_spawn_timer: ResMut<AlienSpawnTimer>,
    time: Res<Time>
) {
    alien_spawn_timer.timer.tick(time.delta());
}

// Spawn an alien from a random starting position
fn spawn_alien(
    mut commands: Commands,
    alien_spawn_timer: Res<AlienSpawnTimer>
) {
    if alien_spawn_timer.timer.finished() {
        let lower_bound = LEFT_WALL + ALIEN_SIZE.x;
        let upper_bound = RIGHT_WALL - ALIEN_SIZE.x;
        let starting_x = rand::thread_rng().gen_range(lower_bound..upper_bound);
        let starting_y = TOP_WALL - ALIEN_SIZE.x / 2.0;
        let alien_position = Vec2::new(starting_x, starting_y);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: ALIEN_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: alien_position.extend(0.0),
                    scale: Vec3::new(ALIEN_SIZE.x, ALIEN_SIZE.y, 1.0),
                    ..default()
                },
                ..default()
            },
            Alien,
            Collider,
            Velocity(INITIAL_ALIEN_DIRECTION.normalize() * ALIEN_SPEED),
        ));
    }
    
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut lives_remaining: ResMut<LivesCounter>,
    laser_query: Query<(Entity, &Transform), With<Laser>>,
    collider_query: Query<(Entity, &Transform, Option<&Alien>), With<Collider>>,
    spaceship_query: Query<&Transform, With<Spaceship>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for (collider_entity, transform, maybe_alien) in &collider_query {
        // Check if collision was with a laser
        for (laser, laser_transform) in laser_query.iter() {
            let laser_size = laser_transform.scale.truncate();
            let collision = collide(
                laser_transform.translation,
                laser_size,
                transform.translation,
                transform.scale.truncate(),
            );
            if collision.is_some() {
                // Sends a collision event so that other systems can react to the collision
                collision_events.send_default();
    
                // Aliens should be despawned and increment the scoreboard on collision
                if maybe_alien.is_some() {
                    scoreboard.score += 3;
                    commands.entity(collider_entity).despawn();
                    commands.entity(laser).despawn();
                }
            }
        }

        // Check if collision was with spaceship
        let spaceship_transform = spaceship_query.single();
        let spaceship_size = spaceship_transform.scale.truncate();
        let spaceship_collision = collide(
            spaceship_transform.translation,
            spaceship_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if spaceship_collision.is_some() && maybe_alien.is_some() {
            lives_remaining.count -= 1;
            commands.entity(collider_entity).despawn();
        }

        // Check if collision was with bottom wall
        let bottom_wall_collision = collide(
            WallLocation::Bottom.position_3d(),
            WallLocation::Bottom.size(),
            transform.translation,
            transform.scale.truncate(),
        );
        if bottom_wall_collision.is_some() && maybe_alien.is_some() {
            scoreboard.score -= 1;
            commands.entity(collider_entity).despawn();
        }
    }
}

fn display_game_over(
    mut commands: Commands, 
    scoreboard: Res<Scoreboard>) {
        let final_score = scoreboard.score.to_string();
        commands.spawn(
        TextBundle::from_sections([
                    TextSection::new(
                        "Game Over",
                        TextStyle {
                            font_size: GAME_OVER_FONT_SIZE,
                            color: TEXT_COLOR,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\nYour score: ",
                        TextStyle {
                            font_size: GAME_OVER_FONT_SIZE,
                            color: TEXT_COLOR,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        final_score,
                        TextStyle {
                            font_size: GAME_OVER_FONT_SIZE,
                            color: SCORE_COLOR,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\nPress esc to exit",
                        TextStyle {
                            font_size: INSTRUCTIONS_FONT_SIZE,
                            color: TEXT_COLOR,
                            ..default()
                        },
                    ),
                ])
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..default()
                }),
        );
}

mod components;
mod constants;
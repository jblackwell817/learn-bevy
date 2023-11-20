//! Shows how to render simple primitive shapes with a single colour.

//! A simplified implementation of the classic game "Breakout".

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle, app::AppExit,
};
use rand::Rng;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 700.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

const BULLET_SIZE: Vec3 = Vec3::new(15.0, 15.0, 0.0);
const BULLET_SPEED: f32 = 700.0;
const ALIEN_SPEED: f32 = 300.0;
const INITIAL_BULLET_DIRECTION: Vec2 = Vec2::new(0., 1.);
const INITIAL_ALIEN_DIRECTION: Vec2 = Vec2::new(0., -1.);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const ALIEN_SIZE: Vec2 = Vec2::new(70., 30.);

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const GAME_OVER_FONT_SIZE: f32 = 60.0;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const PADDLE_HIT_COLOR: Color = Color::rgb(1.0, 0.3, 0.7);
const BULLET_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const ALIEN_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const LIVES_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

const ALIEN_SPAWN_TIME: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(LivesCounter { count: 3 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .init_resource::<AlienSpawnTimer>()
        .add_event::<CollisionEvent>()
        .add_state::<GameState>()
        .add_systems(OnEnter(GameState::InGame), setup)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                fire_gun,
                check_for_collisions,
                play_collision_sound,
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
    InGame,
    GameOver,
}

impl States for GameState {}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Bullet;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Alien;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn position_3d(&self) -> Vec3 {
        match self {
            WallLocation::Left => Vec3::new(LEFT_WALL, 0., 0.),
            WallLocation::Right => Vec3::new(RIGHT_WALL, 0., 0.),
            WallLocation::Bottom => Vec3::new(0., BOTTOM_WALL, 0.),
            WallLocation::Top => Vec3::new(0., TOP_WALL, 0.),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: i16,
}

// This resource tracks the number of lives remaining
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

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let bullet_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(bullet_collision_sound));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
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
}

fn move_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_seconds();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

fn fire_gun(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut paddle_transform = query.single_mut().translation;
    paddle_transform.y = paddle_transform.y + PADDLE_SIZE.y;
    if keyboard_input.just_pressed(KeyCode::Space) {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(BULLET_COLOR)),
                transform: Transform::from_translation(paddle_transform).with_scale(BULLET_SIZE),
                ..default()
            },
            Bullet,
            Velocity(INITIAL_BULLET_DIRECTION.normalize() * BULLET_SPEED),
        ));
    }
}

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

fn tick_alien_spawn_timer(
    mut alien_spawn_timer: ResMut<AlienSpawnTimer>,
    time: Res<Time>
) {
    alien_spawn_timer.timer.tick(time.delta());
}

fn spawn_alien(
    mut commands: Commands,
    alien_spawn_timer: Res<AlienSpawnTimer>
) {
    if alien_spawn_timer.timer.finished() {
        // Pick a random starting position
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
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    collider_query: Query<(Entity, &Transform, Option<&Alien>), With<Collider>>,
    paddle_query: Query<(Entity, &Transform), With<Paddle>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    for (collider_entity, transform, maybe_alien) in &collider_query {
        // Check if collision was with a bullet
        for (bullet, bullet_transform) in bullet_query.iter() {
            let bullet_size = bullet_transform.scale.truncate();
            let collision = collide(
                bullet_transform.translation,
                bullet_size,
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
                    commands.entity(bullet).despawn();
                }
            }
        }

        // Check if collision was with paddle
        let (paddle, paddle_transform) = paddle_query.single();
        let paddle_size = paddle_transform.scale.truncate();
        let paddle_collision = collide(
            paddle_transform.translation,
            paddle_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if paddle_collision.is_some() && maybe_alien.is_some() {
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

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
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
                            color: LIVES_COLOR,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "\nYour score: ",
                        TextStyle {
                            font_size: GAME_OVER_FONT_SIZE,
                            color: LIVES_COLOR,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        final_score,
                        TextStyle {
                            font_size: GAME_OVER_FONT_SIZE,
                            color: LIVES_COLOR,
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
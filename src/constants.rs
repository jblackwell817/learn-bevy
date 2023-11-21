//! All of the constants used in the game

use bevy::prelude::*;

// Sizes and coordinates
pub const SPACESHIP_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
pub const GAP_BETWEEN_SPACESHIP_AND_FLOOR: f32 = 60.0;
pub const SPACESHIP_SPEED: f32 = 700.0;
pub const SPACESHIP_PADDING: f32 = 10.0;
pub const LASER_SIZE: Vec3 = Vec3::new(15.0, 15.0, 0.0);
pub const LASER_SPEED: f32 = 700.0;
pub const ALIEN_SPEED: f32 = 300.0;
pub const INITIAL_LASER_DIRECTION: Vec2 = Vec2::new(0., 1.);
pub const INITIAL_ALIEN_DIRECTION: Vec2 = Vec2::new(0., -1.);
pub const WALL_THICKNESS: f32 = 10.0;
pub const LEFT_WALL: f32 = -450.;
pub const RIGHT_WALL: f32 = 450.;
pub const BOTTOM_WALL: f32 = -300.;
pub const TOP_WALL: f32 = 300.;
pub const ALIEN_SIZE: Vec2 = Vec2::new(70., 30.);

// Text
pub const INSTRUCTIONS_FONT_SIZE: f32 = 15.0;
pub const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
pub const GAME_OVER_FONT_SIZE: f32 = 60.0;

// Colours of objects and text
pub const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
pub const SPACESHIP_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
pub const LASER_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
pub const ALIEN_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
pub const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
pub const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
pub const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

pub const ALIEN_SPAWN_TIME: f32 = 1.0; // new alien every second
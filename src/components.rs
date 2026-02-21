use bevy::prelude::*;

// --- Marker Components ---

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct EnemyBullet;

#[derive(Component)]
pub struct EnemyRow(pub usize);

#[derive(Component)]
pub struct ExplosionParticle;

#[derive(Component)]
pub struct TrailParticle;

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

// --- Physics ---

#[derive(Component)]
pub struct Velocity(pub Vec3);

// --- Game State ---

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

// --- Timers ---

#[derive(Resource)]
pub struct ShootCooldown {
    pub timer: Timer,
}

impl Default for ShootCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct EnemyShootTimer {
    pub timer: Timer,
}

impl Default for EnemyShootTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
        }
    }
}

// --- Enemy Movement ---

#[derive(Resource)]
pub struct EnemyDirection {
    pub dir: f32, // 1.0 = right, -1.0 = left
}

impl Default for EnemyDirection {
    fn default() -> Self {
        Self { dir: 1.0 }
    }
}

// --- Constants ---

pub const ARENA_WIDTH: f32 = 14.0;
pub const ARENA_HEIGHT: f32 = 20.0;
pub const ARENA_HALF_WIDTH: f32 = ARENA_WIDTH / 2.0;

pub const PLAYER_SPEED: f32 = 8.0;
pub const PLAYER_Y: f32 = 0.5;
pub const PLAYER_Z: f32 = 8.0;

pub const BULLET_SPEED: f32 = 15.0;
pub const ENEMY_BULLET_SPEED: f32 = 8.0;

pub const ENEMY_SPEED: f32 = 2.0;
pub const ENEMY_STEP_DOWN: f32 = 0.8;
pub const ENEMY_COLS: usize = 5;
pub const ENEMY_ROWS: usize = 4;
pub const ENEMY_SPACING: f32 = 1.8;
pub const ENEMY_START_Z: f32 = -4.0;
pub const ENEMY_START_Y: f32 = 0.5;

pub const COLLISION_DISTANCE: f32 = 0.8;
pub const GAME_OVER_Z: f32 = 7.0;

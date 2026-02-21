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
    WaveTransition,
    Victory,
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

// --- Wave System ---

#[derive(Resource)]
pub struct CurrentWave {
    pub wave: u32,
}

impl Default for CurrentWave {
    fn default() -> Self {
        Self { wave: 1 }
    }
}

#[derive(Resource)]
pub struct EnemySpeed {
    pub speed: f32,
}

impl Default for EnemySpeed {
    fn default() -> Self {
        Self { speed: 2.0 }
    }
}

#[derive(Resource)]
pub struct WaveTransitionTimer {
    pub timer: Timer,
}

pub struct WaveConfig {
    pub speed: f32,
    pub shoot_interval: f32,
    pub rows: usize,
    pub cols: usize,
    pub z_offset: f32,
}

pub fn wave_config(wave: u32) -> WaveConfig {
    match wave {
        1 => WaveConfig {
            speed: 2.0,
            shoot_interval: 1.5,
            rows: 4,
            cols: 5,
            z_offset: 0.0,
        },
        2 => WaveConfig {
            speed: 2.4,
            shoot_interval: 1.4,
            rows: 4,
            cols: 5,
            z_offset: 0.3,
        },
        3 => WaveConfig {
            speed: 2.8,
            shoot_interval: 1.3,
            rows: 4,
            cols: 5,
            z_offset: 0.6,
        },
        4 => WaveConfig {
            speed: 3.2,
            shoot_interval: 1.2,
            rows: 5,
            cols: 6,
            z_offset: 0.9,
        },
        5 => WaveConfig {
            speed: 3.6,
            shoot_interval: 1.1,
            rows: 5,
            cols: 6,
            z_offset: 1.2,
        },
        6 => WaveConfig {
            speed: 4.0,
            shoot_interval: 1.0,
            rows: 5,
            cols: 6,
            z_offset: 1.5,
        },
        7 => WaveConfig {
            speed: 4.4,
            shoot_interval: 0.9,
            rows: 6,
            cols: 7,
            z_offset: 1.8,
        },
        8 => WaveConfig {
            speed: 4.8,
            shoot_interval: 0.8,
            rows: 6,
            cols: 7,
            z_offset: 2.1,
        },
        9 => WaveConfig {
            speed: 5.2,
            shoot_interval: 0.7,
            rows: 6,
            cols: 7,
            z_offset: 2.4,
        },
        _ => WaveConfig {
            speed: 5.6,
            shoot_interval: 0.6,
            rows: 7,
            cols: 8,
            z_offset: 2.7,
        },
    }
}

#[derive(Component)]
pub struct Barrier {
    pub health: u8,
}

#[derive(Component)]
pub struct WaveTransitionUI;

#[derive(Component)]
pub struct VictoryUI;

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

pub const ENEMY_STEP_DOWN: f32 = 0.8;
pub const ENEMY_SPACING: f32 = 1.8;
pub const ENEMY_START_Z: f32 = -4.0;
pub const ENEMY_START_Y: f32 = 0.5;

pub const COLLISION_DISTANCE: f32 = 0.8;
pub const GAME_OVER_Z: f32 = 7.0;

pub const BARRIER_Z: f32 = 5.5;
pub const BARRIER_HEALTH: u8 = 2;
pub const BARRIER_BLOCK_SIZE: f32 = 0.28;
pub const BARRIER_BLOCK_SPACING: f32 = 0.32;
pub const BARRIER_COLLISION_DISTANCE: f32 = 0.35;

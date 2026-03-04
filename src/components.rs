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
pub struct EnemyCol(pub usize);

#[derive(Component)]
pub struct ExplosionParticle;

#[derive(Component)]
pub struct TrailParticle;

#[derive(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

/// Marks the player as temporarily invulnerable after respawning.
#[derive(Component)]
pub struct PlayerInvulnerable {
    pub timer: Timer,
    /// Toggles visibility for blinking effect.
    pub blink_timer: Timer,
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
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

/// High score persists across games within a session.
#[derive(Resource, Default)]
pub struct HighScore {
    pub value: u32,
}

// --- Lives ---

#[derive(Resource)]
pub struct Lives {
    pub count: u32,
}

impl Default for Lives {
    fn default() -> Self {
        Self { count: 3 }
    }
}

/// Timer for respawning the player after death (when lives remain).
#[derive(Resource)]
pub struct PlayerRespawnTimer {
    pub timer: Timer,
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
    pub base_speed: f32,
}

impl Default for EnemySpeed {
    fn default() -> Self {
        Self { base_speed: 1.5 }
    }
}

/// Tracks how many enemies were in the wave at spawn time,
/// used to calculate dynamic speed-up.
#[derive(Resource)]
pub struct InitialEnemyCount {
    pub count: u32,
}

impl Default for InitialEnemyCount {
    fn default() -> Self {
        Self { count: 55 }
    }
}

#[derive(Resource)]
pub struct WaveTransitionTimer {
    pub timer: Timer,
}

pub struct WaveConfig {
    pub base_speed: f32,
    pub shoot_interval: f32,
    pub rows: usize,
    pub cols: usize,
    pub z_offset: f32,
}

/// Returns configuration for a given wave number.
/// Formation is always 5×11 (like the original).
/// Each successive wave starts aliens slightly closer to the player
/// and increases base speed and fire rate.
pub fn wave_config(wave: u32) -> WaveConfig {
    let extra_z = (wave.saturating_sub(1)).min(8) as f32 * 0.5;
    let base_speed = 1.5 + (wave.saturating_sub(1)).min(12) as f32 * 0.25;
    let shoot_interval = (1.6 - (wave.saturating_sub(1)).min(10) as f32 * 0.08).max(0.5);

    WaveConfig {
        base_speed,
        shoot_interval,
        rows: 5,
        cols: 11,
        z_offset: extra_z,
    }
}

/// Points awarded per row, matching the original arcade game.
/// Row 0 is the topmost (farthest from player) = 30 pts (squid).
/// Rows 1–2 = 20 pts (crab). Rows 3–4 = 10 pts (octopus).
pub fn points_for_row(row: usize) -> u32 {
    match row {
        0 => 30,
        1 | 2 => 20,
        _ => 10,
    }
}

#[derive(Component)]
pub struct Barrier {
    pub health: u8,
}

#[derive(Component)]
pub struct MysteryShip;

/// Bonus points awarded when this mystery ship is destroyed.
#[derive(Component)]
pub struct MysteryShipPoints(pub u32);

/// Fired when a mystery ship is destroyed.
#[derive(Message, Clone)]
pub struct MysteryShipKilledEvent {
    pub points: u32,
    pub world_pos: Vec3,
}

/// Floating score popup that rises and fades after a mystery ship kill.
#[derive(Component)]
pub struct ScorePopup {
    pub timer: Timer,
    pub start_top: f32,
}

#[derive(Component)]
pub struct WaveTransitionUI;

// --- Mystery Ship ---

#[derive(Resource)]
pub struct MysteryShipTimer {
    pub timer: Timer,
}

impl Default for MysteryShipTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(25.0, TimerMode::Repeating),
        }
    }
}

pub const MYSTERY_SHIP_Z: f32 = -8.0;
pub const MYSTERY_SHIP_Y: f32 = 0.5;
pub const MYSTERY_SHIP_SPEED: f32 = 5.0;
pub const MYSTERY_SHIP_COLLISION_DISTANCE: f32 = 1.2;

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
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
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

pub const ARENA_WIDTH: f32 = 18.0;
pub const ARENA_HEIGHT: f32 = 24.0;
pub const ARENA_HALF_WIDTH: f32 = ARENA_WIDTH / 2.0;

pub const PLAYER_SPEED: f32 = 10.0;
pub const PLAYER_Y: f32 = 0.5;
pub const PLAYER_Z: f32 = 8.0;

pub const BULLET_SPEED: f32 = 18.0;
pub const ENEMY_BULLET_SPEED: f32 = 8.0;

pub const ENEMY_STEP_DOWN: f32 = 0.5;
pub const ENEMY_COL_SPACING: f32 = 1.2;
pub const ENEMY_ROW_SPACING: f32 = 1.0;
pub const ENEMY_START_Z: f32 = -5.0;
pub const ENEMY_START_Y: f32 = 0.5;

pub const COLLISION_DISTANCE: f32 = 0.8;
pub const GAME_OVER_Z: f32 = 7.0;

pub const BARRIER_Z: f32 = 5.5;
pub const BARRIER_HEALTH: u8 = 2;
pub const BARRIER_BLOCK_SIZE: f32 = 0.28;
pub const BARRIER_BLOCK_SPACING: f32 = 0.32;
pub const BARRIER_COLLISION_DISTANCE: f32 = 0.35;

/// Maximum number of enemy bullets on screen at once (original had 3).
pub const MAX_ENEMY_BULLETS: usize = 3;

// --- Screen Shake ---

/// Trauma-based screen shake. Add trauma (0.0–1.0) on impact events;
/// the system decays it over time and applies a squared offset to the camera.
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl ScreenShake {
    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }
}

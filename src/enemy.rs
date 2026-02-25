use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::effects::{row_color, row_emissive};
use crate::sound::{SoundKind, SoundQueue};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyDirection>()
            .init_resource::<EnemyShootTimer>()
            .init_resource::<EnemySpeed>()
            .init_resource::<CurrentWave>()
            .init_resource::<InitialEnemyCount>()
            .add_systems(OnEnter(GameState::Playing), setup_wave)
            .add_systems(
                Update,
                (enemy_movement, enemy_shoot, check_wave_cleared)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Voxel alien definitions
// ---------------------------------------------------------------------------

const VOXEL_SIZE: f32 = 0.14;
const VOXEL_STEP: f32 = 0.20;

// (col_x, height_y, depth_z) integer offsets, each multiplied by VOXEL_STEP.
// Positive z = toward player.

// --- Squid (row 0) --- compact square body + tall antennae
const SQUID_BODY: &[(i8, i8, i8)] = &[
    (-1, 0, -1), (0, 0, -1), (1, 0, -1),
    (-1, 0,  0), (0, 0,  0), (1, 0,  0),
    (-1, 0,  1), (0, 0,  1), (1, 0,  1),
];
const SQUID_DETAIL: &[(i8, i8, i8)] = &[
    // glowing eyes at the back-top corners
    (-1, 1, -1), (1, 1, -1),
    // antennae sticking high
    (-1, 2, -1), (1, 2, -1),
];

// --- Crab (rows 1–2) --- wide with side claws
const CRAB_BODY: &[(i8, i8, i8)] = &[
    // narrow back row
    (-1, 0, -1), (0, 0, -1), (1, 0, -1),
    // wide middle row
    (-2, 0, 0), (-1, 0, 0), (0, 0, 0), (1, 0, 0), (2, 0, 0),
    // narrow front row
    (-1, 0,  1), (0, 0,  1), (1, 0,  1),
    // extending claw tips
    (-3, 0, 0), (3, 0, 0),
];
const CRAB_DETAIL: &[(i8, i8, i8)] = &[
    // eyes
    (-1, 1, -1), (1, 1, -1),
    // raised carapace centre
    (0, 1, 0),
];

// --- Octopus (rows 3+) --- large oval body with a dome
const OCTOPUS_BODY: &[(i8, i8, i8)] = &[
    // wide oval body
    (-2, 0, -1), (-1, 0, -1), (0, 0, -1), (1, 0, -1), (2, 0, -1),
    (-2, 0,  0), (-1, 0,  0), (0, 0,  0), (1, 0,  0), (2, 0,  0),
    (-1, 0,  1), (0, 0,  1), (1, 0,  1),
    // dome
    (-1, 1, -1), (0, 1, -1), (1, 1, -1),
    (-1, 1,  0), (0, 1,  0), (1, 1,  0),
];
const OCTOPUS_DETAIL: &[(i8, i8, i8)] = &[
    // raised eyes on top of dome
    (-1, 2, 0), (1, 2, 0),
    // tentacle bumps at the front corners
    (-2, 0, 1), (2, 0, 1),
];

fn alien_type_for_row(row: usize) -> usize {
    match row {
        0 => 0,     // squid
        1 | 2 => 1, // crab
        _ => 2,     // octopus
    }
}

/// Contrasting eye/detail colour per row.
fn detail_color(row: usize) -> (Color, LinearRgba) {
    match row % 4 {
        0 => (Color::srgb(1.0, 0.95, 0.2), LinearRgba::new(8.0, 7.0, 1.0, 1.0)), // yellow
        1 => (Color::srgb(0.2, 0.9, 1.0),  LinearRgba::new(1.0, 7.0, 10.0, 1.0)), // cyan
        2 => (Color::srgb(1.0, 0.3, 1.0),  LinearRgba::new(8.0, 1.0, 8.0, 1.0)),  // magenta
        _ => (Color::srgb(0.9, 1.0, 0.4),  LinearRgba::new(6.0, 8.0, 1.0, 1.0)),  // lime
    }
}

// ---------------------------------------------------------------------------
// Wave spawning
// ---------------------------------------------------------------------------

pub fn spawn_enemy_wave(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &WaveConfig,
) {
    let voxel_mesh = meshes.add(Cuboid::new(VOXEL_SIZE, VOXEL_SIZE, VOXEL_SIZE));

    let grid_width = (config.cols - 1) as f32 * ENEMY_COL_SPACING;
    let start_x = -grid_width / 2.0;

    for row in 0..config.rows {
        let body_mat = materials.add(StandardMaterial {
            base_color: row_color(row),
            emissive: row_emissive(row),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            ..default()
        });
        let (dc, de) = detail_color(row);
        let detail_mat = materials.add(StandardMaterial {
            base_color: dc,
            emissive: de,
            metallic: 0.2,
            perceptual_roughness: 0.2,
            ..default()
        });

        let alien_type = alien_type_for_row(row);
        #[allow(clippy::type_complexity)]
        let (body_voxels, detail_voxels): (&[(i8, i8, i8)], &[(i8, i8, i8)]) = match alien_type {
            0 => (SQUID_BODY, SQUID_DETAIL),
            1 => (CRAB_BODY, CRAB_DETAIL),
            _ => (OCTOPUS_BODY, OCTOPUS_DETAIL),
        };

        for col in 0..config.cols {
            let x = start_x + col as f32 * ENEMY_COL_SPACING;
            let z = (ENEMY_START_Z + config.z_offset) - row as f32 * ENEMY_ROW_SPACING;

            let bm = body_mat.clone();
            let dm = detail_mat.clone();
            let vm = voxel_mesh.clone();

            commands
                .spawn((
                    Transform::from_xyz(x, ENEMY_START_Y, z),
                    Visibility::default(),
                    Enemy,
                    EnemyRow(row),
                    EnemyCol(col),
                ))
                .with_children(|parent| {
                    for &(cx, cy, cz) in body_voxels {
                        parent.spawn((
                            Mesh3d(vm.clone()),
                            MeshMaterial3d(bm.clone()),
                            Transform::from_xyz(
                                cx as f32 * VOXEL_STEP,
                                cy as f32 * VOXEL_STEP,
                                cz as f32 * VOXEL_STEP,
                            ),
                        ));
                    }
                    for &(cx, cy, cz) in detail_voxels {
                        parent.spawn((
                            Mesh3d(vm.clone()),
                            MeshMaterial3d(dm.clone()),
                            Transform::from_xyz(
                                cx as f32 * VOXEL_STEP,
                                cy as f32 * VOXEL_STEP,
                                cz as f32 * VOXEL_STEP,
                            ),
                        ));
                    }
                });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn setup_wave(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_wave: Res<CurrentWave>,
    mut enemy_speed: ResMut<EnemySpeed>,
    mut shoot_timer: ResMut<EnemyShootTimer>,
    mut enemy_dir: ResMut<EnemyDirection>,
    mut initial_count: ResMut<InitialEnemyCount>,
) {
    let config = wave_config(current_wave.wave);

    enemy_speed.base_speed = config.base_speed;
    shoot_timer.timer = Timer::from_seconds(config.shoot_interval, TimerMode::Repeating);
    enemy_dir.dir = 1.0;

    let total = (config.rows * config.cols) as u32;
    initial_count.count = total;

    spawn_enemy_wave(&mut commands, &mut meshes, &mut materials, &config);
}

fn check_wave_cleared(
    mut next_state: ResMut<NextState<GameState>>,
    enemies: Query<(), With<Enemy>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    if !enemies.is_empty() {
        return;
    }

    sound_queue.0.push(SoundKind::WaveCleared);
    // Game loops endlessly like the original — always transition to next wave.
    next_state.set(GameState::WaveTransition);
}

/// Calculates the current effective speed based on how many enemies remain.
/// Fewer enemies → faster movement, matching the original arcade behavior.
fn effective_speed(base_speed: f32, initial: u32, remaining: u32) -> f32 {
    if remaining == 0 {
        return base_speed;
    }
    // Scale speed inversely with remaining count.
    // With 55 initial and 1 remaining, this gives ~7.4x base speed.
    let ratio = initial as f32 / remaining as f32;
    base_speed * ratio.powf(0.7)
}

fn enemy_movement(
    time: Res<Time>,
    enemy_speed: Res<EnemySpeed>,
    initial_count: Res<InitialEnemyCount>,
    mut direction: ResMut<EnemyDirection>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    let remaining = query.iter().len() as u32;
    if remaining == 0 {
        return;
    }

    let speed = effective_speed(enemy_speed.base_speed, initial_count.count, remaining);

    let mut should_reverse = false;
    for transform in &query {
        let x = transform.translation.x;
        if (direction.dir > 0.0 && x >= ARENA_HALF_WIDTH - 0.5)
            || (direction.dir < 0.0 && x <= -ARENA_HALF_WIDTH + 0.5)
        {
            should_reverse = true;
            break;
        }
    }

    if should_reverse {
        direction.dir *= -1.0;
        for mut transform in &mut query {
            transform.translation.z += ENEMY_STEP_DOWN;
        }
    } else {
        let dx = direction.dir * speed * time.delta_secs();
        for mut transform in &mut query {
            transform.translation.x += dx;
        }
    }
}

/// Only the bottommost alive enemy in each column can shoot,
/// matching the original arcade behavior. Max 3 enemy bullets on screen.
#[allow(clippy::too_many_arguments)]
fn enemy_shoot(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EnemyShootTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Transform, &EnemyCol, &EnemyRow), With<Enemy>>,
    existing_bullets: Query<(), With<EnemyBullet>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    // Enforce max enemy bullets on screen.
    if existing_bullets.iter().len() >= MAX_ENEMY_BULLETS {
        return;
    }

    // Find the bottommost enemy in each column (highest row number = closest to player).
    let mut bottom_enemies: std::collections::HashMap<usize, (usize, Vec3)> =
        std::collections::HashMap::new();

    for (transform, col, row) in &query {
        let entry = bottom_enemies.entry(col.0).or_insert((row.0, transform.translation));
        // Higher row number = closer to player (larger z) = bottom of formation
        if row.0 > entry.0 {
            *entry = (row.0, transform.translation);
        }
    }

    let shooters: Vec<Vec3> = bottom_enemies.values().map(|(_, pos)| *pos).collect();
    if shooters.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..shooters.len());
    let pos = shooters[idx];

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 0.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.2, 0.2),
            emissive: LinearRgba::new(15.0, 3.0, 3.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(pos.x, pos.y, pos.z + 0.5),
        EnemyBullet,
        Velocity(Vec3::new(0.0, 0.0, ENEMY_BULLET_SPEED)),
    ));

    sound_queue.0.push(SoundKind::EnemyShoot);
}

use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::components::*;
use crate::effects::{row_color, row_emissive};
use crate::sound::{SoundKind, SoundQueue};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyDirection>()
            .init_resource::<EnemyShootTimer>()
            .init_resource::<EnemySpeed>()
            .init_resource::<EnemyCount>()
            .init_resource::<CurrentWave>()
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

/// (col_x, height_y, depth_z) integer offsets, each multiplied by VOXEL_STEP.
/// Positive z = toward player.

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

    let grid_width = (config.cols - 1) as f32 * ENEMY_SPACING;
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
        let (body_voxels, detail_voxels): (&[(i8, i8, i8)], &[(i8, i8, i8)]) = match alien_type {
            0 => (SQUID_BODY, SQUID_DETAIL),
            1 => (CRAB_BODY, CRAB_DETAIL),
            _ => (OCTOPUS_BODY, OCTOPUS_DETAIL),
        };

        for col in 0..config.cols {
            let x = start_x + col as f32 * ENEMY_SPACING;
            let z = (ENEMY_START_Z + config.z_offset) - row as f32 * ENEMY_SPACING;

            let bm = body_mat.clone();
            let dm = detail_mat.clone();
            let vm = voxel_mesh.clone();

            commands
                .spawn((
                    Transform::from_xyz(x, ENEMY_START_Y, z),
                    Visibility::default(),
                    Enemy,
                    EnemyRow(row),
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

fn setup_wave(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_wave: Res<CurrentWave>,
    mut enemy_speed: ResMut<EnemySpeed>,
    mut shoot_timer: ResMut<EnemyShootTimer>,
    mut enemy_dir: ResMut<EnemyDirection>,
    mut enemy_count: ResMut<EnemyCount>,
) {
    let config = wave_config(current_wave.wave);

    let total = config.rows * config.cols;
    enemy_count.total = total;
    enemy_count.remaining = total;

    enemy_speed.speed = config.speed;
    shoot_timer.timer = Timer::from_seconds(config.shoot_interval, TimerMode::Repeating);
    enemy_dir.dir = 1.0;

    spawn_enemy_wave(&mut commands, &mut meshes, &mut materials, &config);
}

fn check_wave_cleared(
    mut next_state: ResMut<NextState<GameState>>,
    current_wave: Res<CurrentWave>,
    enemies: Query<(), With<Enemy>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    if !enemies.is_empty() {
        return;
    }

    sound_queue.0.push(SoundKind::WaveCleared);
    if current_wave.wave >= 10 {
        next_state.set(GameState::Victory);
    } else {
        next_state.set(GameState::WaveTransition);
    }
}

fn enemy_movement(
    time: Res<Time>,
    enemy_speed: Res<EnemySpeed>,
    enemy_count: Res<EnemyCount>,
    mut direction: ResMut<EnemyDirection>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    // Mirror the original arcade speed-up: remaining enemies accelerate as the
    // fleet thins out.  With all enemies alive the multiplier is 1×; when the
    // last enemy is alive it reaches ~4×.
    let fraction_remaining = if enemy_count.total > 0 {
        enemy_count.remaining as f32 / enemy_count.total as f32
    } else {
        1.0
    };
    let actual_speed = enemy_speed.speed * (1.0 + 3.0 * (1.0 - fraction_remaining));

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
        let dx = direction.dir * actual_speed * time.delta_secs();
        for mut transform in &mut query {
            transform.translation.x += dx;
        }
    }
}

fn enemy_shoot(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EnemyShootTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Transform, With<Enemy>>,
    enemy_bullets: Query<(), With<EnemyBullet>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    // Match the original arcade's ~3 active enemy bullets cap.
    if enemy_bullets.iter().count() >= 3 {
        return;
    }

    let all_positions: Vec<Vec3> = query.iter().map(|t| t.translation).collect();
    if all_positions.is_empty() {
        return;
    }

    // Only the frontmost (highest Z = closest to player) enemy in each column
    // can shoot, replicating the original arcade "plunger" mechanic.
    let mut column_fronts: HashMap<i32, Vec3> = HashMap::new();
    for &pos in &all_positions {
        // Key by column: enemies in the same column always share the same X
        // value (they move as a rigid formation), so rounding to 1 decimal
        // place gives a stable integer key that separates adjacent columns.
        let col_key = (pos.x * 10.0).round() as i32;
        let entry = column_fronts.entry(col_key).or_insert(pos);
        if pos.z > entry.z {
            *entry = pos;
        }
    }

    let eligible: Vec<Vec3> = column_fronts.into_values().collect();
    if eligible.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let pos = eligible[rng.gen_range(0..eligible.len())];

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

use std::collections::HashMap;

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
            .init_resource::<EnemyInitialCount>()
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

fn setup_wave(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    current_wave: Res<CurrentWave>,
    mut enemy_speed: ResMut<EnemySpeed>,
    mut shoot_timer: ResMut<EnemyShootTimer>,
    mut enemy_dir: ResMut<EnemyDirection>,
    mut initial_count: ResMut<EnemyInitialCount>,
) {
    let config = wave_config(current_wave.wave);

    enemy_speed.speed = config.speed;
    shoot_timer.timer = Timer::from_seconds(config.shoot_interval, TimerMode::Repeating);
    enemy_dir.dir = 1.0;
    initial_count.count = config.rows * config.cols;

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
    initial_count: Res<EnemyInitialCount>,
    mut direction: ResMut<EnemyDirection>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    let current_count = query.iter().count();
    if current_count == 0 {
        return;
    }

    // Core Space Invaders mechanic: as aliens are destroyed the remaining ones
    // speed up.  When the last alien is alive it moves at ~6x the base speed.
    let speed_scale = if initial_count.count > 0 {
        (initial_count.count as f32 / current_count as f32).min(6.0)
    } else {
        1.0
    };
    let effective_speed = enemy_speed.speed * speed_scale;

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
        let dx = direction.dir * effective_speed * time.delta_secs();
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
    query: Query<(&Transform, &EnemyCol), With<Enemy>>,
    existing_bullets: Query<(), With<EnemyBullet>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    // Original SI: at most 3 enemy bullets on screen simultaneously
    if existing_bullets.iter().count() >= MAX_ENEMY_BULLETS {
        return;
    }

    // Original SI: only the frontmost alien in each column (highest Z = closest
    // to player) is allowed to fire. Pick one of those shooters at random.
    let mut col_front: HashMap<usize, Vec3> = HashMap::new();
    for (transform, col) in &query {
        let pos = transform.translation;
        col_front
            .entry(col.0)
            .and_modify(|best| {
                if pos.z > best.z {
                    *best = pos;
                }
            })
            .or_insert(pos);
    }

    let front_positions: Vec<Vec3> = col_front.into_values().collect();
    if front_positions.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let pos = front_positions[rng.gen_range(0..front_positions.len())];

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

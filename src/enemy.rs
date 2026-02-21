use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::effects::{row_color, row_emissive};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyDirection>()
            .init_resource::<EnemyShootTimer>()
            .init_resource::<EnemySpeed>()
            .init_resource::<CurrentWave>()
            .add_systems(OnEnter(GameState::Playing), setup_wave)
            .add_systems(
                Update,
                (enemy_movement, enemy_shoot, check_wave_cleared)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn spawn_enemy_wave(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &WaveConfig,
) {
    let mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));

    let row_materials: Vec<Handle<StandardMaterial>> = (0..config.rows)
        .map(|row| {
            materials.add(StandardMaterial {
                base_color: row_color(row),
                emissive: row_emissive(row),
                metallic: 0.5,
                perceptual_roughness: 0.4,
                ..default()
            })
        })
        .collect();

    let grid_width = (config.cols - 1) as f32 * ENEMY_SPACING;
    let start_x = -grid_width / 2.0;

    for (row, row_mat) in row_materials.iter().enumerate() {
        for col in 0..config.cols {
            let x = start_x + col as f32 * ENEMY_SPACING;
            let z = (ENEMY_START_Z + config.z_offset) - row as f32 * ENEMY_SPACING;

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(row_mat.clone()),
                Transform::from_xyz(x, ENEMY_START_Y, z),
                Enemy,
                EnemyRow(row),
            ));
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
) {
    let config = wave_config(current_wave.wave);

    enemy_speed.speed = config.speed;
    shoot_timer.timer = Timer::from_seconds(config.shoot_interval, TimerMode::Repeating);
    enemy_dir.dir = 1.0;

    spawn_enemy_wave(&mut commands, &mut meshes, &mut materials, &config);
}

fn check_wave_cleared(
    mut next_state: ResMut<NextState<GameState>>,
    current_wave: Res<CurrentWave>,
    enemies: Query<(), With<Enemy>>,
) {
    if !enemies.is_empty() {
        return;
    }

    if current_wave.wave >= 10 {
        next_state.set(GameState::Victory);
    } else {
        next_state.set(GameState::WaveTransition);
    }
}

fn enemy_movement(
    time: Res<Time>,
    enemy_speed: Res<EnemySpeed>,
    mut direction: ResMut<EnemyDirection>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
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
        let dx = direction.dir * enemy_speed.speed * time.delta_secs();
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
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    let enemies: Vec<&Transform> = query.iter().collect();
    if enemies.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..enemies.len());
    let pos = enemies[idx].translation;

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
}

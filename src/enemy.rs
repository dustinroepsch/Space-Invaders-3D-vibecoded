use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::effects::row_color;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyDirection>()
            .init_resource::<EnemyShootTimer>()
            .add_systems(Startup, spawn_enemies)
            .add_systems(
                Update,
                (enemy_movement, enemy_shoot).run_if(in_state(GameState::Playing)),
            );
    }
}

fn row_emissive(row: usize) -> LinearRgba {
    match row {
        0 => LinearRgba::new(5.0, 1.0, 1.0, 1.0),
        1 => LinearRgba::new(5.0, 3.0, 0.5, 1.0),
        2 => LinearRgba::new(1.0, 5.0, 1.5, 1.0),
        _ => LinearRgba::new(3.5, 1.0, 5.0, 1.0),
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));

    // Create a material per row
    let row_materials: Vec<Handle<StandardMaterial>> = (0..ENEMY_ROWS)
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

    let grid_width = (ENEMY_COLS - 1) as f32 * ENEMY_SPACING;
    let start_x = -grid_width / 2.0;

    for (row, row_mat) in row_materials.iter().enumerate() {
        for col in 0..ENEMY_COLS {
            let x = start_x + col as f32 * ENEMY_SPACING;
            let z = ENEMY_START_Z - row as f32 * ENEMY_SPACING;

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

fn enemy_movement(
    time: Res<Time>,
    mut direction: ResMut<EnemyDirection>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    // Check if any enemy has hit the boundary
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
        let dx = direction.dir * ENEMY_SPEED * time.delta_secs();
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

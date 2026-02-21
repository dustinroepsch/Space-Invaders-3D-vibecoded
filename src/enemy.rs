use bevy::prelude::*;
use rand::Rng;

use crate::components::*;

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

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));
    let material = materials.add(Color::srgb(0.2, 1.0, 0.3));

    let grid_width = (ENEMY_COLS - 1) as f32 * ENEMY_SPACING;
    let start_x = -grid_width / 2.0;

    for row in 0..ENEMY_ROWS {
        for col in 0..ENEMY_COLS {
            let x = start_x + col as f32 * ENEMY_SPACING;
            let z = ENEMY_START_Z - row as f32 * ENEMY_SPACING;

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(x, ENEMY_START_Y, z),
                Enemy,
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
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.2, 0.2))),
        Transform::from_xyz(pos.x, pos.y, pos.z + 0.5),
        EnemyBullet,
        Velocity(Vec3::new(0.0, 0.0, ENEMY_BULLET_SPEED)),
    ));
}

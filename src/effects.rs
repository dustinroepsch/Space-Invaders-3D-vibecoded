use bevy::prelude::*;
use rand::Rng;

use crate::components::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TrailSpawnTimer {
            timer: Timer::from_seconds(0.03, TimerMode::Repeating),
        })
        .add_systems(
            Update,
            (
                spawn_bullet_trails,
                shrink_over_lifetime,
                despawn_expired_lifetimes,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource)]
struct TrailSpawnTimer {
    timer: Timer,
}

/// Row colors matching enemy.rs: row 0=red, 1=orange, 2=green, 3=purple
pub fn row_color(row: usize) -> Color {
    match row {
        0 => Color::srgb(1.0, 0.2, 0.2),
        1 => Color::srgb(1.0, 0.6, 0.1),
        2 => Color::srgb(0.2, 1.0, 0.3),
        _ => Color::srgb(0.7, 0.2, 1.0),
    }
}

pub fn spawn_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    row: usize,
) {
    let mut rng = rand::thread_rng();
    let color = row_color(row);
    let emissive = row_emissive(row);
    let mesh = meshes.add(Cuboid::new(0.12, 0.12, 0.12));
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive,
        ..default()
    });

    for _ in 0..12 {
        let vx = rng.gen_range(-4.0..4.0);
        let vy = rng.gen_range(1.0..6.0);
        let vz = rng.gen_range(-4.0..4.0);
        let lifetime = rng.gen_range(0.3..0.7);

        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(position),
            Velocity(Vec3::new(vx, vy, vz)),
            ExplosionParticle,
            Lifetime {
                timer: Timer::from_seconds(lifetime, TimerMode::Once),
            },
        ));
    }
}

fn row_emissive(row: usize) -> LinearRgba {
    match row {
        0 => LinearRgba::new(10.0, 2.0, 2.0, 1.0),
        1 => LinearRgba::new(10.0, 6.0, 1.0, 1.0),
        2 => LinearRgba::new(2.0, 10.0, 3.0, 1.0),
        _ => LinearRgba::new(7.0, 2.0, 10.0, 1.0),
    }
}

fn spawn_bullet_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TrailSpawnTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_bullets: Query<&Transform, With<Bullet>>,
    enemy_bullets: Query<&Transform, With<EnemyBullet>>,
) {
    timer.timer.tick(time.delta());
    if !timer.timer.just_finished() {
        return;
    }

    let trail_mesh = meshes.add(Sphere::new(0.06));

    for bt in &player_bullets {
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 0.3, 0.6),
            emissive: LinearRgba::new(8.0, 8.0, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        commands.spawn((
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_translation(bt.translation + Vec3::new(0.0, 0.0, 0.2)),
            TrailParticle,
            Lifetime {
                timer: Timer::from_seconds(0.15, TimerMode::Once),
            },
        ));
    }

    for bt in &enemy_bullets {
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.3, 0.3, 0.6),
            emissive: LinearRgba::new(8.0, 2.0, 2.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        commands.spawn((
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_translation(bt.translation - Vec3::new(0.0, 0.0, 0.2)),
            TrailParticle,
            Lifetime {
                timer: Timer::from_seconds(0.15, TimerMode::Once),
            },
        ));
    }
}

fn shrink_over_lifetime(mut query: Query<(&mut Transform, &Lifetime)>) {
    for (mut transform, lifetime) in &mut query {
        let remaining = lifetime.timer.fraction_remaining();
        let scale = remaining.max(0.01);
        transform.scale = Vec3::splat(scale);
    }
}

fn despawn_expired_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

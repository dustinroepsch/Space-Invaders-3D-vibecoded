use bevy::prelude::*;

use crate::components::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (bullet_movement, bullet_cleanup).run_if(in_state(GameState::Playing)),
        );
    }
}

fn bullet_movement(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

type BulletQuery<'w> = (Entity, &'w Transform);
type BulletFilter = Or<(With<Bullet>, With<EnemyBullet>)>;

fn bullet_cleanup(mut commands: Commands, query: Query<BulletQuery, BulletFilter>) {
    let bounds = -ARENA_HEIGHT / 2.0..=ARENA_HEIGHT / 2.0;
    for (entity, transform) in &query {
        if !bounds.contains(&transform.translation.z) {
            commands.entity(entity).despawn();
        }
    }
}

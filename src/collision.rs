use bevy::prelude::*;

use crate::barrier::barrier_material;
use crate::components::*;
use crate::effects::spawn_explosion;
use crate::sound::{SoundKind, SoundQueue};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_player_invincible,
                bullet_enemy_collision,
                enemy_bullet_player_collision,
                enemy_reaches_bottom,
                bullet_barrier_collision,
                enemy_bullet_barrier_collision,
                bullet_mystery_ship_collision,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    enemies: Query<(Entity, &Transform, &EnemyRow), With<Enemy>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    for (bullet_entity, bullet_transform) in &bullets {
        for (enemy_entity, enemy_transform, enemy_row) in &enemies {
            let distance = bullet_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < COLLISION_DISTANCE {
                let pos = enemy_transform.translation;
                let row = enemy_row.0;
                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
                // Original SI scoring: closest row (row 0) = 10 pts,
                // middle rows = 20 pts, furthest rows = 30 pts.
                let points = match row {
                    0 => 10,
                    1 | 2 => 20,
                    _ => 30,
                };
                score.value += points;
                spawn_explosion(&mut commands, &mut meshes, &mut materials, pos, row);
                sound_queue.0.push(SoundKind::EnemyDie);
                break;
            }
        }
    }
}

fn tick_player_invincible(time: Res<Time>, mut invincible: ResMut<PlayerInvincible>) {
    if let Some(timer) = &mut invincible.timer {
        timer.tick(time.delta());
        if timer.finished() {
            invincible.timer = None;
        }
    }
}

fn enemy_bullet_player_collision(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemy_bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
    players: Query<&Transform, With<Player>>,
    mut lives: ResMut<PlayerLives>,
    mut invincible: ResMut<PlayerInvincible>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    // Skip collision detection during the post-hit invincibility window
    if invincible.timer.is_some() {
        return;
    }

    let Ok(player_transform) = players.single() else {
        return;
    };

    let mut hit_entity: Option<Entity> = None;
    for (bullet_entity, bullet_transform) in &enemy_bullets {
        let distance = bullet_transform
            .translation
            .distance(player_transform.translation);
        if distance < COLLISION_DISTANCE {
            hit_entity = Some(bullet_entity);
            break;
        }
    }

    let Some(hit) = hit_entity else { return };

    commands.entity(hit).despawn();
    sound_queue.0.push(SoundKind::PlayerDie);
    spawn_explosion(
        &mut commands,
        &mut meshes,
        &mut materials,
        player_transform.translation,
        0,
    );

    lives.lives = lives.lives.saturating_sub(1);
    if lives.lives == 0 {
        next_state.set(GameState::GameOver);
    } else {
        // Clear remaining enemy bullets and grant a 2-second invincibility window
        for (e, _) in &enemy_bullets {
            if e != hit {
                commands.entity(e).despawn();
            }
        }
        invincible.timer = Some(Timer::from_seconds(2.0, TimerMode::Once));
    }
}

fn enemy_reaches_bottom(
    mut next_state: ResMut<NextState<GameState>>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for transform in &enemies {
        if transform.translation.z >= GAME_OVER_Z {
            next_state.set(GameState::GameOver);
            return;
        }
    }
}

fn bullet_barrier_collision(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    mut barriers: Query<(Entity, &Transform, &mut Barrier, &MeshMaterial3d<StandardMaterial>)>,
) {
    'bullet: for (bullet_entity, bullet_transform) in &bullets {
        for (barrier_entity, barrier_transform, mut barrier, mat_handle) in &mut barriers {
            let dist = bullet_transform
                .translation
                .distance(barrier_transform.translation);
            if dist < BARRIER_COLLISION_DISTANCE {
                commands.entity(bullet_entity).despawn();
                if barrier.health <= 1 {
                    commands.entity(barrier_entity).despawn();
                } else {
                    barrier.health -= 1;
                    if let Some(mat) = materials.get_mut(&mat_handle.0) {
                        *mat = barrier_material(barrier.health);
                    }
                }
                continue 'bullet;
            }
        }
    }
}

fn enemy_bullet_barrier_collision(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemy_bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
    mut barriers: Query<(Entity, &Transform, &mut Barrier, &MeshMaterial3d<StandardMaterial>)>,
) {
    'bullet: for (bullet_entity, bullet_transform) in &enemy_bullets {
        for (barrier_entity, barrier_transform, mut barrier, mat_handle) in &mut barriers {
            let dist = bullet_transform
                .translation
                .distance(barrier_transform.translation);
            if dist < BARRIER_COLLISION_DISTANCE {
                commands.entity(bullet_entity).despawn();
                if barrier.health <= 1 {
                    commands.entity(barrier_entity).despawn();
                } else {
                    barrier.health -= 1;
                    if let Some(mat) = materials.get_mut(&mat_handle.0) {
                        *mat = barrier_material(barrier.health);
                    }
                }
                continue 'bullet;
            }
        }
    }
}

fn bullet_mystery_ship_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut events: MessageWriter<MysteryShipKilledEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    ships: Query<(Entity, &Transform, &MysteryShipPoints), With<MysteryShip>>,
    mut sound_queue: ResMut<SoundQueue>,
) {
    for (bullet_entity, bullet_transform) in &bullets {
        for (ship_entity, ship_transform, points) in &ships {
            let dist = bullet_transform
                .translation
                .distance(ship_transform.translation);
            if dist < MYSTERY_SHIP_COLLISION_DISTANCE {
                let pos = ship_transform.translation;
                score.value += points.0;
                events.write(MysteryShipKilledEvent {
                    points: points.0,
                    world_pos: pos,
                });
                commands.entity(bullet_entity).despawn();
                commands.entity(ship_entity).despawn();
                spawn_explosion(&mut commands, &mut meshes, &mut materials, pos, 0);
                spawn_explosion(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    pos + Vec3::new(0.0, 0.35, 0.0),
                    2,
                );
                sound_queue.0.push(SoundKind::MysteryShipDie);
                break;
            }
        }
    }
}

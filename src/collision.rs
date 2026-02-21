use bevy::prelude::*;

use crate::components::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                bullet_enemy_collision,
                enemy_bullet_player_collision,
                enemy_reaches_bottom,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet_entity, bullet_transform) in &bullets {
        for (enemy_entity, enemy_transform) in &enemies {
            let distance = bullet_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < COLLISION_DISTANCE {
                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
                score.value += 10;
                break;
            }
        }
    }
}

fn enemy_bullet_player_collision(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    enemy_bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
    players: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = players.single() else {
        return;
    };

    for (bullet_entity, bullet_transform) in &enemy_bullets {
        let distance = bullet_transform
            .translation
            .distance(player_transform.translation);
        if distance < COLLISION_DISTANCE {
            commands.entity(bullet_entity).despawn();
            next_state.set(GameState::GameOver);
            return;
        }
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

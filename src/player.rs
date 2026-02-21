use bevy::prelude::*;

use crate::components::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShootCooldown>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (player_movement, player_shoot).run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 0.5, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.4, 1.0))),
        Transform::from_xyz(0.0, PLAYER_Y, PLAYER_Z),
        Player,
    ));
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    transform.translation.x += direction * PLAYER_SPEED * time.delta_secs();
    transform.translation.x = transform
        .translation
        .x
        .clamp(-ARENA_HALF_WIDTH, ARENA_HALF_WIDTH);
}

fn player_shoot(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cooldown: ResMut<ShootCooldown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Transform, With<Player>>,
) {
    cooldown.timer.tick(time.delta());

    if keyboard.pressed(KeyCode::Space) && cooldown.timer.is_finished() {
        let Ok(player_transform) = query.single() else {
            return;
        };

        let pos = player_transform.translation;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.15, 0.15, 0.4))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 0.2))),
            Transform::from_xyz(pos.x, pos.y + 0.3, pos.z - 0.5),
            Bullet,
            Velocity(Vec3::new(0.0, 0.0, -BULLET_SPEED)),
        ));

        cooldown.timer.reset();
    }
}

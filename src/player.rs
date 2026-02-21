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
    // Body material: metallic blue with emissive glow
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.3, 0.9),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        emissive: LinearRgba::new(0.3, 0.5, 2.0, 1.0),
        ..default()
    });

    // Wing material: darker blue, slightly transparent
    let wing_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.15, 0.6, 0.85),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Cockpit material: bright cyan emissive
    let cockpit_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.9, 1.0),
        emissive: LinearRgba::new(2.0, 8.0, 10.0, 1.0),
        ..default()
    });

    let body_mesh = meshes.add(Cuboid::new(0.6, 0.4, 1.0));
    let wing_mesh = meshes.add(Cuboid::new(0.5, 0.1, 0.6));
    let cockpit_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.3));

    commands
        .spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_mat),
            Transform::from_xyz(0.0, PLAYER_Y, PLAYER_Z),
            Player,
        ))
        .with_children(|parent| {
            // Left wing
            parent.spawn((
                Mesh3d(wing_mesh.clone()),
                MeshMaterial3d(wing_mat.clone()),
                Transform::from_xyz(-0.5, -0.05, 0.1),
            ));
            // Right wing
            parent.spawn((
                Mesh3d(wing_mesh),
                MeshMaterial3d(wing_mat),
                Transform::from_xyz(0.5, -0.05, 0.1),
            ));
            // Cockpit
            parent.spawn((
                Mesh3d(cockpit_mesh),
                MeshMaterial3d(cockpit_mat),
                Transform::from_xyz(0.0, 0.25, -0.2),
            ));
        });
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
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.2),
                emissive: LinearRgba::new(15.0, 15.0, 3.0, 1.0),
                ..default()
            })),
            Transform::from_xyz(pos.x, pos.y + 0.3, pos.z - 0.5),
            Bullet,
            Velocity(Vec3::new(0.0, 0.0, -BULLET_SPEED)),
        ));

        cooldown.timer.reset();
    }
}

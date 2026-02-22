use bevy::prelude::*;

use crate::components::*;
use crate::sound::{SoundKind, SoundQueue};

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

pub fn spawn_player_ship(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.3, 0.9),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        emissive: LinearRgba::new(0.3, 0.5, 2.0, 1.0),
        ..default()
    });

    let wing_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.15, 0.6, 0.85),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

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
            parent.spawn((
                Mesh3d(wing_mesh.clone()),
                MeshMaterial3d(wing_mat.clone()),
                Transform::from_xyz(-0.5, -0.05, 0.1),
            ));
            parent.spawn((
                Mesh3d(wing_mesh),
                MeshMaterial3d(wing_mat),
                Transform::from_xyz(0.5, -0.05, 0.1),
            ));
            parent.spawn((
                Mesh3d(cockpit_mesh),
                MeshMaterial3d(cockpit_mat),
                Transform::from_xyz(0.0, 0.25, -0.2),
            ));
        });
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_player_ship(&mut commands, &mut meshes, &mut materials);
}

/// Reads the `window.touchInput` object set by the mobile HTML buttons.
/// Returns (left, right, fire). Always false on non-WASM targets.
fn touch_input() -> (bool, bool, bool) {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Reflect;
        use wasm_bindgen::JsValue;
        let window = web_sys::window().unwrap();
        let obj = Reflect::get(&window, &JsValue::from_str("touchInput")).unwrap_or(JsValue::NULL);
        let get = |key: &str| {
            Reflect::get(&obj, &JsValue::from_str(key))
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        };
        (get("left"), get("right"), get("fire"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        (false, false, false)
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let (touch_left, touch_right, _) = touch_input();

    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) || touch_left {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) || touch_right {
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
    mut sound_queue: ResMut<SoundQueue>,
) {
    cooldown.timer.tick(time.delta());

    let (_, _, touch_fire) = touch_input();
    if (keyboard.pressed(KeyCode::Space) || touch_fire) && cooldown.timer.is_finished() {
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

        sound_queue.0.push(SoundKind::PlayerShoot);
        cooldown.timer.reset();
    }
}

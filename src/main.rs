mod barrier;
mod bullet;
mod collision;
mod components;
mod crt;
mod effects;
mod enemy;
mod mystery;
mod player;
mod scoreboard;
mod sound;

use bevy::prelude::*;
use rand::Rng;

use barrier::BarrierPlugin;
use bullet::BulletPlugin;
use collision::CollisionPlugin;
use components::{GameState, Score};
use crt::{CrtPlugin, CrtSettings};
use effects::EffectsPlugin;
use enemy::EnemyPlugin;
use mystery::MysteryPlugin;
use player::PlayerPlugin;
use scoreboard::ScoreboardPlugin;
use sound::SoundPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Invaders 3D".into(),
                // On web: fill the browser tab. On desktop: ignored.
                fit_canvas_to_parent: true,
                // Prevent the browser from consuming Space/Arrow keys etc.
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<Score>()
        .add_plugins((
            PlayerPlugin,
            EnemyPlugin,
            BulletPlugin,
            CollisionPlugin,
            ScoreboardPlugin,
            EffectsPlugin,
            BarrierPlugin,
            MysteryPlugin,
            CrtPlugin,
            SoundPlugin,
        ))
        .add_systems(Startup, (setup_scene, spawn_starfield))
        .add_systems(Update, update_ui_scale)
        .run();
}

/// Dynamically adjust Bevy's global UI scale so text and layout stay readable
/// on small mobile screens. We target a reference width of 900 logical pixels;
/// anything narrower is scaled down proportionally (capped at 0.3x minimum so
/// nothing becomes illegible, and 1.0x maximum so desktop layouts are unchanged).
fn update_ui_scale(windows: Query<&Window>, mut ui_scale: ResMut<UiScale>) {
    let Ok(window) = windows.single() else {
        return;
    };
    const REF_WIDTH: f32 = 900.0;
    let scale = (window.width() / REF_WIDTH).clamp(0.3, 1.0);
    if (ui_scale.0 - scale).abs() > 0.001 {
        ui_scale.0 = scale;
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera: lower and slightly further back for a dramatic angle
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 16.0).looking_at(Vec3::new(0.0, 0.0, 1.0), Vec3::Y),
        CrtSettings,
    ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light so nothing is pitch black
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    // Dark reflective ground plane
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(60.0, 60.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.05, 0.05, 0.1),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

fn spawn_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let white_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::new(10.0, 10.0, 10.0, 1.0),
        unlit: true,
        ..default()
    });

    let blue_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.7, 1.0),
        emissive: LinearRgba::new(6.0, 7.0, 10.0, 1.0),
        unlit: true,
        ..default()
    });

    let sphere_mesh = meshes.add(Sphere::new(1.0));

    for _ in 0..250 {
        let radius = rng.gen_range(40.0..80.0);
        // Spherical coordinates, upper hemisphere only (y > 0)
        let theta = rng.gen_range(0.0..std::f32::consts::TAU);
        let phi = rng.gen_range(0.1..std::f32::consts::FRAC_PI_2);

        let x = radius * phi.sin() * theta.cos();
        let y = radius * phi.cos();
        let z = radius * phi.sin() * theta.sin();

        let scale = rng.gen_range(0.05..0.2);
        let mat = if rng.gen_bool(0.7) {
            white_mat.clone()
        } else {
            blue_mat.clone()
        };

        commands.spawn((
            Mesh3d(sphere_mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(scale)),
        ));
    }
}

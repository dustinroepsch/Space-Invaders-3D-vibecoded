mod bullet;
mod collision;
mod components;
mod enemy;
mod player;
mod scoreboard;

use bevy::prelude::*;

use bullet::BulletPlugin;
use collision::CollisionPlugin;
use components::{GameState, Score};
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use scoreboard::ScoreboardPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_resource::<Score>()
        .add_plugins((
            PlayerPlugin,
            EnemyPlugin,
            BulletPlugin,
            CollisionPlugin,
            ScoreboardPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    // Camera: positioned above and behind the play field, looking down at the arena
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 18.0, 14.0).looking_at(Vec3::new(0.0, 0.0, 2.0), Vec3::Y),
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
}

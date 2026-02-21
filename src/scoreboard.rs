use bevy::prelude::*;

use crate::components::*;

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scoreboard)
            .add_systems(Update, update_scoreboard)
            .add_systems(OnEnter(GameState::GameOver), show_game_over)
            .add_systems(
                Update,
                restart_game.run_if(in_state(GameState::GameOver)),
            );
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverUI;

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.85, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));
}

fn update_scoreboard(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Score: {}", score.value);
    }
}

fn show_game_over(mut commands: Commands) {
    // Full-screen centered overlay container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GameOverUI,
        ))
        .with_children(|parent| {
            // GAME OVER text
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));
            // Restart prompt
            parent.spawn((
                Text::new("Press SPACE to restart"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

#[allow(clippy::too_many_arguments)]
fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut enemy_dir: ResMut<EnemyDirection>,
    game_over_ui: Query<Entity, With<GameOverUI>>,
    enemies: Query<Entity, With<Enemy>>,
    bullets: Query<Entity, With<Bullet>>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    player: Query<Entity, With<Player>>,
    explosions: Query<Entity, With<ExplosionParticle>>,
    trails: Query<Entity, With<TrailParticle>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // Despawn game over UI
    for entity in &game_over_ui {
        commands.entity(entity).despawn();
    }

    // Despawn all game entities
    for entity in &enemies {
        commands.entity(entity).despawn();
    }
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
    for entity in &enemy_bullets {
        commands.entity(entity).despawn();
    }
    for entity in &player {
        commands.entity(entity).despawn();
    }
    for entity in &explosions {
        commands.entity(entity).despawn();
    }
    for entity in &trails {
        commands.entity(entity).despawn();
    }

    // Reset score and direction
    score.value = 0;
    enemy_dir.dir = 1.0;

    // Respawn player
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

    // Respawn enemies
    let enemy_mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));
    let row_materials: Vec<Handle<StandardMaterial>> = (0..ENEMY_ROWS)
        .map(|row| {
            let color = crate::effects::row_color(row);
            let emissive = match row {
                0 => LinearRgba::new(5.0, 1.0, 1.0, 1.0),
                1 => LinearRgba::new(5.0, 3.0, 0.5, 1.0),
                2 => LinearRgba::new(1.0, 5.0, 1.5, 1.0),
                _ => LinearRgba::new(3.5, 1.0, 5.0, 1.0),
            };
            materials.add(StandardMaterial {
                base_color: color,
                emissive,
                metallic: 0.5,
                perceptual_roughness: 0.4,
                ..default()
            })
        })
        .collect();

    let grid_width = (ENEMY_COLS - 1) as f32 * ENEMY_SPACING;
    let start_x = -grid_width / 2.0;

    for (row, row_mat) in row_materials.iter().enumerate() {
        for col in 0..ENEMY_COLS {
            let x = start_x + col as f32 * ENEMY_SPACING;
            let z = ENEMY_START_Z - row as f32 * ENEMY_SPACING;

            commands.spawn((
                Mesh3d(enemy_mesh.clone()),
                MeshMaterial3d(row_mat.clone()),
                Transform::from_xyz(x, ENEMY_START_Y, z),
                Enemy,
                EnemyRow(row),
            ));
        }
    }

    // Transition back to playing
    next_state.set(GameState::Playing);
}

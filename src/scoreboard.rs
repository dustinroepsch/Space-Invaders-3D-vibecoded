use bevy::prelude::*;

use crate::components::*;
use crate::player::{spawn_player_ship, touch_input};

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scoreboard)
            .add_systems(Update, update_scoreboard)
            // Game Over
            .add_systems(OnEnter(GameState::GameOver), show_game_over)
            .add_systems(Update, restart_game.run_if(in_state(GameState::GameOver)))
            // Wave Transition
            .add_systems(OnEnter(GameState::WaveTransition), show_wave_transition)
            .add_systems(
                Update,
                wave_transition_tick.run_if(in_state(GameState::WaveTransition)),
            )
            .add_systems(OnExit(GameState::WaveTransition), cleanup_wave_transition);
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverUI;

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((
        Text::new("Lives: 3 | Wave 1 | Score: 0"),
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

fn update_scoreboard(
    score: Res<Score>,
    high_score: Res<HighScore>,
    current_wave: Res<CurrentWave>,
    lives: Res<Lives>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if !score.is_changed() && !current_wave.is_changed() && !lives.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!(
            "Lives: {} | Wave {} | Score: {} | Hi: {}",
            lives.count, current_wave.wave, score.value, high_score.value
        );
    }
}

fn show_game_over(mut commands: Commands, score: Res<Score>) {
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
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", score.value)),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Press SPACE or tap FIRE to restart"),
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

type GameEntityFilter = Or<(
    With<Enemy>,
    With<Bullet>,
    With<EnemyBullet>,
    With<Player>,
    With<ExplosionParticle>,
    With<TrailParticle>,
    With<ScorePopup>,
    With<Barrier>,
    With<MysteryShip>,
)>;

#[allow(clippy::too_many_arguments)]
fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_wave: ResMut<CurrentWave>,
    mut lives: ResMut<Lives>,
    game_over_ui: Query<Entity, With<GameOverUI>>,
    game_entities: Query<Entity, GameEntityFilter>,
) {
    let (_, _, touch_fire) = touch_input();
    if !keyboard.just_pressed(KeyCode::Space) && !touch_fire {
        return;
    }

    // Despawn game over UI
    for entity in &game_over_ui {
        commands.entity(entity).despawn();
    }

    // Despawn all game entities
    for entity in &game_entities {
        commands.entity(entity).despawn();
    }

    // Remove respawn timer if present
    commands.remove_resource::<PlayerRespawnTimer>();

    // Reset score, wave, and lives
    score.value = 0;
    current_wave.wave = 1;
    lives.count = 3;

    spawn_player_ship(&mut commands, &mut meshes, &mut materials);

    // Transition back to playing — OnEnter(Playing) will spawn enemies and barriers
    next_state.set(GameState::Playing);
}

// --- Wave Transition ---

fn show_wave_transition(mut commands: Commands, current_wave: Res<CurrentWave>) {
    let next_wave = current_wave.wave + 1;

    commands.insert_resource(WaveTransitionTimer {
        timer: Timer::from_seconds(2.0, TimerMode::Once),
    });

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            WaveTransitionUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Wave {}", next_wave)),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.9, 1.0)),
            ));
            parent.spawn((
                Text::new("Get Ready!"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.8, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            ));
        });
}

fn wave_transition_tick(
    time: Res<Time>,
    mut timer: ResMut<WaveTransitionTimer>,
    mut current_wave: ResMut<CurrentWave>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.timer.tick(time.delta());

    if timer.timer.just_finished() {
        current_wave.wave += 1;
        next_state.set(GameState::Playing);
    }
}

fn cleanup_wave_transition(
    mut commands: Commands,
    transition_ui: Query<Entity, With<WaveTransitionUI>>,
    bullets: Query<Entity, With<Bullet>>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    explosions: Query<Entity, With<ExplosionParticle>>,
    trails: Query<Entity, With<TrailParticle>>,
    barriers: Query<Entity, With<Barrier>>,
) {
    for entity in &transition_ui {
        commands.entity(entity).despawn();
    }
    for entity in &bullets {
        commands.entity(entity).despawn();
    }
    for entity in &enemy_bullets {
        commands.entity(entity).despawn();
    }
    for entity in &explosions {
        commands.entity(entity).despawn();
    }
    for entity in &trails {
        commands.entity(entity).despawn();
    }
    // Despawn old barriers so they get freshly spawned in OnEnter(Playing)
    for entity in &barriers {
        commands.entity(entity).despawn();
    }
}

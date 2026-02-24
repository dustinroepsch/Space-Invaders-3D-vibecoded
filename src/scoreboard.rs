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
            .add_systems(OnExit(GameState::WaveTransition), cleanup_wave_transition)
            // Victory
            .add_systems(OnEnter(GameState::Victory), show_victory)
            .add_systems(
                Update,
                restart_from_victory.run_if(in_state(GameState::Victory)),
            );
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverUI;

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((
        Text::new("Wave 1 | Score: 0"),
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
    current_wave: Res<CurrentWave>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if !score.is_changed() && !current_wave.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Wave {} | Score: {}", current_wave.wave, score.value);
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

#[allow(clippy::too_many_arguments)]
fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_wave: ResMut<CurrentWave>,
    game_over_ui: Query<Entity, With<GameOverUI>>,
    enemies: Query<Entity, With<Enemy>>,
    bullets: Query<Entity, With<Bullet>>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    player: Query<Entity, With<Player>>,
    explosions: Query<Entity, With<ExplosionParticle>>,
    trails: Query<Entity, With<TrailParticle>>,
    score_popups: Query<Entity, With<ScorePopup>>,
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
    for entity in &score_popups {
        commands.entity(entity).despawn();
    }

    // Reset score, wave, and respawn player
    score.value = 0;
    current_wave.wave = 1;

    spawn_player_ship(&mut commands, &mut meshes, &mut materials);

    // Transition back to playing — OnEnter(Playing) will spawn enemies
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
}

// --- Victory ---

fn show_victory(mut commands: Commands, score: Res<Score>) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.8)),
            VictoryUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("YOU WIN!"),
                TextFont {
                    font_size: 90.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", score.value)),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("Press SPACE or tap FIRE to play again"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::top(Val::Px(25.0)),
                    ..default()
                },
            ));
        });
}

#[allow(clippy::too_many_arguments)]
fn restart_from_victory(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<Score>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_wave: ResMut<CurrentWave>,
    victory_ui: Query<Entity, With<VictoryUI>>,
    enemies: Query<Entity, With<Enemy>>,
    bullets: Query<Entity, With<Bullet>>,
    enemy_bullets: Query<Entity, With<EnemyBullet>>,
    player: Query<Entity, With<Player>>,
    explosions: Query<Entity, With<ExplosionParticle>>,
    trails: Query<Entity, With<TrailParticle>>,
    score_popups: Query<Entity, With<ScorePopup>>,
) {
    let (_, _, touch_fire) = touch_input();
    if !keyboard.just_pressed(KeyCode::Space) && !touch_fire {
        return;
    }

    // Despawn victory UI
    for entity in &victory_ui {
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
    for entity in &score_popups {
        commands.entity(entity).despawn();
    }

    // Reset everything
    score.value = 0;
    current_wave.wave = 1;

    spawn_player_ship(&mut commands, &mut meshes, &mut materials);

    next_state.set(GameState::Playing);
}

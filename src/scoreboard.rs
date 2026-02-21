use bevy::prelude::*;

use crate::components::*;

pub struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scoreboard)
            .add_systems(Update, update_scoreboard)
            .add_systems(OnEnter(GameState::GameOver), show_game_over);
    }
}

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverText;

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
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
    commands.spawn((
        Text::new("GAME OVER"),
        TextFont {
            font_size: 80.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(30.0),
            ..default()
        },
        GameOverText,
    ));
}

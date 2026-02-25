use bevy::audio::PlaybackSettings;
use bevy::prelude::*;
use std::time::Duration;

use crate::components::*;
use crate::sound::{SfxAsset, WaveKind};

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        // init_resource is immediate (not deferred), so MarchState is available
        // when the initial OnEnter(Playing) fires before Startup commands flush.
        app.init_resource::<MarchState>()
            .add_systems(Startup, setup_march_sounds)
            .add_systems(OnEnter(GameState::Playing), reset_march)
            .add_systems(
                Update,
                march_beat.run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// March state — the iconic 4-note heartbeat
// ---------------------------------------------------------------------------

/// Tracks current beat position and timing for the alien march.
#[derive(Resource)]
struct MarchState {
    /// Current beat in the 4-note cycle (0–3).
    step: usize,
    /// Timer controlling interval between beats.
    timer: Timer,
}

impl Default for MarchState {
    fn default() -> Self {
        Self {
            step: 0,
            timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        }
    }
}

/// Pre-baked handles for the 4 march notes.
#[derive(Resource)]
struct MarchSounds {
    notes: [Handle<SfxAsset>; 4],
}

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup_march_sounds(mut commands: Commands, mut assets: ResMut<Assets<SfxAsset>>) {
    // Four descending bass square-wave thumps — the Space Invaders heartbeat.
    // Each note sweeps down slightly for a percussive thud.
    let notes = [
        // Beat 0 — highest, sets the rhythm
        assets.add(SfxAsset {
            start_freq: 150.0,
            end_freq: 130.0,
            duration_secs: 0.10,
            waveform: WaveKind::Square,
            volume: 0.25,
            noise_mix: 0.0,
        }),
        // Beat 1 — step down
        assets.add(SfxAsset {
            start_freq: 130.0,
            end_freq: 112.0,
            duration_secs: 0.10,
            waveform: WaveKind::Square,
            volume: 0.25,
            noise_mix: 0.0,
        }),
        // Beat 2 — step down
        assets.add(SfxAsset {
            start_freq: 112.0,
            end_freq: 97.0,
            duration_secs: 0.10,
            waveform: WaveKind::Square,
            volume: 0.25,
            noise_mix: 0.0,
        }),
        // Beat 3 — lowest, menacing
        assets.add(SfxAsset {
            start_freq: 97.0,
            end_freq: 84.0,
            duration_secs: 0.10,
            waveform: WaveKind::Square,
            volume: 0.25,
            noise_mix: 0.0,
        }),
    ];

    commands.insert_resource(MarchSounds { notes });
}

/// Reset march to beat 0 with slow tempo at the start of each wave.
fn reset_march(mut march: ResMut<MarchState>) {
    march.step = 0;
    march.timer = Timer::from_seconds(0.8, TimerMode::Repeating);
}

// ---------------------------------------------------------------------------
// The beat system — tempo scales with remaining enemies
// ---------------------------------------------------------------------------

/// Plays the next march note on each tick. The interval between beats
/// shrinks as enemies are killed, creating the iconic acceleration.
fn march_beat(
    time: Res<Time>,
    mut march: ResMut<MarchState>,
    march_sounds: Option<Res<MarchSounds>>,
    enemies: Query<(), With<Enemy>>,
    initial_count: Res<InitialEnemyCount>,
    mut commands: Commands,
) {
    let Some(sounds) = march_sounds else {
        return;
    };

    let remaining = enemies.iter().len() as f32;
    let initial = initial_count.count.max(1) as f32;

    // Don't play march if no enemies (wave just cleared).
    if remaining == 0.0 {
        return;
    }

    // Scale tempo: slow with full formation, frantic with last few aliens.
    // 55 enemies → 0.8s, 1 enemy → 0.12s
    let ratio = remaining / initial;
    let interval = 0.12 + 0.68 * ratio;
    march
        .timer
        .set_duration(Duration::from_secs_f32(interval));

    march.timer.tick(time.delta());

    if march.timer.just_finished() {
        let handle = sounds.notes[march.step].clone();
        commands.spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN));
        march.step = (march.step + 1) % 4;
    }
}

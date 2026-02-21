use bevy::audio::{AddAudioSource, Decodable, PlaybackSettings, Source};
use bevy::prelude::*;
use core::time::Duration;

// ---------------------------------------------------------------------------
// Sound trigger queue — any system can push to this, sound plugin drains it
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum SoundKind {
    PlayerShoot,
    EnemyShoot,
    EnemyDie,
    PlayerDie,
    MysteryShipDie,
    WaveCleared,
}

#[derive(Resource, Default)]
pub struct SoundQueue(pub Vec<SoundKind>);

// ---------------------------------------------------------------------------
// SfxAsset — a procedurally generated arcade sound effect
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum WaveKind {
    Sine,
    Square,
    Sawtooth,
}

/// A procedurally synthesized sound: frequency sweep + optional noise mix.
#[derive(Asset, TypePath, Clone)]
pub struct SfxAsset {
    /// Starting frequency of the sweep in Hz.
    pub start_freq: f32,
    /// Ending frequency of the sweep in Hz.
    pub end_freq: f32,
    /// Playback duration in seconds.
    pub duration_secs: f32,
    /// Base waveform shape.
    pub waveform: WaveKind,
    /// Overall amplitude (0 – 1).
    pub volume: f32,
    /// Blend of noise into the signal (0 = pure tone, 1 = pure noise).
    pub noise_mix: f32,
}

// ---------------------------------------------------------------------------
// SfxDecoder — the stateful iterator that generates samples on demand
// ---------------------------------------------------------------------------

const SAMPLE_RATE: u32 = 44_100;

pub struct SfxDecoder {
    start_freq: f32,
    end_freq: f32,
    total_samples: u64,
    current_sample: u64,
    /// Current position within one waveform period [0, 1).
    phase: f32,
    waveform: WaveKind,
    volume: f32,
    noise_mix: f32,
    /// Linear congruential generator state for cheap white noise.
    lcg: u64,
}

impl SfxDecoder {
    fn new(asset: &SfxAsset) -> Self {
        let total_samples = (asset.duration_secs * SAMPLE_RATE as f32) as u64;
        Self {
            start_freq: asset.start_freq,
            end_freq: asset.end_freq,
            total_samples,
            current_sample: 0,
            phase: 0.0,
            waveform: asset.waveform.clone(),
            volume: asset.volume,
            noise_mix: asset.noise_mix,
            lcg: 0xDEAD_BEEF_1234_5678,
        }
    }
}

impl Iterator for SfxDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.current_sample >= self.total_samples {
            return None;
        }

        let t = self.current_sample as f32 / self.total_samples as f32;

        // Exponential frequency sweep — sounds more natural than linear.
        let freq = self.start_freq * (self.end_freq / self.start_freq).powf(t);

        // Advance oscillator phase.
        self.phase += freq / SAMPLE_RATE as f32;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let tone = match self.waveform {
            WaveKind::Sine => (std::f32::consts::TAU * self.phase).sin(),
            WaveKind::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveKind::Sawtooth => self.phase * 2.0 - 1.0,
        };

        // Cheap LCG white noise.
        self.lcg = self
            .lcg
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let noise = ((self.lcg >> 32) as i32) as f32 / i32::MAX as f32;

        let signal = tone * (1.0 - self.noise_mix) + noise * self.noise_mix;

        // Amplitude envelope: linear fade-out over the last 20% of the clip.
        let envelope = if t > 0.8 { (1.0 - t) / 0.2 } else { 1.0 };

        self.current_sample += 1;
        Some(signal * self.volume * envelope)
    }
}

impl Source for SfxDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.total_samples as f32 / SAMPLE_RATE as f32,
        ))
    }
}

impl Decodable for SfxAsset {
    type DecoderItem = f32;
    type Decoder = SfxDecoder;

    fn decoder(&self) -> SfxDecoder {
        SfxDecoder::new(self)
    }
}

// ---------------------------------------------------------------------------
// Pre-baked sound handles
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct SoundAssets {
    player_shoot: Handle<SfxAsset>,
    enemy_shoot: Handle<SfxAsset>,
    enemy_die: Handle<SfxAsset>,
    player_die: Handle<SfxAsset>,
    mystery_die: Handle<SfxAsset>,
    wave_cleared: Handle<SfxAsset>,
}

fn setup_sounds(mut commands: Commands, mut assets: ResMut<Assets<SfxAsset>>) {
    // Short descending square-wave zap — player laser.
    let player_shoot = assets.add(SfxAsset {
        start_freq: 880.0,
        end_freq: 220.0,
        duration_secs: 0.09,
        waveform: WaveKind::Square,
        volume: 0.35,
        noise_mix: 0.0,
    });

    // Deeper, slightly noisier zap — enemy laser.
    let enemy_shoot = assets.add(SfxAsset {
        start_freq: 350.0,
        end_freq: 110.0,
        duration_secs: 0.13,
        waveform: WaveKind::Square,
        volume: 0.28,
        noise_mix: 0.08,
    });

    // Quick sawtooth crunch with half-noise — small explosion.
    let enemy_die = assets.add(SfxAsset {
        start_freq: 420.0,
        end_freq: 35.0,
        duration_secs: 0.18,
        waveform: WaveKind::Sawtooth,
        volume: 0.42,
        noise_mix: 0.50,
    });

    // Long, bass-heavy noisy sweep — dramatic player death.
    let player_die = assets.add(SfxAsset {
        start_freq: 600.0,
        end_freq: 20.0,
        duration_secs: 0.55,
        waveform: WaveKind::Sawtooth,
        volume: 0.55,
        noise_mix: 0.65,
    });

    // Wide sine sweep with noise — big mystery ship explosion.
    let mystery_die = assets.add(SfxAsset {
        start_freq: 1800.0,
        end_freq: 70.0,
        duration_secs: 0.40,
        waveform: WaveKind::Sine,
        volume: 0.48,
        noise_mix: 0.30,
    });

    // Clean ascending sine chirp — wave complete fanfare.
    let wave_cleared = assets.add(SfxAsset {
        start_freq: 440.0,
        end_freq: 880.0,
        duration_secs: 0.28,
        waveform: WaveKind::Sine,
        volume: 0.40,
        noise_mix: 0.0,
    });

    commands.insert_resource(SoundAssets {
        player_shoot,
        enemy_shoot,
        enemy_die,
        player_die,
        mystery_die,
        wave_cleared,
    });
}

// ---------------------------------------------------------------------------
// Play queued sounds by spawning AudioPlayer entities
// ---------------------------------------------------------------------------

fn play_sounds(
    mut commands: Commands,
    mut queue: ResMut<SoundQueue>,
    sounds: Option<Res<SoundAssets>>,
) {
    let Some(sounds) = sounds else {
        return;
    };

    for kind in queue.0.drain(..) {
        let handle: Handle<SfxAsset> = match kind {
            SoundKind::PlayerShoot => sounds.player_shoot.clone(),
            SoundKind::EnemyShoot => sounds.enemy_shoot.clone(),
            SoundKind::EnemyDie => sounds.enemy_die.clone(),
            SoundKind::PlayerDie => sounds.player_die.clone(),
            SoundKind::MysteryShipDie => sounds.mystery_die.clone(),
            SoundKind::WaveCleared => sounds.wave_cleared.clone(),
        };
        commands.spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN));
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<SfxAsset>()
            .init_resource::<SoundQueue>()
            .add_systems(Startup, setup_sounds)
            .add_systems(PostUpdate, play_sounds);
    }
}

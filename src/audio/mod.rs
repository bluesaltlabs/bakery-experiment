use bevy::prelude::*;

#[derive(Resource)]
pub struct UserVolume(pub f32);

impl Default for UserVolume {
    fn default() -> Self {
        Self(0.25)
    }
}

#[derive(Clone)]
pub enum AudioEvent {
    Step,
    Pickup,
    Drop,
    StationDeposit,
    StationComplete,
    ConveyorTick,
    Win,
    Lose,
    TimerWarning,
}

#[derive(Resource, Default)]
pub struct AudioEventQueue(pub Vec<AudioEvent>);

fn note_freq(note: u8) -> f32 {
    440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
}

#[allow(dead_code)]
enum Waveform {
    Square,
    Triangle,
    Sawtooth,
    Sine,
    Noise,
}

struct SoundSpec {
    waveform: Waveform,
    notes: Vec<(u8, f32)>,
    #[allow(dead_code)]
    volume: f32,
}

fn sound_for_event(event: &AudioEvent) -> SoundSpec {
    match event {
        AudioEvent::Step => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(40, 0.05)],
            volume: 0.25,
        },
        AudioEvent::Pickup => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(72, 0.06), (76, 0.06)],
            volume: 0.3,
        },
        AudioEvent::Drop => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(36, 0.08)],
            volume: 0.3,
        },
        AudioEvent::StationDeposit => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(60, 0.06), (64, 0.06)],
            volume: 0.35,
        },
        AudioEvent::StationComplete => SoundSpec {
            waveform: Waveform::Triangle,
            notes: vec![(72, 0.08), (76, 0.08), (79, 0.12)],
            volume: 0.4,
        },
        AudioEvent::ConveyorTick => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(84, 0.02)],
            volume: 0.12,
        },
        AudioEvent::Win => SoundSpec {
            waveform: Waveform::Triangle,
            notes: vec![(60, 0.10), (64, 0.10), (67, 0.10), (72, 0.20)],
            volume: 0.5,
        },
        AudioEvent::Lose => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(72, 0.12), (67, 0.12), (64, 0.12), (60, 0.20)],
            volume: 0.4,
        },
        AudioEvent::TimerWarning => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(76, 0.06)],
            volume: 0.4,
        },
    }
}

fn linear_congruential_gen(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (*seed as f32) / (u32::MAX as f32)
}

#[cfg(target_arch = "wasm32")]
mod wasm_backend;
#[cfg(not(target_arch = "wasm32"))]
mod native_backend;

#[cfg(target_arch = "wasm32")]
pub use wasm_backend::setup_audio_system;
#[cfg(not(target_arch = "wasm32"))]
pub use native_backend::setup_audio_system;

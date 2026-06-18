use bevy::prelude::*;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const SAMPLE_RATE: u32 = 44100;

#[derive(Resource)]
pub struct UserVolume(pub f32);

impl Default for UserVolume {
    fn default() -> Self {
        Self(0.8)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum AudioEvent {
    Step,
    Pickup,
    Drop,
    Interact,
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

#[allow(dead_code)]
struct SoundSpec {
    waveform: Waveform,
    notes: Vec<(u8, f32)>,
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
        AudioEvent::Interact => SoundSpec {
            waveform: Waveform::Square,
            notes: vec![(60, 0.07)],
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

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn render_spec(spec: &SoundSpec) -> Vec<i16> {
    let total_samples: usize = spec
        .notes
        .iter()
        .map(|(_, dur)| (dur * SAMPLE_RATE as f32) as usize)
        .sum();
    let mut buffer = Vec::with_capacity(total_samples);

    for &(note, dur) in &spec.notes {
        let freq = note_freq(note);
        let n = (dur * SAMPLE_RATE as f32) as usize;
        let start = buffer.len();
        buffer.resize(start + n, 0);

        match spec.waveform {
            Waveform::Square => {
                for i in 0..n {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let v = if (t * freq).fract() < 0.5 { 1.0 } else { -1.0 };
                    buffer[start + i] = (v * i16::MAX as f32) as i16;
                }
            }
            Waveform::Triangle => {
                for i in 0..n {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let phase = (t * freq).fract();
                    let v = if phase < 0.5 {
                        4.0 * phase - 1.0
                    } else {
                        3.0 - 4.0 * phase
                    };
                    buffer[start + i] = (v * i16::MAX as f32) as i16;
                }
            }
            Waveform::Sawtooth => {
                for i in 0..n {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let v = 2.0 * (t * freq).fract() - 1.0;
                    buffer[start + i] = (v * i16::MAX as f32) as i16;
                }
            }
            Waveform::Sine => {
                for i in 0..n {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let v = (2.0 * std::f32::consts::PI * freq * t).sin();
                    buffer[start + i] = (v * i16::MAX as f32) as i16;
                }
            }
            Waveform::Noise => {
                let mut seed: u32 = 123456789;
                for i in 0..n {
                    let v = linear_congruential_gen(&mut seed) * 2.0 - 1.0;
                    buffer[start + i] = (v * i16::MAX as f32 * 0.5) as i16;
                }
            }
        }
    }

    let len = buffer.len();
    if len > 0 {
        let attack = (len as f32 * 0.05) as usize;
        let release = (len as f32 * 0.1) as usize;
        for i in 0..len.min(attack) {
            let t = i as f32 / attack as f32;
            buffer[i] = (buffer[i] as f32 * t) as i16;
        }
        for i in len.saturating_sub(release)..len {
            let t = (len - i) as f32 / release as f32;
            buffer[i] = (buffer[i] as f32 * t) as i16;
        }
    }

    buffer
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let file_len = 44 + data_len;
    let mut bytes = Vec::with_capacity(file_len);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(file_len as u32 - 8).to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data_len as u32).to_le_bytes());

    for &sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    bytes
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{
        AudioBufferSourceNode, AudioContext, AudioContextState, GainNode, OscillatorNode,
        OscillatorType,
    };

    use std::sync::OnceLock;

    fn ctx() -> &'static AudioContext {
        static CTX: OnceLock<AudioContext> = OnceLock::new();
        CTX.get_or_init(|| {
            let c = AudioContext::new().expect("Failed to create AudioContext");
            register_gesture_handler(&c);
            c
        })
    }

    fn register_gesture_handler(ctx: &AudioContext) {
        let ctx_clone = ctx.clone();
        let closure = Closure::wrap(Box::new(move || {
            let _ = ctx_clone.resume();
        }) as Box<dyn FnMut()>);

        if let Some(win) = web_sys::window() {
            let func: &js_sys::Function = closure.as_ref().unchecked_ref();
            let _ = win.add_event_listener_with_callback("keydown", func);
            let _ = win.add_event_listener_with_callback("mousedown", func);
            let _ = win.add_event_listener_with_callback("touchstart", func);
        }
        closure.forget();
    }

    pub fn resume_audio() {
        let c = ctx();
        if c.state() == AudioContextState::Suspended {
            let _ = c.resume();
        }
    }

    fn play_note(freq: f32, dur: f32, vol: f32, osc_type: &OscillatorType) {
        let c = ctx();
        let osc: OscillatorNode = c.create_oscillator().unwrap_throw();
        let gain: GainNode = c.create_gain().unwrap_throw();

        osc.set_type(osc_type.clone());
        osc.frequency().set_value(freq);
        gain.gain().set_value(vol);

        osc.connect_with_audio_node(&gain).unwrap_throw();
        gain.connect_with_audio_node(&c.destination()).unwrap_throw();

        let now = c.current_time();
        osc.start_with_when(now).unwrap_throw();
        osc.stop_with_when(now + dur as f64).unwrap_throw();
    }

    fn play_noise(dur: f32, vol: f32) {
        let c = ctx();
        let sample_rate = c.sample_rate() as u32;
        let n_samples = (dur * sample_rate as f32) as u32;

        let buffer = c
            .create_buffer(1, n_samples, sample_rate as f32)
            .unwrap_throw();
        let mut channel_data = buffer.get_channel_data(0).unwrap_throw();

        let mut seed: u32 = 123456789;
        for i in 0..n_samples as usize {
            let v = linear_congruential_gen(&mut seed);
            channel_data[i] = (v * 2.0 - 1.0) * vol;
        }
        buffer.copy_to_channel(&channel_data, 0).unwrap_throw();

        let source: AudioBufferSourceNode = c.create_buffer_source().unwrap_throw();
        source.set_buffer(Some(&buffer));
        source.connect_with_audio_node(&c.destination()).unwrap_throw();
        source.start().unwrap_throw();
    }

    pub fn play_spec(spec: &SoundSpec, user_volume: f32) {
        resume_audio();
        let vol = spec.volume * user_volume;

        let ot = match spec.waveform {
            Waveform::Square => OscillatorType::Square,
            Waveform::Triangle => OscillatorType::Triangle,
            Waveform::Sawtooth => OscillatorType::Sawtooth,
            Waveform::Sine => OscillatorType::Sine,
            Waveform::Noise => {
                let total_dur: f32 = spec.notes.iter().map(|(_, d)| d).sum();
                play_noise(total_dur, vol);
                return;
            }
        };

        for &(note, dur) in &spec.notes {
            play_note(note_freq(note), dur, vol, &ot);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use bevy::audio::{AudioBundle, AudioSource, Volume};

    #[derive(Resource)]
    pub struct AudioHandles {
        pub step: Handle<AudioSource>,
        pub pickup: Handle<AudioSource>,
        pub drop: Handle<AudioSource>,
        pub interact: Handle<AudioSource>,
        pub station_deposit: Handle<AudioSource>,
        pub station_complete: Handle<AudioSource>,
        pub conveyor_tick: Handle<AudioSource>,
        pub win: Handle<AudioSource>,
        pub lose: Handle<AudioSource>,
        pub timer_warning: Handle<AudioSource>,
    }

    fn add_sound(assets: &mut Assets<AudioSource>, spec: &SoundSpec) -> Handle<AudioSource> {
        let samples = render_spec(spec);
        let bytes = wav_bytes(&samples);
        assets.add(AudioSource { bytes: bytes.into() })
    }

    pub fn setup_handles(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
        let mut make = |event: &AudioEvent| add_sound(&mut assets, &sound_for_event(event));
        commands.insert_resource(AudioHandles {
            step: make(&AudioEvent::Step),
            pickup: make(&AudioEvent::Pickup),
            drop: make(&AudioEvent::Drop),
            interact: make(&AudioEvent::Interact),
            station_deposit: make(&AudioEvent::StationDeposit),
            station_complete: make(&AudioEvent::StationComplete),
            conveyor_tick: make(&AudioEvent::ConveyorTick),
            win: make(&AudioEvent::Win),
            lose: make(&AudioEvent::Lose),
            timer_warning: make(&AudioEvent::TimerWarning),
        });
    }

    pub fn process_queue(
        mut commands: Commands,
        mut queue: ResMut<AudioEventQueue>,
        handles: Res<AudioHandles>,
        user_volume: Res<UserVolume>,
    ) {
        let events = std::mem::take(&mut queue.0);
        for event in events {
            let handle = match event {
                AudioEvent::Step => &handles.step,
                AudioEvent::Pickup => &handles.pickup,
                AudioEvent::Drop => &handles.drop,
                AudioEvent::Interact => &handles.interact,
                AudioEvent::StationDeposit => &handles.station_deposit,
                AudioEvent::StationComplete => &handles.station_complete,
                AudioEvent::ConveyorTick => &handles.conveyor_tick,
                AudioEvent::Win => &handles.win,
                AudioEvent::Lose => &handles.lose,
                AudioEvent::TimerWarning => &handles.timer_warning,
            };
            commands.spawn((
                AudioBundle {
                    source: handle.clone(),
                    settings: PlaybackSettings::ONCE.with_volume(Volume::new(user_volume.0)),
                },
            ));
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn setup_audio_system(app: &mut App) {
    app.init_resource::<AudioEventQueue>();
    app.init_resource::<UserVolume>();
    app.add_systems(Update, wasm_audio_system);
}

#[cfg(target_arch = "wasm32")]
fn wasm_audio_system(mut queue: ResMut<AudioEventQueue>, user_volume: Res<UserVolume>) {
    let events = std::mem::take(&mut queue.0);
    for event in events {
        let spec = sound_for_event(&event);
        backend::play_spec(&spec, user_volume.0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_audio_system(app: &mut App) {
    use backend::{setup_handles, process_queue};
    app.init_resource::<AudioEventQueue>();
    app.init_resource::<UserVolume>();
    app.add_systems(Startup, setup_handles);
    app.add_systems(Update, process_queue);
}

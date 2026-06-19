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

fn play_spec(spec: &SoundSpec, user_volume: f32) {
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

fn wasm_audio_system(mut queue: ResMut<AudioEventQueue>, user_volume: Res<UserVolume>) {
    let events = std::mem::take(&mut queue.0);
    for event in events {
        let spec = sound_for_event(&event);
        play_spec(&spec, user_volume.0);
    }
}

pub fn setup_audio_system(app: &mut App) {
    app.init_resource::<AudioEventQueue>();
    app.init_resource::<UserVolume>();
    app.add_systems(Update, wasm_audio_system);
}

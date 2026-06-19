use super::*;
use bevy::audio::{AudioBundle, AudioSource, Volume};

const SAMPLE_RATE: u32 = 44100;

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

#[derive(Resource)]
struct AudioHandles {
    pub step: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub drop: Handle<AudioSource>,
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

fn setup_handles(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    let mut make = |event: &AudioEvent| add_sound(&mut assets, &sound_for_event(event));
    commands.insert_resource(AudioHandles {
        step: make(&AudioEvent::Step),
        pickup: make(&AudioEvent::Pickup),
        drop: make(&AudioEvent::Drop),
        station_deposit: make(&AudioEvent::StationDeposit),
        station_complete: make(&AudioEvent::StationComplete),
        conveyor_tick: make(&AudioEvent::ConveyorTick),
        win: make(&AudioEvent::Win),
        lose: make(&AudioEvent::Lose),
        timer_warning: make(&AudioEvent::TimerWarning),
    });
}

fn process_queue(
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

pub fn setup_audio_system(app: &mut App) {
    app.init_resource::<AudioEventQueue>();
    app.init_resource::<UserVolume>();
    app.add_systems(Startup, setup_handles);
    app.add_systems(Update, process_queue);
}

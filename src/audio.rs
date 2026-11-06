//! Glider PRO's sound: three effect channels and a music channel, as its
//! Sound Manager channels were (Sound.c, Music.c). A sound started on a
//! channel replaces what plays there. When a sound ends by itself the
//! channel is reported back, so the game can do what its channel
//! callbacks did (reset the channel's priority; queue the next piece of
//! music). Mixed in software at 22254 Hz, 8-bit unsigned mono. Without an
//! audio device, or with `--mute`, nothing plays and every sound counts as
//! finished at once.

use crate::mac::Sound;
use rodio::{MixerDeviceSink, Source};
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RATE: u32 = 22_254;
/// Channels 0 to 2 are effects, 3 is music.
pub const CHANNELS: usize = 4;
pub const MUSIC: usize = 3;

#[derive(Clone, Default)]
struct Voice {
    sound: Option<Arc<Sound>>,
    /// Position in the sample, 16.16 fixed point.
    pos: u64,
    step: u64,
    len: u64,
}

#[derive(Default)]
struct Shared {
    /// Sounds to start: (channel, sound; None stops the channel).
    start: Vec<(usize, Option<Arc<Sound>>)>,
    /// Channels whose sound ended by itself.
    finished: Vec<usize>,
}

struct Mixer {
    shared: Arc<Mutex<Shared>>,
    voices: [Voice; CHANNELS],
    volume: Arc<Mutex<[f32; CHANNELS]>>,
    vol: [f32; CHANNELS],
    poll: u32,
}

impl Iterator for Mixer {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.poll == 0 {
            self.poll = 64;
            if let Ok(mut q) = self.shared.lock() {
                for (c, s) in std::mem::take(&mut q.start) {
                    self.voices[c] = match s {
                        Some(s) => Voice {
                            step: ((s.rate.max(1) as u64) << 16) / RATE as u64,
                            len: (s.data.len() as u64) << 16,
                            pos: 0,
                            sound: Some(s),
                        },
                        None => Voice::default(),
                    };
                }
                for c in 0..CHANNELS {
                    let v = &mut self.voices[c];
                    if v.sound.is_some() && v.pos >= v.len {
                        v.sound = None;
                        q.finished.push(c);
                    }
                }
            }
            if let Ok(v) = self.volume.lock() {
                self.vol = *v;
            }
        }
        self.poll -= 1;
        let mut mix = 0.0f32;
        for (c, v) in self.voices.iter_mut().enumerate() {
            if let Some(s) = &v.sound
                && v.pos < v.len
            {
                mix += (s.data[(v.pos >> 16) as usize] as f32 - 128.0) / 128.0 * self.vol[c];
                v.pos += v.step;
            }
        }
        Some((mix * 0.45).clamp(-1.0, 1.0))
    }
}

impl Source for Mixer {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(1).unwrap()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct Audio {
    // the sink must outlive the mixer playing on it
    sink: Option<MixerDeviceSink>,
    shared: Arc<Mutex<Shared>>,
    volume: Arc<Mutex<[f32; CHANNELS]>>,
    /// Channels started while there is no device: they finish at once.
    silent_finished: Vec<usize>,
}

impl Audio {
    pub fn new(off: bool) -> Audio {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let volume = Arc::new(Mutex::new([1.0; CHANNELS]));
        let sink = if off {
            None
        } else {
            match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(mut s) => {
                    s.log_on_drop(false);
                    let m = Mixer { shared: shared.clone(), voices: Default::default(), volume: volume.clone(), vol: [1.0; CHANNELS], poll: 0 };
                    s.mixer().add(m);
                    Some(s)
                }
                Err(e) => {
                    log::warn!("no audio: {e}");
                    None
                }
            }
        };
        Audio { sink, shared, volume, silent_finished: Vec::new() }
    }

    pub fn available(&self) -> bool {
        self.sink.is_some()
    }

    /// Start `s` on `channel`, replacing what plays there; `None` stops it.
    pub fn play(&mut self, channel: usize, s: Option<Arc<Sound>>) {
        if channel >= CHANNELS {
            return;
        }
        if self.sink.is_none() {
            if s.is_some() {
                self.silent_finished.push(channel);
            }
            return;
        }
        if let Ok(mut q) = self.shared.lock() {
            q.start.push((channel, s));
        }
    }

    /// A channel's volume, 0.0 to 1.0.
    pub fn set_volume(&mut self, channel: usize, v: f32) {
        if let Ok(mut vol) = self.volume.lock()
            && channel < CHANNELS
        {
            vol[channel] = v;
        }
    }

    /// Channels whose sound ended since the last call.
    pub fn finished(&mut self) -> Vec<usize> {
        let mut out = std::mem::take(&mut self.silent_finished);
        if let Ok(mut q) = self.shared.lock() {
            out.append(&mut q.finished);
        }
        out
    }
}

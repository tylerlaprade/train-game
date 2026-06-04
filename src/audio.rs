use std::io::Cursor;
use std::num::{NonZeroU16, NonZeroU32};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use rodio::source::Spatial;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

const CHUGGA: &[u8] = include_bytes!("../assets/chugga.flac");
const WHISTLE: &[u8] = include_bytes!("../assets/whistle.flac");
const ANOTHER_WHEEL: &[u8] = include_bytes!("../assets/another_wheel.flac");
const RAIN_VOLUME: f32 = 0.24;

/// Spatial layout for the engine sounds. The two ears straddle the screen
/// center and the emitter slides along the same axis between them, so the chug
/// and whistle pan left↔right with the on-screen engine. Matches the geometry
/// in rodio's own `spatial` example.
const LEFT_EAR: [f32; 3] = [-1.0, 0.0, 0.0];
const RIGHT_EAR: [f32; 3] = [1.0, 0.0, 0.0];

/// How often the audio thread re-reads the engine pan — 10ms tracks the engine
/// smoothly without churn.
const PAN_REFRESH: Duration = Duration::from_millis(10);

pub struct Audio {
    sink: MixerDeviceSink,
    chugga: Player,
    chugga_playing: bool,
    rain: Player,
    rain_playing: bool,
    horn: Player,
    /// Engine pan in [-1, 1], stored as `f32` bits. The render thread writes it
    /// each frame; the audio thread reads it lock-free while panning the chug
    /// and whistle. Avoiding a mutex here keeps a momentarily-stalled render
    /// thread from ever blocking the real-time audio callback.
    pan: Arc<AtomicU32>,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let sink = DeviceSinkBuilder::open_default_sink().ok()?;
        let pan = Arc::new(AtomicU32::new(0.0_f32.to_bits()));

        let chugga = Player::connect_new(sink.mixer());
        let source = Decoder::try_from(Cursor::new(CHUGGA))
            .ok()?
            .repeat_infinite();
        chugga.append(spatialize(source, pan.clone()));
        // The spatial pan attenuates each channel to ~0.75 at center, so the
        // volumes are bumped from their old mono values (chug 0.7, horn 1.0)
        // to keep roughly the original loudness.
        chugga.set_volume(0.9);
        chugga.pause();

        let rain = Player::connect_new(sink.mixer());
        rain.append(RainNoise::new());
        rain.set_volume(0.0);
        rain.pause();

        let horn = Player::connect_new(sink.mixer());
        horn.set_volume(1.3);

        Some(Self {
            sink,
            chugga,
            chugga_playing: false,
            rain,
            rain_playing: false,
            horn,
            pan,
        })
    }

    /// Pan the engine sounds (chug + whistle) to follow the on-screen engine.
    /// `pan` is in [-1.0, 1.0]: -1 hard left, 0 center, +1 hard right.
    pub fn set_engine_pan(&self, pan: f32) {
        self.pan
            .store(pan.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn tick_chugga(&mut self, moving: bool) {
        if moving && !self.chugga_playing {
            self.chugga.play();
            self.chugga_playing = true;
        } else if !moving && self.chugga_playing {
            self.chugga.pause();
            self.chugga_playing = false;
        }
    }

    pub fn tick_rain(&mut self, intensity: f32) {
        let intensity = intensity.clamp(0.0, 1.0);
        if intensity > 0.05 {
            self.rain.set_volume(RAIN_VOLUME * intensity);
            if !self.rain_playing {
                self.rain.play();
                self.rain_playing = true;
            }
        } else if self.rain_playing {
            self.rain.pause();
            self.rain_playing = false;
        }
    }

    pub fn horn(&mut self) {
        if !self.horn.empty() {
            return;
        }
        let Ok(source) = Decoder::try_from(Cursor::new(WHISTLE)) else {
            return;
        };
        self.horn.append(spatialize(source, self.pan.clone()));
    }

    pub fn another_wheel(&mut self) {
        self.play_oneshot(ANOTHER_WHEEL, 1.0);
    }

    fn play_oneshot(&self, data: &'static [u8], volume: f32) {
        let Ok(source) = Decoder::try_from(Cursor::new(data)) else {
            return;
        };
        let player = Player::connect_new(self.sink.mixer());
        player.set_volume(volume);
        player.append(source);
        player.detach();
    }
}

/// Wrap a source so it pans left↔right with the shared engine `pan` (an `f32`
/// in [-1, 1] stored as bits). The source is downmixed to mono and replayed in
/// stereo; the audio thread re-reads `pan` every [`PAN_REFRESH`] with a plain
/// atomic load, so it never has to take a lock that the render thread holds.
fn spatialize<S>(source: S, pan: Arc<AtomicU32>) -> impl Source<Item = f32> + Send + 'static
where
    S: Source<Item = f32> + Send + 'static,
{
    Spatial::new(source, emitter(0.0), LEFT_EAR, RIGHT_EAR).periodic_access(PAN_REFRESH, move |s| {
        let p = f32::from_bits(pan.load(Ordering::Relaxed));
        s.set_positions(emitter(p), LEFT_EAR, RIGHT_EAR);
    })
}

/// Emitter position for a given pan, sliding along the ear axis.
fn emitter(pan: f32) -> [f32; 3] {
    [pan, 0.0, 0.0]
}

#[derive(Clone)]
struct RainNoise {
    state: u32,
    low: f32,
    mid: f32,
}

impl RainNoise {
    fn new() -> Self {
        Self {
            state: 0x5EED_5EED,
            low: 0.0,
            mid: 0.0,
        }
    }

    fn white(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        let sample = (self.state >> 8) as f32 / 0x00FF_FFFF as f32;
        sample * 2.0 - 1.0
    }
}

impl Iterator for RainNoise {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let white = self.white();
        self.low = self.low * 0.985 + white * 0.015;
        self.mid = self.mid * 0.72 + white * 0.28;
        Some((self.low * 0.55 + self.mid * 0.45).clamp(-1.0, 1.0))
    }
}

impl Source for RainNoise {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        NonZeroU16::new(1).unwrap()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        NonZeroU32::new(44_100).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

use std::io::Cursor;
use std::num::{NonZeroU16, NonZeroU32};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use rodio::cpal;
use rodio::cpal::traits::HostTrait;
use rodio::source::Spatial;
use rodio::{Decoder, DeviceSinkBuilder, DeviceTrait, MixerDeviceSink, Player, Source};

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

/// Sample rate of the silent [`Heartbeat`] source. The watchdog compares the
/// heartbeat's sample count against wall-clock time at this rate.
const HEARTBEAT_RATE: u32 = 44_100;

/// How much wall-clock time the watchdog lets pass between readings of the
/// heartbeat counter. Long enough that ordinary scheduling jitter can't look
/// like a stall, short enough that a woken laptop gets its sound back within a
/// frame or two of the lid opening.
const WATCHDOG_INTERVAL: Duration = Duration::from_millis(400);

/// A live device consumes samples at [`HEARTBEAT_RATE`]. Treat anything below
/// this fraction of the expected count as "the stream is no longer running".
const STALL_FRACTION: f64 = 0.25;

/// Minimum gap between attempts to reopen the sink, so a machine with no
/// working audio device doesn't try to open one on every frame.
const REOPEN_COOLDOWN: Duration = Duration::from_millis(1500);

pub struct Audio {
    sink: MixerDeviceSink,
    chugga: Player,
    chugga_playing: bool,
    rain: Player,
    rain_playing: bool,
    rain_intensity: f32,
    horn: Player,
    /// Engine pan in [-1, 1], stored as `f32` bits. The render thread writes it
    /// each frame; the audio thread reads it lock-free while panning the chug
    /// and whistle. Avoiding a mutex here keeps a momentarily-stalled render
    /// thread from ever blocking the real-time audio callback.
    pan: Arc<AtomicU32>,
    /// Samples the OS has pulled through the mixer, counted by [`Heartbeat`].
    pulled: Arc<AtomicU64>,
    /// Set from cpal's stream-error callback when the device goes away.
    lost: Arc<AtomicBool>,
    last_watchdog: Instant,
    last_pulled: u64,
    last_reopen: Instant,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let lost = Arc::new(AtomicBool::new(false));
        let mut sink = open_sink(lost.clone())?;
        sink.log_on_drop(false);
        let pan = Arc::new(AtomicU32::new(0.0_f32.to_bits()));

        let pulled = Arc::new(AtomicU64::new(0));
        sink.mixer().add(Heartbeat {
            pulled: pulled.clone(),
        });

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

        let now = Instant::now();
        Some(Self {
            sink,
            chugga,
            chugga_playing: false,
            rain,
            rain_playing: false,
            rain_intensity: 0.0,
            horn,
            pan,
            pulled,
            lost,
            last_watchdog: now,
            last_pulled: 0,
            last_reopen: now,
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
        self.rain_intensity = intensity;
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

    /// Reopen the OS audio sink if it has stopped playing.
    ///
    /// Closing a MacBook lid tears down the CoreAudio device behind our
    /// stream. The stream object stays alive and rodio keeps happily mixing
    /// into it, but nothing is ever pulled out again, so the game goes silent
    /// until it is restarted. Rebuilding the sink is the only way back, so the
    /// main loop calls this every frame and we watch the [`Heartbeat`] counter
    /// to decide when a rebuild is due.
    pub fn recover_if_stalled(&mut self) {
        let now = Instant::now();
        if !self.is_stalled(now) {
            return;
        }
        if now.duration_since(self.last_reopen) < REOPEN_COOLDOWN {
            return;
        }
        self.last_reopen = now;

        let Some(mut fresh) = Audio::new() else {
            return;
        };
        // Carry the mixing state across so the rebuild is inaudible beyond the
        // gap itself: the chug resumes if the train is rolling, the rain comes
        // back at its current strength, and the engine keeps its pan.
        fresh.set_engine_pan(f32::from_bits(self.pan.load(Ordering::Relaxed)));
        fresh.tick_chugga(self.chugga_playing);
        fresh.tick_rain(self.rain_intensity);
        fresh.last_reopen = now;
        *self = fresh;
    }

    /// Has the OS stopped consuming samples from our mixer?
    fn is_stalled(&mut self, now: Instant) -> bool {
        if self.lost.load(Ordering::Relaxed) {
            return true;
        }

        let since = now.duration_since(self.last_watchdog);
        if since < WATCHDOG_INTERVAL {
            return false;
        }
        let pulled = self.pulled.load(Ordering::Relaxed);
        let advanced = pulled.saturating_sub(self.last_pulled);
        self.last_watchdog = now;
        self.last_pulled = pulled;

        looks_stalled(advanced, since)
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

/// Did the device pull far fewer samples than the elapsed wall-clock time
/// calls for? A running stream tracks [`HEARTBEAT_RATE`] almost exactly, and a
/// torn-down one pulls nothing at all, so the two cases are far apart.
fn looks_stalled(advanced: u64, since: Duration) -> bool {
    let expected = f64::from(HEARTBEAT_RATE) * since.as_secs_f64();
    (advanced as f64) < expected * STALL_FRACTION
}

/// Open the default OS audio sink, mirroring rodio's `open_default_sink` but
/// with our own stream-error callback. The callback flips `lost` so the
/// watchdog reacts immediately, and — just as importantly — keeps rodio's
/// default handler from printing an error over the alternate screen the game
/// is drawing into.
fn open_sink(lost: Arc<AtomicBool>) -> Option<MixerDeviceSink> {
    let on_error = move |_: cpal::StreamError| lost.store(true, Ordering::Relaxed);

    let default = DeviceSinkBuilder::from_default_device()
        .ok()
        .and_then(|builder| {
            builder
                .with_error_callback(on_error.clone())
                .open_sink_or_fallback()
                .ok()
        });
    if default.is_some() {
        return default;
    }

    cpal::default_host()
        .output_devices()
        .ok()?
        .filter(|device| {
            device
                .description()
                .map(|desc| desc.driver().is_some_and(|driver| driver != "null"))
                .unwrap_or(false)
        })
        .find_map(|device| {
            DeviceSinkBuilder::from_device(device)
                .ok()?
                .with_error_callback(on_error.clone())
                .open_sink_or_fallback()
                .ok()
        })
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

/// A silent, endless source that counts how many samples the OS has pulled
/// through the mixer. It is the game's proof that the audio device is still
/// alive: the count climbs at [`HEARTBEAT_RATE`] for as long as the stream is
/// running, and freezes the moment the device disappears.
struct Heartbeat {
    pulled: Arc<AtomicU64>,
}

impl Iterator for Heartbeat {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.pulled.fetch_add(1, Ordering::Relaxed);
        Some(0.0)
    }
}

impl Source for Heartbeat {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        NonZeroU16::new(1).unwrap()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        NonZeroU32::new(HEARTBEAT_RATE).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::{HEARTBEAT_RATE, Heartbeat, looks_stalled};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    #[test]
    fn a_running_device_is_not_reported_as_stalled() {
        let second = Duration::from_secs(1);

        assert!(!looks_stalled(u64::from(HEARTBEAT_RATE), second));
        // Resampling and buffer boundaries mean the count is never exact, so
        // an ordinary shortfall must not trip the watchdog either.
        assert!(!looks_stalled(u64::from(HEARTBEAT_RATE) / 2, second));
    }

    #[test]
    fn a_device_that_stops_pulling_is_reported_as_stalled() {
        // What a closed lid looks like: the stream is still there, but nothing
        // comes out of it no matter how long we wait.
        assert!(looks_stalled(0, Duration::from_millis(400)));
        assert!(looks_stalled(0, Duration::from_secs(600)));
        // A stream that dies part-way through the window counts too.
        assert!(looks_stalled(
            u64::from(HEARTBEAT_RATE) / 20,
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn heartbeat_counts_every_sample_the_device_takes() {
        let pulled = Arc::new(AtomicU64::new(0));
        let mut heartbeat = Heartbeat {
            pulled: pulled.clone(),
        };

        for _ in 0..1000 {
            assert_eq!(heartbeat.next(), Some(0.0));
        }

        assert_eq!(pulled.load(Ordering::Relaxed), 1000);
    }
}

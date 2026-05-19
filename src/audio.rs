use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const STEAM_ENGINE_URL: &str =
    "https://upload.wikimedia.org/wikipedia/commons/6/6c/Steam_engine.ogg";
const WHISTLE_URL: &str =
    "https://upload.wikimedia.org/wikipedia/commons/5/51/WWS_SteamWhistle.ogg";

/// Non-blocking sound system. Real SFX (downloaded + transcoded by ffmpeg)
/// where possible; falls back to macOS `say` per-sound if anything fails.
pub struct Audio {
    chugga: PathBuf,
    horn: PathBuf,
    another_wheel: PathBuf,
    nope: PathBuf,

    chugga_child: Option<Child>,
    horn_child: Option<Child>,
    speak_child: Option<Child>,
    warmup_child: Option<Child>,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let cache = cache_dir()?;
        std::fs::create_dir_all(&cache).ok()?;

        // Try real SFX first; fall back to TTS per-sound if download/convert fails.
        let chugga = ensure_chugga(&cache)
            .unwrap_or_else(|| synth_fallback(&cache, "chugga.aiff", "chuh guh", "Fred", 220));
        let whistle = ensure_whistle(&cache)
            .unwrap_or_else(|| synth_fallback(&cache, "whistle.aiff", "choo choo", "Whisper", 180));
        // Horn: a short, snappy version of the whistle (or just the whistle).
        let horn = whistle.clone();

        // Filename versioned so prior installs regenerate after phrase changes.
        let another_wheel = synth_fallback(
            &cache,
            "another_wheel_v2.aiff",
            "another wheel",
            "Samantha",
            150,
        );
        let nope = synth_fallback(&cache, "nope.aiff", "uh oh", "Junior", 200);
        let warmup_child = warm_audio_driver(&chugga);

        Some(Self {
            chugga,
            horn,
            another_wheel,
            nope,
            chugga_child: None,
            horn_child: None,
            speak_child: None,
            warmup_child,
        })
    }

    pub fn tick_chugga(&mut self, moving: bool) {
        self.reap_warmup();

        // Reap finished chugga child.
        if let Some(c) = self.chugga_child.as_mut()
            && matches!(c.try_wait(), Ok(Some(_)))
        {
            self.chugga_child = None;
        }

        if !moving {
            // Stop chugga the instant the train stops so there's no trailing audio.
            if let Some(mut c) = self.chugga_child.take() {
                let _ = c.kill();
                let _ = c.wait();
            }
            return;
        }

        // Loop chugga: spawn a fresh playback whenever the previous one ends.
        if self.chugga_child.is_none() {
            self.chugga_child = spawn_afplay_quiet(&self.chugga, 0.7);
        }
    }

    pub fn horn(&mut self) {
        self.reap_warmup();

        if let Some(c) = self.horn_child.as_mut()
            && matches!(c.try_wait(), Ok(Some(_)))
        {
            self.horn_child = None;
        }
        if self.horn_child.is_none() {
            self.horn_child = spawn_afplay_quiet(&self.horn, 1.0);
        }
    }

    pub fn another_wheel(&mut self) {
        self.reap_warmup();

        let _ = spawn_afplay_quiet(&self.another_wheel, 1.0);
    }

    #[allow(dead_code)]
    pub fn nope(&mut self) {
        self.reap_warmup();

        let _ = spawn_afplay_quiet(&self.nope, 1.0);
    }

    fn reap_warmup(&mut self) {
        if let Some(c) = self.warmup_child.as_mut()
            && matches!(c.try_wait(), Ok(Some(_)))
        {
            self.warmup_child = None;
        }
    }

    /// Speak arbitrary text. Currently unused (kept in case we wire up a
    /// post-action announcement again). Cancels any prior speech.
    #[allow(dead_code)]
    pub fn speak(&mut self, text: &str) {
        if let Some(mut c) = self.speak_child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        self.speak_child = Command::new("say")
            .arg("-v")
            .arg("Junior")
            .arg("-r")
            .arg("180")
            .arg(text)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        for c in [
            &mut self.chugga_child,
            &mut self.horn_child,
            &mut self.speak_child,
            &mut self.warmup_child,
        ]
        .into_iter()
        .flatten()
        {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

fn cache_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library/Caches/train-game"))
}

fn ensure_chugga(cache: &Path) -> Option<PathBuf> {
    let aiff = cache.join("chugga.aiff");
    if aiff.exists() {
        return Some(aiff);
    }
    let ogg = cache.join("steam_engine.ogg");
    if !ogg.exists() && download(STEAM_ENGINE_URL, &ogg).is_none() {
        return None;
    }
    // Skip the first 1s of the recording (lead-in / silence) and take ~42s
    // of continuous chugging. The clip is already public domain so we just
    // transcode to a mono 22kHz AIFF that afplay handles natively.
    let ok = ffmpeg(
        &["-y", "-ss", "1", "-t", "42", "-i"],
        &ogg,
        &aiff,
        &["-ac", "1", "-ar", "22050"],
    );
    if ok { Some(aiff) } else { None }
}

fn ensure_whistle(cache: &Path) -> Option<PathBuf> {
    let aiff = cache.join("whistle.aiff");
    if aiff.exists() {
        return Some(aiff);
    }
    let ogg = cache.join("whistle.ogg");
    if !ogg.exists() && download(WHISTLE_URL, &ogg).is_none() {
        return None;
    }
    // Trim to first 2.2s — long enough to read as a whistle, short enough
    // not to overrun the periodic interval.
    let ok = ffmpeg(
        &["-y", "-t", "2.2", "-i"],
        &ogg,
        &aiff,
        &["-ac", "1", "-ar", "22050"],
    );
    if ok { Some(aiff) } else { None }
}

fn download(url: &str, dest: &Path) -> Option<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "--max-time", "20", url, "-o"])
        .arg(dest)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    if status.success() { Some(()) } else { None }
}

fn ffmpeg(pre_args: &[&str], input: &Path, output: &Path, post_args: &[&str]) -> bool {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(pre_args).arg(input).args(post_args).arg(output);
    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn synth_fallback(cache: &Path, filename: &str, text: &str, voice: &str, rate: u32) -> PathBuf {
    let path = cache.join(filename);
    if !path.exists() {
        let _ = Command::new("say")
            .arg("-v")
            .arg(voice)
            .arg("-r")
            .arg(rate.to_string())
            .arg("-o")
            .arg(&path)
            .arg(text)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    path
}

fn spawn_afplay_quiet(path: &Path, volume: f32) -> Option<Child> {
    Command::new("afplay")
        .arg("-v")
        .arg(format!("{:.2}", volume))
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn warm_audio_driver(path: &Path) -> Option<Child> {
    Command::new("afplay")
        .arg("-v")
        .arg("0.00")
        .arg("-t")
        .arg("0.05")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

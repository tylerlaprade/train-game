use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Lazily-generated, fire-and-forget sounds via macOS `say` + `afplay`.
/// All playback is non-blocking — we spawn detached children and forget them.
pub struct Audio {
    chugga: PathBuf,
    choo: PathBuf,
    horn: PathBuf,
    yay: PathBuf,
    nope: PathBuf,

    chugga_child: Option<Child>,
    horn_child: Option<Child>,
    speak_child: Option<Child>,

    last_chugga: Option<Instant>,
    chug_step: u32,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let cache = cache_dir()?;
        std::fs::create_dir_all(&cache).ok()?;

        let chugga = cache.join("chugga.aiff");
        let choo = cache.join("choo.aiff");
        let horn = cache.join("horn.aiff");
        let yay = cache.join("yay.aiff");
        let nope = cache.join("nope.aiff");

        // Voice picks: a kid-leaning voice for the encouragement, a low rumble for chugga.
        // Falling back silently if `say` isn't available means we just won't have audio.
        synth(&chugga, "chuh guh", Some("Fred"), Some(220))?;
        synth(&choo, "choo choo", Some("Whisper"), Some(180))?;
        synth(&horn, "toot toot", Some("Bahh"), Some(180))?;
        synth(&yay, "yay! good job!", Some("Junior"), Some(220))?;
        synth(&nope, "uh oh", Some("Junior"), Some(200))?;

        Some(Self {
            chugga,
            choo,
            horn,
            yay,
            nope,
            chugga_child: None,
            horn_child: None,
            speak_child: None,
            last_chugga: None,
            chug_step: 0,
        })
    }

    /// Called every tick. If the train is moving, advance the chugga rhythm.
    /// Plays "chugga" 8 times then "choo choo" twice, on a steady cadence.
    pub fn tick_chugga(&mut self, moving: bool) {
        // Reap finished children so we don't leak file descriptors.
        if let Some(c) = self.chugga_child.as_mut()
            && matches!(c.try_wait(), Ok(Some(_)))
        {
            self.chugga_child = None;
        }
        if !moving {
            return;
        }
        let interval = Duration::from_millis(320);
        let now = Instant::now();
        let due = self.last_chugga.map_or(true, |t| now.duration_since(t) >= interval);
        if !due {
            return;
        }
        // Don't pile up: if the previous chugga is still playing, skip.
        if self.chugga_child.is_some() {
            return;
        }
        let path = if self.chug_step % 10 < 8 { &self.chugga } else { &self.choo };
        self.chugga_child = spawn_afplay(path);
        self.last_chugga = Some(now);
        self.chug_step = self.chug_step.wrapping_add(1);
    }

    pub fn horn(&mut self) {
        if let Some(c) = self.horn_child.as_mut()
            && matches!(c.try_wait(), Ok(Some(_)))
        {
            self.horn_child = None;
        }
        if self.horn_child.is_none() {
            self.horn_child = spawn_afplay(&self.horn);
        }
    }

    pub fn yay(&mut self) {
        let _ = spawn_afplay(&self.yay);
    }

    pub fn nope(&mut self) {
        let _ = spawn_afplay(&self.nope);
    }

    /// Speak arbitrary text (e.g. the current target word) out loud, so a
    /// pre-reading toddler can hear what to type. Cancels any prior speech.
    pub fn speak(&mut self, text: &str) {
        if let Some(c) = self.speak_child.as_mut() {
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
        for c in [&mut self.chugga_child, &mut self.horn_child, &mut self.speak_child]
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

fn synth(path: &PathBuf, text: &str, voice: Option<&str>, rate: Option<u32>) -> Option<()> {
    if path.exists() {
        return Some(());
    }
    let mut cmd = Command::new("say");
    if let Some(v) = voice {
        cmd.arg("-v").arg(v);
    }
    if let Some(r) = rate {
        cmd.arg("-r").arg(r.to_string());
    }
    cmd.arg("-o").arg(path).arg(text);
    let status = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status().ok()?;
    if status.success() { Some(()) } else { None }
}

fn spawn_afplay(path: &PathBuf) -> Option<Child> {
    Command::new("afplay")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

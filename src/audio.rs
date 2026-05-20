use std::io::Cursor;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

const CHUGGA: &[u8] = include_bytes!("../assets/chugga.flac");
const WHISTLE: &[u8] = include_bytes!("../assets/whistle.flac");
const ANOTHER_WHEEL: &[u8] = include_bytes!("../assets/another_wheel.flac");

pub struct Audio {
    sink: MixerDeviceSink,
    chugga: Player,
    chugga_playing: bool,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let sink = DeviceSinkBuilder::open_default_sink().ok()?;
        let chugga = Player::connect_new(sink.mixer());
        let source = Decoder::try_from(Cursor::new(CHUGGA))
            .ok()?
            .repeat_infinite();
        chugga.append(source);
        chugga.set_volume(0.7);
        chugga.pause();
        Some(Self {
            sink,
            chugga,
            chugga_playing: false,
        })
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

    pub fn horn(&mut self) {
        self.play_oneshot(WHISTLE, 1.0);
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

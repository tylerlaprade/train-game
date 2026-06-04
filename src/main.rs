use std::fmt;
use std::io;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{
    self, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
    EnableFocusChange, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};

mod audio;
mod game;
mod renderer;

use game::Game;

const FRAME_DURATION: Duration = Duration::from_millis(33);
const UNLOCK_SEQUENCES: &[&str] = &["quit", "exit"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HideMousePointerWhileTyping;

impl crossterm::Command for HideMousePointerWhileTyping {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\x1b[>2p")
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RestoreMousePointerMode;

impl crossterm::Command for RestoreMousePointerMode {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\x1b[>p")
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

#[derive(Default)]
struct InputState {
    forward_held: bool,
    backward_held: bool,
    last_direction: Option<Direction>,
}

impl InputState {
    fn press(&mut self, direction: Direction) {
        match direction {
            Direction::Forward => self.forward_held = true,
            Direction::Backward => self.backward_held = true,
        }
        self.last_direction = Some(direction);
    }

    fn release(&mut self, direction: Direction) {
        match direction {
            Direction::Forward => self.forward_held = false,
            Direction::Backward => self.backward_held = false,
        }

        if self.last_direction == Some(direction) {
            self.last_direction = match direction {
                Direction::Forward if self.backward_held => Some(Direction::Backward),
                Direction::Backward if self.forward_held => Some(Direction::Forward),
                _ => None,
            };
        }
    }

    fn direction(&self) -> Option<Direction> {
        match self.last_direction {
            Some(Direction::Forward) if self.forward_held => Some(Direction::Forward),
            Some(Direction::Backward) if self.backward_held => Some(Direction::Backward),
            _ if self.forward_held => Some(Direction::Forward),
            _ if self.backward_held => Some(Direction::Backward),
            _ => None,
        }
    }

    fn drive(&self, game: &mut Game) {
        match self.direction() {
            Some(Direction::Forward) => game.nudge_forward(),
            Some(Direction::Backward) => game.nudge_backward(),
            None => {}
        }
    }
}

#[derive(Default)]
struct UnlockState {
    matched: usize,
}

impl UnlockState {
    fn push(&mut self, ch: char) -> bool {
        let ch = ch.to_ascii_lowercase();

        for sequence in UNLOCK_SEQUENCES {
            if sequence.len() > self.matched && sequence.as_bytes()[self.matched] as char == ch {
                self.matched += 1;
                if self.matched == sequence.len() {
                    self.matched = 0;
                    return true;
                }
                return false;
            }
        }

        self.matched = UNLOCK_SEQUENCES
            .iter()
            .filter(|sequence| sequence.as_bytes()[0] as char == ch)
            .map(|_| 1)
            .next()
            .unwrap_or(0);
        false
    }
}

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--check-sprites") {
        renderer::debug_check_sprite_widths();
        println!("sprite widths ok");
        return Ok(());
    }
    if std::env::args().any(|a| a == "--smoke") {
        return smoke_test();
    }

    let (cols, rows) = size().unwrap_or((80, 24));
    let mut game = Game::new(cols, rows);
    let mut audio = audio::Audio::new();
    let mut renderer = renderer::Renderer::new();

    let mut stdout = io::BufWriter::new(io::stdout());
    enable_raw_mode()?;
    let mut report_key_releases = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        EnableMouseCapture,
        HideMousePointerWhileTyping,
        EnableBracketedPaste,
        EnableFocusChange,
        Clear(ClearType::All)
    )?;
    if report_key_releases {
        report_key_releases = execute!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )
        .is_ok();
    }

    let result = run(
        &mut game,
        &mut audio,
        &mut renderer,
        &mut stdout,
        report_key_releases,
    );

    if report_key_releases {
        execute!(stdout, PopKeyboardEnhancementFlags)?;
    }
    execute!(
        stdout,
        DisableBracketedPaste,
        DisableFocusChange,
        DisableMouseCapture,
        RestoreMousePointerMode,
        Show,
        LeaveAlternateScreen
    )?;
    disable_raw_mode()?;

    result
}

fn smoke_test() -> io::Result<()> {
    let mut game = Game::new(200, 40);
    let mut buf: Vec<u8> = Vec::new();
    let mut renderer = renderer::Renderer::new();

    // 1) Force wrap-arounds to exercise the car-on-wrap path.
    let mut added = 0_u32;
    for _ in 0..30 {
        game.head_x = game.cycle() as f32 + 1.0;
        game.nudge_forward();
        added += game.tick();
        renderer.render(&game, &mut buf)?;
        buf.clear();
    }

    // 2) Run a normal-time loop to exercise tick() + smoke physics.
    game.nudge_forward();
    for _ in 0..200 {
        game.tick();
        renderer.render(&game, &mut buf)?;
        buf.clear();
    }

    println!(
        "smoke ok — cars={} added_by_wraps={} cycle={} (train_w={}, screen_w=200) smoke={}",
        game.cars.len(),
        added,
        game.cycle(),
        game.train_total_width(),
        game.smoke.len()
    );
    Ok(())
}

fn run(
    game: &mut Game,
    audio: &mut Option<audio::Audio>,
    renderer: &mut renderer::Renderer,
    stdout: &mut impl io::Write,
    report_key_releases: bool,
) -> io::Result<()> {
    let mut last_frame = Instant::now();
    let mut input = InputState::default();
    let mut unlock = UnlockState::default();

    loop {
        if game.quit {
            return Ok(());
        }

        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(k) => {
                    handle_key(k, game, audio, &mut input, &mut unlock, report_key_releases)
                }
                Event::Resize(c, r) => {
                    game.resize(c, r);
                    execute!(stdout, Clear(ClearType::All))?;
                }
                _ => {}
            }
        }

        if report_key_releases {
            input.drive(game);
        }

        let cars_added = game.tick();
        if let Some(a) = audio.as_mut() {
            a.tick_chugga(game.moving_recently());
            a.set_engine_pan(game.engine_pan());
            a.tick_rain(
                renderer::weather_state(
                    game.distance_traveled,
                    game.started_at.elapsed().as_secs_f32(),
                )
                .rain_audio_intensity(),
            );
            if cars_added > 0 {
                a.another_wheel();
            }
        }
        renderer.render(game, stdout)?;

        let elapsed = last_frame.elapsed();
        if elapsed < FRAME_DURATION {
            let remaining = FRAME_DURATION - elapsed;
            let _ = event::poll(remaining)?;
        }
        last_frame = Instant::now();
    }
}

fn handle_key(
    k: crossterm::event::KeyEvent,
    game: &mut Game,
    audio: &mut Option<audio::Audio>,
    input: &mut InputState,
    unlock: &mut UnlockState,
    report_key_releases: bool,
) {
    if k.kind == KeyEventKind::Release {
        if report_key_releases {
            match k.code {
                KeyCode::Right => input.release(Direction::Forward),
                KeyCode::Left => input.release(Direction::Backward),
                _ => {}
            }
        }
        return;
    }

    match k.code {
        KeyCode::Right => {
            if report_key_releases {
                input.press(Direction::Forward);
            }
            game.nudge_forward();
        }
        KeyCode::Left => {
            if report_key_releases {
                input.press(Direction::Backward);
            }
            game.nudge_backward();
        }
        KeyCode::Char(' ') => {
            if let Some(a) = audio.as_mut() {
                a.horn();
            }
        }
        KeyCode::Char(ch)
            if k.modifiers.contains(KeyModifiers::CONTROL) && ch.eq_ignore_ascii_case(&'b') =>
        {
            game.skip_to_next_biome_transition();
        }
        KeyCode::Char(ch) => {
            if unlock.push(ch) {
                game.quit = true;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Direction, HideMousePointerWhileTyping, InputState, RestoreMousePointerMode, UnlockState,
        handle_key,
    };
    use crate::game::{BIOME_BLEND_START_DISTANCE, BIOME_DEBUG_SKIP_MARGIN, Game};
    use crossterm::Command;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn held_arrow_survives_unrelated_key_release() {
        let mut input = InputState::default();

        input.press(Direction::Forward);
        input.release(Direction::Backward);

        assert_eq!(input.direction(), Some(Direction::Forward));
    }

    #[test]
    fn releasing_active_arrow_falls_back_to_other_held_arrow() {
        let mut input = InputState::default();

        input.press(Direction::Forward);
        input.press(Direction::Backward);
        input.release(Direction::Backward);

        assert_eq!(input.direction(), Some(Direction::Forward));
    }

    #[test]
    fn secret_unlock_sequence_quits_only_after_full_match() {
        let mut unlock = UnlockState::default();

        assert!(!unlock.push('q'));
        assert!(!unlock.push('u'));
        assert!(!unlock.push('i'));
        assert!(unlock.push('t'));
    }

    #[test]
    fn alternate_secret_unlock_sequence_also_quits() {
        let mut unlock = UnlockState::default();

        assert!(!unlock.push('e'));
        assert!(!unlock.push('x'));
        assert!(!unlock.push('i'));
        assert!(unlock.push('t'));
    }

    #[test]
    fn secret_unlock_sequence_resets_after_wrong_key() {
        let mut unlock = UnlockState::default();

        assert!(!unlock.push('p'));
        assert!(!unlock.push('x'));
        for ch in "uit".chars() {
            assert!(!unlock.push(ch));
        }
    }

    #[test]
    fn ctrl_b_skips_to_biome_transition_debug_point() {
        let mut game = Game::new(200, 40);
        let mut audio = None;
        let mut input = InputState::default();
        let mut unlock = UnlockState::default();

        handle_key(
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL),
            &mut game,
            &mut audio,
            &mut input,
            &mut unlock,
            false,
        );

        assert!(
            (game.distance_traveled - (BIOME_BLEND_START_DISTANCE - BIOME_DEBUG_SKIP_MARGIN)).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn mouse_pointer_mode_commands_use_xtsmpointer_sequences() {
        let mut hide = String::new();
        HideMousePointerWhileTyping.write_ansi(&mut hide).unwrap();
        assert_eq!(hide, "\x1b[>2p");

        let mut restore = String::new();
        RestoreMousePointerMode.write_ansi(&mut restore).unwrap();
        assert_eq!(restore, "\x1b[>p");
    }
}

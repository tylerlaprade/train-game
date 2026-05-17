use std::io;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};

mod audio;
mod game;
mod renderer;
mod words;

use game::{Game, WordOutcome};

const FRAME_DURATION: Duration = Duration::from_millis(33);

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--check-sprites") {
        renderer::debug_check_sprite_widths();
        println!("sprite widths ok");
        return Ok(());
    }
    if std::env::args().any(|a| a == "--smoke") {
        return smoke_test();
    }
    if std::env::args().any(|a| a == "--audio-setup") {
        return audio_setup_check();
    }

    let (cols, rows) = size().unwrap_or((80, 24));
    let mut game = Game::new(cols, rows);
    let mut audio = audio::Audio::new();

    if let Some(a) = audio.as_mut() {
        a.speak(&game.target_word);
    }

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    let result = run(&mut game, &mut audio, &mut stdout);

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

fn run(
    game: &mut Game,
    audio: &mut Option<audio::Audio>,
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    let mut last_frame = Instant::now();

    loop {
        if game.quit {
            return Ok(());
        }

        // Drain events that arrived since last frame.
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(k) if k.kind == KeyEventKind::Release => {}
                Event::Key(k) => handle_key(k, game, audio),
                Event::Resize(c, r) => {
                    game.resize(c, r);
                    execute!(stdout, Clear(ClearType::All))?;
                }
                _ => {}
            }
        }

        game.tick();
        if let Some(a) = audio.as_mut() {
            a.tick_chugga(game.moving_recently());
        }
        renderer::render(game, stdout)?;

        // Pace the loop. Sleep via event::poll so input still wakes us early.
        let elapsed = last_frame.elapsed();
        if elapsed < FRAME_DURATION {
            let remaining = FRAME_DURATION - elapsed;
            let _ = event::poll(remaining)?;
        }
        last_frame = Instant::now();
    }
}

fn audio_setup_check() -> io::Result<()> {
    let started = Instant::now();
    let _a = audio::Audio::new();
    let elapsed = started.elapsed();
    let cache = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join("Library/Caches/train-game"))
        .unwrap();
    println!("audio setup took {:?}", elapsed);
    if cache.exists() {
        println!("cache files in {}:", cache.display());
        for entry in std::fs::read_dir(&cache)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            println!("  {:>8} bytes  {}", meta.len(), entry.file_name().to_string_lossy());
        }
    } else {
        println!("cache dir does not exist");
    }
    Ok(())
}

fn smoke_test() -> io::Result<()> {
    let mut game = Game::new(120, 40);
    // 40 word completions = 20 cars across multiple trains (2 wheels per car,
    // 8 cars per train). Exercises wrap-around with a longer-than-screen train
    // and the multi-train layout.
    for _ in 0..40 {
        for ch in game.target_word.clone().chars() {
            let _ = game.handle_letter(ch);
        }
    }
    game.nudge_forward();
    let mut buf: Vec<u8> = Vec::new();
    for _ in 0..200 {
        game.tick();
        renderer::render(&game, &mut buf)?;
        buf.clear();
    }
    let total_cars: usize = game.trains.iter().map(|t| t.cars.len()).sum();
    println!(
        "smoke ok — head_x={:.1} cycle={} trains={} cars={} wheels={} smoke_particles={}",
        game.head_x,
        game.cycle(),
        game.trains.len(),
        total_cars,
        game.total_wheels(),
        game.smoke.len()
    );
    Ok(())
}

fn handle_key(
    k: crossterm::event::KeyEvent,
    game: &mut Game,
    audio: &mut Option<audio::Audio>,
) {
    if k.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(k.code, KeyCode::Char('c' | 'C'))
    {
        game.quit = true;
        return;
    }

    match k.code {
        KeyCode::Esc => game.quit = true,
        KeyCode::Right => game.nudge_forward(),
        KeyCode::Left => game.nudge_backward(),
        KeyCode::Char(' ') => {
            game.horn();
            if let Some(a) = audio.as_mut() {
                a.horn();
            }
        }
        KeyCode::Char('q' | 'Q') => game.quit = true,
        KeyCode::Char(c) if c.is_ascii_alphabetic() => {
            let outcome = game.handle_letter(c);
            match outcome {
                WordOutcome::Correct | WordOutcome::Maxed => {
                    if let Some(a) = audio.as_mut() {
                        a.wuvva_wheel();
                    }
                    // Delay announcing the next word so "wuvva wheel" isn't cut off.
                    let next_word = game.target_word.clone();
                    std::thread::Builder::new()
                        .spawn(move || {
                            std::thread::sleep(Duration::from_millis(1100));
                            let _ = std::process::Command::new("say")
                                .arg("-v")
                                .arg("Junior")
                                .arg("-r")
                                .arg("180")
                                .arg(next_word)
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();
                        })
                        .ok();
                }
                WordOutcome::Wrong => {
                    if let Some(a) = audio.as_mut() {
                        a.nope();
                        a.speak(&game.target_word);
                    }
                }
                WordOutcome::Progress => {}
            }
        }
        _ => {}
    }
}

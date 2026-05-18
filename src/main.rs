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

use game::Game;

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

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    let result = run(&mut game, &mut audio, &mut stdout);

    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
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
    let mut game = Game::new(200, 40);
    let mut buf: Vec<u8> = Vec::new();

    // 1) Force wrap-arounds to exercise the car-on-wrap path.
    let mut added = 0_u32;
    for _ in 0..30 {
        game.head_x = game.cycle() as f32 + 1.0;
        game.nudge_forward();
        added += game.tick();
        renderer::render(&game, &mut buf)?;
        buf.clear();
    }

    // 2) Run a normal-time loop to exercise tick() + smoke physics.
    game.nudge_forward();
    for _ in 0..200 {
        game.tick();
        renderer::render(&game, &mut buf)?;
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
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    let mut last_frame = Instant::now();

    loop {
        if game.quit {
            return Ok(());
        }

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

        let cars_added = game.tick();
        if let Some(a) = audio.as_mut() {
            a.tick_chugga(game.moving_recently());
            if cars_added > 0 {
                a.wuvva_wheel();
            }
        }
        renderer::render(game, stdout)?;

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
) {
    if k.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(k.code, KeyCode::Char('c' | 'C'))
    {
        game.quit = true;
        return;
    }

    match k.code {
        KeyCode::Esc | KeyCode::Char('q' | 'Q') => game.quit = true,
        KeyCode::Right => game.nudge_forward(),
        KeyCode::Left => game.nudge_backward(),
        KeyCode::Char(' ') => {
            game.horn();
            if let Some(a) = audio.as_mut() {
                a.horn();
            }
        }
        _ => {}
    }
}

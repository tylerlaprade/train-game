use std::io::Write;

use crossterm::cursor;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::game::{CarColor, CarKind, Game, SEG_HEIGHT, TRAIN_TOP_ROW_FROM_TOP};

pub struct Sprite {
    pub width: usize,
    pub rows: &'static [&'static str],
}

// ── ENGINE ───────────────────────────────────────────────────────────────────
// Width 40. Smokestack centered around col 25-30. Body is dark grey; sprite-
// special chars (O, =, *, ░, |, _) get their own colors via `char_color`.
pub const ENGINE: Sprite = Sprite {
    width: 40,
    rows: &[
        "                         ____           ",
        "                        |    |          ",
        "                        |    |          ",
        "              __________|____|__________",
        "             /  ___                    \\",
        "            |  |░░░|  STEAM  ___   *   |",
        "            |  |___|________|HL_|______|",
        "             ==(O)====(O)====(O)====(O)=",
    ],
};

// ── CABOOSE ──────────────────────────────────────────────────────────────────
// Width 22. Always red. Has a cupola on top (the little hat where the
// brakeman watches the train from).
pub const CABOOSE: Sprite = Sprite {
    width: 22,
    rows: &[
        "                      ",
        "        _______       ",
        "       | ░ ░ ░ |      ",
        "    ___|_______|___   ",
        "   |   ___       ___| ",
        "   |  |░░░|     |░░░| ",
        "   |__|___|_____|___|_",
        "    =(O)====(O)====(O)",
    ],
};

// ── BOXCAR ───────────────────────────────────────────────────────────────────
pub const BOXCAR: Sprite = Sprite {
    width: 18,
    rows: &[
        "                  ",
        "                  ",
        "                  ",
        "  ________________",
        " | TRAIN  CO.    |",
        " |  ____    ____ |",
        " |_|░░░░|__|░░░░|_",
        " =(O)========(O)==",
    ],
};

// ── TANKER ───────────────────────────────────────────────────────────────────
pub const TANKER: Sprite = Sprite {
    width: 18,
    rows: &[
        "                  ",
        "                  ",
        "     __________   ",
        "    /          \\  ",
        "   |    FUEL    | ",
        "    \\__________/  ",
        " _______________ _",
        " =(O)========(O)==",
    ],
};

// ── HOPPER (open coal/grain car) ─────────────────────────────────────────────
pub const HOPPER: Sprite = Sprite {
    width: 18,
    rows: &[
        "                  ",
        "                  ",
        "                  ",
        "  ________________",
        " |░░░░░░░░░░░░░░░|",
        " |░░░░ COAL ░░░░░|",
        "  \\___________/__ ",
        " =(O)========(O)==",
    ],
};

// ── PASSENGER ────────────────────────────────────────────────────────────────
pub const PASSENGER: Sprite = Sprite {
    width: 18,
    rows: &[
        "                  ",
        "                  ",
        "                  ",
        "  ________________",
        " |░░ ░░ ░░ ░░ ░░|",
        " |              |",
        " |______________|_",
        " =(O)========(O)==",
    ],
};

// ── FLATCAR ──────────────────────────────────────────────────────────────────
pub const FLATCAR: Sprite = Sprite {
    width: 18,
    rows: &[
        "                  ",
        "                  ",
        "                  ",
        "                  ",
        "                  ",
        "                  ",
        "  ________________",
        " =(O)========(O)==",
    ],
};

// ── GONDOLA (low open car) ───────────────────────────────────────────────────
pub const GONDOLA: Sprite = Sprite {
    width: 18,
    rows: &[
        "                  ",
        "                  ",
        "                  ",
        "                  ",
        "  ________________",
        " |              |",
        " |______________|_",
        " =(O)========(O)==",
    ],
};

pub fn car_sprite(kind: CarKind) -> &'static Sprite {
    match kind {
        CarKind::Boxcar => &BOXCAR,
        CarKind::Tanker => &TANKER,
        CarKind::Hopper => &HOPPER,
        CarKind::Passenger => &PASSENGER,
        CarKind::Flatcar => &FLATCAR,
        CarKind::Gondola => &GONDOLA,
    }
}

pub fn car_color_to_term(c: CarColor) -> Color {
    match c {
        CarColor::Yellow => Color::Yellow,
        CarColor::Green => Color::Green,
        CarColor::Cyan => Color::Cyan,
        CarColor::Blue => Color::Blue,
        CarColor::Magenta => Color::Magenta,
        CarColor::Orange => Color::Rgb { r: 220, g: 130, b: 30 },
        CarColor::Brown => Color::Rgb { r: 150, g: 90, b: 50 },
        CarColor::Olive => Color::Rgb { r: 130, g: 130, b: 60 },
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct CellFmt {
    ch: char,
    fg: Color,
    bg: Color,
}

const BLANK: CellFmt = CellFmt {
    ch: ' ',
    fg: Color::Reset,
    bg: Color::Reset,
};

pub fn render(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    let cols = game.screen_cols as usize;
    let rows = game.screen_rows as usize;
    if cols < 20 || rows < 16 {
        queue!(
            out,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            Print("Make the terminal a bit bigger (≥ 60×20) for the train!")
        )?;
        out.flush()?;
        return Ok(());
    }

    let mut grid = vec![BLANK; cols * rows];
    let idx = |r: usize, c: usize| r * cols + c;

    // Sky background (sky-blue tint) for rows above the train.
    let train_top = TRAIN_TOP_ROW_FROM_TOP.min(rows.saturating_sub(SEG_HEIGHT + 4));
    for r in 0..train_top {
        for c in 0..cols {
            grid[idx(r, c)] = CellFmt {
                ch: ' ',
                fg: Color::Reset,
                bg: Color::Rgb { r: 90, g: 150, b: 200 },
            };
        }
    }

    // Ground below the train.
    let ground_row = train_top + SEG_HEIGHT;
    if ground_row < rows {
        for c in 0..cols {
            grid[idx(ground_row, c)] = CellFmt {
                ch: '~',
                fg: Color::Rgb { r: 80, g: 60, b: 30 },
                bg: Color::Rgb { r: 60, g: 100, b: 40 },
            };
        }
    }
    if ground_row + 1 < rows {
        for c in 0..cols {
            grid[idx(ground_row + 1, c)] = CellFmt {
                ch: ' ',
                fg: Color::Reset,
                bg: Color::Rgb { r: 60, g: 100, b: 40 },
            };
        }
    }

    draw_train(&mut grid, cols, rows, train_top, game);
    draw_smoke(&mut grid, cols, rows, game);
    draw_word_ui(&mut grid, cols, rows, game);
    draw_help(&mut grid, cols, rows, game);

    // Flush grid to terminal.
    queue!(out, cursor::MoveTo(0, 0))?;
    let mut cur_fg = Color::Reset;
    let mut cur_bg = Color::Reset;
    queue!(out, ResetColor)?;
    for r in 0..rows {
        queue!(out, cursor::MoveTo(0, r as u16))?;
        for c in 0..cols {
            let cell = grid[idx(r, c)];
            if cell.fg != cur_fg {
                queue!(out, SetForegroundColor(cell.fg))?;
                cur_fg = cell.fg;
            }
            if cell.bg != cur_bg {
                queue!(out, SetBackgroundColor(cell.bg))?;
                cur_bg = cell.bg;
            }
            queue!(out, Print(cell.ch))?;
        }
    }
    queue!(out, ResetColor)?;
    out.flush()?;
    Ok(())
}

fn draw_train(grid: &mut [CellFmt], cols: usize, rows: usize, train_top: usize, game: &Game) {
    let cycle = game.cycle();
    let head_x = game.head_x.floor() as i32;
    // Wrap into [0, cycle) defensively, even though game.tick already does this.
    let head_x = head_x.rem_euclid(cycle.max(1));

    // Draw the train at three offsets so wrap-around is seamless on both sides.
    for shift in [-cycle, 0, cycle] {
        let mut right = head_x + shift;

        let engine = &ENGINE;
        let left = right - (engine.width as i32 - 1);
        draw_sprite(grid, cols, rows, engine, Color::Rgb { r: 60, g: 60, b: 70 }, left, train_top);
        right = left - 1;

        for car in &game.middle_cars {
            let sprite = car_sprite(car.kind);
            let left = right - (sprite.width as i32 - 1);
            draw_sprite(grid, cols, rows, sprite, car_color_to_term(car.color), left, train_top);
            right = left - 1;
        }

        let caboose = &CABOOSE;
        let left = right - (caboose.width as i32 - 1);
        draw_sprite(grid, cols, rows, caboose, Color::Red, left, train_top);
    }

    // Horn flash: a "TOOT!" speech bubble above the engine front.
    if game.horn_active() {
        let bubble = "TOOT TOOT!";
        let engine_front_world_x = head_x;
        // Try both wrap copies.
        for shift in [-cycle, 0, cycle] {
            let x = engine_front_world_x + shift - bubble.len() as i32;
            let y = train_top.saturating_sub(2);
            if y < rows {
                put_text(grid, cols, rows, bubble, x, y, Color::White, Color::Rgb { r: 200, g: 30, b: 30 });
            }
        }
    }
}

fn draw_sprite(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    sprite: &Sprite,
    base: Color,
    left_x: i32,
    top_y: usize,
) {
    for (r_off, row) in sprite.rows.iter().enumerate() {
        let y = top_y + r_off;
        if y >= rows {
            break;
        }
        for (c_off, ch) in row.chars().enumerate() {
            if c_off >= sprite.width {
                break;
            }
            let x = left_x + c_off as i32;
            if x < 0 || x >= cols as i32 {
                continue;
            }
            if ch == ' ' {
                continue;
            }
            let (rendered, fg) = char_visual(ch, base);
            grid[y as usize * cols + x as usize] = CellFmt {
                ch: rendered,
                fg,
                bg: bg_under_train(y, top_y),
            };
        }
    }
}

fn bg_under_train(y: usize, train_top: usize) -> Color {
    if y < train_top {
        Color::Rgb { r: 90, g: 150, b: 200 }
    } else if y >= train_top + SEG_HEIGHT {
        Color::Rgb { r: 60, g: 100, b: 40 }
    } else {
        Color::Reset
    }
}

fn char_visual(ch: char, base: Color) -> (char, Color) {
    match ch {
        'O' => ('O', Color::Black),
        '(' | ')' => (ch, Color::DarkGrey),
        '=' => ('=', Color::Rgb { r: 90, g: 70, b: 50 }),
        '*' => ('*', Color::Yellow),
        '░' => ('░', Color::Rgb { r: 220, g: 230, b: 240 }),
        '▒' => ('▒', Color::White),
        '|' | '_' | '/' | '\\' => (ch, base),
        _ => (ch, base),
    }
}

fn draw_smoke(grid: &mut [CellFmt], cols: usize, rows: usize, game: &Game) {
    for s in &game.smoke {
        let x = s.x.round() as i32;
        let y = s.y.round() as i32;
        if y < 0 || y as usize >= rows.min(crate::game::TRAIN_TOP_ROW_FROM_TOP) {
            continue;
        }
        let (ch, fg) = smoke_visual(s.age);
        // Smoke may be drawn at any wrap-copy of its world x.
        let cycle = game.cycle();
        for shift in [-cycle, 0, cycle] {
            let xs = x + shift;
            if xs >= 0 && (xs as usize) < cols {
                let i = y as usize * cols + xs as usize;
                grid[i] = CellFmt {
                    ch,
                    fg,
                    bg: Color::Rgb { r: 90, g: 150, b: 200 },
                };
            }
        }
    }
}

fn smoke_visual(age: f32) -> (char, Color) {
    let bright = if age < 0.4 {
        ('@', Color::Rgb { r: 250, g: 250, b: 250 })
    } else if age < 0.9 {
        ('%', Color::Rgb { r: 220, g: 220, b: 220 })
    } else if age < 1.6 {
        ('o', Color::Rgb { r: 180, g: 180, b: 180 })
    } else if age < 2.3 {
        ('.', Color::Rgb { r: 140, g: 140, b: 140 })
    } else {
        ('.', Color::Rgb { r: 110, g: 110, b: 110 })
    };
    bright
}

fn put_text(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    text: &str,
    left_x: i32,
    y: usize,
    fg: Color,
    bg: Color,
) {
    if y >= rows {
        return;
    }
    for (i, ch) in text.chars().enumerate() {
        let x = left_x + i as i32;
        if x < 0 || x >= cols as i32 {
            continue;
        }
        grid[y * cols + x as usize] = CellFmt { ch, fg, bg };
    }
}

fn draw_word_ui(grid: &mut [CellFmt], cols: usize, rows: usize, game: &Game) {
    let bottom_panel_top = rows.saturating_sub(5);

    // Background bar for the word area.
    let bar_bg = if game.celebrating() {
        Color::Rgb { r: 30, g: 120, b: 30 }
    } else if game.wrong_flash() {
        Color::Rgb { r: 120, g: 30, b: 30 }
    } else {
        Color::Rgb { r: 25, g: 25, b: 35 }
    };
    for r in bottom_panel_top..rows {
        for c in 0..cols {
            grid[r * cols + c] = CellFmt { ch: ' ', fg: Color::White, bg: bar_bg };
        }
    }

    // "Type the word:"
    let prompt = if game.celebrating() {
        format!("  🎉 YAY!  New car!  Cars: {}/{}", game.middle_cars.len(), crate::game::MAX_MIDDLE_CARS)
    } else if game.middle_cars.len() >= crate::game::MAX_MIDDLE_CARS {
        format!("  All {} cars added! Type words to keep going!", crate::game::MAX_MIDDLE_CARS)
    } else {
        format!("  Type the word:  ({}/{} cars)", game.middle_cars.len(), crate::game::MAX_MIDDLE_CARS)
    };
    put_text(grid, cols, rows, &prompt, 2, bottom_panel_top, Color::White, bar_bg);

    // Render the target word in big letters using the block font below.
    let big_y = bottom_panel_top + 1;
    let letters: Vec<char> = game.target_word.chars().collect();
    let typed_len = game.typed.len();
    let mut x_cursor = 4_i32;
    for (i, ch) in letters.iter().enumerate() {
        let done = i < typed_len;
        let color = if done { Color::Green } else { Color::Yellow };
        x_cursor = draw_big_letter(grid, cols, rows, *ch, x_cursor, big_y, color, bar_bg);
        x_cursor += 1; // spacing between letters
    }

    // Show typed-so-far inline next to the big word, in smaller font.
    let typed_label = format!("you typed: {}", game.typed);
    if big_y + 2 < rows {
        put_text(grid, cols, rows, &typed_label, 4, big_y + 3, Color::Cyan, bar_bg);
    }
}

fn draw_help(grid: &mut [CellFmt], cols: usize, rows: usize, _game: &Game) {
    // Single help line at the very top, sky-tinted background already.
    let help = " ← / → drive    SPACE toot    type letters to add a car    ESC quit";
    let bg = Color::Rgb { r: 90, g: 150, b: 200 };
    put_text(grid, cols, rows, help, 0, 0, Color::White, bg);
}

/// 3-wide × 3-tall block font for the big target word. Returns x after drawing.
fn draw_big_letter(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    ch: char,
    x: i32,
    y: usize,
    fg: Color,
    bg: Color,
) -> i32 {
    let glyph = block_glyph(ch);
    let w = 4; // each glyph is 4 cols wide for breathing room
    for (r_off, line) in glyph.iter().enumerate() {
        if y + r_off >= rows {
            break;
        }
        for (c_off, ch) in line.chars().enumerate() {
            if c_off >= w {
                break;
            }
            let xx = x + c_off as i32;
            if xx < 0 || xx >= cols as i32 {
                continue;
            }
            grid[(y + r_off) * cols + xx as usize] = CellFmt { ch, fg, bg };
        }
    }
    x + w as i32
}

fn block_glyph(ch: char) -> &'static [&'static str] {
    match ch.to_ascii_uppercase() {
        'A' => &[" ▄▄ ", "█▀▀█", "█  █"],
        'B' => &["█▀▀▄", "█▀▀▄", "█▄▄▀"],
        'C' => &[" ▄▄▄", "█   ", " ▀▀▀"],
        'D' => &["█▀▀▄", "█  █", "█▄▄▀"],
        'E' => &["█▀▀▀", "█▀▀ ", "█▄▄▄"],
        'F' => &["█▀▀▀", "█▀▀ ", "█   "],
        'G' => &[" ▄▄▄", "█  █", " ▀▀█"],
        'H' => &["█  █", "█▀▀█", "█  █"],
        'I' => &["▀█▀ ", " █  ", "▄█▄ "],
        'J' => &["  █ ", "  █ ", "█▄█ "],
        'K' => &["█ █ ", "█▀  ", "█ █ "],
        'L' => &["█   ", "█   ", "█▄▄▄"],
        'M' => &["█▄ █", "█▀█ ", "█  █"],
        'N' => &["█▄ █", "█▀▄█", "█  █"],
        'O' => &[" ▄▄ ", "█  █", " ▀▀ "],
        'P' => &["█▀▀▄", "█▀▀ ", "█   "],
        'Q' => &[" ▄▄ ", "█  █", " ▀▀▄"],
        'R' => &["█▀▀▄", "█▀█ ", "█ █ "],
        'S' => &[" ▄▄▄", " ▀▀▄", "▄▄▄▀"],
        'T' => &["▀█▀▀", " █  ", " █  "],
        'U' => &["█  █", "█  █", " ▀▀ "],
        'V' => &["█  █", "█  █", " ▀▀ "],
        'W' => &["█  █", "█▄▄█", "█▀▀█"],
        'X' => &["█ █ ", " ▀  ", "█ █ "],
        'Y' => &["█ █ ", " █  ", " █  "],
        'Z' => &["▀▀█ ", " ▄▀ ", "█▄▄ "],
        _ => &["    ", "    ", "    "],
    }
}

#[allow(dead_code)]
pub fn debug_check_sprite_widths() {
    for (name, sprite) in [
        ("ENGINE", &ENGINE),
        ("CABOOSE", &CABOOSE),
        ("BOXCAR", &BOXCAR),
        ("TANKER", &TANKER),
        ("HOPPER", &HOPPER),
        ("PASSENGER", &PASSENGER),
        ("FLATCAR", &FLATCAR),
        ("GONDOLA", &GONDOLA),
    ] {
        for (i, row) in sprite.rows.iter().enumerate() {
            let n = row.chars().count();
            assert!(
                n <= sprite.width,
                "{} row {} too wide: {} chars vs width {}\n{:?}",
                name,
                i,
                n,
                sprite.width,
                row
            );
        }
        assert_eq!(
            sprite.rows.len(),
            SEG_HEIGHT,
            "{} has wrong height: {} vs {}",
            name,
            sprite.rows.len(),
            SEG_HEIGHT
        );
    }
}

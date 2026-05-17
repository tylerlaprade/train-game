use std::io::Write;

use crossterm::cursor;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::game::{
    CarColor, CarKind, Game, PARTIAL_CAR_WIDTH, SEG_HEIGHT, TRAIN_TOP_ROW_FROM_TOP, Train,
};

pub struct Sprite {
    pub width: usize,
    pub rows: &'static [&'static str],
}

// ── ENGINE ───────────────────────────────────────────────────────────────────
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

const SKY: Color = Color::Rgb { r: 90, g: 150, b: 200 };
const GROUND: Color = Color::Rgb { r: 60, g: 100, b: 40 };
const TRACK: Color = Color::Rgb { r: 90, g: 70, b: 50 };

const BLANK: CellFmt = CellFmt {
    ch: ' ',
    fg: Color::Reset,
    bg: Color::Reset,
};

pub fn render(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    let cols = game.screen_cols as usize;
    let rows = game.screen_rows as usize;
    if cols < 50 || rows < 18 {
        queue!(
            out,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            Print("Make the terminal a bit bigger (>=50x18) for the train!")
        )?;
        out.flush()?;
        return Ok(());
    }

    let mut grid = vec![BLANK; cols * rows];
    let idx = |r: usize, c: usize| r * cols + c;

    // Paint the whole screen with sky first; ground bands get repainted
    // under each train after.
    for r in 0..rows {
        for c in 0..cols {
            grid[idx(r, c)] = CellFmt { ch: ' ', fg: Color::Reset, bg: SKY };
        }
    }

    // Ground band under each train.
    for (train_idx, _train) in game.trains.iter().enumerate() {
        let top = game.train_top_for(train_idx);
        let ground = top + SEG_HEIGHT;
        if ground < rows {
            for c in 0..cols {
                grid[idx(ground, c)] = CellFmt {
                    ch: '~',
                    fg: Color::Rgb { r: 80, g: 60, b: 30 },
                    bg: GROUND,
                };
            }
        }
    }

    draw_all_trains(&mut grid, cols, rows, game);
    draw_smoke(&mut grid, cols, rows, game);
    draw_word_ui(&mut grid, cols, rows, game);
    draw_help(&mut grid, cols, rows, game);

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

fn draw_all_trains(grid: &mut [CellFmt], cols: usize, rows: usize, game: &Game) {
    for (train_idx, train) in game.trains.iter().enumerate() {
        let top_y = game.train_top_for(train_idx);
        if top_y + SEG_HEIGHT > rows {
            break;
        }
        draw_single_train(grid, cols, rows, train, top_y, game);
    }

    if game.horn_active() {
        // Speech bubble above engine of the top train.
        let bubble = "TOOT TOOT!";
        let cycle = game.cycle();
        let head_x = game.head_x.floor() as i32;
        let head_x = head_x.rem_euclid(cycle.max(1));
        let y = TRAIN_TOP_ROW_FROM_TOP.saturating_sub(2);
        for shift in [-cycle, 0, cycle] {
            let x = head_x + shift - bubble.len() as i32;
            put_text(
                grid,
                cols,
                rows,
                bubble,
                x,
                y,
                Color::White,
                Color::Rgb { r: 200, g: 30, b: 30 },
            );
        }
    }
}

fn draw_single_train(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    train: &Train,
    top_y: usize,
    game: &Game,
) {
    let cycle = game.cycle();
    let head_x = game.head_x.floor() as i32;
    let head_x = head_x.rem_euclid(cycle.max(1));

    for shift in [-cycle, 0, cycle] {
        let mut right = head_x + shift;

        let engine = &ENGINE;
        let left = right - (engine.width as i32 - 1);
        draw_sprite(
            grid,
            cols,
            rows,
            engine,
            Color::Rgb { r: 60, g: 60, b: 70 },
            left,
            top_y,
        );
        right = left - 1;

        for car in &train.cars {
            let sprite = car_sprite(car.kind);
            let left = right - (sprite.width as i32 - 1);
            draw_sprite(
                grid,
                cols,
                rows,
                sprite,
                car_color_to_term(car.color),
                left,
                top_y,
            );
            right = left - 1;
        }

        if train.partial_wheels > 0 {
            let left = right - (PARTIAL_CAR_WIDTH - 1);
            draw_partial_wheels(grid, cols, rows, train.partial_wheels, left, top_y);
            right = left - 1;
        }

        let caboose = &CABOOSE;
        let left = right - (caboose.width as i32 - 1);
        draw_sprite(grid, cols, rows, caboose, Color::Red, left, top_y);
    }
}

const WHEEL_X_POSITIONS: &[usize] = &[2, 13];
const PARTIAL_TEMPLATE: &str = " =(O)========(O)==";

fn draw_partial_wheels(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    n_wheels: u8,
    left_x: i32,
    top_y: usize,
) {
    let wheel_row = top_y + 7;
    if wheel_row >= rows {
        return;
    }
    // Draw continuous track under the entire partial slot.
    for c_off in 0..PARTIAL_CAR_WIDTH {
        let x = left_x + c_off;
        if x < 0 || x >= cols as i32 {
            continue;
        }
        grid[wheel_row * cols + x as usize] = CellFmt {
            ch: '=',
            fg: TRACK,
            bg: bg_under_train(wheel_row, top_y),
        };
    }
    // Stamp the visible wheels on top of the track.
    for (i, &wheel_x) in WHEEL_X_POSITIONS.iter().enumerate() {
        if i >= n_wheels as usize {
            break;
        }
        for (c_off, ch) in "(O)".chars().enumerate() {
            let x = left_x + wheel_x as i32 + c_off as i32;
            if x < 0 || x >= cols as i32 {
                continue;
            }
            let fg = if ch == 'O' { Color::Black } else { Color::DarkGrey };
            grid[wheel_row * cols + x as usize] = CellFmt {
                ch,
                fg,
                bg: bg_under_train(wheel_row, top_y),
            };
        }
    }
    let _ = PARTIAL_TEMPLATE; // documentation reference
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
        SKY
    } else if y >= train_top + SEG_HEIGHT {
        GROUND
    } else {
        Color::Reset
    }
}

fn char_visual(ch: char, base: Color) -> (char, Color) {
    match ch {
        'O' => ('O', Color::Black),
        '(' | ')' => (ch, Color::DarkGrey),
        '=' => ('=', TRACK),
        '*' => ('*', Color::Yellow),
        '░' => ('░', Color::Rgb { r: 220, g: 230, b: 240 }),
        '▒' => ('▒', Color::White),
        _ => (ch, base),
    }
}

fn draw_smoke(grid: &mut [CellFmt], cols: usize, _rows: usize, game: &Game) {
    let sky_floor = TRAIN_TOP_ROW_FROM_TOP;
    for s in &game.smoke {
        let x = s.x.round() as i32;
        let y = s.y.round() as i32;
        if y < 1 || y as usize >= sky_floor {
            continue;
        }
        let (ch, fg) = smoke_visual(s.age);
        let cycle = game.cycle();
        for shift in [-cycle, 0, cycle] {
            let xs = x + shift;
            if xs >= 0 && (xs as usize) < cols {
                let i = y as usize * cols + xs as usize;
                grid[i] = CellFmt { ch, fg, bg: SKY };
            }
        }
    }
}

fn smoke_visual(age: f32) -> (char, Color) {
    if age < 0.4 {
        ('@', Color::Rgb { r: 250, g: 250, b: 250 })
    } else if age < 0.9 {
        ('%', Color::Rgb { r: 220, g: 220, b: 220 })
    } else if age < 1.6 {
        ('o', Color::Rgb { r: 180, g: 180, b: 180 })
    } else if age < 2.3 {
        ('.', Color::Rgb { r: 140, g: 140, b: 140 })
    } else {
        ('.', Color::Rgb { r: 110, g: 110, b: 110 })
    }
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
    let panel_top = rows.saturating_sub(5);

    let bar_bg = if game.celebrating() {
        Color::Rgb { r: 30, g: 120, b: 30 }
    } else if game.wrong_flash() {
        Color::Rgb { r: 120, g: 30, b: 30 }
    } else {
        Color::Rgb { r: 25, g: 25, b: 35 }
    };
    for r in panel_top..rows {
        for c in 0..cols {
            grid[r * cols + c] = CellFmt { ch: ' ', fg: Color::White, bg: bar_bg };
        }
    }

    let wheels = game.total_wheels();
    let prompt = if game.celebrating() {
        format!(
            "  WUVVA WHEEL!  Wheels: {wheels}   Trains: {}/{}",
            game.trains.len(),
            game.max_trains()
        )
    } else {
        format!(
            "  Type the word:                  Wheels: {wheels}   Trains: {}/{}",
            game.trains.len(),
            game.max_trains()
        )
    };
    put_text(grid, cols, rows, &prompt, 2, panel_top, Color::White, bar_bg);

    // The target word, in plain readable letters, with the typed-so-far
    // letters bright green and the rest bright yellow. Spaced out so it
    // reads as 3 distinct letters.
    let big_y = panel_top + 2;
    let letters: Vec<char> = game.target_word.chars().collect();
    let spacing = 4; // cols between letter centers
    let total_w = letters.len().saturating_sub(1) * spacing + 1;
    let start_x = (cols.saturating_sub(total_w)) as i32 / 2;
    for (i, ch) in letters.iter().enumerate() {
        let done = i < game.typed.len();
        let color = if done { Color::Green } else { Color::Yellow };
        let upper = ch.to_ascii_uppercase();
        put_text(
            grid,
            cols,
            rows,
            &format!(" {upper} "),
            start_x + (i * spacing) as i32 - 1,
            big_y,
            color,
            bar_bg,
        );
    }
}

fn draw_help(grid: &mut [CellFmt], cols: usize, rows: usize, _game: &Game) {
    let help = " </> drive   SPACE toot   type the spoken word to add a wheel   ESC quit";
    put_text(grid, cols, rows, help, 0, 0, Color::White, SKY);
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

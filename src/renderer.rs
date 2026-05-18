use std::io::Write;

use crossterm::cursor;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::game::{CarColor, CarKind, Game, MAX_CARS, SEG_HEIGHT};

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
const GROUND_DARK: Color = Color::Rgb { r: 45, g: 80, b: 30 };
const TRACK: Color = Color::Rgb { r: 90, g: 70, b: 50 };

const BLANK: CellFmt = CellFmt {
    ch: ' ',
    fg: Color::Reset,
    bg: Color::Reset,
};

pub fn render(game: &Game, out: &mut impl Write) -> std::io::Result<()> {
    let cols = game.screen_cols as usize;
    let rows = game.screen_rows as usize;
    if cols < 50 || rows < 14 {
        queue!(
            out,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            Print("Make the terminal a bit bigger (>=50x14) for the train!")
        )?;
        out.flush()?;
        return Ok(());
    }

    let mut grid = vec![BLANK; cols * rows];
    let idx = |r: usize, c: usize| r * cols + c;

    let train_top = game.train_top();
    let horizon = train_top + SEG_HEIGHT;

    // Sky above horizon, grass below.
    for r in 0..rows {
        let bg = if r < horizon {
            SKY
        } else if r == horizon {
            GROUND
        } else {
            // Darker green further down to give a sense of depth.
            let depth = (r - horizon) as f32 / (rows - horizon).max(1) as f32;
            blend(GROUND, GROUND_DARK, depth)
        };
        let ch = if r == horizon { '~' } else { ' ' };
        let fg = if r == horizon {
            Color::Rgb { r: 80, g: 60, b: 30 }
        } else {
            Color::Reset
        };
        for c in 0..cols {
            grid[idx(r, c)] = CellFmt { ch, fg, bg };
        }
    }

    draw_train(&mut grid, cols, rows, train_top, game);
    draw_smoke(&mut grid, cols, rows, train_top, game);
    draw_top_bar(&mut grid, cols, rows, game);

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

fn blend(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = rgb_of(a);
    let (br, bg, bb) = rgb_of(b);
    let t = t.clamp(0.0, 1.0);
    Color::Rgb {
        r: lerp(ar, br, t),
        g: lerp(ag, bg, t),
        b: lerp(ab, bb, t),
    }
}

fn rgb_of(c: Color) -> (u8, u8, u8) {
    if let Color::Rgb { r, g, b } = c {
        (r, g, b)
    } else {
        (255, 255, 255)
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn draw_train(grid: &mut [CellFmt], cols: usize, rows: usize, train_top: usize, game: &Game) {
    let cycle = game.cycle();
    let head_x = game.head_x.floor() as i32;
    let head_x = head_x.rem_euclid(cycle.max(1));

    // We draw the train at three cycle offsets to handle wrap-around at
    // both edges of the screen seamlessly.
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
            train_top,
        );
        right = left - 1;

        for car in &game.cars {
            let sprite = car_sprite(car.kind);
            let left = right - (sprite.width as i32 - 1);
            draw_sprite(
                grid,
                cols,
                rows,
                sprite,
                car_color_to_term(car.color),
                left,
                train_top,
            );
            right = left - 1;
        }

        let caboose = &CABOOSE;
        let left = right - (caboose.width as i32 - 1);
        draw_sprite(grid, cols, rows, caboose, Color::Red, left, train_top);
    }

    if game.horn_active() {
        let bubble = "TOOT TOOT!";
        for shift in [-cycle, 0, cycle] {
            let x = head_x + shift - bubble.len() as i32;
            let y = train_top.saturating_sub(2);
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

fn draw_smoke(grid: &mut [CellFmt], cols: usize, _rows: usize, train_top: usize, game: &Game) {
    for s in &game.smoke {
        let x = s.x.round() as i32;
        let y = s.y.round() as i32;
        if y < 1 || y >= train_top as i32 {
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

fn draw_top_bar(grid: &mut [CellFmt], cols: usize, rows: usize, game: &Game) {
    let bg = if game.celebrating() {
        Color::Rgb { r: 30, g: 140, b: 30 }
    } else {
        SKY
    };
    for c in 0..cols {
        grid[c] = CellFmt { ch: ' ', fg: Color::White, bg };
    }
    let cars = game.cars.len();
    let left = format!(" </> drive   SPACE toot   ESC quit");
    let right = if game.celebrating() {
        format!("WUVVA wheel!  Cars: {cars}/{MAX_CARS} ")
    } else {
        format!("Cars: {cars}/{MAX_CARS} ")
    };
    put_text(grid, cols, rows, &left, 0, 0, Color::White, bg);
    let right_x = cols as i32 - right.chars().count() as i32;
    put_text(grid, cols, rows, &right, right_x.max(0), 0, Color::White, bg);
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

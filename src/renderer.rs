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
    width: 76,
    rows: &[
        "                                                ________                    ",
        "                                             __|__||__|__                  ",
        "                                            |__  ||  __|                  ",
        "                                   _________|  |____|  |_________        ",
        "                         _________/................................\\       ",
        "                ________/....::::....::::....::::....____......*....\\      ",
        "       ________/.................................__/____\\___________|      ",
        "      |..░░..░░..░░.|........::::........░░....|__|__|__|.........|       ",
        "      |..............|....::..::::..::....░░....|__|__|__|.........|       ",
        "      |______________|_____________________________________/_______/        ",
        "----#----(0)======(0)======(0)======(0)======(0)======#-------------       ",
        "========={===}======={===}======={===}======={===}=================       ",
        "----#------#----------#----------#----------#----------#-------------       ",
        "========================================================================    ",
    ],
};

// ── CABOOSE ──────────────────────────────────────────────────────────────────
pub const CABOOSE: Sprite = Sprite {
    width: 40,
    rows: &[
        "                                        ",
        "             ____________               ",
        "          __|..░..░..░..|__            ",
        "       __|________________|__          ",
        "      |........................|       ",
        "      |..░░....::::....░░.....|       ",
        "      |..::....::::....::.....|       ",
        "      |........................|       ",
        "    __|________________________|__     ",
        "   /______________________________\\    ",
        "====#====(0)==========(0)====#--------",
        "======={===}=========={===}===========",
        "----#--------#--------------#---------",
        "======================================",
    ],
};

pub const BOXCAR: Sprite = Sprite {
    width: 34,
    rows: &[
        "                                  ",
        "                                  ",
        "                                  ",
        "       ______________________     ",
        "    __|......................|__  ",
        "   |..::::....::::....::::...|   ",
        "   |..░░░░....░░░░....░░░░...|   ",
        "   |..::::....::::....::::...|   ",
        "   |..........................|   ",
        " __|__________________________|__",
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
        "----#-------#--------#------#----",
        "==================================",
    ],
};

pub const TANKER: Sprite = Sprite {
    width: 34,
    rows: &[
        "                                  ",
        "                                  ",
        "             __________           ",
        "        ___/............\\___     ",
        "      _/....::::..::::....\\_    ",
        "     /........................\\  ",
        "    |..░░..............░░....|   ",
        "    |......::::..::::........|   ",
        "     \\___....::::....______/    ",
        " _______|________________|____  ",
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
        "----#-------#--------#------#----",
        "==================================",
    ],
};

pub const HOPPER: Sprite = Sprite {
    width: 34,
    rows: &[
        "                                  ",
        "                                  ",
        "                                  ",
        "      ______________________      ",
        "    _/▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒\\_    ",
        "   |▒▒▒▒..::::..::::..▒▒▒▒▒▒|   ",
        "   |▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒|   ",
        "    \\.....::::..::::...../      ",
        " ____\\__________________/____   ",
        " __|________________________|__ ",
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
        "----#-------#--------#------#----",
        "==================================",
    ],
};

pub const PASSENGER: Sprite = Sprite {
    width: 34,
    rows: &[
        "                                  ",
        "                                  ",
        "          ______________          ",
        "      ___|..............|___      ",
        "    _|..░░..░░..░░..░░..░░|_    ",
        "   |..::::..::::..::::..::..|   ",
        "   |..░░..░░..░░..░░..░░...|   ",
        "   |..::::..[]..::::..[]...|   ",
        "   |........................|   ",
        " __|________________________|__ ",
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
        "----#-------#--------#------#----",
        "==================================",
    ],
};

pub const FLATCAR: Sprite = Sprite {
    width: 34,
    rows: &[
        "                                  ",
        "                                  ",
        "                                  ",
        "         ______      ______       ",
        "      __/▒▒▒▒▒▒\\____/▒▒▒▒▒▒\\__   ",
        "     |..::::::..|  |..::::::..|  ",
        "  ___|__________|__|__________|  ",
        " |..............................|",
        "_|______________________________|",
        " __|__________________________|_ ",
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
        "----#-------#--------#------#----",
        "==================================",
    ],
};

pub const GONDOLA: Sprite = Sprite {
    width: 34,
    rows: &[
        "                                  ",
        "                                  ",
        "                                  ",
        "      ______________________      ",
        "   __|......................|__   ",
        "  |..▒▒▒▒▒▒▒▒..::::..▒▒▒▒..|    ",
        "  |..▒▒..::::..▒▒▒▒..::::..|    ",
        "  |..........................|   ",
        "__|__________________________|__",
        " __|________________________|__ ",
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
        "----#-------#--------#------#----",
        "==================================",
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
        CarColor::Orange => Color::Rgb {
            r: 220,
            g: 130,
            b: 30,
        },
        CarColor::Brown => Color::Rgb {
            r: 150,
            g: 90,
            b: 50,
        },
        CarColor::Olive => Color::Rgb {
            r: 130,
            g: 130,
            b: 60,
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CellFmt {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

const GROUND: Color = Color::Rgb {
    r: 60,
    g: 100,
    b: 40,
};
const GROUND_DARK: Color = Color::Rgb {
    r: 45,
    g: 80,
    b: 30,
};
const TRACK: Color = Color::Rgb {
    r: 90,
    g: 70,
    b: 50,
};
const SKY_CYCLE_SECS: f32 = 84.0;

#[derive(Clone, Copy)]
struct SkyPalette {
    top: Color,
    mid: Color,
    horizon: Color,
    cloud: Color,
    cloud_shadow: Color,
    ground: Color,
    ground_dark: Color,
}

#[derive(Clone, Copy)]
struct SkyState {
    palette: SkyPalette,
    phase_time: f32,
    rain: f32,
    stars: f32,
}

const BLANK: CellFmt = CellFmt {
    ch: ' ',
    fg: Color::Reset,
    bg: Color::Reset,
};

pub struct Renderer {
    grid: Vec<CellFmt>,
    last_grid: Vec<CellFmt>,
    cols: usize,
    rows: usize,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            grid: Vec::new(),
            last_grid: Vec::new(),
            cols: 0,
            rows: 0,
        }
    }

    pub fn render(&mut self, game: &Game, out: &mut impl Write) -> std::io::Result<()> {
        let cols = game.screen_cols as usize;
        let rows = game.screen_rows as usize;

        if cols < 80 || rows < 22 {
            self.grid.clear();
            self.last_grid.clear();
            self.cols = 0;
            self.rows = 0;
            queue!(
                out,
                Clear(ClearType::All),
                cursor::MoveTo(0, 0),
                Print("Make the terminal a bit bigger (>=80x22) for the train!")
            )?;
            out.flush()?;
            return Ok(());
        }

        if cols != self.cols || rows != self.rows {
            self.grid = vec![BLANK; cols * rows];
            self.last_grid = vec![BLANK; cols * rows];
            self.cols = cols;
            self.rows = rows;
            queue!(out, Clear(ClearType::All))?;
        }

        let idx = |r: usize, c: usize| r * cols + c;

        let train_top = game.train_top();
        let horizon = train_top + SEG_HEIGHT - 2;
        let sky = sky_state(game.started_at.elapsed().as_secs_f32());

        draw_sky(&mut self.grid, cols, rows, horizon, sky);
        draw_clouds(&mut self.grid, cols, rows, horizon, sky);
        draw_stars(&mut self.grid, cols, horizon, sky);
        draw_rain(&mut self.grid, cols, rows, horizon, sky);

        draw_train(&mut self.grid, cols, rows, train_top, game);
        draw_smoke(&mut self.grid, cols, rows, train_top, game);
        draw_top_bar(&mut self.grid, cols, rows, game, sky.palette.top);

        let mut cur_fg = Color::Reset;
        let mut cur_bg = Color::Reset;
        let mut cursor_x = None;
        let mut cursor_y = None;

        queue!(out, ResetColor)?;

        for r in 0..rows {
            for c in 0..cols {
                let i = idx(r, c);
                let cell = self.grid[i];
                if cell != self.last_grid[i] {
                    if cursor_x != Some(c) || cursor_y != Some(r) {
                        queue!(out, cursor::MoveTo(c as u16, r as u16))?;
                    }
                    if cell.fg != cur_fg {
                        queue!(out, SetForegroundColor(cell.fg))?;
                        cur_fg = cell.fg;
                    }
                    if cell.bg != cur_bg {
                        queue!(out, SetBackgroundColor(cell.bg))?;
                        cur_bg = cell.bg;
                    }
                    queue!(out, Print(cell.ch))?;
                    cursor_x = Some(c + 1);
                    cursor_y = Some(r);
                }
            }
        }

        self.last_grid.copy_from_slice(&self.grid);
        out.flush()?;
        Ok(())
    }
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

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

fn sky_state(elapsed: f32) -> SkyState {
    let phase_time = elapsed % SKY_CYCLE_SECS;
    let phase = phase_time / SKY_CYCLE_SECS;

    let dawn = SkyPalette {
        top: rgb(70, 95, 170),
        mid: rgb(235, 135, 115),
        horizon: rgb(255, 190, 95),
        cloud: rgb(255, 215, 180),
        cloud_shadow: rgb(210, 145, 140),
        ground: rgb(70, 105, 50),
        ground_dark: rgb(40, 75, 36),
    };
    let day = SkyPalette {
        top: rgb(75, 155, 225),
        mid: rgb(105, 180, 235),
        horizon: rgb(170, 220, 245),
        cloud: rgb(245, 250, 255),
        cloud_shadow: rgb(190, 210, 225),
        ground: GROUND,
        ground_dark: GROUND_DARK,
    };
    let rain = SkyPalette {
        top: rgb(55, 75, 95),
        mid: rgb(80, 100, 120),
        horizon: rgb(115, 125, 130),
        cloud: rgb(120, 125, 130),
        cloud_shadow: rgb(70, 75, 82),
        ground: rgb(50, 90, 46),
        ground_dark: rgb(28, 58, 32),
    };
    let sunset = SkyPalette {
        top: rgb(75, 70, 140),
        mid: rgb(210, 95, 100),
        horizon: rgb(255, 150, 70),
        cloud: rgb(255, 175, 130),
        cloud_shadow: rgb(155, 80, 105),
        ground: rgb(70, 90, 42),
        ground_dark: rgb(38, 52, 30),
    };
    let night = SkyPalette {
        top: rgb(8, 14, 38),
        mid: rgb(18, 30, 68),
        horizon: rgb(40, 58, 92),
        cloud: rgb(70, 78, 98),
        cloud_shadow: rgb(28, 34, 55),
        ground: rgb(28, 52, 38),
        ground_dark: rgb(12, 28, 24),
    };

    let palette = if phase < 0.16 {
        blend_palette(dawn, day, smoothstep(phase / 0.16))
    } else if phase < 0.34 {
        day
    } else if phase < 0.47 {
        blend_palette(day, rain, smoothstep((phase - 0.34) / 0.13))
    } else if phase < 0.58 {
        blend_palette(rain, sunset, smoothstep((phase - 0.47) / 0.11))
    } else if phase < 0.72 {
        blend_palette(sunset, night, smoothstep((phase - 0.58) / 0.14))
    } else if phase < 0.93 {
        night
    } else {
        blend_palette(night, dawn, smoothstep((phase - 0.93) / 0.07))
    };

    SkyState {
        palette,
        phase_time,
        rain: bell(phase, 0.43, 0.10),
        stars: smoothstep((phase - 0.67) / 0.08) * (1.0 - smoothstep((phase - 0.94) / 0.05)),
    }
}

fn blend_palette(a: SkyPalette, b: SkyPalette, t: f32) -> SkyPalette {
    SkyPalette {
        top: blend(a.top, b.top, t),
        mid: blend(a.mid, b.mid, t),
        horizon: blend(a.horizon, b.horizon, t),
        cloud: blend(a.cloud, b.cloud, t),
        cloud_shadow: blend(a.cloud_shadow, b.cloud_shadow, t),
        ground: blend(a.ground, b.ground, t),
        ground_dark: blend(a.ground_dark, b.ground_dark, t),
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn bell(x: f32, center: f32, half_width: f32) -> f32 {
    (1.0 - ((x - center).abs() / half_width)).clamp(0.0, 1.0)
}

fn draw_sky(grid: &mut [CellFmt], cols: usize, rows: usize, horizon: usize, sky: SkyState) {
    for r in 0..rows {
        let bg = if r < horizon {
            let depth = r as f32 / horizon.max(1) as f32;
            if depth < 0.58 {
                blend(sky.palette.top, sky.palette.mid, depth / 0.58)
            } else {
                blend(sky.palette.mid, sky.palette.horizon, (depth - 0.58) / 0.42)
            }
        } else if r == horizon {
            sky.palette.ground
        } else {
            let depth = (r - horizon) as f32 / (rows - horizon).max(1) as f32;
            blend(sky.palette.ground, sky.palette.ground_dark, depth)
        };
        let ch = if r == horizon { '~' } else { ' ' };
        let fg = if r == horizon {
            Color::Rgb {
                r: 80,
                g: 60,
                b: 30,
            }
        } else {
            Color::Reset
        };
        for c in 0..cols {
            grid[r * cols + c] = CellFmt { ch, fg, bg };
        }
    }
}

fn draw_clouds(grid: &mut [CellFmt], cols: usize, rows: usize, horizon: usize, sky: SkyState) {
    let upper = horizon.saturating_sub(4).max(2);
    let clouds = [
        (0.08_f32, 0.16_f32, 2.5_f32),
        (0.46_f32, 0.28_f32, 1.6_f32),
        (0.76_f32, 0.10_f32, 2.1_f32),
    ];

    for (x_base, y_base, speed) in clouds {
        let width = 18_i32;
        let cycle = cols as i32 + width;
        let x = ((x_base * cols as f32 + sky.phase_time * speed) as i32).rem_euclid(cycle) - width;
        let y = 2 + (y_base * upper as f32) as usize;
        draw_cloud(grid, cols, rows, x, y.min(horizon.saturating_sub(4)), sky);
    }
}

fn draw_cloud(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    left_x: i32,
    top_y: usize,
    sky: SkyState,
) {
    const CLOUD: &[&str] = &[
        "     .--.         ",
        "  .-(    ).       ",
        " (___.__)__)      ",
    ];

    for (r_off, row) in CLOUD.iter().enumerate() {
        let y = top_y + r_off;
        if y >= rows {
            return;
        }
        for (c_off, ch) in row.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x = left_x + c_off as i32;
            if x < 0 || x >= cols as i32 {
                continue;
            }
            let fg = if r_off == CLOUD.len() - 1 {
                sky.palette.cloud_shadow
            } else {
                sky.palette.cloud
            };
            let i = y * cols + x as usize;
            grid[i] = CellFmt {
                ch,
                fg,
                bg: grid[i].bg,
            };
        }
    }
}

fn draw_stars(grid: &mut [CellFmt], cols: usize, horizon: usize, sky: SkyState) {
    if sky.stars <= 0.05 {
        return;
    }
    let star_rows = horizon.saturating_sub(2).max(1);
    for i in 0..(cols / 5).max(12) {
        let x = (i * 37 + 11) % cols;
        let y = 1 + ((i * 17 + 5) % star_rows);
        if y >= horizon {
            continue;
        }
        let ch = if i % 4 == 0 { '*' } else { '.' };
        let fg = blend(sky.palette.top, rgb(245, 245, 210), sky.stars);
        grid[y * cols + x] = CellFmt {
            ch,
            fg,
            bg: grid[y * cols + x].bg,
        };
    }
}

fn draw_rain(grid: &mut [CellFmt], cols: usize, _rows: usize, horizon: usize, sky: SkyState) {
    if sky.rain <= 0.05 {
        return;
    }

    let offset = (sky.phase_time * 12.0) as usize;
    let fg = blend(rgb(120, 150, 180), rgb(190, 210, 230), sky.rain);
    for y in 1..horizon {
        for x in (0..cols).step_by(4) {
            let px = (x + ((y * 3 + offset) % 4)) % cols;
            if (px + y + offset) % 3 != 0 {
                continue;
            }
            let i = y * cols + px;
            grid[i] = CellFmt {
                ch: '/',
                fg,
                bg: grid[i].bg,
            };
        }
    }
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
            Color::Rgb {
                r: 60,
                g: 60,
                b: 70,
            },
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
                Color::Rgb {
                    r: 200,
                    g: 30,
                    b: 30,
                },
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
            let i = y * cols + x as usize;
            if let Some(cell) = char_visual(ch, base, grid[i].bg) {
                grid[i] = cell;
            }
        }
    }
}

fn char_visual(ch: char, base: Color, bg: Color) -> Option<CellFmt> {
    let detail = contrast_color(base);
    match ch {
        ' ' => None,
        '.' => Some(CellFmt {
            ch: ' ',
            fg: Color::Reset,
            bg: base,
        }),
        '0' => Some(CellFmt {
            ch,
            fg: Color::Black,
            bg,
        }),
        '(' | ')' | '{' | '}' | '#' | '-' => Some(CellFmt {
            ch,
            fg: Color::DarkGrey,
            bg,
        }),
        '=' => Some(CellFmt { ch, fg: TRACK, bg }),
        '*' => Some(CellFmt {
            ch,
            fg: Color::Yellow,
            bg: base,
        }),
        '░' => Some(CellFmt {
            ch,
            fg: Color::Rgb {
                r: 210,
                g: 235,
                b: 255,
            },
            bg: base,
        }),
        '▒' => Some(CellFmt {
            ch,
            fg: Color::Rgb {
                r: 35,
                g: 35,
                b: 35,
            },
            bg: base,
        }),
        _ => Some(CellFmt {
            ch,
            fg: detail,
            bg: base,
        }),
    }
}

fn contrast_color(base: Color) -> Color {
    let (r, g, b) = rgb_of(base);
    let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    if luminance > 150.0 {
        Color::Black
    } else {
        Color::White
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
                grid[i] = CellFmt {
                    ch,
                    fg,
                    bg: grid[i].bg,
                };
            }
        }
    }
}

fn smoke_visual(age: f32) -> (char, Color) {
    if age < 0.4 {
        (
            '@',
            Color::Rgb {
                r: 250,
                g: 250,
                b: 250,
            },
        )
    } else if age < 0.9 {
        (
            '%',
            Color::Rgb {
                r: 220,
                g: 220,
                b: 220,
            },
        )
    } else if age < 1.6 {
        (
            'o',
            Color::Rgb {
                r: 180,
                g: 180,
                b: 180,
            },
        )
    } else if age < 2.3 {
        (
            '.',
            Color::Rgb {
                r: 140,
                g: 140,
                b: 140,
            },
        )
    } else {
        (
            '.',
            Color::Rgb {
                r: 110,
                g: 110,
                b: 110,
            },
        )
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

fn draw_top_bar(grid: &mut [CellFmt], cols: usize, rows: usize, game: &Game, sky_top: Color) {
    let bg = if game.celebrating() {
        Color::Rgb {
            r: 30,
            g: 140,
            b: 30,
        }
    } else {
        sky_top
    };
    for c in 0..cols {
        grid[c] = CellFmt {
            ch: ' ',
            fg: Color::White,
            bg,
        };
    }
    let cars = game.cars.len();
    let left = format!(" ←/→ drive   SPACE toot   EXIT or QUIT to exit");
    let right = if game.celebrating() {
        format!("Another wheel!  Cars: {cars}/{MAX_CARS} ")
    } else {
        format!("Cars: {cars}/{MAX_CARS} ")
    };
    put_text(grid, cols, rows, &left, 0, 0, Color::White, bg);
    let right_x = cols as i32 - right.chars().count() as i32;
    put_text(
        grid,
        cols,
        rows,
        &right,
        right_x.max(0),
        0,
        Color::White,
        bg,
    );
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

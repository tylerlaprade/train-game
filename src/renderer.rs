use std::io::Write;

use crossterm::cursor;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};

use crate::game::{
    BIOME_BLEND_FRACTION, BIOME_BLEND_START_DISTANCE, BIOME_TRANSITION_DISTANCE, CarColor, CarKind,
    DAY_CYCLE_SECS, Game, SEG_HEIGHT,
};

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
        "========={===}======={===}======={===}======={===}=================       ",
        "----#----(0)======(0)======(0)======(0)======(0)======#-------------       ",
        "",
        "",
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
        "======={===}=========={===}===========",
        "====#====(0)==========(0)====#--------",
        "",
        "",
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
        "======{===}========{===}=========",
        "--#====(0)==========(0)====#------",
        "",
        "",
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
        "======{===}========{===}=========",
        "--#====(0)==========(0)====#------",
        "",
        "",
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
        "======{===}========{===}=========",
        "--#====(0)==========(0)====#------",
        "",
        "",
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
        "======{===}========{===}=========",
        "--#====(0)==========(0)====#------",
        "",
        "",
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
        "======{===}========{===}=========",
        "--#====(0)==========(0)====#------",
        "",
        "",
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
        "======{===}========{===}=========",
        "--#====(0)==========(0)====#------",
        "",
        "",
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
const UNDERCARRIAGE: Color = Color::Rgb {
    r: 75,
    g: 80,
    b: 86,
};
const TRACK_RAIL: Color = Color::Rgb {
    r: 165,
    g: 135,
    b: 88,
};
const TRACK_TIE: Color = Color::Rgb {
    r: 95,
    g: 58,
    b: 34,
};
const SKY_CYCLE_SECS: f32 = DAY_CYCLE_SECS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BiomeKind {
    Meadow,
    Forest,
    Mountains,
    Desert,
    Canyon,
    Tundra,
    City,
    Coast,
}

#[derive(Clone, Copy)]
struct Biome {
    kind: BiomeKind,
    ground: Color,
    ground_dark: Color,
    far: Color,
    near: Color,
    accent: Color,
    far_freq: f32,
    far_base: f32,
    far_amp: f32,
    far_detail: f32,
    near_freq: f32,
    near_base: f32,
    near_amp: f32,
    near_detail: f32,
}

#[derive(Clone, Copy)]
struct BiomeVisual {
    kind: BiomeKind,
    ground: Color,
    ground_dark: Color,
    far: Color,
    near: Color,
    accent: Color,
    far_freq: f32,
    far_base: f32,
    far_amp: f32,
    far_detail: f32,
    near_freq: f32,
    near_base: f32,
    near_amp: f32,
    near_detail: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TerrainDepth {
    Far,
    Near,
}

#[derive(Clone, Copy)]
struct TerrainLayer {
    kind: BiomeKind,
    color: Color,
    accent: Color,
    depth: TerrainDepth,
    scroll: f32,
    freq: f32,
    base: f32,
    amp: f32,
    detail_amp: f32,
}

#[derive(Clone, Copy)]
struct BuildingSpec {
    x: i32,
    base_y: i32,
    width: i32,
    height: i32,
    wall: Color,
    light: Color,
}

#[derive(Clone, Copy)]
struct BiomeDetailLayer {
    rows: usize,
    horizon: usize,
    sky: SkyState,
    kind: BiomeKind,
    phase: f32,
}

const BIOMES: [Biome; 8] = [
    Biome {
        kind: BiomeKind::Meadow,
        ground: Color::Rgb {
            r: 58,
            g: 116,
            b: 47,
        },
        ground_dark: Color::Rgb {
            r: 32,
            g: 75,
            b: 34,
        },
        far: Color::Rgb {
            r: 98,
            g: 138,
            b: 70,
        },
        near: Color::Rgb {
            r: 38,
            g: 96,
            b: 36,
        },
        accent: Color::Rgb {
            r: 235,
            g: 215,
            b: 95,
        },
        far_freq: 0.026,
        far_base: 1.0,
        far_amp: 1.4,
        far_detail: 0.5,
        near_freq: 0.052,
        near_base: 1.8,
        near_amp: 2.1,
        near_detail: 1.0,
    },
    Biome {
        kind: BiomeKind::Forest,
        ground: Color::Rgb {
            r: 36,
            g: 92,
            b: 44,
        },
        ground_dark: Color::Rgb {
            r: 18,
            g: 54,
            b: 34,
        },
        far: Color::Rgb {
            r: 54,
            g: 108,
            b: 64,
        },
        near: Color::Rgb {
            r: 22,
            g: 72,
            b: 38,
        },
        accent: Color::Rgb {
            r: 20,
            g: 62,
            b: 28,
        },
        far_freq: 0.032,
        far_base: 1.4,
        far_amp: 2.0,
        far_detail: 0.9,
        near_freq: 0.078,
        near_base: 2.2,
        near_amp: 2.8,
        near_detail: 1.2,
    },
    Biome {
        kind: BiomeKind::Mountains,
        ground: Color::Rgb {
            r: 74,
            g: 92,
            b: 82,
        },
        ground_dark: Color::Rgb {
            r: 42,
            g: 55,
            b: 58,
        },
        far: Color::Rgb {
            r: 120,
            g: 128,
            b: 128,
        },
        near: Color::Rgb {
            r: 78,
            g: 88,
            b: 86,
        },
        accent: Color::Rgb {
            r: 218,
            g: 228,
            b: 230,
        },
        far_freq: 0.048,
        far_base: 3.0,
        far_amp: 5.8,
        far_detail: 2.4,
        near_freq: 0.074,
        near_base: 2.2,
        near_amp: 3.4,
        near_detail: 1.8,
    },
    Biome {
        kind: BiomeKind::Desert,
        ground: Color::Rgb {
            r: 164,
            g: 126,
            b: 67,
        },
        ground_dark: Color::Rgb {
            r: 118,
            g: 82,
            b: 44,
        },
        far: Color::Rgb {
            r: 190,
            g: 146,
            b: 78,
        },
        near: Color::Rgb {
            r: 142,
            g: 94,
            b: 42,
        },
        accent: Color::Rgb {
            r: 70,
            g: 112,
            b: 66,
        },
        far_freq: 0.018,
        far_base: 0.8,
        far_amp: 1.2,
        far_detail: 0.25,
        near_freq: 0.030,
        near_base: 1.4,
        near_amp: 1.6,
        near_detail: 0.45,
    },
    Biome {
        kind: BiomeKind::Canyon,
        ground: Color::Rgb {
            r: 128,
            g: 72,
            b: 48,
        },
        ground_dark: Color::Rgb {
            r: 78,
            g: 42,
            b: 32,
        },
        far: Color::Rgb {
            r: 164,
            g: 92,
            b: 62,
        },
        near: Color::Rgb {
            r: 112,
            g: 52,
            b: 38,
        },
        accent: Color::Rgb {
            r: 205,
            g: 120,
            b: 74,
        },
        far_freq: 0.024,
        far_base: 2.2,
        far_amp: 3.2,
        far_detail: 0.8,
        near_freq: 0.058,
        near_base: 2.2,
        near_amp: 2.8,
        near_detail: 1.1,
    },
    Biome {
        kind: BiomeKind::Tundra,
        ground: Color::Rgb {
            r: 155,
            g: 176,
            b: 180,
        },
        ground_dark: Color::Rgb {
            r: 88,
            g: 112,
            b: 122,
        },
        far: Color::Rgb {
            r: 188,
            g: 205,
            b: 210,
        },
        near: Color::Rgb {
            r: 120,
            g: 148,
            b: 154,
        },
        accent: Color::Rgb {
            r: 240,
            g: 248,
            b: 248,
        },
        far_freq: 0.032,
        far_base: 1.8,
        far_amp: 3.0,
        far_detail: 1.0,
        near_freq: 0.050,
        near_base: 1.3,
        near_amp: 1.8,
        near_detail: 0.7,
    },
    Biome {
        kind: BiomeKind::City,
        ground: Color::Rgb {
            r: 78,
            g: 84,
            b: 86,
        },
        ground_dark: Color::Rgb {
            r: 42,
            g: 48,
            b: 52,
        },
        far: Color::Rgb {
            r: 100,
            g: 108,
            b: 112,
        },
        near: Color::Rgb {
            r: 68,
            g: 74,
            b: 78,
        },
        accent: Color::Rgb {
            r: 185,
            g: 176,
            b: 126,
        },
        far_freq: 0.014,
        far_base: 0.5,
        far_amp: 0.7,
        far_detail: 0.2,
        near_freq: 0.026,
        near_base: 0.8,
        near_amp: 0.8,
        near_detail: 0.25,
    },
    Biome {
        kind: BiomeKind::Coast,
        ground: Color::Rgb {
            r: 62,
            g: 124,
            b: 116,
        },
        ground_dark: Color::Rgb {
            r: 34,
            g: 74,
            b: 82,
        },
        far: Color::Rgb {
            r: 96,
            g: 150,
            b: 148,
        },
        near: Color::Rgb {
            r: 42,
            g: 105,
            b: 112,
        },
        accent: Color::Rgb {
            r: 188,
            g: 220,
            b: 210,
        },
        far_freq: 0.020,
        far_base: 0.7,
        far_amp: 0.9,
        far_detail: 0.25,
        near_freq: 0.042,
        near_base: 1.0,
        near_amp: 1.2,
        near_detail: 0.4,
    },
];

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
    elapsed: f32,
    phase_time: f32,
    rain: f32,
    stars: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrecipitationKind {
    Rain,
    Snow,
}

#[derive(Clone, Copy, Debug)]
pub struct WeatherState {
    pub intensity: f32,
    pub kind: PrecipitationKind,
}

impl WeatherState {
    pub fn rain_audio_intensity(self) -> f32 {
        match self.kind {
            PrecipitationKind::Rain => self.intensity,
            PrecipitationKind::Snow => 0.0,
        }
    }
}

const BLANK: CellFmt = CellFmt {
    ch: ' ',
    fg: Color::Reset,
    bg: Color::Reset,
};
const TERRAIN_FAR_SCROLL_FREQ: f32 = 0.026;
const TERRAIN_NEAR_SCROLL_FREQ: f32 = 0.052;

pub struct Renderer {
    grid: Vec<CellFmt>,
    last_grid: Vec<CellFmt>,
    cols: usize,
    rows: usize,
    truecolor: bool,
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
            truecolor: truecolor_supported(),
        }
    }

    /// Terminals without 24-bit color (e.g. Apple's Terminal.app) garble
    /// `Color::Rgb` sequences, so fall back to the nearest xterm-256 entry.
    fn adapt(&self, c: Color) -> Color {
        match c {
            Color::Rgb { r, g, b } if !self.truecolor => Color::AnsiValue(rgb_to_ansi256(r, g, b)),
            _ => c,
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
        let weather = weather_state_from_sky(game.distance_traveled, sky);
        let biome = biome_visual(game.distance_traveled);

        draw_sky(&mut self.grid, cols, rows, horizon, sky, biome);
        draw_clouds(&mut self.grid, cols, rows, horizon, sky);
        draw_stars(&mut self.grid, cols, horizon, sky);
        draw_terrain(
            &mut self.grid,
            cols,
            horizon,
            sky,
            biome,
            game.distance_traveled,
            self.truecolor,
        );
        draw_tracks(&mut self.grid, cols, rows, horizon, game.distance_traveled);
        draw_biome_details(
            &mut self.grid,
            cols,
            rows,
            horizon,
            sky,
            game.distance_traveled,
        );
        draw_precipitation(&mut self.grid, cols, horizon, sky, weather);

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
                        queue!(out, SetForegroundColor(self.adapt(cell.fg)))?;
                        cur_fg = cell.fg;
                    }
                    if cell.bg != cur_bg {
                        queue!(out, SetBackgroundColor(self.adapt(cell.bg)))?;
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

/// True when the terminal advertises 24-bit color via `COLORTERM`. Terminals
/// that don't (notably Apple's Terminal.app) only handle the xterm-256 palette.
fn truecolor_supported() -> bool {
    matches!(
        std::env::var("COLORTERM").as_deref(),
        Ok("truecolor") | Ok("24bit")
    )
}

/// Map a 24-bit color to the closest xterm-256 palette index, choosing between
/// the 6×6×6 color cube (16–231) and the grayscale ramp (232–255).
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

    let nearest_level = |v: u8| -> usize {
        LEVELS
            .iter()
            .enumerate()
            .min_by_key(|&(_, &lvl)| (v as i32 - lvl as i32).abs())
            .map(|(i, _)| i)
            .unwrap()
    };
    let dist = |a: (u8, u8, u8), x: u8, y: u8, z: u8| -> i32 {
        let dr = a.0 as i32 - x as i32;
        let dg = a.1 as i32 - y as i32;
        let db = a.2 as i32 - z as i32;
        dr * dr + dg * dg + db * db
    };

    let (ri, gi, bi) = (nearest_level(r), nearest_level(g), nearest_level(b));
    let cube = (LEVELS[ri], LEVELS[gi], LEVELS[bi]);
    let cube_idx = (16 + 36 * ri + 6 * gi + bi) as u8;

    // The grayscale ramp only wins for near-neutral tones. Gating it behind low
    // chroma stops colorful values (e.g. meadow greens) from washing out to gray
    // just because the nearest coarse cube corner sits farther in raw RGB space.
    let chroma = r.max(g).max(b) - r.min(g).min(b);
    if chroma <= 16 {
        // Grayscale ramp values run 8, 18, .. 238 at indices 232..=255.
        let avg = (r as u32 + g as u32 + b as u32) / 3;
        let gray_idx = (((avg as i32 - 8) + 5) / 10).clamp(0, 23) as u8;
        let gray_val = 8 + gray_idx * 10;
        if dist((gray_val, gray_val, gray_val), r, g, b) < dist(cube, r, g, b) {
            return 232 + gray_idx;
        }
    }
    cube_idx
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
        cloud: rgb(180, 185, 195),
        cloud_shadow: rgb(140, 145, 155),
        ground: rgb(50, 90, 46),
        ground_dark: rgb(28, 58, 32),
    };
    let sunset = SkyPalette {
        top: rgb(75, 70, 140),
        mid: rgb(210, 95, 100),
        horizon: rgb(255, 150, 70),
        cloud: rgb(255, 200, 150),
        cloud_shadow: rgb(230, 155, 115),
        ground: rgb(70, 90, 42),
        ground_dark: rgb(38, 52, 30),
    };
    let night = SkyPalette {
        top: rgb(8, 14, 38),
        mid: rgb(18, 30, 68),
        horizon: rgb(40, 58, 92),
        cloud: rgb(140, 150, 175),
        cloud_shadow: rgb(95, 105, 130),
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
        elapsed,
        phase_time,
        rain: bell(phase, 0.43, 0.10),
        stars: smoothstep((phase - 0.67) / 0.08) * (1.0 - smoothstep((phase - 0.94) / 0.05)),
    }
}

pub fn weather_state(distance: f32, elapsed: f32) -> WeatherState {
    weather_state_from_sky(distance, sky_state(elapsed))
}

fn weather_state_from_sky(distance: f32, sky: SkyState) -> WeatherState {
    WeatherState {
        intensity: sky.rain,
        kind: match dominant_biome_kind(distance) {
            BiomeKind::Tundra => PrecipitationKind::Snow,
            _ => PrecipitationKind::Rain,
        },
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

fn biome_visual(distance: f32) -> BiomeVisual {
    let (current, next, mix) = biome_transition(distance);
    blend_biome(current, next, mix)
}

fn biome_transition(distance: f32) -> (Biome, Biome, f32) {
    let progress = distance / BIOME_TRANSITION_DISTANCE;
    let base = progress.floor();
    let local = progress - base;
    let blend_start = 1.0 - BIOME_BLEND_FRACTION;
    let mix = if local < blend_start {
        0.0
    } else {
        smoothstep((local - blend_start) / BIOME_BLEND_FRACTION)
    };
    let current_idx = (base as i32).rem_euclid(BIOMES.len() as i32) as usize;
    let next_idx = (current_idx + 1) % BIOMES.len();
    let current = BIOMES[current_idx];
    let next = BIOMES[next_idx];

    (current, next, mix)
}

fn dominant_biome_kind(distance: f32) -> BiomeKind {
    let (current, next, mix) = biome_transition(distance);
    if mix >= 0.5 { next.kind } else { current.kind }
}

fn blend_biome(a: Biome, b: Biome, t: f32) -> BiomeVisual {
    BiomeVisual {
        kind: if t >= 0.5 { b.kind } else { a.kind },
        ground: blend(a.ground, b.ground, t),
        ground_dark: blend(a.ground_dark, b.ground_dark, t),
        far: blend(a.far, b.far, t),
        near: blend(a.near, b.near, t),
        accent: blend(a.accent, b.accent, t),
        far_freq: mix_f32(a.far_freq, b.far_freq, t),
        far_base: mix_f32(a.far_base, b.far_base, t),
        far_amp: mix_f32(a.far_amp, b.far_amp, t),
        far_detail: mix_f32(a.far_detail, b.far_detail, t),
        near_freq: mix_f32(a.near_freq, b.near_freq, t),
        near_base: mix_f32(a.near_base, b.near_base, t),
        near_amp: mix_f32(a.near_amp, b.near_amp, t),
        near_detail: mix_f32(a.near_detail, b.near_detail, t),
    }
}

fn mix_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn biome_lit(base: Color, sky_color: Color) -> Color {
    blend(base, sky_color, 0.35)
}

fn draw_sky(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    horizon: usize,
    sky: SkyState,
    biome: BiomeVisual,
) {
    let ground = biome_lit(biome.ground, sky.palette.ground);
    let ground_dark = biome_lit(biome.ground_dark, sky.palette.ground_dark);

    for r in 0..rows {
        let bg = if r < horizon {
            let depth = r as f32 / horizon.max(1) as f32;
            if depth < 0.58 {
                blend(sky.palette.top, sky.palette.mid, depth / 0.58)
            } else {
                blend(sky.palette.mid, sky.palette.horizon, (depth - 0.58) / 0.42)
            }
        } else if r == horizon {
            ground
        } else {
            let depth = (r - horizon) as f32 / (rows - horizon).max(1) as f32;
            blend(ground, ground_dark, depth)
        };
        for c in 0..cols {
            grid[r * cols + c] = CellFmt {
                ch: ' ',
                fg: Color::Reset,
                bg,
            };
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

fn draw_terrain(
    grid: &mut [CellFmt],
    cols: usize,
    horizon: usize,
    sky: SkyState,
    biome: BiomeVisual,
    phase: f32,
    truecolor: bool,
) {
    if horizon < 3 {
        return;
    }

    let far = blend(
        biome_lit(biome.far, sky.palette.ground),
        sky.palette.horizon,
        0.28,
    );
    let near = biome_lit(biome.near, sky.palette.ground_dark);
    let accent = biome_lit(biome.accent, sky.palette.horizon);

    // Far ridge: washed out toward horizon (atmospheric perspective)
    draw_terrain_layer(
        grid,
        cols,
        horizon,
        TerrainLayer {
            kind: biome.kind,
            color: far,
            accent,
            depth: TerrainDepth::Far,
            scroll: phase * 0.15 * TERRAIN_FAR_SCROLL_FREQ,
            freq: biome.far_freq,
            base: biome.far_base,
            amp: biome.far_amp,
            detail_amp: biome.far_detail,
        },
        truecolor,
    );
    // Near hills: slightly darker than foreground grass, not pitch-black
    draw_terrain_layer(
        grid,
        cols,
        horizon,
        TerrainLayer {
            kind: biome.kind,
            color: near,
            accent,
            depth: TerrainDepth::Near,
            scroll: phase * 0.4 * TERRAIN_NEAR_SCROLL_FREQ,
            freq: biome.near_freq,
            base: biome.near_base,
            amp: biome.near_amp,
            detail_amp: biome.near_detail,
        },
        truecolor,
    );
}

fn draw_terrain_layer(
    grid: &mut [CellFmt],
    cols: usize,
    horizon: usize,
    layer: TerrainLayer,
    truecolor: bool,
) {
    const STEPS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let max_height = horizon.saturating_sub(1);
    if max_height == 0 {
        return;
    }
    for x in 0..cols {
        let base_phase = x as f32 * layer.freq + layer.scroll;
        let h = layer.base
            + layer.amp * 0.5 * (1.0 + base_phase.sin())
            + layer.detail_amp * 0.5 * (base_phase * 3.1 + 1.3).sin();
        if h <= 0.0 {
            continue;
        }
        let height_int = (h as usize).min(max_height);
        let frac = h - h.floor();
        for k in 0..height_int {
            let y = horizon - 1 - k;
            let i = y * cols + x;
            grid[i] = CellFmt {
                ch: ' ',
                fg: Color::Reset,
                bg: layer.color,
            };
        }
        if !truecolor {
            // Terminal.app renders block-element glyphs with seams/hairlines, so
            // snap the silhouette to whole cells: round the fractional top up or down.
            if frac >= 0.5 && height_int < max_height {
                let y = horizon - 1 - height_int;
                let i = y * cols + x;
                grid[i] = CellFmt {
                    ch: ' ',
                    fg: Color::Reset,
                    bg: layer.color,
                };
            }
        } else if frac > 0.125 && height_int < max_height {
            let y = horizon - 1 - height_int;
            let i = y * cols + x;
            // For >= half-terrain cells, flip to an upper block with bg = terrain so
            // overlays (rain, train wheels) replacing the char reveal terrain, not sky.
            grid[i] = if frac >= 0.875 {
                CellFmt {
                    ch: '▔',
                    fg: grid[i].bg,
                    bg: layer.color,
                }
            } else if frac >= 0.5 {
                CellFmt {
                    ch: '▀',
                    fg: grid[i].bg,
                    bg: layer.color,
                }
            } else {
                let idx = ((frac * 8.0) as usize).min(7);
                CellFmt {
                    ch: STEPS[idx],
                    fg: layer.color,
                    bg: grid[i].bg,
                }
            };
        }
        draw_terrain_texture(grid, cols, horizon, layer, x, height_int);
    }
}

fn draw_terrain_texture(
    grid: &mut [CellFmt],
    cols: usize,
    horizon: usize,
    layer: TerrainLayer,
    x: usize,
    height: usize,
) {
    if height == 0 {
        return;
    }

    let top_y = horizon.saturating_sub(height);
    let bottom_y = horizon.saturating_sub(1);
    let world_x = x as i32 + (layer.scroll / layer.freq.max(0.001)).round() as i32;
    let texture_fg = blend(layer.accent, layer.color, 0.25);
    let shadow_fg = blend(layer.color, rgb(28, 28, 26), 0.35);

    match layer.kind {
        BiomeKind::Meadow => {
            if layer.depth == TerrainDepth::Near && world_x.rem_euclid(9) == 0 {
                put_terrain_texture(grid, cols, x, bottom_y, '\'', texture_fg);
            }
            if layer.depth == TerrainDepth::Near && world_x.rem_euclid(23) == 0 {
                put_terrain_texture(grid, cols, x, top_y.max(1), '.', rgb(235, 215, 95));
            }
        }
        BiomeKind::Forest => {
            if world_x.rem_euclid(if layer.depth == TerrainDepth::Far {
                14
            } else {
                10
            }) == 0
            {
                put_terrain_texture(grid, cols, x, top_y, '^', texture_fg);
                if top_y < bottom_y {
                    put_terrain_texture(grid, cols, x, top_y + 1, '|', shadow_fg);
                }
            }
        }
        BiomeKind::Mountains => {
            if world_x.rem_euclid(7) == 0 {
                put_terrain_texture(grid, cols, x, top_y, '^', rgb(230, 238, 238));
            } else if world_x.rem_euclid(11) == 0 && top_y < bottom_y {
                let ch = if world_x.rem_euclid(2) == 0 {
                    '/'
                } else {
                    '\\'
                };
                put_terrain_texture(grid, cols, x, top_y + 1, ch, shadow_fg);
            }
        }
        BiomeKind::Desert => {
            for y in top_y..=bottom_y {
                if (world_x + y as i32 * 5).rem_euclid(13) == 0 {
                    put_terrain_texture(grid, cols, x, y, '-', texture_fg);
                }
            }
        }
        BiomeKind::Canyon => {
            for y in top_y..=bottom_y {
                if (y as i32 + world_x.div_euclid(5)).rem_euclid(3) == 0 {
                    put_terrain_texture(grid, cols, x, y, '─', shadow_fg);
                }
            }
        }
        BiomeKind::Tundra => {
            if world_x.rem_euclid(6) == 0 {
                put_terrain_texture(grid, cols, x, top_y, '.', rgb(238, 248, 248));
            }
            if layer.depth == TerrainDepth::Near && top_y < bottom_y && world_x.rem_euclid(19) == 0
            {
                put_terrain_texture(grid, cols, x, bottom_y, '_', texture_fg);
            }
        }
        BiomeKind::City => {
            if layer.depth == TerrainDepth::Near {
                for y in top_y..=bottom_y {
                    if (world_x + y as i32 * 3).rem_euclid(8) == 0 {
                        put_terrain_texture(grid, cols, x, y, '▪', texture_fg);
                    }
                }
            } else if world_x.rem_euclid(10) == 0 {
                put_terrain_texture(grid, cols, x, top_y, '▌', shadow_fg);
            }
        }
        BiomeKind::Coast => {
            if world_x.rem_euclid(5) == 0 {
                put_terrain_texture(grid, cols, x, top_y, '~', texture_fg);
            }
            if layer.depth == TerrainDepth::Near && world_x.rem_euclid(17) == 0 {
                put_terrain_texture(grid, cols, x, bottom_y, '~', rgb(188, 220, 210));
            }
        }
    }
}

fn put_terrain_texture(grid: &mut [CellFmt], cols: usize, x: usize, y: usize, ch: char, fg: Color) {
    if cols == 0 || x >= cols || y >= grid.len() / cols {
        return;
    }
    let i = y * cols + x;
    grid[i] = CellFmt {
        ch,
        fg,
        bg: grid[i].bg,
    };
}

fn draw_biome_details(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    horizon: usize,
    sky: SkyState,
    phase: f32,
) {
    if horizon + 2 >= rows {
        return;
    }

    for biome in BIOMES {
        draw_biome_detail_layer(
            grid,
            cols,
            BiomeDetailLayer {
                rows,
                horizon,
                sky,
                kind: biome.kind,
                phase,
            },
        );
    }
}

fn draw_biome_detail_layer(grid: &mut [CellFmt], cols: usize, layer: BiomeDetailLayer) {
    let biome = biome_for(layer.kind);
    let fg = blend(
        biome_lit(biome.accent, layer.sky.palette.horizon),
        biome_lit(biome.ground, layer.sky.palette.ground),
        0.35,
    );
    let near = biome_lit(biome.near, layer.sky.palette.ground_dark);

    match layer.kind {
        BiomeKind::Meadow => {
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 9, cols) else {
                    continue;
                };
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 12, 1) else {
                    continue;
                };
                let ch = if detail_hash(slot, 13).is_multiple_of(2) {
                    ','
                } else {
                    '\''
                };
                put_detail(grid, cols, x as i32, base_y, ch, fg);
            }
        }
        BiomeKind::Forest => {
            let foliage = biome_lit(rgb(132, 198, 104), layer.sky.palette.ground_dark);
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 7, cols) else {
                    continue;
                };
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 22, 2) else {
                    continue;
                };
                draw_pine(grid, cols, x as i32, base_y, foliage);
            }
        }
        BiomeKind::Mountains => {
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 18, cols) else {
                    continue;
                };
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 32, 1) else {
                    continue;
                };
                draw_peak(grid, cols, x as i32, base_y, fg);
            }
        }
        BiomeKind::Desert => {
            let tumbleweed_fg = blend(rgb(188, 132, 62), layer.sky.palette.ground, 0.18);
            draw_tumbleweeds(grid, cols, layer, tumbleweed_fg);

            let cactus_fg = blend(rgb(42, 156, 68), layer.sky.palette.ground_dark, 0.18);
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 15, cols) else {
                    continue;
                };
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 42, 3) else {
                    continue;
                };
                draw_cactus(grid, cols, x as i32, base_y, cactus_fg);
            }
        }
        BiomeKind::Canyon => {
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 10, cols) else {
                    continue;
                };
                let height = 1 + (detail_hash(slot, 52) % 3) as i32;
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 53, height)
                else {
                    continue;
                };
                for y in 0..height {
                    put_detail(grid, cols, x as i32, base_y - y, '▉', fg);
                }
            }
        }
        BiomeKind::Tundra => {
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 8, cols) else {
                    continue;
                };
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 62, 1) else {
                    continue;
                };
                put_detail(grid, cols, x as i32, base_y, '.', fg);
            }
        }
        BiomeKind::City => {
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 8, cols) else {
                    continue;
                };
                let width = 2 + (detail_hash(slot, 72) % 3) as i32;
                let height = 2 + (detail_hash(slot, 73) % 2) as i32;
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 74, height)
                else {
                    continue;
                };
                draw_building(
                    grid,
                    cols,
                    BuildingSpec {
                        x: x as i32,
                        base_y,
                        width,
                        height,
                        wall: near,
                        light: fg,
                    },
                );
            }
        }
        BiomeKind::Coast => {
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 5, cols) else {
                    continue;
                };
                if let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 82, 1) {
                    put_detail(grid, cols, x as i32, base_y, '~', fg);
                }
                if let (true, Some(base_y)) = (
                    detail_hash(slot, 82).is_multiple_of(5),
                    foreground_base_y(layer.rows, layer.horizon, slot, 84, 1),
                ) {
                    put_detail(grid, cols, x as i32, base_y, '╷', near);
                }
            }
        }
    }
}

fn biome_for(kind: BiomeKind) -> Biome {
    match kind {
        BiomeKind::Meadow => BIOMES[0],
        BiomeKind::Forest => BIOMES[1],
        BiomeKind::Mountains => BIOMES[2],
        BiomeKind::Desert => BIOMES[3],
        BiomeKind::Canyon => BIOMES[4],
        BiomeKind::Tundra => BIOMES[5],
        BiomeKind::City => BIOMES[6],
        BiomeKind::Coast => BIOMES[7],
    }
}

fn detail_slot_for_kind(
    x: usize,
    phase: f32,
    kind: BiomeKind,
    spacing: i32,
    cols: usize,
) -> Option<i32> {
    let scale = detail_phase_scale(kind);
    let slot = detail_slot(x, phase * scale, spacing)?;
    let world = slot * spacing;
    if detail_biome_for_layer_world(world, slot, scale, cols) == kind {
        Some(slot)
    } else {
        None
    }
}

fn detail_phase_scale(kind: BiomeKind) -> f32 {
    match kind {
        BiomeKind::Mountains => 0.55,
        BiomeKind::Canyon => 0.7,
        BiomeKind::City => 0.45,
        _ => 1.0,
    }
}

fn detail_biome_for_layer_world(world: i32, slot: i32, phase_scale: f32, cols: usize) -> BiomeKind {
    let transition_distance = BIOME_TRANSITION_DISTANCE * phase_scale;
    let blend_width = BIOME_TRANSITION_DISTANCE * BIOME_BLEND_FRACTION * phase_scale;
    let band_start_offset = BIOME_BLEND_START_DISTANCE * phase_scale + cols as f32;
    let world = world as f32;
    let transition_segment = ((world - band_start_offset) / transition_distance).floor();

    if world >= 0.0 && transition_segment < 0.0 {
        return BIOMES[0].kind;
    }

    let from_idx = (transition_segment as i32).rem_euclid(BIOMES.len() as i32) as usize;
    let to_idx = (from_idx + 1) % BIOMES.len();
    let local = world - (transition_segment * transition_distance + band_start_offset);

    if local >= blend_width {
        return BIOMES[to_idx].kind;
    }

    let mix = smoothstep(local / blend_width);
    let threshold = (mix * 100.0).round() as u32;
    if detail_hash(slot, 0xB10E_D17A) % 100 < threshold {
        BIOMES[to_idx].kind
    } else {
        BIOMES[from_idx].kind
    }
}

fn detail_slot(x: usize, phase: f32, spacing: i32) -> Option<i32> {
    let world = x as i32 + phase.round() as i32;
    if world.rem_euclid(spacing) == 0 {
        Some(world.div_euclid(spacing))
    } else {
        None
    }
}

fn foreground_base_y(
    rows: usize,
    horizon: usize,
    slot: i32,
    salt: u32,
    height: i32,
) -> Option<i32> {
    let top = horizon.checked_add(2)? as i32;
    let bottom = rows.checked_sub(1)? as i32;
    let min_base = top + height - 1;
    if min_base > bottom {
        return None;
    }

    let lanes = (bottom - min_base + 1) as u32;
    Some(min_base + (detail_hash(slot, salt) % lanes) as i32)
}

fn detail_hash(slot: i32, salt: u32) -> u32 {
    let mut n = slot as u32 ^ salt.wrapping_mul(0x9E37_79B9);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846C_A68B);
    n ^ (n >> 16)
}

fn put_detail(grid: &mut [CellFmt], cols: usize, x: i32, y: i32, ch: char, fg: Color) {
    if x < 0 || y < 0 || x >= cols as i32 {
        return;
    }
    let rows = grid.len() / cols;
    if y as usize >= rows {
        return;
    }
    let i = y as usize * cols + x as usize;
    grid[i] = CellFmt {
        ch,
        fg,
        bg: grid[i].bg,
    };
}

fn draw_pine(grid: &mut [CellFmt], cols: usize, x: i32, base_y: i32, fg: Color) {
    put_detail(grid, cols, x, base_y - 1, '^', fg);
    put_detail(grid, cols, x - 1, base_y, '^', fg);
    put_detail(grid, cols, x, base_y, '|', fg);
    put_detail(grid, cols, x + 1, base_y, '^', fg);
}

fn draw_peak(grid: &mut [CellFmt], cols: usize, x: i32, y: i32, fg: Color) {
    put_detail(grid, cols, x, y, '^', fg);
    put_detail(grid, cols, x - 1, y + 1, '/', fg);
    put_detail(grid, cols, x + 1, y + 1, '\\', fg);
}

fn draw_cactus(grid: &mut [CellFmt], cols: usize, x: i32, base_y: i32, fg: Color) {
    put_detail(grid, cols, x, base_y - 2, '│', fg);
    put_detail(grid, cols, x - 1, base_y - 2, '┤', fg);
    put_detail(grid, cols, x + 1, base_y - 1, '├', fg);
    put_detail(grid, cols, x, base_y - 1, '│', fg);
    put_detail(grid, cols, x, base_y, '│', fg);
}

fn draw_tumbleweeds(grid: &mut [CellFmt], cols: usize, layer: BiomeDetailLayer, fg: Color) {
    if dominant_biome_kind(layer.phase) != BiomeKind::Desert {
        return;
    }

    let cycle = cols as f32 + 34.0;
    let count = (cols / 38).max(3) + 1;
    for i in 0..count {
        let seed = detail_hash(i as i32, 44);
        let offset = (seed % (cycle as u32 * 10)) as f32 / 10.0;
        let x = cols as f32 + 3.0 - (layer.sky.elapsed * 2.4 + offset).rem_euclid(cycle);
        let x = x.round() as i32;
        if x < -2 || x > cols as i32 + 2 {
            continue;
        }

        let lane_slot = i as i32 + (seed % 97) as i32;
        let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, lane_slot, 45, 1) else {
            continue;
        };
        let frame = ((layer.sky.elapsed * 3.0 + offset * 0.13) as u32) % 4;
        draw_tumbleweed(grid, cols, x, base_y, fg, frame);
    }
}

fn draw_tumbleweed(grid: &mut [CellFmt], cols: usize, x: i32, base_y: i32, fg: Color, frame: u32) {
    let chars = match frame {
        0 => ['-', 'o', '-'],
        1 => ['\\', '@', '/'],
        2 => ['_', 'O', '_'],
        _ => ['/', '@', '\\'],
    };
    for (dx, ch) in [(-1, chars[0]), (0, chars[1]), (1, chars[2])] {
        put_detail(grid, cols, x + dx, base_y, ch, fg);
    }
}

fn draw_building(grid: &mut [CellFmt], cols: usize, spec: BuildingSpec) {
    for dx in 0..spec.width {
        for dy in 0..spec.height {
            let ch = if (dx + dy) % 3 == 0 { '░' } else { '█' };
            let fg = if ch == '░' { spec.light } else { spec.wall };
            put_detail(grid, cols, spec.x + dx, spec.base_y - dy, ch, fg);
        }
    }
}

fn draw_tracks(grid: &mut [CellFmt], cols: usize, rows: usize, top_y: usize, phase: f32) {
    if top_y >= rows {
        return;
    }

    let tie_offset = (phase.round() as i32).rem_euclid(12) as usize;
    for c in 0..cols {
        let i = top_y * cols + c;
        let tie = (c + tie_offset).is_multiple_of(12);
        grid[i] = CellFmt {
            ch: if tie { '#' } else { '-' },
            fg: if tie { TRACK_TIE } else { TRACK_RAIL },
            bg: grid[i].bg,
        };
    }

    let rail_y = top_y + 1;
    if rail_y >= rows {
        return;
    }

    for c in 0..cols {
        let i = rail_y * cols + c;
        grid[i] = CellFmt {
            ch: '=',
            fg: TRACK_RAIL,
            bg: grid[i].bg,
        };
    }
}

fn draw_stars(grid: &mut [CellFmt], cols: usize, horizon: usize, sky: SkyState) {
    if sky.stars <= 0.05 {
        return;
    }
    let star_rows = horizon.saturating_sub(2).max(1);
    for i in 0..(cols / 5).max(12) {
        let x = (detail_hash(i as i32, 0x5354_4152) % cols as u32) as usize;
        let y = 1 + (detail_hash(i as i32, 0x0053_4B59) % star_rows as u32) as usize;
        if y >= horizon {
            continue;
        }
        let ch = if detail_hash(i as i32, 0x2A2A_2A2A) % 4 == 0 {
            '*'
        } else {
            '.'
        };
        let fg = blend(sky.palette.top, rgb(245, 245, 210), sky.stars);
        grid[y * cols + x] = CellFmt {
            ch,
            fg,
            bg: grid[y * cols + x].bg,
        };
    }
}

fn draw_precipitation(
    grid: &mut [CellFmt],
    cols: usize,
    horizon: usize,
    sky: SkyState,
    weather: WeatherState,
) {
    if weather.intensity <= 0.05 {
        return;
    }

    match weather.kind {
        PrecipitationKind::Rain => draw_rain(grid, cols, horizon, sky, weather.intensity),
        PrecipitationKind::Snow => draw_snow(grid, cols, horizon, sky, weather.intensity),
    }
}

fn draw_rain(grid: &mut [CellFmt], cols: usize, horizon: usize, sky: SkyState, intensity: f32) {
    if cols == 0 || intensity <= 0.05 {
        return;
    }

    let cycle = horizon as f32 + 4.0;
    let fg = blend(rgb(120, 150, 180), rgb(190, 210, 230), intensity);
    let drops = ((cols as f32 / 2.8) * intensity.max(0.35)) as usize;
    for drop in 0..drops.max(18) {
        let seed = detail_hash(drop as i32, 0x0A1D_2026);
        let x_seed = (seed % cols.max(1) as u32) as i32;
        let offset = (detail_hash(drop as i32, 0x51A7_2026) % (cycle as u32 * 10)) as f32 / 10.0;
        let y = (sky.elapsed * 5.2 + offset).rem_euclid(cycle) - 2.0;
        let y = y.round() as i32;
        if y < 1 || y >= horizon as i32 {
            continue;
        }

        let x = (x_seed + y.div_euclid(3)).rem_euclid(cols as i32) as usize;
        let i = y as usize * cols + x;
        grid[i] = CellFmt {
            ch: '/',
            fg,
            bg: grid[i].bg,
        };
    }
}

fn draw_snow(grid: &mut [CellFmt], cols: usize, horizon: usize, sky: SkyState, intensity: f32) {
    if cols == 0 || intensity <= 0.05 {
        return;
    }

    let cycle = horizon as f32 + 3.0;
    let fg = blend(rgb(205, 225, 235), rgb(250, 250, 255), intensity);
    let flakes = ((cols as f32 / 3.6) * intensity.max(0.45)) as usize;
    for flake in 0..flakes.max(14) {
        let seed = detail_hash(flake as i32, 0x5A10_2026);
        let x_seed = (seed % cols.max(1) as u32) as i32;
        let offset = (detail_hash(flake as i32, 0xF1A7_2026) % (cycle as u32 * 10)) as f32 / 10.0;
        let y_float = (sky.elapsed * 1.45 + offset).rem_euclid(cycle) - 2.0;
        let y = y_float.round() as i32;
        if y < 1 || y >= horizon as i32 {
            continue;
        }

        let drift = ((sky.elapsed * 0.8 + flake as f32 * 0.7 + y_float * 0.4).sin() * 2.0).round();
        let x = (x_seed + drift as i32).rem_euclid(cols as i32) as usize;
        let i = y as usize * cols + x;
        grid[i] = CellFmt {
            ch: if seed.is_multiple_of(5) { '*' } else { '.' },
            fg,
            bg: grid[i].bg,
        };
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
            fg: UNDERCARRIAGE,
            bg,
        }),
        '=' => Some(CellFmt {
            ch,
            fg: UNDERCARRIAGE,
            bg,
        }),
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

#[derive(Clone, Copy)]
struct TextStyle {
    fg: Color,
    bg: Color,
}

fn put_text(
    grid: &mut [CellFmt],
    cols: usize,
    rows: usize,
    text: &str,
    left_x: i32,
    y: usize,
    style: TextStyle,
) {
    if y >= rows {
        return;
    }
    for (i, ch) in text.chars().enumerate() {
        let x = left_x + i as i32;
        if x < 0 || x >= cols as i32 {
            continue;
        }
        grid[y * cols + x as usize] = CellFmt {
            ch,
            fg: style.fg,
            bg: style.bg,
        };
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
    (0..cols).for_each(|c| {
        grid[c] = CellFmt {
            ch: ' ',
            fg: Color::White,
            bg,
        };
    });
    let left = " ←/→ chugga chugga   SPACE choo choo   EXIT or QUIT to exit";
    put_text(
        grid,
        cols,
        rows,
        left,
        0,
        0,
        TextStyle {
            fg: Color::White,
            bg,
        },
    );
    if game.celebrating() {
        let right = "Another wheel! ";
        let right_x = cols as i32 - right.chars().count() as i32;
        put_text(
            grid,
            cols,
            rows,
            right,
            right_x.max(0),
            0,
            TextStyle {
                fg: Color::White,
                bg,
            },
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_transition_takes_half_day_cycle_at_full_speed() {
        let (start_current, start_next, start_mix) = biome_transition(0.0);
        let (half_current, half_next, half_mix) = biome_transition(BIOME_TRANSITION_DISTANCE * 0.5);
        let (blend_start_current, blend_start_next, blend_start_mix) =
            biome_transition(BIOME_TRANSITION_DISTANCE * (1.0 - BIOME_BLEND_FRACTION));
        let (blend_mid_current, blend_mid_next, blend_mid_mix) =
            biome_transition(BIOME_TRANSITION_DISTANCE * (1.0 - BIOME_BLEND_FRACTION * 0.5));
        let (next_current, next_next, next_mix) = biome_transition(BIOME_TRANSITION_DISTANCE);

        assert_eq!(start_current.kind, BiomeKind::Meadow);
        assert_eq!(start_next.kind, BiomeKind::Forest);
        assert_eq!(start_mix, 0.0);
        assert_eq!(half_current.kind, BiomeKind::Meadow);
        assert_eq!(half_next.kind, BiomeKind::Forest);
        assert_eq!(half_mix, 0.0);
        assert_eq!(blend_start_current.kind, BiomeKind::Meadow);
        assert_eq!(blend_start_next.kind, BiomeKind::Forest);
        assert_eq!(blend_start_mix, 0.0);
        assert_eq!(blend_mid_current.kind, BiomeKind::Meadow);
        assert_eq!(blend_mid_next.kind, BiomeKind::Forest);
        assert!((blend_mid_mix - 0.5).abs() < f32::EPSILON);
        assert_eq!(next_current.kind, BiomeKind::Forest);
        assert_eq!(next_next.kind, BiomeKind::Mountains);
        assert_eq!(next_mix, 0.0);
    }

    #[test]
    fn tundra_weather_uses_snow_instead_of_rain() {
        let rain_time = DAY_CYCLE_SECS * 0.43;
        let meadow = weather_state(0.0, rain_time);
        let tundra = weather_state(BIOME_TRANSITION_DISTANCE * 5.0, rain_time);

        assert_eq!(meadow.kind, PrecipitationKind::Rain);
        assert!(meadow.rain_audio_intensity() > 0.9);
        assert_eq!(tundra.kind, PrecipitationKind::Snow);
        assert_eq!(tundra.rain_audio_intensity(), 0.0);
    }

    #[test]
    fn terrain_layers_draw_biome_specific_texture() {
        let cols = 100;
        let rows = 40;
        let horizon = 20;
        let sky = sky_state(0.0);

        for (idx, biome) in BIOMES.iter().enumerate() {
            let distance = BIOME_TRANSITION_DISTANCE * idx as f32;
            let visual = biome_visual(distance);
            let mut grid = vec![BLANK; cols * rows];
            let expected_chars: &[char] = match biome.kind {
                BiomeKind::Meadow => &['\'', '.'],
                BiomeKind::Forest => &['^', '|'],
                BiomeKind::Mountains => &['^', '/', '\\'],
                BiomeKind::Desert => &['-'],
                BiomeKind::Canyon => &['─'],
                BiomeKind::Tundra => &['.', '_'],
                BiomeKind::City => &['▪', '▌'],
                BiomeKind::Coast => &['~'],
            };

            assert_eq!(visual.kind, biome.kind);
            draw_terrain(&mut grid, cols, horizon, sky, visual, distance, true);

            assert!(
                grid.iter().any(|cell| expected_chars.contains(&cell.ch)),
                "{:?} terrain drew no biome texture",
                biome.kind
            );
        }
    }

    #[test]
    fn cactus_overdraws_overlapping_tumbleweed() {
        let cols = 20;
        let rows = 10;
        let mut grid = vec![BLANK; cols * rows];
        let cactus_fg = rgb(42, 156, 68);
        let base_y = 6;

        draw_tumbleweed(&mut grid, cols, 8, base_y - 2, rgb(188, 132, 62), 1);
        draw_cactus(&mut grid, cols, 8, base_y, cactus_fg);

        for (x, y, ch) in [
            (8, base_y - 2, '│'),
            (7, base_y - 2, '┤'),
            (9, base_y - 1, '├'),
            (8, base_y - 1, '│'),
            (8, base_y, '│'),
        ] {
            let cell = grid[y as usize * cols + x as usize];
            assert_eq!(cell.ch, ch);
            assert_eq!(cell.fg, cactus_fg);
        }
    }

    #[test]
    fn biome_detail_transition_starts_beyond_visible_slots() {
        let cols = 80;
        let phase_scale = detail_phase_scale(BiomeKind::Meadow);
        let first_visible = (BIOME_BLEND_START_DISTANCE * phase_scale).round() as i32;

        for world in first_visible..first_visible + cols as i32 {
            assert_eq!(
                detail_biome_for_layer_world(world, world, phase_scale, cols),
                BiomeKind::Meadow
            );
        }
    }

    #[test]
    fn slow_biome_detail_layers_keep_visible_slots_after_boundary() {
        let cols = 200;
        let phase_scale = detail_phase_scale(BiomeKind::City);
        let city_idx = 6.0;
        let boundary = ((city_idx + 1.0) * BIOME_TRANSITION_DISTANCE * phase_scale).round() as i32;

        assert_eq!(
            detail_biome_for_layer_world(boundary, boundary, phase_scale, cols),
            BiomeKind::City
        );
    }

    #[test]
    fn biome_detail_transition_band_intermingles_biomes() {
        let cols = 80;
        let phase_scale = detail_phase_scale(BiomeKind::Meadow);
        let band_start = (BIOME_BLEND_START_DISTANCE * phase_scale + cols as f32).round() as i32;
        let band_end = band_start
            + (BIOME_TRANSITION_DISTANCE * BIOME_BLEND_FRACTION * phase_scale).round() as i32;
        let mut saw_meadow = false;
        let mut saw_forest = false;

        for world in band_start..band_end {
            match detail_biome_for_layer_world(world, world, phase_scale, cols) {
                BiomeKind::Meadow => saw_meadow = true,
                BiomeKind::Forest => saw_forest = true,
                other => panic!("unexpected biome in first transition band: {other:?}"),
            }
        }

        assert!(saw_meadow);
        assert!(saw_forest);
    }

    #[test]
    fn biome_details_stay_below_tracks_and_skip_asterisks() {
        let cols = 80;
        let rows = 40;
        let horizon = 20;

        for (idx, biome) in BIOMES.iter().enumerate() {
            let phase = BIOME_TRANSITION_DISTANCE * idx as f32;
            let (current, _, _) = biome_transition(phase);
            let mut grid = vec![BLANK; cols * rows];

            assert_eq!(current.kind, biome.kind);

            draw_biome_details(&mut grid, cols, rows, horizon, sky_state(0.0), phase);

            for r in 0..=horizon + 1 {
                for c in 0..cols {
                    assert_eq!(
                        grid[r * cols + c].ch,
                        ' ',
                        "{:?} wrote above foreground at {},{}",
                        biome.kind,
                        c,
                        r
                    );
                }
            }
            assert!(
                grid[(horizon + 2) * cols..]
                    .iter()
                    .any(|cell| cell.ch != ' '),
                "{:?} drew no foreground details",
                biome.kind
            );
            assert!(
                !grid.iter().any(|cell| cell.ch == '*'),
                "{:?} drew an asterisk detail",
                biome.kind
            );
        }
    }

    #[test]
    fn rgb_to_ansi256_maps_into_palette() {
        assert_eq!(rgb_to_ansi256(0, 0, 0), 16); // black cube corner
        assert_eq!(rgb_to_ansi256(255, 255, 255), 231); // white cube corner
        assert_eq!(rgb_to_ansi256(128, 128, 128), 244); // gray ramp exact
        assert_eq!(rgb_to_ansi256(255, 0, 0), 196); // pure red cube
        assert_eq!(rgb_to_ansi256(0, 0, 255), 21); // pure blue cube
        // Colorful values must use the cube, never wash out to the gray ramp.
        assert!(rgb_to_ansi256(58, 116, 47) < 232); // meadow ground stays green
        assert!(rgb_to_ansi256(75, 80, 86) >= 232); // neutral metal stays gray
        for r in (0u16..=255).step_by(17) {
            for g in (0u16..=255).step_by(17) {
                for b in (0u16..=255).step_by(17) {
                    let v = rgb_to_ansi256(r as u8, g as u8, b as u8);
                    assert!((16..=255).contains(&v), "out of range for {r},{g},{b}: {v}");
                }
            }
        }
    }
}

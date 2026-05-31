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
        "----#----(0)======(0)======(0)======(0)======(0)======#-------------       ",
        "========={===}======={===}======={===}======={===}=================       ",
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
        "====#====(0)==========(0)====#--------",
        "======={===}=========={===}===========",
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
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
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
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
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
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
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
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
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
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
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
        "--#====(0)==========(0)====#------",
        "======{===}========{===}=========",
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
    ground: Color,
    ground_dark: Color,
    far: Color,
    near: Color,
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
struct BiomeState {
    current: BiomeKind,
    next: BiomeKind,
    mix: f32,
    visual: BiomeVisual,
}

#[derive(Clone, Copy)]
struct TerrainLayer {
    color: Color,
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
    phase_time: f32,
    rain: f32,
    stars: f32,
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
        let biome = biome_state(game.distance_traveled);

        draw_sky(&mut self.grid, cols, rows, horizon, sky, biome.visual);
        draw_clouds(&mut self.grid, cols, rows, horizon, sky);
        draw_stars(&mut self.grid, cols, horizon, sky);
        draw_terrain(
            &mut self.grid,
            cols,
            horizon,
            sky,
            biome.visual,
            game.distance_traveled,
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

fn biome_state(distance: f32) -> BiomeState {
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

    BiomeState {
        current: current.kind,
        next: next.kind,
        mix,
        visual: blend_biome(current, next, mix),
    }
}

fn blend_biome(a: Biome, b: Biome, t: f32) -> BiomeVisual {
    BiomeVisual {
        ground: blend(a.ground, b.ground, t),
        ground_dark: blend(a.ground_dark, b.ground_dark, t),
        far: blend(a.far, b.far, t),
        near: blend(a.near, b.near, t),
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

    // Far ridge: washed out toward horizon (atmospheric perspective)
    draw_terrain_layer(
        grid,
        cols,
        horizon,
        TerrainLayer {
            color: far,
            scroll: phase * 0.15 * TERRAIN_FAR_SCROLL_FREQ,
            freq: biome.far_freq,
            base: biome.far_base,
            amp: biome.far_amp,
            detail_amp: biome.far_detail,
        },
    );
    // Near hills: slightly darker than foreground grass, not pitch-black
    draw_terrain_layer(
        grid,
        cols,
        horizon,
        TerrainLayer {
            color: near,
            scroll: phase * 0.4 * TERRAIN_NEAR_SCROLL_FREQ,
            freq: biome.near_freq,
            base: biome.near_base,
            amp: biome.near_amp,
            detail_amp: biome.near_detail,
        },
    );
}

fn draw_terrain_layer(grid: &mut [CellFmt], cols: usize, horizon: usize, layer: TerrainLayer) {
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
        if frac > 0.125 && height_int < max_height {
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
    }
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
            for x in 0..cols {
                let Some(slot) = detail_slot_for_kind(x, layer.phase, layer.kind, 15, cols) else {
                    continue;
                };
                let Some(base_y) = foreground_base_y(layer.rows, layer.horizon, slot, 42, 3) else {
                    continue;
                };
                draw_cactus(grid, cols, x as i32, base_y, fg);
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

fn draw_rain(grid: &mut [CellFmt], cols: usize, _rows: usize, horizon: usize, sky: SkyState) {
    if sky.rain <= 0.05 {
        return;
    }

    let offset = (sky.phase_time * 12.0) as usize;
    let fg = blend(rgb(120, 150, 180), rgb(190, 210, 230), sky.rain);
    for y in 1..horizon {
        for x in (0..cols).step_by(4) {
            let px = (x + ((y * 3 + offset) % 4)) % cols;
            if !(px + y + offset).is_multiple_of(3) {
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
        let start = biome_state(0.0);
        let half = biome_state(BIOME_TRANSITION_DISTANCE * 0.5);
        let blend_start = biome_state(BIOME_TRANSITION_DISTANCE * (1.0 - BIOME_BLEND_FRACTION));
        let blend_mid = biome_state(BIOME_TRANSITION_DISTANCE * (1.0 - BIOME_BLEND_FRACTION * 0.5));
        let next = biome_state(BIOME_TRANSITION_DISTANCE);

        assert_eq!(start.current, BiomeKind::Meadow);
        assert_eq!(start.next, BiomeKind::Forest);
        assert_eq!(start.mix, 0.0);
        assert_eq!(half.current, BiomeKind::Meadow);
        assert_eq!(half.next, BiomeKind::Forest);
        assert_eq!(half.mix, 0.0);
        assert_eq!(blend_start.current, BiomeKind::Meadow);
        assert_eq!(blend_start.next, BiomeKind::Forest);
        assert_eq!(blend_start.mix, 0.0);
        assert_eq!(blend_mid.current, BiomeKind::Meadow);
        assert_eq!(blend_mid.next, BiomeKind::Forest);
        assert!((blend_mid.mix - 0.5).abs() < f32::EPSILON);
        assert_eq!(next.current, BiomeKind::Forest);
        assert_eq!(next.next, BiomeKind::Mountains);
        assert_eq!(next.mix, 0.0);
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
            let state = biome_state(phase);
            let mut grid = vec![BLANK; cols * rows];

            assert_eq!(state.current, biome.kind);

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
}

use std::time::Instant;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

pub const MAX_CARS: usize = 24;
pub const SEG_HEIGHT: usize = 8;
pub const SMOKE_RISE: f32 = -8.0;
pub const SMOKE_MAX_AGE: f32 = 3.0;
pub const SMOKE_SPAWN_INTERVAL: f32 = 0.08;
pub const TRAIN_SPEED_CELLS_PER_SEC: f32 = 32.0;
pub const TRAIN_KEEP_MOVING_MS: u128 = 650;
pub const HORN_FLASH_MS: u128 = 600;
pub const CELEBRATE_MS: u128 = 900;
/// Small overshoot past the right edge before the engine wraps back to the
/// left. Tuned to feel like "engine just goes off, then comes back".
pub const SHORT_TRAIN_OVERSHOOT: i32 = 4;
/// Gap between caboose exiting the right and engine re-emerging on the left,
/// for trains longer than the screen.
pub const LONG_TRAIN_TAIL_GAP: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarKind {
    Boxcar,
    Tanker,
    Hopper,
    Passenger,
    Flatcar,
    Gondola,
}

impl CarKind {
    pub fn rotate(idx: usize) -> Self {
        const KINDS: &[CarKind] = &[
            CarKind::Boxcar,
            CarKind::Tanker,
            CarKind::Passenger,
            CarKind::Hopper,
            CarKind::Flatcar,
            CarKind::Gondola,
        ];
        KINDS[idx % KINDS.len()]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CarColor {
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    Orange,
    Brown,
    Olive,
}

impl CarColor {
    pub fn rotate(idx: usize) -> Self {
        const COLORS: &[CarColor] = &[
            CarColor::Yellow,
            CarColor::Green,
            CarColor::Cyan,
            CarColor::Blue,
            CarColor::Magenta,
            CarColor::Orange,
            CarColor::Brown,
            CarColor::Olive,
        ];
        COLORS[idx % COLORS.len()]
    }
}

#[derive(Clone, Debug)]
pub struct Car {
    pub kind: CarKind,
    pub color: CarColor,
}

#[derive(Clone, Debug)]
pub struct Smoke {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub age: f32,
}

pub struct Game {
    pub screen_cols: u16,
    pub screen_rows: u16,

    pub head_x: f32,
    pub velocity: f32,

    pub cars: Vec<Car>,

    pub smoke: Vec<Smoke>,
    smoke_spawn_accum: f32,

    pub rng: SmallRng,
    pub last_tick: Instant,

    pub last_move: Option<Instant>,
    pub last_horn: Option<Instant>,
    pub last_celebrate: Option<Instant>,

    pub quit: bool,
}

impl Game {
    pub fn new(cols: u16, rows: u16) -> Self {
        let now = Instant::now();
        Self {
            screen_cols: cols,
            screen_rows: rows,
            head_x: 0.0,
            velocity: 0.0,
            cars: Vec::new(),
            smoke: Vec::new(),
            smoke_spawn_accum: 0.0,
            rng: SmallRng::from_entropy(),
            last_tick: now,
            last_move: None,
            last_horn: None,
            last_celebrate: None,
            quit: false,
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.screen_cols = cols;
        self.screen_rows = rows;
        let c = self.cycle();
        if c > 0 {
            self.head_x = self.head_x.rem_euclid(c as f32);
        }
    }

    /// Distance the head must travel (in cells) before wrapping back to 0.
    ///
    /// - **Short train** (fits on screen): wrap right after the engine
    ///   pokes off the right edge. Short visual jolt is acceptable.
    /// - **Long train** (wider than screen): wait until the caboose has
    ///   fully exited the right plus a small gap, then re-emerge on the left.
    pub fn cycle(&self) -> i32 {
        let screen = self.screen_cols as i32;
        let train = self.train_total_width();
        if train <= screen {
            // Engine pokes off the right by a few cells then wraps. There's
            // a brief visual jolt (caboose teleports off-screen) — the user
            // explicitly asked for "wrap as soon as engine goes off" over
            // "wait for the whole train".
            screen + SHORT_TRAIN_OVERSHOOT
        } else {
            // Wait until the caboose has fully cleared the right edge, then
            // a small empty gap before the engine re-emerges on the left.
            // Caboose's LEFT edge reaches the screen's RIGHT edge when
            // head_x = screen + train - 1.
            screen + train - 1 + LONG_TRAIN_TAIL_GAP
        }
    }

    pub fn train_total_width(&self) -> i32 {
        let mut w = crate::renderer::ENGINE.width as i32;
        for car in &self.cars {
            w += crate::renderer::car_sprite(car.kind).width as i32;
        }
        w += crate::renderer::CABOOSE.width as i32;
        w
    }

    /// Top row of the train. Puts it ~40% down the screen so there's plenty
    /// of sky for smoke and a grass apron below.
    pub fn train_top(&self) -> usize {
        let rows = self.screen_rows as usize;
        let from_top = (rows as f32 * 0.4) as usize;
        from_top.max(6).min(rows.saturating_sub(SEG_HEIGHT + 1))
    }

    pub fn engine_smokestack_world_x(&self) -> f32 {
        let smokestack_col_in_sprite = 27_i32;
        let engine_left = self.head_x - (crate::renderer::ENGINE.width as f32 - 1.0);
        engine_left + smokestack_col_in_sprite as f32
    }

    pub fn engine_smokestack_top_row(&self) -> i32 {
        self.train_top() as i32
    }

    /// Returns the number of cars added this tick (0 or 1 in practice).
    /// Caller plays audio based on this.
    pub fn tick(&mut self) -> u32 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().min(0.1);
        self.last_tick = now;

        let moving = matches!(self.last_move, Some(t) if now.duration_since(t).as_millis() < TRAIN_KEEP_MOVING_MS);
        if !moving {
            self.velocity *= 0.5_f32.powf(dt * 10.0);
            if self.velocity.abs() < 0.05 {
                self.velocity = 0.0;
            }
        }

        let cycle = self.cycle() as f32;
        let prev_head = self.head_x;
        let unwrapped = prev_head + self.velocity * dt;
        self.head_x = if cycle > 0.0 {
            unwrapped.rem_euclid(cycle)
        } else {
            unwrapped
        };

        // Wrap-around event = one new car.
        let cars_added = if cycle > 0.0
            && self.velocity.abs() > 0.5
            && ((self.velocity > 0.0 && unwrapped >= cycle)
                || (self.velocity < 0.0 && unwrapped < 0.0))
        {
            if self.add_car() { 1 } else { 0 }
        } else {
            0
        };

        self.smoke_spawn_accum += dt;
        while self.smoke_spawn_accum >= SMOKE_SPAWN_INTERVAL {
            self.smoke_spawn_accum -= SMOKE_SPAWN_INTERVAL;
            self.spawn_smoke_puff();
        }

        let wind = -self.velocity * 0.7;
        self.smoke.retain_mut(|s| {
            s.x += (s.vx + wind) * dt;
            s.y += s.vy * dt;
            s.vy *= (0.6_f32).powf(dt);
            s.age += dt;
            s.age < SMOKE_MAX_AGE
        });

        cars_added
    }

    fn spawn_smoke_puff(&mut self) {
        let base_x = self.engine_smokestack_world_x();
        let base_y = self.engine_smokestack_top_row() as f32 - 0.5;
        let n = self.rng.gen_range(2..=4);
        for _ in 0..n {
            let dx: f32 = self.rng.gen_range(-0.8..0.8);
            let dy: f32 = self.rng.gen_range(-0.3..0.3);
            let vx: f32 = self.rng.gen_range(-1.5..1.5);
            let vy: f32 = SMOKE_RISE + self.rng.gen_range(-1.5..1.5);
            self.smoke.push(Smoke {
                x: base_x + dx,
                y: base_y + dy,
                vx,
                vy,
                age: 0.0,
            });
        }
    }

    fn add_car(&mut self) -> bool {
        if self.cars.len() >= MAX_CARS {
            return false;
        }
        let idx = self.cars.len();
        self.cars.push(Car {
            kind: CarKind::rotate(idx),
            color: CarColor::rotate(idx),
        });
        self.last_celebrate = Some(Instant::now());
        true
    }

    pub fn nudge_forward(&mut self) {
        self.velocity = TRAIN_SPEED_CELLS_PER_SEC;
        self.last_move = Some(Instant::now());
    }

    pub fn nudge_backward(&mut self) {
        self.velocity = -TRAIN_SPEED_CELLS_PER_SEC;
        self.last_move = Some(Instant::now());
    }

    pub fn moving_recently(&self) -> bool {
        matches!(self.last_move, Some(t) if t.elapsed().as_millis() < TRAIN_KEEP_MOVING_MS + 200)
    }

    pub fn horn(&mut self) {
        self.last_horn = Some(Instant::now());
    }

    pub fn horn_active(&self) -> bool {
        matches!(self.last_horn, Some(t) if t.elapsed().as_millis() < HORN_FLASH_MS)
    }

    pub fn celebrating(&self) -> bool {
        matches!(self.last_celebrate, Some(t) if t.elapsed().as_millis() < CELEBRATE_MS)
    }
}

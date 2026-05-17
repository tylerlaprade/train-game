use std::time::Instant;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::words;

pub const WHEELS_PER_CAR: u8 = 2;
pub const MAX_CARS_PER_TRAIN: usize = 8;
pub const TRAIN_TOP_ROW_FROM_TOP: usize = 10;
pub const TRAIN_VERTICAL_SPACING: usize = 1; // ground row between stacked trains
pub const SEG_HEIGHT: usize = 8;
pub const PARTIAL_CAR_WIDTH: i32 = 18;
pub const SMOKE_RISE: f32 = -8.0;
pub const SMOKE_MAX_AGE: f32 = 3.0;
pub const SMOKE_SPAWN_INTERVAL: f32 = 0.08;
pub const TRAIN_SPEED_CELLS_PER_SEC: f32 = 18.0;
/// How long after the last arrow-key event the train keeps moving. Must
/// exceed the OS key-repeat initial delay (~500ms on macOS) so holding the
/// key doesn't stall between the first press and the first auto-repeat.
pub const TRAIN_KEEP_MOVING_MS: u128 = 650;
pub const HORN_FLASH_MS: u128 = 600;
pub const CELEBRATE_MS: u128 = 900;
pub const WRAP_GAP: i32 = 6;

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

#[derive(Clone, Debug, Default)]
pub struct Train {
    pub cars: Vec<Car>,
    pub partial_wheels: u8,
}

impl Train {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_width(&self) -> i32 {
        let mut w = crate::renderer::ENGINE.width as i32;
        for car in &self.cars {
            w += crate::renderer::car_sprite(car.kind).width as i32;
        }
        if self.partial_wheels > 0 {
            w += PARTIAL_CAR_WIDTH;
        }
        w += crate::renderer::CABOOSE.width as i32;
        w
    }
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

    pub trains: Vec<Train>,

    pub smoke: Vec<Smoke>,
    smoke_spawn_accum: f32,

    pub target_word: String,
    pub typed: String,

    pub rng: SmallRng,
    pub last_tick: Instant,

    pub last_move: Option<Instant>,
    pub last_horn: Option<Instant>,
    pub last_celebrate: Option<Instant>,
    pub last_wrong: Option<Instant>,

    pub quit: bool,
}

impl Game {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut rng = SmallRng::from_entropy();
        let word = words::random_word(&mut rng).to_string();
        let now = Instant::now();
        Self {
            screen_cols: cols,
            screen_rows: rows,
            head_x: 0.0,
            velocity: 0.0,
            trains: vec![Train::new()],
            smoke: Vec::new(),
            smoke_spawn_accum: 0.0,
            target_word: word,
            typed: String::new(),
            rng,
            last_tick: now,
            last_move: None,
            last_horn: None,
            last_celebrate: None,
            last_wrong: None,
            quit: false,
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.screen_cols = cols;
        self.screen_rows = rows;
        let c = self.cycle();
        self.head_x = self.head_x.rem_euclid(c as f32);
    }

    pub fn cycle(&self) -> i32 {
        self.max_train_width() + self.screen_cols as i32 + WRAP_GAP
    }

    pub fn max_train_width(&self) -> i32 {
        self.trains.iter().map(Train::total_width).max().unwrap_or(0)
    }

    /// How many train rows the current terminal height can hold, given a
    /// fixed help row at top and a 5-row word UI at the bottom.
    pub fn max_trains(&self) -> usize {
        let rows = self.screen_rows as usize;
        let header = TRAIN_TOP_ROW_FROM_TOP;
        let footer = 5;
        let per_train = SEG_HEIGHT + TRAIN_VERTICAL_SPACING;
        if rows < header + per_train + footer {
            return 1;
        }
        let available = rows - header - footer;
        (available / per_train).max(1)
    }

    pub fn train_top_for(&self, train_idx: usize) -> usize {
        TRAIN_TOP_ROW_FROM_TOP + train_idx * (SEG_HEIGHT + TRAIN_VERTICAL_SPACING)
    }

    pub fn engine_smokestack_world_x(&self) -> f32 {
        // Smokestack column inside the engine sprite (the "____" at col 25-28).
        let smokestack_col_in_sprite = 27_i32;
        let engine_left = self.head_x - (crate::renderer::ENGINE.width as f32 - 1.0);
        engine_left + smokestack_col_in_sprite as f32
    }

    pub fn engine_smokestack_top_row(&self) -> i32 {
        self.train_top_for(0) as i32
    }

    pub fn tick(&mut self) {
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

        self.head_x += self.velocity * dt;
        let cycle = self.cycle() as f32;
        if cycle > 0.0 {
            self.head_x = self.head_x.rem_euclid(cycle);
        }

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
    }

    fn spawn_smoke_puff(&mut self) {
        // Smoke comes only from the top (first) train's smokestack.
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

    pub fn wrong_flash(&self) -> bool {
        matches!(self.last_wrong, Some(t) if t.elapsed().as_millis() < 300)
    }

    pub fn total_wheels(&self) -> usize {
        self.trains
            .iter()
            .map(|t| t.cars.len() * WHEELS_PER_CAR as usize + t.partial_wheels as usize)
            .sum()
    }

    pub fn handle_letter(&mut self, ch: char) -> WordOutcome {
        let ch = ch.to_ascii_lowercase();
        let expected = self.target_word.as_bytes()[self.typed.len()] as char;
        if ch == expected {
            self.typed.push(ch);
            if self.typed.len() == self.target_word.len() {
                let added = self.complete_word();
                if added {
                    WordOutcome::Correct
                } else {
                    WordOutcome::Maxed
                }
            } else {
                WordOutcome::Progress
            }
        } else {
            self.typed.clear();
            self.last_wrong = Some(Instant::now());
            WordOutcome::Wrong
        }
    }

    /// Adds one wheel. Returns true if a wheel was actually added (i.e.,
    /// there was capacity), false if all trains were full and there was no
    /// vertical room to start a new one.
    fn complete_word(&mut self) -> bool {
        self.typed.clear();
        self.target_word = words::random_word(&mut self.rng).to_string();
        self.last_celebrate = Some(Instant::now());

        let last_is_full = self
            .trains
            .last()
            .is_some_and(|t| t.cars.len() >= MAX_CARS_PER_TRAIN);
        if last_is_full {
            if self.trains.len() >= self.max_trains() {
                return false;
            }
            self.trains.push(Train::new());
        }

        let total_cars_before: usize = self.trains.iter().map(|t| t.cars.len()).sum();
        let current = self.trains.last_mut().expect("at least one train");
        current.partial_wheels += 1;
        if current.partial_wheels >= WHEELS_PER_CAR {
            current.partial_wheels = 0;
            current.cars.push(Car {
                kind: CarKind::rotate(total_cars_before),
                color: CarColor::rotate(total_cars_before),
            });
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordOutcome {
    Progress,
    /// Word completed and a wheel was added.
    Correct,
    /// Word completed but no room for more wheels (all trains full).
    Maxed,
    Wrong,
}

use std::time::Instant;

use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

pub const SEG_HEIGHT: usize = 14;
pub const SMOKE_RISE: f32 = -8.0;
pub const SMOKE_MAX_AGE: f32 = 3.0;
pub const SMOKE_SPAWN_INTERVAL: f32 = 0.08;
pub const TRAIN_SPEED_CELLS_PER_SEC: f32 = 32.0;
pub const TRAIN_KEEP_MOVING_MS: u128 = 650;
pub const CELEBRATE_MS: u128 = 900;
/// Empty track between the caboose of one loop iteration and the engine of
/// the next, used when the train is long enough that it drives the cycle.
/// Larger than a car width so the gap reads as "empty track" rather than
/// "cars touching."
pub const TRAIN_TAIL_GAP: i32 = 60;

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
    /// Monotonic, unwrapped distance the train has traveled (signed: forward
    /// motion adds, backward motion subtracts). Used to drive parallax so the
    /// terrain stops with the train and reverses on backwards motion.
    pub distance_traveled: f32,

    pub cars: Vec<Car>,
    /// Index in `cars` of a car that hasn't been announced yet. We delay the
    /// "another wheel" voice until the new car first appears on screen so the
    /// audio matches what the player sees.
    pub unannounced_car: Option<usize>,

    pub smoke: Vec<Smoke>,
    smoke_spawn_accum: f32,

    pub rng: SmallRng,
    pub started_at: Instant,
    pub last_tick: Instant,

    pub last_move: Option<Instant>,
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
            distance_traveled: 0.0,
            cars: Vec::new(),
            unannounced_car: None,
            smoke: Vec::new(),
            smoke_spawn_accum: 0.0,
            rng: SmallRng::from_entropy(),
            started_at: now,
            last_tick: now,
            last_move: None,
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

    /// Distance the head must travel before wrapping back to 0. Picked so
    /// three constraints all hold simultaneously:
    ///   * No two engines on screen at once (`cycle > screen + engine_width`).
    ///   * After a new car is inserted behind the engine on wrap, the whole
    ///     `+cycle` copy of the new train is still off screen. This prevents
    ///     shifted cars from popping in on the right before the old train has
    ///     fully cleared.
    ///   * For long trains, a `TRAIN_TAIL_GAP` stretch of empty track
    ///     separates the caboose from the next engine.
    pub fn cycle(&self) -> i32 {
        let screen = i32::from(self.screen_cols);
        let train = self.train_total_width();
        // 34 is the width of every car kind; using the next inserted kind
        // here would be more correct but every kind shares the same width.
        let car_w = crate::renderer::car_sprite(CarKind::Boxcar).width as i32;
        let floor = screen + train + car_w + 5;
        floor.max(train + TRAIN_TAIL_GAP)
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
        let smokestack_col_in_sprite = 49_i32;
        let engine_left = self.head_x - (crate::renderer::ENGINE.width as f32 - 1.0);
        engine_left + smokestack_col_in_sprite as f32
    }

    pub fn engine_smokestack_top_row(&self) -> i32 {
        self.train_top() as i32
    }

    /// Returns the number of new-car *announcements* this tick (0 or 1). A
    /// car can be added on a wrap event but the announcement is deferred
    /// until the new car is actually visible on screen (see `unannounced_car`).
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
        self.distance_traveled += self.velocity * dt;

        // Forward wrap-around event = one new car.
        if cycle > 0.0 && self.velocity > 0.5 && unwrapped >= cycle {
            self.add_car();
        }

        let announce = if let Some(idx) = self.unannounced_car {
            if self.car_is_on_screen(idx) {
                self.unannounced_car = None;
                self.last_celebrate = Some(now);
                1
            } else {
                0
            }
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

        announce
    }

    /// Is car at index `idx` at least partially within the screen at any of
    /// the renderer's three wrap-offsets?
    fn car_is_on_screen(&self, idx: usize) -> bool {
        if idx >= self.cars.len() {
            return false;
        }
        let engine_w = crate::renderer::ENGINE.width as i32;
        let mut right = self.head_x.floor() as i32 - engine_w;
        for (i, car) in self.cars.iter().enumerate() {
            let w = crate::renderer::car_sprite(car.kind).width as i32;
            let left = right - w + 1;
            if i == idx {
                let cycle = self.cycle();
                let screen = self.screen_cols as i32;
                for shift in [-cycle, 0, cycle] {
                    let l = left + shift;
                    let r = right + shift;
                    if r >= 0 && l < screen {
                        return true;
                    }
                }
                return false;
            }
            right = left - 1;
        }
        false
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

    fn add_car(&mut self) {
        let idx = self.cars.len();
        // Insert right behind the engine (cars[0]) rather than right in
        // front of the caboose (cars.push). Visually, the engine "pulls" a
        // new car out — the chain extends backward toward the caboose
        // rather than the caboose shoving back to make room.
        self.cars.insert(
            0,
            Car {
                kind: CarKind::rotate(idx),
                color: CarColor::rotate(idx),
            },
        );
        if let Some(prev) = self.unannounced_car {
            self.unannounced_car = Some(prev + 1);
        } else {
            self.unannounced_car = Some(0);
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

    pub fn celebrating(&self) -> bool {
        matches!(self.last_celebrate, Some(t) if t.elapsed().as_millis() < CELEBRATE_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::{Car, CarColor, CarKind, Game};
    use crate::game::TRAIN_SPEED_CELLS_PER_SEC;
    use std::time::{Duration, Instant};

    #[test]
    fn cycle_keeps_post_insert_train_copy_off_screen_at_wrap_moment() {
        let mut game = Game::new(200, 40);
        for idx in 0..3 {
            game.cars.push(Car {
                kind: CarKind::rotate(idx),
                color: CarColor::rotate(idx),
            });
        }

        let car_w = crate::renderer::car_sprite(super::CarKind::Boxcar).width as i32;
        let train_after_insert = game.train_total_width() + car_w;
        let train_left = game.cycle() - train_after_insert + 1;
        assert!(
            train_left >= game.screen_cols as i32,
            "post-insert train copy would be visible at offset +cycle (left={}, screen={})",
            train_left,
            game.screen_cols,
        );
    }

    #[test]
    fn backward_wrap_does_not_add_car() {
        let mut game = Game::new(200, 40);

        game.head_x = 0.1;
        game.velocity = -TRAIN_SPEED_CELLS_PER_SEC;
        game.last_tick = Instant::now() - Duration::from_secs(1);

        assert_eq!(game.tick(), 0);
        assert_eq!(game.cars.len(), 0);
    }

    #[test]
    fn forward_wrap_adds_car_off_screen_so_announcement_is_deferred() {
        let mut game = Game::new(200, 40);

        game.head_x = (game.cycle() - 1) as f32;
        game.velocity = TRAIN_SPEED_CELLS_PER_SEC;
        game.last_tick = Instant::now() - Duration::from_secs(1);

        let announced = game.tick();
        assert_eq!(game.cars.len(), 1);
        assert_eq!(
            announced, 0,
            "new car should be off-screen at the wrap moment so the voice waits"
        );
    }

    #[test]
    fn new_car_announcement_fires_when_inserted_car_first_reaches_screen() {
        let mut game = Game::new(200, 40);

        game.head_x = (game.cycle() - 1) as f32;
        game.velocity = TRAIN_SPEED_CELLS_PER_SEC;
        game.last_tick = Instant::now() - Duration::from_secs(1);
        assert_eq!(game.tick(), 0);
        assert_eq!(game.unannounced_car, Some(0));

        game.velocity = 0.0;
        game.head_x = crate::renderer::ENGINE.width as f32 - 1.0;
        assert_eq!(game.tick(), 0);
        assert_eq!(game.unannounced_car, Some(0));

        game.head_x = crate::renderer::ENGINE.width as f32;
        assert_eq!(game.tick(), 1);
        assert_eq!(game.unannounced_car, None);
    }
}

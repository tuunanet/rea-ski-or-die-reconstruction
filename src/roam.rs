//! Roaming special object (single instance at DS:0x3A65).
//!
//! Evidence: function 0x14573 (FUN_100c_44b3) waits 400 frames, then spawns a
//! random special (types 2/3/4/10/11/12, thresholds at DS:0x3A24) which walks
//! horizontally with a per-type (x, y, limit, dx) from DS:0x3A35. While it
//! moves it is logged into the frame's hit queue; a hit (function 0x148B0)
//! applies the type's effect and plays a death behavior before respawning.

use crate::arena::clamp_object;
use crate::data::*;
use crate::game::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoamState {
    Waiting,
    Moving,
    Dying,
}

#[derive(Debug, Clone)]
pub struct Roam {
    pub state: RoamState,
    pub kind: u8,
    pub x: i32,
    pub y: i32,
    pub limit: i32,
    pub dx: i32,
    pub behavior: usize,
    pub frame: usize,
    pub delay: i32,
    pub timer: i32,
    pub done: bool,
}

impl Default for Roam {
    fn default() -> Self {
        Roam {
            state: RoamState::Waiting,
            kind: 2,
            x: 0,
            y: 0,
            limit: 0,
            dx: 0,
            behavior: 26,
            frame: 0,
            delay: 0,
            timer: ROAM_RESPAWN_FRAMES,
            done: false,
        }
    }
}

impl Roam {
    pub fn reset(&mut self) {
        *self = Roam::default();
    }

    /// Pick the type exactly like the binary: first threshold <= random wins.
    pub fn pick_type(r: u8) -> usize {
        for (i, t) in ROAM_THRESHOLDS.iter().enumerate() {
            if *t <= r {
                return i;
            }
        }
        ROAM_THRESHOLDS.len() - 1
    }

    fn behavior_for(kind: u8) -> usize {
        match kind {
            2 => 26,
            3 => 25,
            4 => 27,
            10 => 28,
            11 => 29,
            _ => 39,
        }
    }

    pub fn death_behavior(kind: u8) -> usize {
        match kind {
            2 => 31,
            3 => 30,
            4 => 32,
            10 => 33,
            11 => 34,
            _ => 40,
        }
    }

    /// One frame of the roaming object.
    pub fn update(&mut self, rng: &mut Rng) {
        match self.state {
            RoamState::Waiting => {
                self.timer -= 1;
                if self.timer > 0 {
                    return;
                }
                let idx = Self::pick_type(rng.byte());
                let (sx, y, limit, dx) = ROAM_PARAMS[idx];
                self.kind = ROAM_TYPES[idx];
                self.x = sx;
                self.y = y;
                self.limit = limit;
                self.dx = dx;
                self.behavior = Self::behavior_for(self.kind);
                self.frame = 0;
                self.delay = 0;
                self.done = false;
                self.timer = ROAM_RESPAWN_FRAMES;
                self.state = RoamState::Moving;
            }
            RoamState::Moving => {
                self.x += self.dx;
                if self.x < 4 || self.x > self.limit {
                    self.state = RoamState::Waiting;
                    self.timer = ROAM_RESPAWN_FRAMES;
                    return;
                }
                self.tick_behavior();
                // A freshly spawned object may complete its animation instantly
                // only through the shared step; nothing else to do here.
            }
            RoamState::Dying => {
                self.tick_behavior();
                if self.done {
                    self.state = RoamState::Waiting;
                    self.timer = ROAM_RESPAWN_FRAMES;
                }
            }
        }
    }

    /// Advance the animation only; the logical x/y is owned by the roam
    /// movement, and the drawn/checked box adds the per-frame offsets.
    fn tick_behavior(&mut self) {
        if self.done {
            return;
        }
        let beh = &BEHAVIORS[self.behavior];
        self.delay -= 1;
        if self.delay < 0 {
            self.delay = beh.delay;
            self.frame += 1;
            if self.frame >= beh.frames.len() {
                self.frame = beh.frames.len().saturating_sub(1);
                self.done = true;
            }
        }
    }

    /// Box origin including the current animation offset (shared object step).
    pub fn box_origin(&self) -> (i32, i32) {
        let beh = &BEHAVIORS[self.behavior];
        let (_, _, dx, dy) = beh.frames.get(self.frame).copied().unwrap_or((0, 0, 0, 0));
        clamp_object(self.x + dx as i32, self.y + dy as i32)
    }

    /// Current sprite index in the SFFBM bank.
    pub fn sprite(&self) -> u8 {
        BEHAVIORS[self.behavior]
            .frames
            .get(self.frame)
            .map(|f| f.0)
            .unwrap_or(0)
    }

    /// Horizontal mirror flag of the current frame.
    pub fn mirror(&self) -> bool {
        BEHAVIORS[self.behavior]
            .frames
            .get(self.frame)
            .map(|f| f.1 != 0)
            .unwrap_or(false)
    }

    pub fn on_hit(&mut self) {
        self.behavior = Self::death_behavior(self.kind);
        self.frame = 0;
        self.delay = 0;
        self.done = false;
        self.state = RoamState::Dying;
    }

    /// Whether it is currently logged into the frame's hit queue.
    pub fn in_queue(&self) -> bool {
        self.state == RoamState::Moving
    }
}

//! Snowballs thrown by kids (pool B, 3 slots per participant).
//!
//! Evidence: snowballs are created at 0x1448F (`FUN_100c_43cf`) when a kid's
//! script reaches a throw behavior (indices 6..=8) at animation frame 5, and
//! are stepped by the shared object function at 0x164EC. Their trajectory is a
//! scripted ballistic arc: per-frame (dx, dy) offsets from the launch point
//! (behavior table entry 22/23/24, dy rises then falls). A snowball expires
//! when its script finishes (function 0x14501).

use crate::arena::clamp_object;
use crate::data::BEHAVIORS;

#[derive(Debug, Clone, Default)]
pub struct Snowball {
    pub active: bool,
    pub behavior: usize,
    pub base_x: i32,
    pub base_y: i32,
    pub x: i32,
    pub y: i32,
    pub frame: usize,
    pub delay: i32,
    pub done: bool,
}

impl Snowball {
    pub fn spawn(&mut self, behavior: usize, x: i32, y: i32) {
        self.active = true;
        self.behavior = behavior;
        self.base_x = x;
        self.base_y = y;
        self.x = x;
        self.y = y;
        self.frame = 0;
        self.delay = 0;
        self.done = false;
    }

    /// One simulation step of the scripted arc (object step at 0x164EC).
    /// Returns true when the snowball's script completed this step.
    pub fn step(&mut self) -> bool {
        if !self.active {
            return false;
        }
        if self.done {
            return true;
        }
        let beh = &BEHAVIORS[self.behavior];
        if let Some((dx, dy)) = beh.frames.get(self.frame) {
            let (nx, ny) = clamp_object(self.base_x + *dx as i32, self.base_y + *dy as i32);
            self.x = nx;
            self.y = ny;
        }
        self.delay -= 1;
        if self.delay < 0 {
            self.delay = beh.delay;
            self.frame += 1;
            if self.frame >= beh.frames.len() {
                self.frame = beh.frames.len().saturating_sub(1);
                self.done = true;
                return true;
            }
        }
        false
    }

    pub fn rect(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

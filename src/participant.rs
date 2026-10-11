//! Participants, kids (targets) and their scripted behaviour.
//!
//! The original module keeps four participant groups (indices 0..=3), each with
//! five kid slots (the fifth slot's script cursor is disabled with 0xFE in the
//! shipped data, so four active slots are used) and three snowball slots.
//! Relevant original functions:
//!   * 0x1427D - per-participant spawner/step  (FUN_100c_41bd)
//!   * 0x14338 - kid script processor          (FUN_100c_4278)
//!   * 0x1448F - snowball creation             (FUN_100c_43cf)
//!   * 0x164EC - scripted object step          (FUN_100c_642c)

use crate::arena::{clamp_object, kid_out_of_bounds};
use crate::data::*;
use crate::game::{QueueEntry, Target};
use crate::snowball::Snowball;

/// Kid sprites are 8..52 pixels wide and 12..51 tall (SPRITE_SIZES); the
/// collision box tracks the current animation frame, exactly like the
/// original box update in function 0x16E67.
pub const KID_SPRITE_FALLBACK: u8 = 0;

pub const KID_SLOTS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KidEvent {
    /// Kid walked out or finished its script: spawn credit is returned.
    Recycled,
    /// Kid died after being hit: decrements the level "kids" counter.
    Killed,
    /// Kid reached a throw frame: spawn a snowball at (x, y) of kind.
    Throw { x: i32, y: i32, kind: u8 },
}

#[derive(Debug, Clone)]
pub struct Kid {
    pub active: bool,
    pub kind: u8,
    pub behavior: usize,
    pub script: &'static [u8],
    pub script_idx: usize,
    pub base_x: i32,
    pub base_y: i32,
    pub x: i32,
    pub y: i32,
    pub frame: usize,
    pub delay: i32,
    pub done: bool,
    /// Current sprite index in the SFFBM bank.
    pub sprite: u8,
    /// Horizontal mirror flag of the current behaviour frame ([SI+0xE]).
    pub mirror: bool,
}

impl Default for Kid {
    fn default() -> Self {
        Kid {
            active: false,
            kind: 5,
            behavior: 0,
            script: &[],
            script_idx: 0,
            base_x: 0,
            base_y: 0,
            x: 0,
            y: 0,
            frame: 0,
            delay: 0,
            done: false,
            sprite: KID_SPRITE_FALLBACK,
            mirror: false,
        }
    }
}

impl Kid {
    pub fn spawn_from(&mut self, s: &KidSpawn, kind: u8) {
        self.active = true;
        self.kind = kind;
        self.behavior = s.behavior;
        self.script = s.script;
        self.script_idx = 0;
        self.base_x = s.x;
        self.base_y = s.y;
        self.x = s.x;
        self.y = s.y;
        self.frame = 0;
        self.delay = 0;
        self.done = false;
        self.sprite = BEHAVIORS[s.behavior].frames[0].0;
        self.mirror = BEHAVIORS[s.behavior].frames[0].1 != 0;
    }

    /// Collision box: the current animation frame's sprite size, with the
    /// baseline at (x, y) - mirrors the box update in function 0x16E67.
    pub fn rect(&self) -> crate::arena::Rect {
        let (w, h) = SPRITE_SIZES
            .get(self.sprite as usize)
            .map(|(w, h)| (*w as i32, *h as i32))
            .unwrap_or((16, 28));
        crate::arena::Rect {
            x: self.x,
            y: self.y,
            w,
            h,
        }
    }

    /// True while the kid is inside a throw animation (script state 6..=8).
    pub fn in_throw_pose(&self) -> bool {
        matches!(self.behavior, 6..=8)
    }

    /// One scripted step (shared object step at 0x164EC).
    fn step(&mut self) {
        if !self.active || self.done {
            return;
        }
        let beh = &BEHAVIORS[self.behavior];
        if let Some((sprite, mirror, dx, dy)) = beh.frames.get(self.frame) {
            self.sprite = *sprite;
            self.mirror = *mirror != 0;
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
            }
        }
    }

    /// Advance the kid script when its current behavior finished
    /// (kid script processor at 0x14338, [0x16] handling).
    /// Returns an event when the kid despawns, or `Held` otherwise.
    fn advance_script(&mut self) -> Option<KidEvent> {
        let old_state = self.script.get(self.script_idx).copied().unwrap_or(0xFF) as usize;
        self.script_idx += 1;
        let next = match self.script.get(self.script_idx) {
            Some(v) => *v,
            None => 0xFF,
        };
        if next == 0xFF {
            self.active = false;
            return Some(KidEvent::Recycled);
        }
        // Re-base the walk on the current x (small nudge per behavior index).
        if old_state < BEHAVIOR_XOFF.len() {
            self.base_x = self.x + BEHAVIOR_XOFF[old_state] as i32;
        }
        self.behavior = next as usize;
        self.frame = 0;
        self.delay = 0;
        self.done = false;
        self.sprite = BEHAVIORS[next as usize].frames[0].0;
        self.mirror = BEHAVIORS[next as usize].frames[0].1 != 0;
        None
    }

    /// Mark the kid as hit: it switches to its death behavior.
    /// (FUN_1000_48b0 tail: type -> death behavior, sprite state 0xFE.)
    ///
    /// `at_castle` is true for kids in the snow-fort zone (y >= CASTLE_Y).
    /// The shipped binary always gives type 8 the big close-up animation
    /// (behavior 21); here that animation is reserved for castle hits, and a
    /// type-8 kid hit out in the field uses the small type-7 death instead.
    pub fn kill(&mut self, at_castle: bool) {
        let mapped = DEATH_BEHAVIOR
            .iter()
            .find(|(k, _)| *k == self.kind)
            .map(|(_, b)| *b)
            .unwrap_or(18);
        let death = if mapped == CLOSEUP_HIT_BEHAVIOR && !at_castle {
            20 // the close-up belongs to the fort: use the small death instead
        } else {
            mapped
        };
        self.kind = 0xFE;
        self.behavior = death;
        self.base_x = self.x;
        self.frame = 0;
        self.delay = 0;
        self.done = false;
        self.sprite = BEHAVIORS[death].frames[0].0;
        self.mirror = BEHAVIORS[death].frames[0].1 != 0;
    }

    /// Full per-frame kid update. Order mirrors 0x14338:
    /// step -> bounds -> queue log -> throw check -> script advance.
    ///
    /// `log` is the [0x75e3] gate: only kids of the selected participant are
    /// recorded in the frame's hit queue (function 0x14245).
    pub fn update(
        &mut self,
        log: bool,
        target: Target,
        queue: &mut Vec<QueueEntry>,
    ) -> Vec<KidEvent> {
        let mut out = Vec::new();
        if !self.active {
            return out;
        }
        self.step();
        if kid_out_of_bounds(self.kind, self.x) {
            self.active = false;
            out.push(KidEvent::Recycled);
            return out;
        }
        if log && self.kind != 0xFE {
            queue.push(QueueEntry {
                target,
                kind: self.kind,
            });
        }
        // Throw: throw pose, animation frame 5, freshly reloaded delay
        // (conditions at 0x14389-0x143A6 -> snowball creation 0x1448F).
        if self.in_throw_pose() && self.frame == 5 && self.delay == BEHAVIORS[self.behavior].delay {
            out.push(KidEvent::Throw {
                x: self.x,
                y: self.y,
                kind: self.kind,
            });
        }
        if self.done {
            if self.kind == 0xFE {
                self.active = false;
                out.push(KidEvent::Killed);
            } else if let Some(ev) = self.advance_script() {
                out.push(ev);
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub kids: [Kid; KID_SLOTS],
    pub snowballs: [Snowball; 3],
    pub kid_count: u8,
    pub snowball_count: u8,
    pub spawn_timer: u8,
    pub active_kids: u8,
}

impl Default for Participant {
    fn default() -> Self {
        Participant {
            kids: Default::default(),
            snowballs: Default::default(),
            kid_count: 0,
            snowball_count: 0,
            spawn_timer: 1,
            active_kids: 0,
        }
    }
}

impl Participant {
    pub fn reset(&mut self) {
        for k in self.kids.iter_mut() {
            k.active = false;
        }
        for s in self.snowballs.iter_mut() {
            s.active = false;
        }
        self.kid_count = 0;
        self.snowball_count = 0;
        this_spawn_timer_reset(&mut self.spawn_timer);
        self.active_kids = 0;
    }
}

fn this_spawn_timer_reset(t: &mut u8) {
    *t = 1;
}

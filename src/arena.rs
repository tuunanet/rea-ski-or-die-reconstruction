//! Arena geometry recovered from the Snowball War module.
//!
//! The player is clamped to `x in [0, 299]`, `y in [53, 199]` (function at
//! 0x13FDF). Object coordinates are clamped to `x <= 319`, `y <= 199` by the
//! shared object step function at 0x164EC.

use crate::data::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    /// True when `(px, py)` lies inside `[x, x+w] x [y-h, y]`, matching the
    /// original hit test at 0x14879-0x148A1 (y is the object's baseline).
    pub fn contains_player_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px <= self.x + self.w && py <= self.y && py >= self.y - self.h
    }
}

/// Clamp the controlled player position to the playfield.
pub fn clamp_player(x: i32, y: i32) -> (i32, i32) {
    (x.clamp(0, ARENA_W - 1), y.clamp(ARENA_TOP, ARENA_BOTTOM))
}

/// Clamp an object position (object step function behavior).
pub fn clamp_object(x: i32, y: i32) -> (i32, i32) {
    (
        x.clamp(0, OBJECT_X_MAX),
        if y > ARENA_BOTTOM { ARENA_BOTTOM } else { y },
    )
}

/// Kid despawn thresholds by type (function at 0x14338).
/// Returns true when a kid of `kind` walked out of the arena.
pub fn kid_out_of_bounds(kind: u8, x: i32) -> bool {
    if x < 1 {
        return true;
    }
    let limit = match kind {
        5 => 305,
        6 => 299,
        _ => 291,
    };
    x >= limit
}

//! Input semantics recovered from the original keyboard handler.
//!
//! Evidence (REA analysis of SKIEGA, see docs/reverse-engineering-notes.md):
//!   * Direction function at 0x16D0C maps keypad scan codes 0x47/0x48/0x49,
//!     0x4B/0x4D/0x4F/0x50/0x51 to direction codes 7/8/1/6/2/5/4/3
//!     (numpad layout, 0 = none).
//!   * State function at 0x16C78 reports 1 for the key bound in [0x6C91]
//!     (scan 0x1C = Enter) and 2 for [0x6C92] (scan 0x36 = Right Shift).
//!   * Function 0x140EE: while state 2 (RShift) is held, an even direction
//!     2/4/6/8 selects participant (dir/2 - 1) = 0/1/2/3.

/// Per-frame input snapshot produced by the SDL event pump.
#[derive(Debug, Clone, Default)]
pub struct Input {
    /// Direction code 0..=8 (numpad order), 0 when no direction key is held.
    pub dir: u8,
    /// Enter held (throw / start).
    pub action: bool,
    /// Right Shift held (participant-select modifier).
    pub modifier: bool,
    /// Escape: leave the round (approximation of the original exit flag).
    pub exit: bool,
    /// 'P': pause (approximation of the original pause flag).
    pub pause: bool,
    /// 'M' in menu: toggle competition/practice (approximation).
    pub toggle_practice: bool,
}

impl Input {
    /// Participant index selected by the RShift+direction combination,
    /// or None when the combination does not select a participant.
    pub fn select_participant(&self) -> Option<usize> {
        if self.modifier && self.dir != 0 && self.dir.is_multiple_of(2) {
            Some((self.dir / 2 - 1) as usize)
        } else {
            None
        }
    }
}

/// Map a key code to a direction using the original keypad table.
pub fn direction_from_keycode(keypad: u8) -> u8 {
    match keypad {
        0x47 => 7,
        0x48 => 8,
        0x49 => 1,
        0x4B => 6,
        0x4D => 2,
        0x4F => 5,
        0x50 => 4,
        0x51 => 3,
        _ => 0,
    }
}

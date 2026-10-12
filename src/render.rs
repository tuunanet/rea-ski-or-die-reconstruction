//! Software renderer using the original game graphics.
//!
//! Layout, colours and sprite selection follow the recovered code:
//!   * arena background      - SNOWFORT.LZW (image loader 0x17F7B)
//!   * sprites               - SFFBM.LZW bank, indices 0..=164 (0x17FF4)
//!   * HUD text              - 8x8 font at DS:0x6728, drawn by 0x162CE/0x17AE2
//!   * HUD labels/values     - label colours DS:0x7618 (13), value 0x7619 (14)
//!   * cursor marker         - sprites 124 (red) / 143 (green) chosen in 0x13FDF
//!   * compass counts        - sprites 127..=132 at the DS:0x399C positions
//!     (function 0x14160 picks the sprite from the participant's kid count)
//!   * sprite positions use the bottom-left origin and even x, as the original
//!     blitter does (`and bx,0xfffe` in 0x177E7).

use crate::assets::Assets;
use crate::data::*;
use crate::game::{Game, Phase};
use crate::roam::RoamState;

pub const W: usize = 320;
pub const H: usize = 200;

pub struct Framebuffer {
    pub px: Vec<u8>, // RGBA
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    pub fn new() -> Self {
        Framebuffer {
            px: vec![0; W * H * 4],
        }
    }

    fn put(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = (y as usize * W + x as usize) * 4;
        self.px[i] = rgb[0];
        self.px[i + 1] = rgb[1];
        self.px[i + 2] = rgb[2];
        self.px[i + 3] = 255;
    }

    fn clear(&mut self, rgb: [u8; 3]) {
        for i in (0..self.px.len()).step_by(4) {
            self.px[i] = rgb[0];
            self.px[i + 1] = rgb[1];
            self.px[i + 2] = rgb[2];
            self.px[i + 3] = 255;
        }
    }

    pub fn blit_arena(&mut self, a: &Assets) {
        for y in 0..a.arena.h {
            for x in 0..a.arena.w {
                let c = a.color(a.arena.at(x, y));
                self.put(x as i32, y as i32, c);
            }
        }
    }

    /// Draw sprite `index` with its bottom-left corner at (x, y).
    /// `mirror` applies the horizontal flip flag the original blitter takes in
    /// AH (function 0x177E7, per-frame byte [SI+0xE]).
    pub fn blit_sprite(
        &mut self,
        a: &Assets,
        index: u8,
        x: i32,
        y: i32,
        mirror: bool,
        transparent_zero: bool,
    ) {
        let Some(r) = a.sprite(index) else { return };
        let x = x & !1;
        let top = y - r.h as i32;
        for sy in 0..r.h as usize {
            for sx in 0..r.w as usize {
                let v = a.sprite_atlas.at(r.x as usize + sx, r.y as usize + sy);
                if v == 0 && transparent_zero {
                    continue;
                }
                let dx = if mirror { r.w as usize - 1 - sx } else { sx };
                let c = a.color(v);
                self.put(x + dx as i32, top + sy as i32, c);
            }
        }
    }

    /// Blit an SFSHP overlay record: the per-team scenery band at (0,40) and
    /// the per-team compass at (272,3). Both records are complete images that
    /// replace that region of the arena art (the game blits them with function
    /// 0x17739 after the arena is displayed).
    pub fn blit_overlay(
        &mut self,
        a: &Assets,
        index: usize,
        x: i32,
        y: i32,
        transparent_zero: bool,
    ) {
        let Some(rect) = a.overlay_sprite(index) else {
            return;
        };
        for sy in 0..rect.h as usize {
            for sx in 0..rect.w as usize {
                let v = a
                    .overlay_atlas
                    .at(rect.x as usize + sx, rect.y as usize + sy);
                if v == 0 && transparent_zero {
                    continue;
                }
                let c = a.color(v);
                self.put(x + sx as i32, y + sy as i32, c);
            }
        }
    }

    /// Opaque sprite blit (no index-0 transparency).
    pub fn blit_sprite_opaque(&mut self, a: &Assets, index: u8, x: i32, y: i32) {
        self.blit_sprite(a, index, x, y, false, false);
    }

    pub fn text(&mut self, a: &Assets, x: i32, y: i32, s: &str, color_index: u8) {
        let rgb = a.color(color_index);
        let mut cx = x;
        for ch in s.chars() {
            if let Some(glyph) = a.font.glyph(ch) {
                for (row, bits) in glyph.iter().enumerate() {
                    for b in 0..8 {
                        if bits & (0x80 >> b) != 0 {
                            self.put(cx + b, y + row as i32, rgb);
                        }
                    }
                }
            }
            cx += 8;
        }
    }

    pub fn text_center(&mut self, a: &Assets, y: i32, s: &str, color_index: u8) {
        let x = (W as i32 - s.chars().count() as i32 * 8) / 2;
        self.text(a, x.max(0), y, s, color_index);
    }

    /// Text with a one pixel black outline around it.
    ///
    /// The original never needs this: its own messages sit on the banner drawn
    /// by [`draw_banner`], and its full screen text pages clear the display to
    /// a solid colour first. The menu and prompts of this reconstruction are
    /// drawn straight over the arena, where the yellow and light colours would
    /// otherwise disappear into the snow.
    pub fn text_outlined(&mut self, a: &Assets, x: i32, y: i32, s: &str, color_index: u8) {
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            self.text(a, x + dx, y + dy, s, 0);
        }
        self.text(a, x, y, s, color_index);
    }

    pub fn text_center_outlined(&mut self, a: &Assets, y: i32, s: &str, color_index: u8) {
        let x = (W as i32 - s.chars().count() as i32 * 8) / 2;
        self.text_outlined(a, x.max(0), y, s, color_index);
    }
}

/// HUD colours from the recovered data (DS:0x7614..0x761B).
const COL_LABEL: u8 = 13; // light magenta - "SCORE", "TIME", ...
const COL_VALUE: u8 = 14; // yellow - numbers
const COL_NAME: u8 = 12; // light red - player name
const COL_MSG: u8 = 10; // light green - bonus / powerup banners

fn number(v: i32) -> String {
    v.to_string()
}

/// The message banner the original draws behind every in-game message: a solid
/// light blue band with a black edge, then the message on top. Without it the
/// light text all but vanishes against the snow.
fn draw_banner(fb: &mut Framebuffer, a: &Assets, msg: &str) {
    let fill = a.color(BANNER_COLOR);
    let edge = a.color(BANNER_EDGE);
    for y in BANNER_Y0..=BANNER_Y1 {
        for x in BANNER_X0..=BANNER_X1 {
            let c = if y == BANNER_Y1 || x == BANNER_X0 || x == BANNER_X1 {
                edge
            } else {
                fill
            };
            fb.put(x, y, c);
        }
    }
    fb.text(
        a,
        BANNER_TEXT_POS.0,
        BANNER_TEXT_POS.1,
        msg,
        BANNER_TEXT_COLOR,
    );
}

pub fn draw(fb: &mut Framebuffer, g: &Game, a: &Assets) {
    fb.clear(a.color(0));
    match g.phase {
        Phase::Menu => {
            fb.blit_arena(a);
            fb.text_center_outlined(a, 40, "SNOWBALL WAR", COL_LABEL);
            fb.text_center_outlined(a, 56, "SKI OR DIE RECONSTRUCTION", COL_VALUE);
            fb.text_center_outlined(
                a,
                84,
                if g.practice {
                    "MODE: PRACTICE"
                } else {
                    "MODE: COMPETITION"
                },
                COL_MSG,
            );
            fb.text_center_outlined(a, 104, "PRESS ENTER TO START", COL_VALUE);
            fb.text_center_outlined(a, 124, "ARROWS MOVE  ENTER THROWS", COL_LABEL);
            fb.text_center_outlined(a, 136, "RSHIFT+ARROW SWITCHES DIRECTION", COL_LABEL);
            fb.text_center_outlined(a, 152, "M MODE   P PAUSE   ESC END", COL_LABEL);
        }
        Phase::RoundOver => {
            fb.blit_arena(a);
            let (bx, by) = OVERLAY_BACKGROUND_POS;
            fb.blit_overlay(a, OVERLAY_BACKGROUND[g.selected] as usize, bx, by, false);
            draw_entities(fb, g, a);
            // The round end keeps the HUD on screen (the original redraws the
            // final score with 0x146BB), so the score and level stay readable
            // in their usual places instead of as extra text over the snow.
            draw_hud(fb, g, a);
            if let Some(r) = &g.result {
                draw_banner(fb, a, r.message);
            }
            fb.text_center_outlined(a, 130, "PRESS ENTER", COL_VALUE);
        }
        Phase::Playing => {
            fb.blit_arena(a);
            // Per-team scenery band (function 0x14116 draws it at (0,40)).
            let (bx, by) = OVERLAY_BACKGROUND_POS;
            fb.blit_overlay(a, OVERLAY_BACKGROUND[g.selected] as usize, bx, by, false);
            draw_entities(fb, g, a);
            draw_hud(fb, g, a);
            if let Some(m) = g.message {
                draw_banner(fb, a, m);
            }
            if let Some(b) = g.banner {
                fb.text(a, 80, 18, b, COL_MSG);
            }
            if g.paused {
                fb.text_center_outlined(a, 100, "PAUSED", COL_VALUE);
            }
        }
    }
}

fn draw_hud(fb: &mut Framebuffer, g: &Game, a: &Assets) {
    // Compass for the selected team (function 0x14160, (272,3)). The overlay
    // record is a complete compass - un-mirrored record 4 is identical to the
    // compass baked into the arena art - so it replaces that 48x34 region
    // rather than blending with it.
    let (cx, cy) = OVERLAY_COMPASS_POS;
    fb.blit_overlay(a, OVERLAY_COMPASS[g.selected] as usize, cx, cy, false);

    // Recovered label positions (function 0x13DF0).
    fb.text(a, 6, 5, &g.player_name, COL_NAME);
    fb.text(a, 176, 5, "LEVEL", COL_LABEL);
    fb.text(a, 216, 5, &number(g.level), COL_VALUE);
    fb.text(a, 6, 23, "SCORE", COL_LABEL);
    fb.text(a, 6, 30, &number(g.score), COL_VALUE);
    fb.text(a, 80, 23, "SNOWBALLS   KIDS", COL_LABEL);
    fb.text(a, 104, 30, &number(g.snowballs), COL_VALUE);
    fb.text(a, 184, 30, &number(g.kids_left), COL_VALUE);
    fb.text(a, 224, 23, "TIME", COL_LABEL);
    fb.text(a, 232, 30, &number(g.time_left), COL_VALUE);

    // Compass: one count sprite per participant at the recovered positions
    // (function 0x14160: sprite = table[0x39AC + kid_count]).
    for (i, p) in g.participants.iter().enumerate() {
        let count = p.active_kid_count().min(5);
        let (x, y) = COMPASS_MARKERS[i];
        let sprite = 127 + count; // sprites 127..=132 are the count glyphs
        fb.blit_sprite(a, sprite, x, y, false, true);
    }
}

fn draw_entities(fb: &mut Framebuffer, g: &Game, a: &Assets) {
    // Painter's order by baseline y (the original sorts its draw list at
    // function 0x171A7).
    // Only the selected participant's objects are drawn and hittable: the
    // original gates both the draw (0x16E67) and the hit-queue logger (0x14245)
    // on [0x75E3] = (participant == [0x39CD]). The player switches teams with
    // RShift + a cardinal key; the compass shows every team's kid count.
    let mut items: Vec<(i32, u8, bool, i32, i32)> = Vec::new(); // (y, sprite, mirror, x, y)
    if let Some(p) = g.participants.get(g.selected) {
        for k in p.kids.iter() {
            if k.active {
                items.push((k.y, k.sprite, k.mirror, k.x, k.y));
            }
        }
        for s in p.snowballs.iter() {
            if s.active {
                items.push((s.y, s.sprite, s.mirror, s.x, s.y));
            }
        }
    }
    if matches!(g.roam.state, RoamState::Moving | RoamState::Dying) {
        let (x, y) = g.roam.box_origin();
        items.push((y, g.roam.sprite(), g.roam.mirror(), x, y));
    }
    if g.pickup.active {
        // Pickups reuse the snowball sprite (the original logs them as an
        // object with the same descriptor path).
        items.push((g.pickup.y, 141, false, g.pickup.x, g.pickup.y));
    }
    // Cursor marker on top of same-row entities.
    items.push((g.player.y, g.cursor_sprite(), false, g.player.x, g.player.y));
    items.sort_by_key(|it| it.0);

    for (_y, sprite, mirror, x, y) in items {
        fb.blit_sprite(a, sprite, x, y, mirror, true);
    }
}

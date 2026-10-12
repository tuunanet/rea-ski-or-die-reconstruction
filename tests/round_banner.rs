//! The message banner (docs/behavior-spec.md 10).
//!
//! The original never draws its light message text straight onto the snow: the
//! round code fills the rectangle (32,100)-(290,115) with colour 9, edges it in
//! black and then draws the message at (52,105) in colour 14. These tests keep
//! the reconstruction from dropping that banner again (which made the yellow
//! and green text unreadable against the playfield).
//!
//! Skipped when the generated asset pack is missing.

use snowball_war::assets::Assets;
use snowball_war::data::{
    BANNER_COLOR, BANNER_EDGE, BANNER_TEXT_COLOR, BANNER_TEXT_POS, BANNER_X0, BANNER_X1, BANNER_Y0,
    BANNER_Y1,
};
use snowball_war::game::{Game, Phase, RoundResult};
use snowball_war::render::{self, Framebuffer, W};

/// Palette RGB values used by the banner (colour 9 / 0 / 14).
const FILL: [u8; 3] = [84, 84, 252];
const EDGE: [u8; 3] = [0, 0, 0];
const TEXT: [u8; 3] = [252, 252, 84];

fn px(fb: &Framebuffer, x: i32, y: i32) -> [u8; 3] {
    let i = (y as usize * W + x as usize) * 4;
    [fb.px[i], fb.px[i + 1], fb.px[i + 2]]
}

fn count(fb: &Framebuffer, want: [u8; 3], x0: i32, x1: i32, y0: i32, y1: i32) -> usize {
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            if px(fb, x, y) == want {
                n += 1;
            }
        }
    }
    n
}

fn shot(g: &Game, a: &Assets) -> Framebuffer {
    let mut fb = Framebuffer::new();
    render::draw(&mut fb, g, a);
    fb
}

#[test]
fn banner_geometry_matches_the_recovered_values() {
    assert_eq!((BANNER_X0, BANNER_X1), (32, 290));
    assert_eq!((BANNER_Y0, BANNER_Y1), (100, 115));
    assert_eq!(BANNER_COLOR, 9);
    assert_eq!(BANNER_EDGE, 0);
    assert_eq!(BANNER_TEXT_POS, (52, 105));
    assert_eq!(BANNER_TEXT_COLOR, 14);
}

/// The banner plus the black edge it gets from the second pass of 0x16387.
fn assert_banner(fb: &Framebuffer, msg_len: i32) {
    let mid = BANNER_Y0 + 5;
    assert_eq!(px(fb, BANNER_X0, mid), EDGE, "left edge");
    assert_eq!(px(fb, BANNER_X1, mid), EDGE, "right edge");
    assert_eq!(px(fb, BANNER_X0 + 10, BANNER_Y1), EDGE, "bottom edge");
    assert_eq!(px(fb, BANNER_X0 + 2, mid), FILL, "field left of the text");
    // The field is the recovered banner colour right of the text.
    let after_text = (BANNER_TEXT_POS.0 + msg_len * 8 + 4).min(BANNER_X1 - 4);
    assert_eq!(px(fb, after_text, mid), FILL, "banner field");
    // The message itself sits inside the banner in the light colour.
    let n = count(
        fb,
        TEXT,
        BANNER_TEXT_POS.0,
        BANNER_TEXT_POS.0 + msg_len * 8,
        BANNER_TEXT_POS.1,
        BANNER_TEXT_POS.1 + 8,
    );
    assert!(n > (msg_len * 2) as usize, "message pixels: {n}");
    // ...and the banner is visibly not the snow below it.
    assert_ne!(px(fb, after_text, BANNER_Y1 + 6), FILL);
}

#[test]
fn in_game_message_sits_on_the_banner() {
    let Ok(assets) = Assets::load_default() else {
        eprintln!("skipping: assets/snowball.pack not generated");
        return;
    };
    let mut g = Game::new(0x5EED);
    g.start(false);
    g.message = Some("BONUS FOR CLEARING LEVEL");
    assert_banner(&shot(&g, &assets), 24);
}

#[test]
fn round_over_uses_the_banner_and_the_hud() {
    let Ok(assets) = Assets::load_default() else {
        eprintln!("skipping: assets/snowball.pack not generated");
        return;
    };
    let mut g = Game::new(0x5EED);
    g.start(false);
    g.score = 300;
    g.level = 1;
    g.phase = Phase::RoundOver;
    g.message = None;
    g.result = Some(RoundResult {
        message: "YOU'RE OUTTA TIME, SLICK!",
        score: 300,
        level: 1,
    });
    let fb = shot(&g, &assets);
    assert_banner(&fb, 25);

    // The round end keeps the HUD, so the final score and level are readable on
    // the black band instead of as extra light text over the playfield.
    let hud_text = count(&fb, TEXT, 0, 60, 28, 40) + count(&fb, TEXT, 200, 260, 5, 40);
    assert!(hud_text > 20, "HUD score/level pixels: {hud_text}");

    // The prompt is outlined: it needs both the light colour and black pixels.
    let prompt_yellow = count(&fb, TEXT, 100, 220, 130, 138);
    let prompt_black = count(&fb, EDGE, 100, 220, 130, 138);
    assert!(prompt_yellow > 10, "prompt pixels: {prompt_yellow}");
    assert!(prompt_black > 10, "prompt outline pixels: {prompt_black}");
}

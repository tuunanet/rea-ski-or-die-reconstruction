//! Procedural SDL2 rendering for the Snowball War reconstruction.
//!
//! Only original placeholder artwork is used; sprite files from the game are
//! not read. HUD label positions come from the recovered draw calls
//! (functions 0x13DF0, 0x1470B, 0x14763, 0x14A49).

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::data::*;
use crate::font::glyph;
use crate::game::{Game, Phase};

const W: u32 = 320;
const H: u32 = 200;

const SKY: Color = Color::RGB(90, 110, 160);
const HUD_BG: Color = Color::RGB(30, 30, 60);
const SNOW: Color = Color::RGB(235, 240, 250);
const SNOW_SHADE: Color = Color::RGB(205, 215, 235);
const TEXT: Color = Color::RGB(255, 255, 160);
const TEXT_DIM: Color = Color::RGB(200, 200, 220);
const PLAYER_C: Color = Color::RGB(80, 160, 255);
const KID_COLORS: [Color; 4] = [
    Color::RGB(250, 120, 100),
    Color::RGB(250, 190, 90),
    Color::RGB(150, 240, 130),
    Color::RGB(240, 130, 240),
];
const ROAM_C: Color = Color::RGB(255, 255, 255);
const PICKUP_C: Color = Color::RGB(255, 230, 90);
const BALL_C: Color = Color::RGB(255, 255, 255);

fn text(canvas: &mut Canvas<Window>, x: i32, y: i32, s: &str, c: Color) -> Result<(), String> {
    canvas.set_draw_color(c);
    let mut cx = x;
    for ch in s.chars() {
        if let Some(rows) = glyph(ch) {
            for (r, bits) in rows.iter().enumerate() {
                for b in 0..5u8 {
                    if bits & (1 << (4 - b)) != 0 {
                        let px = cx + b as i32;
                        let py = y + r as i32;
                        if (0..W as i32).contains(&px) && (0..H as i32).contains(&py) {
                            canvas.draw_point((px, py))?;
                        }
                    }
                }
            }
        }
        cx += 8; // 8-pixel cell advance (matches the original label grid)
    }
    Ok(())
}

fn fill(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    c: Color,
) -> Result<(), String> {
    canvas.set_draw_color(c);
    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let rect = sdl2::rect::Rect::new(x, y, w as u32, h as u32);
    canvas.fill_rect(rect)?;
    Ok(())
}

/// Simple standing figure drawn from its baseline.
fn figure(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    h: i32,
    c: Color,
    frame: usize,
) -> Result<(), String> {
    let body_top = y - h + 6;
    // head
    fill(canvas, x + 4, body_top - 6, 7, 6, c)?;
    // torso
    fill(canvas, x + 3, body_top, 9, h - 14, c)?;
    // legs
    fill(canvas, x + 3, y - 8, 3, 8, c)?;
    fill(canvas, x + 9, y - 8, 3, 8, c)?;
    // swinging arm (animation)
    let swing = match frame % 4 {
        0 => 0,
        1 => 2,
        2 => 4,
        _ => 2,
    };
    fill(canvas, x - 2 + swing, body_top + 2, 5, 3, c)?;
    fill(canvas, x + 12 - swing, body_top + 2, 5, 3, c)?;
    Ok(())
}

pub fn draw(canvas: &mut Canvas<Window>, g: &Game) -> Result<(), String> {
    canvas.set_draw_color(SKY);
    canvas.clear();

    // Arena backdrop (procedural).
    fill(canvas, 0, 0, W as i32, ARENA_TOP, HUD_BG)?;
    fill(canvas, 0, ARENA_TOP, W as i32, H as i32 - ARENA_TOP, SNOW)?;
    // Snow drifts / snow fort silhouette.
    canvas.set_draw_color(SNOW_SHADE);
    for i in 0..10 {
        let x = 10 + i * 31;
        canvas.draw_line((x, 190), (x + 24, 176 - (i % 3) * 6))?;
    }
    fill(canvas, 120, 60, 60, 6, SNOW_SHADE)?;
    fill(canvas, 130, 54, 40, 6, SNOW_SHADE)?;

    match g.phase {
        Phase::Menu => {
            text(canvas, 92, 60, "SNOWBALL WAR", TEXT)?;
            text(
                canvas,
                40,
                78,
                "SKI OR DIE MINIGAME RECONSTRUCTION",
                TEXT_DIM,
            )?;
            text(
                canvas,
                56,
                104,
                if g.practice {
                    "MODE: PRACTICE"
                } else {
                    "MODE: COMPETITION"
                },
                TEXT,
            )?;
            text(canvas, 52, 122, "PRESS ENTER TO START", TEXT)?;
            text(canvas, 44, 138, "M: MODE  ARROWS: MOVE", TEXT_DIM)?;
            text(canvas, 44, 150, "ENTER: THROW  RSHIFT: SWITCH", TEXT_DIM)?;
        }
        Phase::RoundOver => {
            if let Some(r) = &g.result {
                text(canvas, 52, 70, r.message, TEXT)?;
                let s = format!("SCORE {}", r.score);
                text(canvas, 96, 96, &s, TEXT)?;
                let l = format!("LEVEL {}", r.level);
                text(canvas, 104, 112, &l, TEXT)?;
            }
            text(canvas, 76, 140, "PRESS ENTER", TEXT_DIM)?;
        }
        Phase::Playing => {
            draw_hud(canvas, g)?;
            draw_entities(canvas, g)?;
            if let Some(m) = g.message {
                text(canvas, 52, 105, m, TEXT)?;
            }
            if let Some(b) = g.banner {
                text(canvas, 80, 18, b, PICKUP_C)?;
            }
            if g.paused {
                text(canvas, 128, 100, "PAUSED", TEXT)?;
            }
            if g.flash > 0 {
                canvas.set_draw_color(Color::RGB(255, 255, 255));
                for _ in 0..12 {
                    // spark effect placeholder around the playfield
                    canvas.draw_point((40 + g.flash * 7, 90 + g.flash * 3))?;
                }
            }
        }
    }
    canvas.present();
    Ok(())
}

fn draw_hud(canvas: &mut Canvas<Window>, g: &Game) -> Result<(), String> {
    // Recovered label positions (function 0x13DF0).
    text(
        canvas,
        6,
        5,
        if g.practice { "PRACTICE" } else { "SNOWFIGHT!" },
        TEXT_DIM,
    )?;
    text(canvas, 176, 5, "LEVEL", TEXT)?;
    text(canvas, 216, 5, &g.level.to_string(), TEXT)?;
    text(canvas, 6, 23, "SCORE", TEXT)?;
    text(canvas, 6, 30, &g.score.to_string(), TEXT)?;
    text(canvas, 80, 23, "SNOWBALLS   KIDS", TEXT)?;
    text(canvas, 104, 30, &g.snowballs.to_string(), TEXT)?;
    text(canvas, 184, 30, &g.kids_left.to_string(), TEXT)?;
    text(canvas, 224, 23, "TIME", TEXT)?;
    text(canvas, 232, 30, &g.time_left.to_string(), TEXT)?;
    Ok(())
}

fn draw_entities(canvas: &mut Canvas<Window>, g: &Game) -> Result<(), String> {
    // Painter's order: sort by baseline y (original sorts the draw list at
    // function 0x171A7).
    let mut items: Vec<(i32, u8, usize, i32, i32, i32)> = Vec::new();
    // kind: 0 kid, 1 player, 2 roam, 3 pickup, 4 snowball
    for (pi, p) in g.participants.iter().enumerate() {
        for (si, k) in p.kids.iter().enumerate() {
            if k.active {
                items.push((k.y, 0, si, k.x, k.kind as i32, pi as i32));
            }
        }
        for s in p.snowballs.iter() {
            if s.active {
                items.push((s.y, 4, 0, s.x, 0, 0));
            }
        }
    }
    items.push((g.player.y, 1, 0, g.player.x, 0, 0));
    if g.roam.in_queue() || matches!(g.roam.state, crate::roam::RoamState::Dying) {
        let (x, y) = g.roam.box_origin();
        items.push((y, 2, g.roam.kind as usize, x, 0, 0));
    }
    if g.pickup.active {
        items.push((g.pickup.y, 3, 0, g.pickup.x, g.pickup.kind as i32, 0));
    }
    items.sort_by_key(|it| it.0);

    for (y, kind, extra, x, param, _) in items {
        match kind {
            0 => {
                let color = if param == 0xFE {
                    Color::RGB(120, 120, 140)
                } else {
                    KID_COLORS[((param as usize).wrapping_sub(5)) % KID_COLORS.len()]
                };
                figure(canvas, x, y, KID_H_ABS, color, (extra + y as usize) % 4)?;
            }
            1 => figure(canvas, x, y, 28, PLAYER_C, 0)?,
            2 => figure(canvas, x, y, 24, ROAM_C, extra % 4)?,
            3 => {
                fill(canvas, x, y - 8, 8, 8, PICKUP_C)?;
                fill(canvas, x + 2, y - 6, 4, 4, Color::RGB(255, 255, 255))?;
            }
            _ => {
                canvas.set_draw_color(BALL_C);
                canvas.fill_rect(sdl2::rect::Rect::new(x, y - 3, 5, 5))?;
            }
        }
    }
    Ok(())
}

/// Kid sprite height used for the procedural figure (box height).
const KID_H_ABS: i32 = 28;

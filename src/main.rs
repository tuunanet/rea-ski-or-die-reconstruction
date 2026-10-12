//! Snowball War - SDL2 front end.
//!
//! Owns the window, the key mapping (recovered from the original keyboard
//! handler at 0x16D0C/0x16C78) and a fixed-timestep loop; gameplay lives in
//! the SDL-free `snowball_war` library and rendering rasterises the original
//! graphics into a 320x200 framebuffer that is uploaded once per frame.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;

use snowball_war::assets::Assets;
use snowball_war::game::{Game, SIM_HZ};
use snowball_war::input::{direction_from_keycode, Input};
use snowball_war::render::{self, Framebuffer, H, W};

/// Pixel format used to upload `Framebuffer::px` to the GPU/canvas.
///
/// The framebuffer holds the bytes of every pixel as R, G, B, A, and `RGBA32`
/// is the SDL format whose *in-memory* byte order is exactly that (it aliases
/// `ABGR8888` on little-endian). `ARGB8888` is `B, G, R, A` in memory, which
/// silently swaps the red and blue channels on screen - the EGA palette then
/// looks nothing like the extracted assets.
pub const FRAMEBUFFER_FORMAT: PixelFormatEnum = PixelFormatEnum::RGBA32;

/// Original keyboard scan codes for the eight direction keys.
fn scan_code(sc: Scancode) -> Option<u8> {
    Some(match sc {
        Scancode::Home | Scancode::Kp7 => 0x47,
        Scancode::Up | Scancode::Kp8 => 0x48,
        Scancode::PageUp | Scancode::Kp9 => 0x49,
        Scancode::Left | Scancode::Kp4 => 0x4B,
        Scancode::Right | Scancode::Kp6 => 0x4D,
        Scancode::End | Scancode::Kp1 => 0x4F,
        Scancode::Down | Scancode::Kp2 => 0x50,
        Scancode::PageDown | Scancode::Kp3 => 0x51,
        _ => return None,
    })
}

fn build_input(pressed: &HashSet<Scancode>, prev: &HashSet<Scancode>) -> Input {
    // Direction resolution follows the original scan-code priority order.
    let order = [0x47u8, 0x48, 0x49, 0x4B, 0x4D, 0x4F, 0x50, 0x51];
    let mut dir = 0u8;
    for code in order {
        if pressed.iter().any(|sc| scan_code(*sc) == Some(code)) {
            dir = direction_from_keycode(code);
            break;
        }
    }
    let edge = |sc: Scancode| pressed.contains(&sc) && !prev.contains(&sc);
    Input {
        dir,
        action: pressed.contains(&Scancode::Return) || pressed.contains(&Scancode::Space),
        modifier: pressed.contains(&Scancode::RShift),
        exit: edge(Scancode::Escape),
        pause: edge(Scancode::P),
        toggle_practice: edge(Scancode::M),
    }
}

fn main() -> Result<(), String> {
    let assets = Assets::load_default().map_err(|e| {
        format!(
            "cannot load assets ({e});\n\
             build them from your own Ski or Die copy first:\n  \
             python3 tools/extract/build_assets.py Ski_Or_Die assets"
        )
    })?;

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("Snowball War - Ski or Die reconstruction", 320 * 3, 200 * 3)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(W as u32, H as u32)
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(FRAMEBUFFER_FORMAT, W as u32, H as u32)
        .map_err(|e| e.to_string())?;
    let mut events = sdl.event_pump().map_err(|e| e.to_string())?;

    let mut fb = Framebuffer::new();
    let mut game = Game::new(0x5EED);
    let step = Duration::from_secs_f64(1.0 / SIM_HZ as f64);
    let mut acc = Duration::ZERO;
    let mut last = Instant::now();
    let mut pressed: HashSet<Scancode> = HashSet::new();

    'running: loop {
        let before = pressed.clone();
        for e in events.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    scancode: Some(sc),
                    repeat: false,
                    ..
                } => {
                    pressed.insert(sc);
                }
                Event::KeyUp {
                    scancode: Some(sc), ..
                } => {
                    pressed.remove(&sc);
                }
                _ => {}
            }
        }
        let input = build_input(&pressed, &before);

        let now = Instant::now();
        acc += now - last;
        last = now;
        while acc >= step {
            game.step(&input);
            acc -= step;
        }

        render::draw(&mut fb, &game, &assets);
        texture
            .update(None, &fb.px, W * 4)
            .map_err(|e| e.to_string())?;
        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|e| e.to_string())?;
        canvas.present();
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the window shows must be exactly what the renderer drew. This
    /// guards `FRAMEBUFFER_FORMAT`: the old `ARGB8888` swapped the red and blue
    /// channels, so the EGA palette on screen did not match the extracted
    /// assets (sky blue came out red, yellow text cyan).
    ///
    /// Skipped when the asset pack is absent or no SDL video driver works.
    #[test]
    fn presentation_preserves_rendered_colours() {
        let Ok(assets) = Assets::load_default() else {
            eprintln!("skipping: assets/snowball.pack not generated");
            return;
        };
        // Run headless when the caller has not picked a driver; the dummy
        // driver still backs a real software canvas we can read back.
        if std::env::var_os("SDL_VIDEODRIVER").is_none() {
            std::env::set_var("SDL_VIDEODRIVER", "dummy");
        }
        let Ok(sdl) = sdl2::init() else {
            eprintln!("skipping: SDL2 unavailable");
            return;
        };
        let Ok(video) = sdl.video() else {
            eprintln!("skipping: no SDL video driver");
            return;
        };

        let mut game = Game::new(0x5EED);
        game.start(false);
        let mut fb = Framebuffer::new();
        render::draw(&mut fb, &game, &assets);

        let window = video
            .window("format check", W as u32, H as u32)
            .hidden()
            .build()
            .expect("window");
        let mut canvas = window.into_canvas().software().build().expect("canvas");
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_streaming(FRAMEBUFFER_FORMAT, W as u32, H as u32)
            .expect("texture");
        texture.update(None, &fb.px, W * 4).expect("upload");
        canvas.clear();
        canvas.copy(&texture, None, None).expect("blit");
        canvas.present();

        let shown = canvas
            .read_pixels(None, PixelFormatEnum::RGBA32)
            .expect("read back");
        assert_eq!(shown.len(), fb.px.len(), "unexpected read-back pitch");
        let wrong = fb
            .px
            .iter()
            .zip(shown.iter())
            .filter(|(drawn, shown)| drawn != shown)
            .count();
        assert_eq!(
            wrong,
            0,
            "{wrong} of {} channel bytes differ between the framebuffer and the \
             presented image (check FRAMEBUFFER_FORMAT)",
            fb.px.len()
        );
    }
}

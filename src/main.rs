//! Snowball War - SDL2 front end.
//!
//! Owns the window, the key mapping (recovered from the original keyboard
//! handler at 0x16D0C/0x16C78) and a fixed-timestep loop; all gameplay lives
//! in the SDL-free `snowball_war` library.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

use snowball_war::game::{Game, SIM_HZ};
use snowball_war::input::{direction_from_keycode, Input};
use snowball_war::render;

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
        let hit = pressed.iter().any(|sc| scan_code(*sc) == Some(code));
        if hit {
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
        .set_logical_size(320, 200)
        .map_err(|e| e.to_string())?;
    let mut events = sdl.event_pump().map_err(|e| e.to_string())?;

    let mut game = Game::new(0x5EED);
    let step = Duration::from_secs_f64(1.0 / SIM_HZ as f64);
    let mut acc = Duration::ZERO;
    let mut last = Instant::now();
    // Key state is tracked from key events (the polled keyboard state API
    // panics on some drivers).
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

        render::draw(&mut canvas, &game)?;
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

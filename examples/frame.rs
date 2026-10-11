//! Headless frame dump: renders N simulation steps to a PPM for visual
//! comparison against the original game (no window is created).
//!
//! usage: cargo run --example frame -- <steps> <out.ppm> [practice]

use snowball_war::assets::Assets;
use snowball_war::game::Game;
use snowball_war::input::Input;
use snowball_war::render::{self, Framebuffer, H, W};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let steps: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(120);
    let out = args.get(2).cloned().unwrap_or_else(|| "frame.ppm".into());
    let practice = args.get(3).map(|s| s == "practice").unwrap_or(false);

    let assets = Assets::load_default().expect("assets");
    let mut game = Game::new(0x5EED);
    game.start(practice);
    for i in 0..steps {
        // wiggle the cursor like a player would
        let dir = match (i / 25) % 4 {
            0 => 2,
            1 => 4,
            2 => 6,
            _ => 8,
        };
        let input = Input {
            dir,
            action: i % 30 == 0,
            ..Default::default()
        };
        game.step(&input);
    }
    let mut fb = Framebuffer::new();
    render::draw(&mut fb, &game, &assets);
    let mut bytes = Vec::with_capacity(W * H * 3);
    for i in 0..W * H {
        bytes.extend_from_slice(&fb.px[i * 4..i * 4 + 3]);
    }
    let mut ppm = format!("P6\n{} {}\n255\n", W, H).into_bytes();
    ppm.extend_from_slice(&bytes);
    std::fs::write(&out, ppm).expect("write ppm");
    println!(
        "wrote {out} after {steps} steps (score={} kids={} time={})",
        game.score, game.kids_left, game.time_left
    );
}

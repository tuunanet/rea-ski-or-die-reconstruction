//! Automated gameplay tests for the Snowball War reconstruction.
//!
//! Each test documents the recovered rule it verifies and the original
//! address the rule comes from (see docs/behavior-spec.md).

use snowball_war::arena::{kid_out_of_bounds, Rect as ARect};
use snowball_war::data::*;
use snowball_war::game::{Game, Phase, Target};
use snowball_war::input::Input;
use snowball_war::participant::Kid;

fn playing_game(seed: u32) -> Game {
    let mut g = Game::new(seed);
    g.start(false);
    g
}

/// Stop automatic kid spawns so a test can control arena contents.
fn freeze_spawns(g: &mut Game) {
    for p in g.participants.iter_mut() {
        p.spawn_timer = 0xFF;
    }
}

fn idle() -> Input {
    Input::default()
}

/// Freeze the kid on its current animation frame and aim the cursor so its
/// hot point (x+9, y-8) lands in the middle of the kid's real hit box.
fn aim_at_kid(g: &mut Game, participant: usize, slot: usize) {
    use snowball_war::data::{ATTACK_DX, ATTACK_DY, BEHAVIORS, SPRITE_SIZES};
    let kid = &mut g.participants[participant].kids[slot];
    kid.frame = 0;
    kid.delay = 1000; // keep the frame/sprite stable through the next step
    let (sprite, _mirror, dx, dy) = BEHAVIORS[kid.behavior].frames[0];
    let (w, h) = SPRITE_SIZES[sprite as usize];
    let x = kid.base_x + dx as i32;
    let y = kid.base_y + dy as i32;
    let (hx, hy) = (x + w as i32 / 2, y - h as i32 / 2);
    let r: ARect = ARect {
        x,
        y,
        w: w as i32,
        h: h as i32,
    };
    assert!(r.contains_player_point(hx, hy));
    g.player.x = hx - ATTACK_DX;
    g.player.y = hy + ATTACK_DY;
    g.player.debounce = -1;
}

// 1. Participant movement: 8-way movement with acceleration (0x13FDF).
#[test]
fn player_moves_with_acceleration() {
    let mut g = playing_game(1);
    freeze_spawns(&mut g);
    assert_eq!((g.player.x, g.player.y), (PLAYER_START_X, PLAYER_START_Y));
    let input = Input { dir: 2, ..idle() }; // right
    g.step(&input);
    assert_eq!(g.player.x, PLAYER_START_X + 2, "speed level 0 moves 2 px");
    // After ~7 moving frames the speed word's high byte reaches 1 (x4).
    for _ in 0..6 {
        g.step(&input);
    }
    let before = g.player.x;
    g.step(&input);
    let dx = g.player.x - before;
    assert!(
        dx >= 4,
        "accelerated movement should reach at least 4 px, got {dx}"
    );
    // Keep moving to reach the capped speed level 2 (x6).
    for _ in 0..20 {
        g.step(&input);
    }
    assert_eq!(g.player.speed_level(), 2, "speed level clamps at 2");
    let before = g.player.x;
    g.step(&input);
    assert_eq!(g.player.x - before, 6, "speed level 2 moves 6 px");
    // Releasing the direction resets the acceleration word (0x13FE9).
    g.step(&idle());
    assert_eq!(g.player.accel, 0);
}

// 2. Arena boundary handling (clamps recovered from 0x1404E-0x14073).
#[test]
fn player_is_clamped_to_arena() {
    let mut g = playing_game(2);
    freeze_spawns(&mut g);
    g.player.x = ARENA_W - 1;
    g.player.y = 60;
    g.step(&Input { dir: 2, ..idle() });
    assert_eq!(g.player.x, ARENA_W - 1, "cannot leave the right edge");
    g.player.y = ARENA_BOTTOM - 1;
    g.step(&Input { dir: 4, ..idle() });
    assert_eq!(g.player.y, ARENA_BOTTOM, "cannot leave the bottom edge");
    g.player.y = ARENA_TOP + 1;
    g.step(&Input { dir: 8, ..idle() });
    assert_eq!(g.player.y, ARENA_TOP, "cannot enter the HUD band");
    g.player.x = 0;
    g.step(&Input { dir: 6, ..idle() });
    assert_eq!(g.player.x, 0, "cannot leave the left edge");
}

// Object despawn thresholds (function 0x14338).
#[test]
fn kid_despawn_thresholds_match_types() {
    assert!(!kid_out_of_bounds(5, 1));
    assert!(!kid_out_of_bounds(5, 304));
    assert!(kid_out_of_bounds(5, 305));
    assert!(kid_out_of_bounds(6, 299));
    assert!(!kid_out_of_bounds(6, 298));
    assert!(kid_out_of_bounds(7, 291));
    assert!(!kid_out_of_bounds(7, 290));
    assert!(kid_out_of_bounds(8, 0));
    assert!(kid_out_of_bounds(0xFE, 0));
}

// 3. Snowball creation: a kid in throw pose frame 5 spawns into the pool
//    (conditions at 0x14389-0x143A6, creation at 0x1448F).
#[test]
fn kid_throw_creates_snowball() {
    let mut g = playing_game(3);
    freeze_spawns(&mut g);
    let rec = &KID_SPAWN_SCRIPTS[0][0];
    g.participants[3].kids[0].spawn_from(rec, 5);
    // Force the throw pose so the next step advances to frame 5 with a
    // freshly reloaded delay - exactly when the original spawns a snowball.
    {
        let k = &mut g.participants[3].kids[0];
        k.behavior = 6;
        k.frame = 4;
        k.delay = 0;
        k.done = false;
    }
    g.step(&idle());
    assert!(
        g.participants[3].snowballs.iter().any(|s| s.active),
        "a snowball must be created when the kid reaches throw frame 5"
    );
    assert_eq!(
        g.meter, METER_PER_SNOWBALL,
        "creating a snowball adds 6 to the meter"
    );
    assert_eq!(g.participants[3].snowball_count, 1);
}

// 4. Snowball trajectory: scripted ballistic arc from the behavior table.
#[test]
fn snowball_follows_arc_and_expires() {
    let mut s = snowball_war::snowball::Snowball::default();
    let beh = SNOWBALL_BEHAVIOR;
    s.spawn(beh, 100, 150);
    let frames = &BEHAVIORS[beh].frames;
    // First step writes position from frame 0.
    assert!(!s.step());
    assert_eq!(
        (s.x, s.y),
        (100 + frames[0].2 as i32, 150 + frames[0].3 as i32)
    );
    // The arc rises (negative dy) before falling again.
    let mut min_y = s.y;
    let mut steps = 1;
    while !s.step() && steps < 100 {
        min_y = min_y.min(s.y);
        steps += 1;
    }
    assert!(s.done, "the snowball script must terminate");
    assert!(min_y < 150, "the arc must rise above the launch height");
    assert!(s.y > min_y, "the arc must fall back down (ballistic)");
    assert_eq!(steps + 1, frames.len(), "one step per animation frame");
    // Expiry deactivates it through the participant frame (0x14501).
    let mut g = playing_game(4);
    freeze_spawns(&mut g);
    g.participants[3].snowballs[0].spawn(beh, 50, 50);
    g.participants[3].snowball_count = 1;
    for _ in 0..frames.len() + 2 {
        g.step(&idle());
    }
    assert!(
        !g.participants[3].snowballs[0].active,
        "expired snowballs are removed"
    );
    assert_eq!(g.participants[3].snowball_count, 0);
}

// 5. Collision: the player's throw hits a kid whose box contains the attack
//    point (hit test at 0x14879-0x148A1). Kids' own snowballs never damage
//    the player (no such path exists in the original binary).
#[test]
fn throw_hits_overlapping_kid_only() {
    let mut g = playing_game(5);
    freeze_spawns(&mut g);
    let rec = &KID_SPAWN_SCRIPTS[0][0];
    // Kid at (100, 150): box x=[100,116], y=[122,150].
    g.participants[3].kids[0].spawn_from(rec, 5);
    {
        let k = &mut g.participants[3].kids[0];
        k.base_x = 100;
        k.base_y = 150;
        k.x = 100;
        k.y = 150;
        k.behavior = 35; // idle behavior so it does not walk away
    }
    // Aim the cursor so its hot point (x+9, y-8) is inside the real box.
    aim_at_kid(&mut g, 3, 0);
    let before = g.score;
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.score, before + 100, "hitting a type-5 kid scores 100");
    assert_eq!(
        g.participants[3].kids[0].kind, 0xFE,
        "hit kid switches to death state"
    );
    // Snowballs spent on the throw.
    assert_eq!(g.snowballs, SNOWBALL_START - 1);

    // A kid outside the box is not hit.
    let mut g2 = playing_game(5);
    freeze_spawns(&mut g2);
    g2.participants[3].kids[0].spawn_from(rec, 5);
    {
        let k = &mut g2.participants[3].kids[0];
        k.base_x = 200;
        k.base_y = 150;
        k.x = 200;
        k.y = 150;
        k.behavior = 35;
    }
    g2.player.x = 104;
    g2.player.y = 140;
    g2.player.debounce = 0;
    let before = g2.score;
    g2.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g2.score, before, "no hit outside the box");
    assert_ne!(g2.participants[3].kids[0].kind, 0xFE);

    // Kids' snowballs overlap the player but never damage them.
    g2.participants[0].snowballs[0].spawn(SNOWBALL_BEHAVIOR, g2.player.x, g2.player.y);
    g2.participants[0].snowball_count = 1;
    let (sx, sy) = (g2.player.x, g2.player.y);
    g2.step(&idle());
    assert_eq!(
        (g2.player.x, g2.player.y),
        (sx, sy),
        "player position is unaffected"
    );
    assert_eq!(
        g2.kids_left, KID_TARGET_BASE,
        "kids counter unchanged by snowballs"
    );
}

// 6. Arena obstacles: the recovered arena has no cover/obstacle collision;
//    objects only interact with the arena bounds (despawn thresholds above).
#[test]
fn arena_has_no_obstacle_collision() {
    // Movement is blocked solely by the arena clamps: a full diagonal sweep
    // never stops early.
    let mut g = playing_game(6);
    freeze_spawns(&mut g);
    g.player.x = 10;
    g.player.y = 60;
    let input = Input { dir: 3, ..idle() }; // down-right
    for _ in 0..60 {
        g.step(&input);
    }
    assert_eq!((g.player.x, g.player.y), (ARENA_W - 1, ARENA_BOTTOM));
}

// 7. Hit and damage handling: kid dies after its death animation, which is
//    when the "kids" counter decreases (0x143D8 / 0x14763).
#[test]
fn hit_kid_dies_and_counts_toward_level() {
    let mut g = playing_game(7);
    freeze_spawns(&mut g);
    let rec = &KID_SPAWN_SCRIPTS[0][0];
    g.participants[3].kids[0].spawn_from(rec, 6);
    {
        let k = &mut g.participants[3].kids[0];
        k.base_x = 100;
        k.base_y = 150;
        k.x = 100;
        k.y = 150;
        k.behavior = 35;
    }
    aim_at_kid(&mut g, 3, 0);
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.participants[3].kids[0].kind, 0xFE);
    assert_eq!(
        g.kids_left, KID_TARGET_BASE,
        "counter waits for the death animation"
    );
    assert_eq!(g.score, 75, "type-6 kid scores 75");
    // Run out the death behavior (behavior 19 for type 6).
    let death = BEHAVIORS[19];
    for _ in 0..((death.delay + 1) as usize) * death.frames.len() + 4 {
        g.step(&idle());
    }
    assert!(!g.participants[3].kids[0].active, "dead kid is removed");
    assert_eq!(
        g.kids_left,
        KID_TARGET_BASE - 1,
        "kids counter decreases on death"
    );
}

// 8. Participant "elimination" is the kid death path above; here we verify the
//    credit bookkeeping: unhit despawns refund the spawn credit (0x143D2),
//    kills do not (level target is consumed one by one).
#[test]
fn spawn_credit_recycled_on_despawn_not_on_kill() {
    let mut g = playing_game(8);
    freeze_spawns(&mut g);
    let rec = &KID_SPAWN_SCRIPTS[0][0];
    // Manual spawn consumes a credit like the original spawner.
    g.spawn_credits = KID_TARGET_BASE;
    g.spawn_credits -= 1;
    g.participants[3].kids[0].spawn_from(rec, 7);
    // Walk it out of the arena (x below 1).
    {
        let k = &mut g.participants[3].kids[0];
        k.base_x = 0;
        k.x = 0;
        k.y = 150;
        k.behavior = 35;
        k.frame = 0;
        k.delay = 0;
    }
    let credits_before = g.spawn_credits;
    g.step(&idle());
    assert_eq!(
        g.spawn_credits,
        credits_before + 1,
        "despawn refunds the credit"
    );

    // A kill does not refund.
    g.spawn_credits = 10;
    g.participants[3].kids[1].spawn_from(rec, 5);
    {
        let k = &mut g.participants[3].kids[1];
        k.base_x = 100;
        k.base_y = 150;
        k.x = 100;
        k.y = 150;
        k.behavior = 35;
    }
    g.player.x = 104;
    g.player.y = 140;
    g.player.debounce = 0;
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(
        g.spawn_credits, 10,
        "a kill does not return the spawn credit"
    );
}

// 9. Round completion: clearing the kids target awards the level bonus and
//    starts the next level with a grown target (0x13BF7).
#[test]
fn level_clear_awards_bonus_and_advances_level() {
    let mut g = playing_game(9);
    freeze_spawns(&mut g);
    g.snowballs = 120;
    let score_before = g.score;
    g.kids_left = 0;
    g.step(&idle());
    assert_eq!(g.level, 2, "level advances");
    assert_eq!(g.bonus_kids, KID_TARGET_BONUS, "bonus grows by 25");
    assert_eq!(
        g.score,
        score_before + LEVEL_CLEAR_BONUS + 120,
        "1000 + remaining snowballs are added to the score"
    );
    assert_eq!(g.message, Some("BONUS FOR CLEARING LEVEL"));
    // Freeze frames pass, then the next level is initialized.
    for _ in 0..121 {
        g.step(&idle());
    }
    assert_eq!(
        g.kids_left,
        KID_TARGET_BASE + KID_TARGET_BONUS,
        "target grows"
    );
    assert_eq!(
        g.time_left,
        TIME_START - 1,
        "timer restarts (first tick consumed)"
    );
    assert_eq!(g.level, 2);
    assert_eq!(g.clear_delay, 0);
}

// 10. Restarting a round: timeout ends the round, the action key returns to
//     the menu, and a new session starts fresh (entry at 0x13B0E).
#[test]
fn round_can_be_restarted() {
    let mut g = playing_game(10);
    freeze_spawns(&mut g);
    g.time_left = 0;
    g.step(&idle());
    assert_eq!(g.phase, Phase::RoundOver);
    let res = g.result.expect("round result");
    assert_eq!(res.message, "YOU'RE OUTTA TIME, SLICK!");
    // Remaining snowballs were converted into score.
    assert_eq!(g.snowballs, 0);
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.phase, Phase::Menu);
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.phase, Phase::Playing);
    assert_eq!(g.score, 0, "new session starts with a zero score");
    assert_eq!(g.level, 1);
    assert_eq!(g.snowballs, SNOWBALL_START);
    assert_eq!(g.kids_left, KID_TARGET_BASE);
    assert_eq!(g.bonus_kids, 0);
}

// 11. Deterministic behaviour with a fixed random seed.
#[test]
fn fixed_seed_is_deterministic() {
    let run = |seed: u32| -> String {
        let mut g = playing_game(seed);
        let mut log = String::new();
        for i in 0..400 {
            let dir = (i % 8 + 1) as u8;
            let input = Input {
                dir,
                action: i % 7 == 0,
                ..idle()
            };
            g.step(&input);
            log.push_str(&format!(
                "{},{},{},{},{},{},{}|",
                g.player.x,
                g.player.y,
                g.score,
                g.snowballs,
                g.kids_left,
                g.spawn_credits,
                g.roam.x
            ));
            for p in g.participants.iter() {
                for k in p.kids.iter() {
                    log.push_str(&format!("{}:{}:{}:{},", k.active, k.x, k.y, k.kind));
                }
            }
        }
        log
    };
    assert_eq!(run(0xC0FFEE), run(0xC0FFEE), "same seed, same evolution");
    assert_ne!(run(0xC0FFEE), run(0xBEEF), "different seeds diverge");
}

// The big "snowball in the face" close-up (behavior 21, sprites 152..155) is
// reserved for kids that reached the snow fort: the shipped binary maps it to
// kid type 8 anywhere on the field, which puts the close-up out in the snow.
#[test]
fn closeup_hit_animation_is_reserved_for_castle_kids() {
    let rec = &KID_SPAWN_SCRIPTS[0][0];

    // type-8 kid hit far from the castle -> small death, not the close-up
    let mut far = playing_game(20);
    freeze_spawns(&mut far);
    far.participants[3].kids[0].spawn_from(rec, 8);
    {
        let k = &mut far.participants[3].kids[0];
        k.base_x = 100;
        k.base_y = 140;
        k.behavior = 35;
    }
    aim_at_kid(&mut far, 3, 0);
    far.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(far.participants[3].kids[0].kind, 0xFE);
    assert_eq!(
        far.participants[3].kids[0].behavior, 20,
        "a far hit must not show the close-up face"
    );

    // type-8 kid at the castle -> close-up
    let mut near = playing_game(20);
    freeze_spawns(&mut near);
    near.participants[3].kids[0].spawn_from(rec, 8);
    {
        let k = &mut near.participants[3].kids[0];
        k.base_x = 140;
        k.base_y = CASTLE_Y + 5;
        k.behavior = 35;
    }
    aim_at_kid(&mut near, 3, 0);
    near.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(
        near.participants[3].kids[0].behavior, CLOSEUP_HIT_BEHAVIOR,
        "a castle hit shows the close-up"
    );

    // other kid types keep their own death animation, also at the castle
    let mut other = playing_game(20);
    freeze_spawns(&mut other);
    other.participants[3].kids[0].spawn_from(rec, 5);
    {
        let k = &mut other.participants[3].kids[0];
        k.base_x = 140;
        k.base_y = CASTLE_Y + 5;
        k.behavior = 35;
    }
    aim_at_kid(&mut other, 3, 0);
    other.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(
        other.participants[3].kids[0].behavior, 18,
        "type-5 keeps its own death animation"
    );
}

// Extra: kid spawn cadence (90 frame timer, shared credit budget).
#[test]
fn kids_spawn_on_the_recovered_timer() {
    let mut g = playing_game(11);
    freeze_spawns(&mut g);
    // Re-arm a single participant's timer to observe the interval.
    g.participants[3].spawn_timer = 1;
    g.step(&idle());
    assert!(
        g.participants[3].kids[0].active,
        "first spawn happens immediately"
    );
    assert_eq!(g.spawn_credits, KID_TARGET_BASE - 1);
    for _ in 0..SPAWN_INTERVAL - 1 {
        g.step(&idle());
    }
    assert!(
        !g.participants[3].kids[1].active,
        "no spawn before the 90 frame interval elapses"
    );
    g.step(&idle());
    assert!(
        g.participants[3].kids[1].active,
        "second spawn after 90 frames"
    );
}

// Extra: throw debounce (function 0x147C1: cooldown 4 -> every 5th frame)
// and the unlimited powerup (type 9 hit).
#[test]
fn throw_debounce_and_unlimited_powerup() {
    let mut g = playing_game(12);
    freeze_spawns(&mut g);
    g.player.debounce = 0;
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.snowballs, SNOWBALL_START - 1);
    assert_eq!(
        g.player.debounce, THROW_COOLDOWN,
        "debounce reloads with the cooldown"
    );
    // Holding the key cannot throw again immediately.
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.snowballs, SNOWBALL_START - 1);

    // Unlimited powerup: cooldown becomes 0 for 400 frames.
    g.player.cooldown = 0;
    g.unlimited_timer = UNLIMITED_FRAMES;
    g.player.debounce = -1;
    let before = g.snowballs;
    g.step(&Input {
        action: true,
        ..idle()
    });
    assert_eq!(g.snowballs, before, "unlimited throws cost no snowballs");
}

// Extra: time ticks every 41 frames (function 0x14A49).
#[test]
fn time_decrements_every_41_frames() {
    let mut g = playing_game(13);
    freeze_spawns(&mut g);
    let t0 = g.time_left;
    for _ in 0..40 {
        g.step(&idle());
    }
    assert_eq!(g.time_left, t0, "no tick within 40 frames");
    g.step(&idle());
    assert_eq!(g.time_left, t0 - 1, "tick on the 41st frame");
}

// Extra: participant switch (function 0x140EE) - RShift + cardinal selects.
#[test]
fn participant_switching() {
    let mut g = playing_game(14);
    freeze_spawns(&mut g);
    assert_eq!(g.selected, 3, "level start selects participant 3");
    let input = Input {
        dir: 6,
        modifier: true,
        ..idle()
    }; // left -> 2
    g.step(&input);
    assert_eq!(g.selected, 2, "RShift+left selects participant 2");
    g.step(&input);
    assert_eq!(g.selected, 2, "selection is stable while held");
}

// Kid helper used above: ensure behavior tables referenced by spawn scripts
// exist (guards the extracted data tables).
#[test]
fn extracted_tables_are_complete() {
    for slot in KID_SPAWN_SCRIPTS.iter() {
        for rec in slot.iter() {
            for &b in rec.script.iter() {
                if b == 0xFF {
                    continue;
                }
                let idx = b as usize;
                assert!(idx < BEHAVIOR_COUNT, "behavior {idx} missing");
                assert!(
                    !BEHAVIORS[idx].frames.is_empty(),
                    "behavior {idx} has no frames"
                );
            }
        }
    }
    let k = Kid::default();
    assert!(!k.active);
    let _ = Target::Kid {
        participant: 0,
        slot: 0,
    };
}

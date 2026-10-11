//! Snowball War simulation (deterministic, SDL-free).
//!
//! Reproduces the gameplay rules recovered from the Snowball War module of
//! Ski or Die's SKIEGA executable. Addresses cited below are linear addresses
//! in the REA/Ghidra analysis of the EXEPACK-unpacked load image (load
//! segment 0x1000); see docs/reverse-engineering-notes.md.

use crate::arena::{clamp_player, Rect};
use crate::data::*;
use crate::input::Input;
use crate::participant::{KidEvent, Participant, KID_SLOTS};
use crate::roam::Roam;

/// Fixed simulation rate. The original paces frames from the PIT timer
/// (function 0x161A4 waits on [0x6a26] >= [0x6a29]); the exact divisor is
/// not recoverable, 35 Hz is the documented approximation.
pub const SIM_HZ: u64 = 35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Menu,
    Playing,
    RoundOver,
}

/// Deterministic PRNG (xorshift32) standing in for the original routine at
/// 0x15D2A; seeded explicitly so tests are reproducible.
#[derive(Debug, Clone)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(seed | 1)
    }
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// Uniform byte in 0..=255 (same range as the original).
    pub fn byte(&mut self) -> u8 {
        (self.next_u32() >> 16) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Kid { participant: usize, slot: usize },
    Roam,
    Pickup,
}

/// One record of the frame's hit queue (original stores block/slot/type).
#[derive(Debug, Clone, Copy)]
pub struct QueueEntry {
    pub target: Target,
    pub kind: u8,
}

impl QueueEntry {
    pub fn rect(&self, g: &Game) -> Rect {
        match self.target {
            Target::Kid { participant, slot } => g.participants[participant].kids[slot].rect(),
            Target::Roam => {
                let (x, y) = g.roam.box_origin();
                Rect { x, y, w: 16, h: 24 }
            }
            Target::Pickup => Rect {
                x: g.pickup.x,
                y: g.pickup.y,
                w: 16,
                h: 16,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    /// [0x3fde] acceleration word: +40 per moving frame; the high byte is the
    /// speed level (0..=2, clamped by the code at 0x14049).
    pub accel: u16,
    /// [0x397d] throw debounce counter.
    pub debounce: i32,
    /// [0x397e] throw cooldown (4 normally, 0 while unlimited).
    pub cooldown: i32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            x: PLAYER_START_X,
            y: PLAYER_START_Y,
            accel: 0,
            debounce: 0,
            cooldown: THROW_COOLDOWN,
        }
    }
}

impl Player {
    pub fn speed_level(&self) -> usize {
        ((self.accel >> 8) & 0xFF) as usize
    }

    /// Player movement (function 0x13FDF).
    pub fn update_move(&mut self, dir: u8) {
        if dir == 0 {
            self.accel = 0;
        } else {
            let sl = self.speed_level().min(2);
            self.x += MOVE_DX[sl][dir as usize];
            self.y += MOVE_DY[sl][dir as usize];
            self.accel = self.accel.wrapping_add(40);
            if (self.accel >> 8) > 2 {
                self.accel = (self.accel & 0xFF) | 0x0200;
            }
        }
        let (x, y) = clamp_player(self.x, self.y);
        self.x = x;
        self.y = y;
    }

    /// Point used by the original hit test (x + 9, y - 8).
    pub fn attack_point(&self) -> (i32, i32) {
        (self.x + ATTACK_DX, self.y - ATTACK_DY)
    }
}

impl Game {
    /// Cursor marker sprite: green while throw-ready/unlimited, red otherwise
    /// (function 0x13FDF sets [0x19C] to 0x7C or 0x8F).
    pub fn cursor_sprite(&self) -> u8 {
        if self.player.cooldown == 0 || self.player.debounce != 0 {
            CURSOR_SPRITE_GREEN
        } else {
            CURSOR_SPRITE_RED
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pickup {
    pub active: bool,
    pub kind: u8,
    pub x: i32,
    pub y: i32,
    pub timer: i32,
    pub cursor: usize,
}

impl Default for Pickup {
    fn default() -> Self {
        Pickup {
            active: false,
            kind: 0,
            x: 0,
            y: 0,
            timer: 600,
            cursor: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundResult {
    pub message: &'static str,
    pub score: i32,
    pub level: i32,
}

#[derive(Debug)]
pub struct Game {
    pub phase: Phase,
    pub practice: bool,
    pub rng: Rng,
    pub player: Player,
    pub participants: [Participant; 4],
    /// [0x39cd]; reset to 3 at every level start (function 0x14116).
    pub selected: usize,
    pub score: i32,
    pub snowballs: i32,
    pub kids_left: i32,
    pub level: i32,
    pub time_left: i32,
    pub bonus_kids: i32,
    pub spawn_credits: i32,
    /// [0x3fdc] practice/difficulty meter.
    pub meter: i32,
    pub time_countdown: i32,
    pub ammo_empty_timer: i32,
    pub unlimited_timer: i32,
    pub message_timer: i32,
    pub practice_show: i32,
    pub abort: bool,
    pub slot_cursor: [usize; KID_SLOTS],
    pub roam: Roam,
    pub pickup: Pickup,
    pub queue: Vec<QueueEntry>,
    pub banner: Option<&'static str>,
    pub banner_timer: i32,
    pub flash: i32,
    pub message: Option<&'static str>,
    pub message_hold: i32,
    pub clear_delay: i32,
    pub result: Option<RoundResult>,
    pub paused: bool,
    pub hit_marker: i32,
    /// Name shown top-left in the HUD (original reads [0x9FE] player names).
    pub player_name: String,
}

impl Game {
    pub fn new(seed: u32) -> Self {
        // Entry sequence at 0x3B0E: counters are zeroed once, then the
        // attract/menu wait runs until the action key is pressed.
        Game {
            phase: Phase::Menu,
            practice: false,
            rng: Rng::new(seed),
            player: Player::default(),
            participants: Default::default(),
            selected: 3,
            score: 0,
            snowballs: SNOWBALL_START,
            kids_left: KID_TARGET_BASE,
            level: 1,
            time_left: TIME_START,
            bonus_kids: 0,
            spawn_credits: KID_TARGET_BASE,
            meter: 0,
            time_countdown: 0,
            ammo_empty_timer: AMMO_REFILL_FRAMES,
            unlimited_timer: 0,
            message_timer: 30,
            practice_show: -1,
            abort: false,
            slot_cursor: [0; KID_SLOTS],
            roam: Roam::default(),
            pickup: Pickup::default(),
            queue: Vec::new(),
            banner: None,
            banner_timer: 0,
            flash: 0,
            message: None,
            message_hold: 0,
            clear_delay: 0,
            result: None,
            paused: false,
            hit_marker: 0,
            player_name: "OLD NERD".to_string(),
        }
    }

    /// Start a session (action key on the menu).
    pub fn start(&mut self, practice: bool) {
        self.practice = practice;
        self.phase = Phase::Playing;
        self.result = None;
        self.score = 0;
        self.bonus_kids = 0;
        self.level = 1;
        self.snowballs = SNOWBALL_START;
        self.meter = 0;
        self.clear_delay = 0;
        self.level_init();
    }

    /// Level initialization (function 0x13DF0 plus the participant select at
    /// 0x14116 and the first time tick at 0x14A49).
    fn level_init(&mut self) {
        self.player.x = PLAYER_START_X;
        self.player.y = PLAYER_START_Y;
        self.player.accel = 0;
        self.player.cooldown = THROW_COOLDOWN;
        self.player.debounce = 0;
        self.selected = 3;
        self.abort = false;
        self.roam.reset();
        self.pickup = Pickup::default();
        self.time_countdown = 0;
        self.ammo_empty_timer = AMMO_REFILL_FRAMES;
        self.unlimited_timer = 0;
        self.banner = None;
        self.banner_timer = 0;
        self.message = None;
        self.message_hold = 0;
        self.clear_delay = 0;
        let target = KID_TARGET_BASE + self.bonus_kids;
        self.kids_left = target;
        self.spawn_credits = target;
        for p in self.participants.iter_mut() {
            p.reset();
        }
        self.slot_cursor = [0; KID_SLOTS];
        self.message_timer = 30;
        self.time_left = TIME_START;
        if self.practice {
            self.practice_show = -1;
            self.practice_meter_update();
        } else {
            self.time_countdown = 0;
            self.time_tick();
        }
    }

    fn time_tick(&mut self) {
        self.time_countdown -= 1;
        if self.time_countdown >= 0 {
            return;
        }
        self.time_countdown = TIME_FRAMES_PER_TICK - 1;
        if self.time_left > 0 {
            self.time_left -= 1;
        }
    }

    /// Practice meter check (function 0x14099).
    fn practice_meter_update(&mut self) {
        if self.meter < 0 {
            self.meter = 0;
        }
        let high = self.meter >> 8;
        if self.practice_show != high {
            self.practice_show = high;
        }
        if self.practice_show == PRACTICE_END_METER {
            self.abort = true;
        }
    }

    /// One fixed simulation step.
    pub fn step(&mut self, input: &Input) {
        match self.phase {
            Phase::Menu => {
                if input.toggle_practice {
                    self.practice = !self.practice;
                }
                if input.action {
                    self.start(self.practice);
                }
            }
            Phase::RoundOver => {
                if input.action {
                    self.phase = Phase::Menu;
                    self.result = None;
                }
            }
            Phase::Playing => {
                if input.pause {
                    self.paused = !self.paused;
                }
                if self.paused {
                    return;
                }
                // Freeze while the level-clear banner is shown, then run the
                // next level initialization (original jumps to 0x13B50).
                if self.clear_delay > 0 {
                    self.clear_delay -= 1;
                    if self.clear_delay == 0 {
                        self.level_init();
                    }
                    return;
                }
                self.playing_frame(input);
            }
        }
    }

    fn playing_frame(&mut self, input: &Input) {
        // 0x13B53 - player movement.
        self.player.update_move(input.dir);

        // 0x13B56 - kid spawn/step in the original order 3,1,0,2.
        for &idx in [3usize, 1, 0, 2].iter() {
            self.participant_kids_frame(idx);
        }
        // 0x13B6A - snowball (pool B) processing, same order.
        for &idx in [3usize, 1, 0, 2].iter() {
            self.participant_snowballs_frame(idx);
        }

        // 0x13B81 - roaming object, 0x13B84 - specials, 0x13B87 - message.
        self.roam.update(&mut self.rng);
        if self.roam.in_queue() {
            self.queue.push(QueueEntry {
                target: Target::Roam,
                kind: self.roam.kind,
            });
        }
        self.pickup_tick();

        self.message_timer -= 1;
        if self.message_timer <= 0 {
            self.message_timer = 30;
        }
        if self.banner_timer > 0 {
            self.banner_timer -= 1;
            if self.banner_timer == 0 {
                self.banner = None;
                self.player.cooldown = THROW_COOLDOWN;
            }
        }
        if self.flash > 0 {
            self.flash -= 1;
        }
        if self.hit_marker > 0 {
            self.hit_marker -= 1;
        }
        if self.message_hold > 0 {
            self.message_hold -= 1;
            if self.message_hold == 0 {
                self.message = None;
            }
        }

        // 0x13B8A - practice meter or time tick.
        if self.practice {
            self.practice_meter_update();
        } else {
            self.time_tick();
        }

        // 0x13BB5 - throw processing / hit resolution.
        self.throw_frame(input);
        // 0x13BC8 - queue counters are cleared after processing.
        self.queue.clear();
        // 0x13BCC - participant switch (function 0x140EE).
        if let Some(idx) = input.select_participant() {
            if self.selected != idx {
                self.selected = idx;
                self.message_timer = 30;
            }
        }

        // 0x13BD2 - round checks (kids first, then time/abort, then exit).
        if self.kids_left <= 0 {
            self.level_clear();
            return;
        }
        if self.time_left <= 0 || self.abort {
            let msg = if self.practice {
                "YOU'VE BEEN SNOWED, FROSTY!"
            } else {
                "YOU'RE OUTTA TIME, SLICK!"
            };
            // On time-out the original converts remaining snowballs to score.
            self.score += self.snowballs;
            self.snowballs = 0;
            self.finish_round(msg);
            return;
        }
        if input.exit {
            self.finish_round("SNOWFIGHT COMPETITION");
        }
    }

    fn participant_kids_frame(&mut self, idx: usize) {
        let selected = idx == self.selected;
        // Kid processing (function 0x14338) with per-frame queue logging.
        let mut events: Vec<(usize, KidEvent)> = Vec::new();
        {
            let p = &mut self.participants[idx];
            for (s, kid) in p.kids.iter_mut().enumerate() {
                if kid.active {
                    let target = Target::Kid {
                        participant: idx,
                        slot: s,
                    };
                    for ev in kid.update(selected, target, &mut self.queue) {
                        events.push((s, ev));
                    }
                }
            }
        }
        // Spawn attempt (function 0x1427D): the timer ticks through zero.
        let credits = self.spawn_credits;
        self.participants[idx].spawn_timer = self.participants[idx].spawn_timer.wrapping_sub(1);
        if self.participants[idx].spawn_timer == 0 {
            let free = self.participants[idx].kids.iter().position(|k| !k.active);
            if let (Some(slot), true) = (free, credits > 0) {
                self.participants[idx].spawn_timer = SPAWN_INTERVAL as u8;
                let cursor = self.slot_cursor[slot];
                let rec = &KID_SPAWN_SCRIPTS[slot][cursor];
                self.slot_cursor[slot] = (cursor + 1) % 10;
                let kind = pick_kid_kind(self.rng.byte());
                self.spawn_credits -= 1;
                self.participants[idx].kids[slot].spawn_from(rec, kind);
                if selected {
                    self.queue.push(QueueEntry {
                        target: Target::Kid {
                            participant: idx,
                            slot,
                        },
                        kind,
                    });
                }
            }
        }
        // Kid events (throw / recycle / kill).
        let mut throws: Vec<(usize, u8, i32, i32)> = Vec::new();
        for (_slot, ev) in events {
            match ev {
                KidEvent::Recycled => self.spawn_credits += 1,
                KidEvent::Killed => {
                    // FUN_1000_4763: kids counter decreases after the death
                    // animation completes.
                    self.kids_left -= 1;
                    self.hit_marker = 12;
                }
                KidEvent::Throw { x, y, kind } => throws.push((0, kind, x, y)),
            }
        }
        for (_, kind, x, y) in throws {
            // FUN_100c_43cf: snowball arc behavior depends on kid type.
            let beh = match kind {
                7 => 24,
                6 => 23,
                _ => SNOWBALL_BEHAVIOR,
            };
            self.snowball_create(idx, beh, x, y);
        }
        self.participants[idx].kid_count = self.participants[idx].active_kid_count();
    }

    fn participant_snowballs_frame(&mut self, idx: usize) {
        // FUN_100c_4441: step each active snowball; expiry removes it.
        let mut done_list = Vec::new();
        {
            let p = &mut self.participants[idx];
            for (s, sb) in p.snowballs.iter_mut().enumerate() {
                if sb.active && sb.step() {
                    done_list.push(s);
                }
            }
        }
        for s in done_list {
            let p = &mut self.participants[idx];
            p.snowballs[s].active = false;
            p.snowball_count = p.active_snowball_count();
        }
    }

    fn snowball_create(&mut self, idx: usize, behavior: usize, x: i32, y: i32) {
        self.meter += METER_PER_SNOWBALL; // FUN_100c_43cf adds 6 to [0x3fdc]
        let p = &mut self.participants[idx];
        if let Some(s) = p.snowballs.iter().position(|s| !s.active) {
            p.snowballs[s].spawn(behavior, x, y);
            p.snowball_count = p.active_snowball_count();
        }
    }

    /// Throw handling and hit resolution (functions 0x147C1 / 0x148B0).
    fn throw_frame(&mut self, input: &Input) {
        // Unlimited timer restore ([0x3998] / [0x397e]).
        if self.player.cooldown != THROW_COOLDOWN {
            self.unlimited_timer -= 1;
            if self.unlimited_timer <= 0 {
                self.unlimited_timer = 0;
                self.player.cooldown = THROW_COOLDOWN;
                self.banner = None;
            }
        }
        self.player.debounce -= 1;
        if self.player.debounce >= 0 {
            return;
        }
        self.player.debounce = 0;
        // Free refill while empty ([0x397c], FUN_1000_464B, FUN_1000_463F).
        if self.snowballs == 0 {
            self.ammo_empty_timer -= 1;
            if self.ammo_empty_timer < 0 {
                self.ammo_empty_timer = AMMO_REFILL_FRAMES;
                self.snowballs += AMMO_REFILL_AMOUNT;
                self.spawn_pickup(0);
            }
        }
        if !input.action {
            return;
        }
        // Action key pressed (device state 1).
        self.player.debounce = self.player.cooldown;
        if self.player.cooldown != 0 {
            self.snowballs = (self.snowballs - 1).max(0);
        }
        let (px, py) = self.player.attack_point();
        let hits: Vec<QueueEntry> = self
            .queue
            .iter()
            .rev()
            .filter(|e| e.kind != 0xFE)
            .filter(|e| e.rect(self).contains_player_point(px, py))
            .copied()
            .collect();
        for e in hits {
            self.resolve_hit(e);
        }
    }

    fn resolve_hit(&mut self, e: QueueEntry) {
        match e.kind {
            0 => {
                self.snowballs += PICKUP_SNOWBALLS;
                self.pickup.active = false;
            }
            1 => {
                self.meter -= 0x100;
                self.pickup.active = false;
            }
            4 => {
                self.snowballs = (self.snowballs - 25).max(0);
                self.roam.on_hit();
            }
            9 => {
                self.player.cooldown = 0;
                self.unlimited_timer = UNLIMITED_FRAMES;
                self.banner = Some("UNLIMITED");
                self.banner_timer = UNLIMITED_FRAMES;
                self.pickup.active = false;
            }
            2 | 3 | 10 | 11 | 12 => {
                self.score += score_for(e.kind);
                self.roam.on_hit();
            }
            k if (5..=8).contains(&k) => {
                self.score += score_for(k);
                if let Target::Kid { participant, slot } = e.target {
                    let at_castle = self.participants[participant].kids[slot].y >= CASTLE_Y;
                    self.participants[participant].kids[slot].kill(at_castle);
                    self.flash = 8;
                    self.hit_marker = 20;
                }
            }
            _ => {}
        }
    }

    /// Spawn a special pickup at the next table position (FUN_1000_463F when
    /// called with the forced parameter, e.g. 125 for a snowball pickup).
    fn spawn_pickup(&mut self, forced: u8) {
        let kind = forced;
        let pos_idx = self.pickup.cursor;
        let (x, y) = PICKUP_POSITIONS[pos_idx % PICKUP_POSITIONS.len()];
        self.pickup.cursor = (self.pickup.cursor + 1) % 10;
        self.pickup.active = true;
        self.pickup.kind = kind;
        self.pickup.x = x;
        self.pickup.y = y.min(ARENA_BOTTOM);
        self.pickup.timer = 600;
        // FUN_1000_463F logs the fresh pickup into this frame's queue.
        self.queue.push(QueueEntry {
            target: Target::Pickup,
            kind,
        });
    }

    /// Per-frame special lifetime (function 0x1462F).
    fn pickup_tick(&mut self) {
        if !self.pickup.active {
            self.pickup.timer -= 1;
            if self.pickup.timer > 0 {
                return;
            }
            // Spawn a random special (types 0/1/9, thresholds 125/50/0).
            let r = self.rng.byte();
            let kind = if r >= 125 {
                0
            } else if r >= 50 {
                1
            } else {
                9
            };
            let pos_idx = self.pickup.cursor;
            let (x, y) = PICKUP_POSITIONS[pos_idx % PICKUP_POSITIONS.len()];
            self.pickup.cursor = (self.pickup.cursor + 1) % 10;
            self.pickup.active = true;
            self.pickup.kind = kind;
            self.pickup.x = x;
            self.pickup.y = y.min(ARENA_BOTTOM);
            self.pickup.timer = 600;
            // Falls through: the original logs the pickup on its spawn frame.
        }
        self.pickup.timer -= 1;
        if self.pickup.timer <= 0 {
            self.pickup.active = false;
            self.pickup.timer = 600;
        } else {
            self.queue.push(QueueEntry {
                target: Target::Pickup,
                kind: self.pickup.kind,
            });
        }
    }

    /// Level-clear path (0x13BF7).
    fn level_clear(&mut self) {
        self.score += LEVEL_CLEAR_BONUS + self.snowballs;
        self.message = Some("BONUS FOR CLEARING LEVEL");
        self.message_hold = 120;
        self.bonus_kids += KID_TARGET_BONUS;
        self.level += 1;
        self.meter -= METER_PER_LEVEL;
        if self.meter < 0 {
            self.meter = 0;
        }
        self.clear_delay = 120;
    }

    fn finish_round(&mut self, msg: &'static str) {
        self.result = Some(RoundResult {
            message: msg,
            score: self.score,
            level: self.level,
        });
        self.phase = Phase::RoundOver;
        self.paused = false;
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(0x5EED)
    }
}

/// Kid type selection (threshold table at DS:0x41EF).
pub fn pick_kid_kind(r: u8) -> u8 {
    for (i, t) in KID_THRESHOLDS.iter().enumerate() {
        if *t <= r {
            return KID_TYPES[i];
        }
    }
    *KID_TYPES.last().unwrap()
}

fn score_for(kind: u8) -> i32 {
    SCORE_BY_TYPE
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, s)| *s)
        .unwrap_or(0)
}

impl Participant {
    pub fn active_kid_count(&self) -> u8 {
        self.kids.iter().filter(|k| k.active).count() as u8
    }
    pub fn active_snowball_count(&self) -> u8 {
        self.snowballs.iter().filter(|s| s.active).count() as u8
    }
}

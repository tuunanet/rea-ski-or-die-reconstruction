//! Gameplay constants and tables recovered from the Snowball War module
//! of the Ski or Die DOS executable (SKIEGA, EXEPACK-unpacked load image).
//!
//! Every table below was extracted from the original binary during the REA
//! reverse-engineering session; see docs/reverse-engineering-notes.md for the
//! evidence trail. Only numeric gameplay data is reproduced here - no art,
//! sound or source code from the original is used.

pub const BEHAVIOR_COUNT: usize = 41;
/// Scripted motion behaviors: per-frame (dx, dy) offsets applied to the spawn base.
/// `delay` = frames each animation frame is held (delay 0 -> advance every frame).
pub static BEHAVIORS: [Behavior; BEHAVIOR_COUNT] = [
    Behavior {
        delay: 0,
        frames: &[
            (0, 0),
            (3, 2),
            (7, 2),
            (8, 2),
            (6, 1),
            (10, 2),
            (13, 2),
            (15, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 0),
            (1, 2),
            (-1, 2),
            (-4, 2),
            (-6, 1),
            (-9, 2),
            (-9, 2),
            (-10, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 0),
            (4, 2),
            (12, 2),
            (14, 2),
            (15, 0),
            (20, 2),
            (28, 2),
            (31, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 0),
            (-2, 2),
            (-4, 2),
            (-10, 2),
            (-16, 0),
            (-19, 2),
            (-20, 2),
            (-24, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 0),
            (6, 2),
            (18, 2),
            (22, 4),
            (24, 0),
            (30, 2),
            (42, 3),
            (46, 3),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 0),
            (-4, 2),
            (-8, 3),
            (-16, 4),
            (-25, 0),
            (-28, 2),
            (-32, 3),
            (-37, 3),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (2, 0), (3, 0), (4, 0), (4, 2), (3, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (1, 0), (1, -1), (0, 0), (4, 0), (4, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (-4, 0),
            (-8, 1),
            (1, 2),
            (0, 2),
            (-4, 2),
            (1, 2),
            (0, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[(0, 0), (-1, 0), (2, 0), (0, 0)],
    },
    Behavior {
        delay: 4,
        frames: &[(0, 0), (1, 0), (-1, 0), (-1, 0)],
    },
    Behavior {
        delay: 4,
        frames: &[(0, 0), (-4, 0), (-4, 0), (-4, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (-1, 0),
            (0, 0),
            (2, 0),
            (4, 1),
            (7, 0),
            (10, 0),
            (16, 0),
            (19, 0),
            (21, 0),
            (25, -1),
            (27, 0),
            (27, 0),
            (26, 0),
            (27, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (-1, 0),
            (0, 0),
            (0, 0),
            (-2, -1),
            (-6, 0),
            (-8, 0),
            (-11, 0),
            (-17, 0),
            (-20, 0),
            (-23, 1),
            (-25, 0),
            (-27, 0),
            (-28, 0),
            (-27, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (-2, 0),
            (4, 0),
            (6, 0),
            (12, 0),
            (18, 0),
            (22, 0),
            (25, 0),
            (28, 0),
            (32, 0),
            (34, 0),
            (33, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (1, 0),
            (-1, 0),
            (-5, 0),
            (-8, 0),
            (-11, 0),
            (-15, 0),
            (-21, 0),
            (-27, 0),
            (-29, 0),
            (-35, 0),
            (-33, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (0, 0),
            (14, 3),
            (20, 1),
            (30, 0),
            (36, -1),
            (39, 0),
            (46, -1),
            (51, 1),
            (51, 1),
            (55, 1),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (-4, 0),
            (-4, 0),
            (-9, -2),
            (-16, -1),
            (-19, -2),
            (-25, -1),
            (-35, 0),
            (-41, 2),
            (-46, -2),
            (-55, -1),
            (-55, -1),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (0, -11),
            (-1, -15),
            (-1, -10),
            (-1, -4),
            (0, 1),
            (0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 3), (-1, 8), (2, 9), (-1, 8)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (2, -2), (0, 0), (0, -2), (-1, -4), (-1, -3)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 0), (1, -1), (1, -1), (1, -1), (1, -1), (1, -1)],
    },
    Behavior {
        delay: 0,
        frames: &[
            (-1, -7),
            (-1, -10),
            (-1, -15),
            (-1, -17),
            (-1, -11),
            (-2, -12),
            (-2, -10),
            (-2, -8),
            (-2, -3),
            (-3, 8),
            (-3, 8),
            (-3, 18),
            (-4, 28),
            (-4, 44),
            (-4, 71),
            (-5, 82),
            (-5, 82),
            (-7, 83),
            (-10, 86),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (-1, -16),
            (-1, -18),
            (-2, -21),
            (-2, -23),
            (-2, -21),
            (-2, -19),
            (-3, -14),
            (-4, -5),
            (-4, 3),
            (-4, 11),
            (-5, 22),
            (-6, 42),
            (-7, 51),
            (-9, 60),
            (-9, 60),
            (-11, 61),
            (-14, 64),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (3, -22),
            (3, -23),
            (3, -25),
            (3, -26),
            (3, -26),
            (3, -25),
            (3, -17),
            (3, -15),
            (2, -10),
            (2, -4),
            (2, 2),
            (2, 11),
            (2, 28),
            (2, 43),
            (2, 43),
            (0, 44),
            (-3, 47),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, -1), (0, -1)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 2), (0, 2), (0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 4,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (2, -13),
            (2, -16),
            (3, -16),
            (4, -10),
            (3, -7),
            (5, -2),
            (5, 2),
            (5, 7),
            (5, 10),
            (5, 10),
            (5, 10),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (6, 0),
            (1, 0),
            (1, 0),
            (1, 0),
            (1, 0),
            (1, 0),
            (1, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[(0, 0), (1, -5), (6, -4), (6, -2), (6, -2), (6, -2), (6, -2)],
    },
    Behavior {
        delay: 5,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 5,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 5,
        frames: &[(0, 0), (0, 0), (0, 0), (0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (-7, 0),
            (-14, 0),
            (-14, 0),
            (-12, 0),
            (-2, 0),
            (-2, 0),
            (-2, 0),
            (-7, 0),
            (0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[(0, -4), (-1, -10), (0, -8), (1, -5), (1, -6)],
    },
];

#[derive(Clone, Copy, Debug)]
pub struct Behavior {
    pub delay: i32,
    pub frames: &'static [(i8, i8)],
}

/// Per-slot kid spawn scripts: (spawn_x, spawn_y, initial_behavior, script).
/// The script is a sequence of behavior indices terminated by 0xFF;
/// behavior indices 6..=8 are throw animations (frame 5 spawns a snowball).
pub static KID_SPAWN_SCRIPTS: [[KidSpawn; 10]; 4] = [
    [
        KidSpawn {
            x: 1,
            y: 105,
            behavior: 0,
            script: &[
                0, 0, 6, 0, 0, 6, 0, 0, 9, 6, 0, 0, 9, 0, 0, 9, 6, 0, 0, 0, 0, 6, 9, 0, 0, 255,
            ],
        },
        KidSpawn {
            x: 1,
            y: 102,
            behavior: 0,
            script: &[0, 0, 0, 0, 9, 6, 9, 1, 1, 1, 6, 9, 6, 1, 1, 1, 1, 1, 255],
        },
        KidSpawn {
            x: 303,
            y: 108,
            behavior: 1,
            script: &[
                1, 1, 1, 1, 6, 9, 6, 1, 1, 9, 9, 6, 9, 6, 1, 1, 1, 1, 1, 1, 1, 1, 6, 9, 6, 1, 1, 1,
                1, 9, 6, 9, 1, 1, 1, 1,
            ],
        },
        KidSpawn {
            x: 1,
            y: 105,
            behavior: 0,
            script: &[0, 0, 0, 0, 12, 6, 0, 0, 0, 12, 9, 6, 0, 0, 0, 12, 0, 0, 255],
        },
        KidSpawn {
            x: 1,
            y: 102,
            behavior: 0,
            script: &[
                0, 0, 0, 0, 0, 9, 6, 12, 6, 0, 9, 0, 6, 12, 6, 35, 0, 0, 12, 6, 35, 0, 0, 0, 255,
            ],
        },
        KidSpawn {
            x: 1,
            y: 108,
            behavior: 0,
            script: &[
                0, 6, 12, 6, 12, 6, 35, 9, 0, 0, 0, 0, 0, 12, 6, 9, 0, 0, 12, 9, 6, 0, 0, 255,
            ],
        },
        KidSpawn {
            x: 303,
            y: 102,
            behavior: 1,
            script: &[
                1, 1, 1, 1, 6, 9, 6, 1, 1, 1, 9, 6, 13, 6, 35, 1, 1, 1, 1, 1, 6, 13, 6, 1, 1, 6, 9,
                6, 1, 1, 1, 1, 1, 255,
            ],
        },
        KidSpawn {
            x: 303,
            y: 105,
            behavior: 1,
            script: &[
                1, 1, 1, 1, 1, 1, 1, 13, 6, 12, 0, 0, 0, 12, 6, 35, 1, 1, 1, 1, 1, 1, 1, 13, 6, 35,
                13, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        },
        KidSpawn {
            x: 303,
            y: 108,
            behavior: 1,
            script: &[
                1, 1, 1, 1, 1, 1, 1, 1, 13, 6, 35, 9, 1, 1, 1, 1, 1, 1, 1, 9, 6, 13, 6, 1, 1, 255,
            ],
        },
        KidSpawn {
            x: 303,
            y: 108,
            behavior: 1,
            script: &[1, 9, 6, 13, 6, 35, 1, 9, 6, 35, 0, 12, 6, 0, 0, 255],
        },
    ],
    [
        KidSpawn {
            x: 1,
            y: 125,
            behavior: 2,
            script: &[2, 2, 7, 10, 2, 10, 7, 2, 2, 2, 7, 10, 7, 2, 2, 255],
        },
        KidSpawn {
            x: 298,
            y: 121,
            behavior: 3,
            script: &[3, 10, 10, 7, 3, 3, 3, 3, 7, 7, 10, 7, 3, 3, 3, 7, 3, 3, 255],
        },
        KidSpawn {
            x: 1,
            y: 129,
            behavior: 2,
            script: &[2, 2, 10, 7, 2, 7, 10, 7, 3, 3, 3, 7, 7, 3, 3, 3, 255],
        },
        KidSpawn {
            x: 1,
            y: 121,
            behavior: 2,
            script: &[36, 2, 2, 10, 7, 15, 3, 10, 7, 36, 3, 3, 255],
        },
        KidSpawn {
            x: 3,
            y: 125,
            behavior: 2,
            script: &[
                14, 7, 2, 2, 10, 7, 36, 15, 7, 2, 2, 2, 7, 14, 7, 10, 36, 2, 2, 255,
            ],
        },
        KidSpawn {
            x: 1,
            y: 129,
            behavior: 2,
            script: &[2, 2, 2, 7, 10, 14, 7, 36, 2, 2, 7, 3, 7, 2, 14, 2, 255],
        },
        KidSpawn {
            x: 1,
            y: 121,
            behavior: 2,
            script: &[2, 14, 7, 2, 2, 10, 2, 7, 36, 10, 14, 2, 2, 255],
        },
        KidSpawn {
            x: 298,
            y: 125,
            behavior: 3,
            script: &[3, 3, 3, 10, 7, 15, 3, 10, 3, 3, 7, 15, 3, 3, 255],
        },
        KidSpawn {
            x: 298,
            y: 121,
            behavior: 3,
            script: &[
                3, 15, 10, 7, 15, 7, 3, 3, 7, 36, 10, 7, 3, 3, 15, 7, 10, 3, 3, 255,
            ],
        },
        KidSpawn {
            x: 298,
            y: 129,
            behavior: 3,
            script: &[
                3, 7, 10, 15, 7, 36, 3, 3, 3, 10, 3, 10, 7, 15, 7, 3, 3, 3, 255,
            ],
        },
    ],
    [
        KidSpawn {
            x: 8,
            y: 145,
            behavior: 4,
            script: &[4, 8, 8, 4, 4, 8, 4, 4, 255],
        },
        KidSpawn {
            x: 290,
            y: 140,
            behavior: 5,
            script: &[5, 5, 8, 5, 8, 11, 5, 8, 11, 8, 5, 5, 255],
        },
        KidSpawn {
            x: 8,
            y: 150,
            behavior: 4,
            script: &[8, 4, 4, 8, 11, 5, 5, 8, 11, 8, 5, 5, 255],
        },
        KidSpawn {
            x: 290,
            y: 145,
            behavior: 5,
            script: &[5, 5, 8, 5, 11, 8, 5, 5, 5, 255],
        },
        KidSpawn {
            x: 290,
            y: 140,
            behavior: 17,
            script: &[8, 37, 37, 11, 8, 5, 5, 8, 37, 11, 17, 5, 8, 5, 5, 255],
        },
        KidSpawn {
            x: 290,
            y: 150,
            behavior: 5,
            script: &[8, 37, 5, 11, 17, 8, 11, 5, 5, 8, 11, 8, 5, 5, 255],
        },
        KidSpawn {
            x: 8,
            y: 140,
            behavior: 16,
            script: &[8, 37, 11, 4, 16, 8, 11, 4, 8, 4, 4, 255],
        },
        KidSpawn {
            x: 8,
            y: 150,
            behavior: 4,
            script: &[4, 8, 11, 5, 8, 4, 4, 8, 37, 16, 8, 4, 4, 255],
        },
        KidSpawn {
            x: 8,
            y: 145,
            behavior: 4,
            script: &[16, 8, 37, 11, 4, 4, 8, 37, 5, 11, 8, 4, 4, 4, 255],
        },
        KidSpawn {
            x: 8,
            y: 140,
            behavior: 4,
            script: &[16, 8, 37, 11, 4, 4, 8, 4, 4, 255],
        },
    ],
    [
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
        KidSpawn {
            x: 146,
            y: 169,
            behavior: 38,
            script: &[255],
        },
    ],
];
#[derive(Clone, Copy, Debug)]
pub struct KidSpawn {
    pub x: i32,
    pub y: i32,
    pub behavior: usize,
    pub script: &'static [u8],
}

/// Movement deltas per direction (index 0..=8, numpad order) for speed levels 0..=2.
pub static MOVE_DX: [[i32; 9]; 3] = [
    [0, 2, 2, 2, 0, -2, -2, -2, 0],
    [0, 4, 4, 4, 0, -4, -4, -4, 0],
    [0, 6, 6, 6, 0, -6, -6, -6, 0],
];
pub static MOVE_DY: [[i32; 9]; 3] = [
    [0, -2, 0, 2, 2, 2, 0, -2, -2],
    [0, -4, 0, 4, 4, 4, 0, -4, -4],
    [0, -6, 0, 6, 6, 6, 0, -6, -6],
];

/// Kid type selection: first threshold <= random byte selects the type (binary order).
pub static KID_THRESHOLDS: [u8; 4] = [190, 127, 50, 0];
pub static KID_TYPES: [u8; 4] = [5, 6, 7, 8];
/// Score granted per kid type on a confirmed hit.
pub static SCORE_BY_TYPE: [(u8, i32); 8] = [
    (2, 50),
    (3, 30),
    (5, 100),
    (6, 75),
    (7, 50),
    (8, 25),
    (10, 30),
    (11, 30),
];
/// Death animation behavior per kid type (types 5..=8).
pub static DEATH_BEHAVIOR: [(u8, usize); 4] = [(5, 18), (6, 19), (7, 20), (8, 21)];
/// Snowball arc behavior chosen when a kid of the given type throws (type 7/6 differ).
pub static SNOWBALL_BEHAVIOR: usize = 22;
/// X nudge applied to the walk base when leaving behavior index `old` (binary table 0x41f6).
pub static BEHAVIOR_XOFF: [i8; 6] = [2, -4, 4, -6, 6, -8];

/// Roaming special object: respawn delay, random type thresholds and travel params.
pub const ROAM_RESPAWN_FRAMES: i32 = 400;
pub static ROAM_THRESHOLDS: [u8; 6] = [200, 160, 120, 80, 40, 0];
pub static ROAM_TYPES: [u8; 6] = [2, 3, 4, 10, 11, 12];
/// (start_x, y, x_limit, dx) per selected type.
pub static ROAM_PARAMS: [(i32, i32, i32, i32); 6] = [
    (300, 110, 300, -4),
    (275, 140, 275, -4),
    (0, 155, 285, 4),
    (290, 155, 290, -2),
    (2, 140, 286, 2),
    (290, 155, 290, -2),
];
/// Positions used for out-of-ammo snowball pickups (x, y).
pub static PICKUP_POSITIONS: [(i32, i32); 5] =
    [(282, 271), (222, 217), (171, 127), (118, 67), (24, 12)];

// --- recovered scalar constants (see behavior-spec.md) ---
pub const ARENA_W: i32 = 300; // playfield width (player x clamp 0..299)
pub const ARENA_TOP: i32 = 53; // player y minimum (HUD occupies rows above)
pub const ARENA_BOTTOM: i32 = 199; // player/object y maximum
pub const OBJECT_X_MAX: i32 = 319; // object x clamp from [0x6ae1]
pub const PLAYER_START_X: i32 = 150; // initial player x ([0x190])
pub const PLAYER_START_Y: i32 = 150; // initial player y ([0x192])
pub const SPAWN_INTERVAL: i32 = 90; // frames between kid spawns per participant
pub const MAX_KIDS_PER_GROUP: i32 = 4; // active kid slots (slot 5 cursor is disabled)
pub const MAX_SNOWBALLS_PER_GROUP: i32 = 3; // pool B slots per participant
pub const KID_TARGET_BASE: i32 = 50; // kids needed per level
pub const KID_TARGET_BONUS: i32 = 25; // bonus kids added per cleared level
pub const LEVEL_CLEAR_BONUS: i32 = 1000; // score bonus for clearing a level
pub const TIME_START: i32 = 60; // time units at level start
pub const TIME_FRAMES_PER_TICK: i32 = 41; // frames between time decrements
pub const SNOWBALL_START: i32 = 200; // snowballs at level start
pub const THROW_COOLDOWN: i32 = 4; // frames of throw debounce ([0x397e])
pub const UNLIMITED_FRAMES: i32 = 400; // duration of the unlimited-snowball powerup
pub const AMMO_REFILL_AMOUNT: i32 = 10; // snowballs granted when empty ([0x397c] timer fires)
pub const AMMO_REFILL_FRAMES: i32 = 50; // frames between free refills while empty
pub const PICKUP_SNOWBALLS: i32 = 100; // snowballs granted by a type-0 pickup hit
pub const PRACTICE_END_METER: i32 = 15; // practice meter high byte that ends a practice round
pub const METER_PER_SNOWBALL: i32 = 6; // [0x3fdc] added when a kid throws
pub const METER_PER_LEVEL: i32 = 768; // [0x3fdc] subtracted when a level is cleared
pub const ATTACK_DX: i32 = 9; // player attack point x offset (px + 9)
pub const ATTACK_DY: i32 = 8; // player attack point y offset (py - 8)
pub const SIM_HZ: i32 = 35; // simulation steps per second (approximation, see notes)

/// Direction codes (numpad order) used by the original keyboard handler.
pub const DIR_NONE: u8 = 0;

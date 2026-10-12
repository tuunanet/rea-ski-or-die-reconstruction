//! Gameplay constants and tables recovered from the Snowball War module
//! of the Ski or Die DOS executable (SKIEGA, EXEPACK-unpacked load image).
//!
//! Every table below was extracted from the original binary during the REA
//! reverse-engineering session; see docs/reverse-engineering-notes.md and
//! docs/assets.md for the evidence trail. Only numeric gameplay data is
//! reproduced here - no art, sound or source code from the original is used.

pub const BEHAVIOR_COUNT: usize = 41;
/// Scripted motion behaviors: per-frame (sprite, mirror, dx, dy).
///  * sprite  - index into the SFFBM sprite bank (0..=164)
///  * mirror  - horizontal mirror flag ([SI+0xE], applied by 0x177E7)
///  * dx, dy  - offsets from the spawn base position
///
/// `delay` = frames each animation frame is held (0 -> advance every frame).
#[derive(Clone, Copy, Debug)]
pub struct Behavior {
    pub delay: i32,
    pub frames: &'static [(u8, u8, i8, i8)],
}
pub static BEHAVIORS: [Behavior; BEHAVIOR_COUNT] = [
    Behavior {
        delay: 0,
        frames: &[
            (14, 0, 0, 0),
            (15, 0, 3, 2),
            (16, 0, 7, 2),
            (17, 0, 8, 2),
            (18, 0, 6, 1),
            (19, 0, 10, 2),
            (16, 0, 13, 2),
            (20, 0, 15, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (14, 1, 0, 0),
            (15, 1, 1, 2),
            (16, 1, -1, 2),
            (17, 1, -4, 2),
            (18, 1, -6, 1),
            (19, 1, -9, 2),
            (15, 1, -9, 2),
            (20, 1, -10, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (7, 0, 0, 0),
            (8, 0, 4, 2),
            (9, 0, 12, 2),
            (10, 0, 14, 2),
            (11, 0, 15, 0),
            (12, 0, 20, 2),
            (9, 0, 28, 2),
            (13, 0, 31, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (7, 1, 0, 0),
            (8, 1, -2, 2),
            (9, 1, -4, 2),
            (10, 1, -10, 2),
            (11, 1, -16, 0),
            (12, 1, -19, 2),
            (9, 1, -20, 2),
            (13, 1, -24, 2),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 0, 0, 0),
            (1, 0, 6, 2),
            (2, 0, 18, 2),
            (3, 0, 22, 4),
            (4, 0, 24, 0),
            (5, 0, 30, 2),
            (2, 0, 42, 3),
            (6, 0, 46, 3),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (0, 1, 0, 0),
            (1, 1, -4, 2),
            (2, 1, -8, 3),
            (3, 1, -16, 4),
            (4, 1, -25, 0),
            (5, 1, -28, 2),
            (2, 1, -32, 3),
            (6, 1, -37, 3),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (35, 0, 0, 0),
            (36, 0, 2, 0),
            (37, 0, 3, 0),
            (38, 0, 4, 0),
            (39, 0, 4, 2),
            (40, 0, 3, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (29, 0, 0, 0),
            (30, 0, 1, 0),
            (31, 0, 1, -1),
            (32, 0, 0, 0),
            (33, 0, 4, 0),
            (34, 0, 4, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (21, 0, 0, 0),
            (22, 0, -4, 0),
            (23, 0, -8, 1),
            (24, 0, 1, 2),
            (25, 0, 0, 2),
            (26, 0, -4, 2),
            (27, 0, 1, 2),
            (28, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[(35, 0, 0, 0), (62, 0, -1, 0), (64, 0, 2, 0), (63, 0, 0, 0)],
    },
    Behavior {
        delay: 4,
        frames: &[(29, 0, 0, 0), (59, 0, 1, 0), (61, 0, -1, 0), (60, 0, -1, 0)],
    },
    Behavior {
        delay: 4,
        frames: &[
            (21, 0, 0, 0),
            (56, 0, -4, 0),
            (58, 0, -4, 0),
            (57, 0, -4, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (35, 0, 0, 0),
            (62, 0, -1, 0),
            (63, 0, 0, 0),
            (64, 0, 2, 0),
            (86, 1, 4, 1),
            (85, 1, 7, 0),
            (84, 1, 10, 0),
            (83, 1, 16, 0),
            (82, 1, 19, 0),
            (81, 1, 21, 0),
            (80, 1, 25, -1),
            (79, 1, 27, 0),
            (63, 0, 27, 0),
            (62, 0, 26, 0),
            (35, 0, 27, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (35, 0, 0, 0),
            (62, 0, -1, 0),
            (63, 0, 0, 0),
            (79, 0, 0, 0),
            (80, 0, -2, -1),
            (81, 0, -6, 0),
            (82, 0, -8, 0),
            (83, 0, -11, 0),
            (84, 0, -17, 0),
            (85, 0, -20, 0),
            (86, 0, -23, 1),
            (64, 0, -25, 0),
            (63, 0, -27, 0),
            (62, 0, -28, 0),
            (35, 0, -27, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (59, 0, 0, 0),
            (60, 0, -2, 0),
            (78, 1, 4, 0),
            (77, 1, 6, 0),
            (76, 1, 12, 0),
            (75, 1, 18, 0),
            (74, 1, 22, 0),
            (73, 1, 25, 0),
            (72, 1, 28, 0),
            (61, 0, 32, 0),
            (59, 0, 34, 0),
            (29, 0, 33, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (29, 0, 0, 0),
            (59, 0, 1, 0),
            (61, 0, -1, 0),
            (72, 0, -5, 0),
            (73, 0, -8, 0),
            (74, 0, -11, 0),
            (75, 0, -15, 0),
            (76, 0, -21, 0),
            (77, 0, -27, 0),
            (78, 0, -29, 0),
            (60, 0, -35, 0),
            (59, 0, -33, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (56, 0, 0, 0),
            (57, 0, 0, 0),
            (70, 1, 14, 3),
            (69, 1, 20, 1),
            (68, 1, 30, 0),
            (67, 1, 36, -1),
            (66, 1, 39, 0),
            (65, 1, 46, -1),
            (58, 0, 51, 1),
            (56, 0, 51, 1),
            (21, 0, 55, 1),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (21, 0, 0, 0),
            (56, 0, -4, 0),
            (58, 0, -4, 0),
            (65, 0, -9, -2),
            (66, 0, -16, -1),
            (67, 0, -19, -2),
            (68, 0, -25, -1),
            (69, 0, -35, 0),
            (70, 0, -41, 2),
            (71, 0, -46, -2),
            (58, 0, -55, -1),
            (57, 0, -55, -1),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (52, 0, 0, 0),
            (53, 0, 0, -11),
            (54, 0, -1, -15),
            (54, 0, -1, -10),
            (54, 0, -1, -4),
            (55, 0, 0, 1),
            (55, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (47, 0, 0, 0),
            (48, 0, 0, 3),
            (49, 0, -1, 8),
            (50, 0, 2, 9),
            (51, 0, -1, 8),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (41, 0, 0, 0),
            (42, 0, 2, -2),
            (43, 0, 0, 0),
            (44, 0, 0, -2),
            (45, 0, -1, -4),
            (46, 0, -1, -3),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (152, 0, 0, 0),
            (153, 0, 0, 0),
            (154, 0, 1, -1),
            (155, 0, 1, -1),
            (154, 0, 1, -1),
            (155, 0, 1, -1),
            (155, 0, 1, -1),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (134, 0, -1, -7),
            (134, 0, -1, -10),
            (135, 0, -1, -15),
            (135, 0, -1, -17),
            (136, 0, -1, -11),
            (137, 0, -2, -12),
            (137, 0, -2, -10),
            (137, 0, -2, -8),
            (138, 0, -2, -3),
            (139, 0, -3, 8),
            (139, 0, -3, 8),
            (139, 0, -3, 18),
            (140, 0, -4, 28),
            (141, 0, -4, 44),
            (142, 0, -4, 71),
            (142, 0, -5, 82),
            (144, 0, -5, 82),
            (145, 0, -7, 83),
            (146, 0, -10, 86),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (134, 0, -1, -16),
            (134, 0, -1, -18),
            (135, 0, -2, -21),
            (135, 0, -2, -23),
            (136, 0, -2, -21),
            (136, 0, -2, -19),
            (137, 0, -3, -14),
            (137, 0, -4, -5),
            (138, 0, -4, 3),
            (139, 0, -4, 11),
            (140, 0, -5, 22),
            (140, 0, -6, 42),
            (141, 0, -7, 51),
            (142, 0, -9, 60),
            (144, 0, -9, 60),
            (145, 0, -11, 61),
            (146, 0, -14, 64),
        ],
    },
    Behavior {
        delay: 0,
        frames: &[
            (136, 0, 3, -22),
            (136, 0, 3, -23),
            (136, 0, 3, -25),
            (136, 0, 3, -26),
            (137, 0, 3, -26),
            (137, 0, 3, -25),
            (138, 0, 3, -17),
            (138, 0, 3, -15),
            (139, 0, 2, -10),
            (140, 0, 2, -4),
            (140, 0, 2, 2),
            (141, 0, 2, 11),
            (142, 0, 2, 28),
            (142, 0, 2, 43),
            (144, 0, 2, 43),
            (145, 0, 0, 44),
            (146, 0, -3, 47),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[(87, 0, 0, 0), (88, 0, 0, 0), (89, 0, 0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[(99, 0, 0, 0), (100, 0, 0, -1), (101, 0, 0, -1)],
    },
    Behavior {
        delay: 3,
        frames: &[(96, 1, 0, 0), (95, 1, 0, 2), (94, 1, 0, 2), (93, 1, 0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (117, 0, 0, 0),
            (118, 0, 0, 0),
            (119, 0, 0, 0),
            (120, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (108, 0, 0, 0),
            (109, 0, 0, 0),
            (110, 0, 0, 0),
            (111, 0, 0, 0),
            (112, 0, 0, 0),
            (113, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[
            (90, 0, 0, 0),
            (91, 0, 0, 0),
            (92, 0, 0, 0),
            (92, 0, 0, 0),
            (92, 0, 0, 0),
            (92, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (101, 0, 0, 0),
            (102, 0, 2, -13),
            (103, 0, 2, -16),
            (104, 0, 3, -16),
            (105, 0, 4, -10),
            (106, 0, 3, -7),
            (102, 0, 5, -2),
            (103, 0, 5, 2),
            (104, 0, 5, 7),
            (107, 0, 5, 10),
            (107, 0, 5, 10),
            (107, 0, 5, 10),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (96, 1, 0, 0),
            (97, 1, 0, 0),
            (98, 1, 0, 0),
            (97, 1, 0, 0),
            (98, 1, 0, 0),
            (97, 1, 0, 0),
            (98, 1, 0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (117, 0, 0, 0),
            (121, 0, 6, 0),
            (122, 0, 1, 0),
            (123, 0, 1, 0),
            (122, 0, 1, 0),
            (123, 0, 1, 0),
            (122, 0, 1, 0),
            (123, 0, 1, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[
            (108, 0, 0, 0),
            (114, 0, 1, -5),
            (115, 0, 6, -4),
            (116, 0, 6, -2),
            (116, 0, 6, -2),
            (116, 0, 6, -2),
            (116, 0, 6, -2),
        ],
    },
    Behavior {
        delay: 5,
        frames: &[(40, 0, 0, 0), (40, 0, 0, 0), (40, 0, 0, 0), (40, 0, 0, 0)],
    },
    Behavior {
        delay: 5,
        frames: &[(34, 0, 0, 0), (34, 0, 0, 0), (34, 0, 0, 0), (34, 0, 0, 0)],
    },
    Behavior {
        delay: 5,
        frames: &[(28, 0, 0, 0), (28, 0, 0, 0), (28, 0, 0, 0), (28, 0, 0, 0)],
    },
    Behavior {
        delay: 3,
        frames: &[
            (147, 0, 0, 0),
            (148, 0, -7, 0),
            (149, 0, -14, 0),
            (149, 0, -14, 0),
            (150, 0, -12, 0),
            (151, 0, -2, 0),
            (151, 0, -2, 0),
            (151, 0, -2, 0),
            (148, 0, -7, 0),
            (147, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 3,
        frames: &[
            (156, 0, 0, 0),
            (157, 0, 0, 0),
            (158, 0, 0, 0),
            (159, 0, 0, 0),
            (160, 0, 0, 0),
            (159, 0, 0, 0),
            (158, 0, 0, 0),
            (157, 0, 0, 0),
        ],
    },
    Behavior {
        delay: 4,
        frames: &[
            (161, 0, 0, -4),
            (162, 0, -1, -10),
            (163, 0, 0, -8),
            (164, 0, 1, -5),
            (164, 0, 1, -6),
        ],
    },
];

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

/// The big "snowball in the face" hit animation: behaviour 21, whose frames
/// use the large close-up sprites 152..155 (a kid peeking over the snow fort
/// getting snowballed). The shipped binary selects it purely by kid type -
/// type 8 always dies with it, wherever the kid is - which puts the close-up
/// out in the field; the reconstruction gates it on the castle instead.
pub const CLOSEUP_HIT_BEHAVIOR: usize = 21;

/// Kid y from which it counts as "at the castle". The snow fort occupies the
/// bottom of the arena and the centre spawn row is y = 169 (edge spawns stay
/// around y = 102..148), so the two zones do not overlap.
pub const CASTLE_Y: i32 = 165;

/// Cursor marker sprites: red (idle) and green (throw ready/unlimited).
pub const CURSOR_SPRITE_RED: u8 = 124;
pub const CURSOR_SPRITE_GREEN: u8 = 143;

/// Per-team overlay sprites. Function 0x14116 draws a scenery band from the
/// SFSHP record bank at (0, 40) when a team is selected (DS:0x39CE = 2,1,3,0),
/// and function 0x14160 draws the compass highlight at (272, 3)
/// (DS:0x39D2 = 6,5,7,4). Works on the overlay atlas, so switching teams
/// changes both the arena background and the compass wedge.
pub static OVERLAY_BACKGROUND: [u8; 4] = [2, 1, 3, 0];
pub static OVERLAY_COMPASS: [u8; 4] = [6, 5, 7, 4];
/// Draw positions recovered from 0x14116 / 0x14160.
pub const OVERLAY_BACKGROUND_POS: (i32, i32) = (0, 40);
pub const OVERLAY_COMPASS_POS: (i32, i32) = (272, 3);

/// In-game message banner, recovered from the round code (0x13C17 level clear
/// and 0x13C6D round end). Both call the rectangle filler 0x16387 with
/// (32,100)-(290,115) and colour [0x7614] (=9, light blue) to fill the banner,
/// then again with colour 0 to put a black edge on the bottom and sides, and
/// finally 0x1630B draws the message at (52,105) in [0x7619] (=14, yellow).
/// The banner is what keeps the light message text readable over the snow.
pub const BANNER_X0: i32 = 32;
pub const BANNER_X1: i32 = 290;
pub const BANNER_Y0: i32 = 100;
pub const BANNER_Y1: i32 = 115;
pub const BANNER_COLOR: u8 = 9;
pub const BANNER_EDGE: u8 = 0;
pub const BANNER_TEXT_POS: (i32, i32) = (52, 105);
pub const BANNER_TEXT_COLOR: u8 = 14;

/// Compass status positions for the four participants (x, y) - [0x399C].
pub static COMPASS_MARKERS: [(i32, i32); 4] = [(302, 20), (292, 28), (280, 20), (292, 13)];

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

/// Kid type selection: first threshold <= random byte selects the type.
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
/// Default snowball arc behavior (kid types 5/8); 6 -> 23, 7 -> 24.
pub const SNOWBALL_BEHAVIOR: usize = 22;
/// X nudge applied to the walk base when leaving behavior index `old` (0x41F6).
pub static BEHAVIOR_XOFF: [i8; 6] = [2, -4, 4, -6, 6, -8];

/// Roaming special object parameters.
pub const ROAM_RESPAWN_FRAMES: i32 = 400;
pub static ROAM_THRESHOLDS: [u8; 6] = [200, 160, 120, 80, 40, 0];
pub static ROAM_TYPES: [u8; 6] = [2, 3, 4, 10, 11, 12];
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
pub const ARENA_W: i32 = 300; // playfield width (cursor x clamp 0..299)
pub const ARENA_TOP: i32 = 53; // cursor y minimum (HUD occupies rows above)
pub const ARENA_BOTTOM: i32 = 199; // cursor/object y maximum
pub const OBJECT_X_MAX: i32 = 319; // object x clamp from [0x6ae1]
pub const PLAYER_START_X: i32 = 150; // initial cursor x ([0x190])
pub const PLAYER_START_Y: i32 = 150; // initial cursor y ([0x192])
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
pub const AMMO_REFILL_AMOUNT: i32 = 10; // snowballs granted when empty
pub const AMMO_REFILL_FRAMES: i32 = 50; // frames between free refills while empty
pub const PICKUP_SNOWBALLS: i32 = 100; // snowballs granted by a type-0 pickup hit
pub const PRACTICE_END_METER: i32 = 15; // practice meter high byte that ends a practice round
pub const METER_PER_SNOWBALL: i32 = 6; // [0x3fdc] added when a kid throws
pub const METER_PER_LEVEL: i32 = 768; // [0x3fdc] subtracted when a level is cleared
pub const ATTACK_DX: i32 = 9; // cursor hot point x offset (x + 9)
pub const ATTACK_DY: i32 = 8; // cursor hot point y offset (y - 8)
pub const SIM_HZ: i32 = 35; // simulation steps per second (approximation, see notes)

/// Pixel size of every SFFBM sprite (bank index 0..=164).
pub static SPRITE_SIZES: [(u8, u8); 165] = [
    (28, 28),
    (24, 29),
    (16, 29),
    (20, 31),
    (28, 28),
    (24, 29),
    (20, 30),
    (20, 21),
    (20, 21),
    (12, 21),
    (16, 22),
    (20, 21),
    (20, 21),
    (12, 22),
    (16, 14),
    (12, 14),
    (8, 14),
    (12, 15),
    (16, 15),
    (12, 14),
    (12, 15),
    (24, 31),
    (28, 30),
    (24, 29),
    (20, 31),
    (24, 33),
    (32, 27),
    (20, 29),
    (20, 29),
    (16, 22),
    (16, 22),
    (12, 20),
    (20, 23),
    (12, 19),
    (16, 22),
    (12, 16),
    (12, 16),
    (12, 16),
    (12, 17),
    (12, 15),
    (12, 15),
    (24, 32),
    (20, 22),
    (24, 31),
    (24, 27),
    (28, 14),
    (28, 15),
    (16, 20),
    (20, 20),
    (28, 21),
    (20, 13),
    (24, 11),
    (12, 18),
    (12, 10),
    (16, 7),
    (12, 11),
    (28, 30),
    (28, 26),
    (28, 22),
    (16, 21),
    (16, 18),
    (16, 15),
    (16, 15),
    (12, 14),
    (12, 10),
    (24, 16),
    (20, 18),
    (16, 21),
    (20, 20),
    (24, 19),
    (28, 15),
    (20, 22),
    (16, 11),
    (16, 10),
    (12, 14),
    (12, 13),
    (16, 13),
    (16, 10),
    (16, 15),
    (12, 10),
    (12, 7),
    (12, 9),
    (8, 10),
    (8, 10),
    (12, 9),
    (12, 8),
    (12, 11),
    (44, 18),
    (44, 18),
    (44, 17),
    (32, 17),
    (32, 19),
    (32, 16),
    (36, 20),
    (36, 23),
    (36, 25),
    (36, 24),
    (36, 26),
    (36, 26),
    (16, 16),
    (16, 15),
    (16, 16),
    (16, 16),
    (20, 17),
    (16, 15),
    (16, 17),
    (16, 13),
    (20, 9),
    (32, 16),
    (32, 16),
    (36, 15),
    (36, 15),
    (32, 15),
    (32, 15),
    (28, 14),
    (24, 12),
    (20, 15),
    (28, 35),
    (28, 36),
    (28, 34),
    (28, 35),
    (20, 38),
    (24, 39),
    (24, 40),
    (20, 15),
    (28, 12),
    (32, 15),
    (4, 5),
    (4, 5),
    (4, 5),
    (4, 5),
    (4, 5),
    (4, 5),
    (20, 14),
    (4, 2),
    (4, 3),
    (4, 4),
    (8, 5),
    (8, 6),
    (8, 7),
    (12, 9),
    (12, 10),
    (12, 11),
    (20, 15),
    (12, 10),
    (20, 13),
    (24, 19),
    (48, 18),
    (52, 25),
    (60, 44),
    (60, 46),
    (52, 47),
    (60, 49),
    (60, 48),
    (56, 51),
    (56, 51),
    (28, 27),
    (24, 28),
    (16, 28),
    (20, 28),
    (28, 27),
    (24, 28),
    (28, 26),
    (28, 21),
    (28, 18),
];

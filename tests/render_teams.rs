//! Visual regression tests for the per-team overlays (see assets.md 3.3).
//!
//! The pixel checks need the generated asset pack (`assets/snowball.pack`).
//! When it is absent (fresh clone / CI) only the data-table checks run.

use snowball_war::assets::Assets;
use snowball_war::data::{
    COMPASS_MARKERS, OVERLAY_BACKGROUND, OVERLAY_BACKGROUND_POS, OVERLAY_COMPASS,
    OVERLAY_COMPASS_POS,
};
use snowball_war::game::Game;
use snowball_war::render::{self, Framebuffer, H, W};

/// Each of the four directions selects its own SFSHP record for both the
/// scenery band (DS:0x39CE) and the compass (DS:0x39D2), recovered from
/// functions 0x14116 / 0x14160.
#[test]
fn per_team_overlay_tables_are_distinct_and_in_range() {
    assert_eq!(OVERLAY_BACKGROUND, [2, 1, 3, 0]);
    assert_eq!(OVERLAY_COMPASS, [6, 5, 7, 4]);
    for team in 0..4 {
        assert!(OVERLAY_BACKGROUND[team] < 4, "band record is a 0-3 strip");
        assert!(
            (4..8).contains(&OVERLAY_COMPASS[team]),
            "compass is a 4-7 record"
        );
    }
    let mut bands = OVERLAY_BACKGROUND.to_vec();
    bands.sort_unstable();
    bands.dedup();
    assert_eq!(bands.len(), 4, "every team has its own band");
    let mut compasses = OVERLAY_COMPASS.to_vec();
    compasses.sort_unstable();
    compasses.dedup();
    assert_eq!(compasses.len(), 4, "every team has its own compass wedge");
    assert_eq!(OVERLAY_COMPASS_POS, (272, 3));
    assert_eq!(OVERLAY_BACKGROUND_POS, (0, 40));
}

fn render_team(assets: &Assets, team: usize) -> Framebuffer {
    let mut game = Game::new(0x5EED);
    game.start(false);
    game.selected = team;
    let mut fb = Framebuffer::new();
    render::draw(&mut fb, &game, assets);
    fb
}

fn region_differs(
    a: &Framebuffer,
    b: &Framebuffer,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
) -> usize {
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = (y * W + x) * 4;
            if a.px[p..p + 3] != b.px[p..p + 3] {
                n += 1;
            }
        }
    }
    n
}

/// Green wedge pixels of the rendered compass, as (count, centroid).
fn wedge_pixels(fb: &Framebuffer) -> (usize, f64, f64) {
    let (cx, cy) = OVERLAY_COMPASS_POS;
    let (mut n, mut sx, mut sy) = (0usize, 0f64, 0f64);
    for y in cy as usize..cy as usize + 34 {
        for x in cx as usize..cx as usize + 48 {
            let p = (y * W + x) * 4;
            // palette index 10 = (84, 252, 84)
            if fb.px[p] == 84 && fb.px[p + 1] == 252 && fb.px[p + 2] == 84 {
                n += 1;
                sx += x as f64;
                sy += y as f64;
            }
        }
    }
    if n == 0 {
        (0, 0.0, 0.0)
    } else {
        (n, sx / n as f64, sy / n as f64)
    }
}

#[test]
fn switching_teams_redraws_background_and_compass() {
    let Ok(assets) = Assets::load_default() else {
        eprintln!("skipping: assets/snowball.pack not generated");
        return;
    };
    let team3 = render_team(&assets, 3);
    let team0 = render_team(&assets, 0);

    // The scenery band spans (0,40)..(320,98) behind the playfield.
    let band = region_differs(&team3, &team0, 0, 40, W, 98);
    assert!(
        band > 500,
        "background band should follow the team, differs in {band} px"
    );

    // The compass covers record 4..7 at (272,3), 48x34.
    let wedge = region_differs(&team3, &team0, 272, 3, 320, 37);
    assert!(
        wedge > 100,
        "compass should follow the team, differs in {wedge} px"
    );

    // Playfield pixels below the band stay identical (same simulation seed).
    assert_eq!(region_differs(&team3, &team0, 0, 100, W, H), 0);
}

/// The compass must show exactly one green wedge per team, pointing at the
/// sector where that team's count marker sits (COMPASS_MARKERS). Before the
/// SFSHP orientation fix and the replacement blit, switching teams left the
/// arena's wedge behind and drew a second (mirrored) one.
#[test]
fn each_team_shows_one_wedge_in_its_own_sector() {
    let Ok(assets) = Assets::load_default() else {
        eprintln!("skipping: assets/snowball.pack not generated");
        return;
    };
    let (cx, cy) = OVERLAY_COMPASS_POS;
    let centre = (cx + 24, cy + 17);
    for (team, &(mx, my)) in COMPASS_MARKERS.iter().enumerate() {
        let fb = render_team(&assets, team);
        let (n, wx, wy) = wedge_pixels(&fb);
        assert!(n > 80, "team {team}: expected a green wedge, found {n} px");

        // Compare the wedge direction against every marker: the team's own
        // marker must be the closest one (that is how the original pairs the
        // wedge with the little count glyphs around the compass).
        let wedge = (wx - centre.0 as f64, wy - centre.1 as f64);
        let norm = (wedge.0 * wedge.0 + wedge.1 * wedge.1).sqrt();
        let best = COMPASS_MARKERS
            .iter()
            .enumerate()
            .map(|(i, &(ox, oy))| {
                let m = (ox - centre.0, oy - centre.1);
                let dot = (wedge.0 * m.0 as f64 + wedge.1 * m.1 as f64)
                    / (norm * ((m.0 * m.0 + m.1 * m.1) as f64).sqrt());
                (dot, i)
            })
            .fold((f64::MIN, usize::MAX), |a, b| if b.0 > a.0 { b } else { a });
        assert_eq!(
            best.1, team,
            "team {team} wedge points at marker {} (marker at ({mx},{my}))",
            best.1
        );
    }
}

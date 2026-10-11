# Snowball War — a REA-driven reconstruction of a Ski or Die minigame

An independent, playable reconstruction of the **Snowball War** ("snowball
blast") minigame from the DOS version of *Ski or Die* (Epic MegaGames),
written in Rust with SDL2. The project exists to demonstrate **REA**
(Reverse Engineer Anything) as an AI-assisted reverse-engineering workflow:
analyze the shipped binary, recover the gameplay rules with evidence, write
them down, then rebuild the minigame from that specification.

Only Snowball War is investigated and rebuilt — not the skiing,
snowboarding, skateboarding or other events, and not the original engine.

## Quick start — download the game, prepare the assets, play

1. **Download the original DOS release** you own (the copy used for analysis
   matches this listing) and **extract the archive into `Ski_Or_Die/`**:

   <https://www.abandonwaredos.com/abandonware-game.php?gid=1074>

   ```text
   Ski_Or_Die/           <- unzip the release here (any sub-directory layout works)
   ```

2. **Build the runtime assets** from that directory (Python 3, standard
   library only — nothing from the game is committed):

   ```bash
   python3 tools/extract/build_assets.py Ski_Or_Die assets
   ```

3. **Build and play:**

   ```bash
   cargo run --release
   ```

Prerequisites: Rust stable with Cargo, the SDL2 development library
(`libsdl2-dev`, pkg-config name `sdl2`) and Python 3 — see
[Build](#build) for details. The extractor discovers the files itself, undoes
the EXEPACK compression of the game executable, decodes the LZSS containers
and image formats, recovers the 8x8 HUD font from inside the executable and
writes `assets/snowball.pack` (SHA-256 printed; the release used here gives
`d78508a3…15ad`, 1 079 163 bytes) plus byte-exact audio data in
`assets/audio/`. Add `--preview` for PNG/PPM preview images.

## Why Snowball War

* Self-contained: one event module inside a shared executable, with its own
  entry point, frame loop, HUD and round rules.
* Rich but tractable mechanics: 8-way accelerating movement, scripted
  targets, ballistic snowball arcs, hit resolution, ammo/score economy,
  levels, timers and a practice mode.
* The binary contains readable event strings (`snowfight competition`,
  `snowballs   kids`, `bonus for clearing level`), which anchor the analysis.

## The original executable (reference only — not committed)

| Item | Value |
|---|---|
| Directory | `Ski_Or_Die/` (kept in this working copy, listed in `.gitignore`) |
| Analyzed file | `SKIEGA` — the 16-colour EGA build, plain `MS-DOS executable, MZ` |
| SHA-256 | `6638bd4d4e6e43136d1fcf69b800baa4ad5a1c7850b2f0eed4607d46fdcead70` |
| Compression | EXEPACK (stub at `CS:0x0CA6:0x0012`, message `!Packed file is corrupt`) |
| Unpacked image | 0xF7C0 bytes, SHA-256 `79ee524c51ebf424c8f94acbd48396159b77c4b87fed637b710f8a3a6f3e87d3` |

The executable is compressed with EXEPACK, so a faithful unpack step was
required before analysis; the shipped file is never modified. The original
game, its data files, its art and its audio stay out of the repository.

## How REA was used

1. `rea doctor` showed that DOS MZ targets need REA's documented Ghidra
   path (the "Ghidra 16-bit real-mode adapter"); Hopper, the other
   configured provider, rejects DOS formats.
2. Ghidra **12.1.4** + JDK **21** were installed and verified with
   `rea doctor` (`ghidra-version`, `ghidra-headless`, `ghidra-java` all
   healthy).
3. The EXEPACK stub was replayed to produce an unpacked load image, which
   was wrapped in a minimal MZ header and verified by REA:
   `rea inspect-native-load-image --provider ghidra` → `status: verified`,
   entry `0x100C5`, language `x86:LE:16:Real Mode`, load segment `0x1000`.
4. Function discovery used REA's caller-declared seed mechanism
   (`REA_GHIDRA_SEED_FILE`) because event modules are only reached by
   indirect dispatch: **559 seeded functions created, 4494 code seeds
   decoded, 0 failures** (reported by REA; flagged as caller assertions).
5. Behavior was recovered with `rea trace`, `rea function` and
   `rea search` over the Snowball War module (entry `0x3B0E`, frame loop
   `0x3B53`, HUD `0x3DF0`, movement `0x3FDF`, spawner `0x427D`, script
   engine `0x4338`, snowball creation `0x448F`, throw/hit `0x47C1`/`0x48B0`,
   time tick `0x4A49`, object step `0x64EC`, input `0x6C78`/`0x6D0C`).

Because this session's MCP server was started before `GHIDRA_INSTALL_DIR`
existed, the analysis calls ran through the **REA CLI** (same package and
providers); the opencode config now carries the Ghidra environment for
future sessions. Full detail, commands and evidence excerpts:
[`docs/reverse-engineering-notes.md`](docs/reverse-engineering-notes.md).

## Original assets (extracted and used)

Graphics and audio were extracted from the shipped data files with
`tools/extract/build_assets.py` (LZSS containers + the three recovered image
formats) and are used by the reconstruction:

| Asset | Source | Notes |
|---|---|---|
| `assets/snowball.pack` | generated by the extractor | everything the Rust game loads at runtime |
| `assets/audio/*` | tunes + `*.SMB` banks | byte-exact audio data (playback not implemented yet) |
| `assets/preview/arena.png` (`--preview`) | `SNOWFORT.LZW` | 320x200 Snowball War arena |
| `assets/preview/sprites.png` (`--preview`) | `SFFBM.LZW` | 165 sprites: kids, snowballs, cursor, specials |
| `assets/preview/overlay.png` (`--preview`) | `SFSHP.LZW` | sky strips/compass variants (not referenced by the snowball code) |
| font (inside the pack) | executable DS:0x6728 | the game's 8x8 HUD font |
| `assets/snowball.pack` | generated by the extractor | dependency-free binary the Rust game loads |
| `assets/audio/*` | tunes + `*.SMB` banks | byte-exact audio data (playback not implemented yet) |

Notable findings (all in `docs/assets.md`): the sprite records are stored
**mirrored horizontally** with a **per-frame mirror byte** that the blitter
applies (`0x177E7`, flag from `[SI+0xE]`), colour index 0 is transparent, and
the earlier "obfuscated" look of third-party tool output came from an extra
vertical flip in its BMP writer. The HUD font lives in the executable, so HUD
text is pixel-identical. The startup copy protection is a code-wheel question
(`CPSHP.LZW` + "please enter number in hole"), not asset encryption.

```bash
cargo run --example frame -- 300 frame.ppm   # headless render for comparison
```

## What was recovered

**Confirmed** (code-backed; see the behavior spec for addresses):

* Arena: 300×200 playfield, player clamped to `x∈[0,299]`, `y∈[53,199]`,
  start (150,150); objects clamp at `x≤319`, `y≤199`.
* Controls: 8-way numpad/arrows move the throwing **cursor**, **Enter** =
  throw/start, **Right Shift + cardinal key** switches the selected kid team
  (0…3, shown as the four compass counts).
* Movement acceleration: 2 → 4 → 6 px/frame (+40 per frame, cap 2, instant
  reset on release).
* Kids: 4 spawn scripts per slot taken from the original tables, spawn every
  90 frames per participant from a shared credit budget, four types with
  26/25/30/20 % odds worth 100/75/50/25 points, scripted walk/throw/death
  behaviors, despawn thresholds 305/299/291.
* Snowballs: created by kids on throw-animation frame 5 (3-slot pool per
  participant), **scripted ballistic arcs** (rise then fall, 19/17/13
  frames), expiry at script end, +6 meter per throw.
* Player throw: 5-frame cadence, −1 snowball (free while `unlimited`),
  box hit test at `(x+9, y−8)` against queued objects, per-type score and
  death effects.
* Rounds: 50+kids target, `+25` bonus per clear, `score += 1000 +
  snowballs` on clear, 60-unit timer ticking every 41 frames, time-out
  converts remaining snowballs to score, practice mode ends at meter 0x0F.
* Specials: roaming object every 400 frames (6 types), pickups every 600
  frames (snowball +100 / meter −256 / `unlimited` 400 frames), free
  +10 ammo every 50 frames while empty.

**Uncertain / approximated**

* The `[0x75e3]` gate is implemented exactly: only the selected team's kids
  are visible/hittable, and the player switches teams to reach the level
  target (documented in the spec, §9).
* Hit boxes use the real extracted sprite dimensions; the exact frame rate
  (35 Hz chosen), the PRNG algorithm (seeded xorshift32), pause/exit keys
  (P/Esc), the fifth kid slot (disabled in data → 4 used) and the audio
  driver (data extracted, playback not implemented) remain approximations or
  open items.

## Build

Prerequisites (Linux):

* Rust stable (tested with 1.93) and Cargo
* SDL2 development library — `libsdl2-dev` (pkg-config name `sdl2`)

```bash
cargo build --release
cargo test                 # 17 gameplay tests
cargo fmt --check
cargo clippy --all-targets --all-features
```

## Run

```bash
cargo run --release
```

A 960×600 window opens (320×200 logical, integer-scaled).

## Controls

| Key | Action |
|---|---|
| Arrow keys / keypad (8-way) | Move the throwing cursor (accelerates while held) |
| Enter (or Space) | Start · throw a snowball at the cursor (repeat while held) |
| Right Shift + ←/→/↑/↓ | Switch kid team (0…3); the compass shows each team's count |
| M | Toggle competition / practice (menu) |
| P | Pause |
| Esc | End the round |
| Window close | Quit |

## Repository layout

```
src/assets.rs       loads assets/snowball.pack (arena, sprites, font)
src/game.rs         simulation: states, frames, throw/hit, round rules
src/participant.rs  participant teams, kid scripts, spawn logic
src/snowball.rs     snowball lifecycle + scripted arc step
src/roam.rs         roaming special object
src/arena.rs        arena geometry and hit boxes
src/input.rs        recovered input semantics
src/render.rs       software rasteriser: original arena/sprites/font
src/main.rs         window, key events, fixed 35 Hz timestep
src/data.rs         gameplay tables recovered from the binary (+ provenance)
examples/frame.rs   headless frame renderer (comparison with the original)
tests/gameplay.rs   17 automated gameplay tests
tools/extract/      standalone asset extractor (run before the game)
docs/               reverse-engineering notes, behavior spec, asset notes
```

## Current limitations

* No sound/music playback yet: the tunes and the per-card sound banks are
  extracted into `assets/audio/`, but the original player is a driver inside
  the executable (init 0x6105, entry at `[0x6A20]`, tune loader 0x1814D) and
  reproducing it needs an FM (OPL2) core or a driver port — see
  `docs/assets.md` §4.
* No side-by-side runtime comparison: no DOS emulator is available in this
  environment, so validation is static (REA verification + extracted tables).
  If DOSBox becomes available, the scenarios in the notes are the natural
  first comparisons (boundary clamp, point-blank throw, level clear).
* Menu/transition screens of the surrounding game are out of scope; the
  reconstruction has its own menu and a round-over screen.
* Graphics and font are original placeholder art; no proprietary assets.

## License

MIT for the reconstruction. The original game remains copyrighted by its
owners; nothing proprietary is included in this repository.

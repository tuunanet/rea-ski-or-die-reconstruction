# Reverse-engineering notes — Snowball War (Ski or Die, DOS)

This document records how the original binary was investigated with REA,
which findings are supported by evidence, and which questions remain open.
Addresses are **linear offsets in the unpacked load image** unless stated
otherwise; REA/Ghidra reports them with a load segment of `0x1000`, so a
load-image offset `X` appears as address `0x10000 + X`.

## 1. Tooling and environment

| Item | Value |
|---|---|
| REA | `rea-agents` 6.4.0 (MCP server `io.github.morluto/rea` + CLI `rea`) |
| Deep provider | Ghidra 12.1.4 (installed to `~/tools/ghidra_12.1.4_PUBLIC`) |
| JDK | Temurin 21.0.12.1 (installed to `~/tools/jdk-21`) |
| Other providers | Hopper 5 (`/opt/hopper`) — rejects DOS targets; IDA — not configured |

`rea doctor` initially reported the environment unhealthy because the only
configured deep provider (Hopper) answered:

> `DOS MZ/COM analysis requires the Ghidra 16-bit real-mode adapter.`

REA's documented path for this format is the bring-your-own-Ghidra adapter
(`docs/ghidra-dos.md` upstream: `MzLoader`, language `x86:LE:16:Real Mode`,
fixed load segment `0x1000`, verified against Ghidra 12.1.4 + JDK 21).
After installing Ghidra/JDK and setting `GHIDRA_INSTALL_DIR` + `JAVA_HOME`,
`rea doctor` reported every Ghidra check healthy (`ghidra-version 12.1.4`,
`ghidra-headless ok`, `ghidra-java 21.0.12.1`).

The opencode MCP server for this session had been launched before those
variables existed in `~/.config/opencode/opencode.jsonc`, so every analysis
call was made through the **REA CLI** (same package, same providers) with the
environment set explicitly, e.g.:

```bash
export GHIDRA_INSTALL_DIR=~/tools/ghidra_12.1.4_PUBLIC
export JAVA_HOME=~/tools/jdk-21
export REA_GHIDRA_SEED_FILE=/tmp/opencode/seeds.tsv
rea inspect  /tmp/opencode/unpack/skiega.unpacked.mz --provider ghidra --json
rea function /tmp/opencode/unpack/skiega.unpacked.mz 0x13df0 --provider ghidra --json
rea trace    /tmp/opencode/unpack/skiega.unpacked.mz --query "snowfight" --provider ghidra --json
```

The config file was updated so that future sessions expose Ghidra to the MCP
server as well (restart opencode to activate).

## 2. Identifying the target executable

Directory `Ski_Or_Die/` (kept out of the repository) contains:

| File | Role | Evidence |
|---|---|---|
| `S.EXE` | video-adapter launcher (MS C 1988 runtime) | strings `skicga/skiega/skitga`, `Cannot find that video card` |
| `SKICGA`, `SKIEGA`, `SKITGA` | the game executables (CGA/EGA/Tandy) | MZ headers, strings of all `.LZW` data files |
| `SKIORDIE.COM` | crack intro loader (The Humble Guys) | self-decrypting `xor al,0x45` loop at file offset 0x0F; decrypted strings `spboat3.dat`, crack group text — irrelevant to gameplay |
| `*.LZW` | compressed game data (e.g. `SNOWFORT.LZW`, `SFFBM.LZW`, `SFSHP.LZW`) | loaded by the executable (file names inside the binary) |
| `SKI`, `SKI2.BIN`, `HISCORES` | data files | referenced by the executable |

Chosen analysis target: **`SKIEGA`** (the 16-colour EGA build)
SHA-256 `6638bd4d4e6e43136d1fcf69b800baa4ad5a1c7850b2f0eed4607d46fdcead70`.

### 2.1 The executable is EXEPACK-compressed

`file` reports a plain `MS-DOS executable, MZ`; `e_cparhdr=0x20`,
`e_minalloc=0x31BD` (≈201 KB of extra memory) and the entry point
`0x0CA6:0x0012` sits in the last 167 bytes of the image — the classic
EXEPACK stub layout. Disassembling the stub (capstone, 16-bit mode)
showed the documented EXEPACK algorithm:

```
0x0CA72  mov ax,es / add ax,0x10 / push cs / pop ds / mov [4],ax
         add ax,[0xc] / mov es,ax / mov cx,[6] / std / rep movsb   ; copy stub to scratch
0x0CA92  mov dx,[0xe] / push ax / mov ax,0x38 / push ax / retf     ; jump into the copy
         ... backward stream: control byte, count word, payload
         0xB0 = REP STOSB fill, 0xB2 = REP MOVSB copy, bit0 = last record
         ... relocation table at stub+0x12D, then ljmp [0]:[2]
```

Because REA states that *"packed targets require a separately identified
unpacked artifact for original-program analysis"*, the stream was replayed
by a small Python program (`unexepack.py`, session scratch space) that:

1. reads the MZ header and the parameter block at `CS:0000`
   (`[6]=0x199` stub length, `[0xc]=0x0F7C` scratch/unpacked size);
2. starts the control stream at `0x0CA5F` and decodes backward
   (`B0/B2`, final record with bit 0 set);
3. copies the stored prefix through unchanged (the packer stops writing at
   the same offset where the stream ends);
4. rebuilds the relocation table (38 entries) from the stub.

Result: a 0xF7C0-byte load image, SHA-256
`79ee524c51ebf424c8f94acbd48396159b77c4b87fed637b710f8a3a6f3e87d3`.
Cross-checks that make the reconstruction trustworthy:

* all game strings become readable (`snowfight competition`, `snowfort.lzw`, …);
* the entry at `0x00C5` disassembles gaplessly (1437 instructions without a
  decode failure) and its immediate `0x0819` matches relocation entry #198;
* CGA/Tandy builds unpack with the same code and are ≈60 % byte-identical
  to the EGA build (shared game code, different video code).

The image was then wrapped in a minimal, self-consistent MZ header
(512-byte header, 38 relocation entries, `CS:IP = 0x000C:0x0005` → `0x00C5`,
`SS:SP = 0x0F7C:0x0200`, `minalloc=0x1000`). The wrapper is a **derived
analysis artifact** (SHA-256 `432c311e78b8bcc22e238b7ef0b2fbfaab531a53a4d1f0d0298708f0a6ebbf7b`);
the shipped file was never modified.

REA verification of the artifact (`rea inspect-native-load-image`):

```
status: verified   format: dos-mz   header_bytes: 512   module_bytes: 63424
entry: 0x100c5 (relative_segment 12, offset 5)
language_id: x86:LE:16:Real Mode   loader: MzLoader   load_segment: 0x1000
```

### 2.2 Function discovery needed caller-declared seeds

With the default Ghidra analyzers only 90 procedures were discovered: the
event modules are entered through indirect dispatch, so plain auto-analysis
never reaches them (the gap `0x10E97 … 0x15D2A` contained no functions).
REA supports exactly this case through `REA_GHIDRA_SEED_FILE`
(`address<TAB>function|code|label` lines applied before auto-analysis).

A recursive 8086 traversal (`harvest.py`) harvested direct `call`/far-`jmp`
targets from the entry point plus two extra roots (the first code after the
startup region and the Snowball War entry), and the device input functions
that are only reachable through jump tables. Seed outcome reported by REA:

```
559 functions created, 1 already present, 0 failed;
4494 code seeds decoded, 0 failed; 0 seeds outside initialized memory
"Seeded functions, code and names are caller assertions, not Ghidra discoveries."
```

## 3. Locating Snowball War

Strings unique to the mode (load-image offsets):

| Offset | String |
|---|---|
| `0xBA3E` | `skiing to snowball blast` (transition screen) |
| `0xBA57` | `next snowfighter` |
| `0xBA68` | `snowfight competition` (event title / score screen) |
| `0xBA8B` | `snowballs   kids` (HUD labels) |
| `0xBA9C` | ` bonus for clearing level` |
| `0xBAB6` | `you've been snowed, frosty!` |
| `0xBAD2` | `you're outta time, slick!` |
| `0xBAEC` | `unlimited` (powerup banner) |
| `0x8469` | `snowfort.lzw` (arena data) |

The data segment starts at `0x8190` (`mov ax,0x819` at entry), so DS-relative
offsets were converted (`0xBA68 → DS:0x38D8`) and searched in code: each
string has exactly one reference, all inside one module:

```
next snowfighter        DS:0x38C7  -> 0x3D03
snowfight competition   DS:0x38D8  -> 0x3D15
bonus for clearing lvl  DS:0x390C  -> 0x3C29
skiing to snowball blast DS:0x38AE -> 0x3D4B
you're outta time       DS:0x3942  -> 0x3C8E
you've been snowed      DS:0x3926  -> 0x3C98
```

Module boundary: entry `0x3B0E`, frame loop `0x3B50/0x3B53`, exit at
`0x3D28`, HUD/level init `0x3DF0`, shared code up to `0x4D00`
(next module starts afterwards).

### 3.1 Recovered function map (linear offsets)

| Address | Role |
|---|---|
| `0x000C5` | program entry (`mov ax,0x819 / mov ds,ax`, then calls) |
| `0x3B0E` | Snowball War entry: counter init, level init, wait for Enter |
| `0x3B50` | level (re)start; `0x3B53` frame loop head |
| `0x3D29` | module init (key bindings, slot resets, arena width 319) |
| `0x3DF0` | level init + HUD labels (`score`, `snowballs   kids`, `level`, `time`) |
| `0x3FDF` | player movement, acceleration, arena clamps, player draw |
| `0x4099` | practice meter check (ends practice at 0x0F00) |
| `0x40EE` | participant switch input (RShift + cardinal key) |
| `0x4160` | message cycler (every 30 frames) |
| `0x41C9` | reset roaming object/special after a switch |
| `0x4245` | frame hit-queue logger (gated by `[0x75e3]`) |
| `0x427D` | per-participant kid spawner (90-frame timer, credit budget) |
| `0x4338` | kid script processor: bounds, log, throw, script advance |
| `0x4413`/`0x4455`/`0x445F` | script/behavior (re)initialisation |
| `0x448F` | **snowball creation** (kid throw, frame 5 of behavior 6/7/8) |
| `0x4501` | per-participant snowball pool update/expiry |
| `0x4573` | roaming special object (400-frame respawn, walk, queue) |
| `0x462F` | periodic specials (600 frames; types 0/1/9) |
| `0x463F` | forced special spawn (out-of-ammo pickup) |
| `0x46BB` | `score += AX` + redraw |
| `0x470B` | `snowballs += AX` (clamped at 0) + redraw |
| `0x4763` | `kids -= AX` + redraw |
| `0x47C1` | **throw processing**: cooldown, ammo, queue hit test |
| `0x48B0` | **hit resolution** by type (score / powerups / death behavior) |
| `0x4A49` | competition time tick (−1 every 41 frames) |
| `0x64EC` | shared object step: scripted per-frame (dx, dy), frame delay |
| `0x6E67` | draw/box update: sprite size → collision box, draw list queue |
| `0x6C78`/`0x6D0C` | keyboard state (Enter / RShift) and direction (keypad) |
| `0x6A01`/`0x6A0B` | per-device direction/state jump tables (`DS:0x6C53`/`0x6C4B`) |
| `0x60E4` | frame pacing (`while [0x6a26] < [0x6a29]`) |
| `0x71A7` | draw-list y-sort (painter's order) |
| `0x5D2A` | pseudo-random byte |

## 4. Mechanics recovered (with evidence)

### 4.1 Arena and participants — CONFIRMED

* Player position is the shared word pair `[0x190]/[0x192]` (x/y), initialised
  to `(150,150)` by `0x3DF0`, clamped to `x∈[0,299]`, `y∈[53,199]` in `0x3FDF`
  (`cmp [si],0x12f / jbe`, `cmp [si+2],0x35 / ncmp 0xc7`).
  The 53-pixel floor is the HUD band.
* Objects clamp `x∈[0,319]` (`[0x6AE1]=0x013F`, written by `0x3DE9`) and
  `y ≤ 199` (`0x64EC`).
* Four participant groups (indices 0…3) exist; each has:
  * five kid slots — table `DS:0x3B04 → DS:0x3B0C` (20 slot pointers,
    stride 31 bytes, blocks like `0x3FFE`);
  * three snowball slots — table `DS:0x3A88 → DS:0x3A90` (12 pointers);
  * a spawn timer `[0x3985+idx]`, a state counter `[0x3981+idx]`
    (= active kids), a snowball counter `[0x3A84+idx]`.
* The fifth kid slot's script cursor is `0xFE` in the shipped data, i.e. it
  is disabled; the reconstruction uses four active slots.
* Group 3 is the default controlled group (`[0x39CD]=3`, written by `0x3EA8`
  and re-applied every level by `0x4116`).

### 4.2 Controls — CONFIRMED

* Directions: keyboard function `0x6D0C` scans key states
  `[0x6B08 + scan]` for scans `0x47,0x48,0x49,0x4B,0x4D,0x4F,0x50,0x51`
  (keypad/arrows) and returns numpad codes
  `7,8,1,6,2,5,4,3` (0 = none). Priority order is the scan order above.
* State: function `0x6C78` returns 1 for `[0x6C91]=0x1C` (**Enter**),
  2 for `[0x6C92]=0x36` (**Right Shift**), 3 for `[0x6C93]=0xFF` (unbound).
  Bindings are installed by the module init `0x3D80`.
* Throw/start: device state 1 (Enter) — `0x47C1`.
* Participant switch: state 2 (RShift) + even direction `2/4/6/8` selects
  participant `(dir/2)-1` = `0/1/2/3` — `0x40EE`.
* Movement is continuous (per frame, table driven), aiming is not
  independent: the throw uses a fixed point relative to the player.
* No jump/duck action exists in this module.
* Pause/exit flags exist (`[0x6C07]=1` exit, `=0x19` pause, handled by
  `0x6821`/entry wait) but the exact keys that set them were not recovered.

### 4.3 Movement — CONFIRMED

Function `0x3FDF`:

```
dir = device_direction()
if dir == 0: [0x3FDE] = 0                     // release resets acceleration
level = min([0x3FDF], 2)                      // high byte of the accel word
x += MOVE_DX[level][dir]; y += MOVE_DY[level][dir]
[0x3FDE] += 40                                // one word: accel + speed level
if [0x3FDF] > 2: [0x3FDF] = 2                 // clamp (0x14049)
clamp x to [0,299], y to [53,199]
```

Speed tables (words at `DS:0x16/0x28`, `0xB2/0xC4`, `0xD6/0xE8`):

| level | dx | dy |
|---|---|---|
| 0 | `0,2,2,2,0,-2,-2,-2,0` | `0,-2,0,2,2,2,0,-2,-2` |
| 1 | `0,4,4,4,0,-4,-4,-4,0` | `0,-4,0,4,4,4,0,-4,-4` |
| 2 | `0,6,6,6,0,-6,-6,-6,0` | `0,-6,0,6,6,6,0,-6,-6` |

Because the level lives in the high byte of a word that grows by 40 per
moving frame, the player accelerates from 2 px to 4 px after ≈7 frames and
to the cap of 6 px after ≈13 frames; releasing the key resets instantly.

### 4.4 Snowball mechanics — CONFIRMED (highest priority)

* **Creation** (`0x448F`, called from `0x4338`): when a kid of the current
  participant is in a throw behavior (script state `6…8`) **and** its
  animation frame counter equals **5** **and** the frame delay was just
  reloaded, a snowball is taken from the participant's pool (3 slots,
  `[0x3A84+idx]` counter), initialised at the kid's position, and given an
  arc behavior chosen by kid type: type 7 → behavior 24, type 6 →
  behavior 23, otherwise behavior 22. The meter `[0x3FDC] += 6`.
* **Motion** (`0x64EC`): every frame the position is recomputed as
  `spawn_base + (dx[frame], dy[frame])` from the behavior table, clamped to
  the arena; the frame advances when a per-frame delay counter underflows
  (delay = `behavior[0x0B] − [0x6AE3]`; `[0x6AE3]=0` in this module, so
  delay 0 = every frame, delay 3 = every 4th frame).
  The three snowball arcs (behaviors 22/23/24, 19/17/13 frames) rise
  (`dy` from −7 to −17) and then fall (up to `dy=+18`) while drifting
  sideways — **the trajectory is a scripted ballistic arc, not physics**;
  there is no gravity/wind integration and no bouncing.
* **Expiry** (`0x4501`): when the arc script completes
  (`[slot+0x16]=1`), the snowball is removed, the participant's counter is
  decremented, a sound cue plays for the controlled group, and an 8/256
  8/256 random side effect fires (a short CRTC scroll jitter, see 0x141D7 —
  this routine is a screen effect, not audio). Snowballs are never removed by
  arena bounds.
* **The player's throw** (`0x47C1`): device state 1 →
  sound, debounce `[0x397D]=[0x397E]` (cooldown 4 ⇒ one throw per 5 frames),
  `[0x397E] != 0` ⇒ `snowballs -= 1` (free while unlimited), then every
  queued object of the current frame whose box contains the player point
  `(x+9, y−8)` is hit-resolved (`0x48B0`). Objects already marked `0xFE`
  are skipped.
* **Box test** (`0x14879`): `obj.x ≤ px+9 ≤ obj.x + obj.w` and
  `obj.y − obj.h ≤ py−8 ≤ obj.y` — y is the object's baseline, the box is
  derived from the sprite descriptor in `0x6E67` (segment pointers resolved
  at runtime; widths/heights are **not statically recoverable**, see §6).
* **Hit resolution** (`0x48B0`) by type:

| type | effect |
|---|---|
| 0 | `snowballs += 100` (pickup), special deactivated |
| 1 | `[0x3FDC] -= 0x100` (meter) |
| 2 | `score += 50`, roaming object plays death behavior 31 |
| 3 | `score += 30`, death behavior 30 |
| 4 | `snowballs −= 25`, death behavior 32 |
| 5 / 6 / 7 / 8 | `score += 100 / 75 / 50 / 25`, kid marked `0xFE` and given death behavior 18/19/20/21 |
| 9 | unlimited snowballs: `[0x397E]=0`, `[0x3998]=400` frames, banner `unlimited` |
| 10 / 11 / 12 | `score += 30`, death behaviors 33/34/40 |

* **Damage/elimination**: kids have no health — one hit starts the death
  behavior; when that script finishes (`0x4338`, type `0xFE` path) the
  `kids` counter is decremented (`[0x398C] -= 1` via `0x4763`) and the slot
  is freed **without** refunding the spawn credit. Unhit despawns
  (walked out of the arena, or script end `0xFF`) **do** refund the credit
  (`[0x3994] += 1`). The player cannot be damaged: no code path writes a
  player-damage state.

### 4.5 Kids (targets) and their scripts — CONFIRMED

* Spawn cadence (`0x427D`): per participant a byte timer decrements every
  frame; on the 1→0 transition, if the shared credit budget `[0x3994] > 0`
  and a slot is free, the timer reloads to **90**, the budget is spent, and
  the next script is taken from the per-slot 10-entry list
  (`DS:0x41FC` cursors, `DS:0x4200` lists). Script records hold
  `(spawn_x, spawn_y, initial_state, …script)`.
  Spawn positions recovered from the four lists: left/right arena edges
  (x≈1…8 or 290…303, y≈102…145) and the centre (146,169) — the centre
  records sit on the player's start position.
* Kid type (`0x42F2`): thresholds `190/127/50/0` → types `5/6/7/8`
  (≈26 %/25 %/30 %/20 %).
* Script advance (`0x4338`): a behavior ends when all frames elapsed; then
  `old_state ≤ 5` adds a small x nudge from `DS:0x41F6 = [2,−4,4,−6,6,−8]`,
  the next script byte becomes the new behavior/state, and `0xFF` ends the
  kid (credit refunded).
* Despawn thresholds: `x < 1`, and `x ≥ 305/299/291` for types 5/6/other.
* Behaviors: walk behaviors 0–5 (8 frames, delay 0 = one frame each),
  throw behaviors 6/7/8 (6–8 frames, delay 3 = 4 frames each), idle 35–37,
  deaths 18–21, arcs 22–24 — all extracted into `src/data.rs`.

### 4.6 Round rules — CONFIRMED

* Level target `[0x398C] = 50 + bonus` (init in `0x3DF0`); bonus grows by
  25 per cleared level; clear condition `kids == 0` at `0x3BD2`.
* Clear bonus: `score += 1000 + snowballs`, message
  ` bonus for clearing level`, then the full level init runs again
  (`jmp 0x3B50`) with a new 60-unit timer.
* Time `[0x399A] = 60`, decremented by `0x4A49` every **41 frames**
  (`[0x397B]` counts 40→−1). Time-out (`time == 0`) or abort flag `[0x18F]`
  ends the round; on time-out the remaining snowballs are converted to
  score (`score += snowballs`), message `you're outta time, slick!`
  (competition) / `you've been snowed, frosty!` (practice).
* Practice mode (`[0xC39]=1`, function `0x4099`): the meter `[0x3FDC]`
  (grows by 6 per snowball created, shrinks 0x100 on type-1 hits, shrinks
  0x300 per cleared level) ends the round when its high byte reaches `0x0F`.
* Specials: `0x462F` spawns a random special every 600 frames
  (thresholds 125/50/0 → types 0/1/9) at positions from `DS:0x39F6`; it
  stays hittable for 600 frames. Running out of snowballs additionally
  grants `+10` every 50 frames and forces a type-0 pickup (`0x47C1` →
  `0x463F` with parameter 125).
* Round over → score/transition screens drawn with `snowfight competition`
  (event id 3); the reconstruction stops at a round-over screen.

### 4.7 Timing — PARTIALLY CONFIRMED

* Simulation is frame based; each frame the draw list is y-sorted (`0x71A7`)
  and the frame pacer `0x60E4` waits on a timer counter
  `while [0x6A26] < [0x6A29]`.
* The exact frame rate (the value written to `[0x6A29]`, installed by
  shared startup code) was not recovered; the reconstruction uses a fixed
  35 Hz step (`SIM_HZ`) — see §6.

## 4.8 Assets (graphics/audio) — added in a later pass

The complete graphics and audio extraction, the pixel-orientation rules, the
per-frame mirror flag, the 8x8 font location and the audio findings are
documented separately in [`assets.md`](assets.md). Two corrections performed
there: the routine at `0x141D7` is a CRTC scroll effect (not AdLib music), and
the sprite records are stored mirrored with a per-frame mirror byte.

## 5. Data tables recovered

Extracted verbatim into `src/data.rs` (numeric gameplay data only — no art,
no audio, no source code):

* movement tables (3 speeds × 9 directions) and the acceleration rule;
* kid type thresholds/types and per-type score values;
* 41 behavior records (per-frame dx/dy, frame delays);
* 40 kid spawn scripts (4 slots × 10 records with spawn positions);
* roaming special parameters (6 types: thresholds, x/y/limit/dx);
* pickup positions, death-behavior mapping, x-nudge table;
* scalar constants (90-frame spawn, 41-frame time tick, 60-unit timer,
  50-kid target, 25 bonus steps, 1000 clear bonus, 200 starting snowballs,
  throw cooldown 4, unlimited duration 400, meter end 0x0F, …).

## 6. Open questions and limitations

1. **`[0x75e3]` gating — resolved in the asset pass.**
   Both the draw (`0x6E67`) and the hit-queue logger (`0x4245`) are gated on
   `[0x75e3] == 1`, and `0x427D/0x4501` set it to
   `(participant == [0x39CD])`: only the **selected team's** kids are visible
   and hittable. A single team alone offers ≈27 spawns per 60×41-frame level
   against a target of 50 hits, but the compass (function `0x14160` draws each
   team's kid count as sprites 127..132) shows that the player is expected to
   **switch teams** with RShift + a cardinal key, which exposes all four
   teams' spawns (≈109 per level). The reconstruction implements the gate and
   the switch, so this is no longer an open tension.
2. **Hit-box sizes** come from sprite descriptors (`LES BX,[BX+0x6DC4]`).
   After extracting the sprite bank (`docs/assets.md`) the real per-sprite
   dimensions are known and are used via `SPRITE_SIZES`; the box tracks the
   current animation frame, exactly like `0x16E67`.
3. **Frame rate**: unknown (`[0x6A29]` not traced to its writer); 35 Hz
   chosen. All recovered intervals are frame ratios, so gameplay balance is
   preserved; wall-clock durations scale with the chosen rate.
4. **Random algorithm** at `0x5D2A` not reversed; a seeded xorshift32 is
   used (deterministic tests).
5. **Sounds/music**: the card-specific `.smb` banks and the three tunes are
   extracted byte-exact into `assets/audio/`; the *driver* is code inside the
   executable (init 0x6105, entry installed at `[0x6A20]`, tune loader
   0x1814D). Playback is not reproduced yet — see `docs/assets.md` §4.
6. **Exit/pause key bindings** ([0x6C07]) not recovered; ESC/P used as
   documented approximations.
7. **Fifth kid slot**: the shipped cursor table disables it, but the
   spawner searches five slots; the reconstruction uses four slots to avoid
   the original's undefined-data path.
8. **Roaming object internal layout** (`0x3A65` block vs. separate x/dx
   variables) was only partially reconstructed; movement/effects are
   reproduced, minor internal coordinates approximated.
9. **No runtime comparison**: DOSBox (or any DOS emulator) is not installed
   in this environment and could not be installed without root, so
   Phase-8-style side-by-side observation was not possible. Validation is
   against the static analysis above (REA `inspect-native-load-image`
   "verified", function dossiers, tables extracted from the image).
10. **Original assets**: the extracted graphics (`assets/`) *are* used by the
    reconstruction — arena background, sprite bank and the executable's 8x8
    font — plus byte-exact audio data; see `docs/assets.md` for formats,
    orientation rules and the audio status. No original code is used.

## 7. Reproduction checklist

```bash
# 1. toolchain (once)
#    Ghidra 12.1.4 + JDK 21, then:
export GHIDRA_INSTALL_DIR=~/tools/ghidra_12.1.4_PUBLIC
export JAVA_HOME=~/tools/jdk-21
npx -y rea-agents@6.4.0 doctor          # ghidra checks must be healthy

# 2. unpack (scratch artifacts, not committed)
python3 unexepack.py Ski_Or_Die/SKIEGA skiega.unpacked.bin
python3 makemz.py    skiega.unpacked.bin skiega.unpacked.mz

# 3. seed + analyze
python3 harvest.py skiega.unpacked.bin seeds.tsv 0xead 0x3b0e 0x6c78 0x6d0c
export REA_GHIDRA_SEED_FILE=$PWD/seeds.tsv
rea inspect-native-load-image skiega.unpacked.mz --provider ghidra --json
rea function skiega.unpacked.mz 0x13df0 --provider ghidra --json
rea function skiega.unpacked.mz 0x147c1 --provider ghidra --json
```

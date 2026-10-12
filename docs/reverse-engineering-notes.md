# Reverse-engineering notes — Snowball War (Ski or Die, DOS)

This document records three things: how the analysis used REA on the
original binary, which findings have evidence, and which questions are
still open. Addresses are **linear offsets in the unpacked load image**
unless this document states otherwise. REA and Ghidra report them with a
load segment of `0x1000`, so a load-image offset `X` appears as address
`0x10000 + X`.

## 1. Tooling and environment

| Item | Value |
|---|---|
| REA | `rea-agents` 6.4.0 (MCP server `io.github.morluto/rea` + CLI `rea`) |
| Deep provider | Ghidra 12.1.4 (installed to `~/tools/ghidra_12.1.4_PUBLIC`) |
| JDK | Temurin 21.0.12.1 (installed to `~/tools/jdk-21`) |
| Other providers | Hopper 5 (`/opt/hopper`) rejects DOS targets, and IDA is not configured |

`rea doctor` initially reported the environment unhealthy because the
only configured deep provider (Hopper) answered:

> `DOS MZ/COM analysis requires the Ghidra 16-bit real-mode adapter.`

REA recommends the bring-your-own-Ghidra adapter for this format
(`docs/ghidra-dos.md` upstream: `MzLoader`, language
`x86:LE:16:Real Mode`, fixed load segment `0x1000`, checked against
Ghidra 12.1.4 + JDK 21). Ghidra and the JDK were installed, and
`GHIDRA_INSTALL_DIR` and `JAVA_HOME` were set. After that, `rea doctor`
reported every Ghidra check healthy (`ghidra-version 12.1.4`,
`ghidra-headless ok`, `ghidra-java 21.0.12.1`).

The opencode MCP server for this session started before those variables
existed in `~/.config/opencode/opencode.jsonc`. Every analysis call
therefore ran through the **REA CLI** (the same package and providers)
with the environment set explicitly, for example:

```bash
export GHIDRA_INSTALL_DIR=~/tools/ghidra_12.1.4_PUBLIC
export JAVA_HOME=~/tools/jdk-21
export REA_GHIDRA_SEED_FILE=/tmp/opencode/seeds.tsv
rea inspect  /tmp/opencode/unpack/skiega.unpacked.mz --provider ghidra --json
rea function /tmp/opencode/unpack/skiega.unpacked.mz 0x13df0 --provider ghidra --json
rea trace    /tmp/opencode/unpack/skiega.unpacked.mz --query "snowfight" --provider ghidra --json
```

The config file was updated so that future sessions expose Ghidra to the
MCP server as well (restart opencode to activate).

## 2. Identifying the target executable

Directory `Ski_Or_Die/` (kept out of the repository) contains:

| File | Role | Evidence |
|---|---|---|
| `S.EXE` | video-adapter launcher (MS C 1988 runtime) | strings `skicga/skiega/skitga`, `Cannot find that video card` |
| `SKICGA`, `SKIEGA`, `SKITGA` | the game executables (CGA/EGA/Tandy) | MZ headers, strings of all `.LZW` data files |
| `SKIORDIE.COM` | crack intro loader (The Humble Guys) | self-decrypting `xor al,0x45` loop at file offset 0x0F. Decrypted strings: `spboat3.dat`, crack group text — irrelevant to gameplay |
| `*.LZW` | compressed game data (for example `SNOWFORT.LZW`, `SFFBM.LZW`, `SFSHP.LZW`) | loaded by the executable (file names inside the binary) |
| `SKI`, `SKI2.BIN`, `HISCORES` | data files | referenced by the executable |

Selected analysis target: **`SKIEGA`** (the 16-colour EGA build)
SHA-256 `6638bd4d4e6e43136d1fcf69b800baa4ad5a1c7850b2f0eed4607d46fdcead70`.

### 2.1 The executable is EXEPACK-compressed

`file` reports a plain `MS-DOS executable, MZ`. The values are
`e_cparhdr=0x20` and `e_minalloc=0x31BD` (≈201 KB of extra memory). The
entry point `0x0CA6:0x0012` sits in the last 167 bytes of the image —
the classic EXEPACK stub layout. A disassembly of the stub (capstone,
16-bit mode) shows the documented EXEPACK algorithm:

```
0x0CA72  mov ax,es / add ax,0x10 / push cs / pop ds / mov [4],ax
         add ax,[0xc] / mov es,ax / mov cx,[6] / std / rep movsb   ; copy stub to scratch
0x0CA92  mov dx,[0xe] / push ax / mov ax,0x38 / push ax / retf     ; jump into the copy
         ... backward stream: control byte, count word, payload
         0xB0 = REP STOSB fill, 0xB2 = REP MOVSB copy, bit0 = last record
         ... relocation table at stub+0x12D, then ljmp [0]:[2]
```

REA states that *"packed targets require a separately identified
unpacked artifact for original-program analysis"*. The analysis replayed
the stream with a small Python program (`unexepack.py`, session scratch
space) that:

1. It reads the MZ header and the parameter block at `CS:0000`
   (`[6]=0x199` is the stub length, `[0xc]=0x0F7C` is the scratch and
   unpacked size).
2. It starts the control stream at `0x0CA5F` and decodes backward
   (`B0/B2`, the final record has bit 0 set).
3. It copies the stored prefix through unchanged (the packer stops
   writing at the same offset where the stream ends).
4. It rebuilds the relocation table (38 entries) from the stub.

The result is a 0xF7C0-byte load image, SHA-256
`79ee524c51ebf424c8f94acbd48396159b77c4b87fed637b710f8a3a6f3e87d3`.
These cross-checks make the reconstruction trustworthy:

* All game strings become readable (`snowfight competition`,
  `snowfort.lzw`, …).
* The entry at `0x00C5` disassembles gaplessly (1437 instructions
  without a decode failure), and its immediate `0x0819` matches
  relocation entry #198.
* The CGA and Tandy builds unpack with the same code and are ≈60 %
  byte-identical to the EGA build (shared game code, different video
  code).

The image was then wrapped in a minimal, self-consistent MZ header
(512-byte header, 38 relocation entries, `CS:IP = 0x000C:0x0005` →
`0x00C5`, `SS:SP = 0x0F7C:0x0200`, `minalloc=0x1000`). The wrapper is a
**derived analysis artifact** (SHA-256
`432c311e78b8bcc22e238b7ef0b2fbfaab531a53a4d1f0d0298708f0a6ebbf7b`).
The shipped file was never modified.

REA checked the artifact (`rea inspect-native-load-image`):

```
status: verified   format: dos-mz   header_bytes: 512   module_bytes: 63424
entry: 0x100c5 (relative_segment 12, offset 5)
language_id: x86:LE:16:Real Mode   loader: MzLoader   load_segment: 0x1000
```

### 2.2 The search for functions needed seeds from callers

The default Ghidra analyzers found only 90 procedures. The event modules
are entered through indirect dispatch, so plain auto-analysis never
reaches them (the gap `0x10E97 … 0x15D2A` contained no functions). REA
supports exactly this case with `REA_GHIDRA_SEED_FILE`
(`address<TAB>function|code|label` lines that run before auto-analysis).

A recursive 8086 traversal (`harvest.py`) found the direct `call` and
far-`jmp` targets. It started from the entry point and two extra roots:
the first code after the startup region, and the Snowball War entry. It
also found the device input functions that are reachable only through
jump tables. The seed result that REA reported:

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

The data segment starts at `0x8190` (`mov ax,0x819` at entry). The
analysis converted the DS-relative offsets (`0xBA68 → DS:0x38D8`) and
searched for them in the code. Each string has exactly one reference,
and all references are inside one module:

```
next snowfighter        DS:0x38C7  -> 0x3D03
snowfight competition   DS:0x38D8  -> 0x3D15
bonus for clearing lvl  DS:0x390C  -> 0x3C29
skiing to snowball blast DS:0x38AE -> 0x3D4B
you're outta time       DS:0x3942  -> 0x3C8E
you've been snowed      DS:0x3926  -> 0x3C98
```

The module boundary is: entry `0x3B0E`, frame loop `0x3B50/0x3B53`,
exit at `0x3D28`, HUD and level init `0x3DF0`, shared code up to
`0x4D00` (the next module starts afterwards).

### 3.1 Recovered function map (linear offsets)

| Address | Role |
|---|---|
| `0x000C5` | program entry (`mov ax,0x819 / mov ds,ax`, then calls) |
| `0x3B0E` | Snowball War entry: counter init, level init, wait for Enter |
| `0x3B50` | level (re)start, `0x3B53` frame loop head |
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
| `0x462F` | periodic specials (600 frames, types 0/1/9) |
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

* Player position is the shared word pair `[0x190]/[0x192]` (x/y).
  `0x3DF0` sets it to `(150,150)`. Function `0x3FDF` clamps it to
  `x∈[0,299]` and `y∈[53,199]` (`cmp [si],0x12f / jbe`,
  `cmp [si+2],0x35 / ncmp 0xc7`). The 53-pixel floor is the HUD band.
* Objects clamp `x∈[0,319]` (`[0x6AE1]=0x013F`, written by `0x3DE9`)
  and `y ≤ 199` (`0x64EC`).
* There are four participant groups (indices 0…3). Each group has:

  * Five kid slots. The table `DS:0x3B04 → DS:0x3B0C` holds 20 slot
    pointers with stride 31 bytes (blocks like `0x3FFE`).
  * Three snowball slots. The table `DS:0x3A88 → DS:0x3A90` holds 12
    pointers.
  * A spawn timer `[0x3985+idx]`, a state counter `[0x3981+idx]`
    (= active kids), and a snowball counter `[0x3A84+idx]`.

* The fifth kid slot's script cursor is `0xFE` in the shipped data, so
  it is disabled. The reconstruction uses four active slots.
* Group 3 is the default controlled group (`[0x39CD]=3`, written by
  `0x3EA8` and re-applied every level by `0x4116`).

### 4.2 Controls — CONFIRMED

* The keyboard function `0x6D0C` scans the key states
  `[0x6B08 + scan]` for scans `0x47,0x48,0x49,0x4B,0x4D,0x4F,0x50,0x51`
  (keypad and arrows). It returns numpad codes `7,8,1,6,2,5,4,3`
  (0 = none). The priority order is the scan order above.
* **State**: function `0x6C78` returns 1 for `[0x6C91]=0x1C` (**Enter**),
  2 for `[0x6C92]=0x36` (**Right Shift**), and 3 for `[0x6C93]=0xFF`
  (unbound). The module init `0x3D80` installs the bindings.
* **Throw and start**: device state 1 (Enter) uses `0x47C1`.
* **Participant switch**: state 2 (RShift) + an even direction
  `2/4/6/8` selects participant `(dir/2)-1` = `0/1/2/3` (`0x40EE`).
* Movement is continuous (table driven, every frame). Aiming is not
  independent: the throw uses a fixed point relative to the player.
* This module has no jump or duck action.
* Pause and exit flags exist (`[0x6C07]=1` is exit, `=0x19` is pause,
  and `0x6821`/entry wait handles them). The exact keys that set them
  were not recovered.

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

The level lives in the high byte of a word that grows by 40 per moving
frame. The player accelerates from 2 px to 4 px after ≈7 frames, and to
the cap of 6 px after ≈13 frames. Releasing the key resets the speed at
once.

### 4.4 Snowball mechanics — CONFIRMED (highest priority)

* **Creation** (`0x448F`, called from `0x4338`): a kid of the current
  participant creates a snowball when it is in a throw behavior (script
  state `6…8`), its animation frame counter equals **5**, and the frame
  delay was just reloaded. The snowball comes from the participant's
  pool (3 slots, `[0x3A84+idx]` counter), starts at the kid's position,
  and gets an arc behavior that depends on the kid type: type 7 →
  behavior 24, type 6 → behavior 23, otherwise behavior 22. The meter
  grows by 6: `[0x3FDC] += 6`.
* **Motion** (`0x64EC`): every frame the position is
  `spawn_base + (dx[frame], dy[frame])` from the behavior table, and the
  arena clamps it. The frame advances when a per-frame delay counter
  underflows (delay = `behavior[0x0B] − [0x6AE3]`, and `[0x6AE3]=0` in
  this module, so delay 0 = every frame and delay 3 = every 4th frame).
  The three snowball arcs (behaviors 22/23/24, 19/17/13 frames) rise
  (`dy` from −7 to −17) and then fall (up to `dy=+18`) while drifting
  sideways. **The trajectory is a scripted ballistic arc, not physics.**
  There is no gravity or wind integration, and no bouncing.
* **Expiry** (`0x4501`): when the arc script completes (`[slot+0x16]=1`),
  the end of the script removes the snowball and decrements the
  participant's counter. It plays a sound cue for the
  controlled group. An 8/256 random side effect also fires (a short CRTC
  scroll jitter, see 0x141D7 — this routine is a screen effect, not
  audio). Snowballs are never removed by arena bounds.
* **The player's throw** (`0x47C1`): device state 1 starts the sound and
  sets debounce `[0x397D]=[0x397E]` (cooldown 4, so one throw per 5
  frames). If `[0x397E] != 0`, the throw runs `snowballs -= 1` (it is
  free while unlimited). Every queued object of the current frame whose
  box contains the player point `(x+9, y−8)` is hit-resolved (`0x48B0`).
  Objects already marked `0xFE` are skipped.
* **Box test** (`0x14879`): `obj.x ≤ px+9 ≤ obj.x + obj.w` and
  `obj.y − obj.h ≤ py−8 ≤ obj.y`. The y value is the object's baseline.
  `0x6E67` takes the box from the sprite descriptor (segment pointers
  are resolved at runtime, and widths and heights are **not statically
  recoverable**, see §6).
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

* **Damage and elimination**: kids have no health. One hit starts the
  death behavior. When that script ends (`0x4338`, type `0xFE` path),
  the `kids` counter is decremented (`[0x398C] -= 1` via `0x4763`) and
  the slot becomes free. The spawn credit is **not** refunded. Unhit
  despawns (walked out of the arena, or script end `0xFF`) **do** refund
  the credit (`[0x3994] += 1`). The player cannot be damaged: no code
  path writes a player-damage state.

### 4.4b Close-up ("snowball in the face") animation — CONFIRMED mapping

`0x148B0` maps kid type 8 to death behavior 21. The frames of behavior
21 use the large close-up sprites 152..155 (a kid peeking over the snow
fort while others throw snowballs). Types 5/6/7 use behaviors 18/19/20
with the small kid sprites. The binary has no position check: the
animation is drawn at the kid's block (`0x14A2F` copies the block x/y
into the slot base), so the original also shows the close-up wherever
the type-8 kid was hit. This build does not reference sprites 147..151
(the "head rising over the wall" steps) in any behavior or immediate.
The reconstruction restricts the close-up to kids in the fort zone
(`CASTLE_Y`). This is a deliberate deviation that play testing
requested. See `behavior-spec.md` §8.

### 4.5 Kids (targets) and their scripts — CONFIRMED

* **Spawn cadence** (`0x427D`): each participant has a byte timer that
  decrements every frame. At the 1→0 transition, the spawn happens when
  the shared credit budget `[0x3994] > 0` and a slot is free. The timer
  then reloads to **90**, the budget spends one credit, and the next
  script comes from the per-slot 10-entry list (`DS:0x41FC` cursors,
  `DS:0x4200` lists). Script records hold
  `(spawn_x, spawn_y, initial_state, …script)`. The spawn positions come
  from the four lists: the left and right arena edges (x≈1…8 or
  290…303, y≈102…145) and the centre (146,169). The centre records sit
  on the player's start position.
* **Kid type** (`0x42F2`): thresholds `190/127/50/0` → types `5/6/7/8`
  (≈26 %/25 %/30 %/20 %).
* **Script advance** (`0x4338`): a behavior ends when all frames
  elapsed. Then `old_state ≤ 5` adds a small x nudge from
  `DS:0x41F6 = [2,−4,4,−6,6,−8]`, the next script byte becomes the new
  behavior/state, and `0xFF` ends the kid (credit refunded).
* **Despawn thresholds**: `x < 1`, and `x ≥ 305/299/291` for types
  5/6/other.
* **Behaviors**: walk behaviors 0–5 (8 frames, delay 0 = one frame
  each), throw behaviors 6/7/8 (6–8 frames, delay 3 = 4 frames each),
  idle 35–37, deaths 18–21, arcs 22–24 — all extracted into
  `src/data.rs`.

### 4.6 Round rules — CONFIRMED

* **Level target**: `[0x398C] = 50 + bonus` (`0x3DF0` initializes it).
  The bonus grows by 25 per cleared level. The clear condition is
  `kids == 0` at `0x3BD2`.
* **Clear bonus**: `score += 1000 + snowballs`, message
  ` bonus for clearing level`, then the full level init runs again
  (`jmp 0x3B50`) with a new 60-unit timer.
* **Time**: `[0x399A] = 60`, decremented by `0x4A49` every **41 frames**
  (`[0x397B]` counts 40→−1). Time-out (`time == 0`) or the abort flag
  `[0x18F]` ends the round. On time-out the remaining snowballs are
  converted to score (`score += snowballs`). The message is
  `you're outta time, slick!` (competition) or
  `you've been snowed, frosty!` (practice).
* **Practice mode** (`[0xC39]=1`, function `0x4099`): the meter
  `[0x3FDC]` grows by 6 per snowball created, shrinks 0x100 on type-1
  hits, and shrinks 0x300 per cleared level. The round ends when the
  meter high byte reaches `0x0F`.
* **Specials**: `0x462F` spawns a random special every 600 frames
  (thresholds 125/50/0 → types 0/1/9) at positions from `DS:0x39F6`.
  The special stays hittable for 600 frames. When the player has no
  snowballs, the game grants `+10` every 50 frames and forces a type-0
  pickup (`0x47C1` → `0x463F` with parameter 125).
* **Round over**: the score and transition screens use
  `snowfight competition` (event id 3). The reconstruction stops at a
  round-over screen.

### 4.7 Timing — PARTIALLY CONFIRMED

* The simulation is frame based. Each frame the draw list is y-sorted
  (`0x71A7`), and the frame pacer `0x60E4` waits on a timer counter
  `while [0x6A26] < [0x6A29]`.
* The exact frame rate was not recovered (the value written to
  `[0x6A29]` comes from shared startup code). The reconstruction uses a
  fixed 35 Hz step (`SIM_HZ`) — see §6.

### 4.8 Graphics and audio assets

[`assets.md`](assets.md) documents the full graphics and audio
extraction, the pixel-orientation rules, the per-frame mirror flag, the
8x8 font location, and the audio findings. The asset pass made two
corrections: the routine at `0x141D7` is a CRTC scroll effect (not AdLib
music), and the sprite records are stored mirrored with a per-frame
mirror byte.

## 5. Data tables recovered

Extracted verbatim into `src/data.rs` (numeric gameplay data only — no
art, no audio, no source code):

* Movement tables (3 speeds × 9 directions) and the acceleration rule.
* Kid type thresholds and types, and the per-type score values.
* 41 behavior records (per-frame dx/dy, frame delays).
* 40 kid spawn scripts (4 slots × 10 records with spawn positions).
* Roaming special parameters (6 types: thresholds, x/y/limit/dx).
* Pickup positions, death-behavior mapping, x-nudge table.
* Scalar constants (90-frame spawn, 41-frame time tick, 60-unit timer,
  50-kid target, 25 bonus steps, 1000 clear bonus, 200 starting
  snowballs, throw cooldown 4, unlimited duration 400, meter end 0x0F, …).

## 6. Open questions and limitations

1. **`[0x75e3]` gating — resolved in the asset pass.** The draw step
   (`0x6E67`) and the hit-queue logger (`0x4245`) run only when
   `[0x75e3] == 1`. Functions `0x427D/0x4501` set the flag to
   `(participant == [0x39CD])`, so only the **selected team's** kids are
   visible and hittable. One team alone offers ≈27 spawns per
   60×41-frame level against a target of 50 hits. The compass shows that
   the player must **switch teams** with RShift and a cardinal key
   (function `0x14160` draws each team's kid count as sprites 127..132).
   The switch exposes all four teams' spawns (≈109 per level). The
   reconstruction implements the gate and the switch, so this question is
   closed.
2. **Hit-box sizes** come from sprite descriptors
   (`LES BX,[BX+0x6DC4]`). After extracting the sprite bank
   (`docs/assets.md`), the real per-sprite dimensions are known and are
   used via `SPRITE_SIZES`. The box tracks the current animation frame,
   exactly like `0x16E67`.
3. **Frame rate**: unknown (`[0x6A29]` was not traced to its writer).
   The reconstruction uses 35 Hz. All recovered intervals are frame
   ratios, so the gameplay balance is preserved. The wall-clock durations
   scale with the selected rate.
4. **Random algorithm**: the routine at `0x5D2A` was not reversed. The
   reconstruction uses a seeded xorshift32 (deterministic tests).
5. **Sound and music**: the card-specific `.smb` banks and the three
   tunes are in `assets/audio/`, byte-exact. The *driver* is code inside
   the executable (init 0x6105, entry installed at `[0x6A20]`, tune
   loader 0x1814D). Playback is not reproduced yet — see
   `docs/assets.md` §4.
6. **Exit and pause key bindings** ([0x6C07]): not recovered. The
   reconstruction uses ESC and P as documented approximations.
7. **Fifth kid slot**: the shipped cursor table disables it, but the
   spawner searches five slots. The reconstruction uses four slots to
   avoid the original's path that reads undefined data.
8. **Roaming object internal layout** (`0x3A65` block vs. separate x/dx
   variables): only a partial reconstruction exists. The movement and
   effects are reproduced, and the minor internal coordinates are
   approximations.
9. **No runtime comparison**: DOSBox (or any DOS emulator) is not
   installed in this environment and could not be installed without
   root, so Phase-8-style side-by-side observation was not possible. The
   validation is the static analysis above (REA
   `inspect-native-load-image` reports "verified", plus the function
   dossiers and the tables that the image provides).
10. **Original assets**: the reconstruction *does* use the extracted
    graphics (`assets/`): the arena background, the sprite bank, and the
    executable's 8x8 font. It also uses byte-exact audio data. See
    `docs/assets.md` for the formats, the orientation rules, and the
    audio status. No original code is used.

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

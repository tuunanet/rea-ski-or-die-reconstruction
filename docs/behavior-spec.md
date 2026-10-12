# Behavior specification — Snowball War

This document describes the minigame in a way that is independent of the
implementation. The rules come from the Ski or Die DOS executable. Every
rule carries a confidence label:

* **CONFIRMED** — code or data recovered with REA supports the rule
  directly (the address is cited, see `reverse-engineering-notes.md`).
* **APPROXIMATION** — behavior that the reconstruction selects where the
  original could not be recovered statically. It also covers an
  interpretation that conflicting evidence forces.

The spec uses frame-based time. One *frame* is one simulation step.

---

## 1. Game states

```
                 Enter (start key)
   ┌──────┐ ─────────────────────────► ┌─────────┐
   │ Menu │                            │ Playing │
   └──────┘ ◄───────────────────────── └─────────┘
       ▲         Enter (after round)        │  kids == 0  → level clear
       │                                    │  time == 0  → time-out
       │         ┌───────────┐              │  abort flag → time-out
       └──────── │ RoundOver │ ◄────────────┘
                 └───────────┘   ESC (exit) → RoundOver
```

* **Menu** is the attract state. The original waits for device state 1 at
  `0x3B0E`. The reconstruction offers a competition and practice toggle
  here (APPROXIMATION: the original selects the mode in the shared event
  menu, and that code was out of scope).
* **Playing** is one *level* of a session. A session starts with
  `score=0, level=1, snowballs=200, meter=0` (`0x3B0E`).
* **RoundOver** shows the final message, the score, and the level. The
  original then continues into score and transition screens that use the
  string `snowfight competition`.
* Pause (APPROXIMATION, key `P`) freezes the simulation inside `Playing`.

## 2. Level state (first set by `0x3DF0`)

| Variable | Init | Meaning |
|---|---|---|
| cursor x,y | (150,150) | throwing cursor position |
| selected participant | 3 (`0x4116`) | `[0x39CD]` |
| kids target | 50 + bonus | remaining hits needed |
| spawn credits | 50 + bonus | spawns allowed (refilled on unhit despawns) |
| bonus | += 25 on clear | grows the next target |
| time | 60 | seconds-like units |
| snowballs | *persist across levels* | ammo |
| score | *persists* | score |
| throw cooldown | 4 | frames between throws |
| spawn timers | 1 | per participant, first spawn on frame 1 |

## 3. Participants and objects

There are four participant groups (0…3). Each group owns:

* **Kids (targets)** — up to 4 at the same time. The fifth slot's cursor
  is disabled in the shipped data (APPROXIMATION: the original searches 5
  slots and would read undefined data on the fifth).
* **Snowballs** — up to 3 in flight (`[0x3A84+idx]`).

The player controls one participant at a time (`selected`). The position,
the throw, and the score are **session-global**. Switching only changes
which group's kids are logged and visible, and the original gate decides
this (see §9).

The object kinds and their hit types are:

| kind | hit type | life |
|---|---|---|
| kid | 5/6/7/8 (random) | script ends or the kid leaves the arena. The type is 0xFE after a hit. |
| kid snowball | — (never hittable) | arc script ends |
| roaming special | 2/3/4/10/11/12 | walks until out of range. 400-frame respawn. |
| pickup special | 0/1/9 | 600-frame lifetime. 600-frame respawn. |

## 4. Input semantics (CONFIRMED, `0x6D0C`/`0x6C78`/`0x40EE`)

| Action | Binding | Semantics |
|---|---|---|
| Move | keypad/arrows (8-way) | direction code 1…8 (numpad layout), 0 = none. The first pressed key in scan order wins. |
| Throw / Start | Enter (state 1) | held → repeats every 5 frames (cooldown 4 + debounce) |
| Switch participant | Right Shift (state 2) + cardinal key | even direction 2/4/6/8 → participant 0/1/2/3 |
| Exit round | Esc (APPROXIMATION of `[0x6C07]=1`) | → RoundOver |
| Pause | P (APPROXIMATION of `[0x6C07]=0x19`) | freeze |

The controlled marker is a **cursor/cross-hair**, not a character. The
cursor uses sprite 124 (red) or 143 (green) while the throw is ready or
`unlimited`. Kids are separate objects. The throw tests the cursor hot
point `(x+9, y−8)` inside the box of a kid.

## 5. Movement (CONFIRMED, `0x3FDF`)

```
if dir == 0: accel_word = 0                      # instant reset
else:
    level = min(high_byte(accel_word), 2)
    x += DX[level][dir]; y += DY[level][dir]
    accel_word += 40                             # high byte is the speed level
    if high_byte > 2: high_byte = 2
x = clamp(x, 0, 299); y = clamp(y, 53, 199)
```

| level | speed (dx table) | reached after |
|---|---|---|
| 0 | 2 px/frame (diagonals same) | start |
| 1 | 4 px/frame | ≈7 moving frames |
| 2 | 6 px/frame (cap) | ≈13 moving frames |

There is no jump or duck action. There is no collision with kids or arena
props: the arena has **no cover or obstacle collision** (CONFIRMED). The
clamps and the kid despawn thresholds are the only blocking logic.

## 6. Kids: spawn, motion, script (CONFIRMED, `0x427D`, `0x4338`, `0x64EC`)

```
per participant, per frame:
  timer -= 1
  if timer == 0 and credits > 0 and a slot is free:
      timer = 90
      credits -= 1
      record = scripts[slot][cursor[slot]]; cursor[slot] = (cursor+1) mod 10
      kind = pick(random)                     # thresholds 190/127/50/0 → 5/6/7/8
      kid = Kid(record.x, record.y, record.script)
      log(kid) into the frame queue if this group is selected
```

The kid frame step runs at `0x64EC` and `0x4338`:

```
position = clamp(base + frames[frame], x≤319, y≤199)
delay -= 1
if delay < 0:
    delay = behavior.delay          # 0 → every frame, 3 → every 4th frame
    frame += 1
    if frame == behavior.frame_count: done = true
# then, in the script processor:
if x < 1 or x ≥ type_limit (305/299/291): despawn, credits += 1
if not dying: log into the frame queue (selected group only)
if state in {6,7,8} and frame == 5 and delay just reloaded:
    create snowball at the kid's position      # the throw
if done:
    if kind == 0xFE: despawn, kids_target -= 1     # death finished
    else if next script byte == 0xFF: despawn, credits += 1
    else:
        base_x = x + nudge[old_state]     # nudge = [2,-4,4,-6,6,-8]
        behavior = next script byte; frame = 0; delay = 0
```

The recovered spawn positions are the left and right edges (x≈1…8,
290…303) at y≈102…145, and the centre (146,169).

## 7. Snowballs (CONFIRMED, `0x448F`, `0x64EC`, `0x4501`)

* **Creation** — a snowball comes only from a kid's throw (frame 5 of
  behavior 6/7/8). It goes into the 3-slot pool of the owning
  participant. The arc behavior depends on the kid type:
  `7 → 24, 6 → 23, else → 22`. The creation adds 6 to the meter:
  `meter += 6`.
* **Motion** — the arc is scripted. Every frame the position is
  `pos = launch + (dx[frame], dy[frame])`, and the arena clamps it. The
  frame advances every `delay+1` frames (arcs use delay 0, so one frame
  per step). Arc 22 has 19 frames and the values
  `dy = 0,-7,-10,-15,-17,-11,-12,-10,-8,-3,+8,+8,+18,…` with a slow
  sideways drift. The snowball **rises and then falls**. There is **no
  gravity integration, no bounce, and no wind**.
* **Expiry** — the end of the script removes the snowball and decrements
  the pool count. It plays a cue for the selected group, and it starts a
  music event with chance 8/256.
* **Player throw** (`0x47C1`) — Enter starts the throw. The game sets
  `debounce = cooldown`. If `cooldown != 0`, it then runs
  `snowballs -= 1` (clamped at 0). **Every** queued object whose box
  contains `(x+9, y−8)` is hit-resolved, so one throw can hit several
  overlapping objects. Queued objects with type `0xFE` are skipped.
* **Collision box** — the box is `x … x+w` and `y-h … y` (y is the
  baseline). The size comes from the sprite of the **current animation
  frame** (`0x16E67` writes the sprite's width and height into the object
  block). The boxes therefore come from the extracted sprite dimensions
  (`SPRITE_SIZES`, CONFIRMED via `docs/assets.md`).

## 8. Damage, scoring, elimination (CONFIRMED, `0x48B0`, `0x4763`)

* A hit kid becomes type `0xFE` and plays its death behavior
  (18/19/20/21 for types 5/6/7/8). Behavior 21 is the **big close-up**
  ("snowball in the face", sprites 152..155) that belongs to the snow
  fort. The shipped binary selects it only by kid type, so a type-8 kid
  hit in the open field also shows the close-up there. The reconstruction
  keeps the type-8 mapping **only at the castle**
  (`y >= CASTLE_Y = 165`). A type-8 kid hit in the field uses the small
  type-7 death (behavior 20) instead (APPROXIMATION agreed with the
  project owner, `src/data.rs`). **Only when that script ends** does
  `kids_target -= 1` and the slot become free. The spawn credit is *not*
  returned: hits consume the budget permanently.
* **Scores**: a kid of type `5/6/7/8` gives `100/75/50/25`. The roaming
  object gives `2 → 50` and `3/10/11/12 → 30`. Type `4 → snowballs −25`.
  Pickup `0 → snowballs +100`, pickup `1 → meter −256`, and pickup
  `9 → unlimited snowballs for 400 frames`.
* The player has **no health and no elimination**. Nothing damages the
  player. Only the clock and the practice meter end a round.
* **Ammo**: the round starts with 200 snowballs. A throw costs one
  snowball, and it is free during `unlimited`. While the player has no
  snowballs, the game gives +10 every 50 frames and forces a type-0
  pickup.
* **HUD** (the draw positions are recovered):

  * `SCORE` label at (6,23), value at (6,30).
  * `SNOWBALLS   KIDS` label at (80,23), values at (104,30) and
    (184,30).
  * `LEVEL` label at (176,5), value after the label.
  * `TIME` label at (224,23), value at (232,30).
  * The status line is at (6,5).
  * The `unlimited` banner is at (80,18).
  * The round messages are at (52,105).

## 9. Teams, compass and the visibility gate — CONFIRMED

The four participants are four **teams** (directions) of kids. Each team
spawns its own kids every 90 frames. The compass at the top right comes
from the arena art. Function `0x14160` annotates it. For every
participant it draws `table[DS:0x39AC + active_kids]`, that is sprites
**127..132** (the little count glyphs), at the DS:0x399C positions.

The selected direction also drives the two `SFSHP.LZW` overlays. Function
`0x14116` redraws the **horizon scenery band**
`table[DS:0x39CE + team]` (`2,1,3,0`) at `(0,40)`. Function `0x14160`
redraws the **compass** `table[DS:0x39D2 + team]` (`6,5,7,4`) at
`(272,3)`. The records are complete images. The wedge points E, S, W, or
N for teams 0..3, the same sectors as the count markers at DS:0x399C.
The compass rectangle is replaced rather than blended. Both overlays run
on a switch, so the background and the compass follow the team (details
in `assets.md` §3.3).

The draw step (`0x6E67`) and the hit-queue logger (`0x4245`) run only
when `[0x75E3] == 1`. Functions `0x427D/0x4501` set the flag to
`(participant == selected)`. **Only the selected team's kids are visible
and hittable.** The roaming object, the pickups, and the cursor are
drawn and logged without this gate (their calls run while the flag is 1).

The level target stays reachable in this way. The player switches teams
with RShift and a cardinal key, so all four teams' spawns (4 per 90
frames, ≈109 per level) can be attacked. One team alone would offer only
≈27 spawns against a fixed target of 50 hits. The reconstruction
implements the gate exactly (`render` and `game` draw and hit only
`g.selected`).

## 10. Round transitions (CONFIRMED, `0x3BD2`…`0x3D28`)

```
every frame, after the throw step:
  if kids_target == 0:                       # level clear
      score += 1000 + snowballs
      show "bonus for clearing level" (freeze ≈120 frames)
      bonus += 25; level += 1; meter −= 768 (clamped ≥ 0)
      re-run the level init (time, target, credits, positions, selected=3)
  else if time == 0 or abort:
      score += snowballs; snowballs = 0
      message = practice ? "you've been snowed, frosty!"
                         : "you're outta time, slick!"
      → RoundOver
  else if exit: → RoundOver
```

The meter high byte `== 0x0F` ends practice mode by setting the abort
flag (`0x4099`).

### 10.1 The message banner (CONFIRMED, `0x13C17` / `0x13C6D`)

Both round endings draw the message in the same way, and the banner makes
the light text readable over the snow:

```
mov ax,0x20   ; x0 =  32
mov si,0x64   ; y0 = 100
mov bx,0x122  ; x1 = 290
mov di,0x73   ; y1 = 115
mov cl,[0x7614] ; 9 (light blue)   ch = [0x7615] = 10
call 0x16387  ; fill (32,100)-(290,115) with cl, then re-call with cl=0
              ; (ch is zeroed at 0x638C, so the second pass edges in black)
mov si,0x390c ; " bonus for clearing level" (0x3926/0x3942 for the time-out)
mov ax,0x34   ; x = 52
mov bx,0x69   ; y = 105
mov cl,[0x7619] ; 14 (yellow)
call 0x1630B  ; draw the message
```

`0x1630B` is the string drawer (AX = x, BX = y, CL = colour, SI = text,
and byte 0x01 in the text is a line break). `0x17AE2` is the glyph
blitter. `0x146BB` redraws the score, and the HUD stays on screen. The
level end does **not** put extra light text on the playfield.

The reconstruction draws exactly this banner for `Game::message` (in
play) and for the round-over message. The original draws its own menu and
prompts on a cleared screen, or does not have them at all. The
reconstruction gives these elements a one-pixel black outline instead, so
the yellow and light colours stay readable over the arena.

`[0x397B]` counts 40→−1, so `time −= 1` every 41 frames. A level lasts
60 units = 2460 frames (≈70 s at the reconstruction's 35 Hz).

## 11. Timing summary (CONFIRMED ratios / APPROXIMATE Hz)

| Event | Frames |
|---|---|
| kid spawn per participant | 90 |
| throw debounce (cooldown 4) | 5 |
| time decrement | 41 |
| level duration | 60 × 41 = 2460 |
| message cycler | 30 |
| ammo refill while empty | 50 |
| roaming respawn | 400 |
| pickup lifetime / respawn | 600 |
| unlimited powerup | 400 |
| level-clear freeze | ≈120 (APPROXIMATION of the fixed delays at `0x3C42`) |
| simulation rate | 35 Hz (APPROXIMATION, §6.3 of the notes) |

## 12. Randomness

All random draws are uniform bytes (original `0x5D2A`): kid type, roaming
type, pickup type, and music chance. The reconstruction replaces them
with a seeded xorshift32 (APPROXIMATION), so runs are reproducible.

## 13. Explicit non-goals

* Sound and music playback — the data is in `assets/audio/`, but the
  driver is not re-implemented (see `docs/assets.md` §4).
* Other Ski or Die minigames — out of scope by project definition.
* Original source code — never used. The reconstruction reads the
  extracted graphics and font from `assets/snowball.pack`.

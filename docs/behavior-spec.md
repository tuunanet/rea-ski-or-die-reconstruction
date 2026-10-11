# Behavior specification — Snowball War

Implementation-independent description of the minigame as recovered from the
Ski or Die DOS executable. Every rule carries a confidence label:

* **CONFIRMED** — directly supported by code/data recovered with REA
  (address cited; see `reverse-engineering-notes.md`).
* **APPROXIMATION** — behaviour chosen where the original could not be
  recovered statically, or an interpretation forced by conflicting evidence.

The spec uses frame-based time: one *frame* is one simulation step.

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

* **Menu** — attract state (original waits for device state 1 at `0x3B0E`).
  The reconstruction exposes a competition/practice toggle here
  (APPROXIMATION: the original selects the mode in the shared event menu,
  whose code was out of scope).
* **Playing** — one *level* of a session. A session starts with
  `score=0, level=1, snowballs=200, meter=0` (`0x3B0E`).
* **RoundOver** — shows the final message, score, level (original continues
  into score/transition screens drawn with `snowfight competition`).
* Pause (APPROXIMATION, key `P`) freezes the simulation inside `Playing`.

## 2. Level state (initialized by `0x3DF0`)

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

Four participant groups (0…3). Each owns:

* **Kids (targets)** — up to 4 concurrent (the fifth slot's cursor is
  disabled in shipped data; APPROXIMATION: the original searches 5 slots and
  would read undefined data on the fifth).
* **Snowballs** — up to 3 in flight (`[0x3A84+idx]`).

The player controls one participant at a time (`selected`); the position,
throw and score are **session-global** — switching only changes which
group's kids are logged/visible per the original gate (see §9).

Object kinds and their hit types:

| kind | hit type | life |
|---|---|---|
| kid | 5/6/7/8 (random) | script ends or leaves arena; 0xFE after a hit |
| kid snowball | — (never hittable) | arc script ends |
| roaming special | 2/3/4/10/11/12 | walks until out of range; 400-frame respawn |
| pickup special | 0/1/9 | 600-frame lifetime, 600-frame respawn |

## 4. Input semantics (CONFIRMED, `0x6D0C`/`0x6C78`/`0x40EE`)

| Action | Binding | Semantics |
|---|---|---|
| Move | keypad/arrows (8-way) | direction code 1…8 (numpad layout), 0 = none; first pressed key in scan order wins |
| Throw / Start | Enter (state 1) | held → repeats every 5 frames (cooldown 4 + debounce) |
| Switch participant | Right Shift (state 2) + cardinal key | even direction 2/4/6/8 → participant 0/1/2/3 |
| Exit round | Esc (APPROXIMATION of `[0x6C07]=1`) | → RoundOver |
| Pause | P (APPROXIMATION of `[0x6C07]=0x19`) | freeze |

The controlled marker is a **cursor/cross-hair** (sprite 124 red, or 143 green
while the throw is ready/unlimited), not a character: kids are separate objects
and the throw tests the cursor hot point `(x+9, y−8)` inside a kid's box.

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

No jump/duck; no collision with kids or arena props (the arena has
**no cover/obstacle collision** — CONFIRMED: the only blocking logic are the
clamps and the kid despawn thresholds).

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

Kid frame (`0x64EC` + `0x4338`):

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

Spawn positions (recovered): the left/right edges (x≈1…8, 290…303) at
y≈102…145 and the centre (146,169).

## 7. Snowballs (CONFIRMED, `0x448F`, `0x64EC`, `0x4501`)

* **Creation** — only from a kid's throw (frame 5 of behavior 6/7/8), into
  the owning participant's 3-slot pool; arc behavior by kid type:
  `7 → 24, 6 → 23, else → 22`; `meter += 6`.
* **Motion** — scripted ballistic arc: every frame
  `pos = launch + (dx[frame], dy[frame])`, clamped to the arena; frame
  advances every `delay+1` frames (arcs use delay 0 ⇒ one frame per step).
  Arc 22 (19 frames): `dy = 0,-7,-10,-15,-17,-11,-12,-10,-8,-3,+8,+8,+18,…`
  with a slow sideways drift — **rise then fall, no gravity integration,
  no bounce, no wind**.
* **Expiry** — script end removes the snowball, decrements the pool count,
  plays a cue for the selected group, 8/256 music event.
* **Player throw** (`0x47C1`) — on Enter:
  `debounce = cooldown`; if `cooldown != 0` then `snowballs -= 1`
  (clamped at 0); then **every** queued object whose box contains
  `(x+9, y−8)` is hit-resolved (a single throw can hit several overlapping
  objects). Queued objects with type `0xFE` are skipped.
* **Collision box** — `x … x+w`, `y-h … y` (y = baseline). The size is the
  **current animation frame's sprite** (`0x16E67` writes the sprite's
  width/height into the object block), so boxes are taken from the extracted
  sprite dimensions (`SPRITE_SIZES`, CONFIRMED via `docs/assets.md`).

## 8. Damage, scoring, elimination (CONFIRMED, `0x48B0`, `0x4763`)

* A hit kid becomes type `0xFE` and plays its death behavior
  (18/19/20/21 for types 5/6/7/8). **Only when that script finishes** does
  `kids_target -= 1` and the slot free up (the spawn credit is *not*
  returned — hits consume the budget permanently).
* Scores: kid `5/6/7/8 → 100/75/50/25`; roaming `2 → 50`,
  `3/10/11/12 → 30`; `4 → snowballs −25`; pickup `0 → snowballs +100`;
  pickup `1 → meter −256`; pickup `9 → unlimited snowballs for 400 frames`.
* There is **no player health/elimination**: the player is never damaged;
  only the clock (and the practice meter) end a round.
* Ammo: start 200; −1 per throw (free during unlimited); while empty,
  +10 every 50 frames plus a forced type-0 pickup.
* HUD (recovered draw positions): `SCORE (6,23)` value `(6,30)`;
  `SNOWBALLS   KIDS (80,23)` values `(104,30)` / `(184,30)`;
  `LEVEL (176,5)` value after the label; `TIME (224,23)` value `(232,30)`;
  status line `(6,5)`; `unlimited` banner `(80,18)`;
  round messages `(52,105)`.

## 9. Teams, compass and the visibility gate — CONFIRMED

The four participants are four **teams/directions** of kids. Each team spawns
its own kids (every 90 frames) and the compass at the top right (drawn by the
arena art) is annotated by function `0x14160`: for every participant it draws
`table[DS:0x39AC + active_kids]`, i.e. sprites **127..132** (the little count
glyphs), at the DS:0x399C positions.

Both the draw (`0x6E67`) and the hit-queue logger (`0x4245`) are gated on
`[0x75E3] == 1`, which `0x427D/0x4501` set to `(participant == selected)`:
**only the selected team's kids are visible and hittable**. The roaming
object, pickups and the cursor are drawn/logged unconditionally (their calls
run while the flag is 1).

This is also how the level target stays reachable: the player switches teams
with RShift + a cardinal key, so all four teams' spawns (4 per 90 frames,
≈109 per level) can be attacked, while a single team alone would offer only
≈27 spawns against a fixed target of 50 hits. The reconstruction implements
the gate exactly (`render`/`game` draw and hit only `g.selected`).

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

Practice end: meter high byte `== 0x0F` sets the abort flag (`0x4099`).

Time: `[0x397B]` counts 40→−1 ⇒ `time −= 1` every 41 frames; a level lasts
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
type, pickup type, music chance. The reconstruction substitutes a seeded
xorshift32 (APPROXIMATION) so runs are reproducible.

## 13. Explicit non-goals

* Sound/music playback — the data is extracted (`assets/audio/`), the driver
  reimplementation is pending (see docs/assets.md §4).
* Other Ski or Die minigames — out of scope by project definition.
* Original art/music/source — never used; graphics are procedural.

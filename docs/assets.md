# Asset extraction — Ski or Die / Snowball War

This document explains how the original graphics and audio data were
extracted and checked, and how the Rust reconstruction uses them. All
evidence comes from the REA analysis of the EXEPACK-unpacked `SKIEGA`
load image (see `reverse-engineering-notes.md`).

## 1. Container format

Every data file is named `*.LZW`, but the codec is **LZSS** (not LZW):

```
+0x00  u32  uncompressed size (little endian)
+0x04       LZSS bit stream: 1-bit literal flag per byte, 8-bit flag groups,
           ring buffer 4096, match length 3..18
```

The file loaders reach the decoder in the executable
(`FUN_100c_5def`/`FUN_100c_5e27`). Every `*_LZW` loader calls this
decoder, and the tune loader at `0x1814D` calls it too.
`tools/extract/extract_assets.py` implements the same algorithm. During
the investigation, an independent third-party C implementation of the
same codec cross-checked every result.

## 2. Image formats

| Kind | Files | Layout |
|---|---|---|
| Single image | `SNOWFORT`, `SPLASH`, `SHOP`, `TSQUARE`, `SIGNUP`, `HISCORE`, `ACROJUMP` | `u8 bytes_per_row`, `u8 height`, raw 4-bit pixels. width = bytes*2 (for example 160x200 → 320x200) |
| Sprite bank | `SFFBM`, `SBFBM`, `TBFBM`, `AJFBM`, `DHFBM`, `MNFBM` | `u16 count`, then per sprite: `u8 bytes_per_row`, `u8 height`, then per row `u8 gap`, `u8 pixel_count`, `pixel_count` bytes of packed nibbles |
| Record bank | `SFSHP`, `SBSHP`, `TBSHP`, `AJSHP`, `DHSHP`, `MNSHP`, `CPSHP` | `u16 count`, then per record: `u8 bytes_per_row`, `u8 height`, raw nibbles |
| Tiles / maps | `*TIL`, `*TILE`, `*MAP` | Snowball War does not use them |

Sprite bank loaders: `0x17FF4` registers each record as (segment, offset)
in the **descriptor table at DS:0x6DC4** and copies the raw record into
the graphics buffer. `0x1807F` does the same for the record banks into a
second table at DS:0x6CFC. The Snowball War module (`0x3D29`) loads the
sprite bank with index 0, so **SFFBM sprite indices are exactly
0..=164**. This matches every sprite index in the recovered behavior
tables.

### 2.1 Pixel orientation and the mirror flag

The sprite data is easy to read. It is stored in the layout that a VGA
planar blitter needs:

* Sprite records are stored **mirrored horizontally** relative to the
  canonical (non-mirrored) sprite. The third-party extraction tool used
  during the investigation records the same ("All images must be fliped
  horizontaly").
* Whether a sprite is drawn mirrored is a **per-frame flag**. The
  behavior frame layout is `(sprite, mirror, dx, dy)`, and `0x164EC`
  copies byte 1 into the object's `[SI+0xE]`. The flusher `0x16F16`
  loads this byte into `AH`, and the blitter `0x177E7` mirrors when
  `AH == 1`. An example from the behavior table DS:0x2DAC (entry 0 vs
  entry 1): `(14,0,0,0) (15,0,3,2) …` walks right, and
  `(14,1,0,0) (15,1,1,2) …` walks left.
* Single full-screen images are stored in display order (the compass in
  `SNOWFORT` reads N/E/S/W correctly with no flip). The third-party
  tool's BMP writer added an extra vertical flip, so its output is
  wrong. The `SFSHP` overlay bank is also in display order (its records
  match the arena's compass and horizon band, §3.3). Only the `SFFBM`
  sprite bank gets the mirror treatment.

To check the result, render a frame with the extracted data and compare
it with the original screen:

```bash
cargo run --example frame -- 300 frame.ppm   # headless render for comparison
```

This command reproduces the HUD text, font, and colours, the arena, the
compass, and the characters.

### 2.2 Palette

The game uses the 16-colour EGA/VGA palette (`PALETTE` in the
extractor). Colour index 0 is the background and transparent colour for
sprites. The HUD colour indices come from DS:0x7614..0x761B (labels
13 = light magenta, values 14 = yellow, player name 12 = light red,
banners 10 = light green).

The palette is not a guess. The video-mode routine at lm 0x17494 sets
BIOS mode 0Dh (`mov ax,0x000D; int 0x10`, 320x200 EGA planar). It then
calls `int 0x10` with `AX=0x1002` (set all palette registers) on the
17-byte table at DS:0x75F8 = `00 01 02 03 04 05 06 07 10 11 12 13 14 15 16 17 00`.
This table maps EGA colour *i* to VGA DAC entry *i* (0-7) and *i+8*
(8-15). The on-screen RGB values are the VGA BIOS default EGA palette,
the same 8-bit values that the extractor writes into the previews and
the pack. The executable does not access the DAC ports (0x3C8/0x3C9)
anywhere, so there are no runtime palette effects and no custom colours.

For this reason, the front end must give SDL a framebuffer whose
*memory* layout is R, G, B, A (`FRAMEBUFFER_FORMAT = RGBA32`, that is
ABGR8888 on little-endian). If SDL reads the same bytes as `ARGB8888`,
it interprets them as B, G, R, A and swaps red and blue. The screen then
disagrees with the extracted assets: the blue sky becomes red, and the
yellow text becomes cyan. `src/main.rs` has a read-back test for this.

### 2.3 Font

An 8x8 font is **inside the executable** at DS:0x6728 (lm 0xE8B8). It
has 8 bytes per character and starts at ASCII 0x20. Each byte is one
row, with the most significant bit on the left. The glyph blitter is
`0x17AE2` (`mov si,0x6728` / `sub al,0x20` / `mul bl`). The string
drawer is `0x162CE`. The extractor copies all 96 glyphs into the asset
pack, so the HUD text is pixel-identical.

## 3. Snowball War asset set

| Asset | Source | Use |
|---|---|---|
| `assets/preview/arena.png`, pack section | `SNOWFORT.LZW` | 320x200 arena background (sky, mountains, village, snow, fort, compass) |
| `assets/preview/sprites.png` + pack | `SFFBM.LZW` (165 sprites) | kids (walk/throw/hit/death), snowballs, hit sparks, cursor marker (124 red / 143 green), roaming specials, big characters |
| `assets/preview/overlay.png` + pack | `SFSHP.LZW` (24 records) | per-team scenery band (0-3) and per-team compass (4-7), plus extra characters — see 3.3 |
| font (inside the pack) | executable DS:0x6728 | HUD text |
| `assets/snowball.pack` | generated by the extractor | single dependency-free binary that the Rust game loads |

### 3.1 Hit boxes

`0x16E67` stores the sprite's width and height into the object block
(`[SI+4]`, `[SI+6]`). The throw test (`0x14879`) checks
`x ≤ px+9 ≤ x+w` and `y−h ≤ py−8 ≤ y`. The reconstruction therefore
takes hit boxes from `SPRITE_SIZES` (the real sprite dimensions) instead
of the fixed 16x28 placeholder that it used earlier.

### 3.2 HUD details recovered with the assets

* Cursor: `[0x190]/[0x192]` is drawn with sprite **124 (red)** or **143
  (green)** — a cross-hair marker, not a character. Kids are separate
  pool objects. The cursor turns green while the throw cooldown or the
  `unlimited` state is active (`0x13FDF` sets `[0x19C] = 0x7C` or
  `0x8F`).
* Compass: function `0x14160` draws one sprite per participant at the
  positions in DS:0x399C. The sprite is `table[0x39AC + kid_count]`,
  that is **sprites 127..=132**, the little count glyphs.
* Player name: the name sits at the top left and comes from the
  `[0x9FE]` name table. The SKI data file holds the default names
  LOTHAR/CHRISTA/ATLE/…. The reconstruction defaults to the screenshot's
  "OLD NERD".

### 3.3 Per-team background and compass

The Snowball War module does use the `SFSHP.LZW` records (the records
are shared with the other events, so a plain search for the overlay
table misses them). Function `0x17739` blits a record (the descriptor is
at DS:0x6CFC, and the loader at `0x1807F` fills it). `AL` picks the
video page (`DS:0x75E4 = A000, A200, A400, A600` — the 8 KB EGA 320x200
pages).

* **Scenery band** — `0x14116` (level init `0x13DF0` and the team-switch
  path `0x140EE`) draws record `table[DS:0x39CE + team]` at `(0, 40)`.
  The table is `2, 1, 3, 0`, so each direction gets its own
  sky/mountain/horizon band.
* **Compass** — `0x14160` (the 30-frame HUD cycler, called from the main
  loop) draws record `table[DS:0x39D2 + team]` at `(272, 3)`, that is
  records **6, 5, 7, 4**: wedges pointing E, S, W, N for teams 0..3.
  This matches the four kid-count markers (`COMPASS_MARKERS`), which sit
  in the same sectors.

The records are **complete images in display order** (they are *not*
mirrored like the `SFFBM` sprite records, and index 0 is not
transparent):

* Un-mirrored record 4 differs from the compass already baked into the
  arena art at `(272,3)` in exactly 2 pixels, so at level start
  (team 3 → record 4) the drawn compass is the same as the arena's own
  compass.
* Each of `5/6/7` is the same compass with only the green wedge in a new
  place (~450 px differ), and the wedge centroid points N, S, E, or W as
  listed above.
* The strips match the arena's horizon band (the village and house sit
  at the same x), so reversing them puts the scene in the wrong place.

Because the records are full images, the reconstruction *replaces* the
record rectangle (an opaque blit) instead of blending. The original's
per-plane AND blit (`and es:[bp], al` with the sequencer map mask
8/4/2/1) can only clear bits. The compass region has bit 0 set in just
26% of its pixels, so a per-plane AND would leave the compass mostly
black. It could never paint the wedge into a new (black) sector.

Switching the active direction (RShift+arrow) changes both the horizon
scenery and the compass, and this matches the original. The
reconstruction draws the band after the arena and before the entities.
It draws the compass from `draw_hud`.

## 4. Audio

The extractor writes these files byte-exact into `assets/audio/`:

| File | Size | Role |
|---|---|---|
| `sbtune.lzw` | 4146 | Snowboard event music (loaded at 0x4CAE) |
| `tbtune.lzw` | 5375 | Innertube-thrash event music |
| `dhtune.lzw` | 11128 | Downhill event music |
| `ski2.bin` | 33714 | title/menu music (same container header) |
| `adski.smb` / `mtski.smb` / `tnski.smb` / `pcski.smb` | 4121 / 8259 / 3440 / 1835 | per-card **sound data banks** (AdLib, MT-32, Tandy, PC speaker) |

The analysis produced these findings:

* The `.smb` files are **data, not code** (they do not disassemble as
  8086). The sound *driver* lives in the executable. The setup at
  lm 0x04E0..0x0531 selects a card and loads the matching `*.smb` into
  segment `[0x6A2C]`. It calls the driver init at `0x6105` and installs
  the far entry point in `[0x6A20]`. Games call it as `lcall [0x6A20]`
  with a command in AX/AH.
* Tune files share the container header of the music data (`7F 00 00 01 …`
  with 0x30-byte track headers). The tune loader is `0x1814D`. The
  player reads the buffer that this loader prepares.
* **Playback is not implemented yet.** Faithful music playback needs
  either an OPL2/FM emulation that the recovered driver commands drive,
  or a port of the driver. REA can supply this (the driver and its
  command set are in the executable and decompile cleanly), but it was
  out of scope for this pass. The reconstruction plays no sound.

## 5. Copy protection

The protection is a **code-wheel question at startup**, not asset
encryption. `CPSHP.LZW` holds the wheel images ("AP", "BB", "GP", …),
and the game asks `please enter number in hole` (strings at
lm 0xEC04/0xEC43). The supplied dump is the cracked release
(`SKIORDIE.COM` crack intro, and the patched check code documented by
the third-party analysis of the same release), so the question is
skipped. No asset needed any decryption beyond LZSS.

## 6. Reproducing the assets (standalone tool)

`tools/extract/build_assets.py` is self-contained. It only needs
Python 3 and a directory with your own extraction of the DOS release.
The tool does these tasks:

1. It finds the game files recursively (any sub-directory layout).
2. It selects the EGA executable (`SKIEGA`, else `SKITGA`/`SKICGA`, else
   `S.EXE`) and **undoes EXEPACK** itself (`unexepack`, the same
   algorithm as §1).
3. It finds the 8x8 font inside the unpacked image by pattern (glyph
   blitter `mov si,<font>` + `sub al,0x20 / mul bl`, with a font-blob
   signature as a fallback), so the tool also works for other builds.
4. It decodes `SNOWFORT.LZW`, `SFFBM.LZW`, and `SFSHP.LZW` with LZSS. It
   decodes the image formats and applies the orientation and mirror
   rules of §2.1.
5. It writes `assets/snowball.pack` (and prints its SHA-256) and copies
   the audio data into `assets/audio/`. With `--preview` it also writes
   PNG previews (Pillow optional, PPM otherwise).

```bash
python3 tools/extract/build_assets.py Ski_Or_Die assets [--preview]
```

These values are the reference for the release used during the
analysis: pack SHA-256
`1a45a8486a24e77e3ac5381cd17f2b6d4c1ef1292bf62c68c0151b0252cf100a`
(1 079 163 bytes), arena 320x200, 165 sprites, 24 overlay records, and
96 glyphs. `assets/` is generated and git-ignored, so nothing from the
original game needs to be committed.

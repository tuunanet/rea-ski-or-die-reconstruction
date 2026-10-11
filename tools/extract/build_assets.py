#!/usr/bin/env python3
"""Build the assets the Rust Snowball War reconstruction needs.

Standalone: give it a directory containing an extracted Ski or Die DOS
release and it discovers the files, unpacks the EXEPACK-compressed executable,
decodes the LZSS containers and the three image formats, recovers the 8x8 HUD
font from the executable and writes:

    <out>/snowball.pack        everything the game loads at runtime
    <out>/audio/*.lzw,*.smb    byte-exact tune / sound-bank data
    <out>/preview/...          optional human-readable previews (--preview)

Only Python 3 and the standard library are required; Pillow is used for PNG
previews when available (PPM otherwise). Nothing from the original game is
committed - the pack is generated locally from the user's own copy.

usage:
    python3 tools/extract/build_assets.py [game_dir] [out_dir] [--preview]
"""
import argparse
import hashlib
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from extract_assets import (  # noqa: E402
    PALETTE, bm_bank, find_file, find_font, list_bank, load_image, lzss,
    nibbles_to_rows, plausible_font, to_rgba, write_ppm, FONT_GLYPHS,
)

try:
    from PIL import Image
except Exception:  # pragma: no cover - previews fall back to PPM
    Image = None

EXE_NAMES = ["SKIEGA", "SKITGA", "SKICGA", "SKIEGA.EXE", "S.EXE", "SKI.EXE"]
AUDIO_FILES = ["SBTUNE.LZW", "TBTUNE.LZW", "DHTUNE.LZW", "SKI2.BIN",
               "ADSKI.SMB", "MTSKI.SMB", "TNSKI.SMB", "PCSKI.SMB"]


def pack_sheet(imgs, cols):
    """Atlas layout used by both the pack and the previews: cell = max+1."""
    cw = max(i.width for i in imgs) + 1
    ch = max(i.height for i in imgs) + 1
    rows = (len(imgs) + cols - 1) // cols
    out = Image.new("RGBA", (cols * cw, rows * ch), (0, 0, 0, 0))
    rects = []
    for n, im in enumerate(imgs):
        r, c = divmod(n, cols)
        x, y = c * cw, r * ch
        out.paste(im, (x, y))
        rects.append((x, y, im.width, im.height))
    return out, rects


def indexed_atlas(sprites, cols):
    """Same layout as pack_sheet, but as palette indices (what the pack stores)."""
    cw = max(w for _rows, w, _h in sprites) + 1
    ch = max(h for _rows, _w, h in sprites) + 1
    rows = (len(sprites) + cols - 1) // cols
    aw, ah = cols * cw, rows * ch
    px = bytearray(aw * ah)
    rects = []
    for n, (srows, w, h) in enumerate(sprites):
        r, c = divmod(n, cols)
        ox, oy = c * cw, r * ch
        for y in range(h):
            for x in range(w):
                px[(oy + y) * aw + ox + x] = srows[y][x]
        rects.append((ox, oy, w, h))
    return px, aw, ah, rects


def build_pack(arena, sprites, overlay, font):
    """Serialise the SOWA v1 pack (see docs/assets.md)."""
    out = bytearray(b"SOWA")
    out += (1).to_bytes(2, "little")
    for c in PALETTE:
        out += bytes(c)
    rows, w, h = arena
    out += w.to_bytes(2, "little") + h.to_bytes(2, "little")
    out += bytes(v for row in rows for v in row)
    spx, saw, sah, srects = indexed_atlas(sprites, 16)
    out += len(srects).to_bytes(2, "little")
    for (x, y, sw, sh) in srects:
        out += x.to_bytes(2, "little") + y.to_bytes(2, "little")
        out += sw.to_bytes(2, "little") + sh.to_bytes(2, "little")
    out += saw.to_bytes(2, "little") + sah.to_bytes(2, "little")
    out += bytes(spx)
    opx, oaw, oah, orects = indexed_atlas(overlay, 6)
    out += len(orects).to_bytes(2, "little")
    for (x, y, sw, sh) in orects:
        out += x.to_bytes(2, "little") + y.to_bytes(2, "little")
        out += sw.to_bytes(2, "little") + sh.to_bytes(2, "little")
    out += oaw.to_bytes(2, "little") + oah.to_bytes(2, "little")
    out += bytes(opx)
    out += bytes([0x20]) + FONT_GLYPHS.to_bytes(2, "little") + bytes([8, 8])
    # load_font() returns the executable's raw glyph bytes (8 rows per glyph,
    # most significant bit left), which is exactly what the pack stores.
    out += font[: FONT_GLYPHS * 8]
    return bytes(out)


def load_font(game_dir):
    for name in EXE_NAMES:
        path = find_file(game_dir, [name])
        if not path:
            continue
        image = load_image(path)
        off = find_font(image)
        if off is None:
            continue
        print(f"  executable  {os.path.basename(path)} ({len(image)} bytes unpacked, "
              f"font at {hex(off)})")
        return image[off:off + FONT_GLYPHS * 8]
    raise SystemExit("error: could not find the 8x8 font in any game executable "
                     f"(looked for {EXE_NAMES})")


def read_container(game_dir, name):
    path = find_file(game_dir, [name])
    if not path:
        raise SystemExit(f"error: missing data file {name} under {game_dir}")
    raw = open(path, "rb").read()
    decoded = lzss(raw)
    print(f"  data        {os.path.basename(path)} ({len(raw)} -> {len(decoded)} bytes)")
    return decoded


def main(argv):
    ap = argparse.ArgumentParser(description="build Snowball War assets")
    ap.add_argument("game_dir", nargs="?", default="Ski_Or_Die",
                    help="directory with the extracted Ski or Die release")
    ap.add_argument("out_dir", nargs="?", default="assets",
                    help="output directory (default: assets)")
    ap.add_argument("--preview", action="store_true", help="also write preview images")
    args = ap.parse_args(argv)

    if not os.path.isdir(args.game_dir):
        raise SystemExit(f"error: {args.game_dir} not found - extract the Ski or Die "
                         "release there first")
    os.makedirs(os.path.join(args.out_dir, "audio"), exist_ok=True)

    font = load_font(args.game_dir)

    fort = list_bank(read_container(args.game_dir, "SNOWFORT.LZW"))
    if len(fort) != 1:
        raise SystemExit("error: unexpected SNOWFORT.LZW layout")
    arena = fort[0]
    sprite_rows, sprite_sizes = bm_bank(read_container(args.game_dir, "SFFBM.LZW"))
    sprites = [(rows, w, h) for rows, (w, h) in zip(sprite_rows, sprite_sizes)]
    overlay = list_bank(read_container(args.game_dir, "SFSHP.LZW"))
    if not sprites or not overlay:
        raise SystemExit("error: decoded sprite banks are empty")

    pack = build_pack(arena, sprites, overlay, font)
    pack_path = os.path.join(args.out_dir, "snowball.pack")
    with open(pack_path, "wb") as fh:
        fh.write(pack)
    print(f"wrote {pack_path} ({len(pack)} bytes, sha256 {hashlib.sha256(pack).hexdigest()})")
    print(f"  arena {arena[1]}x{arena[2]}, {len(sprites)} sprites, "
          f"{len(overlay)} overlay records, {FONT_GLYPHS} font glyphs")

    if args.preview:
        pdir = os.path.join(args.out_dir, "preview")
        os.makedirs(pdir, exist_ok=True)
        rows, aw, ah = arena
        if Image is not None:
            img = Image.new("RGB", (aw, ah))
            px = img.load()
            for y in range(ah):
                for x in range(aw):
                    px[x, y] = PALETTE[rows[y][x] & 0x0F]
            img.save(os.path.join(pdir, "arena.png"))
            sheet, _ = pack_sheet([to_rgba(r, w, h) for r, w, h in sprites], 16)
            sheet.save(os.path.join(pdir, "sprites.png"))
            osheet, _ = pack_sheet([to_rgba(r, w, h) for r, w, h in overlay], 6)
            osheet.save(os.path.join(pdir, "overlay.png"))
            print("wrote previews (Pillow: arena.png, sprites.png, overlay.png)")
        else:
            write_ppm(os.path.join(pdir, "arena.ppm"), rows, aw, ah)
            print("wrote previews (PPM: arena.ppm; install Pillow for PNGs)")

    copied = 0
    for name in AUDIO_FILES:
        src = find_file(args.game_dir, [name])
        if src:
            with open(src, "rb") as fh:
                data = fh.read()
            with open(os.path.join(args.out_dir, "audio", name.lower()), "wb") as fh:
                fh.write(data)
            copied += 1
    print(f"copied {copied} audio data files into {args.out_dir}/audio")


if __name__ == "__main__":
    main(sys.argv[1:])

#!/usr/bin/env python3
"""Extract Ski or Die assets into game-ready form for the Rust reconstruction.

The container format (.LZW) and the image formats were recovered by REA
analysis of SKIEGA and are documented in docs/assets.md; this script ports
the decoding rules, cross-checked against an independent third-party C
implementation of the same codec during the investigation, and emits:

  assets/arena.png        - Snowball War arena background (320x200)
  assets/sprites.png      - SFFBM sprite bank atlas (index 0..164), RGBA
  assets/sprites.json     - atlas rectangles + logical pixel sizes
  assets/overlay.png      - SFSHP overlay shapes atlas (compass, sky strips)
  assets/overlay.json
  assets/audio/*.bin      - byte-exact tune/driver data (not decoded)

Every sprite pixel with EGA colour index 0 is treated as transparent, which
matches the in-game blitters (index 0 is the background colour).
"""
import json, os, struct, sys

try:  # Pillow is only needed for the optional PNG previews
    from PIL import Image
except Exception:  # pragma: no cover
    Image = None

RING = 4096
MATCH = 18
THRESH = 2

# EGA/VGA default 16-colour palette used by the game (palette[0..15]).
PALETTE = [
    (0x00, 0x00, 0x00), (0x00, 0x00, 0xA8), (0x00, 0xA8, 0x00), (0x00, 0xA8, 0xA8),
    (0xA8, 0x00, 0x00), (0xA8, 0x00, 0xA8), (0xA8, 0x54, 0x00), (0xA8, 0xA8, 0xA8),
    (0x54, 0x54, 0x54), (0x54, 0x54, 0xFC), (0x54, 0xFC, 0x54), (0x54, 0xFC, 0xFC),
    (0xFC, 0x54, 0x54), (0xFC, 0x54, 0xFC), (0xFC, 0xFC, 0x54), (0xFC, 0xFC, 0xFC),
]


def lzss(data: bytes) -> bytes:
    size = struct.unpack_from("<I", data, 0)[0]
    src = data[4:]
    text = bytearray(b"\x20" * (RING - MATCH)) + bytearray(RING)
    r = RING - MATCH
    out = bytearray()
    flags = 0
    p = 0
    while len(out) < size:
        flags >>= 1
        if (flags & 0x100) == 0:
            if p >= len(src):
                break
            flags = src[p] | 0xFF00
            p += 1
        if flags & 1:
            if p >= len(src):
                break
            c = src[p]
            p += 1
            out.append(c)
            text[r] = c
            r = (r + 1) & (RING - 1)
        else:
            if p + 1 >= len(src):
                break
            i = src[p]
            j = src[p + 1]
            p += 2
            i |= (j & 0xF0) << 4
            for k in range((j & 0x0F) + THRESH + 1):
                c = text[(i + k) & (RING - 1)]
                out.append(c)
                text[r] = c
                r = (r + 1) & (RING - 1)
    return bytes(out[:size])


def nibbles_to_rows(raw: bytes, bytes_per_row: int, rows: int):
    """Return a list of rows of 4-bit indices, top row first.

    Verified against the original screen: the stored row order is the display
    order (the compass lettering reads N/E/S/W correctly with no vertical
    flip); the third-party tool's BMP writer added an extra flip."""
    rows_data = []
    for y in range(rows):
        row = raw[y * bytes_per_row:(y + 1) * bytes_per_row]
        px = []
        for b in row:
            px.append(b >> 4)
            px.append(b & 0x0F)
        rows_data.append(px)
    return rows_data


def raw_image(raw: bytes):
    w = raw[0]
    h = raw[1]
    return nibbles_to_rows(raw[2:], w, h), w * 2, h


def rle_sprite(data: bytes):
    """SFFBM sprite: header width(bpr)/height then per-row [gap][count][data]."""
    bpr = data[0]
    h = data[1]
    p = 2
    rows = []
    max_px = 0
    for _ in range(h):
        gap = data[p]
        cnt = data[p + 1]
        p += 2
        px = [0] * gap
        for k in range(cnt):
            b = data[p + k]
            px.append(b >> 4)
            px.append(b & 0x0F)
        p += cnt
        px += [0] * (bpr * 2 - len(px))
        max_px = max(max_px, len(px))
        rows.append(px)
    # Sprite records are stored mirrored horizontally and blitted top-down;
    # the original blitter mirrors at draw time when the behaviour frame flag
    # ([SI+0xE]) is set (function 0x177E7). Verified against the compass
    # lettering in SFSHP.
    rows = [list(reversed(r)) for r in rows]
    return rows, bpr * 2, h


def sprite_bytes_size(data: bytes) -> int:
    bpr = data[0]
    h = data[1]
    p = 2
    for _ in range(h):
        cnt = data[p + 1]
        p += 2 + cnt
    return p


def to_rgba(rows, w, h, transparent_zero=True, pad_to=None, pad_h=None):
    pw = pad_to or w
    ph = pad_h or h
    img = Image.new("RGBA", (pw, ph), (0, 0, 0, 0))
    px = img.load()
    for y, row in enumerate(rows):
        for x, v in enumerate(row):
            if x >= pw:
                break
            if v == 0 and transparent_zero:
                continue
            px[x, y] = PALETTE[v] + (255,)
    return img


def skin(rows, w, h):
    img = Image.new("RGB", (w, h))
    px = img.load()
    for y, row in enumerate(rows):
        for x, v in enumerate(row):
            px[x, y] = PALETTE[v]
    return img


def atlas(images, cols=None):
    """Simple shelf packer; returns atlas image + rects."""
    cols = cols or 16
    cell_w = max((im.width for im in images), default=1) + 1
    cell_h = max((im.height for im in images), default=1) + 1
    rows = (len(images) + cols - 1) // cols
    out = Image.new("RGBA", (cols * cell_w, rows * cell_h), (0, 0, 0, 0))
    rects = []
    for n, im in enumerate(images):
        r, c = divmod(n, cols)
        x, y = c * cell_w, r * cell_h
        out.paste(im, (x, y))
        rects.append({"index": n, "x": x, "y": y, "w": im.width, "h": im.height})
    return out, rects


def bm_bank(raw: bytes):
    count = struct.unpack_from("<H", raw, 0)[0]
    p = 2
    sprites = []
    sizes = []
    for _ in range(count):
        size = sprite_bytes_size(raw[p:])
        rows, w, h = rle_sprite(raw[p:p + size])
        sprites.append(rows)
        sizes.append((w, h))
        p += size
    assert p == len(raw), f"BM bank trailing bytes: {p} != {len(raw)}"
    return sprites, sizes


def list_bank(raw: bytes, mirror: bool = False):
    """Single full image (width*height == size-2) or a bank of
    (width, height, raw) records preceded by a u16 count.

    ``mirror`` reverses every stored row; the sprite bank (SFFBM) needs it
    because its records are stored mirrored and the original blitter flips
    them back per frame.  The overlay bank (SFSHP) is stored in display order
    - its un-mirrored record 4 is pixel-identical to the compass baked into
    SNOWFORT.LZW at (272,3) until 2 stray pixels, and the scenery strips line
    up with the arena's own horizon band (verified in assets.md 3.3)."""
    if raw[0] * raw[1] == len(raw) - 2:
        w, h = raw[0], raw[1]
        return [(nibbles_to_rows(raw[2:], w, h), w * 2, h)]
    count = struct.unpack_from("<H", raw, 0)[0]
    p = 2
    imgs = []
    for _ in range(count):
        w = raw[p]
        h = raw[p + 1]
        rows = nibbles_to_rows(raw[p + 2:p + 2 + w * h], w, h)
        if mirror:
            rows = [list(reversed(r)) for r in rows]
        imgs.append((rows, w * 2, h))
        p += 2 + w * h
    return imgs


def find_file(root, names):
    """Case-insensitive recursive search for the first of names."""
    want = {n.lower() for n in names}
    for dirpath, _dirs, files in os.walk(root):
        for f in files:
            if f.lower() in want:
                return os.path.join(dirpath, f)
        # also match without extension variants is unnecessary here
    return None


def load_image(path):
    """Return the load image of a DOS MZ file, undoing EXEPACK if present."""
    d = open(path, "rb").read()
    img = unexepack(d)
    if img is not None:
        return img
    hdr = struct.unpack_from("<H", d, 8)[0] * 16
    return d[hdr:]


def unexepack(d):
    """Reproduce the EXEPACK stub of SKIEGA; None when not EXEPACK-packed."""
    if len(d) < 0x40 or d[:2] != b"MZ":
        return None
    (cblp, cp, crlc, cparhdr, minalloc, maxalloc, ss, sp, csum, ip, cs, lfarlc, ovno) = (
        struct.unpack_from("<13H", d, 2)
    )
    lm = d[cparhdr * 16:]
    base = cs * 16          # parameter block (see below)
    entry = base + ip       # entry point = the decompression stub
    # stub signature: mov ax,cs / add ax,0x10 / push cs / pop ds
    if ip != 0x12 or entry + 6 > len(lm) or lm[entry:entry + 6] != bytes(
            [0x8C, 0xC0, 0x05, 0x10, 0x00, 0x0E]):
        return None
    p = lambda o: struct.unpack_from("<H", lm, base + o)[0]
    copy_len = p(0x06)
    copy_para = p(0x0C)
    if base + copy_len != len(lm) or copy_para == 0:
        return None
    stub = lm[base:base + copy_len]
    # The stub's ES is one paragraph below CS (DX marker), so the stream
    # starts at lm[base - 0x10 + 0x0F] = lm[base - 1].
    pos = base - 0x10 + 0x0F                # backward control stream start
    for _ in range(0x10):
        if lm[pos] != 0xFF:
            break
        pos -= 1
    out = bytearray(copy_para * 16)
    di = len(out) - 1
    while True:
        control = lm[pos]
        count = lm[pos - 2] | (lm[pos - 1] << 8)
        kind = control & 0xFE
        if kind == 0xB0:
            fill = lm[pos - 3]
            for _ in range(count):
                out[di] = fill
                di -= 1
            pos -= 4
        elif kind == 0xB2:
            for k in range(count):
                out[di] = lm[pos - 3 - k]
                di -= 1
            pos -= 3 + count
        else:
            return None
        if control & 1:
            break
    # stored prefix is kept in place
    out[:pos + 1] = lm[:pos + 1]
    # relocation table inside the relocated stub
    si = 0x12D
    w = lambda o: struct.unpack_from("<H", stub, o)[0]
    dx = 0
    while True:
        cnt = w(si); si += 2
        while cnt:
            off = w(si); si += 2
            linear = dx * 16 + off
            if linear + 1 < len(out):
                v = struct.unpack_from("<H", out, linear)[0] & 0xFFFF
                struct.pack_into("<H", out, linear, v)
            cnt -= 1
        if dx == 0xF000:
            break
        dx += 0x1000
    return bytes(out)


FONT_GLYPHS = 96


def plausible_font(data):
    if len(data) < FONT_GLYPHS * 8:
        return False
    g = lambda c: data[(c - 0x20) * 8:(c - 0x20) * 8 + 8]
    return (all(b == 0 for b in g(0x20)) and any(g(ord("A"))) and any(g(ord("Z")))
            and any(g(ord("0"))) and any(g(ord("9"))) and any(g(ord("a"))) and any(g(ord("z"))))


def find_font(image):
    """Locate the executable's 8x8 font (DS:0x6728 in the analysed build).

    Primary: the glyph blitter contains `mov si,<font> / sub al,0x20 / mul bl`
    (function 0x17AE2). Fallback: the font blob signature (space, '!', '"').
    """
    pat = bytes([0xBE])
    tail = bytes([0x32, 0xE4, 0x2C, 0x20, 0xF6])
    for i in range(len(image) - 9):
        if image[i:i + 1] == pat and image[i + 3:i + 8] == tail:
            imm = struct.unpack_from("<H", image, i + 1)[0]
            for ds in (0x819, 0x1000, 0x1019):
                off = ds * 16 + imm
                if off + 8 < len(image) and plausible_font(image[off:]):
                    return off
    sig = bytes([0] * 8 + [0x30, 0x78, 0x30, 0x00, 0x30, 0x00, 0x00, 0x00, 0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    off = image.find(sig)
    return off if off >= 0 and plausible_font(image[off:]) else None


def write_ppm(path, sprite_rows, w, h):
    """Minimal preview writer that needs no third-party library."""
    with open(path, "wb") as fh:
        fh.write(b"P6\n%d %d\n255\n" % (w, h))
        buf = bytearray()
        for y in range(h):
            for x in range(w):
                buf += bytes(PALETTE[sprite_rows[y][x] & 0x0F])
        fh.write(buf)


def main(game_dir: str, out_dir: str):
    os.makedirs(out_dir, exist_ok=True)
    os.makedirs(os.path.join(out_dir, "audio"), exist_ok=True)
    read = lambda n: open(os.path.join(game_dir, n), "rb").read()

    # --- arena background -------------------------------------------------
    fort = list_bank(lzss(read("SNOWFORT.LZW")))
    assert len(fort) == 1, f"SNOWFORT entries: {len(fort)}"
    rows, w, h = fort[0]
    skin(rows, w, h).save(os.path.join(out_dir, "arena.png"))
    print(f"arena.png {w}x{h}")

    # --- SFFBM sprite bank (index 0..164) ---------------------------------
    sprites, sizes = bm_bank(lzss(read("SFFBM.LZW")))
    imgs = [to_rgba(r, s[0], s[1]) for r, s in zip(sprites, sizes)]
    sheet, rects = atlas(imgs, cols=16)
    sheet.save(os.path.join(out_dir, "sprites.png"))
    json.dump({"count": len(imgs), "sprites": rects,
               "pixel_size": [{"index": i, "w": sizes[i][0], "h": sizes[i][1]} for i in range(len(imgs))]},
              open(os.path.join(out_dir, "sprites.json"), "w"), indent=1)
    print(f"sprites.png {sheet.size} ({len(imgs)} sprites)")

    # --- SFSHP overlay bank (compass etc.) --------------------------------
    over = list_bank(lzss(read("SFSHP.LZW")))
    oimgs = [to_rgba(r, w2, h2) for r, w2, h2 in over]
    osheet, orects = atlas(oimgs, cols=6)
    osheet.save(os.path.join(out_dir, "overlay.png"))
    json.dump({"count": len(oimgs), "sprites": orects,
               "pixel_size": [{"index": i, "w": over[i][1], "h": over[i][2]} for i in range(len(over))]},
              open(os.path.join(out_dir, "overlay.json"), "w"), indent=1)
    print(f"overlay.png {osheet.size} ({len(oimgs)} sprites)")

    # --- audio data (byte exact) -----------------------------------------
    for name in ("SBTUNE.LZW", "TBTUNE.LZW", "DHTUNE.LZW", "SKI2.BIN",
                 "ADSKI.SMB", "MTSKI.SMB", "TNSKI.SMB", "PCSKI.SMB"):
        data = read(name)
        open(os.path.join(out_dir, "audio", name.lower()), "wb").write(data)
    print("audio/ written:", sorted(os.listdir(os.path.join(out_dir, "audio"))))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])

//! Original game graphics loaded from `assets/snowball.pack`.
//!
//! The pack is produced by `tools/extract/build_assets.py` from the shipped
//! Ski or Die data files (LZSS containers + the recovered image formats) and
//! the 8x8 font found in the executable at DS:0x6728. Format (little endian):
//!
//! ```text
//! "SOWA" u16 version
//! palette[16] rgb
//! u16 arena_w, arena_h, arena pixels (indices)
//! u16 n, n * (u16 x, y, w, h)          sprite atlas rectangles
//! u16 atlas_w, atlas_h, atlas pixels
//! u16 n, n * (u16 x, y, w, h)          overlay atlas rectangles
//! u16 overlay_w, overlay_h, overlay pixels
//! u16 first_char, count, glyph_w, glyph_h, glyph pixels (rows, MSB left)
//! ```

use std::path::Path;

pub const DEFAULT_PACK: &str = "assets/snowball.pack";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

impl Image {
    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.px[y * self.w + x]
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    pub first: u8,
    pub count: usize,
    pub w: usize,
    pub h: usize,
    pub glyphs: Vec<u8>, // count * w bytes, one byte per row (MSB = left)
}

impl Font {
    pub fn glyph(&self, c: char) -> Option<&[u8]> {
        let code = c as u32;
        if code < self.first as u32 || code >= self.first as u32 + self.count as u32 {
            return None;
        }
        let idx = (code - self.first as u32) as usize;
        Some(&self.glyphs[idx * self.w..(idx + 1) * self.w])
    }
}

#[derive(Debug, Clone)]
pub struct Assets {
    pub palette: [[u8; 3]; 16],
    pub arena: Image,
    pub sprites: Vec<Rect>,
    pub sprite_atlas: Image,
    pub overlay: Vec<Rect>,
    pub overlay_atlas: Image,
    pub font: Font,
}

struct Reader<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> u8 {
        let v = self.d[self.p];
        self.p += 1;
        v
    }
    fn u16(&mut self) -> u16 {
        let v = u16::from_le_bytes([self.d[self.p], self.d[self.p + 1]]);
        self.p += 2;
        v
    }
    fn take(&mut self, n: usize) -> &'a [u8] {
        let v = &self.d[self.p..self.p + n];
        self.p += n;
        v
    }
    fn rects(&mut self, n: usize) -> Vec<Rect> {
        (0..n)
            .map(|_| Rect {
                x: self.u16(),
                y: self.u16(),
                w: self.u16(),
                h: self.u16(),
            })
            .collect()
    }
}

impl Assets {
    pub fn load(path: impl AsRef<Path>) -> Result<Assets, String> {
        let bytes = std::fs::read(path.as_ref())
            .map_err(|e| format!("{}: {e}", path.as_ref().display()))?;
        Self::parse(&bytes)
    }

    /// Try the default pack, then a pack next to the executable.
    pub fn load_default() -> Result<Assets, String> {
        if let Ok(a) = Self::load(DEFAULT_PACK) {
            return Ok(a);
        }
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let cand = exe.parent().unwrap_or(Path::new(".")).join(DEFAULT_PACK);
        Self::load(cand)
    }

    pub fn parse(d: &[u8]) -> Result<Assets, String> {
        let mut r = Reader { d, p: 0 };
        if r.take(4) != b"SOWA" {
            return Err("bad asset pack magic".into());
        }
        let _version = r.u16();
        let mut palette = [[0u8; 3]; 16];
        for c in palette.iter_mut() {
            *c = [r.u8(), r.u8(), r.u8()];
        }
        let aw = r.u16() as usize;
        let ah = r.u16() as usize;
        let arena = Image {
            w: aw,
            h: ah,
            px: r.take(aw * ah).to_vec(),
        };
        let ns = r.u16() as usize;
        let sprites = r.rects(ns);
        let sw = r.u16() as usize;
        let sh = r.u16() as usize;
        let sprite_atlas = Image {
            w: sw,
            h: sh,
            px: r.take(sw * sh).to_vec(),
        };
        let no = r.u16() as usize;
        let overlay = r.rects(no);
        let ow = r.u16() as usize;
        let oh = r.u16() as usize;
        let overlay_atlas = Image {
            w: ow,
            h: oh,
            px: r.take(ow * oh).to_vec(),
        };
        let first = r.u8();
        let count = r.u16() as usize;
        let gw = r.u8() as usize;
        let gh = r.u8() as usize;
        let glyphs = r.take(count * gw).to_vec();
        Ok(Assets {
            palette,
            arena,
            sprites,
            sprite_atlas,
            overlay,
            overlay_atlas,
            font: Font {
                first,
                count,
                w: gw,
                h: gh,
                glyphs,
            },
        })
    }

    pub fn color(&self, index: u8) -> [u8; 3] {
        self.palette[(index & 0x0F) as usize]
    }

    pub fn sprite(&self, index: u8) -> Option<Rect> {
        self.sprites.get(index as usize).copied()
    }

    pub fn sprite_pixels(&self, index: u8) -> Option<&[u8]> {
        let r = self.sprite(index)?;
        Some(&self.sprite_atlas.px[r.y as usize * self.sprite_atlas.w + r.x as usize..])
    }

    pub fn overlay_sprite(&self, index: usize) -> Option<Rect> {
        self.overlay.get(index).copied()
    }
}

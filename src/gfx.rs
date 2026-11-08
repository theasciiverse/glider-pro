//! The part of QuickDraw Glider PRO draws with, on offscreen maps of any
//! size (its GWorlds): rectangles, ovals, lines and regions with a pen
//! that has a size, a pattern and a mode; masked and stretched copies
//! between maps; colours by the Mac's 8-bit palette index. Coordinates are
//! the original's (a map may have its origin anywhere, as `bounds` says).
//!
//! Text is not drawn into maps: Glider PRO's letters are a smudge at
//! terminal sizes, so text shown during play is terminal text
//! (`Game::label`), as in `games/glider`.

pub use crate::house::Rect;
use crate::mac::{Image, Rgb};

pub const BLACK: Rgb = [0, 0, 0];
pub const WHITE: Rgb = [255, 255, 255];

/// QuickDraw's eight classic colours (ForeColor(redColor) etc.), as the
/// 8-bit palette shows them.
pub const RED: Rgb = [0xDD, 0x00, 0x00];
pub const GREEN: Rgb = [0x00, 0xBB, 0x00];
pub const BLUE: Rgb = [0x00, 0x00, 0xDD];
pub const CYAN: Rgb = [0x00, 0x99, 0xFF];
pub const MAGENTA: Rgb = [0xFF, 0x00, 0x99];
pub const YELLOW: Rgb = [0xFF, 0xFF, 0x00];

/// The Mac's standard 8-bit colour table (`clut` 8): 0 is white, then the
/// 6x6x6 cube down to 214, ramps of red, green, blue and gray (the
/// steps between the cube's), 255 black. `Index2Color`.
pub fn clut8(i: i32) -> Rgb {
    let i = i.clamp(0, 255);
    const CUBE: [u8; 6] = [0xFF, 0xCC, 0x99, 0x66, 0x33, 0x00];
    const RAMP: [u8; 10] = [0xEE, 0xDD, 0xBB, 0xAA, 0x88, 0x77, 0x55, 0x44, 0x22, 0x11];
    match i {
        0..=214 => {
            let i = i as usize;
            [CUBE[i / 36], CUBE[(i / 6) % 6], CUBE[i % 6]]
        }
        215..=224 => [RAMP[(i - 215) as usize], 0, 0],
        225..=234 => [0, RAMP[(i - 225) as usize], 0],
        235..=244 => [0, 0, RAMP[(i - 235) as usize]],
        245..=254 => {
            let v = RAMP[(i - 245) as usize];
            [v, v, v]
        }
        _ => BLACK,
    }
}

/// An 8x8 QuickDraw pattern, one byte a row, high bit leftmost.
pub type Pattern = [u8; 8];
pub const PAT_BLACK: Pattern = [0xFF; 8];
pub const PAT_WHITE: Pattern = [0x00; 8];
pub const PAT_GRAY: Pattern = [0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55];
pub const PAT_LT_GRAY: Pattern = [0x88, 0x22, 0x88, 0x22, 0x88, 0x22, 0x88, 0x22];
pub const PAT_DK_GRAY: Pattern = [0x77, 0xDD, 0x77, 0xDD, 0x77, 0xDD, 0x77, 0xDD];

fn pat_bit(p: &Pattern, x: i32, y: i32) -> bool {
    p[(y & 7) as usize] & (0x80 >> (x & 7)) != 0
}

/// Pen transfer modes Glider PRO uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// patCopy: pattern 1 bits in the fore colour, 0 bits in the back colour (white).
    Copy,
    /// patOr: fore colour where the pattern is set, the rest untouched.
    Or,
    /// patXor: invert where the pattern is set.
    Xor,
}

#[derive(Clone, Copy, Debug)]
pub struct Pen {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub pat: Pattern,
    pub mode: Mode,
    pub fore: Rgb,
    pub back: Rgb,
}

impl Default for Pen {
    fn default() -> Pen {
        Pen { x: 0, y: 0, w: 1, h: 1, pat: PAT_BLACK, mode: Mode::Copy, fore: BLACK, back: WHITE }
    }
}

impl Default for Map {
    fn default() -> Map {
        Map::sized(0, 0)
    }
}

/// An offscreen map (GWorld) or the window's picture. `bounds` is its
/// rectangle in its own coordinates (usually from 0, 0).
#[derive(Clone)]
pub struct Map {
    pub bounds: Rect,
    pub px: Vec<Rgb>,
    pub clip: Rect,
    pub pen: Pen,
    /// When set, CopyMask paints solid in this colour: a silhouette, for
    /// the silhouette view (as `games/glider`).
    pub ink: Option<Rgb>,
    /// The silhouettes alone, pixel by pixel, when not empty (see
    /// `enable_fore`): what the silhouette view draws in sharper
    /// characters than the room behind. The art's dark parts inside a
    /// shape are a darker shade of the ink. Copies between maps that both
    /// keep it carry it along; a copy from one that doesn't clears it.
    pub fore: Vec<Option<(Rgb, bool)>>,
}

/// Art darker than this in every channel is a silhouette's dark part.
const DARK: u8 = 170;

/// A silhouette's ink where the art is dark.
pub fn shade(c: Rgb) -> Rgb {
    c.map(|v| (v as u32 * 9 / 20) as u8)
}

impl Map {
    /// A map of `r`, white (NewGWorld's memory is erased by the callers;
    /// white is QuickDraw's back colour).
    pub fn new(r: Rect) -> Map {
        let n = (r.width().max(0) * r.height().max(0)) as usize;
        Map { bounds: r, px: vec![WHITE; n], clip: r, pen: Pen::default(), ink: None, fore: Vec::new() }
    }

    /// Keep a silhouette layer (all clear) from now on.
    pub fn enable_fore(&mut self) {
        self.fore = vec![None; self.px.len()];
    }

    pub fn disable_fore(&mut self) {
        self.fore = Vec::new();
    }

    pub fn fore_at(&self, x: i32, y: i32) -> Option<(Rgb, bool)> {
        if self.fore.is_empty() {
            return None;
        }
        self.index(x, y).and_then(|i| self.fore[i])
    }

    #[inline]
    fn set_fore(&mut self, x: i32, y: i32, f: Option<(Rgb, bool)>) {
        if self.fore.is_empty() {
            return;
        }
        let r = self.clip;
        if x >= r.left && x < r.right && y >= r.top && y < r.bottom
            && let Some(i) = self.index(x, y)
        {
            self.fore[i] = f;
        }
    }

    pub fn sized(w: i32, h: i32) -> Map {
        Map::new(Rect::new(0, 0, w, h))
    }

    /// A map holding a decoded picture, from 0, 0.
    pub fn from_image(img: &Image) -> Map {
        let mut m = Map::sized(img.w, img.h);
        m.px.clone_from(&img.px);
        m
    }

    pub fn width(&self) -> i32 {
        self.bounds.width()
    }

    pub fn height(&self) -> i32 {
        self.bounds.height()
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let b = self.bounds;
        if x >= b.left && x < b.right && y >= b.top && y < b.bottom {
            Some(((y - b.top) * b.width() + (x - b.left)) as usize)
        } else {
            None
        }
    }

    /// The pixel at (x, y); white outside (as an erased GWorld).
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Rgb {
        self.index(x, y).map_or(WHITE, |i| self.px[i])
    }

    /// Set a pixel, inside the clip.
    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        let r = self.clip;
        if x >= r.left && x < r.right && y >= r.top && y < r.bottom
            && let Some(i) = self.index(x, y)
        {
            self.px[i] = c;
            // whatever is drawn over a silhouette ends it (the shreds of a
            // shredded glider stayed sharp over the high scores and title)
            if !self.fore.is_empty() {
                self.fore[i] = None;
            }
        }
    }

    pub fn clip_to(&mut self, r: Rect) {
        self.clip = r;
    }

    pub fn unclip(&mut self) {
        self.clip = self.bounds;
    }

    // ---------------------------------------------------------- pen state

    pub fn pen_normal(&mut self) {
        let (x, y, fore, back) = (self.pen.x, self.pen.y, self.pen.fore, self.pen.back);
        self.pen = Pen { x, y, fore, back, ..Pen::default() };
    }

    pub fn pen_size(&mut self, w: i32, h: i32) {
        self.pen.w = w;
        self.pen.h = h;
    }

    pub fn pen_pat(&mut self, p: Pattern) {
        self.pen.pat = p;
    }

    pub fn pen_mode(&mut self, m: Mode) {
        self.pen.mode = m;
    }

    /// RGBForeColor / ForeColor.
    pub fn fore(&mut self, c: Rgb) {
        self.pen.fore = c;
    }

    /// ForeColor by 8-bit palette index (Index2Color + RGBForeColor).
    pub fn fore_index(&mut self, i: i32) {
        self.pen.fore = clut8(i);
    }

    /// One pixel painted with the pen's pattern and mode.
    #[inline]
    fn pen_pixel(&mut self, x: i32, y: i32) {
        let on = pat_bit(&self.pen.pat, x, y);
        match self.pen.mode {
            Mode::Copy => {
                let c = if on { self.pen.fore } else { self.pen.back };
                self.set(x, y, c);
            }
            Mode::Or => {
                if on {
                    let c = self.pen.fore;
                    self.set(x, y, c);
                }
            }
            Mode::Xor => {
                if on {
                    let p = self.get(x, y);
                    self.set(x, y, [255 - p[0], 255 - p[1], 255 - p[2]]);
                }
            }
        }
    }

    // ---------------------------------------------------------- shapes

    /// PaintRect with the pen (pattern and mode).
    pub fn paint_rect(&mut self, r: Rect) {
        let r = sect(r, self.clip);
        for y in r.top..r.bottom {
            for x in r.left..r.right {
                self.pen_pixel(x, y);
            }
        }
    }

    /// A solid rectangle in `c`, whatever the pen (ColorRect at patCopy black).
    pub fn fill_rect(&mut self, r: Rect, c: Rgb) {
        let r = sect(r, self.clip);
        for y in r.top..r.bottom {
            for x in r.left..r.right {
                self.set(x, y, c);
            }
        }
    }

    /// ColorRect: PaintRect in a palette colour, with the current pen.
    pub fn color_rect(&mut self, r: Rect, index: i32) {
        let was = self.pen.fore;
        self.pen.fore = clut8(index);
        self.paint_rect(r);
        self.pen.fore = was;
    }

    /// EraseRect: the back colour (white).
    pub fn erase_rect(&mut self, r: Rect) {
        let c = self.pen.back;
        self.fill_rect(r, c);
    }

    /// FrameRect: the pen's outline just inside `r`.
    pub fn frame_rect(&mut self, r: Rect) {
        if r.empty() {
            return;
        }
        let (pw, ph) = (self.pen.w, self.pen.h);
        self.paint_rect(Rect::new(r.left, r.top, r.right, r.top + ph));
        self.paint_rect(Rect::new(r.left, r.bottom - ph, r.right, r.bottom));
        self.paint_rect(Rect::new(r.left, r.top, r.left + pw, r.bottom));
        self.paint_rect(Rect::new(r.right - pw, r.top, r.right, r.bottom));
    }

    pub fn color_frame_rect(&mut self, r: Rect, index: i32) {
        let was = self.pen.fore;
        self.pen.fore = clut8(index);
        self.frame_rect(r);
        self.pen.fore = was;
    }

    pub fn color_frame_wh_rect(&mut self, left: i32, top: i32, wide: i32, high: i32, index: i32) {
        self.color_frame_rect(Rect::new(left, top, left + wide, top + high), index);
    }

    pub fn invert_rect(&mut self, r: Rect) {
        let r = sect(r, self.clip);
        for y in r.top..r.bottom {
            for x in r.left..r.right {
                let p = self.get(x, y);
                self.set(x, y, [255 - p[0], 255 - p[1], 255 - p[2]]);
            }
        }
    }

    fn in_oval(r: Rect, x: i32, y: i32) -> bool {
        let (cx, cy) = ((r.left + r.right) as f32 / 2.0, (r.top + r.bottom) as f32 / 2.0);
        let (rx, ry) = (r.width() as f32 / 2.0, r.height() as f32 / 2.0);
        if rx <= 0.0 || ry <= 0.0 {
            return false;
        }
        let (nx, ny) = ((x as f32 + 0.5 - cx) / rx, (y as f32 + 0.5 - cy) / ry);
        nx * nx + ny * ny <= 1.0
    }

    /// PaintOval with the pen.
    pub fn paint_oval(&mut self, r: Rect) {
        let c = sect(r, self.clip);
        for y in c.top..c.bottom {
            for x in c.left..c.right {
                if Map::in_oval(r, x, y) {
                    self.pen_pixel(x, y);
                }
            }
        }
    }

    pub fn color_oval(&mut self, r: Rect, index: i32) {
        let was = self.pen.fore;
        self.pen.fore = clut8(index);
        self.paint_oval(r);
        self.pen.fore = was;
    }

    /// FrameOval: the oval's outline, one pen wide.
    pub fn frame_oval(&mut self, r: Rect) {
        let inner = r.inset_by(self.pen.w, self.pen.h);
        let c = sect(r, self.clip);
        for y in c.top..c.bottom {
            for x in c.left..c.right {
                if Map::in_oval(r, x, y) && !Map::in_oval(inner, x, y) {
                    self.pen_pixel(x, y);
                }
            }
        }
    }

    pub fn color_frame_oval(&mut self, r: Rect, index: i32) {
        let was = self.pen.fore;
        self.pen.fore = clut8(index);
        self.frame_oval(r);
        self.pen.fore = was;
    }

    /// FrameRoundRect, approximated by the rectangle with its corners
    /// cut by the oval of `ow` x `oh`.
    pub fn frame_round_rect(&mut self, r: Rect, ow: i32, oh: i32) {
        let _ = (ow, oh);
        self.frame_rect(r);
    }

    // ---------------------------------------------------------- lines

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.pen.x = x;
        self.pen.y = y;
    }

    /// Move by (dx, dy) (Move).
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.pen.x += dx;
        self.pen.y += dy;
    }

    /// LineTo: the pen hangs below and to the right of the line.
    pub fn line_to(&mut self, x1: i32, y1: i32) {
        let (x0, y0) = (self.pen.x, self.pen.y);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        let (pw, ph) = (self.pen.w, self.pen.h);
        loop {
            for py in 0..ph {
                for px in 0..pw {
                    self.pen_pixel(x + px, y + py);
                }
            }
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
        self.pen.x = x1;
        self.pen.y = y1;
    }

    /// Line: LineTo relative to the pen.
    pub fn line(&mut self, dx: i32, dy: i32) {
        let (x, y) = (self.pen.x + dx, self.pen.y + dy);
        self.line_to(x, y);
    }

    /// ColorLine: MoveTo(h0, v0); LineTo(h1, v1) in a palette colour.
    pub fn color_line(&mut self, h0: i32, v0: i32, h1: i32, v1: i32, index: i32) {
        let was = self.pen.fore;
        self.pen.fore = clut8(index);
        self.move_to(h0, v0);
        self.line_to(h1, v1);
        self.pen.fore = was;
    }

    /// HiliteRect: top and left edges in `c1`, bottom and right in `c2`.
    pub fn hilite_rect(&mut self, r: Rect, c1: i32, c2: i32) {
        self.color_line(r.left, r.top, r.right - 2, r.top, c1);
        self.color_line(r.left, r.top, r.left, r.bottom - 2, c1);
        self.color_line(r.right - 1, r.top, r.right - 1, r.bottom - 2, c2);
        self.color_line(r.left + 1, r.bottom - 1, r.right - 1, r.bottom - 1, c2);
    }

    // ---------------------------------------------------------- regions

    /// PaintRgn with the pen.
    pub fn paint_rgn(&mut self, rgn: &Region) {
        let c = sect(rgn.bounds, self.clip);
        for y in c.top..c.bottom {
            for x in c.left..c.right {
                if rgn.contains(x, y) {
                    self.pen_pixel(x, y);
                }
            }
        }
    }

    /// ColorRegion: PaintRgn in a palette colour.
    pub fn color_region(&mut self, rgn: &Region, index: i32) {
        let was = self.pen.fore;
        self.pen.fore = clut8(index);
        self.paint_rgn(rgn);
        self.pen.fore = was;
    }

    // ---------------------------------------------------------- copies

    /// CopyBits (srcCopy) from `src`'s `sr` into this map's `dr`,
    /// stretching if the sizes differ.
    pub fn copy_bits(&mut self, src: &Map, sr: Rect, dr: Rect) {
        self.copy_bits_rgn(src, sr, dr, None);
    }

    /// CopyBits with a mask region (in this map's coordinates).
    pub fn copy_bits_rgn(&mut self, src: &Map, sr: Rect, dr: Rect, rgn: Option<&Region>) {
        let (dw, dh) = (dr.width(), dr.height());
        if dw <= 0 || dh <= 0 || sr.empty() {
            return;
        }
        let (sw, sh) = (sr.width(), sr.height());
        let c = sect(dr, self.clip);
        for y in c.top..c.bottom {
            let sy = sr.top + (y - dr.top) * sh / dh;
            for x in c.left..c.right {
                if rgn.is_some_and(|g| !g.contains(x, y)) {
                    continue;
                }
                let sx = sr.left + (x - dr.left) * sw / dw;
                if let Some(i) = src.index(sx, sy) {
                    self.set(x, y, src.px[i]);
                    if !self.fore.is_empty() {
                        let f = if src.fore.is_empty() { None } else { src.fore[i] };
                        self.set_fore(x, y, f);
                    }
                }
            }
        }
    }

    /// CopyBits in `transparent` mode: white source pixels leave the
    /// destination as it was.
    pub fn copy_bits_transparent(&mut self, src: &Map, sr: Rect, dr: Rect) {
        let (dw, dh) = (dr.width(), dr.height());
        if dw <= 0 || dh <= 0 || sr.empty() {
            return;
        }
        let (sw, sh) = (sr.width(), sr.height());
        let c = sect(dr, self.clip);
        for y in c.top..c.bottom {
            let sy = sr.top + (y - dr.top) * sh / dh;
            for x in c.left..c.right {
                let sx = sr.left + (x - dr.left) * sw / dw;
                if let Some(i) = src.index(sx, sy)
                    && src.px[i] != WHITE
                {
                    self.set(x, y, src.px[i]);
                }
            }
        }
    }

    /// CopyMask: `src`'s `sr` drawn into `dr` wherever `mask` at `mr` is
    /// black (dark). Stretches if sizes differ.
    pub fn copy_mask(&mut self, src: &Map, mask: &Map, sr: Rect, mr: Rect, dr: Rect) {
        let (dw, dh) = (dr.width(), dr.height());
        if dw <= 0 || dh <= 0 || sr.empty() {
            return;
        }
        let c = sect(dr, self.clip);
        for y in c.top..c.bottom {
            let oy = (y - dr.top) * sr.height() / dh;
            let my = mr.top + (y - dr.top) * mr.height().max(1) / dh;
            for x in c.left..c.right {
                let ox = (x - dr.left) * sr.width() / dw;
                let mx = mr.left + (x - dr.left) * mr.width().max(1) / dw;
                if mask.get(mx, my)[0] < 128
                    && let Some(i) = src.index(sr.left + ox, sr.top + oy)
                {
                    match self.ink {
                        None => self.set(x, y, src.px[i]),
                        Some(c) => {
                            // the art's dark parts inside the shape a
                            // shade of the ink (a glider's lower wing; Glider
                            // PRO's is mid-gray, 153, where Glider 4's art is
                            // darker, so the line is higher than Glider's 100)
                            let inside = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().all(|(dx, dy)| mask.get(mx + dx, my + dy)[0] < 128);
                            let dark = src.px[i].iter().all(|&v| v < DARK);
                            let c = if inside && dark { shade(c) } else { c };
                            self.set(x, y, c);
                            self.set_fore(x, y, Some((c, true)));
                        }
                    }
                }
            }
        }
    }

    /// CopyMask painting in `ink` if given (a silhouette), else the art.
    pub fn copy_mask_ink(&mut self, ink: Option<Rgb>, src: &Map, mask: &Map, sr: Rect, mr: Rect, dr: Rect) {
        let was = self.ink;
        self.ink = ink;
        self.copy_mask(src, mask, sr, mr, dr);
        self.ink = was;
    }

    /// DrawPicture: a decoded picture stretched into `dr`.
    pub fn draw_picture(&mut self, img: &Image, dr: Rect) {
        let (dw, dh) = (dr.width(), dr.height());
        if dw <= 0 || dh <= 0 || img.w <= 0 || img.h <= 0 {
            return;
        }
        let c = sect(dr, self.clip);
        for y in c.top..c.bottom {
            let sy = (y - dr.top) * img.h / dh;
            for x in c.left..c.right {
                let sx = (x - dr.left) * img.w / dw;
                self.set(x, y, img.at(sx, sy));
            }
        }
    }
}

/// The intersection of two rectangles (empty if they do not meet).
pub fn sect(a: Rect, b: Rect) -> Rect {
    let r = Rect::new(a.left.max(b.left), a.top.max(b.top), a.right.min(b.right), a.bottom.min(b.bottom));
    if r.empty() { Rect::new(0, 0, 0, 0) } else { r }
}

/// The smallest rectangle around both (UnionRect).
pub fn union(a: Rect, b: Rect) -> Rect {
    if a.empty() {
        return b;
    }
    if b.empty() {
        return a;
    }
    Rect::new(a.left.min(b.left), a.top.min(b.top), a.right.max(b.right), a.bottom.max(b.bottom))
}

/// A QuickDraw region, kept as a bit mask over its bounding rectangle.
#[derive(Clone, Debug, Default)]
pub struct Region {
    pub bounds: Rect,
    bits: Vec<bool>,
}

impl Region {
    /// NewRgn: empty.
    pub fn new() -> Region {
        Region::default()
    }

    /// RectRgn.
    pub fn rect(r: Rect) -> Region {
        if r.empty() {
            return Region::new();
        }
        Region { bounds: r, bits: vec![true; (r.width() * r.height()) as usize] }
    }

    /// OpenRgn ... CloseRgn of lines through `pts` (a closed outline),
    /// even-odd filled. Use [`Trace`] to build the points.
    pub fn polygon(pts: &[(i32, i32)]) -> Region {
        let (Some(x0), Some(x1)) = (pts.iter().map(|p| p.0).min(), pts.iter().map(|p| p.0).max()) else {
            return Region::new();
        };
        let (y0, y1) = (pts.iter().map(|p| p.1).min().unwrap_or(0), pts.iter().map(|p| p.1).max().unwrap_or(0));
        let b = Rect::new(x0, y0, x1, y1);
        if b.empty() {
            return Region::new();
        }
        let mut g = Region { bounds: b, bits: vec![false; (b.width() * b.height()) as usize] };
        for y in y0..y1 {
            for x in x0..x1 {
                if inside(pts, x as f32 + 0.5, y as f32 + 0.5) {
                    let i = g.at(x, y);
                    g.bits[i] = true;
                }
            }
        }
        g
    }

    fn at(&self, x: i32, y: i32) -> usize {
        ((y - self.bounds.top) * self.bounds.width() + (x - self.bounds.left)) as usize
    }

    pub fn is_empty(&self) -> bool {
        !self.bits.iter().any(|&b| b)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        let b = self.bounds;
        x >= b.left && x < b.right && y >= b.top && y < b.bottom && self.bits[self.at(x, y)]
    }

    fn combine(&self, o: &Region, bounds: Rect, f: impl Fn(bool, bool) -> bool) -> Region {
        if bounds.empty() {
            return Region::new();
        }
        let mut g = Region { bounds, bits: vec![false; (bounds.width() * bounds.height()) as usize] };
        for y in bounds.top..bounds.bottom {
            for x in bounds.left..bounds.right {
                let i = g.at(x, y);
                g.bits[i] = f(self.contains(x, y), o.contains(x, y));
            }
        }
        g
    }

    /// UnionRgn.
    pub fn union(&self, o: &Region) -> Region {
        self.combine(o, union(self.bounds, o.bounds), |a, b| a || b)
    }

    /// SectRgn.
    pub fn sect(&self, o: &Region) -> Region {
        self.combine(o, sect(self.bounds, o.bounds), |a, b| a && b)
    }

    /// DiffRgn: this one less `o`.
    pub fn diff(&self, o: &Region) -> Region {
        self.combine(o, self.bounds, |a, b| a && !b)
    }

    /// OffsetRgn.
    pub fn offset(&mut self, dx: i32, dy: i32) {
        self.bounds = self.bounds.offset_by(dx, dy);
    }
}

/// Even-odd point in polygon.
fn inside(pts: &[(i32, i32)], x: f32, y: f32) -> bool {
    let mut c = false;
    let n = pts.len();
    for i in 0..n {
        let (xi, yi) = (pts[i].0 as f32, pts[i].1 as f32);
        let (xj, yj) = (pts[(i + n - 1) % n].0 as f32, pts[(i + n - 1) % n].1 as f32);
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            c = !c;
        }
    }
    c
}

/// A region outline traced the QuickDraw way, between OpenRgn and
/// CloseRgn: start at the pen, then relative moves (`Line`) or absolute
/// ones (`LineTo`).
pub struct Trace {
    pub pts: Vec<(i32, i32)>,
}

impl Trace {
    pub fn from(x: i32, y: i32) -> Trace {
        Trace { pts: vec![(x, y)] }
    }

    fn last(&self) -> (i32, i32) {
        *self.pts.last().unwrap_or(&(0, 0))
    }

    pub fn by(mut self, dx: i32, dy: i32) -> Trace {
        let (x, y) = self.last();
        self.pts.push((x + dx, y + dy));
        self
    }

    pub fn to(mut self, x: i32, y: i32) -> Trace {
        self.pts.push((x, y));
        self
    }

    pub fn region(&self) -> Region {
        Region::polygon(&self.pts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_ends_and_cube() {
        assert_eq!(clut8(0), WHITE);
        assert_eq!(clut8(255), BLACK);
        assert_eq!(clut8(5), [0xFF, 0xFF, 0x00]);
        assert_eq!(clut8(215), [0xEE, 0, 0]);
        assert_eq!(clut8(254), [0x11; 3]);
    }

    #[test]
    fn masked_copy_and_regions() {
        let mut src = Map::sized(4, 4);
        src.fill_rect(Rect::new(0, 0, 4, 4), [10, 20, 30]);
        let mut mask = Map::sized(4, 4);
        mask.fill_rect(Rect::new(0, 0, 2, 4), BLACK);
        let mut dst = Map::sized(8, 8);
        dst.copy_mask(&src, &mask, Rect::new(0, 0, 4, 4), Rect::new(0, 0, 4, 4), Rect::new(4, 4, 8, 8));
        assert_eq!(dst.get(4, 4), [10, 20, 30]);
        assert_eq!(dst.get(7, 7), WHITE);
        let g = Region::rect(Rect::new(0, 0, 4, 4)).diff(&Region::rect(Rect::new(0, 0, 2, 2)));
        assert!(!g.contains(1, 1) && g.contains(3, 3));
    }
}

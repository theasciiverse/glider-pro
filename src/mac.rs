//! The classic Mac formats Glider 4.0 ships in, read directly so the game
//! uses John Calhoun's own files: MacBinary (the art files), resource
//! forks, DeRez text (`Glider.r`, `snd.r`), QuickDraw pictures (PICT 1 and
//! 2), sampled sounds (`snd `) and bitmap fonts (`FONT`).

use std::collections::HashMap;

pub type Rgb = [u8; 3];

/// Resources by (type, id).
pub type Resources = HashMap<([u8; 4], i16), Vec<u8>>;

fn be16(d: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([d[at], d[at + 1]])
}

fn be32(d: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([d[at], d[at + 1], d[at + 2], d[at + 3]])
}

/// The resource fork of a MacBinary file.

/// Every resource in a resource fork.
pub fn resource_fork(rf: &[u8]) -> Resources {
    let mut out = HashMap::new();
    let data_off = be32(rf, 0) as usize;
    let map_off = be32(rf, 4) as usize;
    let m = &rf[map_off..];
    let tl = be16(m, 24) as usize;
    let types = be16(m, tl) as usize + 1;
    for t in 0..types {
        let e = tl + 2 + t * 8;
        let kind = [m[e], m[e + 1], m[e + 2], m[e + 3]];
        let count = be16(m, e + 4) as usize + 1;
        let refs = tl + be16(m, e + 6) as usize;
        for r in 0..count {
            let re = refs + r * 12;
            let id = be16(m, re) as i16;
            let at = data_off + (be32(m, re + 4) & 0xFF_FFFF) as usize;
            let len = be32(rf, at) as usize;
            out.insert((kind, id), rf[at + 4..at + 4 + len].to_vec());
        }
    }
    out
}

/// Every resource in DeRez text: `data 'TYPE' (id, ...) {` then lines of
/// `$"0A1B ..."`, ending `};`. Line ends may be CR or LF.
pub fn derez(text: &[u8]) -> Resources {
    let mut out = HashMap::new();
    let mut cur: Option<(([u8; 4], i16), Vec<u8>)> = None;
    for line in text.split(|&b| b == b'\r' || b == b'\n') {
        if line.starts_with(b"data '") && line.len() > 12 {
            let kind = [line[6], line[7], line[8], line[9]];
            let rest = &line[11..];
            let open = rest.iter().position(|&b| b == b'(').map_or(0, |p| p + 1);
            let num: String = rest[open..].iter().take_while(|&&b| b == b'-' || b.is_ascii_digit()).map(|&b| b as char).collect();
            if let Ok(id) = num.parse::<i16>() {
                cur = Some(((kind, id), Vec::new()));
            }
        } else if line.starts_with(b"};") {
            if let Some((k, v)) = cur.take() {
                out.insert(k, v);
            }
        } else if let Some((_, v)) = cur.as_mut() {
            let Some(s) = line.windows(2).position(|w| w == b"$\"") else { continue };
            let hex = &line[s + 2..];
            let end = hex.iter().position(|&b| b == b'"').unwrap_or(hex.len());
            let digits: Vec<u8> = hex[..end].iter().filter(|b| b.is_ascii_hexdigit()).copied().collect();
            for pair in digits.chunks(2) {
                if pair.len() == 2 {
                    let h = |c: u8| (c as char).to_digit(16).unwrap_or(0) as u8;
                    v.push(h(pair[0]) << 4 | h(pair[1]));
                }
            }
        }
    }
    out
}

/// A decoded picture, white where nothing was drawn.
#[derive(Clone)]
pub struct Image {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Rgb>,
}

impl Image {
    pub fn new(w: i32, h: i32, c: Rgb) -> Image {
        Image { w, h, px: vec![c; (w * h) as usize] }
    }

    pub fn at(&self, x: i32, y: i32) -> Rgb {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return [255; 3];
        }
        self.px[(y * self.w + x) as usize]
    }
}

fn unpack_bits(src: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < src.len() {
        let n = src[i] as usize;
        i += 1;
        if n < 128 {
            let end = (i + n + 1).min(src.len());
            out.extend_from_slice(&src[i..end]);
            i = end;
        } else if n > 128 {
            if let Some(&b) = src.get(i) {
                out.extend(std::iter::repeat_n(b, 257 - n));
            }
            i += 1;
        }
    }
}

/// A QuickDraw picture, version 1 or 2, as far as Glider's art goes: bit
/// and pixel maps (packed or not), clip regions and comments. Anything
/// else is an error.
pub fn pict(v: &[u8]) -> Result<Image, String> {
    let top = be16(v, 2) as i16 as i32;
    let left = be16(v, 4) as i16 as i32;
    let bottom = be16(v, 6) as i16 as i32;
    let right = be16(v, 8) as i16 as i32;
    let mut img = Image::new(right - left, bottom - top, [255; 3]);
    let mut p = 10;
    let v1 = v[p..p + 2] == [0x11, 0x01];
    if v1 {
        p += 2;
    } else if v[p..p + 4] == [0x00, 0x11, 0x02, 0xFF] {
        p += 4;
    } else {
        return Err("not a PICT".into());
    }
    let short = |p: usize| be16(v, p) as i16 as i32;
    while p < v.len() {
        if !v1 && p & 1 == 1 {
            p += 1;
        }
        let op = if v1 {
            p += 1;
            v[p - 1] as u16
        } else {
            p += 2;
            be16(v, p - 2)
        };
        match op {
            0x0000 | 0x001E => {}
            0x0C00 => p += 24,
            0x0001 => p += be16(v, p) as usize,
            0x00A0 => p += 2,
            // RGBFgCol, RGBBkCol, OpColor: a colour, not needed
            0x001A | 0x001B | 0x001F => p += 6,
            0x009A => p = direct_bits(v, p, &mut img, left, top)?,
            0x00A1 => p += 4 + be16(v, p + 2) as usize,
            0x00FF => break,
            0x0090 | 0x0098 | 0x0099 => {
                let rb_raw = be16(v, p);
                p += 2;
                let pixmap = rb_raw & 0x8000 != 0;
                let row_bytes = (rb_raw & 0x3FFF) as usize;
                let (bt, bl, bb, br) = (short(p), short(p + 2), short(p + 4), short(p + 6));
                p += 8;
                let mut pal: HashMap<u16, Rgb> = HashMap::new();
                let depth;
                if pixmap {
                    p += 2 + 2 + 4 + 4 + 4; // version, packType, packSize, hRes, vRes
                    depth = be16(v, p + 2) as usize; // after pixelType
                    p += 8 + 12; // pixelType..cmpSize, planeBytes, pmTable, pmReserved
                    p += 4; // ctSeed
                    let flags = be16(v, p);
                    let n = be16(v, p + 2) as usize + 1;
                    p += 4;
                    for i in 0..n {
                        let idx = be16(v, p);
                        let c = [v[p + 2], v[p + 4], v[p + 6]];
                        pal.insert(if flags & 0x8000 != 0 { i as u16 } else { idx }, c);
                        p += 8;
                    }
                } else {
                    depth = 1;
                    pal.insert(0, [255; 3]);
                    pal.insert(1, [0; 3]);
                }
                let (st, sl, sb, sr) = (short(p), short(p + 2), short(p + 4), short(p + 6));
                let (dt, dl, db, dr) = (short(p + 8), short(p + 10), short(p + 12), short(p + 14));
                p += 18; // src, dst, mode
                if op == 0x0099 {
                    p += be16(v, p) as usize; // mask region, not needed
                }
                let mut line = Vec::with_capacity(row_bytes);
                let rows = (bb - bt) as usize;
                for y in 0..rows {
                    line.clear();
                    if op == 0x0090 || row_bytes < 8 {
                        line.extend_from_slice(&v[p..p + row_bytes]);
                        p += row_bytes;
                    } else {
                        let n = if row_bytes > 250 {
                            p += 2;
                            be16(v, p - 2) as usize
                        } else {
                            p += 1;
                            v[p - 1] as usize
                        };
                        unpack_bits(&v[p..p + n], &mut line);
                        p += n;
                    }
                    let sy = bt + y as i32;
                    if sy < st || sy >= sb {
                        continue;
                    }
                    let per_byte = 8 / depth;
                    let mask = (1u16 << depth) - 1;
                    for x in 0..(br - bl) {
                        let sx = bl + x;
                        if sx < sl || sx >= sr {
                            continue;
                        }
                        let Some(&byte) = line.get(x as usize / per_byte) else { continue };
                        let shift = 8 - depth * (x as usize % per_byte + 1);
                        let idx = (byte as u16 >> shift) & mask;
                        // source rectangle to destination, scaled as DrawPicture
                        // would only when they differ, which Glider's never do
                        let (ox, oy) = (dl + (sx - sl) * (dr - dl) / (sr - sl), dt + (sy - st) * (db - dt) / (sb - st));
                        let (ix, iy) = (ox - left, oy - top);
                        if ix >= 0 && iy >= 0 && ix < img.w && iy < img.h {
                            img.px[(iy * img.w + ix) as usize] = *pal.get(&idx).unwrap_or(&[255, 0, 255]);
                        }
                    }
                }
            }
            _ => return Err(format!("PICT opcode {op:#06x} at {p}")),
        }
    }
    Ok(img)
}

/// DirectBitsRect: 16- or 32-bit pixels, packed per row (16-bit as runs of
/// words, packType 3; 32-bit as runs of bytes over planes, packType 4, or
/// plain RGB, packType 2). Returns where the next opcode starts.
fn direct_bits(v: &[u8], mut p: usize, img: &mut Image, left: i32, top: i32) -> Result<usize, String> {
    let short = |p: usize| be16(v, p) as i16 as i32;
    p += 4; // baseAddr
    let row_bytes = (be16(v, p) & 0x3FFF) as usize;
    let (bt, bl, bb, br) = (short(p + 2), short(p + 4), short(p + 6), short(p + 8));
    let pack_type = be16(v, p + 12);
    let pixel_size = be16(v, p + 30);
    let cmp_count = be16(v, p + 32) as usize;
    p += 46;
    let (st, sl, sb, sr) = (short(p), short(p + 2), short(p + 4), short(p + 6));
    let (dt, dl, db, dr) = (short(p + 8), short(p + 10), short(p + 12), short(p + 14));
    p += 18;
    let width = (br - bl) as usize;
    let mut line = Vec::with_capacity(row_bytes);
    for y in 0..(bb - bt) {
        line.clear();
        let packed = row_bytes >= 8 && pack_type != 1;
        if !packed {
            line.extend_from_slice(v.get(p..p + row_bytes).ok_or("short pixel data")?);
            p += row_bytes;
        } else {
            let n = if row_bytes > 250 {
                p += 2;
                be16(v, p - 2) as usize
            } else {
                p += 1;
                v[p - 1] as usize
            };
            let src = v.get(p..p + n).ok_or("short pixel data")?;
            p += n;
            if pixel_size == 16 && pack_type == 3 {
                // runs of 16-bit words
                let mut i = 0;
                while i < src.len() {
                    let c = src[i] as usize;
                    i += 1;
                    if c < 128 {
                        let end = (i + (c + 1) * 2).min(src.len());
                        line.extend_from_slice(&src[i..end]);
                        i = end;
                    } else if c > 128 && i + 1 < src.len() {
                        for _ in 0..(257 - c) {
                            line.extend_from_slice(&src[i..i + 2]);
                        }
                        i += 2;
                    }
                }
            } else if pack_type == 2 {
                line.extend_from_slice(src);
            } else {
                unpack_bits(src, &mut line);
            }
        }
        let sy = bt + y;
        if sy < st || sy >= sb {
            continue;
        }
        for x in 0..width {
            let rgb: Rgb = match (pixel_size, pack_type) {
                (16, _) => {
                    let Some(w) = line.get(x * 2..x * 2 + 2) else { continue };
                    let w = u16::from_be_bytes([w[0], w[1]]);
                    let c5 = |s: u16| (((w >> s) & 31) * 255 / 31) as u8;
                    [c5(10), c5(5), c5(0)]
                }
                (_, 2) => {
                    let Some(c) = line.get(x * 3..x * 3 + 3) else { continue };
                    [c[0], c[1], c[2]]
                }
                (_, 4) | (_, 0) if packed => {
                    // planes of `width` bytes: alpha first when there are four
                    let skip = if cmp_count >= 4 { width } else { 0 };
                    let at = |k: usize| line.get(skip + k * width + x).copied().unwrap_or(0);
                    [at(0), at(1), at(2)]
                }
                _ => {
                    let Some(c) = line.get(x * 4..x * 4 + 4) else { continue };
                    [c[1], c[2], c[3]]
                }
            };
            let sx = bl + x as i32;
            if sx < sl || sx >= sr {
                continue;
            }
            let (ox, oy) = (dl + (sx - sl) * (dr - dl) / (sr - sl).max(1), dt + (sy - st) * (db - dt) / (sb - st).max(1));
            let (ix, iy) = (ox - left, oy - top);
            if ix >= 0 && iy >= 0 && ix < img.w && iy < img.h {
                img.px[(iy * img.w + ix) as usize] = rgb;
            }
        }
    }
    Ok(p)
}

/// BinHex 4.0: the file's data and resource forks.
pub fn binhex(text: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    const ALPHABET: &[u8] = b"!\"#$%&'()*+,-012345689@ABCDEFGHIJKLMNPQRSTUVXYZ[`abcdefhijklmpqr";
    let start = text.windows(42).position(|w| w.starts_with(b"(This file must be converted with BinHex"))?;
    let body = &text[start..];
    let colon = body.iter().position(|&b| b == b':')? + 1;
    // six bits a character, to the closing colon
    let mut bits: u32 = 0;
    let mut nbits = 0;
    let mut raw = Vec::new();
    for &c in &body[colon..] {
        if c == b':' {
            break;
        }
        let Some(v) = ALPHABET.iter().position(|&a| a == c) else { continue };
        bits = bits << 6 | v as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            raw.push((bits >> nbits) as u8);
            bits &= (1 << nbits) - 1;
        }
    }
    // run-length: 0x90 n repeats the last byte n - 1 more times (0x90 0 is 0x90)
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let b = raw[i];
        if b == 0x90 && i + 1 < raw.len() {
            let n = raw[i + 1];
            if n == 0 {
                out.push(0x90);
            } else if let Some(&last) = out.last() {
                out.extend(std::iter::repeat_n(last, n as usize - 1));
            }
            i += 2;
        } else {
            out.push(b);
            i += 1;
        }
    }
    let nlen = *out.first()? as usize;
    let mut p = 1 + nlen + 1 + 4 + 4 + 2;
    let dlen = be32(&out, p) as usize;
    let rlen = be32(&out, p + 4) as usize;
    p += 8 + 2;
    let data = out.get(p..p + dlen)?.to_vec();
    p += dlen + 2;
    let rsrc = out.get(p..p + rlen)?.to_vec();
    Some((data, rsrc))
}

/// A sampled sound, 8-bit unsigned mono.
#[derive(Clone)]
pub struct Sound {
    pub rate: u32,
    pub data: Vec<u8>,
}

/// An `snd ` resource of format 1 or 2 holding one standard sound header.
pub fn snd(v: &[u8]) -> Option<Sound> {
    let mut p = 0;
    let format = be16(v, 0);
    p += 2;
    if format == 1 {
        let mods = be16(v, p) as usize;
        p += 2 + mods * 6;
    } else {
        p += 2; // refCount
    }
    let cmds = be16(v, p) as usize;
    p += 2;
    for c in 0..cmds {
        let at = p + c * 8;
        let cmd = be16(v, at) & 0x7FFF;
        if cmd == 0x51 || cmd == 0x50 {
            let h = be32(v, at + 4) as usize;
            let len = be32(v, h + 4) as usize;
            let rate = be32(v, h + 8) >> 16;
            let encode = v[h + 20];
            if encode != 0 {
                return None;
            }
            let start = h + 22;
            let data = v.get(start..start + len)?.to_vec();
            return Some(Sound { rate, data });
        }
    }
    None
}

/// A Pascal string from a byte slice whose first byte is the length.
pub fn pstring(b: &[u8]) -> Vec<u8> {
    let n = (b[0] as usize).min(b.len() - 1);
    b[1..1 + n].to_vec()
}

/// MacRoman to UTF-8, for names shown in the terminal.
pub fn mac_roman(b: &[u8]) -> String {
    const HIGH: &str = "ÄÅÇÉÑÖÜáàâäãåçéèêëíìîïñóòôöõúùûü†°¢£§•¶ß®©™´¨≠ÆØ∞±≤≥¥µ∂∑∏π∫ªºΩæø¿¡¬√ƒ≈∆«»… ÀÃÕŒœ\u{2013}\u{2014}“”‘’÷◊ÿŸ⁄€‹›ﬁﬂ‡·‚„‰ÂÊÁËÈÍÎÏÌÓÔ\u{F8FF}ÒÚÛÙıˆ˜¯˘˙˚¸˝˛ˇ";
    let high: Vec<char> = HIGH.chars().collect();
    b.iter().map(|&c| if c < 128 { c as char } else { high.get(c as usize - 128).copied().unwrap_or('?') }).collect()
}

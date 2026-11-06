//! Glider PRO house files (`houseType` in GliderStructs.h): the data fork
//! holds the house, big-endian, 866 bytes then 348 a room; the resource
//! fork holds the house's own pictures and sounds. The houses that come
//! with Glider PRO are BinHex files in its repository and built in.

use crate::mac::{self, Resources, pstring};

pub const HEADER: usize = 866;
pub const ROOM: usize = 348;
pub const MAX_ROOM_OBS: usize = 24;
pub const NUM_TILES: usize = 8;
pub const TILE_WIDE: i32 = 64;
pub const TILE_HIGH: i32 = 322;
pub const ROOM_WIDE: i32 = TILE_WIDE * NUM_TILES as i32;
/// No room there (an empty spot of the house's grid).
pub const ROOM_IS_EMPTY: i32 = -1;

/// A QuickDraw rectangle, in SetRect's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect { left, top, right, bottom }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn offset_by(self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.left + dx, self.top + dy, self.right + dx, self.bottom + dy)
    }

    pub fn inset_by(self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.left + dx, self.top + dy, self.right - dx, self.bottom - dy)
    }

    /// The same size, moved so its top left is at (x, y).
    pub fn at(self, x: i32, y: i32) -> Rect {
        self.offset_by(x - self.left, y - self.top)
    }

    pub fn empty(&self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }

    pub fn sect(&self, o: &Rect) -> bool {
        self.left.max(o.left) < self.right.min(o.right) && self.top.max(o.top) < self.bottom.min(o.bottom)
    }
}

/// An object's ten bytes of data, read as the union member its kind uses
/// (blowerType, furnitureType, ... clutterType). The fields keep the C
/// names; which mean what depends on the kind.
#[derive(Clone, Copy, Debug, Default)]
pub struct Object {
    pub what: i32,
    /// The raw ten bytes, for the kinds' own readings below.
    pub data: [u8; 10],
}

fn be16(d: &[u8], at: usize) -> i32 {
    i16::from_be_bytes([d[at], d[at + 1]]) as i32
}

fn be32(d: &[u8], at: usize) -> i32 {
    i32::from_be_bytes([d[at], d[at + 1], d[at + 2], d[at + 3]])
}

impl Object {
    /// `topLeft`, a Mac Point (v, h), of every kind but furniture and clutter.
    pub fn top_left(&self) -> (i32, i32) {
        (be16(&self.data, 2), be16(&self.data, 0))
    }

    /// `bounds` of furniture and clutter (top, left, bottom, right).
    pub fn bounds(&self) -> Rect {
        let d = &self.data;
        Rect::new(be16(d, 2), be16(d, 0), be16(d, 6), be16(d, 4))
    }

    /// The 16-bit field after topLeft (distance, length, tall, delay, height).
    pub fn short4(&self) -> i32 {
        be16(&self.data, 4)
    }

    /// The 16-bit field at byte 6 (points, where).
    pub fn short6(&self) -> i32 {
        be16(&self.data, 6)
    }

    /// `pict` of furniture and clutter.
    pub fn pict(&self) -> i32 {
        be16(&self.data, 8)
    }

    pub fn byte(&self, i: usize) -> u8 {
        self.data[i]
    }

    /// `initial` and `state` sit in different places by kind; callers pick.
    pub fn flag(&self, i: usize) -> bool {
        self.data[i] != 0
    }
}

/// A Mac `Point` (v, h stored in that order).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Point {
    pub h: i32,
    pub v: i32,
}

impl Point {
    pub const fn new(h: i32, v: i32) -> Point {
        Point { h, v }
    }
}

fn get_pt(d: &[u8; 10], at: usize) -> Point {
    Point { v: be16(d, at), h: be16(d, at + 2) }
}
fn put16(d: &mut [u8; 10], at: usize, v: i32) {
    d[at..at + 2].copy_from_slice(&(v as i16).to_be_bytes());
}
fn put_pt(d: &mut [u8; 10], at: usize, p: Point) {
    put16(d, at, p.v);
    put16(d, at + 2, p.h);
}
fn get_rect(d: &[u8; 10]) -> Rect {
    Rect::new(be16(d, 2), be16(d, 0), be16(d, 6), be16(d, 4))
}
fn put_rect(d: &mut [u8; 10], r: Rect) {
    put16(d, 0, r.top);
    put16(d, 2, r.left);
    put16(d, 4, r.bottom);
    put16(d, 6, r.right);
}

/// The union members of `objectType.data` (GliderStructs.h), each read
/// from and written back to the object's ten bytes: `obj.a().state`,
/// `obj.set_a(a)`. Field names are the C ones.
macro_rules! union_view {
    ($get:ident, $set:ident, $T:ident { $($f:ident : $ty:tt @ $at:expr),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct $T { $(pub $f: union_view!(@ty $ty)),* }
        impl Object {
            pub fn $get(&self) -> $T {
                let d = &self.data;
                $T { $($f: union_view!(@get $ty, d, $at)),* }
            }
            pub fn $set(&mut self, v: $T) {
                let d = &mut self.data;
                $(union_view!(@put $ty, d, $at, v.$f);)*
            }
        }
    };
    (@ty point) => { Point };
    (@ty rect) => { Rect };
    (@ty short) => { i32 };
    (@ty byte) => { i32 };
    (@ty bool) => { bool };
    (@get point, $d:ident, $at:expr) => { get_pt($d, $at) };
    (@get rect, $d:ident, $at:expr) => { get_rect($d) };
    (@get short, $d:ident, $at:expr) => { be16($d, $at) };
    (@get byte, $d:ident, $at:expr) => { $d[$at] as i32 };
    (@get bool, $d:ident, $at:expr) => { $d[$at] != 0 };
    (@put point, $d:ident, $at:expr, $v:expr) => { put_pt($d, $at, $v) };
    (@put rect, $d:ident, $at:expr, $v:expr) => { put_rect($d, $v) };
    (@put short, $d:ident, $at:expr, $v:expr) => { put16($d, $at, $v) };
    (@put byte, $d:ident, $at:expr, $v:expr) => { $d[$at] = $v as u8 };
    (@put bool, $d:ident, $at:expr, $v:expr) => { $d[$at] = $v as u8 };
}

union_view!(a, set_a, BlowerType { top_left: point @ 0, distance: short @ 4, initial: bool @ 6, state: bool @ 7, vector: byte @ 8, tall: byte @ 9 });
union_view!(b, set_b, FurnitureType { bounds: rect @ 0, pict: short @ 8 });
union_view!(c, set_c, BonusType { top_left: point @ 0, length: short @ 4, points: short @ 6, state: bool @ 8, initial: bool @ 9 });
union_view!(d, set_d, TransportType { top_left: point @ 0, tall: short @ 4, where_: short @ 6, who: byte @ 8, wide: byte @ 9 });
union_view!(e, set_e, SwitchType { top_left: point @ 0, delay: short @ 4, where_: short @ 6, who: byte @ 8, type_: byte @ 9 });
union_view!(f, set_f, LightType { top_left: point @ 0, length: short @ 4, byte0: byte @ 6, byte1: byte @ 7, initial: bool @ 8, state: bool @ 9 });
union_view!(g, set_g, ApplianceType { top_left: point @ 0, height: short @ 4, byte0: byte @ 6, delay: byte @ 7, initial: bool @ 8, state: bool @ 9 });
union_view!(h, set_h, EnemyType { top_left: point @ 0, length: short @ 4, delay: byte @ 6, byte0: byte @ 7, initial: bool @ 8, state: bool @ 9 });
union_view!(i, set_i, ClutterType { bounds: rect @ 0, pict: short @ 8 });

#[derive(Clone, Debug, Default)]
pub struct Room {
    pub name: Vec<u8>,
    pub bounds: i32,
    pub left_start: u8,
    pub right_start: u8,
    pub visited: bool,
    pub background: i32,
    pub tiles: [i32; NUM_TILES],
    pub floor: i32,
    pub suite: i32,
    pub openings: i32,
    pub num_objects: i32,
    pub objects: [Object; MAX_ROOM_OBS],
}

/// `scoresType` as stored in the house file (292 bytes at offset 528):
/// Pascal strings left as MacRoman bytes.
#[derive(Clone, Debug, Default)]
pub struct HouseScores {
    pub banner: Vec<u8>,
    pub names: [Vec<u8>; 10],
    pub scores: [i32; 10],
    pub time_stamps: [u32; 10],
    pub levels: [i32; 10],
}

impl HouseScores {
    fn parse(d: &[u8]) -> HouseScores {
        let mut s = HouseScores { banner: pstring(&d[0..32]), ..Default::default() };
        for i in 0..10 {
            s.names[i] = pstring(&d[32 + i * 16..48 + i * 16]);
            s.scores[i] = be32(d, 192 + i * 4);
            s.time_stamps[i] = be32(d, 232 + i * 4) as u32;
            s.levels[i] = be16(d, 272 + i * 2);
        }
        s
    }
}

#[derive(Clone, Debug, Default)]
pub struct House {
    pub version: i32,
    /// `timeStamp`; bit 0 set: the house is locked (ReadHouse).
    pub time_stamp: i32,
    pub flags: i32,
    /// `initial`: where the glider starts in the first room.
    pub initial: Point,
    pub banner: Vec<u8>,
    pub trailer: Vec<u8>,
    /// `highScores`, as stored in the house.
    pub high_scores: HouseScores,
    pub has_game: bool,
    pub first_room: i32,
    pub rooms: Vec<Room>,
    /// The house's own pictures and sounds (its resource fork).
    pub res: Resources,
}

impl House {
    pub fn parse(data: &[u8], rsrc: &[u8]) -> Option<House> {
        if data.len() < HEADER {
            return None;
        }
        let n = be16(data, 864).max(0) as usize;
        let mut rooms = Vec::with_capacity(n);
        for r in 0..n {
            let o = HEADER + r * ROOM;
            let Some(d) = data.get(o..o + ROOM) else { break };
            let mut room = Room {
                name: pstring(&d[0..28]),
                bounds: be16(d, 28),
                left_start: d[30],
                right_start: d[31],
                visited: d[33] != 0,
                background: be16(d, 34),
                tiles: [0; NUM_TILES],
                floor: be16(d, 52),
                suite: be16(d, 54),
                openings: be16(d, 56),
                num_objects: be16(d, 58).clamp(0, MAX_ROOM_OBS as i32),
                objects: [Object::default(); MAX_ROOM_OBS],
            };
            for (i, t) in room.tiles.iter_mut().enumerate() {
                *t = be16(d, 36 + i * 2).clamp(0, 7);
            }
            for (i, ob) in room.objects.iter_mut().enumerate() {
                let q = 60 + i * 12;
                let mut data = [0u8; 10];
                data.copy_from_slice(&d[q + 2..q + 12]);
                *ob = Object { what: be16(d, q), data };
            }
            rooms.push(room);
        }
        let res = if rsrc.len() > 16 { std::panic::catch_unwind(|| mac::resource_fork(rsrc)).unwrap_or_default() } else { Resources::new() };
        Some(House {
            version: be16(data, 0),
            time_stamp: be32(data, 4),
            flags: be32(data, 8),
            initial: Point { v: be16(data, 12), h: be16(data, 14) },
            banner: pstring(&data[16..272]),
            trailer: pstring(&data[272..528]),
            high_scores: HouseScores::parse(&data[528..820]),
            has_game: data[860] != 0,
            first_room: be16(data, 862),
            rooms,
            res,
        })
    }

    /// The room on `floor` in `suite`, if the house has one there.
    pub fn room_at(&self, floor: i32, suite: i32) -> Option<usize> {
        self.rooms.iter().position(|r| r.suite != ROOM_IS_EMPTY && r.floor == floor && r.suite == suite)
    }
}

macro_rules! houses {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_bytes!(concat!("../Houses/", $name, ".binhex")) as &[u8])),*]
    };
}

/// The houses in the Glider PRO repository, as BinHex. Sampler is left
/// out: its resource fork is damaged (and Aerofoil leaves it out too).
pub const BUILT_IN: &[(&str, &[u8])] = houses![
    "Demo House",
    "Art Museum",
    "CD Demo House",
    "California or Bust!",
    "Castle o' the Air",
    "Davis Station",
    "Empty House",
    "Fun House",
    "Grand Prix",
    "ImagineHouse PRO II",
    "In The Mirror",
    "Land of Illusion",
    "Leviathan",
    "Metropolis",
    "Nemo's Market",
    "Rainbow's End",
    "Slumberland",
    "SpacePods",
    "Teddy World",
    "The Asylum Pro",
    "Titanic",
];

pub fn load(name: &str) -> Option<House> {
    let (_, text) = BUILT_IN.iter().find(|(n, _)| *n == name)?;
    let (data, rsrc) = mac::binhex(text)?;
    House::parse(&data, &rsrc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_built_in_house_reads() {
        for (n, _) in BUILT_IN {
            let h = load(n).unwrap_or_else(|| panic!("{n}"));
            assert!(!h.rooms.is_empty(), "{n}");
            assert!(h.rooms.iter().any(|r| r.suite != ROOM_IS_EMPTY), "{n}: no rooms in use");
        }
    }
}

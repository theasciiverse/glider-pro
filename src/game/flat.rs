//! The silhouette view's furniture, as `games/glider` draws it: the
//! furniture Calhoun drew with QuickDraw (tables, deck tables, shelves,
//! cabinets, counters, dressers, mirrors) flat in wood and glass tones,
//! and its straight edges recorded for the view to draw over the picture
//! in box-drawing characters (`view::draw_edges`). Only the room on
//! screen; the original view draws Calhoun's furniture.

use super::*;
use crate::mac::Rgb;

/// Furniture tones, as Glider's.
pub const WOOD_INK: Rgb = [150, 100, 55];
pub const SHELF_INK: Rgb = [185, 145, 90];
pub const FRAME_INK: Rgb = [125, 82, 45];
pub const GLASS_INK: Rgb = [150, 175, 190];

/// A straight edge of the house, in room coordinates (0..512, 0..322).
#[derive(Clone, Copy, Debug)]
pub enum Edge {
    /// A horizontal bar (table top, shelf) along the middle of the rect.
    Bar(Rect),
    /// A vertical post (a table's leg) along the middle of the rect.
    Post(Rect),
    /// An outline (cabinet, counter, dresser, mirror).
    Outline(Rect),
}

/// An edge's colour: its furniture's, lighter.
fn light(c: Rgb) -> Rgb {
    [c[0].saturating_add(60), c[1].saturating_add(55), c[2].saturating_add(45)]
}

// (ObjectDraw.c's)
const TABLE_BASE_TOP: i32 = 296;
const BRACKET_INSET: i32 = 18;
const SHELF_DEEP: i32 = 4;
const BRACKET_THICK: i32 = 5;

impl Game {
    /// In the silhouette view, draw `what` at `r` (back map coordinates)
    /// flat and record its edges; false where the original drawing stays
    /// (another view, a room around, or not furniture).
    pub fn flat_furniture(&mut self, what: i32, neighbor: i32, r: &Rect) -> bool {
        if !self.silhouette || neighbor != CENTRAL_ROOM {
            return false;
        }
        let r = *r;
        let room = self.local_rooms_dest[CENTRAL_ROOM as usize];
        let rel = |q: Rect| q.offset_by(-room.left, -room.top);
        let down = self.play_origin_v;
        match what {
            TABLE | DECK_TABLE => {
                self.back_src_map.fill_rect(r, WOOD_INK);
                let h_center = (r.left + r.right) / 2;
                let base = TABLE_BASE_TOP + down;
                if r.bottom < base {
                    let post = Rect::new(h_center - 3, r.bottom, h_center + 2, base);
                    self.back_src_map.fill_rect(post, WOOD_INK);
                    self.room_edges.push((Edge::Post(rel(post)), light(WOOD_INK)));
                }
                let src = if what == TABLE { self.table_src } else { self.deck_src };
                let mut foot = src;
                q_offset_rect(&mut foot, -half_rect_wide(&src) + r.left + half_rect_wide(&r), base);
                self.back_src_map.copy_mask_ink(Some(WOOD_INK), &self.furniture_src_map, &self.furniture_mask_map, src, src, foot);
                self.room_edges.push((Edge::Bar(rel(r)), light(WOOD_INK)));
            }
            SHELF => {
                self.back_src_map.fill_rect(r, SHELF_INK);
                let src = self.shelf_src;
                for left in [r.left + BRACKET_INSET, r.right - BRACKET_INSET - SHELF_DEEP - BRACKET_THICK] {
                    let mut b = src;
                    zero_rect_corner(&mut b);
                    q_offset_rect(&mut b, left, r.bottom);
                    self.back_src_map.copy_mask_ink(Some(SHELF_INK), &self.furniture_src_map, &self.furniture_mask_map, src, src, b);
                }
                self.room_edges.push((Edge::Bar(rel(r)), light(SHELF_INK)));
            }
            CABINET | COUNTER | DRESSER => {
                self.back_src_map.fill_rect(r, WOOD_INK);
                self.room_edges.push((Edge::Outline(rel(r)), light(WOOD_INK)));
            }
            MIRROR => {
                self.back_src_map.fill_rect(r, FRAME_INK);
                self.back_src_map.fill_rect(r.inset_by(3, 3), GLASS_INK);
                self.room_edges.push((Edge::Outline(rel(r)), light(FRAME_INK)));
            }
            _ => return false,
        }
        true
    }

    /// The room on screen drawn again for another view (V): its background
    /// and objects as RedrawRoomLighting draws them, without starting its
    /// flames, enemies and the rest over.
    pub fn redraw_for_view(&mut self) {
        if !self.playing || self.this_room_number < 0 || self.this_room_number as usize >= self.house.rooms.len() {
            return;
        }
        let room_v = self.house.rooms[self.this_room_number as usize].floor;
        self.room_edges.clear();
        self.draw_room_background(self.local_numbers[CENTRAL_ROOM as usize], CENTRAL_ROOM, room_v);
        self.room_backdrop = Some(self.back_src_map.clone());
        self.backdrop_serial += 1;
        self.draw_a_rooms_objects(CENTRAL_ROOM, true);
        self.draw_lighting();
        self.restore_work_map();
        let r = self.local_rooms_dest[CENTRAL_ROOM as usize];
        self.add_rect_to_work_rects(&r);
    }
}

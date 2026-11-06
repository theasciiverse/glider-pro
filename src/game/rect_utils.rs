//! RectUtils.c and the Toolbox's rectangle calls, as free functions with
//! the C names in snake_case. A `Rect *` the C changes is `&mut Rect`.

use crate::gfx::Rect;
use crate::house::Point;

pub fn q_set_rect(r: &mut Rect, l: i32, t: i32, rt: i32, b: i32) {
    *r = Rect::new(l, t, rt, b);
}

/// SetRect (same as QSetRect).
pub fn set_rect(r: &mut Rect, l: i32, t: i32, rt: i32, b: i32) {
    q_set_rect(r, l, t, rt, b);
}

pub fn q_offset_rect(r: &mut Rect, h: i32, v: i32) {
    *r = r.offset_by(h, v);
}

/// OffsetRect (same as QOffsetRect).
pub fn offset_rect(r: &mut Rect, h: i32, v: i32) {
    q_offset_rect(r, h, v);
}

pub fn h_offset_rect(r: &mut Rect, h: i32) {
    r.left += h;
    r.right += h;
}

pub fn v_offset_rect(r: &mut Rect, v: i32) {
    r.top += v;
    r.bottom += v;
}

/// InsetRect.
pub fn inset_rect(r: &mut Rect, h: i32, v: i32) {
    *r = r.inset_by(h, v);
}

pub fn rect_wide(r: &Rect) -> i32 {
    r.right - r.left
}

pub fn rect_tall(r: &Rect) -> i32 {
    r.bottom - r.top
}

pub fn half_rect_wide(r: &Rect) -> i32 {
    (r.right - r.left) / 2
}

pub fn half_rect_tall(r: &Rect) -> i32 {
    (r.bottom - r.top) / 2
}

pub fn zero_rect_corner(r: &mut Rect) {
    r.right -= r.left;
    r.bottom -= r.top;
    r.left = 0;
    r.top = 0;
}

pub fn center_rect_on_point(r: &mut Rect, where_: Point) {
    zero_rect_corner(r);
    let (hw, ht) = (half_rect_wide(r), half_rect_tall(r));
    q_offset_rect(r, -hw, -ht);
    q_offset_rect(r, where_.h, where_.v);
}

pub fn center_rect_in_rect(a: &mut Rect, b: &Rect) {
    let (w, t) = (rect_wide(a), rect_tall(a));
    a.left = b.left + (rect_wide(b) - w) / 2;
    a.right = a.left + w;
    a.top = b.top + (rect_tall(b) - t) / 2;
    a.bottom = a.top + t;
}

pub fn normalize_rect(r: &mut Rect) {
    if r.left > r.right {
        std::mem::swap(&mut r.left, &mut r.right);
    }
    if r.top > r.bottom {
        std::mem::swap(&mut r.top, &mut r.bottom);
    }
}

pub fn is_rect_left_of_rect(r1: &Rect, r2: &Rect) -> bool {
    let offset = (r1.right - r1.left) - (r2.right - r2.left) / 2;
    r1.left < r2.left + offset
}

pub fn force_rect_in_rect(small: &mut Rect, large: &Rect) -> bool {
    let mut changed = false;
    normalize_rect(small);
    if small.bottom - small.top > large.bottom - large.top {
        small.bottom = small.top + (large.bottom - large.top);
        changed = true;
    }
    if small.right - small.left > large.right - large.left {
        small.right = small.left + (large.right - large.left);
        changed = true;
    }
    let h = large.left - small.left;
    if h > 0 {
        offset_rect(small, h, 0);
        changed = true;
    }
    let h = large.right - small.right;
    if h < 0 {
        offset_rect(small, h, 0);
        changed = true;
    }
    let v = large.top - small.top;
    if v > 0 {
        offset_rect(small, 0, v);
        changed = true;
    }
    let v = large.bottom - small.bottom;
    if v < 0 {
        offset_rect(small, 0, v);
        changed = true;
    }
    changed
}

/// QUnionSimilarRect: `c` = the bounds of `a` and `b`.
pub fn q_union_similar_rect(a: &Rect, b: &Rect, c: &mut Rect) {
    c.left = a.left.min(b.left);
    c.top = a.top.min(b.top);
    c.right = a.right.max(b.right);
    c.bottom = a.bottom.max(b.bottom);
}

/// UnionRect.
pub fn union_rect(a: &Rect, b: &Rect, c: &mut Rect) {
    *c = crate::gfx::union(*a, *b);
}

/// SectRect: the intersection into `c`; true if not empty.
pub fn sect_rect(a: &Rect, b: &Rect, c: &mut Rect) -> bool {
    *c = crate::gfx::sect(*a, *b);
    !c.empty()
}

/// Do they overlap? (SectRect with the result thrown away.)
pub fn rects_meet(a: &Rect, b: &Rect) -> bool {
    a.sect(b)
}

/// PtInRect.
pub fn pt_in_rect(p: Point, r: &Rect) -> bool {
    p.h >= r.left && p.h < r.right && p.v >= r.top && p.v < r.bottom
}

/// EmptyRect.
pub fn empty_rect(r: &Rect) -> bool {
    r.empty()
}

/// EqualRect.
pub fn equal_rect(a: &Rect, b: &Rect) -> bool {
    a == b
}

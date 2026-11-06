//! ObjectDraw.c: drawing blowers, furniture and prizes into the back map.

use super::*;
use crate::gfx::{self, Mode, PAT_GRAY, Trace};

pub const K8_WHITE_COLOR: i32 = 0;
pub const YELLOW_COLOR: i32 = 5;
pub const GOLD_COLOR: i32 = 11;
pub const K8_RED_COLOR: i32 = 35;
pub const PALE_VIOLET_COLOR: i32 = 42;
pub const K8_LT_TAN_COLOR: i32 = 52;
pub const K8_BAMBOO_COLOR: i32 = 53;
pub const DARK_FLESH_COLOR: i32 = 58;
pub const K8_TAN_COLOR: i32 = 94;
pub const K8_PISS_YELLOW_COLOR: i32 = 95;
pub const K8_ORANGE_COLOR: i32 = 59;
pub const K8_BROWN_COLOR: i32 = 137;
pub const K8_RED4_COLOR: i32 = 143;
pub const K8_SKY_COLOR: i32 = 150;
pub const K8_EARTH_BLUE_COLOR: i32 = 170;
pub const K8_DK_RED_COLOR: i32 = 222;
pub const K8_DK_RED2_COLOR: i32 = 223;
pub const INTENSE_GREEN_COLOR: i32 = 225;
pub const INTENSE_BLUE_COLOR: i32 = 235;
pub const K8_PUMPKIN_COLOR: i32 = 101;
pub const K8_LTST_GRAY_COLOR: i32 = 245;
pub const K8_LTST_GRAY2_COLOR: i32 = 246;
pub const K8_LTST_GRAY3_COLOR: i32 = 43;
pub const K8_LTST_GRAY4_COLOR: i32 = 247;
pub const K8_LTST_GRAY5_COLOR: i32 = 248;
pub const K8_LT_GRAY_COLOR: i32 = 249;
pub const K8_GRAY_COLOR: i32 = 250;
pub const K8_GRAY2_COLOR: i32 = 251;
pub const K8_DK_GRAY_COLOR: i32 = 252;
pub const K8_DK_GRAY2_COLOR: i32 = 253;
pub const K8_DK_GRAY3_COLOR: i32 = 172;
pub const K8_DKST_GRAY_COLOR: i32 = 254;
pub const K8_BLACK_COLOR: i32 = 255;

const TIKI_POLE_BASE: i32 = 300;
const TABLE_BASE_TOP: i32 = 296;
const TABLE_SHADOW_TOP: i32 = 312;
const TABLE_SHADOW_OFFSET: i32 = 12;
const BRACKET_INSET: i32 = 18;
const SHELF_DEEP: i32 = 4;
const BRACKET_THICK: i32 = 5;
const SHELF_SHADOW_OFF: i32 = 12;
const CABINET_DEEP: i32 = 4;
const CABINET_SHADOW_OFF: i32 = 6;
const COUNTER_FOOTER_HIGH: i32 = 12;
const COUNTER_STRIP_WIDE: i32 = 6;
const COUNTER_STRIP_TALL: i32 = 29;
const COUNTER_PANEL_DROP: i32 = 12;
const DRESSER_TOP_THICK: i32 = 4;
const DRESSER_CREASE: i32 = 9;
const DRESSER_DRAWER_DROP: i32 = 12;
const DRESSER_SIDE_SPARE: i32 = 14;
const STOOL_BASE: i32 = 304;

/// GetTime's DateTimeRec, the fields Glider PRO reads.
#[derive(Clone, Copy, Debug, Default)]
pub struct DateTimeRec {
    pub year: i32,
    /// 1 to 12.
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
}

/// GetTime: the local date and time. The UTC offset is read from the
/// system's time zone file (TZif, `$TZ` or /etc/localtime); UTC if none.
pub fn get_time() -> DateTimeRec {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let t = now + utc_offset(now);
    let days = t.div_euclid(86400);
    let secs = t.rem_euclid(86400);
    // Days to civil date (Howard Hinnant's algorithm).
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    DateTimeRec { year: y as i32, month: m as i32, day: d as i32, hour: (secs / 3600) as i32, minute: ((secs / 60) % 60) as i32 }
}

fn utc_offset(now: i64) -> i64 {
    let path = match std::env::var("TZ") {
        Ok(tz) if !tz.is_empty() => {
            let tz = tz.trim_start_matches(':').to_string();
            if tz.starts_with('/') { tz } else { format!("/usr/share/zoneinfo/{tz}") }
        }
        _ => "/etc/localtime".to_string(),
    };
    let Ok(d) = std::fs::read(path) else { return 0 };
    tzif_offset(&d, now).unwrap_or(0)
}

/// The UTC offset at `now` from a TZif file's version 1 data.
fn tzif_offset(d: &[u8], now: i64) -> Option<i64> {
    if d.len() < 44 || &d[0..4] != b"TZif" {
        return None;
    }
    let n = |i: usize| u32::from_be_bytes([d[20 + i * 4], d[21 + i * 4], d[22 + i * 4], d[23 + i * 4]]) as usize;
    let (timecnt, typecnt) = (n(3), n(4));
    let times = 44;
    let idx = times + timecnt * 4;
    let types = idx + timecnt;
    if d.len() < types + typecnt * 6 || typecnt == 0 {
        return None;
    }
    let mut ty = 0usize;
    for i in 0..timecnt {
        let at = times + i * 4;
        let t = i32::from_be_bytes([d[at], d[at + 1], d[at + 2], d[at + 3]]) as i64;
        if t <= now {
            ty = d[idx + i] as usize;
        } else {
            break;
        }
    }
    if ty >= typecnt {
        return None;
    }
    let at = types + ty * 6;
    Some(i32::from_be_bytes([d[at], d[at + 1], d[at + 2], d[at + 3]]) as i64)
}

impl Game {
    //--------------------------------------------------------------  DrawSimpleBlowers

    pub fn draw_simple_blowers(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.blower_src_map, &self.blower_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawTiki

    pub fn draw_tiki(&mut self, the_rect: &Rect, down: i32) {
        let dark_gray_c = K8_DKST_GRAY_COLOR;
        let light_wood_c = K8_BAMBOO_COLOR;
        let dark_wood_c = K8_PISS_YELLOW_COLOR;
        let m = &mut self.back_src_map;

        if the_rect.bottom < TIKI_POLE_BASE + down {
            m.color_line(the_rect.left + 11, the_rect.bottom - 1, the_rect.left + 11, TIKI_POLE_BASE + down - 1, dark_gray_c);
            m.color_line(the_rect.left + 12, the_rect.bottom - 1, the_rect.left + 12, TIKI_POLE_BASE + down, light_wood_c);
            m.color_line(the_rect.left + 13, the_rect.bottom - 1, the_rect.left + 13, TIKI_POLE_BASE + down, dark_wood_c);
            m.color_line(the_rect.left + 14, the_rect.bottom - 1, the_rect.left + 14, TIKI_POLE_BASE + down, dark_wood_c);
            m.color_line(the_rect.left + 15, the_rect.bottom - 1, the_rect.left + 15, TIKI_POLE_BASE + down - 1, dark_gray_c);
        }

        let sr = self.src_rects[TIKI as usize];
        self.back_src_map.copy_mask(&self.blower_src_map, &self.blower_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawInvisibleBlower

    pub fn draw_invisible_blower(&mut self, the_rect: &Rect) {
        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, 0, 0, 24, 24);
        q_offset_rect(&mut temp_rect, the_rect.left, the_rect.top);
        self.back_src_map.color_frame_rect(temp_rect, 192);
    }

    //--------------------------------------------------------------  DrawLiftArea

    pub fn draw_lift_area(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, 192);
    }

    //--------------------------------------------------------------  DrawTable

    pub fn draw_table(&mut self, table_top: &Rect, down: i32) {
        let brown_c = K8_BROWN_COLOR;
        let tan_c = K8_TAN_COLOR;
        let dk_red_c = K8_DK_RED2_COLOR;
        let black_c = K8_BLACK_COLOR;
        let mut table_top = *table_top;
        let m = &mut self.back_src_map;

        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, table_top.left, 0, table_top.right, rect_wide(&table_top) / 10);
        let ht = half_rect_tall(&temp_rect);
        q_offset_rect(&mut temp_rect, 0, -ht + TABLE_SHADOW_TOP + down);
        q_offset_rect(&mut temp_rect, TABLE_SHADOW_OFFSET, -TABLE_SHADOW_OFFSET);
        m.pen_pat(PAT_GRAY);
        m.pen_mode(Mode::Or);
        m.color_oval(temp_rect, K8_DKST_GRAY_COLOR);
        m.pen_normal();

        inset_rect(&mut table_top, 0, 1);
        m.color_rect(table_top, brown_c);
        inset_rect(&mut table_top, 0, -1);

        let t = table_top;
        m.color_line(t.left, t.top + 1, t.left, t.top + 1, K8_WHITE_COLOR);
        m.color_line(t.left + 1, t.top, t.right - 2, t.top, K8_WHITE_COLOR);
        m.color_line(t.right - 1, t.top + 1, t.right - 1, t.top + 1, K8_WHITE_COLOR);

        m.color_line(t.left + 1, t.top + 1, t.right - 2, t.top + 1, tan_c);
        m.color_line(t.left, t.top + 2, t.left, t.bottom - 2, tan_c);

        m.color_line(t.left + 1, t.bottom - 1, t.right - 2, t.bottom - 1, black_c);
        m.color_line(t.right - 1, t.top + 2, t.right - 1, t.bottom - 2, black_c);

        m.color_line(t.left + 1, t.bottom - 2, t.right - 2, t.bottom - 2, dk_red_c);

        if t.bottom < TABLE_BASE_TOP + down {
            let h_center = (t.left + t.right) / 2;

            m.color_line(h_center - 3, t.bottom, h_center - 3, TABLE_BASE_TOP + down, black_c);
            m.color_line(h_center - 2, t.bottom, h_center - 2, TABLE_BASE_TOP + down, K8_LT_GRAY_COLOR);
            m.color_line(h_center - 1, t.bottom, h_center - 1, TABLE_BASE_TOP + down, K8_GRAY_COLOR);
            m.color_line(h_center, t.bottom, h_center, TABLE_BASE_TOP + down, K8_DK_GRAY_COLOR);
            m.color_line(h_center + 1, t.bottom, h_center + 1, TABLE_BASE_TOP + down, black_c);

            let v_shadow = t.bottom + rect_wide(&t) / 4 - 2;
            if v_shadow > TABLE_BASE_TOP + down {
                m.color_line(h_center - 2, t.bottom, h_center - 2, TABLE_BASE_TOP + down, K8_DK_GRAY_COLOR);
                m.color_line(h_center - 1, t.bottom, h_center - 1, TABLE_BASE_TOP + down, K8_DK_GRAY_COLOR);
                m.color_line(h_center, t.bottom, h_center, TABLE_BASE_TOP + down, black_c);
            } else {
                m.color_line(h_center - 2, t.bottom, h_center - 2, v_shadow, K8_DK_GRAY_COLOR);
                m.color_line(h_center - 1, t.bottom, h_center - 1, v_shadow + 1, K8_DK_GRAY_COLOR);
                m.color_line(h_center, t.bottom, h_center, v_shadow + 2, black_c);
            }
        }

        let table_src = self.table_src;
        let mut temp_rect = table_src;
        q_offset_rect(&mut temp_rect, -half_rect_wide(&table_src) + t.left + half_rect_wide(&t), TABLE_BASE_TOP + down);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, table_src, table_src, temp_rect);
    }

    //--------------------------------------------------------------  DrawShelf

    pub fn draw_shelf(&mut self, shelf_top: &Rect) {
        let brown_c = K8_BROWN_COLOR;
        let lt_tan_c = K8_LT_TAN_COLOR;
        let tan_c = K8_TAN_COLOR;
        let dk_red_c = K8_DK_RED2_COLOR;
        let black_c = K8_BLACK_COLOR;
        let mut shelf_top = *shelf_top;
        let m = &mut self.back_src_map;

        let shadow_rgn = Trace::from(shelf_top.left, shelf_top.bottom)
            .by(SHELF_SHADOW_OFF, SHELF_SHADOW_OFF)
            .by(rect_wide(&shelf_top) - SHELF_DEEP, 0)
            .by(0, -SHELF_THICK + 1)
            .by(-SHELF_SHADOW_OFF, -SHELF_SHADOW_OFF)
            .to(shelf_top.left, shelf_top.bottom)
            .region();
        m.pen_pat(PAT_GRAY);
        m.pen_mode(Mode::Or);
        m.color_region(&shadow_rgn, K8_DKST_GRAY_COLOR);
        m.pen_normal();

        inset_rect(&mut shelf_top, 0, 1);
        m.color_rect(shelf_top, brown_c);
        inset_rect(&mut shelf_top, 0, -1);

        let s = shelf_top;
        m.color_line(s.left + 1, s.top, s.left + 1 + SHELF_DEEP, s.top, lt_tan_c);
        m.color_line(s.left, s.top + 1, s.left + SHELF_DEEP, s.top + 1, tan_c);
        m.color_line(s.left, s.top + 2, s.left + SHELF_DEEP, s.top + 2, tan_c);
        m.color_line(s.left, s.top + 3, s.left + SHELF_DEEP, s.top + 3, tan_c);
        m.color_line(s.left + 1, s.bottom - 1, s.left + 1 + SHELF_DEEP, s.bottom - 1, dk_red_c);
        m.color_line(s.left + 2 + SHELF_DEEP, s.bottom - 1, s.right - 2, s.bottom - 1, black_c);
        m.color_line(s.left + 2 + SHELF_DEEP, s.top, s.right - 2, s.top, tan_c);
        m.color_line(s.right - 1, s.top + 1, s.right - 1, s.bottom - 2, black_c);

        let shelf_src = self.shelf_src;
        let mut temp_rect = shelf_src;
        zero_rect_corner(&mut temp_rect);
        q_offset_rect(&mut temp_rect, s.left + BRACKET_INSET, s.bottom);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, shelf_src, shelf_src, temp_rect);

        zero_rect_corner(&mut temp_rect);
        q_offset_rect(&mut temp_rect, s.right - BRACKET_INSET - SHELF_DEEP - BRACKET_THICK, s.bottom);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, shelf_src, shelf_src, temp_rect);
    }

    //--------------------------------------------------------------  DrawCabinet

    pub fn draw_cabinet(&mut self, cabinet: &Rect) {
        let brown_c = K8_BROWN_COLOR;
        let dk_gray_c = K8_DKST_GRAY_COLOR;
        let lt_tan_c = K8_LT_TAN_COLOR;
        let tan_c = K8_TAN_COLOR;
        let dk_red_c = K8_DK_RED2_COLOR;
        let _black_c = K8_BLACK_COLOR;
        let mut cabinet = *cabinet;
        let m = &mut self.back_src_map;

        let shadow_rgn = Trace::from(cabinet.left, cabinet.bottom)
            .by(CABINET_SHADOW_OFF, CABINET_SHADOW_OFF)
            .by(rect_wide(&cabinet), 0)
            .by(0, -rect_tall(&cabinet) + CABINET_DEEP)
            .by(-CABINET_SHADOW_OFF, -CABINET_SHADOW_OFF)
            .to(cabinet.left, cabinet.bottom)
            .region();
        m.pen_pat(PAT_GRAY);
        m.pen_mode(Mode::Or);
        m.color_region(&shadow_rgn, dk_gray_c);
        m.pen_normal();

        inset_rect(&mut cabinet, 1, 1); // fill bulk of cabinet brown
        m.color_rect(cabinet, brown_c);
        inset_rect(&mut cabinet, -1, -1);

        let c = cabinet;
        let mut temp_rect = c; // add lighter left side
        temp_rect.right = temp_rect.left + CABINET_DEEP;
        m.color_rect(temp_rect, tan_c);
        // hilight top edge
        m.color_line(c.left + 1, c.top + 1, c.left + CABINET_DEEP, c.top + 1, lt_tan_c);
        m.color_line(c.left + CABINET_DEEP, c.top + 1, c.right - 3, c.top + 1, tan_c);
        // shadow bottom edge

        m.color_line(c.left + CABINET_DEEP + 3, c.top + 5, c.left + CABINET_DEEP + 3, c.bottom - 6, tan_c);
        m.color_line(c.left + CABINET_DEEP + 4, c.top + 5, c.left + CABINET_DEEP + 4, c.bottom - 6, tan_c);
        m.color_line(c.left + CABINET_DEEP + 9, c.top + 10, c.left + CABINET_DEEP + 9, c.bottom - 11, dk_gray_c);

        m.color_line(c.right - 4, c.top + 6, c.right - 4, c.bottom - 5, dk_red_c);
        m.color_line(c.right - 5, c.top + 5, c.right - 5, c.bottom - 6, dk_gray_c);
        m.color_line(c.right - 10, c.top + 10, c.right - 10, c.bottom - 11, tan_c);

        m.color_line(c.left + CABINET_DEEP + 4, c.top + 4, c.left + CABINET_DEEP + 4, c.top + 4, lt_tan_c);
        m.color_line(c.left + CABINET_DEEP + 5, c.top + 4, c.right - 6, c.top + 4, tan_c);
        m.color_line(c.left + CABINET_DEEP + 10, c.top + 9, c.right - 11, c.top + 9, dk_gray_c);

        m.color_line(c.right - 5, c.bottom - 5, c.right - 5, c.bottom - 5, dk_red_c);
        m.color_line(c.left + CABINET_DEEP + 6, c.bottom - 4, c.right - 5, c.bottom - 4, dk_red_c);
        m.color_line(c.left + CABINET_DEEP + 5, c.bottom - 5, c.right - 6, c.bottom - 5, dk_gray_c);

        m.color_line(c.left + CABINET_DEEP + 10, c.bottom - 10, c.right - 11, c.bottom - 10, tan_c);

        let hinge_src = self.hinge_src;
        let mut temp_rect = hinge_src;
        zero_rect_corner(&mut temp_rect);
        q_offset_rect(&mut temp_rect, c.left + CABINET_DEEP + 2, c.top + 10);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, hinge_src, hinge_src, temp_rect);

        let mut temp_rect = hinge_src;
        zero_rect_corner(&mut temp_rect);
        q_offset_rect(&mut temp_rect, c.left + CABINET_DEEP + 2, c.bottom - 26);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, hinge_src, hinge_src, temp_rect);

        let handle_src = self.handle_src;
        let mut temp_rect = handle_src;
        zero_rect_corner(&mut temp_rect);
        q_offset_rect(&mut temp_rect, c.right - 8, c.top + half_rect_tall(&c) - half_rect_tall(&handle_src));
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, handle_src, handle_src, temp_rect);

        self.back_src_map.frame_rect(c);
    }

    //--------------------------------------------------------------  DrawSimpleFurniture

    pub fn draw_simple_furniture(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawCounter

    pub fn draw_counter(&mut self, counter: &Rect) {
        let brown_c = K8_BROWN_COLOR;
        let dk_gray_c = K8_DKST_GRAY_COLOR;
        let tan_c = K8_TAN_COLOR;
        let black_c = K8_BLACK_COLOR;
        let dkst_red_c = K8_DK_RED2_COLOR;
        let mut counter = *counter;
        let m = &mut self.back_src_map;

        let shadow_rgn = Trace::from(counter.right - 2, counter.bottom)
            .by(10, -10)
            .by(0, -rect_tall(&counter) + 29)
            .by(2, 0)
            .by(0, -7)
            .by(-12, -12)
            .to(counter.right - 2, counter.bottom)
            .region();
        m.pen_pat(PAT_GRAY);
        m.pen_mode(Mode::Or);
        m.color_region(&shadow_rgn, dk_gray_c);
        m.pen_normal();

        inset_rect(&mut counter, 2, 2);
        m.color_rect(counter, brown_c);
        inset_rect(&mut counter, -2, -2);

        let c = counter;
        let mut temp_rect = c;
        temp_rect.top = temp_rect.bottom - COUNTER_FOOTER_HIGH;
        temp_rect.left += 2;
        temp_rect.right -= 2;
        m.color_rect(temp_rect, dk_gray_c);
        m.color_line(c.left + 2, c.bottom - COUNTER_FOOTER_HIGH, c.right - 3, c.bottom - COUNTER_FOOTER_HIGH, black_c);
        m.color_line(c.left + 2, c.bottom - COUNTER_FOOTER_HIGH + 1, c.right - 3, c.bottom - COUNTER_FOOTER_HIGH + 1, black_c);
        m.color_line(c.right - 3, c.bottom - COUNTER_FOOTER_HIGH, c.right - 3, c.bottom - 1, black_c);
        m.color_line(c.left + 2, c.bottom - COUNTER_FOOTER_HIGH, c.left + 2, c.bottom - 1, K8_DK_GRAY_COLOR);

        m.color_line(c.right - 2, c.top, c.right - 2, c.bottom - COUNTER_FOOTER_HIGH - 1, dkst_red_c);
        m.color_line(c.left + 1, c.top + 8, c.left + 1, c.bottom - COUNTER_FOOTER_HIGH - 1, tan_c);

        m.color_line(c.left - 1, c.top, c.right, c.top, K8_LTST_GRAY_COLOR);
        m.color_line(c.left - 1, c.top + 1, c.right, c.top + 1, K8_LTST_GRAY2_COLOR);
        m.color_line(c.left - 1, c.top + 2, c.right, c.top + 2, K8_LTST_GRAY3_COLOR);
        m.color_line(c.left - 1, c.top + 3, c.right, c.top + 3, K8_LTST_GRAY4_COLOR);
        m.color_line(c.left - 1, c.top + 4, c.right, c.top + 4, K8_LTST_GRAY5_COLOR);
        m.color_line(c.left - 1, c.top + 5, c.right, c.top + 5, K8_LTST_GRAY5_COLOR);
        m.color_line(c.left - 1, c.top + 6, c.right, c.top + 6, K8_LTST_GRAY5_COLOR);
        m.color_line(c.left - 1, c.top, c.left - 1, c.top + 6, K8_LTST_GRAY_COLOR);

        m.color_line(c.right, c.top, c.right, c.top + 6, K8_LT_GRAY_COLOR);
        m.color_line(c.left + 1, c.top + 7, c.right - 2, c.top + 7, dkst_red_c);
        m.color_line(c.left + 1, c.top + 8, c.right - 2, c.top + 8, dkst_red_c);

        // (The C restores the port here, but HiliteRect below still draws
        // into the back map: SetGWorld put it back to the back map's caller,
        // which in play is the back map itself.)
        let mut n_rects = rect_wide(&c) / 40;
        if n_rects == 0 {
            n_rects = 1;
        }
        let width = ((rect_wide(&c) - COUNTER_STRIP_WIDE) / n_rects) - COUNTER_STRIP_WIDE;
        q_set_rect(&mut temp_rect, 0, 0, width, rect_tall(&c) - COUNTER_STRIP_TALL);
        q_offset_rect(&mut temp_rect, c.left + COUNTER_STRIP_WIDE, c.top + COUNTER_PANEL_DROP);
        for _ in 0..n_rects {
            m.hilite_rect(temp_rect, tan_c, dkst_red_c);
            inset_rect(&mut temp_rect, 4, 4);
            m.hilite_rect(temp_rect, dkst_red_c, tan_c);
            inset_rect(&mut temp_rect, -4, -4);
            q_offset_rect(&mut temp_rect, COUNTER_STRIP_WIDE + width, 0);
        }
    }

    //--------------------------------------------------------------  DrawDresser

    pub fn draw_dresser(&mut self, dresser: &Rect) {
        let yellow_c = K8_PISS_YELLOW_COLOR;
        let brown_c = K8_BROWN_COLOR;
        let _dk_gray_c = K8_DKST_GRAY_COLOR;
        let lt_tan_c = K8_LT_TAN_COLOR;
        let dkst_red_c = K8_DK_RED2_COLOR;
        let mut dresser = *dresser;
        let _ = DRESSER_DRAWER_DROP;

        {
            let m = &mut self.back_src_map;
            let shadow_rgn = Trace::from(dresser.left + 10, dresser.bottom + 9)
                .by(rect_wide(&dresser) - 11, 0)
                .by(9, -9)
                .by(0, -rect_tall(&dresser) + 12)
                .by(-9, -9)
                .by(-rect_wide(&dresser) + 11, 0)
                .to(dresser.left + 10, dresser.bottom + 9)
                .region();
            m.pen_pat(PAT_GRAY);
            m.pen_mode(Mode::Or);
            m.color_region(&shadow_rgn, K8_DKST_GRAY_COLOR);
            m.pen_normal();

            inset_rect(&mut dresser, 2, 2);
            m.color_rect(dresser, K8_PUMPKIN_COLOR);
            m.hilite_rect(dresser, K8_ORANGE_COLOR, dkst_red_c);
            inset_rect(&mut dresser, -2, -2);

            let d = dresser;
            let mut temp_rect = d;
            temp_rect.bottom = temp_rect.top + DRESSER_TOP_THICK;
            m.color_rect(temp_rect, K8_PISS_YELLOW_COLOR);
            m.hilite_rect(temp_rect, lt_tan_c, dkst_red_c);
            m.color_line(d.left + 2, d.top + DRESSER_TOP_THICK, d.right - 3, d.top + DRESSER_TOP_THICK, K8_RED4_COLOR);

            m.color_line(d.left + DRESSER_CREASE, d.top + DRESSER_TOP_THICK + 1, d.left + DRESSER_CREASE, d.bottom - 4, K8_RED4_COLOR);
            m.color_line(d.right - DRESSER_CREASE, d.top + DRESSER_TOP_THICK + 1, d.right - DRESSER_CREASE, d.bottom - 4, K8_ORANGE_COLOR);
        }

        let d = dresser;
        let mut n_rects = rect_tall(&d) / 30;
        if n_rects == 0 {
            n_rects = 1;
        }
        let height = (rect_tall(&d) - 14) / n_rects - 4;
        let mut temp_rect = Rect::default();
        let mut dest = Rect::default();
        q_set_rect(&mut temp_rect, 0, 0, rect_wide(&d) - DRESSER_SIDE_SPARE, height);
        q_offset_rect(&mut temp_rect, d.left + 7, d.top + 10);
        let knob_src = self.knob_src;
        for _ in 0..n_rects {
            let m = &mut self.back_src_map;
            m.color_line(temp_rect.left + 1, temp_rect.bottom, temp_rect.right, temp_rect.bottom, dkst_red_c);
            m.color_line(temp_rect.right, temp_rect.top + 1, temp_rect.right, temp_rect.bottom, dkst_red_c);
            m.color_rect(temp_rect, yellow_c);
            m.hilite_rect(temp_rect, lt_tan_c, brown_c);
            inset_rect(&mut temp_rect, 1, 1);
            m.hilite_rect(temp_rect, lt_tan_c, brown_c);
            inset_rect(&mut temp_rect, -1, -1);

            q_set_rect(&mut dest, -4, -4, 4, 4);
            q_offset_rect(&mut dest, half_rect_tall(&temp_rect), half_rect_tall(&temp_rect));
            q_offset_rect(&mut dest, temp_rect.left, temp_rect.top);
            self.back_src_map.copy_bits(&self.furniture_src_map, knob_src, dest);

            q_set_rect(&mut dest, -4, -4, 4, 4);
            q_offset_rect(&mut dest, -half_rect_tall(&temp_rect), half_rect_tall(&temp_rect));
            q_offset_rect(&mut dest, temp_rect.right, temp_rect.top);
            self.back_src_map.copy_bits(&self.furniture_src_map, knob_src, dest);

            q_offset_rect(&mut temp_rect, 0, DRESSER_TOP_THICK + height);
        }

        let left_foot_src = self.left_foot_src;
        dest = left_foot_src;
        zero_rect_corner(&mut dest);
        q_offset_rect(&mut dest, d.left + 6, d.bottom - 2);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, left_foot_src, left_foot_src, dest);

        let right_foot_src = self.right_foot_src;
        dest = right_foot_src;
        zero_rect_corner(&mut dest);
        q_offset_rect(&mut dest, d.right - 19, d.bottom - 2);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, right_foot_src, right_foot_src, dest);
    }

    //--------------------------------------------------------------  DrawDeckTable

    pub fn draw_deck_table(&mut self, table_top: &Rect, down: i32) {
        let bamboo_c = K8_BAMBOO_COLOR;
        let brown_c = K8_BROWN_COLOR;
        let dk_gray_c = K8_DKST_GRAY_COLOR;
        let mut table_top = *table_top;
        let m = &mut self.back_src_map;

        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, table_top.left, 0, table_top.right, rect_wide(&table_top) / 10);
        let ht = half_rect_tall(&temp_rect);
        q_offset_rect(&mut temp_rect, 0, -ht + TABLE_SHADOW_TOP + down);
        q_offset_rect(&mut temp_rect, TABLE_SHADOW_OFFSET, -TABLE_SHADOW_OFFSET);
        m.pen_pat(PAT_GRAY);
        m.pen_mode(Mode::Or);
        m.color_oval(temp_rect, dk_gray_c);
        m.pen_normal();

        inset_rect(&mut table_top, 0, 1);
        m.color_rect(table_top, GOLD_COLOR);
        inset_rect(&mut table_top, 0, -1);

        let t = table_top;
        m.color_line(t.left, t.top + 1, t.left, t.top + 1, K8_WHITE_COLOR);
        m.color_line(t.left + 1, t.top, t.right - 2, t.top, K8_WHITE_COLOR);
        m.color_line(t.right - 1, t.top + 1, t.right - 1, t.top + 1, K8_WHITE_COLOR);

        m.color_line(t.left + 1, t.top + 1, t.right - 2, t.top + 1, YELLOW_COLOR);
        m.color_line(t.left, t.top + 2, t.left, t.bottom - 2, YELLOW_COLOR);

        m.color_line(t.left + 1, t.bottom - 1, t.right - 2, t.bottom - 1, brown_c);
        m.color_line(t.right - 1, t.top + 2, t.right - 1, t.bottom - 2, brown_c);

        m.color_line(t.left + 1, t.bottom - 2, t.right - 2, t.bottom - 2, bamboo_c);

        if t.bottom < TABLE_BASE_TOP + down {
            let h_center = (t.left + t.right) / 2;

            m.color_line(h_center - 3, t.bottom, h_center - 3, TABLE_BASE_TOP + down, dk_gray_c);
            m.color_line(h_center - 2, t.bottom, h_center - 2, TABLE_BASE_TOP + down, K8_WHITE_COLOR);
            m.color_line(h_center - 1, t.bottom, h_center - 1, TABLE_BASE_TOP + down, K8_WHITE_COLOR);
            m.color_line(h_center, t.bottom, h_center, TABLE_BASE_TOP + down, K8_LT_GRAY_COLOR);
            m.color_line(h_center + 1, t.bottom, h_center + 1, TABLE_BASE_TOP + down, dk_gray_c);

            let v_shadow = t.bottom + rect_wide(&t) / 4 - 2;
            if v_shadow > TABLE_BASE_TOP + down {
                m.color_line(h_center - 2, t.bottom, h_center - 2, TABLE_BASE_TOP + down, K8_LT_GRAY_COLOR);
                m.color_line(h_center - 1, t.bottom, h_center - 1, TABLE_BASE_TOP + down, K8_LT_GRAY_COLOR);
                m.color_line(h_center, t.bottom, h_center, TABLE_BASE_TOP + down, dk_gray_c);
            } else {
                m.color_line(h_center - 2, t.bottom, h_center - 2, v_shadow, K8_LT_GRAY_COLOR);
                m.color_line(h_center - 1, t.bottom, h_center - 1, v_shadow + 1, K8_LT_GRAY_COLOR);
                m.color_line(h_center, t.bottom, h_center, v_shadow + 2, dk_gray_c);
            }
        }

        let deck_src = self.deck_src;
        let mut temp_rect = deck_src;
        zero_rect_corner(&mut temp_rect);
        q_offset_rect(&mut temp_rect, -half_rect_wide(&deck_src) + t.left + half_rect_wide(&t), TABLE_BASE_TOP + down);
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, deck_src, deck_src, temp_rect);
    }

    //--------------------------------------------------------------  DrawStool

    pub fn draw_stool(&mut self, the_rect: &Rect, down: i32) {
        let gray_c = K8_DK_GRAY2_COLOR;
        let dk_gray_c = K8_DKST_GRAY_COLOR;
        let m = &mut self.back_src_map;
        let r = *the_rect;

        if r.bottom < STOOL_BASE + down {
            m.color_line(r.left + 21, r.bottom - 1, r.left + 21, STOOL_BASE + down - 1, K8_DK_GRAY_COLOR);
            m.color_line(r.left + 22, r.bottom - 1, r.left + 22, STOOL_BASE + down, K8_GRAY2_COLOR);
            m.color_line(r.left + 23, r.bottom - 1, r.left + 23, STOOL_BASE + down, K8_DK_GRAY_COLOR);
            m.color_line(r.left + 24, r.bottom - 1, r.left + 24, STOOL_BASE + down, K8_DK_GRAY3_COLOR);
            m.color_line(r.left + 25, r.bottom - 1, r.left + 25, STOOL_BASE + down, gray_c);
            m.color_line(r.left + 26, r.bottom - 1, r.left + 26, STOOL_BASE + down - 1, dk_gray_c);
        }

        let sr = self.src_rects[STOOL as usize];
        self.back_src_map.copy_mask(&self.furniture_src_map, &self.furniture_mask_map, sr, sr, r);
    }

    //--------------------------------------------------------------  DrawInvisObstacle

    pub fn draw_invis_obstacle(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, K8_BROWN_COLOR);
    }

    //--------------------------------------------------------------  DrawInvisBounce

    pub fn draw_invis_bounce(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, K8_RED_COLOR);
    }

    //--------------------------------------------------------------  DrawRedClock

    pub fn draw_red_clock(&mut self, the_rect: &Rect) {
        let sr = self.src_rects[RED_CLOCK as usize];
        self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, *the_rect);

        let time_rec = get_time();
        let mut hour = time_rec.hour % 12;
        if hour == 0 {
            hour = 12;
        }
        let minutes = time_rec.minute;

        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 4, 6);
        q_offset_rect(&mut dest, the_rect.left + 5, the_rect.top + 7);
        if hour > 9 {
            self.draw_clock_digit(hour / 10, &dest);
        }
        q_offset_rect(&mut dest, 4, 0);
        self.draw_clock_digit(hour % 10, &dest);
        q_offset_rect(&mut dest, 6, 0);
        self.draw_clock_digit(minutes / 10, &dest);
        q_offset_rect(&mut dest, 4, 0);
        self.draw_clock_digit(minutes % 10, &dest);
    }

    //--------------------------------------------------------------  DrawClockDigit

    pub fn draw_clock_digit(&mut self, number: i32, dest: &Rect) {
        let sr = self.digits[number as usize];
        self.back_src_map.copy_bits(&self.bonus_src_map, sr, *dest);
    }

    //--------------------------------------------------------------  DrawBlueClock

    pub fn draw_blue_clock(&mut self, the_rect: &Rect) {
        let sr = self.src_rects[BLUE_CLOCK as usize];
        self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, *the_rect);

        let dest = Point::new(the_rect.left + 13, the_rect.top + 13);
        let time_rec = get_time();
        let hour = time_rec.hour % 12;
        let minutes = ((time_rec.minute + 2) / 5) % 12;
        self.draw_clock_hands(dest, minutes, hour);
    }

    //--------------------------------------------------------------  DrawYellowClock

    pub fn draw_yellow_clock(&mut self, the_rect: &Rect) {
        let sr = self.src_rects[YELLOW_CLOCK as usize];
        self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, *the_rect);

        let dest = Point::new(the_rect.left + 13, the_rect.top + 15);
        let time_rec = get_time();
        let hour = time_rec.hour % 12;
        let minutes = ((time_rec.minute + 2) / 5) % 12;
        self.draw_clock_hands(dest, minutes, hour);
    }

    //--------------------------------------------------------------  DrawCuckoo

    pub fn draw_cuckoo(&mut self, the_rect: &Rect) {
        let sr = self.src_rects[CUCKOO as usize];
        self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, *the_rect);

        let dest = Point::new(the_rect.left + 19, the_rect.top + 31);
        let time_rec = get_time();
        let hour = time_rec.hour % 12;
        let minutes = ((time_rec.minute + 2) / 5) % 12;
        self.draw_large_clock_hands(dest, minutes, hour);
    }

    //--------------------------------------------------------------  DrawClockHands

    pub fn draw_clock_hands(&mut self, where_: Point, big_hand: i32, little_hand: i32) {
        let m = &mut self.back_src_map;
        m.move_to(where_.h, where_.v);
        match big_hand {
            0 => m.line(0, -6),
            1 => m.line(3, -5),
            2 => m.line(5, -3),
            3 => m.line(6, 0),
            4 => m.line(5, 3),
            5 => m.line(3, 5),
            6 => m.line(0, 6),
            7 => m.line(-3, 5),
            8 => m.line(-5, 3),
            9 => m.line(-6, 0),
            10 => m.line(-5, -3),
            11 => m.line(-3, -5),
            _ => {}
        }

        m.move_to(where_.h, where_.v);
        match little_hand {
            0 => m.line(0, -4),
            1 => m.line(2, -3),
            2 => m.line(3, -2),
            3 => m.line(4, 0),
            4 => m.line(3, 2),
            5 => m.line(2, 3),
            6 => m.line(0, 4),
            7 => m.line(-2, 3),
            8 => m.line(-3, 2),
            9 => m.line(-4, 0),
            10 => m.line(-3, -2),
            11 => m.line(-2, -3),
            _ => {}
        }
    }

    //--------------------------------------------------------------  DrawClockHands

    pub fn draw_large_clock_hands(&mut self, where_: Point, big_hand: i32, little_hand: i32) {
        let m = &mut self.back_src_map;
        m.fore(gfx::WHITE);

        m.move_to(where_.h, where_.v);
        match big_hand {
            0 => m.line(0, -10),
            1 => m.line(5, -9),
            2 => m.line(9, -5),
            3 => m.line(10, 0),
            4 => m.line(9, 5),
            5 => m.line(5, 9),
            6 => m.line(0, 10),
            7 => m.line(-5, 9),
            8 => m.line(-9, 5),
            9 => m.line(-10, 0),
            10 => m.line(-9, -5),
            11 => m.line(-5, -9),
            _ => {}
        }

        m.move_to(where_.h, where_.v);
        match little_hand {
            0 => m.line(0, -6),
            1 => m.line(3, -5),
            2 => m.line(5, -3),
            3 => m.line(6, 0),
            4 => m.line(5, 3),
            5 => m.line(3, 5),
            6 => m.line(0, 6),
            7 => m.line(-3, 5),
            8 => m.line(-5, 3),
            9 => m.line(-6, 0),
            10 => m.line(-5, -3),
            11 => m.line(-3, -5),
            _ => {}
        }

        m.fore(gfx::BLACK);
    }

    //--------------------------------------------------------------  DrawSimplePrizes

    pub fn draw_simple_prizes(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawGreaseRt

    pub fn draw_grease_rt(&mut self, the_rect: &Rect, distance: i32, state: bool) {
        let mut dest = *the_rect;
        if state {
            // grease upright
            let sr = self.grease_src_rt[0];
            self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, dest);
        } else {
            // grease spilled
            q_offset_rect(&mut dest, 6, 0);
            let sr = self.grease_src_rt[3];
            self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, dest);

            let mut spill = Rect::default();
            q_set_rect(&mut spill, 0, -2, distance - 5, 0);
            q_offset_rect(&mut spill, dest.right - 1, dest.bottom);
            self.back_src_map.paint_rect(spill);
        }
    }

    //--------------------------------------------------------------  DrawGreaseLf

    pub fn draw_grease_lf(&mut self, the_rect: &Rect, distance: i32, state: bool) {
        let mut dest = *the_rect;
        if state {
            // grease upright
            let sr = self.grease_src_lf[0];
            self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, dest);
        } else {
            // grease spilled
            q_offset_rect(&mut dest, -6, 0);
            let sr = self.grease_src_lf[3];
            self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, dest);

            let mut spill = Rect::default();
            q_set_rect(&mut spill, -distance + 5, -2, 0, 0);
            q_offset_rect(&mut spill, dest.left + 1, dest.bottom);
            self.back_src_map.paint_rect(spill);
        }
    }

    //--------------------------------------------------------------  DrawBands

    pub fn draw_foil(&mut self, the_rect: &Rect) {
        let sr = self.src_rects[FOIL as usize];
        self.back_src_map.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawInvisBonus

    pub fn draw_invis_bonus(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_oval(*the_rect, 227);
    }

    //--------------------------------------------------------------  DrawSlider

    pub fn draw_slider(&mut self, the_rect: &Rect) {
        self.back_src_map.frame_rect(*the_rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date() {
        let t = get_time();
        assert!((1..=12).contains(&t.month) && (0..24).contains(&t.hour) && (0..60).contains(&t.minute));
    }
}

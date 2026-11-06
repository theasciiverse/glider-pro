//! Transitions.c: bringing the work map onto the screen (pour, wipe, dump).

use super::*;

pub const MAX_COLUMNS_WIDE: i32 = 96;
pub const CHIP_HIGH: i32 = 20;
pub const CHIP_WIDE: i32 = 16;
pub const WIPE_RECT_THICK: i32 = 4;
/// The wipe shows itself in about this many steps of a tick each (the C
/// copies straight to the screen, as fast as the Mac can).
const WIPE_SHOWN_STEPS: i32 = 16;

impl Game {
    //--------------------------------------------------------------  PourScreenOn

    pub fn pour_screen_on(&mut self, the_rect: &Rect) {
        let mut column_rects = [Rect::default(); MAX_COLUMNS_WIDE as usize];
        let mut column_progress = [0i32; MAX_COLUMNS_WIDE as usize];

        let col_wide = the_rect.right / CHIP_WIDE; // determine # of cols
        let row_tall = (the_rect.bottom / CHIP_HIGH) + 1; // determine # of rows

        let mut working = true;
        let mut cols_complete = 0;
        for i in 0..col_wide as usize {
            column_progress[i] = 0;
            q_set_rect(&mut column_rects[i], 0, 0, CHIP_WIDE, CHIP_HIGH);
            q_offset_rect(
                &mut column_rects[i],
                (i as i32 * CHIP_WIDE) + the_rect.left,
                the_rect.top,
            );
        }

        let mut chips = 0;
        while working {
            let mut i;
            loop {
                i = self.random_int(col_wide) as usize;
                if column_progress[i] < row_tall {
                    break;
                }
            }

            if column_rects[i].left < the_rect.left {
                column_rects[i].left = the_rect.left;
            }
            if column_rects[i].top < the_rect.top {
                column_rects[i].top = the_rect.top;
            }
            if column_rects[i].right > the_rect.right {
                column_rects[i].right = the_rect.right;
            }
            if column_rects[i].bottom > the_rect.bottom {
                column_rects[i].bottom = the_rect.bottom;
            }

            self.main_window_map
                .copy_bits(&self.work_src_map, column_rects[i], column_rects[i]);

            q_offset_rect(&mut column_rects[i], 0, CHIP_HIGH);
            column_progress[i] += 1;
            if column_progress[i] >= row_tall {
                cols_complete += 1;
                if cols_complete >= col_wide {
                    working = false;
                }
            }

            chips += 1;
            if chips % 32 == 0 {
                self.delay_ticks(1);
            }
        }
    }

    //--------------------------------------------------------------  WipeScreenOn

    pub fn wipe_screen_on(&mut self, direction: i32, the_rect: &Rect) {
        let mut wipe_rect = *the_rect;
        let (h_offset, v_offset, count);
        match direction {
            ABOVE => {
                wipe_rect.bottom = wipe_rect.top + WIPE_RECT_THICK;
                h_offset = 0;
                v_offset = WIPE_RECT_THICK;
                count = ((the_rect.bottom - the_rect.top) / WIPE_RECT_THICK) + 1;
            }

            TO_RIGHT => {
                wipe_rect.left = wipe_rect.right - WIPE_RECT_THICK;
                h_offset = -WIPE_RECT_THICK;
                v_offset = 0;
                count = self.work_src_rect.right / WIPE_RECT_THICK;
            }

            BELOW => {
                wipe_rect.top = wipe_rect.bottom - WIPE_RECT_THICK;
                h_offset = 0;
                v_offset = -WIPE_RECT_THICK;
                count = ((the_rect.bottom - the_rect.top) / WIPE_RECT_THICK) + 1;
            }

            TO_LEFT => {
                wipe_rect.right = wipe_rect.left + WIPE_RECT_THICK;
                h_offset = WIPE_RECT_THICK;
                v_offset = 0;
                count = self.work_src_rect.right / WIPE_RECT_THICK;
            }

            _ => {
                // (the C leaves these uninitialized; nothing is wiped)
                h_offset = 0;
                v_offset = 0;
                count = 0;
            }
        }

        let every = (count / WIPE_SHOWN_STEPS).max(1);
        for i in 0..count {
            self.main_window_map
                .copy_bits(&self.work_src_map, wipe_rect, wipe_rect);

            q_offset_rect(&mut wipe_rect, h_offset, v_offset);

            if wipe_rect.top < the_rect.top {
                wipe_rect.top = the_rect.top;
            } else if wipe_rect.top > the_rect.bottom {
                wipe_rect.top = the_rect.bottom;
            }
            if wipe_rect.bottom < the_rect.top {
                wipe_rect.bottom = the_rect.top;
            } else if wipe_rect.bottom > the_rect.bottom {
                wipe_rect.bottom = the_rect.bottom;
            }

            if (i + 1) % every == 0 {
                self.delay_ticks(1);
            }
        }
    }

    //--------------------------------------------------------------  DumpScreenOn

    pub fn dump_screen_on(&mut self, the_rect: &Rect) {
        self.main_window_map
            .copy_bits(&self.work_src_map, *the_rect, *the_rect);
    }
}

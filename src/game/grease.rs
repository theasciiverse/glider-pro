//! Grease.c: jars of grease that fall over and spill a slick line.

use super::*;

pub const GREASE_IDLE: i32 = 0;
pub const GREASE_FALLING: i32 = 1;
pub const GREASE_SPREADING: i32 = 2;
pub const GREASE_SPILT_IDLE: i32 = 3;

impl Game {
    //--------------------------------------------------------------  HandleGrease

    // Goes through all grease objects currently on screen and handles
    // them.  If they're upright, nothing happens, but if they're in
    // the course of falling or spilling, this function will handle
    // advancing the spill, etc.

    pub fn handle_grease(&mut self) {
        let mut src = Rect::default();

        if self.num_grease == 0 {
            return;
        }

        for i in 0..self.num_grease as usize {
            if self.grease[i].mode == GREASE_FALLING {
                self.grease[i].frame += 1;
                if self.grease[i].frame >= 3 {
                    // grease completely tipped
                    self.grease[i].frame = 3;
                    self.grease[i].mode = GREASE_SPREADING;
                    let hot = self.grease[i].hot_num as usize;
                    self.hot_spots[hot].action = SLIDE_IT;
                    self.hot_spots[hot].is_on = true;
                    if self.grease[i].is_right {
                        q_set_rect(&mut src, 0, -2, 2, 0);
                    } else {
                        q_set_rect(&mut src, -2, -2, 0, 0);
                    }
                    q_offset_rect(&mut src, -self.play_origin_h, -self.play_origin_v);
                    q_offset_rect(&mut src, self.grease[i].start, self.grease[i].dest.bottom);
                    self.hot_spots[hot].bounds = src;
                }

                q_set_rect(&mut src, 0, 0, 32, 27);
                q_offset_rect(&mut src, 0, self.grease[i].frame * 27);
                let dest = self.grease[i].dest;
                let map_num = self.grease[i].map_num as usize;
                if let Some(saved) = self.saved_maps[map_num].map.as_ref() {
                    self.work_src_map.copy_bits(saved, src, dest);
                    self.back_src_map.copy_bits(saved, src, dest);
                }

                self.add_rect_to_work_rects(&dest);
                if self.grease[i].is_right {
                    q_offset_rect(&mut self.grease[i].dest, 2, 0);
                } else {
                    q_offset_rect(&mut self.grease[i].dest, -2, 0);
                }
            } else if self.grease[i].mode == GREASE_SPREADING {
                let hot = self.grease[i].hot_num as usize;
                if self.grease[i].is_right {
                    q_set_rect(&mut src, 0, -2, 2, 0);
                    q_offset_rect(&mut src, self.grease[i].start, self.grease[i].dest.bottom);
                    self.grease[i].start += 2;
                    self.hot_spots[hot].bounds.right += 2;
                } else {
                    q_set_rect(&mut src, -2, -2, 0, 0);
                    q_offset_rect(&mut src, self.grease[i].start, self.grease[i].dest.bottom);
                    self.grease[i].start -= 2;
                    self.hot_spots[hot].bounds.left -= 2;
                }

                self.back_src_map.paint_rect(src);
                self.work_src_map.paint_rect(src);
                self.add_rect_to_work_rects(&src);

                if self.grease[i].is_right {
                    if self.grease[i].start >= self.grease[i].stop {
                        self.grease[i].mode = GREASE_SPILT_IDLE;
                    }
                } else {
                    if self.grease[i].start <= self.grease[i].stop {
                        self.grease[i].mode = GREASE_SPILT_IDLE;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  BackupGrease
    // This makes copies of the region of the screen the grease is about
    // to be drawn to.  It is called in the "set up" when a player has
    // just entered a new room.  The "grease jar falling over" animation
    // is set up here.

    pub fn backup_grease(&mut self, src: &mut Rect, index: i32, is_right: bool) {
        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 32, 27);
        let index = index as usize;
        let Some(mut saved) = self.saved_maps[index].map.take() else {
            log::warn!("BackupGrease: saved map {index} missing");
            return;
        };
        for i in 0..4 {
            saved.copy_bits(&self.back_src_map, *src, dest);

            if is_right {
                let r = self.grease_src_rt[i];
                saved.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, r, r, dest);
                q_offset_rect(src, 2, 0);
            } else {
                let r = self.grease_src_lf[i];
                saved.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, r, r, dest);
                q_offset_rect(src, -2, 0);
            }
            q_offset_rect(&mut dest, 0, 27);
        }
        self.saved_maps[index].map = Some(saved);
    }

    //--------------------------------------------------------------  ReBackUpGrease
    // Just like the above function but it is called while the player is
    // active in a room and has changed the lighting situation (like turned
    // off or on the lights).  It assumes certain data strucutures are
    // already declared from an earlier call to the above funciton.

    pub fn re_back_up_grease(&mut self, where_: i32, who: i32) -> i32 {
        for i in 0..self.num_grease as usize {
            if self.grease[i].where_ == where_ && self.grease[i].who == who {
                if self.grease[i].mode == GREASE_IDLE || self.grease[i].mode == GREASE_FALLING {
                    let mut src = self.grease[i].dest;
                    let (map_num, is_right) = (self.grease[i].map_num, self.grease[i].is_right);
                    self.backup_grease(&mut src, map_num, is_right);
                }
                return i as i32;
            }
        }

        -1
    }

    //--------------------------------------------------------------  AddGrease

    // Called when a new room is being set up during a game.  This adds
    // another jar of grease to the queue of jars to be handled.

    pub fn add_grease(
        &mut self,
        where_: i32,
        who: i32,
        h: i32,
        v: i32,
        distance: i32,
        is_right: bool,
    ) -> i32 {
        let mut src = Rect::default();
        let mut bounds = Rect::default();

        if self.num_grease >= MAX_GREASE {
            return -1;
        }

        q_set_rect(&mut src, 0, 0, 32, 27);
        q_offset_rect(&mut src, h, v);

        q_set_rect(&mut bounds, 0, 0, 32, 27 * 4);
        let saved_num = self.back_up_to_saved_map(&bounds, where_, who);
        if saved_num != -1 {
            self.backup_grease(&mut src, saved_num, is_right);
            if is_right {
                q_offset_rect(&mut src, -8, 0);
            } else {
                q_offset_rect(&mut src, 8, 0);
            }
            let n = self.num_grease as usize;
            let gr = &mut self.grease[n];
            gr.who = who;
            gr.where_ = where_;
            gr.dest = src;
            gr.map_num = saved_num;
            gr.mode = GREASE_IDLE;
            gr.frame = -1;
            if is_right {
                gr.is_right = true;
                gr.start = src.right + 4;
                gr.stop = src.right + distance;
            } else {
                gr.is_right = false;
                gr.start = src.left - 4;
                gr.stop = src.left - distance;
            }
            self.num_grease += 1;

            self.num_grease - 1
        } else {
            -1
        }
    }

    //--------------------------------------------------------------  SpillGrease

    // A player has knocked a jar of grease over - this function flags that.

    pub fn spill_grease(&mut self, who: i32, index: i32) {
        if self.grease[who as usize].mode == GREASE_IDLE {
            self.grease[who as usize].mode = GREASE_FALLING;
            self.grease[who as usize].hot_num = index;
            self.play_priority_sound(GREASE_SPILL_SOUND, GREASE_SPILL_PRIORITY);
        }
    }

    //--------------------------------------------------------------  RedrawAllGrease
    // Called to redraw all the black lines of spilt grease.

    pub fn redraw_all_grease(&mut self) {
        if self.num_grease == 0 {
            return;
        }

        for i in 0..self.num_grease as usize {
            // hotNum is set only once a jar is spilt; the C reads
            // hotSpots[garbage] for an idle jar but then skips it.
            let hot = self.grease[i].hot_num;
            if self.grease[i].mode == GREASE_IDLE || hot < 0 || hot as usize >= self.hot_spots.len()
            {
                continue;
            }
            let mut src = self.hot_spots[hot as usize].bounds;
            if self.grease[i].where_ == self.this_room_number
                && (src.bottom - src.top) == 2
                && self.grease[i].mode != GREASE_IDLE
            {
                q_offset_rect(&mut src, self.play_origin_h, self.play_origin_v);

                self.back_src_map.paint_rect(src);
                self.work_src_map.paint_rect(src);
                self.add_rect_to_work_rects(&src);
            }
        }
    }
}

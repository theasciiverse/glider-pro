//! DynamicMaps.c: the saved backgrounds under animated objects (flames,
//! pendulums, stars), sparkles, flying points and shredded gliders.

use super::*;

//==============================================================  Functions

impl Game {
    //--------------------------------------------------------------  NilSavedMaps
    // Deletes array of "dyanmics" offscreen pixmaps.

    pub fn nil_saved_maps(&mut self) {
        if self.saved_maps.len() < MAX_SAVED_MAPS as usize {
            self.saved_maps.resize(MAX_SAVED_MAPS as usize, SavedType::default());
        }
        for i in 0..MAX_SAVED_MAPS as usize {
            self.saved_maps[i].map = None;
            self.saved_maps[i].where_ = -1;
            self.saved_maps[i].who = -1;
        }
        self.num_saved_maps = 0;
    }

    //--------------------------------------------------------------  BackUpToSavedMap
    // Saves a copy of the room behind an object to an array of pixmaps.
    // Then when the object in question is drawn, there is a copy of the
    // room that it obscured so that, should the player get the object,
    // it can be made to "disappear".

    pub fn back_up_to_saved_map(&mut self, the_rect: &Rect, where_: i32, who: i32) -> i32 {
        if self.num_saved_maps >= MAX_SAVED_MAPS {
            return -1;
        }

        let n = self.num_saved_maps as usize;
        let mut map_rect = *the_rect;
        zero_rect_corner(&mut map_rect);
        self.saved_maps[n].dest = *the_rect;
        let mut map = Map::new(map_rect);

        map.copy_bits(&self.back_src_map, *the_rect, map_rect);
        self.saved_maps[n].map = Some(map);

        self.saved_maps[n].where_ = where_;
        self.saved_maps[n].who = who;
        self.num_saved_maps += 1;

        self.num_saved_maps - 1 // return array index
    }

    //--------------------------------------------------------------  ReBackUpSavedMap
    // This function is similar to the above, but assumes there is already
    // a slot in the pixmap array for the object.  It re-copies the background
    // and is needed when the lights in the room go on or off.

    pub fn re_back_up_saved_map(&mut self, the_rect: &Rect, where_: i32, who: i32) -> i32 {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who {
                let mut map_rect = *the_rect;
                zero_rect_corner(&mut map_rect);

                if let Some(m) = self.saved_maps[i].map.as_mut() {
                    m.copy_bits(&self.back_src_map, *the_rect, map_rect);
                }

                return i as i32;
            }
        }

        -1
    }

    //--------------------------------------------------------------  RestoreFromSavedMap

    // This copies the saved background swatch to the screen - effectively
    // covering up or "erasing" the object.

    pub fn restore_from_saved_map(&mut self, where_: i32, who: i32, do_sparkle: bool) {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who && self.saved_maps[i].map.is_some() {
                let dest = self.saved_maps[i].dest;
                let mut map_rect = dest;
                zero_rect_corner(&mut map_rect);

                if let Some(m) = &self.saved_maps[i].map {
                    self.back_src_map.copy_bits(m, map_rect, dest);
                    self.work_src_map.copy_bits(m, map_rect, dest);
                }

                self.add_rect_to_work_rects(&dest);

                if do_sparkle {
                    let mut bounds = dest;
                    q_offset_rect(&mut bounds, -self.play_origin_h, -self.play_origin_v);
                    self.add_sparkle(&mut bounds);
                    self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                }
                break;
            }
        }
    }

    //--------------------------------------------------------------  AddSparkle

    // This adds a "sparkle" object to the fixed array of sparkles.
    // (The C offsets the caller's rect by the play origin, as here.)

    pub fn add_sparkle(&mut self, the_rect: &mut Rect) {
        if self.num_sparkles < MAX_SPARKLES {
            the_rect.left += self.play_origin_h;
            the_rect.right += self.play_origin_h;
            the_rect.top += self.play_origin_v;
            the_rect.bottom += self.play_origin_v;

            let mut centered_rect = self.sparkle_src[0];
            center_rect_in_rect(&mut centered_rect, the_rect);

            for i in 0..MAX_SPARKLES as usize {
                if self.sparkles[i].mode == -1 {
                    self.sparkles[i].bounds = centered_rect;
                    self.sparkles[i].mode = 0;
                    self.num_sparkles += 1;
                    break;
                }
            }
        }
    }

    //--------------------------------------------------------------  AddFlyingPoint

    // This adds a "flying point" object to the array of flying points.

    pub fn add_flying_point(&mut self, the_rect: &mut Rect, points: i32, h_vel: i32, v_vel: i32) {
        if self.num_flying_pts < MAX_FLYING_PTS {
            the_rect.left += self.play_origin_h;
            the_rect.right += self.play_origin_h;
            the_rect.top += self.play_origin_v;
            the_rect.bottom += self.play_origin_v;

            let mut centered_rect = self.points_src[0];
            center_rect_in_rect(&mut centered_rect, the_rect);

            for i in 0..MAX_FLYING_PTS as usize {
                let p = &mut self.flying_points[i];
                if p.mode == -1 {
                    p.dest = centered_rect;
                    p.whole = centered_rect;
                    p.loops = 0;
                    p.h_vel = h_vel;
                    p.v_vel = v_vel;
                    let (start, stop) = match points {
                        100 => (12, 14),
                        250 => (9, 11),
                        300 => (6, 8),
                        500 => (3, 5),
                        _ => (0, 2),
                    };
                    p.start = start;
                    p.stop = stop;
                    p.mode = p.start;
                    self.num_flying_pts += 1;
                    break;
                }
            }
        }
    }

    //--------------------------------------------------------------  BackUpFlames

    // This makes copies of the area of the room behind a flame.  The flame
    // graphic can be "copy masked" to this pixmap then and then simple
    // CopyBits() calls will properly draw the flame on screen with the
    // proper background.

    pub fn back_up_flames(&mut self, src: &Rect, index: i32) {
        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 16, 15);
        for i in 0..NUM_CANDLE_FLAMES as usize {
            if let Some(m) = self.saved_maps[index as usize].map.as_mut() {
                // Copy background to map.
                m.copy_bits(&self.back_src_map, *src, dest);

                // Copy flame to map.
                m.copy_mask(&self.blower_src_map, &self.blower_mask_map, self.flame[i], self.flame[i], dest);
            }

            q_offset_rect(&mut dest, 0, 15);
        }
    }

    //--------------------------------------------------------------  ReBackUpFlames

    // Like the above function but this is called when the lighting changes
    // in a room (lights go on or off).

    pub fn re_back_up_flames(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who {
                for f in 0..self.num_flames as usize {
                    if self.flames[f].who == i as i32 {
                        let dest = self.flames[f].dest;
                        self.back_up_flames(&dest, i as i32);
                        return;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  AddCandleFlame

    // This adds a candle flame to tha array of flames.

    pub fn add_candle_flame(&mut self, where_: i32, who: i32, h: i32, v: i32) {
        if self.num_flames >= MAX_CANDLES || h < 16 || v < 15 {
            return;
        }

        let mut src = Rect::default();
        q_set_rect(&mut src, 0, 0, 16, 15);
        q_offset_rect(&mut src, h - 8, v - 15);
        // (thisMac.isDepth == 4: odd lefts moved; always 8 here)
        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 16, 15 * NUM_CANDLE_FLAMES);
        let saved_num = self.back_up_to_saved_map(&bounds, where_, who);
        if saved_num != -1 {
            self.back_up_flames(&src, saved_num);
            let mode = self.random_int(NUM_CANDLE_FLAMES);
            let n = self.num_flames as usize;
            let f = &mut self.flames[n];
            f.dest = src;
            f.mode = mode;
            q_set_rect(&mut f.src, 0, 0, 16, 15);
            q_offset_rect(&mut f.src, 0, f.mode * 15);
            f.who = saved_num;
            self.num_flames += 1;
        }
    }

    //--------------------------------------------------------------  BackUpTikiFlames
    // This is like the function BackUpFlames() but customized for Tiki torches.

    pub fn back_up_tiki_flames(&mut self, src: &Rect, index: i32) {
        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 8, 10);
        for i in 0..NUM_TIKI_FLAMES as usize {
            if let Some(m) = self.saved_maps[index as usize].map.as_mut() {
                // copy background to map
                m.copy_bits(&self.back_src_map, *src, dest);

                // copy flame to map
                m.copy_mask(
                    &self.blower_src_map,
                    &self.blower_mask_map,
                    self.tiki_flame[i],
                    self.tiki_flame[i],
                    dest,
                );
            }

            q_offset_rect(&mut dest, 0, 10);
        }
    }

    //--------------------------------------------------------------  ReBackUpTikiFlames

    // This is like the function ReBackUpFlames() but customized for Tiki torches.

    pub fn re_back_up_tiki_flames(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who {
                for f in 0..self.num_tiki_flames as usize {
                    if self.tiki_flames[f].who == i as i32 {
                        let dest = self.tiki_flames[f].dest;
                        self.back_up_tiki_flames(&dest, i as i32);
                        return;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  AddTikiFlame

    // This adds a tiki flame to the array of tiki flames.

    pub fn add_tiki_flame(&mut self, where_: i32, who: i32, h: i32, v: i32) {
        if self.num_tiki_flames >= MAX_TIKIS || h < 8 || v < 10 {
            return;
        }

        let mut src = Rect::default();
        q_set_rect(&mut src, 0, 0, 8, 10);
        // (thisMac.isDepth == 4: odd h moved; always 8 here)
        q_offset_rect(&mut src, h, v);
        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 8, 10 * NUM_TIKI_FLAMES);
        let saved_num = self.back_up_to_saved_map(&bounds, where_, who);
        if saved_num != -1 {
            self.back_up_tiki_flames(&src, saved_num);
            let mode = self.random_int(NUM_TIKI_FLAMES);
            let n = self.num_tiki_flames as usize;
            let f = &mut self.tiki_flames[n];
            f.dest = src;
            f.mode = mode;
            q_set_rect(&mut f.src, 0, 0, 8, 10);
            q_offset_rect(&mut f.src, 0, f.mode * 10);
            f.who = saved_num;
            self.num_tiki_flames += 1;
        }
    }

    //--------------------------------------------------------------  BackUpBBQCoals

    // Another one - but for BBQ coals.

    pub fn back_up_bbq_coals(&mut self, src: &Rect, index: i32) {
        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 32, 9);
        for i in 0..NUM_BBQ_COALS as usize {
            if let Some(m) = self.saved_maps[index as usize].map.as_mut() {
                // copy background to map
                m.copy_bits(&self.back_src_map, *src, dest);

                // copy flame to map
                m.copy_mask(&self.blower_src_map, &self.blower_mask_map, self.coals[i], self.coals[i], dest);
            }

            q_offset_rect(&mut dest, 0, 9);
        }
    }

    //--------------------------------------------------------------  ReBackUpBBQCoals

    // Sense a pattern here?

    pub fn re_back_up_bbq_coals(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who {
                for f in 0..self.num_coals as usize {
                    if self.bbq_coals[f].who == i as i32 {
                        let dest = self.bbq_coals[f].dest;
                        self.back_up_bbq_coals(&dest, i as i32);
                        return;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  AddBBQCoals

    // Adds BBQ coals to the array of BBQ coals.

    pub fn add_bbq_coals(&mut self, where_: i32, who: i32, h: i32, v: i32) {
        if self.num_coals >= MAX_COALS || h < 32 || v < 9 {
            return;
        }

        let mut src = Rect::default();
        q_set_rect(&mut src, 0, 0, 32, 9);
        // (thisMac.isDepth == 4: odd h moved; always 8 here)
        q_offset_rect(&mut src, h, v);
        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 32, 9 * NUM_BBQ_COALS);
        let saved_num = self.back_up_to_saved_map(&bounds, where_, who);
        if saved_num != -1 {
            self.back_up_bbq_coals(&src, saved_num);
            let mode = self.random_int(NUM_BBQ_COALS);
            let n = self.num_coals as usize;
            let f = &mut self.bbq_coals[n];
            f.dest = src;
            f.mode = mode;
            q_set_rect(&mut f.src, 0, 0, 32, 9);
            q_offset_rect(&mut f.src, 0, f.mode * 9);
            f.who = saved_num;

            self.num_coals += 1;
        }
    }

    //--------------------------------------------------------------  BackUpPendulum
    // Just like many of the previous functions, but for the pendulum on the
    // cuckoo clock.

    pub fn back_up_pendulum(&mut self, src: &Rect, index: i32) {
        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 32, 28);
        for i in 0..NUM_PENDULUMS as usize {
            if let Some(m) = self.saved_maps[index as usize].map.as_mut() {
                m.copy_bits(&self.back_src_map, *src, dest);

                m.copy_mask(
                    &self.bonus_src_map,
                    &self.bonus_mask_map,
                    self.pendulum_src[i],
                    self.pendulum_src[i],
                    dest,
                );
            }

            q_offset_rect(&mut dest, 0, 28);
        }
    }

    //--------------------------------------------------------------  ReBackUpPendulum

    // Backs up the pendulums in the event of lights going on or off.

    pub fn re_back_up_pendulum(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who {
                for f in 0..self.num_pendulums as usize {
                    if self.pendulums[f].who == i as i32 {
                        let dest = self.pendulums[f].dest;
                        self.back_up_pendulum(&dest, i as i32);
                        return;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  AddPendulum

    // Adds a pendulum to the array of pendulums.

    pub fn add_pendulum(&mut self, where_: i32, who: i32, h: i32, v: i32) {
        if self.num_pendulums >= MAX_PENDULUMS || h < 32 || v < 28 {
            return;
        }

        self.clock_frame = 10;
        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 32, 28 * NUM_PENDULUMS);
        let saved_num = self.back_up_to_saved_map(&bounds, where_, who);
        if saved_num != -1 {
            let mut src = Rect::default();
            q_set_rect(&mut src, 0, 0, 32, 28);
            // (thisMac.isDepth == 4: odd h moved; always 8 here)
            q_offset_rect(&mut src, h, v);
            self.back_up_pendulum(&src, saved_num);
            let to_or_fro = self.random_int(2) == 0;
            let n = self.num_pendulums as usize;
            let p = &mut self.pendulums[n];
            p.dest = src;
            p.mode = 1;
            p.to_or_fro = to_or_fro;
            p.active = true;
            q_set_rect(&mut p.src, 0, 0, 32, 28);
            q_offset_rect(&mut p.src, 0, 28);
            p.who = saved_num;
            p.where_ = where_;
            p.link = who;
            self.num_pendulums += 1;
        }
    }

    //--------------------------------------------------------------  BackUpStar

    // Makes a copy of background beneath a star.

    pub fn back_up_star(&mut self, src: &Rect, index: i32) {
        let mut dest = Rect::default();
        q_set_rect(&mut dest, 0, 0, 32, 31);
        for i in 0..6 {
            if let Some(m) = self.saved_maps[index as usize].map.as_mut() {
                m.copy_bits(&self.back_src_map, *src, dest);

                // copy flame to map
                m.copy_mask(&self.bonus_src_map, &self.bonus_mask_map, self.star_src[i], self.star_src[i], dest);
            }

            q_offset_rect(&mut dest, 0, 31);
        }
    }

    //--------------------------------------------------------------  ReBackUpStar

    // Re-backs up the stars - in the event of lighting switch.

    pub fn re_back_up_star(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_saved_maps as usize {
            if self.saved_maps[i].where_ == where_ && self.saved_maps[i].who == who {
                for f in 0..self.num_stars as usize {
                    if self.the_stars[f].who == i as i32 {
                        let dest = self.the_stars[f].dest;
                        self.back_up_star(&dest, i as i32);
                        return;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  AddStar

    // Adds a star to the star array.

    pub fn add_star(&mut self, where_: i32, who: i32, h: i32, v: i32) {
        if self.num_stars >= MAX_STARS {
            return;
        }

        let mut src = Rect::default();
        q_set_rect(&mut src, 0, 0, 32, 31);
        // (thisMac.isDepth == 4: odd h moved; always 8 here)
        q_offset_rect(&mut src, h, v);

        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 32, 31 * 6);
        let saved_num = self.back_up_to_saved_map(&bounds, where_, who);
        if saved_num != -1 {
            self.back_up_star(&src, saved_num);
            let mode = self.random_int(6);
            let n = self.num_stars as usize;
            let s = &mut self.the_stars[n];
            s.dest = src;
            s.mode = mode;
            q_set_rect(&mut s.src, 0, 0, 32, 31);
            q_offset_rect(&mut s.src, 0, s.mode * 31);
            s.who = saved_num;
            s.link = who;
            s.where_ = where_;

            self.num_stars += 1;
        }
    }

    //--------------------------------------------------------------  StopPendulum

    // Will set a flag to kill a pendulum.

    pub fn stop_pendulum(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_pendulums as usize {
            if self.pendulums[i].link == who && self.pendulums[i].where_ == where_ {
                self.pendulums[i].active = false;
            }
        }
    }

    //--------------------------------------------------------------  StopStar

    // Will set a flag to kill a star.

    pub fn stop_star(&mut self, where_: i32, who: i32) {
        for i in 0..self.num_stars as usize {
            if self.the_stars[i].link == who && self.the_stars[i].where_ == where_ {
                self.the_stars[i].mode = -1;
            }
        }
    }

    //--------------------------------------------------------------  AddAShreddedGlider

    // Adds a shredded glider.

    pub fn add_a_shredded_glider(&mut self, bounds: &Rect) {
        if self.num_shredded > MAX_SHREDDED {
            return;
        }
        // The C's test lets one more in than the array holds (writing past
        // its end); here that one is dropped.
        let n = self.num_shredded as usize;
        if n >= self.shreds.len() {
            return;
        }

        let s = &mut self.shreds[n];
        s.bounds.left = bounds.left + 4;
        s.bounds.right = s.bounds.left + 40;
        s.bounds.top = bounds.top + 14;
        s.bounds.bottom = s.bounds.top;
        s.frame = 0;

        self.num_shredded += 1;
    }

    //--------------------------------------------------------------  RemoveShreds

    // Remves the shredded glider.

    pub fn remove_shreds(&mut self) {
        let mut largest = 0;
        let mut who: i32 = -1;
        for i in 0..self.num_shredded {
            if self.shreds[i as usize].frame > largest {
                largest = self.shreds[i as usize].frame;
                who = i;
            }
        }

        if who != -1 {
            let w = who as usize;
            if who == self.num_shredded - 1 {
                self.num_shredded -= 1;
                self.shreds[w].frame = 0;
            } else {
                self.num_shredded -= 1;
                let n = self.num_shredded as usize;
                self.shreds[w].bounds = self.shreds[n].bounds;
                self.shreds[w].frame = self.shreds[n].frame;
                self.shreds[n].frame = 0;
            }
        }
    }

    //--------------------------------------------------------------  ZeroFlamesAndTheLike

    // Zeroes all counters that indicate the number of flames, pendulums, etc there
    // are in a room.  Called before a room is drawn.  As the room is drawn, the
    // above functions are called and the counters incremented as objects of the
    // specified types are drawn.

    pub fn zero_flames_and_the_like(&mut self) {
        self.num_flames = 0;
        self.num_tiki_flames = 0;
        self.num_coals = 0;
        self.num_pendulums = 0;
        self.num_grease = 0;
        self.num_stars = 0;
        self.num_shredded = 0;
        self.num_chimes = 0;
    }
}

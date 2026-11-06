//! Render.c: each frame's compositing: sprites into the work map, then
//! the changed rects work -> main window and back -> work.

use super::*;
use crate::gfx;

pub const MAX_GARBAGE_RECTS: i32 = 48;

/// CopyMask with the destination's clip set to a region (SetClip(rgn)):
/// pixels outside `rgn` are left as they were. No region clips everything.
fn copy_mask_rgn(dst: &mut Map, src: &Map, mask: &Map, sr: Rect, mr: Rect, dr: Rect, rgn: Option<&Region>) {
    let c = gfx::sect(dr, dst.clip);
    let mut was = Vec::with_capacity((c.width().max(0) * c.height().max(0)) as usize);
    for y in c.top..c.bottom {
        for x in c.left..c.right {
            was.push(dst.get(x, y));
        }
    }
    dst.copy_mask(src, mask, sr, mr, dr);
    let mut k = 0;
    for y in c.top..c.bottom {
        for x in c.left..c.right {
            if !rgn.is_some_and(|g| g.contains(x, y)) {
                dst.set(x, y, was[k]);
            }
            k += 1;
        }
    }
}

//==============================================================  Functions

impl Game {
    fn garbage_rects_allocated(&mut self) {
        let n = MAX_GARBAGE_RECTS as usize;
        if self.work2_main_rects.len() < n {
            self.work2_main_rects.resize(n, Rect::default());
        }
        if self.back2_work_rects.len() < n {
            self.back2_work_rects.resize(n, Rect::default());
        }
    }

    //--------------------------------------------------------------  AddRectToWorkRects

    pub fn add_rect_to_work_rects(&mut self, the_rect: &Rect) {
        if self.num_work2_main < MAX_GARBAGE_RECTS - 1 {
            self.garbage_rects_allocated();
            let j = self.just_rooms_rect;
            let r = &mut self.work2_main_rects[self.num_work2_main as usize];
            *r = *the_rect;
            if r.left < j.left {
                r.left = j.left;
            } else if r.right > j.right {
                r.right = j.right;
            }
            if r.top < j.top {
                r.top = j.top;
            } else if r.bottom > j.bottom {
                r.bottom = j.bottom;
            }
            self.num_work2_main += 1;
        }
    }

    //--------------------------------------------------------------  AddRectToBackRects

    pub fn add_rect_to_back_rects(&mut self, the_rect: &Rect) {
        if self.num_back2_work < MAX_GARBAGE_RECTS - 1 {
            self.garbage_rects_allocated();
            let w = self.work_src_rect;
            let r = &mut self.back2_work_rects[self.num_back2_work as usize];
            *r = *the_rect;
            if r.left < 0 {
                r.left = 0;
            } else if r.right > w.right {
                r.right = w.right;
            }
            if r.top < 0 {
                r.top = 0;
            } else if r.bottom > w.bottom {
                r.bottom = w.bottom;
            }
            self.num_back2_work += 1;
        }
    }

    //--------------------------------------------------------------  AddRectToWorkRectsWhole

    pub fn add_rect_to_work_rects_whole(&mut self, the_rect: &Rect) {
        if self.num_work2_main < MAX_GARBAGE_RECTS - 1 {
            let w = self.work_src_rect;
            if the_rect.right <= w.left || the_rect.bottom <= w.top || the_rect.left >= w.right || the_rect.top >= w.bottom {
                return;
            }

            self.garbage_rects_allocated();
            let r = &mut self.work2_main_rects[self.num_work2_main as usize];
            *r = *the_rect;

            if r.left < w.left {
                r.left = w.left;
            } else if r.right > w.right {
                r.right = w.right;
            }
            if r.top < w.top {
                r.top = w.top;
            } else if r.bottom > w.bottom {
                r.bottom = w.bottom;
            }

            if r.right == r.left || r.top == r.bottom {
                return;
            }

            self.num_work2_main += 1;
        }
    }

    //--------------------------------------------------------------  DrawReflection

    pub fn draw_reflection(&mut self, g: usize, one_or_two: bool) {
        if self.gliders[g].dont_draw {
            return;
        }

        // (which = facing: unused here, as in the C)

        let gl = self.gliders[g];
        let mut dest = gl.dest;
        q_offset_rect(&mut dest, self.play_origin_h - 20, self.play_origin_v - 16);

        // SetClip(mirrorRgn) on the work map for the copy
        let rgn = self.mirror_rgn.as_ref();
        if one_or_two {
            if self.show_foil {
                copy_mask_rgn(
                    &mut self.work_src_map,
                    &self.glid2_src_map,
                    &self.glid_mask_map,
                    gl.src,
                    gl.mask,
                    dest,
                    rgn,
                );
            } else {
                copy_mask_rgn(
                    &mut self.work_src_map,
                    &self.glid_src_map,
                    &self.glid_mask_map,
                    gl.src,
                    gl.mask,
                    dest,
                    rgn,
                );
            }
        } else {
            copy_mask_rgn(
                &mut self.work_src_map,
                &self.glid2_src_map,
                &self.glid_mask_map,
                gl.src,
                gl.mask,
                dest,
                rgn,
            );
        }

        let mut src = gl.whole;
        q_offset_rect(&mut src, self.play_origin_h - 20, self.play_origin_v - 16);
        self.add_rect_to_work_rects(&src);
        self.add_rect_to_back_rects(&dest);
    }

    //--------------------------------------------------------------  RenderFlames

    pub fn render_flames(&mut self) {
        if self.num_flames == 0 && self.num_tiki_flames == 0 && self.num_coals == 0 {
            return;
        }

        for i in 0..self.num_flames as usize {
            let f = &mut self.flames[i];
            f.mode += 1;
            f.src.top += 15;
            f.src.bottom += 15;
            if f.mode >= NUM_CANDLE_FLAMES {
                f.mode = 0;
                f.src.top = 0;
                f.src.bottom = 15;
            }
            let f = *f;

            if let Some(m) = &self.saved_maps[f.who as usize].map {
                self.work_src_map.copy_bits(m, f.src, f.dest);
            }

            self.add_rect_to_work_rects(&f.dest);
        }

        for i in 0..self.num_tiki_flames as usize {
            let f = &mut self.tiki_flames[i];
            f.mode += 1;
            f.src.top += 10;
            f.src.bottom += 10;
            if f.mode >= NUM_TIKI_FLAMES {
                f.mode = 0;
                f.src.top = 0;
                f.src.bottom = 10;
            }
            let f = *f;

            if let Some(m) = &self.saved_maps[f.who as usize].map {
                self.work_src_map.copy_bits(m, f.src, f.dest);
            }

            self.add_rect_to_work_rects(&f.dest);
        }

        for i in 0..self.num_coals as usize {
            let f = &mut self.bbq_coals[i];
            f.mode += 1;
            f.src.top += 9;
            f.src.bottom += 9;
            if f.mode >= NUM_BBQ_COALS {
                f.mode = 0;
                f.src.top = 0;
                f.src.bottom = 9;
            }
            let f = *f;

            if let Some(m) = &self.saved_maps[f.who as usize].map {
                self.work_src_map.copy_bits(m, f.src, f.dest);
            }

            self.add_rect_to_work_rects(&f.dest);
        }
    }

    //--------------------------------------------------------------  RenderPendulums

    pub fn render_pendulums(&mut self) {
        let mut played_tik_tok = false;

        if self.num_pendulums == 0 {
            return;
        }

        self.clock_frame += 1;
        if self.clock_frame == 10 || self.clock_frame == 15 {
            if self.clock_frame >= 15 {
                self.clock_frame = 0;
            }

            for i in 0..self.num_pendulums as usize {
                if self.pendulums[i].active {
                    if self.pendulums[i].to_or_fro {
                        let p = &mut self.pendulums[i];
                        p.mode += 1;
                        p.src.top += 28;
                        p.src.bottom += 28;

                        if p.mode >= 2 {
                            p.to_or_fro = !p.to_or_fro;
                            if !played_tik_tok {
                                self.play_priority_sound(TIK_SOUND, TIK_PRIORITY);
                                played_tik_tok = true;
                            }
                        }
                    } else {
                        let p = &mut self.pendulums[i];
                        p.mode -= 1;
                        p.src.top -= 28;
                        p.src.bottom -= 28;

                        if p.mode <= 0 {
                            p.to_or_fro = !p.to_or_fro;
                            if !played_tik_tok {
                                self.play_priority_sound(TOK_SOUND, TOK_PRIORITY);
                                played_tik_tok = true;
                            }
                        }
                    }

                    let p = self.pendulums[i];
                    if let Some(m) = &self.saved_maps[p.who as usize].map {
                        self.work_src_map.copy_bits(m, p.src, p.dest);
                    }

                    self.add_rect_to_work_rects(&p.dest);
                }
            }
        }
    }

    //--------------------------------------------------------------  RenderFlyingPoints

    pub fn render_flying_points(&mut self) {
        if self.num_flying_pts == 0 {
            return;
        }

        for i in 0..MAX_FLYING_PTS as usize {
            if self.flying_points[i].mode != -1 {
                let p = &mut self.flying_points[i];
                if p.mode > p.stop {
                    p.mode = p.start;
                    p.loops += 1;
                }

                if p.loops >= MAX_FLYING_POINTS_LOOP {
                    let dest = p.dest;
                    self.add_rect_to_work_rects(&dest);
                    self.flying_points[i].mode = -1;
                    self.num_flying_pts -= 1;
                } else {
                    p.dest.left += p.h_vel;
                    p.dest.right += p.h_vel;

                    if p.h_vel > 0 {
                        p.whole.right = p.dest.right;
                    } else {
                        p.whole.left = p.dest.left;
                    }

                    p.dest.top += p.v_vel;
                    p.dest.bottom += p.v_vel;

                    if p.v_vel > 0 {
                        p.whole.bottom = p.dest.bottom;
                    } else {
                        p.whole.top = p.dest.top;
                    }
                    let p = *p;

                    let s = self.points_src[p.mode as usize];
                    self.work_src_map
                        .copy_mask(&self.points_src_map, &self.points_mask_map, s, s, p.dest);

                    self.add_rect_to_work_rects(&p.whole);
                    self.add_rect_to_back_rects(&p.dest);
                    self.flying_points[i].whole = p.dest;
                    self.flying_points[i].mode += 1;
                }
            }
        }
    }

    //--------------------------------------------------------------  RenderSparkles

    pub fn render_sparkles(&mut self) {
        if self.num_sparkles == 0 {
            return;
        }

        for i in 0..MAX_SPARKLES as usize {
            let s = self.sparkles[i];
            if s.mode != -1 {
                if s.mode >= NUM_SPARKLE_MODES {
                    self.add_rect_to_work_rects(&s.bounds);
                    self.sparkles[i].mode = -1;
                    self.num_sparkles -= 1;
                } else {
                    let src = self.sparkle_src[s.mode as usize];
                    self.work_src_map
                        .copy_mask(&self.bonus_src_map, &self.bonus_mask_map, src, src, s.bounds);

                    self.add_rect_to_work_rects(&s.bounds);
                    self.add_rect_to_back_rects(&s.bounds);
                    self.sparkles[i].mode += 1;
                }
            }
        }
    }

    //--------------------------------------------------------------  RenderStars

    pub fn render_stars(&mut self) {
        if self.num_stars == 0 {
            return;
        }

        for i in 0..self.num_stars as usize {
            if self.the_stars[i].mode != -1 {
                let s = &mut self.the_stars[i];
                s.mode += 1;
                s.src.top += 31;
                s.src.bottom += 31;
                if s.mode >= 6 {
                    s.mode = 0;
                    s.src.top = 0;
                    s.src.bottom = 31;
                }
                let s = *s;

                if let Some(m) = &self.saved_maps[s.who as usize].map {
                    self.work_src_map.copy_bits(m, s.src, s.dest);
                }

                self.add_rect_to_work_rects(&s.dest);
            }
        }
    }

    //--------------------------------------------------------------  RenderGlider

    pub fn render_glider(&mut self, g: usize, one_or_two: bool) {
        let gl = self.gliders[g];
        if gl.dont_draw {
            return;
        }

        let which = if gl.facing == FACE_RIGHT { 0 } else { 1 };

        if self.shadow_visible {
            let mut dest = gl.dest_shadow;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);

            if gl.mode == GLIDER_COMING_UP || gl.mode == GLIDER_GOING_DOWN {
                let mut src = self.shadow_src[which];
                src.right = src.left + (dest.right - dest.left);

                self.work_src_map
                    .copy_mask(&self.shadow_src_map, &self.shadow_mask_map, src, src, dest);
            } else if gl.mode == GLIDER_COMING_DOWN {
                let mut src = self.shadow_src[which];
                src.left = src.right - (dest.right - dest.left);

                self.work_src_map
                    .copy_mask(&self.shadow_src_map, &self.shadow_mask_map, src, src, dest);
            } else {
                let s = self.shadow_src[which];
                self.work_src_map.copy_mask(&self.shadow_src_map, &self.shadow_mask_map, s, s, dest);
            }
            let mut src = gl.whole_shadow;
            q_offset_rect(&mut src, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&src);
            self.add_rect_to_back_rects(&dest);
        }

        let mut dest = gl.dest;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);

        if one_or_two {
            if !self.two_player_game && self.show_foil {
                self.work_src_map
                    .copy_mask(&self.glid2_src_map, &self.glid_mask_map, gl.src, gl.mask, dest);
            } else {
                self.work_src_map
                    .copy_mask(&self.glid_src_map, &self.glid_mask_map, gl.src, gl.mask, dest);
            }
        } else {
            self.work_src_map
                .copy_mask(&self.glid2_src_map, &self.glid_mask_map, gl.src, gl.mask, dest);
        }

        let mut src = gl.whole;
        q_offset_rect(&mut src, self.play_origin_h, self.play_origin_v);
        self.add_rect_to_work_rects(&src);
        self.add_rect_to_back_rects(&dest);
    }

    //--------------------------------------------------------------  RenderBands

    pub fn render_bands(&mut self) {
        if self.num_bands == 0 {
            return;
        }

        for i in 0..self.num_bands as usize {
            let mut dest = self.bands[i].dest;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            let s = self.band_rects[self.bands[i].mode as usize];
            self.work_src_map.copy_mask(&self.bands_src_map, &self.bands_mask_map, s, s, dest);

            self.add_rect_to_work_rects(&dest);
            self.add_rect_to_back_rects(&dest);
        }
    }

    //--------------------------------------------------------------  RenderShreds

    pub fn render_shreds(&mut self) {
        if self.num_shredded > 0 {
            for i in 0..self.num_shredded as usize {
                if self.shreds[i].frame == 0 {
                    self.shreds[i].bounds.bottom += 1;
                    let b = self.shreds[i].bounds;
                    let high = b.bottom - b.top;
                    if high >= 35 {
                        self.shreds[i].frame = 1;
                    }
                    let mut src = self.shred_src_rect;
                    src.top = src.bottom - high;
                    let mut dest = b;
                    q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
                    self.work_src_map
                        .copy_mask(&self.shred_src_map, &self.shred_mask_map, src, src, dest);
                    self.add_rect_to_back_rects(&dest);
                    dest.top -= 1;
                    self.add_rect_to_work_rects(&dest);
                    self.play_priority_sound(SHRED_SOUND, SHRED_PRIORITY);
                } else if self.shreds[i].frame < 20 {
                    self.shreds[i].bounds.top += 4;
                    self.shreds[i].bounds.bottom += 4;
                    let mut dest = self.shreds[i].bounds;
                    q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
                    self.shreds[i].frame += 1;
                    if self.shreds[i].frame < 20 {
                        let s = self.shred_src_rect;
                        self.work_src_map.copy_mask(&self.shred_src_map, &self.shred_mask_map, s, s, dest);
                    } else {
                        // AddSparkle offsets the shred's own bounds, as in the C
                        let mut b = self.shreds[i].bounds;
                        self.add_sparkle(&mut b);
                        self.shreds[i].bounds = b;
                        self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                    }
                    self.add_rect_to_back_rects(&dest);
                    dest.top -= 4;
                    self.add_rect_to_work_rects(&dest);
                }
            }
        }
    }

    //--------------------------------------------------------------  CopyRectsQD

    pub fn copy_rects_qd(&mut self) {
        for i in 0..self.num_work2_main as usize {
            let r = self.work2_main_rects[i];
            self.main_window_map.copy_bits(&self.work_src_map, r, r);
        }

        for i in 0..self.num_back2_work as usize {
            let r = self.back2_work_rects[i];
            self.work_src_map.copy_bits(&self.back_src_map, r, r);
        }
    }

    // CopyRectsAssm (the 68k assembly variant of CopyRectsQD): not translated.

    //--------------------------------------------------------------  RenderFrame

    pub fn render_frame(&mut self) {
        if self.has_mirror {
            self.draw_reflection(0, true);
            if self.two_player_game {
                self.draw_reflection(1, false);
            }
        }
        self.handle_grease();
        self.render_pendulums();
        if self.even_frame {
            self.render_flames();
        } else {
            self.render_stars();
        }
        self.render_dynamics();
        self.render_flying_points();
        self.render_sparkles();
        self.render_glider(0, true);
        if self.two_player_game {
            self.render_glider(1, false);
        }
        self.render_shreds();
        self.render_bands();

        // while (TickCount() < nextFrame) {}: the play loop paces the
        // frames here (ticks advance by the loop).
        self.next_frame = self.ticks + TICKS_PER_FRAME;

        self.copy_rects_qd();

        self.num_work2_main = 0;
        self.num_back2_work = 0;
    }

    //--------------------------------------------------------------  InitGarbageRects

    pub fn init_garbage_rects(&mut self) {
        self.garbage_rects_allocated();
        self.num_work2_main = 0;
        self.num_back2_work = 0;

        self.num_sparkles = 0;
        for i in 0..MAX_SPARKLES as usize {
            self.sparkles[i].mode = -1;
        }

        self.num_flying_pts = 0;
        for i in 0..MAX_FLYING_PTS as usize {
            self.flying_points[i].mode = -1;
        }

        self.next_frame = self.ticks + TICKS_PER_FRAME;
    }

    //--------------------------------------------------------------  CopyRectBackToWork

    pub fn copy_rect_back_to_work(&mut self, the_rect: &Rect) {
        self.work_src_map.copy_bits(&self.back_src_map, *the_rect, *the_rect);
    }

    //--------------------------------------------------------------  CopyRectWorkToBack

    pub fn copy_rect_work_to_back(&mut self, the_rect: &Rect) {
        self.back_src_map.copy_bits(&self.work_src_map, *the_rect, *the_rect);
    }

    //--------------------------------------------------------------  CopyRectWorkToMain

    pub fn copy_rect_work_to_main(&mut self, the_rect: &Rect) {
        self.main_window_map.copy_bits(&self.work_src_map, *the_rect, *the_rect);
    }

    //--------------------------------------------------------------  CopyRectMainToWork

    pub fn copy_rect_main_to_work(&mut self, the_rect: &Rect) {
        self.work_src_map.copy_bits(&self.main_window_map, *the_rect, *the_rect);
    }

    //--------------------------------------------------------------  CopyRectMainToBack

    pub fn copy_rect_main_to_back(&mut self, the_rect: &Rect) {
        self.back_src_map.copy_bits(&self.main_window_map, *the_rect, *the_rect);
    }

    //--------------------------------------------------------------  AddToMirrorRegion

    pub fn add_to_mirror_region(&mut self, the_rect: &Rect) {
        match &self.mirror_rgn {
            None => self.mirror_rgn = Some(Region::rect(*the_rect)),
            Some(m) => self.mirror_rgn = Some(m.union(&Region::rect(*the_rect))),
        }
        self.has_mirror = true;
    }

    //--------------------------------------------------------------  ZeroMirrorRegion

    pub fn zero_mirror_region(&mut self) {
        self.mirror_rgn = None;
        self.has_mirror = false;
    }
}

//! Dynamics.c: collisions with, drawing and handling of the moving
//! ("dynamic") objects: toast, balloons, copters, darts, appliances...

use super::*;

const SHOVE_VELOCITY: i32 = 8;

//==============================================================  Functions

impl Game {
    //--------------------------------------------------------------  CheckDynamicCollision

    // Checks for a collision betwen the glider and one of the dynamic objects.
    // For example, did the glider hit a flying piece of toast?

    pub fn check_dynamic_collision(&mut self, who: i32, g: usize, do_offset: bool) {
        let w = who as usize;
        let mut dinah_rect = self.dinahs[w].dest;
        if do_offset {
            q_offset_rect(&mut dinah_rect, -self.play_origin_h, -self.play_origin_v);
        }

        if self.sect_glider(g, &dinah_rect, true) {
            let mode = self.gliders[g].mode;
            if mode == GLIDER_NORMAL
                || mode == GLIDER_FACE_LEFT
                || mode == GLIDER_FACE_RIGHT
                || mode == GLIDER_BURNING
                || mode == GLIDER_GOING_FOIL
                || mode == GLIDER_LOSING_FOIL
            {
                if self.foil_total > 0 || mode == GLIDER_LOSING_FOIL {
                    if is_rect_left_of_rect(&dinah_rect, &self.gliders[g].dest) {
                        self.gliders[g].h_desired_vel = SHOVE_VELOCITY;
                    } else {
                        self.gliders[g].h_desired_vel = -SHOVE_VELOCITY;
                    }
                    if self.dinahs[w].v_vel < 0 {
                        self.gliders[g].v_desired_vel = self.dinahs[w].v_vel;
                    }
                    self.play_priority_sound(FOIL_HIT_SOUND, FOIL_HIT_PRIORITY);
                    if self.even_frame && self.foil_total > 0 {
                        self.foil_total -= 1;
                        if self.foil_total <= 0 {
                            self.start_glider_foil_losing(g);
                        }
                    }
                } else {
                    self.start_glider_fading_out(g);
                    self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                }
            }
        }
    }

    /// The test every handler repeats in the C: check the dynamic against
    /// whichever gliders are playing. `do_offset` is what the C passes when
    /// both (or the only) glider are checked; with one of two players left
    /// it always passes false.
    pub(crate) fn check_dynamic_collision_players(&mut self, who: i32, do_offset: bool) {
        if self.two_player_game {
            if self.one_player_left {
                if self.player_dead == self.gliders[0].which {
                    self.check_dynamic_collision(who, 1, false);
                } else {
                    self.check_dynamic_collision(who, 0, false);
                }
            } else {
                self.check_dynamic_collision(who, 0, do_offset);
                self.check_dynamic_collision(who, 1, do_offset);
            }
        } else {
            self.check_dynamic_collision(who, 0, do_offset);
        }
    }

    //--------------------------------------------------------------  DidBandHitDynamic

    // Checks to see if a rubber band struck a dynamic.

    pub fn did_band_hit_dynamic(&self, who: i32) -> bool {
        let dinah_rect = self.dinahs[who as usize].dest;
        let mut collided = false;

        for i in 0..self.num_bands as usize {
            let b = self.bands[i].dest;
            collided = !(b.bottom < dinah_rect.top || b.top > dinah_rect.bottom || b.right < dinah_rect.left || b.left > dinah_rect.right);
            if collided {
                break;
            }
        }

        collided
    }

    //--------------------------------------------------------------  RenderToast

    // The following handful of functions handle drawing specific "dynamic" objecsts.

    pub fn render_toast(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            let mut dest = self.dinahs[w].dest;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            let mut src = self.bread_src[self.dinahs[w].frame as usize];
            let v_clip = self.dinahs[w].dest.bottom - self.dinahs[w].h_vel;
            if v_clip > 0 {
                src.bottom -= v_clip;
                dest.bottom -= v_clip;
            }

            self.work_src_map
                .copy_mask_ink(self.sil("toast"), &self.toast_src_map, &self.toast_mask_map, src, src, dest);

            self.add_rect_to_back_rects(&dest);
            let mut dest = self.dinahs[w].whole;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);
        }
    }

    //--------------------------------------------------------------  RenderBalloon

    pub fn render_balloon(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            let mut dest = self.dinahs[w].dest;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            let src = self.balloon_src[self.dinahs[w].frame as usize];

            self.work_src_map
                .copy_mask_ink(self.sil("balloon"), &self.balloon_src_map, &self.balloon_mask_map, src, src, dest);

            self.add_rect_to_back_rects(&dest);
            let mut dest = self.dinahs[w].whole;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);
        }
    }

    //--------------------------------------------------------------  RenderCopter

    pub fn render_copter(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            let mut dest = self.dinahs[w].dest;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            let src = self.copter_src[self.dinahs[w].frame as usize];

            self.work_src_map
                .copy_mask_ink(self.sil("copter"), &self.copter_src_map, &self.copter_mask_map, src, src, dest);

            self.add_rect_to_back_rects(&dest);
            let mut dest = self.dinahs[w].whole;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);
        }
    }

    //--------------------------------------------------------------  RenderDart

    pub fn render_dart(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            let mut dest = self.dinahs[w].dest;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            let src = self.dart_src[self.dinahs[w].frame as usize];

            self.work_src_map.copy_mask_ink(self.sil("dart"), &self.dart_src_map, &self.dart_mask_map, src, src, dest);

            self.add_rect_to_back_rects(&dest);
            let mut dest = self.dinahs[w].whole;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);
        }
    }

    //--------------------------------------------------------------  RenderBall

    pub fn render_ball(&mut self, who: i32) {
        let w = who as usize;
        let mut dest = self.dinahs[w].dest;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
        let src = self.ball_src[self.dinahs[w].frame as usize];

        self.work_src_map.copy_mask_ink(self.sil("ball"), &self.ball_src_map, &self.ball_mask_map, src, src, dest);

        self.add_rect_to_back_rects(&dest);
        let mut dest = self.dinahs[w].whole;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
        self.add_rect_to_work_rects(&dest);
    }

    //--------------------------------------------------------------  RenderDrip

    pub fn render_drip(&mut self, who: i32) {
        let w = who as usize;
        let mut dest = self.dinahs[w].dest;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
        let src = self.drip_src[self.dinahs[w].frame as usize];

        self.work_src_map.copy_mask_ink(self.sil("drip"), &self.drip_src_map, &self.drip_mask_map, src, src, dest);

        self.add_rect_to_back_rects(&dest);
        let mut dest = self.dinahs[w].whole;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
        self.add_rect_to_work_rects(&dest);
    }

    //--------------------------------------------------------------  RenderFish

    pub fn render_fish(&mut self, who: i32) {
        let w = who as usize;
        let mut dest = self.dinahs[w].dest;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
        let src = self.fish_src[self.dinahs[w].frame as usize];

        if self.dinahs[w].moving {
            self.work_src_map.copy_mask_ink(self.sil("fish"), &self.fish_src_map, &self.fish_mask_map, src, src, dest);
            self.add_rect_to_back_rects(&dest);
            let mut dest = self.dinahs[w].whole;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);
        } else {
            self.work_src_map.copy_bits(&self.fish_src_map, src, dest);
            self.add_rect_to_back_rects(&dest);
            let mut dest = self.dinahs[w].whole;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);
        }
    }

    //--------------------------------------------------------------  HandleSparkleObject

    // The following handful of functions are called each game frame and handle
    // the movement and state of the various types of dynamic objects.

    pub fn handle_sparkle_object(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].active {
            // is it on?
            if self.dinahs[w].frame <= 0 {
                // it is idle
                self.dinahs[w].timer -= 1;
                if self.dinahs[w].timer <= 0 {
                    self.dinahs[w].timer = self.random_int(240) + 60; // reset timer
                    self.dinahs[w].frame = NUM_SPARKLE_MODES; // time to sparkle
                    let mut temp_rect = self.dinahs[w].dest;
                    self.add_sparkle(&mut temp_rect);
                    self.play_priority_sound(MYSTIC_SOUND, MYSTIC_PRIORITY);
                }
            } else {
                // it's sparkling
                self.dinahs[w].frame -= 1;
            }
        }
    }

    //--------------------------------------------------------------  HandleToast

    pub fn handle_toast(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            if self.even_frame {
                self.dinahs[w].frame += 1;
                if self.dinahs[w].frame >= NUM_BREAD_PICTS {
                    self.dinahs[w].frame = 0;
                }
            }
            self.check_dynamic_collision_players(who, false);
            let d = &mut self.dinahs[w];
            v_offset_rect(&mut d.dest, d.v_vel);
            d.whole = d.dest;
            if d.v_vel > 0 {
                d.whole.top -= d.v_vel;
            } else {
                d.whole.bottom -= d.v_vel;
            }
            d.v_vel += 1; // falls
            if d.v_vel > d.count {
                let mut dest = d.whole;
                q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
                self.add_rect_to_work_rects(&dest);
                let d = &mut self.dinahs[w];
                d.moving = false;
                d.frame = d.timer;
                self.play_priority_sound(TOAST_LAND_SOUND, TOAST_LAND_PRIORITY);
            }
        } else {
            if self.dinahs[w].active {
                self.dinahs[w].frame -= 1;
            }
            if self.dinahs[w].frame <= 0 {
                if self.dinahs[w].active {
                    let d = &mut self.dinahs[w];
                    d.v_vel = -d.count;
                    d.frame = 0;
                    d.moving = true;
                    self.play_priority_sound(TOAST_LAUNCH_SOUND, TOAST_LAUNCH_PRIORITY);
                } else {
                    self.dinahs[w].frame = self.dinahs[w].timer;
                }
            }
        }
    }

    /// The appliance handlers' "copy this picture of the appliance to the
    /// back map at the dynamic's rect" (CopyBits applianceSrcMap ->
    /// backSrcMap, then AddRectToBackRects).
    fn appliance_to_back(&mut self, src: Rect, w: usize) {
        let dest = self.dinahs[w].dest;
        self.back_src_map.copy_bits(&self.appliance_src_map, src, dest);
        self.add_rect_to_back_rects(&dest);
    }

    fn dinah_to_work_rects(&mut self, w: usize) {
        let dest = self.dinahs[w].dest;
        self.add_rect_to_work_rects(&dest);
    }

    //--------------------------------------------------------------  HandleMacPlus

    pub fn handle_mac_plus(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].timer > 0 {
            self.dinahs[w].timer -= 1;
            let timer = self.dinahs[w].timer;
            if self.dinahs[w].active {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_BEEP_SOUND, MAC_BEEP_PRIORITY);
                    self.appliance_to_back(self.plus_screen2, w);
                } else if timer == 30 {
                    self.play_priority_sound(MAC_ON_SOUND, MAC_ON_PRIORITY);
                }
            } else {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_OFF_SOUND, MAC_OFF_PRIORITY);
                    self.appliance_to_back(self.plus_screen1, w);
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleTV

    pub fn handle_tv(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].timer > 0 {
            self.dinahs[w].timer -= 1;
            let timer = self.dinahs[w].timer;
            if self.dinahs[w].active {
                // The C skips the screen when a QuickTime movie plays on
                // this TV (thisMac.hasQT && hasMovie && tvInRoom &&
                // who == tvWithMovieNumber); there are no movies here.
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(TV_ON_SOUND, TV_ON_PRIORITY);
                    self.appliance_to_back(self.tv_screen2, w);
                }
            } else {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(TV_OFF_SOUND, TV_OFF_PRIORITY);
                    self.appliance_to_back(self.tv_screen1, w);
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleCoffee

    pub fn handle_coffee(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].timer > 0 {
            self.dinahs[w].timer -= 1;
            let timer = self.dinahs[w].timer;
            if self.dinahs[w].active {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                    self.dinahs[w].timer = 200 + self.random_int(200);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_ON_SOUND, MAC_ON_PRIORITY);
                    self.appliance_to_back(self.coffee_light2, w);
                } else if timer == 100 {
                    self.play_priority_sound(COFFEE_SOUND, COFFEE_PRIORITY);
                    self.dinahs[w].timer = 200 + self.random_int(200);
                }
            } else {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_OFF_SOUND, MAC_OFF_PRIORITY);
                    self.appliance_to_back(self.coffee_light1, w);
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleOutlet

    pub fn handle_outlet(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].position != 0 {
            self.dinahs[w].timer -= 1;

            self.check_dynamic_collision_players(who, true);

            if self.dinahs[w].timer <= 0 {
                let d = &mut self.dinahs[w];
                d.frame = 0;
                d.position = 0;
                d.timer = d.count;
            } else {
                if self.dinahs[w].timer % 5 == 0 {
                    self.play_priority_sound(ZAP_SOUND, ZAP_PRIORITY);
                }
                self.dinahs[w].frame += 1;
                if self.dinahs[w].frame >= NUM_OUTLET_PICTS {
                    self.dinahs[w].frame = 1;
                }
            }

            let d = self.dinahs[w];
            if d.position != 0 || d.h_vel > 0 {
                self.work_src_map
                    .copy_bits(&self.appliance_src_map, self.outlet_src[d.frame as usize], d.dest);
            } else {
                // SetPort((GrafPtr)workSrcMap);
                self.work_src_map.paint_rect(d.dest);
            }
            self.add_rect_to_work_rects(&d.dest);
        } else {
            if self.dinahs[w].active {
                self.dinahs[w].timer -= 1;
            }

            if self.dinahs[w].timer <= 0 {
                if self.dinahs[w].active {
                    self.dinahs[w].position = 1;
                    self.dinahs[w].timer = LENGTH_OF_ZAP;
                    self.play_priority_sound(ZAP_SOUND, ZAP_PRIORITY);
                } else {
                    self.dinahs[w].timer = self.dinahs[w].count;
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleVCR

    pub fn handle_vcr(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].timer > 0 {
            self.dinahs[w].timer -= 1;
            let timer = self.dinahs[w].timer;
            if self.dinahs[w].active {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                    self.dinahs[w].timer = 115;
                } else if timer == 5 {
                    self.play_priority_sound(MAC_ON_SOUND, MAC_ON_PRIORITY);
                } else if timer == 1 {
                    self.play_priority_sound(VCR_SOUND, VCR_PRIORITY);
                    self.appliance_to_back(self.vcr_time2, w);
                } else if timer == 100 {
                    self.dinah_to_work_rects(w);
                    self.dinahs[w].timer = 115;
                    self.dinahs[w].frame = 1 - self.dinahs[w].frame;
                } else if timer == 101 {
                    if self.dinahs[w].frame == 0 {
                        self.appliance_to_back(self.vcr_time2, w);
                    } else {
                        self.appliance_to_back(self.vcr_time1, w);
                    }
                }
            } else {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_OFF_SOUND, MAC_OFF_PRIORITY);
                    self.appliance_to_back(self.vcr_time1, w);
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleStereo

    pub fn handle_stereo(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].timer > 0 {
            self.dinahs[w].timer -= 1;
            let timer = self.dinahs[w].timer;
            if self.dinahs[w].active {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                    self.toggle_music_while_playing();
                } else if timer == 1 {
                    self.play_priority_sound(MAC_ON_SOUND, MAC_ON_PRIORITY);
                    self.appliance_to_back(self.stereo_light2, w);
                }
            } else {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                    self.toggle_music_while_playing();
                } else if timer == 1 {
                    self.play_priority_sound(MAC_OFF_SOUND, MAC_OFF_PRIORITY);
                    self.appliance_to_back(self.stereo_light1, w);
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleMicrowave

    pub fn handle_microwave(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].timer > 0 {
            self.dinahs[w].timer -= 1;
            let timer = self.dinahs[w].timer;
            if self.dinahs[w].active {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_ON_SOUND, MAC_ON_PRIORITY);
                    self.microwave_lights(w, self.micro_on);
                }
            } else {
                if timer == 0 {
                    self.dinah_to_work_rects(w);
                } else if timer == 1 {
                    self.play_priority_sound(MAC_OFF_SOUND, MAC_OFF_PRIORITY);
                    self.microwave_lights(w, self.micro_off);
                }
            }
        }
    }

    /// HandleMicrowave's three 16-pixel lights copied side by side.
    fn microwave_lights(&mut self, w: usize, src: Rect) {
        let mut dest = self.dinahs[w].dest;
        dest.right = dest.left + 16;
        self.back_src_map.copy_bits(&self.appliance_src_map, src, dest);
        q_offset_rect(&mut dest, 16, 0);
        self.back_src_map.copy_bits(&self.appliance_src_map, src, dest);
        q_offset_rect(&mut dest, 16, 0);
        self.back_src_map.copy_bits(&self.appliance_src_map, src, dest);
        let r = self.dinahs[w].dest;
        self.add_rect_to_back_rects(&r);
    }
}

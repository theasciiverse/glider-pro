//! RubberBands.c: the rubber bands the glider fires.

use super::*;

pub const RUBBER_BAND_VELOCITY: i32 = 20;
pub const BAND_FALL_COUNT: i32 = 4;
pub const KILL_BAND_MODE: i32 = -1;

impl Game {
    //--------------------------------------------------------------  CheckBandCollision

    pub fn check_band_collision(&mut self, who: i32) {
        let who = who as usize;
        let mut collided: bool;
        let mut nothing_collided = true;

        if self.left_thresh == LEFT_WALL_LIMIT && self.bands[who].dest.left < LEFT_WALL_LIMIT {
            if self.bands[who].h_vel < 0 {
                self.bands[who].h_vel = -self.bands[who].h_vel;
            }
            self.bands[who].dest.left = LEFT_WALL_LIMIT;
            self.bands[who].dest.right = self.bands[who].dest.left + 16;
            self.play_priority_sound(BAND_REBOUND_SOUND, BAND_REBOUND_PRIORITY);
        } else if self.right_thresh == RIGHT_WALL_LIMIT
            && self.bands[who].dest.right > RIGHT_WALL_LIMIT
        {
            if self.bands[who].h_vel > 0 {
                self.bands[who].h_vel = -self.bands[who].h_vel;
            }
            self.bands[who].dest.right = RIGHT_WALL_LIMIT;
            self.bands[who].dest.left = self.bands[who].dest.right - 16;
            self.play_priority_sound(BAND_REBOUND_SOUND, BAND_REBOUND_PRIORITY);
        }

        for i in 0..self.n_hot_spots as usize {
            if self.hot_spots[i].is_on {
                let action = self.hot_spots[i].action;
                if action == DISSOLVE_IT
                    || action == REWARD_IT
                    || action == SWITCH_IT
                    || action == TRIGGER_IT
                    || action == BOUNCE_IT
                {
                    let bd = self.bands[who].dest;
                    let hb = self.hot_spots[i].bounds;
                    collided = !(bd.bottom < hb.top
                        || bd.top > hb.bottom
                        || bd.right < hb.left
                        || bd.left > hb.right);

                    if collided {
                        nothing_collided = false; // we have detected a collision
                        if self.band_hit_last != i as i32 {
                            // don't count it if same as last frame
                            // we don't want rapid on/off toggles
                            self.band_hit_last = i as i32; // note who so we don't double-toggle it
                            if action == DISSOLVE_IT || action == BOUNCE_IT {
                                let b = &mut self.bands[who];
                                if b.h_vel > 0 {
                                    if (b.dest.right - b.h_vel) < hb.left {
                                        b.h_vel = -b.h_vel;
                                        b.dest.right = hb.left;
                                        b.dest.left = b.dest.right - 16;
                                    } else {
                                        b.mode = KILL_BAND_MODE;
                                    }
                                } else {
                                    if (b.dest.left - b.h_vel) > hb.right {
                                        b.h_vel = -b.h_vel;
                                        b.dest.left = hb.right;
                                        b.dest.right = b.dest.left + 16;
                                    } else {
                                        b.mode = KILL_BAND_MODE;
                                    }
                                }
                                self.play_priority_sound(BAND_REBOUND_SOUND, BAND_REBOUND_PRIORITY);
                                break;
                            } else if action == REWARD_IT {
                                let who_linked = self.hot_spots[i].who;
                                let mo = self.master_objects[who_linked as usize];
                                if mo.the_object.what == GREASE_RT
                                    || mo.the_object.what == GREASE_LF
                                {
                                    if self.set_object_state(
                                        self.this_room_number,
                                        mo.object_num,
                                        0,
                                        who_linked,
                                    ) {
                                        self.spill_grease(mo.dyna_num, mo.hot_num);
                                    }
                                    self.hot_spots[i].is_on = false;
                                }
                            } else if action == SWITCH_IT {
                                self.handle_switches(i);
                            } else if action == TRIGGER_IT {
                                self.arm_trigger(i);
                            }
                        }
                    }
                }
            }
        }

        if nothing_collided {
            // the rubberband has hit nothing
            self.band_hit_last = -1; // so make note of that for the next time
        }

        if self.bands[who].h_vel != 0 {
            let bd = self.bands[who].dest;
            let gd = self.gliders[0].dest;
            collided = !(bd.bottom < gd.top
                || bd.top > gd.bottom
                || bd.right < gd.left
                || bd.left > gd.right);

            if collided {
                if !self.two_player_game || !self.one_player_left || self.player_dead == PLAYER2 {
                    self.gliders[0].h_vel += self.bands[who].h_vel / 2;
                    self.bands[who].h_vel = 0;
                    self.play_priority_sound(HIT_WALL_SOUND, HIT_WALL_PRIORITY);
                }
            }

            if self.two_player_game {
                let bd = self.bands[who].dest;
                let gd = self.gliders[1].dest;
                collided = !(bd.bottom < gd.top
                    || bd.top > gd.bottom
                    || bd.right < gd.left
                    || bd.left > gd.right);

                if collided {
                    if !self.one_player_left || self.player_dead == PLAYER1 {
                        self.gliders[1].h_vel += self.bands[who].h_vel / 2;
                        self.bands[who].h_vel = 0;
                        self.play_priority_sound(HIT_WALL_SOUND, HIT_WALL_PRIORITY);
                    }
                }
            }
        }
        if self.bands[who].dest.left < LEFT_WALL_LIMIT
            || self.bands[who].dest.right > RIGHT_WALL_LIMIT
        {
            self.bands[who].mode = KILL_BAND_MODE;
        } else if self.bands[who].dest.bottom > FLOOR_LIMIT {
            self.bands[who].mode = KILL_BAND_MODE;
        }
    }

    //--------------------------------------------------------------  HandleBands

    pub fn handle_bands(&mut self) {
        if self.num_bands == 0 {
            return;
        }

        for i in 0..self.num_bands as usize {
            self.bands[i].mode += 1;
            if self.bands[i].mode > 2 {
                self.bands[i].mode = 0;
            }

            self.bands[i].count += 1;
            if self.bands[i].count >= BAND_FALL_COUNT {
                self.bands[i].v_vel += 1;
                self.bands[i].count = 0;
            }

            let mut dest = self.bands[i].dest;
            q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&dest);

            let b = &mut self.bands[i];
            b.dest.left += b.h_vel;
            b.dest.right += b.h_vel;
            b.dest.top += b.v_vel;
            b.dest.bottom += b.v_vel;

            self.check_band_collision(i as i32);
        }

        let mut count = 0usize;
        loop {
            while self.bands[count].mode == KILL_BAND_MODE {
                self.bands[count].mode = 0;
                self.kill_band(count as i32);
            }
            count += 1;
            if count as i32 >= self.num_bands {
                break;
            }
        }
    }

    //--------------------------------------------------------------  AddBand

    pub fn add_band(&mut self, g: usize, h: i32, v: i32, direction: bool) -> bool {
        if self.num_bands >= MAX_RUBBER_BANDS {
            return false;
        }

        let n = self.num_bands as usize;
        let tipped = self.gliders[g].tipped;
        let b = &mut self.bands[n];
        b.mode = 0;
        b.count = 0;
        if tipped {
            b.v_vel = -2;
        } else {
            b.v_vel = 0;
        }
        b.dest.left = h - 8;
        b.dest.right = h + 8;
        b.dest.top = v - 3;
        b.dest.bottom = v + 3;

        if direction == FACE_LEFT {
            b.dest.left -= 32;
            b.dest.right -= 32;
            b.h_vel = -RUBBER_BAND_VELOCITY;
        } else {
            b.dest.left += 32;
            b.dest.right += 32;
            b.h_vel = RUBBER_BAND_VELOCITY;
        }

        let hv = b.h_vel;
        self.gliders[g].h_vel -= hv / 2;
        self.num_bands += 1;

        self.play_priority_sound(FIRE_BAND_SOUND, FIRE_BAND_PRIORITY);
        true
    }

    //--------------------------------------------------------------  KillBand

    pub fn kill_band(&mut self, which: i32) {
        let last_band = self.num_bands - 1;
        if which != last_band {
            self.bands[which as usize] = self.bands[last_band as usize];
        }

        self.num_bands -= 1;
    }

    //--------------------------------------------------------------  KillAllBands

    pub fn kill_all_bands(&mut self) {
        for i in 0..MAX_RUBBER_BANDS as usize {
            self.bands[i].mode = 0;
        }

        self.num_bands = 0;
    }
}

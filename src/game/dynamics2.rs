//! Dynamics2.c: the handlers of the enemies (balloons, copters, darts,
//! balls, drips, fish).

use super::*;

const BALLOON_STOP: i32 = 8;
const BALLOON_START: i32 = 310;
const COPTER_START: i32 = 8;
const COPTER_STOP: i32 = 310;
const DART_VELOCITY: i32 = 6;
const DART_STOP: i32 = 310;
const ENEMY_DROP_SPEED: i32 = 8;

//==============================================================  Functions

impl Game {
    /// The idle branch shared by balloon, copter and dart: count down to
    /// appearing, with a sparkle kStartSparkle frames before (or on
    /// appearing, when the delay is shorter than that).
    fn enemy_idle(&mut self, w: usize) {
        if self.dinahs[w].active {
            self.dinahs[w].timer -= 1;
            if self.dinahs[w].timer <= 0 {
                self.dinahs[w].moving = true;
                if self.dinahs[w].count < START_SPARKLE {
                    let mut dest = self.dinahs[w].dest;
                    self.add_sparkle(&mut dest);
                    self.play_priority_sound(ENEMY_IN_SOUND, ENEMY_IN_PRIORITY);
                }
            } else if self.dinahs[w].timer == START_SPARKLE {
                let mut dest = self.dinahs[w].dest;
                self.add_sparkle(&mut dest);
                self.play_priority_sound(ENEMY_IN_SOUND, ENEMY_IN_PRIORITY);
            }
        }
    }

    /// The "gone out of the room" start shared by balloon, copter and dart:
    /// refresh where it was, sparkle where it is.
    fn enemy_out(&mut self, w: usize) {
        let mut dest = self.dinahs[w].whole;
        q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
        self.add_rect_to_work_rects(&dest);
        let mut dest = self.dinahs[w].dest;
        self.add_sparkle(&mut dest);
        self.play_priority_sound(ENEMY_OUT_SOUND, ENEMY_OUT_PRIORITY);
    }

    //--------------------------------------------------------------  HandleBalloon

    pub fn handle_balloon(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            if self.dinahs[w].v_vel < 0 {
                if self.even_frame {
                    self.dinahs[w].frame += 1;
                    if self.dinahs[w].frame >= 6 {
                        self.dinahs[w].frame = 0;
                    }
                }
                self.check_dynamic_collision_players(who, false);

                if self.num_bands > 0 && self.did_band_hit_dynamic(who) {
                    self.dinahs[w].frame = 6;
                    self.dinahs[w].v_vel = ENEMY_DROP_SPEED;
                    self.play_priority_sound(POP_SOUND, POP_PRIORITY);
                } else {
                    let d = &mut self.dinahs[w];
                    v_offset_rect(&mut d.dest, d.v_vel);
                    d.whole = d.dest;
                    d.whole.bottom -= d.v_vel;
                }
            } else {
                if self.even_frame {
                    self.dinahs[w].frame += 1;
                    if self.dinahs[w].frame >= 8 {
                        self.dinahs[w].frame = 6;
                    }
                }

                let d = &mut self.dinahs[w];
                v_offset_rect(&mut d.dest, d.v_vel);
                d.whole = d.dest;
                d.whole.top -= d.v_vel;
            }

            if self.dinahs[w].dest.top <= BALLOON_STOP || self.dinahs[w].dest.bottom >= BALLOON_START {
                self.enemy_out(w);
                let tall = rect_tall(&self.balloon_src[0]);
                let d = &mut self.dinahs[w];
                d.moving = false;
                d.v_vel = -2;
                d.timer = d.count;
                d.dest.bottom = BALLOON_START;
                d.dest.top = d.dest.bottom - tall;
                d.whole = d.dest;
            }
        } else {
            // balloon is idle, waiting to appear
            self.enemy_idle(w);
        }
    }

    //--------------------------------------------------------------  HandleCopter

    pub fn handle_copter(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            // is 'copter about?
            if self.dinahs[w].h_vel != 0 {
                // 'copter was not shot
                self.dinahs[w].frame += 1;
                if self.dinahs[w].frame >= 8 {
                    self.dinahs[w].frame = 0;
                }
                self.check_dynamic_collision_players(who, false);
                if self.num_bands > 0 && self.did_band_hit_dynamic(who) {
                    self.dinahs[w].frame = 8;
                    self.dinahs[w].h_vel = 0;
                    self.dinahs[w].v_vel = ENEMY_DROP_SPEED;
                    self.play_priority_sound(PAPER_CRUNCH_SOUND, PAPER_CRUNCH_PRIORITY);
                } else {
                    let d = &mut self.dinahs[w];
                    h_offset_rect(&mut d.dest, d.h_vel);
                    v_offset_rect(&mut d.dest, d.v_vel);
                    d.whole = d.dest;
                    d.whole.top -= d.v_vel;
                    if d.h_vel < 0 {
                        d.whole.right -= d.h_vel;
                    } else {
                        d.whole.left -= d.h_vel;
                    }
                }
            } else {
                // 'copter was shot
                let d = &mut self.dinahs[w];
                d.frame += 1;
                if d.frame >= 10 {
                    d.frame = 8;
                }
                v_offset_rect(&mut d.dest, d.v_vel);
                d.whole = d.dest;
                d.whole.top -= d.v_vel;
            }

            if self.dinahs[w].dest.top <= COPTER_START || self.dinahs[w].dest.bottom >= COPTER_STOP {
                self.enemy_out(w);
                let tall = rect_tall(&self.copter_src[0]);
                let d = &mut self.dinahs[w];
                d.moving = false;
                d.v_vel = 2;
                if d.type_ == COPTER_LF {
                    d.h_vel = -1;
                } else {
                    d.h_vel = 1;
                }
                d.timer = d.count;
                d.dest.top = COPTER_START;
                d.dest.bottom = d.dest.top + tall;
                d.dest.left = d.position;
                d.dest.right = d.dest.left + 32;
                d.whole = d.dest;
            }
        } else {
            self.enemy_idle(w);
        }
    }

    //--------------------------------------------------------------  HandleDart

    pub fn handle_dart(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            // Dart has appeared
            if self.dinahs[w].h_vel != 0 {
                // meaning it isn't falling
                self.check_dynamic_collision_players(who, false);
                if self.num_bands > 0 && self.did_band_hit_dynamic(who) {
                    if self.dinahs[w].type_ == DART_LF {
                        self.dinahs[w].frame = 1;
                    } else {
                        self.dinahs[w].frame = 3;
                    }
                    self.dinahs[w].h_vel = 0;
                    self.dinahs[w].v_vel = ENEMY_DROP_SPEED;
                    self.play_priority_sound(PAPER_CRUNCH_SOUND, PAPER_CRUNCH_PRIORITY);
                } else {
                    let d = &mut self.dinahs[w];
                    h_offset_rect(&mut d.dest, d.h_vel);
                    v_offset_rect(&mut d.dest, d.v_vel);
                    d.whole = d.dest;
                    d.whole.top -= d.v_vel;
                    if d.h_vel < 0 {
                        d.whole.right -= d.h_vel;
                    } else {
                        d.whole.left -= d.h_vel;
                    }
                }
            } else {
                // dart is falling straight down
                let d = &mut self.dinahs[w];
                v_offset_rect(&mut d.dest, d.v_vel);
                d.whole = d.dest;
                d.whole.top -= d.v_vel;
            }

            let dd = self.dinahs[w].dest;
            if dd.left <= 0 || dd.right >= ROOM_WIDE || dd.bottom >= DART_STOP {
                self.enemy_out(w);
                let wide = rect_wide(&self.dart_src[0]);
                let tall = rect_tall(&self.dart_src[0]);
                let d = &mut self.dinahs[w];
                d.moving = false;
                d.v_vel = 2;
                if d.type_ == DART_LF {
                    d.frame = 0;
                    d.h_vel = -DART_VELOCITY;
                    d.dest.right = ROOM_WIDE;
                    d.dest.left = d.dest.right - wide;
                } else {
                    d.frame = 2;
                    d.h_vel = DART_VELOCITY;
                    d.dest.left = 0;
                    d.dest.right = d.dest.left + wide;
                }
                d.timer = d.count;
                d.dest.top = d.position;
                d.dest.bottom = d.dest.top + tall;
                d.whole = d.dest;
            }
        } else {
            self.enemy_idle(w);
        }
    }

    //--------------------------------------------------------------  HandleBall

    pub fn handle_ball(&mut self, who: i32) {
        let w = who as usize;
        self.check_dynamic_collision_players(who, false);

        if self.dinahs[w].moving {
            // is ball bouncing?
            let d = &mut self.dinahs[w];
            v_offset_rect(&mut d.dest, d.v_vel);
            if d.dest.bottom >= d.position {
                // bounce!
                d.whole = d.dest;
                d.whole.top -= d.v_vel;
                d.whole.bottom = d.position;
                d.dest.bottom = d.position;
                d.dest.top = d.dest.bottom - 32;
                if d.active {
                    d.v_vel = d.count;
                } else {
                    d.v_vel = -((d.v_vel * 3) / 4);
                    if d.v_vel == 0 {
                        d.moving = false; // stop bounce
                    }
                }
                if d.whole.bottom < d.dest.bottom {
                    d.whole.bottom = d.dest.bottom;
                }
                self.play_priority_sound(BOUNCE_SOUND, BOUNCE_PRIORITY);
                if self.dinahs[w].moving {
                    self.dinahs[w].frame = 1;
                }
            } else {
                d.whole = d.dest;
                if d.v_vel > 0 {
                    d.whole.top -= d.v_vel;
                } else {
                    d.whole.bottom -= d.v_vel;
                }
                if self.even_frame {
                    d.v_vel += 1;
                }
                d.frame = 0;
            }
        } else {
            if self.dinahs[w].active {
                self.dinahs[w].v_vel = self.dinahs[w].count;
                self.dinahs[w].moving = true;
                self.even_frame = true;
            }
        }
    }

    //--------------------------------------------------------------  HandleDrip

    pub fn handle_drip(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            if self.even_frame {
                self.dinahs[w].frame = 9 - self.dinahs[w].frame;
            }
            self.check_dynamic_collision_players(who, false);

            let d = &mut self.dinahs[w];
            v_offset_rect(&mut d.dest, d.v_vel);
            if d.dest.bottom >= d.position {
                let mut dest = d.whole;
                q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
                self.add_rect_to_work_rects(&dest);
                let d = &mut self.dinahs[w];
                d.dest.top = d.h_vel;
                d.dest.bottom = d.dest.top + 12;
                self.play_priority_sound(DROP_SOUND, DROP_PRIORITY);
                let d = &mut self.dinahs[w];
                d.v_vel = 0;
                d.timer = d.count;
                d.frame = 3;
                d.moving = false;
            } else {
                d.whole = d.dest;
                d.whole.top -= d.v_vel;
                if self.even_frame {
                    d.v_vel += 1;
                }
            }
        } else {
            if self.dinahs[w].active {
                self.dinahs[w].timer -= 1;

                let timer = self.dinahs[w].timer;
                if timer == 6 {
                    self.dinahs[w].frame = 0;
                } else if timer == 4 {
                    self.dinahs[w].frame = 1;
                } else if timer == 2 {
                    self.dinahs[w].frame = 2;
                } else if timer <= 0 {
                    let d = &mut self.dinahs[w];
                    v_offset_rect(&mut d.dest, 3);
                    d.whole = d.dest;
                    d.moving = true;
                    d.frame = 4;
                    self.play_priority_sound(DRIP_SOUND, DRIP_PRIORITY);
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleFish

    pub fn handle_fish(&mut self, who: i32) {
        let w = who as usize;
        if self.dinahs[w].moving {
            // fish leaping
            if self.dinahs[w].v_vel >= 0 && self.dinahs[w].frame < 7 {
                self.dinahs[w].frame += 1;
            }
            self.check_dynamic_collision_players(who, false);

            let d = &mut self.dinahs[w];
            v_offset_rect(&mut d.dest, d.v_vel);
            if d.dest.bottom >= d.position {
                // splash down
                let mut dest = d.whole;
                q_offset_rect(&mut dest, self.play_origin_h, self.play_origin_v);
                self.add_rect_to_work_rects(&dest);
                let d = &mut self.dinahs[w];
                d.dest.bottom = d.position;
                d.dest.top = d.dest.bottom - 16;
                d.whole = d.dest;
                d.whole.top -= 2;
                self.play_priority_sound(DROP_SOUND, DROP_PRIORITY);
                let d = &mut self.dinahs[w];
                d.v_vel = d.count;
                d.timer = d.h_vel;
                d.frame = 0;
                d.moving = false;
                self.play_priority_sound(FISH_IN_SOUND, FISH_IN_PRIORITY);
            } else {
                d.whole = d.dest;
                if d.v_vel > 0 {
                    d.whole.top -= d.v_vel;
                } else {
                    d.whole.bottom -= d.v_vel;
                }
                if self.even_frame {
                    d.v_vel += 1;
                }
            }
        } else {
            // fish idle
            let d = &mut self.dinahs[w];
            d.whole = d.dest;
            if (d.timer & 0x0003) == 0x0003 {
                d.frame += 1;
                if d.frame > 3 {
                    d.frame = 0;
                }
                if d.frame == 1 || d.frame == 2 {
                    d.dest.top += 1;
                    d.dest.bottom += 1;
                    d.whole.bottom += 1;
                } else {
                    d.dest.top -= 1;
                    d.dest.bottom -= 1;
                    d.whole.top -= 1;
                }
            }
            if d.active {
                d.timer -= 1;
                if d.timer <= 0 {
                    // fish leaps
                    d.whole = d.dest;
                    d.moving = true;
                    d.frame = 4;
                    self.play_priority_sound(FISH_OUT_SOUND, FISH_OUT_PRIORITY);
                }
            }
        }
    }
}

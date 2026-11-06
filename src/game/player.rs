//! Player.c: moving the glider in each of its modes (drawing it is Render.c).

use super::*;

pub const GRAVITY: i32 = 3;
pub const H_IMPULSE: i32 = 2;
pub const V_IMPULSE: i32 = 2;
pub const MAX_H_VEL: i32 = 16;
pub const SHREDDER_COUNTDOWN: i32 = -68;

pub const CLIMB_STAIRS_SPEED: i32 = -4;
pub const V_CLIMB_STAIRS_SPEED: i32 = -4;
pub const H_CLIMB_STAIRS_SPEED: i32 = -4;
pub const V_DROP_STAIRS_SPEED: i32 = 4;
pub const H_DROP_STAIRS_SPEED: i32 = 4;
pub const V_DROP_DUCT_SPEED: i32 = 4;
pub const V_RISE_DUCT_SPEED: i32 = -4;
pub const H_PUSH_MAIL_SPEED: i32 = -4;
pub const H_PUSH_MAIL_RT_SPEED: i32 = 4;
pub const H_MAIL_PULL_SPEED: i32 = 4;
pub const H_MAIL_PULL_RT_SPEED: i32 = -4;
pub const V_MAIL_DROP_SPEED: i32 = 2;
pub const DROP_SHRED_SLOW: i32 = 1;
pub const DROP_SHRED_FAST: i32 = 4;

/// How a glider leaves the room at the end of a stairs, transporter, duct
/// or mailbox animation (the C repeats the same block for each).
#[derive(Clone, Copy, Debug)]
enum Exit {
    Room(i32),
    Transport,
    Duct,
    Mail,
}

impl Game {
    /// `thisGlider->src = gliderSrc[i]; thisGlider->mask = gliderSrc[i];`
    fn pl_set_src(&mut self, g: usize, i: i32) {
        let r = self.glider_src[i as usize];
        self.gliders[g].src = r;
        self.gliders[g].mask = r;
    }

    /// The fade (in, out, transporter) frame: gliderSrc of the fade
    /// sequence at `frame`, offset for a glider facing left.
    fn pl_set_fade_src(&mut self, g: usize) {
        let gl = self.gliders[g];
        let mut i = self.fade_in_sequence[gl.frame as usize];
        if gl.facing == FACE_LEFT {
            i += LEFT_FADE_OFFSET;
        }
        self.pl_set_src(g, i);
    }

    /// Normal or burning again, after coming out of stairs, a duct or a
    /// mailbox.
    fn pl_normal_or_burning(&mut self, g: usize) {
        if self.gliders[g].frame == WAS_BURNING {
            self.flag_glider_burning(g);
        } else {
            self.flag_glider_normal(g);
        }
    }

    /// The glider's whole rects copied work to main (it vanished from
    /// the room), and not drawn.
    fn pl_erase_to_main(&mut self, g: usize) {
        let mut temp_rect = self.gliders[g].whole;
        q_offset_rect(&mut temp_rect, self.play_origin_h, self.play_origin_v);
        self.copy_rect_work_to_main(&temp_rect);
        temp_rect = self.gliders[g].whole_shadow;
        q_offset_rect(&mut temp_rect, self.play_origin_h, self.play_origin_v);
        self.copy_rect_work_to_main(&temp_rect);
        self.gliders[g].dont_draw = true;
    }

    fn pl_do_exit(&mut self, g: usize, exit: Exit) {
        match exit {
            Exit::Room(where_) => self.move_room_to_room(g, where_),
            Exit::Transport => self.transport_room_to_room(g),
            Exit::Duct => self.move_duct_to_duct(g),
            Exit::Mail => self.move_mail_to_mail(g),
        }
    }

    /// The glider has left: with two players, it waits in limbo for the
    /// other (marked by `escaped`), or both go when the other already
    /// left the same way; the survivor goes when one player is left.
    fn pl_leave_room(&mut self, g: usize, escaped: i32, exit: Exit) {
        if self.two_player_game {
            if self.one_player_left {
                if self.player_dead == PLAYER1 {
                    self.pl_do_exit(1, exit);
                } else {
                    self.pl_do_exit(0, exit);
                }
            } else {
                if self.other_player_escaped == escaped {
                    self.other_player_escaped = NO_ONE_ESCAPED;
                    self.pl_do_exit(g, exit);
                } else {
                    self.other_player_escaped = escaped;
                    self.refresh_scoreboard(ESCAPED_TITLE_MODE);
                    self.flag_glider_in_limbo(g, true);
                }
            }
        } else {
            self.pl_do_exit(g, exit);
        }
    }

    //--------------------------------------------------------------  MoveGlider

    pub fn move_glider(&mut self, g: usize) {
        let gl = &mut self.gliders[g];
        if gl.h_vel > gl.h_desired_vel {
            gl.h_vel -= H_IMPULSE;
            if gl.h_vel < gl.h_desired_vel {
                gl.h_vel = gl.h_desired_vel;
            }
        } else if gl.h_vel < gl.h_desired_vel {
            gl.h_vel += H_IMPULSE;
            if gl.h_vel > gl.h_desired_vel {
                gl.h_vel = gl.h_desired_vel;
            }
        }
        gl.h_desired_vel = 0;

        if gl.v_vel > gl.v_desired_vel {
            gl.v_vel -= V_IMPULSE;
            if gl.v_vel < gl.v_desired_vel {
                gl.v_vel = gl.v_desired_vel;
            }
        } else if gl.v_vel < gl.v_desired_vel {
            gl.v_vel += V_IMPULSE;
            if gl.v_vel > gl.v_desired_vel {
                gl.v_vel = gl.v_desired_vel;
            }
        }
        gl.v_desired_vel = GRAVITY;

        if gl.h_vel < 0 {
            if gl.h_vel < -MAX_H_VEL {
                gl.h_vel = -MAX_H_VEL;
            }

            gl.was_h_vel = gl.h_vel;

            gl.whole.right = gl.dest.right;
            gl.dest.left += gl.h_vel;
            gl.dest.right += gl.h_vel;
            gl.whole.left = gl.dest.left;

            gl.whole_shadow.right = gl.dest_shadow.right;
            gl.dest_shadow.left += gl.h_vel;
            gl.dest_shadow.right += gl.h_vel;
            gl.whole_shadow.left = gl.dest_shadow.left;
        } else {
            if gl.h_vel > MAX_H_VEL {
                gl.h_vel = MAX_H_VEL;
            }

            gl.was_h_vel = gl.h_vel;

            gl.whole.left = gl.dest.left;
            gl.dest.left += gl.h_vel;
            gl.dest.right += gl.h_vel;
            gl.whole.right = gl.dest.right;

            gl.whole_shadow.left = gl.dest_shadow.left;
            gl.dest_shadow.left += gl.h_vel;
            gl.dest_shadow.right += gl.h_vel;
            gl.whole_shadow.right = gl.dest_shadow.right;
        }

        if gl.v_vel < 0 {
            gl.was_v_vel = gl.v_vel;

            gl.whole.bottom = gl.dest.bottom;
            gl.dest.top += gl.v_vel;
            gl.dest.bottom += gl.v_vel;
            gl.whole.top = gl.dest.top;
        } else {
            gl.was_v_vel = gl.v_vel;

            gl.whole.top = gl.dest.top;
            gl.dest.top += gl.v_vel;
            gl.dest.bottom += gl.v_vel;
            gl.whole.bottom = gl.dest.bottom;
        }
    }

    //--------------------------------------------------------------  MoveGliderNormal

    pub fn move_glider_normal(&mut self, g: usize) {
        let gl = self.gliders[g];
        if gl.facing == FACE_LEFT {
            if gl.sliding {
                self.pl_set_src(g, 30);
                self.gliders[g].sliding = false;
            } else if gl.tipped {
                self.pl_set_src(g, 3);
            } else {
                self.pl_set_src(g, 2);
            }
        } else {
            if gl.sliding {
                self.pl_set_src(g, 29);
                self.gliders[g].sliding = false;
            } else if gl.tipped {
                self.pl_set_src(g, 1);
            } else {
                self.pl_set_src(g, 0);
            }
        }

        self.move_glider(g);
    }

    //--------------------------------------------------------------  MoveGliderBurning

    pub fn move_glider_burning(&mut self, g: usize) {
        self.gliders[g].frame += 1;
        if self.gliders[g].frame > 3 {
            self.gliders[g].frame = 0;
        }

        let frame = self.gliders[g].frame;
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 25 + frame);
        } else {
            self.pl_set_src(g, 21 + frame);
        }

        self.gliders[g].was_mode -= 1;
        if self.gliders[g].was_mode <= 0 {
            self.start_glider_fading_out(g);
            self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
        }
        self.move_glider(g);
    }

    //--------------------------------------------------------------  FadeGliderIn

    pub fn fade_glider_in(&mut self, g: usize) {
        if self.gliders[g].frame == 0 {
            self.play_priority_sound(FADE_IN_SOUND, FADE_IN_PRIORITY);
        }

        self.gliders[g].frame += 1;
        if self.gliders[g].frame >= LAST_FADE_SEQUENCE {
            self.flag_glider_normal(g);
            self.gliders[g].entered_rect = self.gliders[g].dest;
        } else {
            self.pl_set_fade_src(g);
        }
    }

    //--------------------------------------------------------------  TransportGliderIn

    pub fn transport_glider_in(&mut self, g: usize) {
        if self.gliders[g].frame == 0 {
            self.play_priority_sound(TRANS_IN_SOUND, TRANS_IN_PRIORITY);
        }

        self.gliders[g].frame += 1;
        if self.gliders[g].frame >= LAST_FADE_SEQUENCE {
            self.flag_glider_normal(g);
            self.gliders[g].entered_rect = self.gliders[g].dest;
        } else {
            self.pl_set_fade_src(g);
        }
    }

    //--------------------------------------------------------------  FadeGliderOut

    pub fn fade_glider_out(&mut self, g: usize) {
        self.gliders[g].frame -= 1;
        if self.gliders[g].frame < 0 {
            self.off_a_mortal(g);
        } else {
            self.pl_set_fade_src(g);
        }
    }

    //--------------------------------------------------------------  MoveGliderUpStairs

    pub fn move_glider_up_stairs(&mut self, g: usize) {
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 2);
        } else {
            self.pl_set_src(g, 1);
        }

        let gl = &mut self.gliders[g];
        gl.whole.bottom = gl.dest.bottom;
        gl.dest.top += CLIMB_STAIRS_SPEED;
        gl.dest.bottom += CLIMB_STAIRS_SPEED;
        gl.whole.top = gl.dest.top;

        let v_not_clipped = gl.dest.bottom - 29;
        if v_not_clipped < GLIDER_HIGH {
            if v_not_clipped <= 0 {
                gl.dest.top = gl.dest.bottom;
                gl.src.top = gl.src.bottom;
                gl.mask.top = gl.mask.bottom;
                self.taking_the_stairs = true;
                self.pl_leave_room(g, PLAYER_ESCAPED_UP_STAIRS, Exit::Room(ABOVE));
            } else {
                gl.dest.top = gl.dest.bottom - v_not_clipped;
                gl.src.top = gl.src.bottom - v_not_clipped;
                gl.mask.top = gl.mask.bottom - v_not_clipped;
            }
        }
    }

    //--------------------------------------------------------------  FinishGliderUpStairs

    pub fn finish_glider_up_stairs(&mut self, g: usize) {
        self.pl_set_src(g, 2);

        let right_clip = self.right_clip;
        let gl = &mut self.gliders[g];
        gl.whole.bottom = gl.dest.bottom;
        gl.dest.top += V_CLIMB_STAIRS_SPEED;
        gl.dest.bottom += V_CLIMB_STAIRS_SPEED;
        gl.whole.top = gl.dest.top;

        gl.whole.right = gl.dest.right;
        gl.dest.left += H_CLIMB_STAIRS_SPEED;
        gl.dest.right += H_CLIMB_STAIRS_SPEED;
        gl.whole.left = gl.dest.left;

        gl.whole_shadow.right = gl.dest_shadow.right;
        gl.dest_shadow.left += H_CLIMB_STAIRS_SPEED;
        gl.dest_shadow.right += H_CLIMB_STAIRS_SPEED;
        gl.whole_shadow.left = gl.dest_shadow.left;

        let h_not_clipped = right_clip - gl.dest.left;
        if h_not_clipped < GLIDER_WIDE {
            gl.dest.right = gl.dest.left + h_not_clipped;
            gl.src.right = gl.src.left + h_not_clipped;
            gl.mask.right = gl.mask.left + h_not_clipped;
            gl.dest_shadow.right = gl.dest.right;
        } else {
            self.pl_normal_or_burning(g);
            let gl = &mut self.gliders[g];
            gl.h_vel = H_CLIMB_STAIRS_SPEED;
            gl.h_desired_vel = H_CLIMB_STAIRS_SPEED;
            gl.v_vel = V_CLIMB_STAIRS_SPEED;
            gl.v_desired_vel = V_CLIMB_STAIRS_SPEED;
            gl.entered_rect = gl.dest;
        }
    }

    //--------------------------------------------------------------  MoveGliderDownStairs

    pub fn move_glider_down_stairs(&mut self, g: usize) {
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 2);
        } else {
            self.pl_set_src(g, 0);
        }

        let right_clip = self.right_clip;
        let gl = &mut self.gliders[g];
        gl.whole.left = gl.dest.left;
        gl.dest.left += H_DROP_STAIRS_SPEED;
        gl.dest.right += H_DROP_STAIRS_SPEED;
        gl.whole.right = gl.dest.right;

        gl.whole_shadow.left = gl.dest_shadow.left;
        gl.dest_shadow.left += H_DROP_STAIRS_SPEED;
        gl.dest_shadow.right += H_DROP_STAIRS_SPEED;
        gl.whole_shadow.right = gl.dest_shadow.right;

        gl.whole.top = gl.dest.top;
        gl.dest.top += V_DROP_STAIRS_SPEED;
        gl.dest.bottom += V_DROP_STAIRS_SPEED;
        gl.whole.bottom = gl.dest.bottom;

        let h_not_clipped = right_clip - gl.dest.left;
        if h_not_clipped < GLIDER_WIDE {
            if h_not_clipped <= 0 {
                gl.dest.right = gl.dest.left;
                gl.src.right = gl.src.left;
                gl.mask.right = gl.mask.left;
                gl.dest_shadow.right = gl.dest.right;
                self.taking_the_stairs = true;
                self.pl_leave_room(g, PLAYER_ESCAPED_DOWN_STAIRS, Exit::Room(BELOW));
            } else {
                gl.dest.right = gl.dest.left + h_not_clipped;
                gl.src.right = gl.src.left + h_not_clipped;
                gl.mask.right = gl.mask.left + h_not_clipped;
                gl.dest_shadow.right = gl.dest.right;
            }
        }
    }

    //--------------------------------------------------------------  FinishGliderDownStairs

    pub fn finish_glider_down_stairs(&mut self, g: usize) {
        self.pl_set_src(g, 0);

        let left_clip = self.left_clip;
        let gl = &mut self.gliders[g];
        gl.whole.left = gl.dest.left;
        gl.dest.left += H_DROP_STAIRS_SPEED;
        gl.dest.right += H_DROP_STAIRS_SPEED;
        gl.whole.right = gl.dest.right;

        gl.whole_shadow.left = gl.dest_shadow.left;
        gl.dest_shadow.left += H_DROP_STAIRS_SPEED;
        gl.dest_shadow.right += H_DROP_STAIRS_SPEED;
        gl.whole_shadow.right = gl.dest_shadow.right;

        gl.whole.top = gl.dest.top;
        gl.dest.top += V_DROP_STAIRS_SPEED;
        gl.dest.bottom += V_DROP_STAIRS_SPEED;
        gl.whole.bottom = gl.dest.bottom;

        let h_not_clipped = gl.dest.right - left_clip;
        if h_not_clipped < GLIDER_WIDE {
            gl.dest.left = gl.dest.right - h_not_clipped;
            gl.src.left = gl.src.right - h_not_clipped;
            gl.mask.left = gl.mask.right - h_not_clipped;
            gl.dest_shadow.left = gl.dest.left;
        } else {
            self.pl_normal_or_burning(g);
            let gl = &mut self.gliders[g];
            gl.h_vel = H_DROP_STAIRS_SPEED;
            gl.h_desired_vel = H_DROP_STAIRS_SPEED;
            gl.v_vel = V_DROP_STAIRS_SPEED;
            gl.v_desired_vel = V_DROP_STAIRS_SPEED;
            gl.entered_rect = gl.dest;
        }
    }

    //--------------------------------------------------------------  MoveGliderFaceLeft

    pub fn move_glider_face_left(&mut self, g: usize) {
        let frame = self.gliders[g].frame;
        self.pl_set_src(g, frame);

        self.move_glider(g);

        let gl = &mut self.gliders[g];
        gl.frame -= 1;
        if gl.frame < FIRST_ABOUT_FACE_FRAME {
            gl.mode = GLIDER_NORMAL;
            gl.facing = FACE_LEFT;
        }
    }

    //--------------------------------------------------------------  MoveGliderFaceRight

    pub fn move_glider_face_right(&mut self, g: usize) {
        let frame = self.gliders[g].frame;
        self.pl_set_src(g, frame);

        self.move_glider(g);

        let gl = &mut self.gliders[g];
        gl.frame += 1;
        if gl.frame > LAST_ABOUT_FACE_FRAME {
            gl.mode = GLIDER_NORMAL;
            gl.facing = FACE_RIGHT;
        }
    }

    //--------------------------------------------------------------  TransportGliderOut

    pub fn transport_glider_out(&mut self, g: usize) {
        self.gliders[g].frame -= 1;
        if self.gliders[g].frame < 0 {
            self.pl_erase_to_main(g);
            self.pl_leave_room(g, PLAYER_TRANSPORTED_OUT, Exit::Transport);
        } else {
            self.pl_set_fade_src(g);
        }
    }

    /// The glider drifts a pixel a frame towards its clip's left edge
    /// (lining up with a duct; the C repeats it in both duct moves).
    fn pl_drift_to_clip(&mut self, g: usize) {
        let gl = &mut self.gliders[g];
        if gl.dest.left < gl.clip.left {
            gl.whole.left = gl.dest.left;
            gl.dest.left += 1;
            gl.dest.right += 1;
            gl.whole.right = gl.dest.right;

            gl.whole_shadow.left = gl.dest_shadow.left;
            gl.dest_shadow.left += 1;
            gl.dest_shadow.right += 1;
            gl.whole_shadow.right = gl.dest_shadow.right;
        } else if gl.dest.left > gl.clip.left {
            gl.whole.right = gl.dest.right;
            gl.dest.left -= 1;
            gl.dest.right -= 1;
            gl.whole.left = gl.dest.left;

            gl.whole_shadow.right = gl.dest_shadow.right;
            gl.dest_shadow.left -= 1;
            gl.dest_shadow.right -= 1;
            gl.whole_shadow.left = gl.dest_shadow.left;
        }
    }

    //--------------------------------------------------------------  MoveGliderDownDuct

    pub fn move_glider_down_duct(&mut self, g: usize) {
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 2);
        } else {
            self.pl_set_src(g, 0);
        }

        self.pl_drift_to_clip(g);

        let gl = &mut self.gliders[g];
        gl.whole.top = gl.dest.top;
        gl.dest.top += V_DROP_DUCT_SPEED;
        gl.dest.bottom += V_DROP_DUCT_SPEED;
        gl.whole.bottom = gl.dest.bottom;

        let v_not_clipped = 315 - gl.dest.top;
        if v_not_clipped < GLIDER_HIGH {
            if v_not_clipped <= 0 {
                self.pl_erase_to_main(g);
                self.pl_leave_room(g, PLAYER_DUCKED_OUT, Exit::Duct);
            } else {
                gl.dest.bottom = gl.dest.top + v_not_clipped;
                gl.src.bottom = gl.src.top + v_not_clipped;
                gl.mask.bottom = gl.mask.top + v_not_clipped;
            }
        }
    }

    //--------------------------------------------------------------  MoveGliderUpDuct

    pub fn move_glider_up_duct(&mut self, g: usize) {
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 2);
        } else {
            self.pl_set_src(g, 0);
        }

        self.pl_drift_to_clip(g);

        let gl = &mut self.gliders[g];
        gl.whole.bottom = gl.dest.bottom;
        gl.dest.top += V_RISE_DUCT_SPEED;
        gl.dest.bottom += V_RISE_DUCT_SPEED;
        gl.whole.top = gl.dest.top;

        let v_not_clipped = gl.dest.bottom - (CEILING_TRANS_TOP + 1);
        if v_not_clipped < GLIDER_HIGH {
            if v_not_clipped <= 0 {
                self.pl_erase_to_main(g);
                self.pl_leave_room(g, PLAYER_DUCKED_OUT, Exit::Duct);
            } else {
                gl.dest.top = gl.dest.bottom - v_not_clipped;
                gl.src.top = gl.src.bottom - v_not_clipped;
                gl.mask.top = gl.mask.bottom - v_not_clipped;
            }
        }
    }

    //--------------------------------------------------------------  FinishGliderMailingLeft

    pub fn finish_glider_mailing_left(&mut self, g: usize) {
        if self.gliders[g].dest.left == self.gliders[g].dest.right {
            self.play_priority_sound(TRANS_IN_SOUND, TRANS_IN_PRIORITY);
        }

        let gl = &mut self.gliders[g];
        gl.whole.right = gl.dest.right;
        gl.dest.left += H_PUSH_MAIL_SPEED;
        gl.dest.right += H_PUSH_MAIL_SPEED;
        gl.whole.left = gl.dest.left;

        gl.whole_shadow.right = gl.dest_shadow.right;
        gl.dest_shadow.left += H_PUSH_MAIL_SPEED;
        gl.dest_shadow.right += H_PUSH_MAIL_SPEED;
        gl.whole_shadow.left = gl.dest_shadow.left;

        let h_not_clipped = gl.clip.right - gl.dest.left;
        if h_not_clipped < GLIDER_WIDE {
            gl.dest.right = gl.dest.left + h_not_clipped;
            gl.src.right = gl.src.left + h_not_clipped;
            gl.mask.right = gl.mask.left + h_not_clipped;
            gl.dest_shadow.right = gl.dest.right;
        } else {
            self.pl_normal_or_burning(g);
            self.gliders[g].entered_rect = self.gliders[g].dest;
        }
    }

    //--------------------------------------------------------------  FinishGliderMailingRight

    pub fn finish_glider_mailing_right(&mut self, g: usize) {
        if self.gliders[g].dest.left == self.gliders[g].dest.right {
            self.play_priority_sound(TRANS_IN_SOUND, TRANS_IN_PRIORITY);
        }

        let gl = &mut self.gliders[g];
        gl.whole.left = gl.dest.left;
        gl.dest.left += H_PUSH_MAIL_RT_SPEED;
        gl.dest.right += H_PUSH_MAIL_RT_SPEED;
        gl.whole.right = gl.dest.right;

        gl.whole_shadow.left = gl.dest_shadow.left;
        gl.dest_shadow.left += H_PUSH_MAIL_RT_SPEED;
        gl.dest_shadow.right += H_PUSH_MAIL_RT_SPEED;
        gl.whole_shadow.right = gl.dest_shadow.right;

        let h_not_clipped = gl.dest.right - gl.clip.left;
        if h_not_clipped < GLIDER_WIDE {
            gl.dest.left = gl.dest.right - h_not_clipped;
            gl.src.left = gl.src.right - h_not_clipped;
            gl.mask.left = gl.mask.right - h_not_clipped;
            gl.dest_shadow.left = gl.dest.left;
        } else {
            self.pl_normal_or_burning(g);
            self.gliders[g].entered_rect = self.gliders[g].dest;
        }
    }

    //--------------------------------------------------------------  FinishGliderDuctingIn

    pub fn finish_glider_ducting_in(&mut self, g: usize) {
        if self.gliders[g].dest.top == self.gliders[g].dest.bottom {
            self.play_priority_sound(TRANS_IN_SOUND, TRANS_IN_PRIORITY);
        }

        let gl = &mut self.gliders[g];
        gl.whole.top = gl.dest.top;
        gl.dest.top += V_DROP_DUCT_SPEED;
        gl.dest.bottom += V_DROP_DUCT_SPEED;
        gl.whole.bottom = gl.dest.bottom;

        let v_not_clipped = gl.dest.bottom - (CEILING_TRANS_TOP + 1);
        if v_not_clipped < GLIDER_HIGH {
            gl.dest.top = gl.dest.bottom - v_not_clipped;
            gl.src.top = gl.src.bottom - v_not_clipped;
            gl.mask.top = gl.mask.bottom - v_not_clipped;
        } else {
            self.pl_normal_or_burning(g);
            self.gliders[g].entered_rect = self.gliders[g].dest;
            self.flag_still_overs(g);
        }
    }

    //--------------------------------------------------------------  MoveGliderInMailLeft

    pub fn move_glider_in_mail_left(&mut self, g: usize) {
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 2);
        } else {
            self.pl_set_src(g, 0);
        }

        let gl = &mut self.gliders[g];
        if gl.dest.top < gl.clip.top {
            let mut from_ideal_top = gl.clip.top - gl.dest.top;
            if from_ideal_top > V_MAIL_DROP_SPEED {
                from_ideal_top = V_MAIL_DROP_SPEED;
            }

            gl.whole.top = gl.dest.top;
            gl.dest.top += from_ideal_top;
            gl.dest.bottom += from_ideal_top;
            gl.whole.bottom = gl.dest.bottom;
        }

        gl.whole.left = gl.dest.left;
        gl.dest.left += H_MAIL_PULL_SPEED;
        gl.dest.right += H_MAIL_PULL_SPEED;
        gl.whole.right = gl.dest.right;

        gl.whole_shadow.left = gl.dest_shadow.left;
        gl.dest_shadow.left += H_MAIL_PULL_SPEED;
        gl.dest_shadow.right += H_MAIL_PULL_SPEED;
        gl.whole_shadow.right = gl.dest_shadow.right;

        let h_not_clipped = gl.clip.right - gl.dest.left;
        if h_not_clipped < GLIDER_WIDE {
            if h_not_clipped <= 0 {
                self.pl_erase_to_main(g);
                self.pl_leave_room(g, PLAYER_MAILED_OUT, Exit::Mail);
            } else {
                gl.dest.right = gl.dest.left + h_not_clipped;
                gl.src.right = gl.src.left + h_not_clipped;
                gl.mask.right = gl.mask.left + h_not_clipped;
                gl.dest_shadow.right = gl.dest.right;
            }
        }
    }

    //--------------------------------------------------------------  MoveGliderInMailRight

    pub fn move_glider_in_mail_right(&mut self, g: usize) {
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, 2);
        } else {
            self.pl_set_src(g, 0);
        }

        let gl = &mut self.gliders[g];
        if gl.dest.top < gl.clip.top {
            let mut from_ideal_top = gl.clip.top - gl.dest.top;
            if from_ideal_top > V_MAIL_DROP_SPEED {
                from_ideal_top = V_MAIL_DROP_SPEED;
            }

            gl.whole.top = gl.dest.top;
            gl.dest.top += from_ideal_top;
            gl.dest.bottom += from_ideal_top;
            gl.whole.bottom = gl.dest.bottom;
        }

        gl.whole.right = gl.dest.right;
        gl.dest.left += H_MAIL_PULL_RT_SPEED;
        gl.dest.right += H_MAIL_PULL_RT_SPEED;
        gl.whole.left = gl.dest.left;

        gl.whole_shadow.right = gl.dest_shadow.right;
        gl.dest_shadow.left += H_MAIL_PULL_RT_SPEED;
        gl.dest_shadow.right += H_MAIL_PULL_RT_SPEED;
        gl.whole_shadow.left = gl.dest_shadow.left;

        let h_not_clipped = gl.dest.right - gl.clip.left;
        if h_not_clipped < GLIDER_WIDE {
            if h_not_clipped <= 0 {
                self.pl_erase_to_main(g);
                self.pl_leave_room(g, PLAYER_MAILED_OUT, Exit::Mail);
            } else {
                gl.dest.left = gl.dest.right - h_not_clipped;
                gl.src.left = gl.src.right - h_not_clipped;
                gl.mask.left = gl.mask.right - h_not_clipped;
                gl.dest_shadow.left = gl.dest.left;
            }
        }
    }

    /// LoadGraphic into the two players' glider maps (with or without foil).
    fn pl_load_glider_maps(&mut self, id1: i32, id2: i32) {
        let r = self.glid_src_rect;
        if let Some(img) = self.picture(id1) {
            self.glid_src_map.draw_picture(&img, r);
        }
        if let Some(img) = self.picture(id2) {
            self.glid2_src_map.draw_picture(&img, r);
        }
    }

    //--------------------------------------------------------------  DeckGliderInFoil

    pub fn deck_glider_in_foil(&mut self, g: usize) {
        self.show_foil = true;

        if self.two_player_game {
            self.pl_load_glider_maps(GLIDER_FOIL_PICT_ID, GLIDER_FOIL2_PICT_ID);
        }

        let frame = self.gliders[g].frame;
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, (frame + 2) + LEFT_FADE_OFFSET);
        } else {
            self.pl_set_src(g, frame + 2);
        }
    }

    //--------------------------------------------------------------  MoveGliderFoilGoing

    pub fn move_glider_foil_going(&mut self, g: usize) {
        self.gliders[g].frame += 1;
        if self.gliders[g].frame > 8 {
            self.flag_glider_normal(g);
        } else {
            let frame = self.gliders[g].frame;
            if frame < 5 {
                if self.gliders[g].facing == FACE_LEFT {
                    self.pl_set_src(g, (10 - frame) + LEFT_FADE_OFFSET);
                } else {
                    self.pl_set_src(g, 10 - frame);
                }
            } else {
                self.deck_glider_in_foil(g);
            }
        }
        self.move_glider(g);
    }

    //--------------------------------------------------------------  RemoveFoilFromGlider

    pub fn remove_foil_from_glider(&mut self, g: usize) {
        self.show_foil = false;

        if self.two_player_game {
            self.pl_load_glider_maps(GLIDER_PICT_ID, GLIDER2_PICT_ID);
        }

        let frame = self.gliders[g].frame;
        if self.gliders[g].facing == FACE_LEFT {
            self.pl_set_src(g, (frame + 2) + LEFT_FADE_OFFSET);
        } else {
            self.pl_set_src(g, frame + 2);
        }
    }

    //--------------------------------------------------------------  MoveGliderFoilLosing

    pub fn move_glider_foil_losing(&mut self, g: usize) {
        self.gliders[g].frame += 1;
        if self.gliders[g].frame > 8 {
            self.flag_glider_normal(g);
        } else {
            let frame = self.gliders[g].frame;
            if frame < 5 {
                if self.gliders[g].facing == FACE_LEFT {
                    self.pl_set_src(g, (10 - frame) + LEFT_FADE_OFFSET);
                } else {
                    self.pl_set_src(g, 10 - frame);
                }
            } else {
                self.remove_foil_from_glider(g);
            }
        }
        self.move_glider(g);
    }

    //--------------------------------------------------------------  MoveGliderShredding

    pub fn move_glider_shredding(&mut self, g: usize) {
        if self.gliders[g].frame > 0 {
            if self.gliders[g].facing == FACE_LEFT {
                self.pl_set_src(g, 2);
            } else {
                self.pl_set_src(g, 0);
            }

            let gl = &mut self.gliders[g];
            let v_not_clipped = gl.frame - gl.dest.top;
            if v_not_clipped < GLIDER_HIGH {
                gl.whole.top = gl.dest.top;
                gl.dest.top += DROP_SHRED_SLOW;
                gl.dest.bottom += DROP_SHRED_SLOW;
                gl.whole.bottom = gl.dest.bottom;
                self.shadow_visible = false;
                self.play_priority_sound(SHRED_SOUND, SHRED_PRIORITY);
            } else {
                gl.whole.top = gl.dest.top;
                gl.dest.top += DROP_SHRED_FAST;
                gl.dest.bottom += DROP_SHRED_FAST;
                gl.whole.bottom = gl.dest.bottom;
            }

            let gl = &mut self.gliders[g];
            let v_not_clipped = gl.frame - gl.dest.top;
            if v_not_clipped < GLIDER_HIGH {
                if v_not_clipped <= 0 {
                    let dest = gl.dest;
                    self.add_a_shredded_glider(&dest);
                    self.gliders[g].frame = SHREDDER_COUNTDOWN;
                } else {
                    gl.dest.bottom = gl.dest.top + v_not_clipped;
                    gl.src.bottom = gl.src.top + v_not_clipped;
                    gl.mask.bottom = gl.mask.top + v_not_clipped;
                }
            }
        } else {
            self.gliders[g].frame += 1;
            if self.gliders[g].frame >= 0 {
                self.off_a_mortal(g);
            }
        }
    }

    //--------------------------------------------------------------  HandleIdleGlider

    pub fn handle_idle_glider(&mut self, g: usize) {
        let gl = &mut self.gliders[g];
        gl.h_vel -= 1;
        if gl.h_vel <= 0 {
            gl.mode = gl.was_mode;
            gl.dont_draw = false;
        }
    }

    //--------------------------------------------------------------  HandleGlider

    pub fn handle_glider(&mut self, g: usize) {
        match self.gliders[g].mode {
            GLIDER_NORMAL => self.move_glider_normal(g),
            GLIDER_FADING_IN => self.fade_glider_in(g),
            GLIDER_FADING_OUT => self.fade_glider_out(g),
            GLIDER_GOING_UP => self.move_glider_up_stairs(g),
            GLIDER_COMING_UP => self.finish_glider_up_stairs(g),
            GLIDER_GOING_DOWN => self.move_glider_down_stairs(g),
            GLIDER_COMING_DOWN => self.finish_glider_down_stairs(g),
            GLIDER_FACE_LEFT => self.move_glider_face_left(g),
            GLIDER_FACE_RIGHT => self.move_glider_face_right(g),
            GLIDER_BURNING => self.move_glider_burning(g),
            GLIDER_TRANSPORTING => self.transport_glider_out(g),
            GLIDER_DUCTING_DOWN => self.move_glider_down_duct(g),
            GLIDER_DUCTING_UP => self.move_glider_up_duct(g),
            GLIDER_DUCTING_IN => self.finish_glider_ducting_in(g),
            GLIDER_MAIL_IN_LEFT => self.move_glider_in_mail_left(g),
            GLIDER_MAIL_OUT_LEFT => self.finish_glider_mailing_left(g), // <-- G _[]
            GLIDER_MAIL_IN_RIGHT => self.move_glider_in_mail_right(g),
            GLIDER_MAIL_OUT_RIGHT => self.finish_glider_mailing_right(g), // []_ G -->
            GLIDER_GOING_FOIL => self.move_glider_foil_going(g),
            GLIDER_LOSING_FOIL => self.move_glider_foil_losing(g),
            GLIDER_SHREDDING => self.move_glider_shredding(g),
            GLIDER_IN_LIMBO => {}
            GLIDER_IDLE => self.handle_idle_glider(g),
            GLIDER_TRANSPORTING_IN => self.transport_glider_in(g),
            _ => {}
        }

        let gl = &mut self.gliders[g];
        gl.ignore_left = false;
        gl.ignore_right = false;
        gl.ignore_ground = false;
    }

    //--------------------------------------------------------------  OffsetGlider

    pub fn offset_glider(&mut self, g: usize, where_: i32) {
        if self.two_player_game && self.one_player_left && self.gliders[g].which == self.player_dead
        {
            return;
        }

        let gl = &mut self.gliders[g];
        match where_ {
            TO_RIGHT => {
                gl.dest.left += ROOM_WIDE;
                gl.dest.right += ROOM_WIDE;
                gl.dest_shadow.left += ROOM_WIDE;
                gl.dest_shadow.right += ROOM_WIDE;
                gl.whole = gl.dest;
                gl.whole_shadow = gl.dest_shadow;
            }
            TO_LEFT => {
                gl.dest.left -= ROOM_WIDE;
                gl.dest.right -= ROOM_WIDE;
                gl.dest_shadow.left -= ROOM_WIDE;
                gl.dest_shadow.right -= ROOM_WIDE;
                gl.whole = gl.dest;
                gl.whole_shadow = gl.dest_shadow;
            }
            ABOVE => {
                gl.dest.top -= TILE_HIGH;
                gl.dest.bottom -= TILE_HIGH;
                gl.whole = gl.dest;
            }
            BELOW => {
                gl.dest.top += TILE_HIGH;
                gl.dest.bottom += TILE_HIGH;
                gl.whole = gl.dest;
            }
            _ => {}
        }
    }

    //--------------------------------------------------------------  OffAMortal

    pub fn off_a_mortal(&mut self, g: usize) {
        if self.game_over {
            return;
        }

        if self.num_shredded > 0 {
            self.remove_shreds();
        }

        self.mortals -= 1;
        if self.mortals < 0 {
            self.hide_glider(g);
            if self.two_player_game {
                if self.mortals < -1 {
                    // both players are now dead
                    self.flag_game_over();
                    self.gliders[g].dont_draw = true;
                } else {
                    self.flag_glider_in_limbo(g, false);
                    self.gliders[g].dont_draw = true;
                    self.one_player_left = true;
                    self.player_dead = self.gliders[g].which;
                }
            } else {
                self.flag_game_over();
                self.gliders[g].dont_draw = true;
            }
        } else {
            self.quick_gliders_refresh();
            self.hide_glider(g);
        }

        if self.mortals >= 0 {
            if self.gliders[g].mode == GLIDER_GOING_FOIL {
                self.deck_glider_in_foil(g);
            }

            self.flag_glider_normal(g);
            if self.player_suicide {
                self.follow_the_leader();
            } else {
                self.start_glider_fading_in(g);
                let gl = &mut self.gliders[g];
                gl.dest = gl.entered_rect;
                gl.whole = gl.dest;
                gl.dest_shadow.left = gl.dest.left;
                gl.dest_shadow.right = gl.dest.right;
                gl.whole_shadow = gl.dest_shadow;
            }
        } else if self.mortals == -1 && self.one_player_left && !self.game_over {
            // the survivor: the glider of the player not dead
            let other = if self.player_dead == PLAYER1 { 1 } else { 0 };
            match self.other_player_escaped {
                PLAYER_ESCAPED_UP | PLAYER_ESCAPING_UP_STAIRS | PLAYER_ESCAPED_UP_STAIRS => {
                    self.move_room_to_room(other, ABOVE);
                }
                PLAYER_ESCAPED_DOWN | PLAYER_ESCAPING_DOWN_STAIRS | PLAYER_ESCAPED_DOWN_STAIRS => {
                    self.move_room_to_room(other, BELOW);
                }
                PLAYER_ESCAPED_LEFT => self.move_room_to_room(other, TO_LEFT),
                PLAYER_ESCAPED_RIGHT => self.move_room_to_room(other, TO_RIGHT),
                PLAYER_TRANSPORTED_OUT => self.transport_room_to_room(other),
                PLAYER_MAILED_OUT => self.move_mail_to_mail(other),
                PLAYER_DUCKED_OUT => self.move_duct_to_duct(other),
                _ => {}
            }

            self.other_player_escaped = PLAYER_IS_DEAD_FOREVER;
        }
    }
}

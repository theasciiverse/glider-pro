//! Modes.c: putting a glider into a mode (fading, transporting, ducting,
//! mailing, burning, idle, ...).

use super::*;

pub const FRAMES_TO_BURN: i32 = 60;
pub const V_GLIDER_APPEARS_COMING_UP: i32 = 100;
pub const V_GLIDER_APPEARS_COMING_DOWN: i32 = 100;

impl Game {
    /// The repeated tail of the fade functions: the glider's src and mask
    /// from the fade-in sequence at its frame, by facing.
    fn set_glider_fade_src(&mut self, g: usize) {
        let frame = self.gliders[g].frame;
        let r = if self.gliders[g].facing == FACE_LEFT {
            self.glider_src[(self.fade_in_sequence[frame as usize] + LEFT_FADE_OFFSET) as usize]
        } else {
            self.glider_src[self.fade_in_sequence[frame as usize] as usize]
        };
        self.gliders[g].src = r;
        self.gliders[g].mask = r;
    }

    //--------------------------------------------------------------  StartGliderFadingIn

    pub fn start_glider_fading_in(&mut self, g: usize) {
        if self.foil_total <= 0 {
            self.show_foil = false;
        }

        self.gliders[g].mode = GLIDER_FADING_IN;
        self.gliders[g].whole = self.gliders[g].dest;
        self.gliders[g].frame = 0;
        self.gliders[g].dont_draw = false;
        self.set_glider_fade_src(g);
    }

    //--------------------------------------------------------------  StartGliderTransportingIn

    pub fn start_glider_transporting_in(&mut self, g: usize) {
        if self.foil_total <= 0 {
            self.show_foil = false;
        }

        self.gliders[g].mode = GLIDER_TRANSPORTING_IN;
        self.gliders[g].whole = self.gliders[g].dest;
        self.gliders[g].frame = 0;
        self.gliders[g].dont_draw = false;
        self.set_glider_fade_src(g);
    }

    //--------------------------------------------------------------  StartGliderFadingOut

    pub fn start_glider_fading_out(&mut self, g: usize) {
        if self.gliders[g].mode == GLIDER_FADING_OUT {
            return;
        }

        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        if rect_tall(&self.gliders[g].dest) > GLIDER_HIGH {
            let mut temp_bounds = self.gliders[g].dest;
            q_offset_rect(&mut temp_bounds, self.play_origin_h, self.play_origin_v);
            self.add_rect_to_work_rects(&temp_bounds);
            if self.has_mirror {
                temp_bounds = self.gliders[g].dest;
                q_offset_rect(
                    &mut temp_bounds,
                    self.play_origin_h - 20,
                    self.play_origin_v - 16,
                );
                self.add_rect_to_work_rects(&temp_bounds);
            }
            self.gliders[g].dest.right = self.gliders[g].dest.left + GLIDER_WIDE;
            self.gliders[g].dest.top = self.gliders[g].dest.bottom - GLIDER_HIGH;
        }
        self.gliders[g].mode = GLIDER_FADING_OUT;
        self.gliders[g].whole = self.gliders[g].dest;
        self.gliders[g].frame = LAST_FADE_SEQUENCE - 1;
        self.set_glider_fade_src(g);
    }

    //--------------------------------------------------------------  StartGliderGoingUpStairs

    pub fn start_glider_going_up_stairs(&mut self, g: usize) {
        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        if self.gliders[g].mode == GLIDER_BURNING {
            self.gliders[g].frame = WAS_BURNING;
        } else {
            self.gliders[g].frame = 0;
        }

        self.gliders[g].mode = GLIDER_GOING_UP;
    }

    //--------------------------------------------------------------  StartGliderGoingDownStairs

    pub fn start_glider_going_down_stairs(&mut self, g: usize) {
        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        if self.gliders[g].mode == GLIDER_BURNING {
            self.gliders[g].frame = WAS_BURNING;
        } else {
            self.gliders[g].frame = 0;
        }

        self.gliders[g].mode = GLIDER_GOING_DOWN;
        self.right_clip = self.get_up_stairs_right_edge();
    }

    /// The common part of the transit starts: where the hot spot `who`
    /// links to (transRoom, linkedToWhat, transRect).
    fn find_trans_link(&mut self, who: usize) {
        let who_linked = self.hot_spots[who].who;
        self.trans_room = self.master_objects[who_linked as usize].room_link;
        let obj_linked = self.master_objects[who_linked as usize].object_link;
        self.linked_to_what = self.what_are_we_linked_to(self.trans_room, obj_linked);

        let mut obj = self.house.rooms[self.trans_room as usize].objects[obj_linked as usize];
        let mut trans_rect = self.trans_rect;
        self.get_object_rect(&mut obj, &mut trans_rect);
        self.trans_rect = trans_rect;
    }

    //--------------------------------------------------------------  StartGliderMailingIn

    pub fn start_glider_mailing_in(&mut self, g: usize, bounds: &Rect, who: usize) {
        self.play_priority_sound(TRANS_OUT_SOUND, TRANS_OUT_PRIORITY);

        self.find_trans_link(who);

        self.gliders[g].frame = 0;
        self.gliders[g].clip = *bounds;
        let top_sought = bounds.bottom - rect_tall(&self.gliders[g].dest);
        self.gliders[g].clip.top = top_sought;
    }

    //--------------------------------------------------------------  StartGliderMailingOut

    pub fn start_glider_mailing_out(&mut self, g: usize) {
        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        if self.linked_to_what == LINKED_TO_LEFT_MAILBOX {
            self.gliders[g].facing = FACE_LEFT;
            self.gliders[g].mode = GLIDER_MAIL_OUT_LEFT;
            self.gliders[g].src = self.glider_src[2];
            self.gliders[g].mask = self.glider_src[2];
        } else {
            self.gliders[g].facing = FACE_RIGHT;
            self.gliders[g].mode = GLIDER_MAIL_OUT_RIGHT;
            self.gliders[g].src = self.glider_src[0];
            self.gliders[g].mask = self.glider_src[0];
        }

        let gl = &mut self.gliders[g];
        gl.h_vel = 0;
        gl.v_vel = 0;
        gl.h_desired_vel = 0;
        gl.v_desired_vel = 0;
        gl.tipped = false;
        gl.dont_draw = false;
    }

    //--------------------------------------------------------------  StartGliderDuctingDown

    pub fn start_glider_ducting_down(&mut self, g: usize, bounds: &Rect, who: usize) {
        self.play_priority_sound(TRANS_OUT_SOUND, TRANS_OUT_PRIORITY);

        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        self.find_trans_link(who);

        self.gliders[g].frame = 0;
        self.gliders[g].clip = *bounds;
        let left_sought = bounds.left + ((rect_wide(bounds) - GLIDER_WIDE) / 2);
        self.gliders[g].clip.left = left_sought;

        self.gliders[g].mode = GLIDER_DUCTING_DOWN;
    }

    //--------------------------------------------------------------  StartGliderDuctingUp

    pub fn start_glider_ducting_up(&mut self, g: usize, bounds: &Rect, who: usize) {
        self.play_priority_sound(TRANS_OUT_SOUND, TRANS_OUT_PRIORITY);

        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        self.find_trans_link(who);

        self.gliders[g].frame = 0;
        self.gliders[g].clip = *bounds;
        let left_sought = bounds.left + ((rect_wide(bounds) - GLIDER_WIDE) / 2);
        self.gliders[g].clip.left = left_sought;

        self.gliders[g].mode = GLIDER_DUCTING_UP;
    }

    //--------------------------------------------------------------  StartGliderDuctingIn

    pub fn start_glider_ducting_in(&mut self, g: usize) {
        self.gliders[g].mode = GLIDER_DUCTING_IN;
        self.gliders[g].whole = self.gliders[g].dest;
        self.gliders[g].dont_draw = false;
    }

    //--------------------------------------------------------------  StartGliderTransporting

    pub fn start_glider_transporting(&mut self, g: usize, who: usize) {
        self.play_priority_sound(TRANS_OUT_SOUND, TRANS_OUT_PRIORITY);

        if self.gliders[g].mode == GLIDER_GOING_FOIL {
            self.deck_glider_in_foil(g);
        } else if self.gliders[g].mode == GLIDER_LOSING_FOIL {
            self.remove_foil_from_glider(g);
        }

        self.find_trans_link(who);

        let gl = &mut self.gliders[g];
        gl.dest.right = gl.dest.left + GLIDER_WIDE;
        gl.dest.bottom = gl.dest.top + GLIDER_HIGH;
        gl.dest_shadow.right = gl.dest_shadow.left + GLIDER_WIDE;
        gl.dest_shadow.bottom = gl.dest_shadow.top + SHADOW_HIGH;
        gl.mode = GLIDER_TRANSPORTING;
        gl.whole = gl.dest;
        gl.frame = LAST_FADE_SEQUENCE - 1;
        self.set_glider_fade_src(g);
    }

    //--------------------------------------------------------------  FlagGliderNormal

    pub fn flag_glider_normal(&mut self, g: usize) {
        let src0 = self.glider_src[0];
        let src2 = self.glider_src[2];
        let gl = &mut self.gliders[g];
        gl.dest.right = gl.dest.left + GLIDER_WIDE;
        gl.dest.bottom = gl.dest.top + GLIDER_HIGH;
        gl.dest_shadow.right = gl.dest_shadow.left + GLIDER_WIDE;
        gl.dest_shadow.bottom = gl.dest_shadow.top + SHADOW_HIGH;
        gl.mode = GLIDER_NORMAL;
        if gl.facing == FACE_LEFT {
            gl.src = src2;
            gl.mask = src2;
        } else {
            gl.src = src0;
            gl.mask = src0;
        }
        gl.h_vel = 0;
        gl.v_vel = 0;
        gl.h_desired_vel = 0;
        gl.v_desired_vel = 0;
        gl.tipped = false;
        gl.ignore_left = false;
        gl.ignore_right = false;
        gl.ignore_ground = false;
        gl.dont_draw = false;
        gl.frame = 0;
        self.shadow_visible = self.is_shadow_visible();
    }

    //--------------------------------------------------------------  FlagGliderShredding

    pub fn flag_glider_shredding(&mut self, g: usize, bounds: &Rect) {
        self.play_priority_sound(CAUGHT_FIRE_SOUND, CAUGHT_FIRE_PRIORITY);
        let src0 = self.glider_src[0];
        let src2 = self.glider_src[2];
        let gl = &mut self.gliders[g];
        gl.dest.left = bounds.left + 36;
        gl.dest.right = gl.dest.left + GLIDER_WIDE;
        gl.dest.bottom = gl.dest.top + GLIDER_HIGH;
        if gl.dest.left > gl.whole.left {
            gl.whole.right = gl.dest.right;
            gl.whole_shadow.right = gl.dest.right;
        } else {
            gl.whole.left = gl.dest.left;
            gl.whole_shadow.left = gl.dest.left;
        }
        gl.dest_shadow.left = gl.dest.left;
        gl.dest_shadow.right = gl.dest_shadow.left + GLIDER_WIDE;
        gl.dest_shadow.bottom = gl.dest_shadow.top + SHADOW_HIGH;
        gl.mode = GLIDER_SHREDDING;
        if gl.facing == FACE_LEFT {
            gl.src = src2;
            gl.mask = src2;
        } else {
            gl.src = src0;
            gl.mask = src0;
        }
        gl.h_vel = 0;
        gl.v_vel = 0;
        gl.h_desired_vel = 0;
        gl.v_desired_vel = 0;
        gl.frame = bounds.bottom - 3;
        gl.tipped = false;
    }

    //--------------------------------------------------------------  FlagGliderBurning

    pub fn flag_glider_burning(&mut self, g: usize) {
        self.play_priority_sound(CAUGHT_FIRE_SOUND, CAUGHT_FIRE_PRIORITY);

        let src21 = self.glider_src[21];
        let src25 = self.glider_src[25];
        let gl = &mut self.gliders[g];
        gl.dest.right = gl.dest.left + GLIDER_WIDE;
        gl.dest.top = gl.dest.bottom - GLIDER_BURNING_HIGH;
        gl.dest_shadow.right = gl.dest_shadow.left + GLIDER_WIDE;
        gl.dest_shadow.bottom = gl.dest_shadow.top + SHADOW_HIGH;
        gl.mode = GLIDER_BURNING;
        if gl.facing == FACE_LEFT {
            gl.src = src25;
            gl.mask = src25;
        } else {
            gl.src = src21;
            gl.mask = src21;
        }
        gl.h_vel = 0;
        gl.v_vel = 0;
        gl.h_desired_vel = 0;
        gl.v_desired_vel = 0;
        gl.frame = 0;
        gl.was_mode = FRAMES_TO_BURN; // used to count down burning
        gl.tipped = false;
    }

    //--------------------------------------------------------------  FlagGliderFaceLeft

    pub fn flag_glider_face_left(&mut self, g: usize) {
        self.gliders[g].mode = GLIDER_FACE_LEFT;
        self.gliders[g].frame = LAST_ABOUT_FACE_FRAME;
        self.gliders[g].src = self.glider_src[LAST_ABOUT_FACE_FRAME as usize];
        self.gliders[g].mask = self.glider_src[LAST_ABOUT_FACE_FRAME as usize];
    }

    //--------------------------------------------------------------  FlagGliderFaceRight

    pub fn flag_glider_face_right(&mut self, g: usize) {
        self.gliders[g].mode = GLIDER_FACE_RIGHT;
        self.gliders[g].frame = FIRST_ABOUT_FACE_FRAME;
        self.gliders[g].src = self.glider_src[FIRST_ABOUT_FACE_FRAME as usize];
        self.gliders[g].mask = self.glider_src[FIRST_ABOUT_FACE_FRAME as usize];
    }

    //--------------------------------------------------------------  FlagGliderInLimbo

    pub fn flag_glider_in_limbo(&mut self, g: usize, say_it: bool) {
        self.gliders[g].was_mode = self.gliders[g].mode;
        self.gliders[g].mode = GLIDER_IN_LIMBO;
        if say_it && (self.said_follow < 3) {
            self.play_priority_sound(FOLLOW_SOUND, FOLLOW_PRIORITY);
            self.said_follow += 1;
        }
        self.first_player = self.gliders[g].which;
    }

    //--------------------------------------------------------------  UndoGliderLimbo

    pub fn undo_glider_limbo(&mut self, g: usize) {
        if self.two_player_game
            && self.one_player_left
            && (self.gliders[g].which == self.player_dead)
        {
            return;
        }

        if self.gliders[g].mode == GLIDER_IN_LIMBO {
            self.gliders[g].mode = self.gliders[g].was_mode;
        }
        self.gliders[g].dont_draw = false;
    }

    //--------------------------------------------------------------  ToggleGliderFacing

    pub fn toggle_glider_facing(&mut self, g: usize) {
        if self.gliders[g].mode != GLIDER_NORMAL {
            return;
        }

        if self.gliders[g].facing == FACE_LEFT {
            self.flag_glider_face_right(g);
        } else {
            self.flag_glider_face_left(g);
        }
    }

    //--------------------------------------------------------------  InsureGliderFacingRight

    pub fn insure_glider_facing_right(&mut self, g: usize) {
        if self.two_player_game
            && self.one_player_left
            && (self.gliders[g].which == self.player_dead)
        {
            return;
        }

        if (self.gliders[g].facing == FACE_LEFT) && (self.gliders[g].mode != GLIDER_BURNING) {
            self.flag_glider_face_right(g);
        }
    }

    //--------------------------------------------------------------  InsureGliderFacingLeft

    pub fn insure_glider_facing_left(&mut self, g: usize) {
        if self.two_player_game
            && self.one_player_left
            && (self.gliders[g].which == self.player_dead)
        {
            return;
        }

        if (self.gliders[g].facing == FACE_RIGHT) && (self.gliders[g].mode != GLIDER_BURNING) {
            self.flag_glider_face_left(g);
        }
    }

    //--------------------------------------------------------------  ReadyGliderForTripUpStairs

    pub fn ready_glider_for_trip_up_stairs(&mut self, g: usize) {
        if self.two_player_game
            && (self.gliders[g].which == self.player_dead)
            && self.one_player_left
        {
            return;
        }

        let src2 = self.glider_src[2];
        {
            let gl = &mut self.gliders[g];
            gl.facing = FACE_LEFT;
            gl.mode = GLIDER_COMING_UP;
            gl.src = src2;
            gl.mask = src2;
            gl.h_vel = 0;
            gl.v_vel = 0;
            gl.h_desired_vel = 0;
            gl.v_desired_vel = 0;
            gl.tipped = false;
        }

        self.right_clip = self.get_up_stairs_right_edge();
        let right_clip = self.right_clip;
        {
            let gl = &mut self.gliders[g];
            gl.dest = gl.src;
            zero_rect_corner(&mut gl.dest);
            q_offset_rect(&mut gl.dest, right_clip, V_GLIDER_APPEARS_COMING_UP);
            gl.whole = gl.dest;
            gl.dest_shadow.left = gl.dest.left;
            gl.dest_shadow.right = gl.dest.right;
            gl.whole_shadow = gl.dest_shadow;
        }

        self.finish_glider_up_stairs(g);
    }

    //--------------------------------------------------------------  ReadyGliderForTripDownStairs

    pub fn ready_glider_for_trip_down_stairs(&mut self, g: usize) {
        if self.two_player_game
            && (self.gliders[g].which == self.player_dead)
            && self.one_player_left
        {
            return;
        }

        let src0 = self.glider_src[0];
        {
            let gl = &mut self.gliders[g];
            gl.facing = FACE_RIGHT;
            gl.mode = GLIDER_COMING_DOWN;
            gl.src = src0;
            gl.mask = src0;
            gl.h_vel = 0;
            gl.v_vel = 0;
            gl.h_desired_vel = 0;
            gl.v_desired_vel = 0;
            gl.tipped = false;
        }

        self.left_clip = self.get_down_stairs_left_edge();
        let left_clip = self.left_clip;
        {
            let gl = &mut self.gliders[g];
            gl.dest = gl.src;
            zero_rect_corner(&mut gl.dest);
            q_offset_rect(
                &mut gl.dest,
                left_clip - GLIDER_WIDE,
                V_GLIDER_APPEARS_COMING_DOWN,
            );
            gl.whole = gl.dest;
            gl.dest_shadow.left = gl.dest.left;
            gl.dest_shadow.right = gl.dest.right;
            gl.whole_shadow = gl.dest_shadow;
        }

        self.finish_glider_down_stairs(g);
    }

    /// The tail of the foil functions: src and mask by frame (10 - frame).
    fn set_glider_foil_src(&mut self, g: usize) {
        let frame = self.gliders[g].frame;
        let r = if self.gliders[g].facing == FACE_LEFT {
            self.glider_src[((10 - frame) + LEFT_FADE_OFFSET) as usize]
        } else {
            self.glider_src[(10 - frame) as usize]
        };
        self.gliders[g].src = r;
        self.gliders[g].mask = r;
    }

    //--------------------------------------------------------------  StartGliderFoilGoing

    pub fn start_glider_foil_going(&mut self, g: usize) {
        if (self.gliders[g].mode == GLIDER_GOING_FOIL) || (self.gliders[g].mode == GLIDER_IN_LIMBO)
        {
            return;
        }

        self.quick_foil_refresh(false);

        self.gliders[g].mode = GLIDER_GOING_FOIL;
        self.gliders[g].whole = self.gliders[g].dest;
        self.gliders[g].frame = 0;
        self.set_glider_foil_src(g);
    }

    //--------------------------------------------------------------  StartGliderFoilLosing

    pub fn start_glider_foil_losing(&mut self, g: usize) {
        if (self.gliders[g].mode == GLIDER_LOSING_FOIL) || (self.gliders[g].mode == GLIDER_IN_LIMBO)
        {
            return;
        }

        self.quick_foil_refresh(false);
        self.play_priority_sound(FIZZLE_SOUND, FIZZLE_PRIORITY);

        self.gliders[g].mode = GLIDER_LOSING_FOIL;
        self.gliders[g].whole = self.gliders[g].dest;
        self.gliders[g].frame = 0;
        self.set_glider_foil_src(g);
    }

    //--------------------------------------------------------------  TagGliderIdle

    pub fn tag_glider_idle(&mut self, g: usize) {
        if self.two_player_game
            && self.one_player_left
            && (self.gliders[g].which == self.player_dead)
        {
            return;
        }

        self.gliders[g].was_mode = self.gliders[g].mode;
        self.gliders[g].mode = GLIDER_IDLE;
        self.gliders[g].h_vel = 30; // used for 30 frame delay
    }
}

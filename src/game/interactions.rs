//! Interactions.c: the glider meeting walls, floors, ceilings and hot spots.

use super::*;

pub const FLOOR_VENT_LIFT: i32 = -6;
pub const CEILING_VENT_DROP: i32 = 8;
pub const FAN_STRENGTH: i32 = 12;
pub const BATTERY_SUPPLY: i32 = 50; // about 2 rooms worth of thrust
pub const HELIUM_SUPPLY: i32 = 150;
pub const BANDS_SUPPLY: i32 = 8;
pub const FOIL_SUPPLY: i32 = 8;
pub const KILL_WEBBED_GLIDER: i32 = 150;

impl Game {
    //--------------------------------------------------------------  GliderHitSides

    pub fn glider_hit_top(&mut self, g: usize, the_rect: &Rect) -> bool {
        let mut glide_bounds = Rect::default();
        let offset;
        let hit_top;

        let gl = self.gliders[g];
        glide_bounds.left = gl.dest.left + 5;
        glide_bounds.top = gl.dest.top + 5;
        glide_bounds.right = gl.dest.right - 5;
        glide_bounds.bottom = gl.dest.bottom - 5;

        glide_bounds.left -= gl.was_h_vel;
        glide_bounds.right -= gl.was_h_vel;

        if the_rect.bottom < glide_bounds.top {
            hit_top = false;
        } else if the_rect.top > glide_bounds.bottom {
            hit_top = false;
        } else if the_rect.right < glide_bounds.left {
            hit_top = false;
        } else if the_rect.left > glide_bounds.right {
            hit_top = false;
        } else {
            hit_top = true;
        }

        if !hit_top {
            self.play_priority_sound(FOIL_HIT_SOUND, FOIL_HIT_PRIORITY);
            self.foil_total -= 1;
            if self.foil_total <= 0 {
                self.start_glider_foil_losing(g);
            }

            let gl = &mut self.gliders[g];
            glide_bounds.left += gl.was_h_vel;
            glide_bounds.right += gl.was_h_vel;
            if gl.h_vel > 0 {
                offset = 2 + glide_bounds.right - the_rect.left;
            } else {
                offset = 2 + glide_bounds.left - the_rect.right;
            }

            gl.h_vel = -gl.h_vel - offset;
        }

        hit_top
    }

    //--------------------------------------------------------------  SectGlider

    pub fn sect_glider(&self, g: usize, the_rect: &Rect, scrutinize: bool) -> bool {
        let gl = &self.gliders[g];
        let mut glide_bounds = gl.dest;
        if gl.mode == GLIDER_BURNING {
            glide_bounds.top += 6;
        }

        if scrutinize {
            glide_bounds.left += 5;
            glide_bounds.top += 5;
            glide_bounds.right -= 5;
            glide_bounds.bottom -= 5;
        }

        if the_rect.bottom < glide_bounds.top {
            false
        } else if the_rect.top > glide_bounds.bottom {
            false
        } else if the_rect.right < glide_bounds.left {
            false
        } else if the_rect.left > glide_bounds.right {
            false
        } else {
            true
        }
    }

    //--------------------------------------------------------------  GliderInRect

    pub fn glider_in_rect(&self, g: usize, the_rect: &Rect) -> bool {
        let glide_bounds = self.gliders[g].dest;

        if glide_bounds.top < the_rect.top {
            false
        } else if glide_bounds.bottom > the_rect.bottom {
            false
        } else if glide_bounds.left < the_rect.left {
            false
        } else if glide_bounds.right > the_rect.right {
            false
        } else {
            true
        }
    }

    //--------------------------------------------------------------  BounceGlider

    pub fn bounce_glider(&mut self, g: usize, the_rect: &Rect) {
        let glide_bounds = self.gliders[g].dest;
        if (the_rect.right - glide_bounds.left) < (glide_bounds.right - the_rect.left) {
            self.gliders[g].h_vel = the_rect.right - glide_bounds.left;
        } else {
            self.gliders[g].h_vel = the_rect.left - glide_bounds.right;
        }
        if self.foil_total > 0 {
            self.play_priority_sound(FOIL_HIT_SOUND, FOIL_HIT_PRIORITY);
        } else {
            self.play_priority_sound(HIT_WALL_SOUND, HIT_WALL_PRIORITY);
        }
    }

    /// The fade-out of a glider that hit the floor (repeated in the C).
    fn ia_floor_fade_out(&mut self, g: usize) {
        self.gliders[g].v_vel = FLOOR_LIMIT - self.gliders[g].dest.bottom;
        self.start_glider_fading_out(g);
        self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
    }

    /// A burning glider leaving the room, or webbed too long: it fades out
    /// (repeated in the C).
    fn ia_burnt_fade_out(&mut self, g: usize) {
        self.gliders[g].was_mode = 0;
        self.start_glider_fading_out(g);
        self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
    }

    /// Two players, a glider going up out of the room (CheckEscapeUpTwo's
    /// repeated block).
    fn ia_escape_up(&mut self, g: usize) {
        if self.other_player_escaped == NO_ONE_ESCAPED {
            self.other_player_escaped = PLAYER_ESCAPED_UP;
            self.refresh_scoreboard(ESCAPED_TITLE_MODE);
            self.flag_glider_in_limbo(g, true);
        } else if self.other_player_escaped == PLAYER_ESCAPED_UP {
            self.other_player_escaped = NO_ONE_ESCAPED;
            self.move_room_to_room(g, ABOVE);
        } else {
            self.play_priority_sound(DONT_EXIT_SOUND, DONT_EXIT_PRIORITY);
            let gl = &mut self.gliders[g];
            let offset = NO_CEILING_LIMIT - gl.dest.top;
            gl.v_vel = -gl.v_vel + offset;
        }
    }

    /// Two players, a glider going down out of the room.
    fn ia_escape_down(&mut self, g: usize) {
        if self.other_player_escaped == NO_ONE_ESCAPED {
            self.other_player_escaped = PLAYER_ESCAPED_DOWN;
            self.refresh_scoreboard(ESCAPED_TITLE_MODE);
            self.flag_glider_in_limbo(g, true);
        } else if self.other_player_escaped == PLAYER_ESCAPED_DOWN {
            self.other_player_escaped = NO_ONE_ESCAPED;
            self.move_room_to_room(g, BELOW);
        } else {
            self.play_priority_sound(DONT_EXIT_SOUND, DONT_EXIT_PRIORITY);
            let gl = &mut self.gliders[g];
            let offset = NO_FLOOR_LIMIT - gl.dest.bottom;
            gl.v_vel = -gl.v_vel + offset;
        }
    }

    /// Two players, a glider going out to the left.
    fn ia_escape_left(&mut self, g: usize) {
        if self.other_player_escaped == NO_ONE_ESCAPED {
            self.other_player_escaped = PLAYER_ESCAPED_LEFT;
            self.refresh_scoreboard(ESCAPED_TITLE_MODE);
            self.flag_glider_in_limbo(g, true);
        } else if self.other_player_escaped == PLAYER_ESCAPED_LEFT {
            self.other_player_escaped = NO_ONE_ESCAPED;
            self.move_room_to_room(g, TO_LEFT);
        } else {
            self.play_priority_sound(DONT_EXIT_SOUND, DONT_EXIT_PRIORITY);
            let gl = &mut self.gliders[g];
            let offset = NO_LEFT_WALL_LIMIT - gl.dest.left;
            gl.h_vel = -gl.h_vel + offset;
        }
    }

    /// Two players, a glider going out to the right.
    fn ia_escape_right(&mut self, g: usize) {
        if self.other_player_escaped == NO_ONE_ESCAPED {
            self.other_player_escaped = PLAYER_ESCAPED_RIGHT;
            self.refresh_scoreboard(ESCAPED_TITLE_MODE);
            self.flag_glider_in_limbo(g, true);
        } else if self.other_player_escaped == PLAYER_ESCAPED_RIGHT {
            self.other_player_escaped = NO_ONE_ESCAPED;
            self.move_room_to_room(g, TO_RIGHT);
        } else {
            self.play_priority_sound(DONT_EXIT_SOUND, DONT_EXIT_PRIORITY);
            let gl = &mut self.gliders[g];
            let offset = NO_RIGHT_WALL_LIMIT - gl.dest.right;
            gl.h_vel = -gl.h_vel + offset;
        }
    }

    /// The tiles under the glider's left and right edges (`>> 6`: / 64).
    fn ia_glider_tiles(&self, g: usize) -> (i32, i32) {
        let gl = &self.gliders[g];
        (gl.dest.left >> 6, gl.dest.right >> 6)
    }

    //--------------------------------------------------------------  CheckEscapeUpTwo

    pub fn check_escape_up_two(&mut self, g: usize) {
        if self.top_open {
            if self.gliders[g].dest.top < NO_CEILING_LIMIT {
                self.ia_escape_up(g);
            }
        } else if self.this_background == DIRT {
            let (left_tile, right_tile) = self.ia_glider_tiles(g);

            if (0..8).contains(&left_tile) && (0..8).contains(&right_tile) {
                let lt = self.this_tiles[left_tile as usize];
                let rt = self.this_tiles[right_tile as usize];
                if (lt == 5 || lt == 6) && (rt == 5 || rt == 6) {
                    if self.gliders[g].dest.top < NO_CEILING_LIMIT {
                        self.ia_escape_up(g);
                    }
                } else {
                    self.gliders[g].v_vel = CEILING_LIMIT - self.gliders[g].dest.top;
                }
            } else {
                self.gliders[g].v_vel = CEILING_LIMIT - self.gliders[g].dest.top;
            }
        } else {
            self.gliders[g].v_vel = CEILING_LIMIT - self.gliders[g].dest.top;
        }
    }

    //--------------------------------------------------------------  CheckEscapeUp

    pub fn check_escape_up(&mut self, g: usize) {
        if self.top_open {
            if self.gliders[g].dest.top < NO_CEILING_LIMIT {
                self.move_room_to_room(g, ABOVE);
            }
        } else if self.this_background == DIRT {
            let (left_tile, right_tile) = self.ia_glider_tiles(g);

            if (0..8).contains(&left_tile) && (0..8).contains(&right_tile) {
                let lt = self.this_tiles[left_tile as usize];
                let rt = self.this_tiles[right_tile as usize];
                if (lt == 5 || lt == 6) && (rt == 5 || rt == 6) {
                    if self.gliders[g].dest.top < NO_CEILING_LIMIT {
                        self.move_room_to_room(g, ABOVE);
                    }
                } else {
                    self.gliders[g].v_vel = CEILING_LIMIT - self.gliders[g].dest.top;
                }
            } else {
                self.gliders[g].v_vel = CEILING_LIMIT - self.gliders[g].dest.top;
            }
        } else {
            self.gliders[g].v_vel = CEILING_LIMIT - self.gliders[g].dest.top;
        }
    }

    //--------------------------------------------------------------  CheckEscapeDownTwo

    pub fn check_escape_down_two(&mut self, g: usize) {
        if self.bottom_open {
            if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                self.ia_escape_down(g);
            }
        } else if self.this_background == DIRT {
            let (left_tile, right_tile) = self.ia_glider_tiles(g);

            if (0..8).contains(&left_tile) && (0..8).contains(&right_tile) {
                let lt = self.this_tiles[left_tile as usize];
                let rt = self.this_tiles[right_tile as usize];
                if (lt == 2 || lt == 3) && (rt == 2 || rt == 3) {
                    if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                        self.ia_escape_down(g);
                    }
                } else {
                    if self.gliders[g].ignore_ground {
                        if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                            self.move_room_to_room(g, BELOW);
                        }
                    } else {
                        self.ia_floor_fade_out(g);
                    }
                }
            }
            // (tiles out of range: nothing, as in the C)
        } else {
            if self.gliders[g].ignore_ground {
                if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                    self.move_room_to_room(g, BELOW);
                }
            } else {
                self.ia_floor_fade_out(g);
            }
        }
    }

    //--------------------------------------------------------------  CheckEscapeDown

    pub fn check_escape_down(&mut self, g: usize) {
        if self.bottom_open {
            if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                self.move_room_to_room(g, BELOW);
            }
        } else if self.this_background == DIRT {
            let (left_tile, right_tile) = self.ia_glider_tiles(g);

            if (0..8).contains(&left_tile) && (0..8).contains(&right_tile) {
                let lt = self.this_tiles[left_tile as usize];
                let rt = self.this_tiles[right_tile as usize];
                if (lt == 2 || lt == 3) && (rt == 2 || rt == 3) {
                    if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                        self.move_room_to_room(g, BELOW);
                    }
                } else {
                    if self.gliders[g].ignore_ground {
                        if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                            self.move_room_to_room(g, BELOW);
                        }
                    } else {
                        self.ia_floor_fade_out(g);
                    }
                }
            } else {
                if self.gliders[g].ignore_ground {
                    if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                        self.move_room_to_room(g, BELOW);
                    }
                } else {
                    self.ia_floor_fade_out(g);
                }
            }
        } else {
            if self.gliders[g].ignore_ground {
                if self.gliders[g].dest.bottom > NO_FLOOR_LIMIT {
                    self.move_room_to_room(g, BELOW);
                }
            } else {
                self.ia_floor_fade_out(g);
            }
        }
    }

    //--------------------------------------------------------------  CheckRoofCollision

    pub fn check_roof_collision(&mut self, g: usize) {
        let gl = self.gliders[g];
        let offset = (gl.dest.left + HALF_GLIDER_WIDE) >> 6; // / 64
        if offset >= 0 && offset <= 7 && !gl.sliding {
            let tile_over = self.this_tiles[offset as usize];
            let into = (gl.dest.left + HALF_GLIDER_WIDE) - (offset << 6);
            if tile_over == 1 {
                if into > (250 - gl.dest.bottom) {
                    self.ia_floor_fade_out(g);
                }
            } else if tile_over == 2 {
                if into > (186 - gl.dest.bottom) {
                    self.ia_floor_fade_out(g);
                }
            } else if tile_over == 5 {
                if (64 - into) > (186 - gl.dest.bottom) {
                    self.ia_floor_fade_out(g);
                }
            } else if tile_over == 6 {
                if (64 - into) > (250 - gl.dest.bottom) {
                    self.ia_floor_fade_out(g);
                }
            } else {
                self.ia_floor_fade_out(g);
            }
        }
    }

    /// Hitting the left or right wall (repeated in the C).
    fn ia_hit_wall_sound(&mut self) {
        if self.foil_total > 0 {
            self.play_priority_sound(FOIL_HIT_SOUND, FOIL_HIT_PRIORITY);
        } else {
            self.play_priority_sound(HIT_WALL_SOUND, HIT_WALL_PRIORITY);
        }
    }

    //--------------------------------------------------------------  CheckEscapeLeftTwo

    pub fn check_escape_left_two(&mut self, g: usize) {
        if self.left_thresh == LEFT_WALL_LIMIT {
            if self.gliders[g].ignore_left {
                if self.gliders[g].dest.left < NO_LEFT_WALL_LIMIT {
                    self.ia_escape_left(g);
                }
            } else {
                self.ia_hit_wall_sound();
                let gl = &mut self.gliders[g];
                let offset = LEFT_WALL_LIMIT - gl.dest.left;
                gl.h_vel = -gl.h_vel + offset;
            }
        } else {
            self.ia_escape_left(g);
        }
    }

    //--------------------------------------------------------------  CheckEscapeLeft

    pub fn check_escape_left(&mut self, g: usize) {
        if self.left_thresh == LEFT_WALL_LIMIT {
            if self.gliders[g].ignore_left {
                if self.gliders[g].dest.left < NO_LEFT_WALL_LIMIT {
                    self.move_room_to_room(g, TO_LEFT);
                }
            } else {
                self.ia_hit_wall_sound();
                let gl = &mut self.gliders[g];
                let offset = LEFT_WALL_LIMIT - gl.dest.left;
                gl.h_vel = -gl.h_vel + offset;
            }
        } else {
            self.move_room_to_room(g, TO_LEFT);
        }
    }

    //--------------------------------------------------------------  CheckEscapeRightTwo

    pub fn check_escape_right_two(&mut self, g: usize) {
        if self.right_thresh == RIGHT_WALL_LIMIT {
            if self.gliders[g].ignore_right {
                if self.gliders[g].dest.right > NO_RIGHT_WALL_LIMIT {
                    self.ia_escape_right(g);
                }
            } else {
                self.ia_hit_wall_sound();
                let gl = &mut self.gliders[g];
                let offset = RIGHT_WALL_LIMIT - gl.dest.right;
                gl.h_vel = -gl.h_vel + offset;
            }
        } else {
            self.ia_escape_right(g);
        }
    }

    //--------------------------------------------------------------  CheckEscapeRight

    pub fn check_escape_right(&mut self, g: usize) {
        if self.right_thresh == RIGHT_WALL_LIMIT {
            if self.gliders[g].ignore_right {
                if self.gliders[g].dest.right > NO_RIGHT_WALL_LIMIT {
                    self.move_room_to_room(g, TO_RIGHT);
                }
            } else {
                self.ia_hit_wall_sound();
                let gl = &mut self.gliders[g];
                let offset = RIGHT_WALL_LIMIT - gl.dest.right;
                gl.h_vel = -gl.h_vel + offset;
            }
        } else {
            self.move_room_to_room(g, TO_RIGHT);
        }
    }

    //--------------------------------------------------------------  CheckGliderInRoom

    pub fn check_glider_in_room(&mut self, g: usize) {
        let mode = self.gliders[g].mode;
        if mode == GLIDER_NORMAL
            || mode == GLIDER_FACE_LEFT
            || mode == GLIDER_FACE_RIGHT
            || mode == GLIDER_BURNING
        {
            let two = self.two_player_game && !self.one_player_left;
            if self.gliders[g].dest.top < CEILING_LIMIT {
                if self.gliders[g].mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if two {
                    self.check_escape_up_two(g);
                } else {
                    self.check_escape_up(g);
                }
            } else if self.gliders[g].dest.bottom > FLOOR_LIMIT {
                if self.gliders[g].mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if two {
                    self.check_escape_down_two(g);
                } else {
                    self.check_escape_down(g);
                }
            } else if self.this_background == ROOF && self.gliders[g].dest.bottom > ROOF_LIMIT {
                self.check_roof_collision(g);
            }

            if self.gliders[g].dest.left < self.left_thresh {
                if self.gliders[g].mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.two_player_game && !self.one_player_left {
                    self.check_escape_left_two(g);
                } else {
                    self.check_escape_left(g);
                }
            } else if self.gliders[g].dest.right > self.right_thresh {
                if self.gliders[g].mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.two_player_game && !self.one_player_left {
                    self.check_escape_right_two(g);
                } else {
                    self.check_escape_right(g);
                }
            }
        }
    }

    /// The common reward step: the object restored, a sparkle, the glider
    /// slowed by half (HandleRewards' repeated block for prizes).
    fn ia_reward_sparkle(&mut self, g: usize, object_num: i32, bounds: &mut Rect) {
        self.play_priority_sound(ENERGIZE_SOUND, ENERGIZE_PRIORITY);
        self.restore_from_saved_map(self.this_room_number, object_num, false);
        self.add_sparkle(bounds);
        self.gliders[g].h_vel /= 2;
        self.gliders[g].v_vel /= 2;
    }

    /// A clock collected: points fly off, the glider slows to a quarter.
    fn ia_reward_clock(
        &mut self,
        g: usize,
        object_num: i32,
        bounds: &Rect,
        sound: i32,
        priority: i32,
        fly: i32,
        score: i32,
        cuckoo: bool,
    ) {
        let mut bounds = *bounds;
        self.play_priority_sound(sound, priority);
        self.restore_from_saved_map(self.this_room_number, object_num, false);
        if cuckoo {
            self.stop_pendulum(self.this_room_number, object_num);
        }
        let (hv, vv) = (self.gliders[g].h_vel, self.gliders[g].v_vel);
        self.add_flying_point(&mut bounds, fly, hv / 2, vv / 2);
        self.gliders[g].h_vel /= 4;
        self.gliders[g].v_vel /= 4;
        self.the_score += score;
        self.redraw_all_grease();
    }

    //--------------------------------------------------------------  HandleRewards

    pub fn handle_rewards(&mut self, g: usize, who: usize) {
        let who_linked = self.hot_spots[who].who;
        let mut bounds = self.hot_spots[who].bounds;
        let mo = self.master_objects[who_linked as usize];
        let room = self.this_room_number;
        let on = mo.object_num;

        match mo.the_object.what {
            RED_CLOCK => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_clock(
                        g,
                        on,
                        &bounds,
                        BEEPS_SOUND,
                        BEEPS_PRIORITY,
                        100,
                        RED_CLOCK_POINTS,
                        false,
                    );
                }
                self.hot_spots[who].is_on = false;
            }
            BLUE_CLOCK => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_clock(
                        g,
                        on,
                        &bounds,
                        BUZZER_SOUND,
                        BUZZER_PRIORITY,
                        300,
                        BLUE_CLOCK_POINTS,
                        false,
                    );
                }
                self.hot_spots[who].is_on = false;
            }
            YELLOW_CLOCK => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_clock(
                        g,
                        on,
                        &bounds,
                        DING_SOUND,
                        DING_PRIORITY,
                        500,
                        YELLOW_CLOCK_POINTS,
                        false,
                    );
                }
                self.hot_spots[who].is_on = false;
            }
            CUCKOO => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_clock(
                        g,
                        on,
                        &bounds,
                        CUCKOO_SOUND,
                        CUCKOO_PRIORITY,
                        1000,
                        CUCKOO_CLOCK_POINTS,
                        true,
                    );
                }
                self.hot_spots[who].is_on = false;
            }
            PAPER => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_sparkle(g, on, &mut bounds);
                    self.mortals += 1;
                    if self.two_player_game && !self.one_player_left {
                        self.mortals += 1;
                    }
                    self.quick_gliders_refresh();
                    self.redraw_all_grease();
                }
                self.hot_spots[who].is_on = false;
            }
            BATTERY => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_sparkle(g, on, &mut bounds);
                    if self.battery_total > 0 {
                        // positive number means battery power
                        self.battery_total += BATTERY_SUPPLY;
                    } else {
                        // negative number means helium gas
                        self.battery_total = BATTERY_SUPPLY;
                    }
                    if self.two_player_game && !self.one_player_left {
                        self.battery_total += BATTERY_SUPPLY;
                    }
                    self.quick_battery_refresh(false);
                    self.redraw_all_grease();
                }
                self.hot_spots[who].is_on = false;
            }
            BANDS => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_sparkle(g, on, &mut bounds);
                    self.bands_total += BANDS_SUPPLY;
                    if self.two_player_game && !self.one_player_left {
                        self.bands_total += BANDS_SUPPLY;
                    }
                    self.quick_bands_refresh(false);
                    self.redraw_all_grease();
                }
                self.hot_spots[who].is_on = false;
            }
            GREASE_RT | GREASE_LF => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.spill_grease(mo.dyna_num, mo.hot_num);
                }
                self.hot_spots[who].is_on = false;
            }
            FOIL => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_sparkle(g, on, &mut bounds);
                    self.foil_total += FOIL_SUPPLY;
                    if self.two_player_game && !self.one_player_left {
                        self.foil_total += FOIL_SUPPLY;
                    }
                    self.start_glider_foil_going(g);
                    self.redraw_all_grease();
                }
                self.hot_spots[who].is_on = false;
            }
            INVIS_BONUS => {
                let points = mo.the_object.c().points;
                if self.set_object_state(room, on, 0, who_linked) {
                    self.play_priority_sound(BONUS_SOUND, BONUS_PRIORITY);
                    let (hv, vv) = (self.gliders[g].h_vel, self.gliders[g].v_vel);
                    self.add_flying_point(&mut bounds, points, hv / 2, vv / 2);
                    self.gliders[g].h_vel /= 4;
                    self.gliders[g].v_vel /= 4;
                    self.the_score += points;
                }
                self.hot_spots[who].is_on = false;
            }
            STAR => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.play_priority_sound(ENERGIZE_SOUND, ENERGIZE_PRIORITY);
                    self.restore_from_saved_map(room, on, false);
                    self.add_sparkle(&mut bounds);
                    self.stop_star(room, on);
                    self.num_stars_remaining -= 1;
                    if self.num_stars_remaining <= 0 {
                        self.flag_game_over();
                    } else {
                        self.display_stars_remaining();
                    }
                    self.redraw_all_grease();
                    self.the_score += STAR_POINTS;
                }
                self.hot_spots[who].is_on = false;
            }
            SPARKLE => {}
            HELIUM => {
                if self.set_object_state(room, on, 0, who_linked) {
                    self.ia_reward_sparkle(g, on, &mut bounds);
                    if self.battery_total < 0 {
                        // if negative, it is already helium gas
                        self.battery_total -= HELIUM_SUPPLY;
                    } else {
                        // if positive, it is battery power
                        self.battery_total = -HELIUM_SUPPLY;
                    }
                    if self.two_player_game && !self.one_player_left {
                        self.battery_total -= HELIUM_SUPPLY;
                    }
                    self.quick_battery_refresh(false);
                    self.redraw_all_grease();
                }
                self.hot_spots[who].is_on = false;
            }
            SLIDER => {}
            _ => {}
        }
    }

    //--------------------------------------------------------------  HandleSwitches

    pub fn handle_switches(&mut self, who: usize) {
        if self.hot_spots[who].still_over {
            return;
        }

        let who_linked = self.hot_spots[who].who; // what is switch's obj. #
        let mo = self.master_objects[who_linked as usize];
        let room_linked = mo.room_link;
        let object_linked = mo.object_link;
        let link_index = mo.local_link;
        // `bounds` is never set in the C (an uninitialized Rect passed to
        // AddSparkle below); an empty rect here.
        let mut bounds = Rect::default();
        // change state of linked obj.
        if self.set_object_state(
            room_linked,
            object_linked,
            mo.the_object.e().type_,
            link_index,
        ) {
            let mut new_rect = self.hot_spots[who].bounds;
            q_offset_rect(&mut new_rect, self.play_origin_h, self.play_origin_v);
            let new_state = self.new_state;
            match mo.the_object.what {
                LIGHT_SWITCH => {
                    self.play_priority_sound(SWITCH_SOUND, SWITCH_PRIORITY);
                    self.draw_light_switch(&new_rect, new_state);
                }
                MACHINE_SWITCH => {
                    self.play_priority_sound(SWITCH_SOUND, SWITCH_PRIORITY);
                    self.draw_machine_switch(&new_rect, new_state);
                }
                THERMOSTAT => {
                    self.play_priority_sound(SWITCH_SOUND, SWITCH_PRIORITY);
                    self.draw_thermostat(&new_rect, new_state);
                }
                POWER_SWITCH => {
                    self.play_priority_sound(SWITCH_SOUND, SWITCH_PRIORITY);
                    self.draw_power_switch(&new_rect, new_state);
                }
                KNIFE_SWITCH => {
                    self.play_priority_sound(SWITCH_SOUND, SWITCH_PRIORITY);
                    self.draw_knife_switch(&new_rect, new_state);
                }
                INVIS_SWITCH => {}
                _ => {}
            }
            self.copy_rect_back_to_work(&new_rect);
            self.add_rect_to_work_rects(&new_rect);

            if link_index != -1 {
                let lo = self.master_objects[link_index as usize];
                let dyna = lo.dyna_num;
                match lo.the_object.what {
                    RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | PAPER | BATTERY | BANDS | FOIL
                    | STAR | HELIUM => {
                        self.restore_from_saved_map(room_linked, object_linked, true);
                        self.add_sparkle(&mut bounds);
                    }
                    CUCKOO => {
                        self.restore_from_saved_map(room_linked, object_linked, true);
                        self.stop_pendulum(room_linked, object_linked);
                    }
                    GREASE_RT | GREASE_LF => {
                        self.spill_grease(lo.dyna_num, lo.hot_num);
                    }
                    INVIS_BONUS | SLIDER => {}
                    DELUXE_TRANS => {}
                    SOUND_TRIGGER => {
                        self.play_priority_sound(TRIGGER_SOUND, TRIGGER_PRIORITY);
                    }
                    CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP
                    | FLOURESCENT | TRACK_LIGHT | INVIS_LIGHT => {
                        self.redraw_room_lighting();
                    }
                    SHREDDER => {}
                    TOASTER => self.toggle_toaster(dyna),
                    MAC_PLUS => self.toggle_mac_plus(dyna),
                    GUITAR => self.play_priority_sound(CHORD_SOUND, CHORD_PRIORITY),
                    TV => self.toggle_tv(dyna),
                    COFFEE => self.toggle_coffee(dyna),
                    OUTLET => self.toggle_outlet(dyna),
                    VCR => self.toggle_vcr(dyna),
                    STEREO => self.toggle_stereos(dyna),
                    MICROWAVE => self.toggle_microwave(dyna),
                    BALLOON => self.toggle_balloon(dyna),
                    COPTER_LF | COPTER_RT => self.toggle_copter(dyna),
                    DART_LF | DART_RT => self.toggle_dart(dyna),
                    BALL => self.toggle_ball(dyna),
                    DRIP => self.toggle_drip(dyna),
                    FISH => self.toggle_fish(dyna),
                    _ => {}
                }
            }
        }

        self.hot_spots[who].still_over = true;
    }

    //--------------------------------------------------------------  HandleMicrowaveAction

    pub fn handle_microwave_action(&mut self, who: usize, g: usize) {
        if self.hot_spots[who].still_over {
            return;
        }

        let mut killed = false;
        let who_linked = self.hot_spots[who].who; // what is microwave's obj. #
        let obj = self.master_objects[who_linked as usize].the_object;
        if obj.g().state {
            let kills = obj.g().byte0;
            if (kills & 0x0001) == 0x0001 && self.bands_total > 0 {
                self.bands_total = 0;
                killed = true;
                self.quick_bands_refresh(false);
            }
            if (kills & 0x0002) == 0x0002 && self.battery_total != 0 {
                self.battery_total = 0;
                killed = true;
                self.quick_battery_refresh(false);
            }
            if (kills & 0x0004) == 0x0004 && self.foil_total > 0 {
                self.foil_total = 0;
                killed = true;
                self.start_glider_foil_losing(g);
            }
        }

        if killed {
            self.play_priority_sound(MICROWAVED_SOUND, MICROWAVED_PRIORITY);
        }
    }

    //--------------------------------------------------------------  HandleHotSpotCollision

    pub fn handle_hot_spot_collision(&mut self, g: usize, who: usize, index: i32) {
        let hot = self.hot_spots[who];
        let two = self.two_player_game && !self.one_player_left;
        match hot.action {
            LIFT_IT => {
                self.gliders[g].v_desired_vel = FLOOR_VENT_LIFT;
            }
            DROP_IT => {
                self.gliders[g].v_desired_vel = CEILING_VENT_DROP;
            }
            PUSH_IT_LEFT => {
                self.gliders[g].h_desired_vel += -FAN_STRENGTH;
            }
            PUSH_IT_RIGHT => {
                self.gliders[g].h_desired_vel += FAN_STRENGTH;
            }
            DISSOLVE_IT => {
                if self.gliders[g].mode != GLIDER_FADING_OUT {
                    if self.foil_total > 0 || self.gliders[g].mode == GLIDER_LOSING_FOIL {
                        if self.glider_hit_top(g, &hot.bounds) {
                            self.start_glider_fading_out(g);
                            self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                        } else {
                            if self.foil_total > 0 {
                                self.foil_total -= 1;
                                if self.foil_total <= 0 {
                                    self.start_glider_foil_losing(g);
                                }
                            }
                        }
                    } else {
                        self.start_glider_fading_out(g);
                        self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                    }
                }
            }
            REWARD_IT => {
                self.handle_rewards(g, who);
            }
            MOVE_IT_UP => {
                if !self.gliders[g].held_right && self.glider_in_rect(g, &hot.bounds) {
                    let mode = self.gliders[g].mode;
                    if mode == GLIDER_BURNING {
                        self.ia_burnt_fade_out(g);
                    } else if two {
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            if mode != GLIDER_GOING_UP && mode != GLIDER_IN_LIMBO {
                                self.other_player_escaped = PLAYER_ESCAPING_UP_STAIRS;
                                self.refresh_scoreboard(ESCAPED_TITLE_MODE);
                                self.start_glider_going_up_stairs(g);
                            }
                        } else if self.other_player_escaped == PLAYER_ESCAPED_UP_STAIRS {
                            if mode != GLIDER_GOING_UP && mode != GLIDER_IN_LIMBO {
                                self.start_glider_going_up_stairs(g);
                            }
                        }
                    } else {
                        self.start_glider_going_up_stairs(g);
                    }
                }
            }
            MOVE_IT_DOWN => {
                if !self.gliders[g].held_left && self.glider_in_rect(g, &hot.bounds) {
                    let mode = self.gliders[g].mode;
                    if mode == GLIDER_BURNING {
                        self.ia_burnt_fade_out(g);
                    } else if two {
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            if mode != GLIDER_GOING_DOWN && mode != GLIDER_IN_LIMBO {
                                self.other_player_escaped = PLAYER_ESCAPING_DOWN_STAIRS;
                                self.refresh_scoreboard(ESCAPED_TITLE_MODE);
                                self.start_glider_going_down_stairs(g);
                            }
                        } else if self.other_player_escaped == PLAYER_ESCAPED_DOWN_STAIRS {
                            if mode != GLIDER_GOING_DOWN && mode != GLIDER_IN_LIMBO {
                                self.start_glider_going_down_stairs(g);
                            }
                        }
                    } else {
                        self.start_glider_going_down_stairs(g);
                    }
                }
            }
            SWITCH_IT => {
                self.handle_switches(who);
            }
            SHRED_IT => {
                if self.gliders[g].mode != GLIDER_SHREDDING && self.glider_in_rect(g, &hot.bounds) {
                    if self.foil_total > 0 || self.gliders[g].mode == GLIDER_LOSING_FOIL {
                        self.play_priority_sound(FOIL_HIT_SOUND, FOIL_HIT_PRIORITY);
                        if self.foil_total > 0 {
                            self.foil_total -= 1;
                            if self.foil_total <= 0 {
                                self.start_glider_foil_losing(g);
                            }
                        }
                    } else {
                        self.flag_glider_shredding(g, &hot.bounds);
                    }
                }
            }
            STRUM_IT => {
                if !self.hot_spots[who].still_over {
                    self.play_priority_sound(CHORD_SOUND, CHORD_PRIORITY);
                    self.hot_spots[who].still_over = true;
                }
            }
            TRIGGER_IT | LG_TRIGGER => {
                self.arm_trigger(who);
            }
            BURN_IT => {
                let mode = self.gliders[g].mode;
                if mode != GLIDER_BURNING && mode != GLIDER_FADING_OUT {
                    if self.foil_total > 0 || mode == GLIDER_LOSING_FOIL {
                        self.gliders[g].v_desired_vel = FLOOR_VENT_LIFT;
                        if self.foil_total > 0 {
                            self.play_priority_sound(SIZZLE_SOUND, SIZZLE_PRIORITY);
                            self.foil_total -= 1;
                            if self.foil_total <= 0 {
                                self.start_glider_foil_losing(g);
                            }
                        }
                    } else {
                        self.flag_glider_burning(g);
                    }
                }
            }
            SLIDE_IT => {
                self.gliders[g].sliding = true;
                self.gliders[g].v_vel = hot.bounds.top - self.gliders[g].dest.bottom;
            }
            TRANSPORT_IT => {
                let mode = self.gliders[g].mode;
                if mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.glider_in_rect(g, &hot.bounds)
                    && mode != GLIDER_TRANSPORTING
                    && mode != GLIDER_FADING_OUT
                {
                    if two {
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            if mode != GLIDER_IN_LIMBO {
                                self.active_rect_escaped = index;
                                self.start_glider_transporting(g, who);
                            }
                        } else if self.other_player_escaped == PLAYER_TRANSPORTED_OUT {
                            if mode != GLIDER_IN_LIMBO && self.active_rect_escaped == index {
                                self.start_glider_transporting(g, who);
                            }
                        }
                    } else {
                        self.start_glider_transporting(g, who);
                    }
                }
            }
            IGNORE_LEFT_WALL => {
                self.gliders[g].ignore_left = true;
            }
            IGNORE_RIGHT_WALL => {
                self.gliders[g].ignore_right = true;
            }
            MAIL_IT_LEFT => {
                // mailbox open to right
                let gl = self.gliders[g];
                if gl.mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.glider_in_rect(g, &hot.bounds)
                    && gl.mode != GLIDER_MAIL_OUT_RIGHT
                    && gl.mode != GLIDER_MAIL_IN_LEFT
                    && gl.mode != GLIDER_FADING_OUT
                    && ((gl.facing == FACE_RIGHT && !gl.tipped)
                        || (gl.facing == FACE_LEFT && gl.tipped))
                {
                    if two {
                        // two gliders to handle
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            // other glider in room
                            if gl.mode != GLIDER_IN_LIMBO {
                                // this glider is active
                                self.active_rect_escaped = index;
                                self.start_glider_mailing_in(g, &hot.bounds, who);
                                self.gliders[g].mode = GLIDER_MAIL_IN_LEFT;
                            }
                        } else if self.other_player_escaped == PLAYER_MAILED_OUT {
                            // other glider left here
                            if gl.mode != GLIDER_IN_LIMBO && self.active_rect_escaped == index {
                                // []_ <--G
                                self.start_glider_mailing_in(g, &hot.bounds, who);
                                self.gliders[g].mode = GLIDER_MAIL_IN_LEFT;
                            }
                        }
                    } else {
                        // only 1 glider in game
                        self.start_glider_mailing_in(g, &hot.bounds, who);
                        self.gliders[g].mode = GLIDER_MAIL_IN_LEFT;
                    }
                }
            }
            MAIL_IT_RIGHT => {
                // mailbox open to left
                let gl = self.gliders[g];
                if gl.mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.glider_in_rect(g, &hot.bounds)
                    && gl.mode != GLIDER_MAIL_OUT_LEFT
                    && gl.mode != GLIDER_MAIL_IN_RIGHT
                    && gl.mode != GLIDER_FADING_OUT
                    && ((gl.facing == FACE_RIGHT && gl.tipped)
                        || (gl.facing == FACE_LEFT && !gl.tipped))
                {
                    if two {
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            if gl.mode != GLIDER_IN_LIMBO {
                                self.active_rect_escaped = index;
                                self.start_glider_mailing_in(g, &hot.bounds, who);
                                self.gliders[g].mode = GLIDER_MAIL_IN_RIGHT;
                            }
                        } else if self.other_player_escaped == PLAYER_MAILED_OUT {
                            if gl.mode != GLIDER_IN_LIMBO && self.active_rect_escaped == index {
                                self.start_glider_mailing_in(g, &hot.bounds, who);
                                self.gliders[g].mode = GLIDER_MAIL_IN_RIGHT;
                            }
                        }
                    } else {
                        self.start_glider_mailing_in(g, &hot.bounds, who);
                        self.gliders[g].mode = GLIDER_MAIL_IN_RIGHT;
                    }
                }
            }
            DUCT_IT_DOWN => {
                let mode = self.gliders[g].mode;
                if mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.glider_in_rect(g, &hot.bounds)
                    && mode != GLIDER_DUCTING_DOWN
                    && mode != GLIDER_FADING_OUT
                {
                    if two {
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            if mode != GLIDER_IN_LIMBO {
                                self.active_rect_escaped = index;
                                self.start_glider_ducting_down(g, &hot.bounds, who);
                            }
                        } else if self.other_player_escaped == PLAYER_DUCKED_OUT {
                            if mode != GLIDER_IN_LIMBO && self.active_rect_escaped == index {
                                self.start_glider_ducting_down(g, &hot.bounds, who);
                            }
                        }
                    } else {
                        self.start_glider_ducting_down(g, &hot.bounds, who);
                    }
                }
            }
            DUCT_IT_UP => {
                let mode = self.gliders[g].mode;
                if mode == GLIDER_BURNING {
                    self.ia_burnt_fade_out(g);
                } else if self.glider_in_rect(g, &hot.bounds)
                    && mode != GLIDER_DUCTING_UP
                    && mode != GLIDER_DUCTING_IN
                    && mode != GLIDER_FADING_OUT
                    && !self.hot_spots[who].still_over
                {
                    if two {
                        if self.other_player_escaped == NO_ONE_ESCAPED {
                            if mode != GLIDER_IN_LIMBO {
                                self.active_rect_escaped = index;
                                self.start_glider_ducting_up(g, &hot.bounds, who);
                            }
                        } else if self.other_player_escaped == PLAYER_DUCKED_OUT {
                            if mode != GLIDER_IN_LIMBO && self.active_rect_escaped == index {
                                self.start_glider_ducting_up(g, &hot.bounds, who);
                            }
                        }
                    } else {
                        self.start_glider_ducting_up(g, &hot.bounds, who);
                        self.hot_spots[who].still_over = true;
                    }
                }
            }
            MICROWAVE_IT => {
                if self.glider_in_rect(g, &hot.bounds) {
                    self.handle_microwave_action(who, g);
                }
            }
            IGNORE_GROUND => {
                self.gliders[g].ignore_ground = true;
            }
            BOUNCE_IT => {
                self.bounce_glider(g, &hot.bounds);
            }
            CHIME_IT => {
                if !self.hot_spots[who].still_over {
                    self.strike_chime();
                    self.hot_spots[who].still_over = true;
                }
            }
            WEB_IT => {
                if self.glider_in_rect(g, &hot.bounds) && self.gliders[g].mode != GLIDER_BURNING {
                    self.web_glider(g, &hot.bounds);
                } else if self.gliders[g].mode == GLIDER_BURNING
                    && self.glider_in_rect(g, &hot.bounds)
                {
                    self.ia_burnt_fade_out(g);
                }
            }
            SOUND_IT => {
                if !self.hot_spots[who].still_over {
                    self.play_priority_sound(TRIGGER_SOUND, TRIGGER_PRIORITY);
                    self.hot_spots[who].still_over = true;
                }
            }
            _ => {}
        }
    }

    //--------------------------------------------------------------  CheckForHotSpots

    pub fn check_for_hot_spots(&mut self) {
        for i in 0..self.n_hot_spots as usize {
            if self.hot_spots[i].is_on {
                if self.two_player_game {
                    let mut hit_object = false;
                    let (b, s) = (self.hot_spots[i].bounds, self.hot_spots[i].do_scrutinize);
                    if self.sect_glider(0, &b, s) {
                        if self.one_player_left {
                            if self.player_dead == PLAYER2 {
                                self.handle_hot_spot_collision(0, i, i as i32);
                                hit_object = true;
                            }
                        } else {
                            self.handle_hot_spot_collision(0, i, i as i32);
                            hit_object = true;
                        }
                    }

                    // (re-read: the first glider's collision may change it)
                    let (b, s) = (self.hot_spots[i].bounds, self.hot_spots[i].do_scrutinize);
                    if self.sect_glider(1, &b, s) {
                        if self.one_player_left {
                            if self.player_dead == PLAYER1 {
                                self.handle_hot_spot_collision(1, i, i as i32);
                                hit_object = true;
                            }
                        } else {
                            self.handle_hot_spot_collision(1, i, i as i32);
                            hit_object = true;
                        }
                    }
                    if !hit_object {
                        self.hot_spots[i].still_over = false;
                    }
                } else {
                    let (b, s) = (self.hot_spots[i].bounds, self.hot_spots[i].do_scrutinize);
                    if self.sect_glider(0, &b, s) {
                        self.handle_hot_spot_collision(0, i, i as i32);
                    } else {
                        self.hot_spots[i].still_over = false;
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  HandleInteraction

    pub fn handle_interaction(&mut self) {
        self.check_for_hot_spots();
        if self.two_player_game {
            if self.one_player_left {
                if self.player_dead == PLAYER1 {
                    self.check_glider_in_room(1);
                } else {
                    self.check_glider_in_room(0);
                }
            } else {
                self.check_glider_in_room(0);
                self.check_glider_in_room(1);
            }
        } else {
            self.check_glider_in_room(0);
        }
    }

    //--------------------------------------------------------------  FlagStillOvers

    pub fn flag_still_overs(&mut self, g: usize) {
        for i in 0..self.n_hot_spots as usize {
            if self.hot_spots[i].is_on {
                let (b, s) = (self.hot_spots[i].bounds, self.hot_spots[i].do_scrutinize);
                self.hot_spots[i].still_over = self.sect_glider(g, &b, s);
            } else {
                self.hot_spots[i].still_over = false;
            }
        }
    }

    //--------------------------------------------------------------  WebGlider

    pub fn web_glider(&mut self, g: usize, web_bounds: &Rect) {
        if self.gliders[g].mode == GLIDER_BURNING && self.glider_in_rect(g, web_bounds) {
            self.ia_burnt_fade_out(g);
            return;
        }

        let gl = self.gliders[g];
        let h_dist = ((web_bounds.right - gl.dest.right) + (web_bounds.left - gl.dest.left)) >> 3;
        let v_dist = ((web_bounds.bottom - gl.dest.bottom) + (web_bounds.top - gl.dest.top)) >> 3;

        if gl.h_desired_vel != 0 {
            if self.even_frame {
                self.gliders[g].h_vel = h_dist;
                self.gliders[g].v_vel = v_dist;
                self.play_priority_sound(WEB_TWANG_SOUND, WEB_TWANG_PRIORITY);
            }
        } else {
            self.gliders[g].h_desired_vel = 0;
            self.gliders[g].v_desired_vel = 0;
        }

        self.gliders[g].was_mode += 1;
        if self.gliders[g].was_mode >= KILL_WEBBED_GLIDER {
            self.ia_burnt_fade_out(g);
        }
    }
}

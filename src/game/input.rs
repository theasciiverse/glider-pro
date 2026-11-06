//! Input.c: reading the keys into the glider's thrust, battery and bands.
//!
//! GetKeys / BitTst on the glider's key codes read `self.keys` instead: the
//! player one keys (`left`, `right`, `batt`, `band`) or player two's
//! (`left2` ...) by the glider's `which`. The glider's `left_key` etc. (set
//! only by the cut Settings dialog) are not used.
//!
//! Pausing: DoPause's waiting loops are the terminal's. `do_pause` shows
//! the pause picture and sets `paused`; while `paused`, the terminal stops
//! calling the play loop and waits for the pause key (Escape, or Tab: see
//! `is_esc_pause_key`), then calls `end_pause`. Command-Q (`keys.quit`)
//! ends the game (`playing = false`); saving a game (Command-S, the
//! "save game?" alert) is cut, and demo recording (CREATEDEMODATA) too.

use super::*;

pub const NORMAL_THRUST: i32 = 5;
pub const HYPER_THRUST: i32 = 8;
pub const HELIUM_LIFT: i32 = 4;
pub const ESC_PAUSE_PICT_ID: i32 = 1015;
pub const TAB_PAUSE_PICT_ID: i32 = 1016;

/// The four keys of a glider, as GetKeys would report them.
#[derive(Clone, Copy, Debug, Default)]
pub struct GliderKeys {
    pub left: bool,
    pub right: bool,
    pub batt: bool,
    pub band: bool,
}

impl Game {
    /// BitTst(&theKeys, thisGlider->leftKey) etc. for the glider `g`.
    pub fn glider_keys(&self, g: usize) -> GliderKeys {
        let k = &self.keys;
        if self.gliders[g].which == PLAYER1 {
            GliderKeys {
                left: k.left,
                right: k.right,
                batt: k.batt,
                band: k.band,
            }
        } else {
            GliderKeys {
                left: k.left2,
                right: k.right2,
                batt: k.batt2,
                band: k.band2,
            }
        }
    }

    /// The pause key down: Escape if `is_esc_pause_key`, else Tab.
    pub fn pause_key_down(&self) -> bool {
        (self.is_esc_pause_key && self.keys.escape) || (!self.is_esc_pause_key && self.keys.tab)
    }

    /// The pause picture's place, centred in the house rect (DoPause).
    pub fn pause_bounds(&self) -> Rect {
        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 214, 54);
        center_rect_in_rect(&mut bounds, &self.house_rect);
        bounds
    }

    //--------------------------------------------------------------  LogDemoKey
    // (CREATEDEMODATA only: not kept.)

    //--------------------------------------------------------------  DoCommandKey

    pub fn do_command_key(&mut self) {
        if self.keys.quit {
            self.playing = false;
            self.paused = false;
            // QuerySaveGame / SaveGame2: saved games are cut.
        }
        // Command-S (save the game): cut.
    }

    //--------------------------------------------------------------  DoPause

    /// The first half of DoPause: shows the pause picture and flags the
    /// pause; the terminal waits for the pause key, then calls `end_pause`.
    pub fn do_pause(&mut self) {
        let bounds = self.pause_bounds();
        let id = if self.is_esc_pause_key {
            ESC_PAUSE_PICT_ID
        } else {
            TAB_PAUSE_PICT_ID
        };
        if let Some(img) = self.picture(id) {
            self.main_window_map.draw_picture(&img, bounds);
        } else {
            log::warn!("PICT {id} missing");
        }

        self.paused = true;
    }

    /// The end of DoPause: puts back what the pause picture covered.
    pub fn end_pause(&mut self) {
        self.paused = false;
        let bounds = self.pause_bounds();
        self.main_window_map
            .copy_bits(&self.work_src_map, bounds, bounds);
    }

    //--------------------------------------------------------------  DoBatteryEngaged

    pub fn do_battery_engaged(&mut self, g: usize) {
        let gl = &mut self.gliders[g];
        if gl.facing == FACE_LEFT {
            if gl.tipped {
                gl.h_vel += HYPER_THRUST;
            } else {
                gl.h_vel -= HYPER_THRUST;
            }
        } else {
            if gl.tipped {
                gl.h_vel -= HYPER_THRUST;
            } else {
                gl.h_vel += HYPER_THRUST;
            }
        }

        self.battery_total -= 1;

        if self.battery_total == 0 {
            self.quick_battery_refresh(false);
            self.play_priority_sound(FIZZLE_SOUND, FIZZLE_PRIORITY);
        } else {
            if !self.battery_was_engaged {
                self.battery_frame = 0;
            }
            if self.battery_frame == 0 {
                self.play_priority_sound(THRUST_SOUND, THRUST_PRIORITY);
            }
            self.battery_frame += 1;
            if self.battery_frame >= 4 {
                self.battery_frame = 0;
            }
            self.battery_was_engaged = true;
        }
    }

    //--------------------------------------------------------------  DoHeliumEngaged

    pub fn do_helium_engaged(&mut self, g: usize) {
        self.gliders[g].v_desired_vel = -HELIUM_LIFT;
        self.battery_total += 1;

        if self.battery_total == 0 {
            self.quick_battery_refresh(false);
            self.play_priority_sound(FIZZLE_SOUND, FIZZLE_PRIORITY);
            self.battery_was_engaged = false;
        } else {
            if !self.battery_was_engaged {
                self.battery_frame = 0;
            }
            if self.battery_frame == 0 {
                self.play_priority_sound(HISS_SOUND, HISS_PRIORITY);
            }
            self.battery_frame += 1;
            if self.battery_frame >= 4 {
                self.battery_frame = 0;
            }
            self.battery_was_engaged = true;
        }
    }

    //--------------------------------------------------------------  GetDemoInput

    pub fn get_demo_input(&mut self, g: usize) {
        if self.gliders[g].which == PLAYER1 {
            // GetKeys: `self.keys` is already this frame's.
            if self.keys.quit {
                self.do_command_key();
            }
        }

        if self.gliders[g].mode == GLIDER_BURNING {
            let gl = &mut self.gliders[g];
            if gl.facing == FACE_LEFT {
                gl.h_desired_vel -= NORMAL_THRUST;
            } else {
                gl.h_desired_vel += NORMAL_THRUST;
            }
        } else {
            {
                let gl = &mut self.gliders[g];
                gl.held_left = false;
                gl.held_right = false;
                gl.tipped = false;
            }

            let di = self.demo_index as usize;
            if di < self.demo_data.len() && self.game_frame == self.demo_data[di].frame {
                match self.demo_data[di].key {
                    0 => {
                        // left key
                        let gl = &mut self.gliders[g];
                        gl.h_desired_vel += NORMAL_THRUST;
                        gl.tipped = gl.facing == FACE_LEFT;
                        gl.held_right = true;
                        gl.fire_held = false;
                    }
                    1 => {
                        // right key
                        let gl = &mut self.gliders[g];
                        gl.h_desired_vel -= NORMAL_THRUST;
                        gl.tipped = gl.facing == FACE_RIGHT;
                        gl.held_left = true;
                        gl.fire_held = false;
                    }
                    2 => {
                        // battery key
                        if self.battery_total > 0 {
                            self.do_battery_engaged(g);
                        } else {
                            self.do_helium_engaged(g);
                        }
                        self.gliders[g].fire_held = false;
                    }
                    3 => {
                        // rubber band key
                        if !self.gliders[g].fire_held {
                            let gl = self.gliders[g];
                            if self.add_band(g, gl.dest.left + 24, gl.dest.top + 10, gl.facing) {
                                self.bands_total -= 1;
                                if self.bands_total <= 0 {
                                    self.quick_bands_refresh(false);
                                }

                                self.gliders[g].fire_held = true;
                            }
                        }
                    }
                    _ => {}
                }

                self.demo_index += 1;
            } else {
                self.gliders[g].fire_held = false;
            }

            if self.pause_key_down() {
                self.do_pause();
            }
        }
    }

    //--------------------------------------------------------------  GetInput

    pub fn get_input(&mut self, g: usize) {
        if self.gliders[g].which == PLAYER1 {
            // GetKeys: `self.keys` is already this frame's.
            if self.keys.quit {
                self.do_command_key();
            }
        }

        if self.gliders[g].mode == GLIDER_BURNING {
            let gl = &mut self.gliders[g];
            if gl.facing == FACE_LEFT {
                gl.h_desired_vel -= NORMAL_THRUST;
            } else {
                gl.h_desired_vel += NORMAL_THRUST;
            }
        } else {
            let keys = self.glider_keys(g);
            self.gliders[g].held_left = false;
            self.gliders[g].held_right = false;
            if keys.right {
                // right key
                if keys.left {
                    self.toggle_glider_facing(g);
                    self.gliders[g].held_left = true;
                } else {
                    let gl = &mut self.gliders[g];
                    gl.h_desired_vel += NORMAL_THRUST;
                    gl.tipped = gl.facing == FACE_LEFT;
                    gl.held_right = true;
                }
            } else if keys.left {
                // left key
                let gl = &mut self.gliders[g];
                gl.h_desired_vel -= NORMAL_THRUST;
                gl.tipped = gl.facing == FACE_RIGHT;
                gl.held_left = true;
            } else {
                self.gliders[g].tipped = false;
            }

            if keys.batt && self.battery_total != 0 && self.gliders[g].mode == GLIDER_NORMAL {
                if self.battery_total > 0 {
                    self.do_battery_engaged(g);
                } else {
                    self.do_helium_engaged(g);
                }
            } else {
                self.battery_was_engaged = false;
            }

            if keys.band && self.bands_total > 0 && self.gliders[g].mode == GLIDER_NORMAL {
                if !self.gliders[g].fire_held {
                    let gl = self.gliders[g];
                    if self.add_band(g, gl.dest.left + 24, gl.dest.top + 10, gl.facing) {
                        self.bands_total -= 1;
                        if self.bands_total <= 0 {
                            self.quick_bands_refresh(false);
                        }

                        self.gliders[g].fire_held = true;
                    }
                }
            } else {
                self.gliders[g].fire_held = false;
            }

            if self.other_player_escaped != NO_ONE_ESCAPED
                && self.keys.delete
                && self.gliders[g].which
                && !self.one_player_left
            {
                self.force_kill_glider();
            }

            if self.pause_key_down() {
                self.do_pause();
            }
        }
    }

    //--------------------------------------------------------------  QuerySaveGame
    // (the "save game?" alert: saved games are cut.)
}

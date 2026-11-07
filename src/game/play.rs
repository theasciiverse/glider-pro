//! Play.c: starting a game, the play loop, the telephone and the chimes.
//!
//! The C's `PlayGame` loops until the game ends; here the terminal drives
//! it: `new_game` sets up and returns with `playing` true, the terminal
//! calls `play_game_frame` once every kTicksPerFrame ticks (advancing
//! `ticks`) while `playing && !quitting`, then `play_game_finish` (what
//! follows the loop in NewGame: game over has been shown, back to the
//! splash).

use super::*;
use crate::gfx;

pub const HOUSE_BANNER_ALERT: i32 = 1009;
pub const INITIAL_GLIDERS: i32 = 2;
pub const RING_DELAY: i32 = 90;
pub const RING_SPREAD: i32 = 25000; // 25000
pub const RING_BASE_DELAY: i32 = 5000; // 5000
pub const CHIME_DELAY: i32 = 180;

impl Game {
    /// LoadGraphic(id) into one of the glider maps (they keep their size).
    fn load_glider_graphic(&mut self, second: bool, id: i32) {
        let r = self.glid_src_rect;
        match self.picture(id) {
            Some(img) => {
                if second {
                    self.glid2_src_map.draw_picture(&img, r);
                } else {
                    self.glid_src_map.draw_picture(&img, r);
                }
            }
            None => log::warn!("PICT {id} missing"),
        }
    }

    //--------------------------------------------------------------  NewGame

    /// NewGame up to `PlayGame()`: the game's set-up. Returns with
    /// `playing` true; the play loop is `play_game_frame`.
    pub fn new_game(&mut self, mode: i32) {
        self.adjust_scoreboard_height();
        self.game_over = false;
        self.the_mode = PLAY_MODE;
        if self.is_play_music_game {
            if !self.is_music_on {
                let the_err = self.start_music();
                if the_err != 0 {
                    log::warn!("YellowAlert kYellowNoMusic {the_err}");
                    self.failed_music = true;
                }
            }
            self.set_musical_mode(PLAY_GAME_SCORE_MODE);
        } else {
            if self.is_music_on {
                self.stop_the_music();
            }
        }
        if mode != RESUME_GAME_MODE {
            self.set_objects_to_defaults();
        }
        // HideCursor: none.
        if mode == RESUME_GAME_MODE {
            // SetHouseToSavedRoom: saved games are cut.
            log::warn!("resume game: saved games are not kept");
        } else if mode == NEW_GAME_MODE {
            self.set_house_to_first_room();
        }
        self.determine_room_openings();
        self.nil_saved_maps();

        self.game_frame = 0;
        self.num_bands = 0;
        self.demo_index = 0;
        self.said_follow = 0;
        self.other_player_escaped = NO_ONE_ESCAPED;
        self.one_player_left = false;
        self.player_suicide = false;

        if self.two_player_game {
            // initialize glider(s)
            self.init_glider(0, NEW_GAME_MODE);
            self.init_glider(1, NEW_GAME_MODE);
            self.load_glider_graphic(false, GLIDER_PICT_ID);
            self.load_glider_graphic(true, GLIDER2_PICT_ID);
        } else {
            self.init_glider(0, mode);
            self.load_glider_graphic(false, GLIDER_PICT_ID);
            self.load_glider_graphic(true, GLIDER_FOIL_PICT_ID);
        }

        // paint strip on screen black
        let mut temp_rect = self.screen;
        temp_rect.top = temp_rect.bottom - 20; // thisMac.menuHigh
        self.main_window_map.fill_rect(temp_rect, gfx::BLACK);

        // (COMPILEQT: the TV's movie; not kept)

        let r = self.work_src_rect;
        self.work_src_map.fill_rect(r, gfx::BLACK);
        //	if (quickerTransitions)
        //		DissBitsChunky(&workSrcRect);
        //	else
        //		DissBits(&workSrcRect);

        self.draw_locale();
        self.refresh_scoreboard(NORMAL_TITLE_MODE);
        //	if (quickerTransitions)
        //		DissBitsChunky(&justRoomsRect);
        //	else
        //		DissBits(&justRoomsRect);
        let just_rooms_rect = self.just_rooms_rect;
        if mode == NEW_GAME_MODE {
            self.bring_up_banner();
            self.dump_screen_on(&just_rooms_rect);
        } else if mode == RESUME_GAME_MODE {
            self.display_stars_remaining();
            self.dump_screen_on(&just_rooms_rect);
        } else {
            self.dump_screen_on(&just_rooms_rect);
        }

        self.init_garbage_rects();
        self.start_glider_fading_in(0);
        if self.two_player_game {
            self.start_glider_fading_in(1);
            self.tag_glider_idle(1);
            self.gliders[1].dont_draw = true;
        }
        self.init_telephone();
        self.was_play_music_pref = self.is_play_music_game;

        // (COMPILEQT: the TV's movie; not kept)

        self.playing = true; // everything before this line is game set-up
        // PlayGame() follows: play_game_frame, then play_game_finish.
    }

    /// The rest of NewGame, after `PlayGame()` returns: everything
    /// following is after a game has ended.
    pub fn play_game_finish(&mut self) {
        // (the room's text, a calendar's month, goes with the room)
        self.labels.clear();
        // (the end of PlayGame: BUILD_ARCADE_VERSION only)

        self.is_play_music_game = self.was_play_music_pref;
        self.zero_mirror_region();

        // (COMPILEQT: the TV's movie; not kept)

        self.two_player_game = false;
        self.the_mode = SPLASH_MODE;
        // InitCursor: none.
        if self.is_play_music_idle {
            if !self.is_music_on {
                let the_err = self.start_music();
                if the_err != 0 {
                    log::warn!("YellowAlert kYellowNoMusic {the_err}");
                    self.failed_music = true;
                }
            }
            self.set_musical_mode(PLAY_WHOLE_SCORE_MODE);
        } else {
            if self.is_music_on {
                self.stop_the_music();
            }
        }
        self.nil_saved_maps();
        self.blacken_scoreboard();
        // UpdateMenus(false): no menus.

        if !self.game_over {
            // InvalWindowRect(mainWindow, &mainWindowRect): the terminal
            // redraws the main window anyway.
            let r = self.work_src_rect;
            self.work_src_map.fill_rect(r, gfx::BLACK);
            let mut temp_rect = Rect::default();
            q_set_rect(&mut temp_rect, 0, 0, 640, 460);
            q_offset_rect(&mut temp_rect, self.splash_origin_h, self.splash_origin_v);
            if let Some(img) = self.picture(SPLASH8_BIT_PICT) {
                self.work_src_map.draw_picture(&img, temp_rect);
            }
        }
        // WaitCommandQReleased: none.
        self.demo_going = false;
        self.increment_mode_time = self.ticks + IDLE_SPLASH_TICKS;
    }

    //--------------------------------------------------------------  DoDemoGame

    // DoDemoGame (close the house, open the demo house by index, NewGame,
    // reopen the house) is the terminal's: it loads the demo house, sets
    // `demo_going`, and calls `new_game(NEW_GAME_MODE)`.

    //--------------------------------------------------------------  InitGlider

    pub fn init_glider(&mut self, g: usize, mode: i32) {
        let mut dest = self.gliders[g].dest;
        self.where_does_glider_begin(&mut dest, mode);
        self.gliders[g].dest = dest;

        if mode == RESUME_GAME_MODE {
            // numStarsRemaining = smallGame.wasStarsLeft: saved games are cut.
        } else if mode == NEW_GAME_MODE {
            self.num_stars_remaining = self.count_stars_in_house();
        }

        if mode == RESUME_GAME_MODE {
            // (saved games are cut: smallGame's score, gliders, energy,
            // bands, foil, glider state and facing are not restored)
            match self.gliders[g].mode {
                GLIDER_BURNING => self.flag_glider_burning(g),

                _ => self.flag_glider_normal(g),
            }
        } else {
            self.the_score = 0;
            self.mortals = INITIAL_GLIDERS;
            if self.two_player_game {
                self.mortals += INITIAL_GLIDERS;
            }
            self.battery_total = 0;
            self.bands_total = 0;
            self.foil_total = 0;
            self.gliders[g].mode = GLIDER_NORMAL;
            self.gliders[g].facing = FACE_RIGHT;
            self.gliders[g].src = self.glider_src[0];
            self.gliders[g].mask = self.glider_src[0];
            self.show_foil = false;
        }

        let gl = &mut self.gliders[g];
        q_set_rect(&mut gl.dest_shadow, 0, 0, GLIDER_WIDE, SHADOW_HIGH);
        q_offset_rect(&mut gl.dest_shadow, gl.dest.left, SHADOW_TOP);
        gl.whole_shadow = gl.dest_shadow;

        gl.h_vel = 0;
        gl.v_vel = 0;
        gl.h_desired_vel = 0;
        gl.v_desired_vel = 0;

        gl.tipped = false;
        gl.sliding = false;
        gl.dont_draw = false;
    }

    //--------------------------------------------------------------  SetHouseToFirstRoom

    pub fn set_house_to_first_room(&mut self) {
        let first_room = self.get_first_room_number();
        self.force_this_room(first_room);
    }

    //--------------------------------------------------------------  SetHouseToSavedRoom

    // SetHouseToSavedRoom: saved games are cut.

    //--------------------------------------------------------------  HandlePlayEvent

    // HandlePlayEvent: the terminal handles events (window updates,
    // suspend and resume).

    //--------------------------------------------------------------  PlayGame

    /// One pass of PlayGame's `while ((playing) && (!quitting))` loop. The
    /// terminal calls it once every kTicksPerFrame ticks.
    pub fn play_game_frame(&mut self) {
        if !self.playing || self.quitting {
            return;
        }

        self.game_frame += 1;
        self.even_frame = !self.even_frame;

        // if (doBackground) do HandlePlayEvent() while (switchedOut):
        // the terminal's.

        self.handle_telephone();

        if self.two_player_game {
            self.handle_dynamics();
            if !self.game_over {
                self.get_input(0);
                self.get_input(1);
                self.handle_interaction();
            }
            self.handle_triggers();
            self.handle_bands();
            if !self.game_over {
                self.handle_glider(0);
                self.handle_glider(1);
            }
            if self.playing {
                // (COMPILEQT: MoviesTask)
                self.render_frame();
                self.handle_dynamic_scoreboard();
            }
        } else {
            self.handle_dynamics();
            if !self.game_over {
                if self.demo_going {
                    self.get_demo_input(0);
                } else {
                    self.get_input(0);
                }
                self.handle_interaction();
            }
            self.handle_triggers();
            self.handle_bands();
            if !self.game_over {
                self.handle_glider(0);
            }
            if self.playing {
                // (COMPILEQT: MoviesTask)
                self.render_frame();
                self.handle_dynamic_scoreboard();
            }
        }

        if self.game_over {
            self.count_down -= 1;
            if self.count_down <= 0 {
                self.hide_glider(0);
                self.refresh_scoreboard(NORMAL_TITLE_MODE);

                // (BUILD_ARCADE_VERSION: paint over the scoreboard; not
                // this version)

                if self.mortals < 0 {
                    self.do_died_game_over();
                } else {
                    self.do_game_over();
                }
            }
        }
    }

    //--------------------------------------------------------------  SetObjectsToDefaults

    pub fn set_objects_to_defaults(&mut self) {
        let num_rooms = self.house.rooms.len();
        let is_play_music_game = self.is_play_music_game;

        for r in 0..num_rooms {
            self.house.rooms[r].visited = false;
            for i in 0..MAX_ROOM_OBS as usize {
                let obj = &mut self.house.rooms[r].objects[i];
                match obj.what {
                    FLOOR_VENT | CEILING_VENT | FLOOR_BLOWER | CEILING_BLOWER | LEFT_FAN
                    | RIGHT_FAN | SEWER_GRATE | INVIS_BLOWER | GRECO_VENT | SEWER_BLOWER
                    | LIFT_AREA => {
                        let mut a = obj.a();
                        a.state = a.initial;
                        obj.set_a(a);
                    }

                    RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | CUCKOO | PAPER | BATTERY | BANDS
                    | GREASE_RT | GREASE_LF | FOIL | INVIS_BONUS | STAR | SPARKLE | HELIUM => {
                        let mut c = obj.c();
                        c.state = c.initial;
                        obj.set_c(c);
                    }

                    DELUXE_TRANS => {
                        let mut d = obj.d();
                        let init_state = (d.wide & 0xF0) >> 4;
                        d.wide &= 0xF0;
                        d.wide += init_state;
                        obj.set_d(d);
                    }

                    CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP
                    | FLOURESCENT | TRACK_LIGHT | INVIS_LIGHT => {
                        let mut f = obj.f();
                        f.state = f.initial;
                        obj.set_f(f);
                    }

                    STEREO => {
                        let mut gg = obj.g();
                        gg.state = is_play_music_game;
                        obj.set_g(gg);
                    }

                    SHREDDER | TOASTER | MAC_PLUS | GUITAR | TV | COFFEE | OUTLET | VCR
                    | MICROWAVE => {
                        let mut gg = obj.g();
                        gg.state = gg.initial;
                        obj.set_g(gg);
                    }

                    BALLOON | COPTER_LF | COPTER_RT | DART_LF | DART_RT | BALL | DRIP | FISH => {
                        let mut h = obj.h();
                        h.state = h.initial;
                        obj.set_h(h);
                    }

                    _ => {}
                }
            }
        }
    }

    //--------------------------------------------------------------  HideGlider

    pub fn hide_glider(&mut self, g: usize) {
        let mut temp_rect = self.gliders[g].whole;
        q_offset_rect(&mut temp_rect, self.play_origin_h, self.play_origin_v);
        self.copy_rect_work_to_main(&temp_rect);

        if self.has_mirror {
            q_offset_rect(&mut temp_rect, -20, -16);
            self.copy_rect_work_to_main(&temp_rect);
        }

        temp_rect = self.gliders[g].whole_shadow;
        q_offset_rect(&mut temp_rect, self.play_origin_h, self.play_origin_v);
        self.copy_rect_work_to_main(&temp_rect);
    }

    //--------------------------------------------------------------  InitTelephone

    pub fn init_telephone(&mut self) {
        self.the_phone.next_ring = self.random_int(RING_SPREAD) + RING_BASE_DELAY;
        self.the_phone.rings = self.random_int(3) + 3;
        self.the_phone.delay = RING_DELAY;

        self.the_chimes.next_ring = self.random_int(CHIME_DELAY) + 1;
    }

    //--------------------------------------------------------------  HandleTelephone

    pub fn handle_telephone(&mut self) {
        if !self.phone_bit_set {
            if self.the_phone.next_ring == 0 {
                if self.the_phone.delay == 0 {
                    self.the_phone.delay = RING_DELAY;
                    self.play_priority_sound(PHONE_RING_SOUND, PHONE_RING_PRIORITY);
                    self.the_phone.rings -= 1;
                    if self.the_phone.rings == 0 {
                        self.the_phone.next_ring = self.random_int(RING_SPREAD) + RING_BASE_DELAY;
                        self.the_phone.rings = self.random_int(3) + 3;
                    }
                } else {
                    self.the_phone.delay -= 1;
                }
            } else {
                self.the_phone.next_ring -= 1;
            }
        }
        // handle also the wind chimes (if they are present)

        if self.num_chimes > 0 {
            if self.the_chimes.next_ring == 0 {
                if self.random_int(2) == 0 {
                    self.play_priority_sound(CHIME1_SOUND, CHIME1_PRIORITY);
                } else {
                    self.play_priority_sound(CHIME2_SOUND, CHIME2_PRIORITY);
                }

                let mut delay_time = CHIME_DELAY / self.num_chimes;
                if delay_time < 2 {
                    delay_time = 2;
                }

                self.the_chimes.next_ring = self.random_int(delay_time) + 1;
            } else {
                self.the_chimes.next_ring -= 1;
            }
        }
    }

    //--------------------------------------------------------------  StrikeChime

    pub fn strike_chime(&mut self) {
        self.the_chimes.next_ring = 0;
    }

    //--------------------------------------------------------------  RestoreEntireGameScreen

    pub fn restore_entire_game_screen(&mut self) {
        // HideCursor: none.
        let temp_rect = self.screen;
        self.main_window_map.fill_rect(temp_rect, gfx::BLACK);

        self.draw_locale();
        self.refresh_scoreboard(NORMAL_TITLE_MODE);
        //	if (quickerTransitions)
        //		DissBitsChunky(&justRoomsRect);
        //	else
        //		DissBits(&justRoomsRect);
    }
}

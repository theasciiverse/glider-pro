//! GameOver.c: the end of a game, either the house completed (falling
//! stars and the angel) or the last glider lost (pages blowing across the
//! screen, spelling GAME OVER).

use super::*;
use crate::gfx;
use crate::gfx::Region;

pub const NUM_COUNT_DOWN_FRAMES: i32 = 16;
pub const PAGE_FRAMES: i32 = 14;
pub const PAGES_PICT_ID: i32 = 1990;
pub const PAGES_MASK_ID: i32 = 1989;
pub const LETTERS_PICT_ID: i32 = 1988;
pub const MILKYWAY_PICT_ID: i32 = 1021;

pub const STAR_FALLS: i32 = 8;
pub const PAGE_SPACING: i32 = 40;
pub const PAGE_RIGHT_OFFSET: i32 = 128;
pub const PAGE_BACK_UP: i32 = 128;

impl Game {
    /// The C's busy-wait until TickCount() reaches the next frame, reading
    /// the keys and events: shows the screen, lets `n` ticks pass; true if
    /// a key or button was pressed (the C's modifier keys, mouseDown or
    /// keyDown).
    pub fn delay_ticks_input(&mut self, n: i32) -> bool {
        let mut hit = false;
        if let Some(mut io) = self.screen_io.take() {
            io.present(self);
            hit = io.wait(n, self);
            self.screen_io = Some(io);
        }
        self.ticks += n;
        hit
    }

    //--------------------------------------------------------------  DoGameOver

    // Handles a game over.  This is a game over where the player has
    // completed the house.

    pub fn do_game_over(&mut self) {
        let labels_were = self.labels.len();
        self.playing = false;
        self.set_up_final_screen();
        let r = self.main_window_rect;
        self.main_window_map.color_rect(r, 244);
        self.do_game_over_star_animation();
        self.labels.truncate(labels_were);
        if !self.test_high_score() {
            self.redraw_splash_screen();
        }
    }

    //--------------------------------------------------------------  SetUpFinalScreen

    // This sets up the game over screen (again, this function is for when
    // the player completes the house).

    pub fn set_up_final_screen(&mut self) {
        let work_src_rect = self.work_src_rect;
        self.work_src_map.color_rect(work_src_rect, 244);
        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, 0, 0, 640, 460);
        center_rect_in_rect(&mut temp_rect, &work_src_rect);
        if let Some(img) = self.picture(MILKYWAY_PICT_ID) {
            self.work_src_map.draw_picture(&img, temp_rect);
        }
        let mut text_down = temp_rect.top;
        if text_down < 0 {
            text_down = 0;
        }

        let temp_str = self.house.trailer.clone();

        // Terminal text: the C draws each line black, then white one pixel
        // up and left.
        let mut count = 0;
        loop {
            let sub_str = banner::get_line_of_text(&temp_str, count);
            let text = banner::line_text(&sub_str);
            // TextWidth in 12-point bold: about 7 pixels a character.
            let offset =
                ((self.screen.right - self.screen.left) - text.chars().count() as i32 * 7) / 2;
            if !text.is_empty() {
                self.label(offset, text_down + 32 + (count * 20), text, gfx::WHITE, 0);
            }
            count += 1;
            if sub_str.is_empty() {
                break;
            }
        }

        self.copy_rect_work_to_back(&work_src_rect);

        for i in 0..5usize {
            // initialize the falling stars
            self.pages[i].dest = self.star_src[0];
            let h = self.work_src_rect.right
                + self.random_int(self.work_src_rect.right / 5)
                + (self.work_src_rect.right / 4) * i as i32;
            let v = self.random_int(self.work_src_rect.bottom) - self.work_src_rect.bottom / 2;
            q_offset_rect(&mut self.pages[i].dest, h, v);
            self.pages[i].was = self.pages[i].dest;
            self.pages[i].frame = self.random_int(6);
        }
    }

    //--------------------------------------------------------------  DoGameOverStarAnimation

    // This handles the falling stars and the flying angel when a player
    // completes a house.

    pub fn do_game_over_star_animation(&mut self) {
        let mut angel_dest = self.angel_src_rect;
        q_offset_rect(&mut angel_dest, -96, 0);
        let mut no_interuption = true;
        let mut count = 0;
        let mut pass = 0;
        // FlushEvents: none.

        while no_interuption {
            if (angel_dest.left % 32) == 0 {
                // add a star
                self.play_priority_sound(MYSTIC_SOUND, MYSTIC_PRIORITY);
                let mut which = angel_dest.left / 32;
                which %= 5;
                let w = which as usize;
                zero_rect_corner(&mut self.pages[w].dest);
                q_offset_rect(&mut self.pages[w].dest, angel_dest.left, angel_dest.bottom);
                if count < (which + 1) {
                    count = which + 1;
                }
            }

            for i in 0..count as usize {
                self.pages[i].frame += 1;
                if self.pages[i].frame >= 6 {
                    self.pages[i].frame = 0;
                }

                let sr = self.star_src[self.pages[i].frame as usize];
                self.work_src_map.copy_mask(
                    &self.bonus_src_map,
                    &self.bonus_mask_map,
                    sr,
                    sr,
                    self.pages[i].dest,
                );

                self.pages[i].was = self.pages[i].dest;
                self.pages[i].was.top -= STAR_FALLS;

                let was = self.pages[i].was;
                let dest = self.pages[i].dest;
                self.add_rect_to_work_rects_whole(&was);
                self.add_rect_to_back_rects(&dest);

                if self.pages[i].dest.top < self.work_src_rect.bottom {
                    q_offset_rect(&mut self.pages[i].dest, 0, STAR_FALLS);
                }
            }

            if angel_dest.left <= (self.work_src_rect.right + 2) {
                let ar = self.angel_src_rect;
                self.work_src_map.copy_mask(
                    &self.angel_src_map,
                    &self.angel_mask_map,
                    ar,
                    ar,
                    angel_dest,
                );
                angel_dest.left -= 2;
                self.add_rect_to_work_rects_whole(&angel_dest);
                angel_dest.left += 2;
                self.add_rect_to_back_rects(&angel_dest);
                q_offset_rect(&mut angel_dest, 2, 0);
                pass = 0;
            }

            self.copy_rects_qd();

            self.num_work2_main = 0;
            self.num_back2_work = 0;

            if self.delay_ticks_input(2) {
                no_interuption = false;
            }

            if pass < 80 {
                pass += 1;
            } else {
                self.wait_for_input_event(5);
                no_interuption = false;
            }
        }
    }

    //--------------------------------------------------------------  FlagGameOver

    // Called to indicate (flag) that a game is over.  Actual game over
    // sequence comes up after a short delay.

    pub fn flag_game_over(&mut self) {
        self.game_over = true;
        self.count_down = NUM_COUNT_DOWN_FRAMES;
        self.set_musical_mode(PLAY_WHOLE_SCORE_MODE);
    }

    //--------------------------------------------------------------  InitDiedGameOver
    // This is called when a game is over due to the fact that the player
    // lost their last glider (died), not due to getting through the entire
    // house.  This function initializes the strucures/variables.

    pub fn init_died_game_over(&mut self) {
        q_set_rect(&mut self.page_src_rect, 0, 0, 25, 32 * 8);
        self.game_over_src_map = self.load_map(LETTERS_PICT_ID, self.page_src_rect);

        q_set_rect(&mut self.page_src_rect, 0, 0, 32, 32 * PAGE_FRAMES);
        self.page_src_map = self.load_map(PAGES_PICT_ID, self.page_src_rect);

        self.page_mask_map = self.load_map(PAGES_MASK_ID, self.page_src_rect);

        for i in 0..PAGE_FRAMES as usize {
            // initialize src page rects
            q_set_rect(&mut self.page_src[i], 0, 0, 32, 32);
            q_offset_rect(&mut self.page_src[i], 0, 32 * i as i32);
        }

        let screen = self.screen;
        for i in 0..8usize {
            // initialize dest page rects
            let d = &mut self.pages[i].dest;
            q_set_rect(d, 0, 0, 32, 32);
            center_rect_in_rect(d, &screen);
            q_offset_rect(d, -screen.left, -screen.top);
            if i < 4 {
                q_offset_rect(d, -PAGE_SPACING * (4 - i as i32), 0);
            } else {
                q_offset_rect(d, PAGE_SPACING * (i as i32 - 3), 0);
            }
            q_offset_rect(
                d,
                (screen.right - screen.left) / -2,
                (screen.right - screen.left) / -2,
            );
            if d.left % 2 == 1 {
                q_offset_rect(d, 1, 0);
            }
            self.pages[i].was = self.pages[i].dest;
            self.pages[i].frame = 0;
            self.pages[i].counter = self.random_int(32);
            self.pages[i].stuck = false;
        }

        for i in 0..8usize {
            q_set_rect(&mut self.letters_src[i], 0, 0, 25, 32);
            q_offset_rect(&mut self.letters_src[i], 0, 32 * i as i32);
        }

        self.room_rgn = Some(Region::rect(self.just_rooms_rect));
        self.pages_stuck = 0;
        self.stop_pages = ((self.screen.bottom - self.screen.top) / 2) - 16;
    }

    //--------------------------------------------------------------  HandlePages

    // This handles the pieces of paper that blow across the screen.

    pub fn handle_pages(&mut self) {
        for i in 0..8usize {
            if (self.pages[i].dest.bottom + self.random_int(8)) > self.stop_pages {
                self.pages[i].frame = 0;
                if !self.pages[i].stuck {
                    self.pages[i].dest.right = self.pages[i].dest.left + 25;
                    self.pages[i].stuck = true;
                    self.pages_stuck += 1;
                }
            } else {
                if self.pages[i].frame == 0 {
                    self.pages[i].counter -= 1;
                    if self.pages[i].counter <= 0 {
                        self.pages[i].frame = 1;
                    }
                } else if self.pages[i].frame == 7 {
                    self.pages[i].counter -= 1;
                    if self.pages[i].counter <= 0 {
                        self.pages[i].frame = 8;
                        if self.random_int(2) == 0 {
                            self.play_priority_sound(PAPER3_SOUND, PAPERS_PRIORITY);
                        } else {
                            self.play_priority_sound(PAPER4_SOUND, PAPERS_PRIORITY);
                        }
                    } else {
                        q_offset_rect(&mut self.pages[i].dest, 10, 10);
                    }
                } else {
                    self.pages[i].frame += 1;
                    match self.pages[i].frame {
                        5 => q_offset_rect(&mut self.pages[i].dest, 6, 6),

                        6 => q_offset_rect(&mut self.pages[i].dest, 8, 8),

                        7 => {
                            q_offset_rect(&mut self.pages[i].dest, 8, 8);
                            self.pages[i].counter = self.random_int(4) + 4;
                        }

                        8 | 9 => q_offset_rect(&mut self.pages[i].dest, 8, 8),

                        10 => q_offset_rect(&mut self.pages[i].dest, 6, 6),

                        PAGE_FRAMES => {
                            q_offset_rect(&mut self.pages[i].dest, 8, 0);
                            self.pages[i].frame = 0;
                            self.pages[i].counter = self.random_int(8) + 8;
                            if self.random_int(2) == 0 {
                                self.play_priority_sound(PAPER1_SOUND, PAPERS_PRIORITY);
                            } else {
                                self.play_priority_sound(PAPER2_SOUND, PAPERS_PRIORITY);
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  DrawPages

    // This function does the drawing for the pieces of paper that blow
    // across the screen.

    pub fn draw_pages(&mut self) {
        for i in 0..8usize {
            if self.pages[i].stuck {
                self.work_src_map.copy_bits_rgn(
                    &self.game_over_src_map,
                    self.letters_src[i],
                    self.pages[i].dest,
                    self.room_rgn.as_ref(),
                );
            } else {
                let sr = self.page_src[self.pages[i].frame as usize];
                self.work_src_map.copy_mask(
                    &self.page_src_map,
                    &self.page_mask_map,
                    sr,
                    sr,
                    self.pages[i].dest,
                );
            }

            let dest = self.pages[i].dest;
            let mut was = self.pages[i].was;
            q_union_similar_rect(&dest, &was.clone(), &mut was);
            self.pages[i].was = was;
            self.add_rect_to_work_rects(&was);
            self.add_rect_to_back_rects(&dest);

            self.copy_rects_qd();

            self.num_work2_main = 0;
            self.num_back2_work = 0;

            self.pages[i].was = self.pages[i].dest;
        }
    }

    //--------------------------------------------------------------  DoDiedGameOver

    // This is called when a game is over due to the fact that the player
    // lost their last glider (died), not due to getting through the entire
    // house.

    pub fn do_died_game_over(&mut self) {
        let mut user_aborted = false;
        self.init_died_game_over();
        let work_src_rect = self.work_src_rect;
        self.copy_rect_main_to_work(&work_src_rect);
        self.copy_rect_main_to_back(&work_src_rect);
        // FlushEvents: none.

        while self.pages_stuck < 8 {
            self.handle_pages();
            self.draw_pages();
            if self.delay_ticks_input(2) {
                self.pages_stuck = 8;
                user_aborted = true;
            }
        }

        self.room_rgn = None;

        self.page_src_map = Map::default();

        self.page_mask_map = Map::default();

        self.game_over_src_map = Map::default();
        self.playing = false;

        if self.demo_going {
            if !user_aborted {
                self.wait_for_input_event(1);
            }
        } else {
            if !user_aborted {
                self.wait_for_input_event(10);
            }
            self.test_high_score();
        }
        self.redraw_splash_screen();
    }

    //--------------------------------------------------------------  RedrawSplashScreen

    /// MainWindow.c's RedrawSplashScreen (a file no group translates;
    /// game over and the high scores end with it). The C's DissBits is
    /// commented out there, so the splash drawn into the work map never
    /// reached the screen (and CopyRectMainToWork then drew over it); it is
    /// shown here, as meant. DrawOnSplash (the house's name) is the
    /// terminal's.
    pub fn redraw_splash_screen(&mut self) {
        let work_src_rect = self.work_src_rect;
        self.work_src_map.fill_rect(work_src_rect, gfx::BLACK);
        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, 0, 0, 640, 460);
        q_offset_rect(&mut temp_rect, self.splash_origin_h, self.splash_origin_v);
        if let Some(img) = self.picture(SPLASH8_BIT_PICT) {
            self.work_src_map.draw_picture(&img, temp_rect);
        }
        //	if (quickerTransitions)
        //		DissBitsChunky(&workSrcRect);
        //	else
        //		DissBits(&workSrcRect);
        self.dump_screen_on(&work_src_rect);
        self.copy_rect_main_to_work(&work_src_rect);
    }
}

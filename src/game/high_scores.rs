//! HighScores.c: the house's high score table, testing a finished game's
//! score against it, and showing it.
//!
//! The Mac dialogs for the player's name and banner are cut: the terminal
//! asks and leaves the answers in `pending_name` / `pending_banner`.
//! The scores file in the Preferences folder is not written yet
//! (`write_scores_to_disk`).

use super::*;
use crate::gfx;

pub const HIGH_SCORES_PICT_ID: i32 = 1994;
pub const HIGH_SCORES_MASK_ID: i32 = 1998;
pub const HIGH_NAME_DIALOG_ID: i32 = 1020;
pub const HIGH_BANNER_DIALOG_ID: i32 = 1021;

pub const SCORE_SPACING: i32 = 18;
pub const SCORE_WIDE: i32 = 352;
pub const KIMS_LIFTED: i32 = 4;

/// The name and banner Glider PRO starts with (Main.c, no prefs yet).
pub const DEFAULT_HIGH_NAME: &str = "Your Name";
pub const DEFAULT_HIGH_BANNER: &str = "Your Message Here";

/// StringWidth, roughly: the terminal draws the text; this only places it
/// as the C centred it (bold applFont about 7 pixels a character at 12
/// points, 8 at 14).
fn approx_string_width(s: &str, size: i32) -> i32 {
    s.chars().count() as i32 * if size >= 14 { 8 } else { 7 }
}

/// PasStringCopyNum: at most `n` characters.
fn truncated(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

impl Game {
    //--------------------------------------------------------------  DoHighScores
    // Handles fading in and cleaning up the high scores screen.

    pub fn do_high_scores(&mut self) {
        let labels_were = self.labels.len();
        let work_src_rect = self.work_src_rect;
        self.work_src_map.fill_rect(work_src_rect, gfx::BLACK);
        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, 0, 0, 640, 480);
        q_offset_rect(&mut temp_rect, self.splash_origin_h, self.splash_origin_v);
        if let Some(img) = self.picture(STAR_PICT_ID) {
            self.work_src_map.draw_picture(&img, temp_rect);
        }
        //	if (quickerTransitions)
        //		DissBitsChunky(&workSrcRect);
        //	else
        //		DissBits(&workSrcRect);
        self.draw_high_scores();
        //	if (quickerTransitions)
        //		DissBitsChunky(&workSrcRect);
        //	else
        //		DissBits(&workSrcRect);
        // (with the DissBits commented out the scores never reached the
        // screen; shown here, as meant)
        self.dump_screen_on(&work_src_rect);
        self.delay_ticks(60);
        self.wait_for_input_event(30);
        self.labels.truncate(labels_were);

        self.redraw_splash_screen();
    }

    //--------------------------------------------------------------  DrawHighScores
    // Draws the actual scores on the screen. (The texts are labels, at the
    // C's pen positions in work map coordinates, in the colour drawn on
    // top; the black shadows under them are left out.)

    pub fn draw_high_scores(&mut self) {
        let score_left = ((self.screen.right - self.screen.left) - SCORE_WIDE) / 2;
        let drop_it = 129 + self.splash_origin_v;

        let mut temp_rect = Rect::default();
        q_set_rect(&mut temp_rect, 0, 0, 332, 30);
        let temp_map = self.load_map(HIGH_SCORES_PICT_ID, temp_rect);

        let temp_mask = self.load_map(HIGH_SCORES_MASK_ID, temp_rect);

        let mut temp_rect2 = temp_rect;
        q_offset_rect(
            &mut temp_rect2,
            score_left + (SCORE_WIDE - 332) / 2,
            drop_it - 60,
        );

        self.work_src_map
            .copy_mask(&temp_map, &temp_mask, temp_rect, temp_rect, temp_rect2);

        // TextSize(14)
        let temp_str = format!("\u{2022} {} \u{2022}", self.this_house_name);
        let x = score_left + ((SCORE_WIDE - approx_string_width(&temp_str, 14)) / 2);
        self.label(x, drop_it - 65, temp_str, gfx::CYAN, 0);

        // TextSize(12)
        // message for score #1
        let temp_str = self.high_scores.banner.clone();
        let banner_width = approx_string_width(&temp_str, 12);
        self.label(
            score_left + (SCORE_WIDE - banner_width) / 2,
            drop_it - KIMS_LIFTED - 1,
            temp_str,
            gfx::YELLOW,
            0,
        );

        q_set_rect(&mut temp_rect, 0, 0, banner_width + 8, SCORE_SPACING);
        q_offset_rect(
            &mut temp_rect,
            score_left - 3 + (SCORE_WIDE - banner_width) / 2,
            drop_it + 5 - SCORE_SPACING - KIMS_LIFTED,
        );
        self.work_src_map.fore(gfx::BLACK);
        self.work_src_map.frame_rect(temp_rect);
        q_offset_rect(&mut temp_rect, -1, -1);
        self.work_src_map.fore(gfx::YELLOW);
        self.work_src_map.frame_rect(temp_rect);
        self.work_src_map.fore(gfx::BLACK);

        for i in 0..MAX_SCORES {
            let iu = i as usize;
            if self.high_scores.scores[iu] > 0 {
                // the baseline of the top-drawn text of row i
                let y = if i == 0 {
                    drop_it - 1 - SCORE_SPACING - KIMS_LIFTED
                } else {
                    drop_it - 1 + (i * SCORE_SPACING)
                };
                let last = i == self.last_high_score;

                // draw placing number
                let temp_str = (i + 1).to_string();
                self.label(
                    score_left,
                    y,
                    temp_str,
                    if last { gfx::WHITE } else { gfx::CYAN },
                    0,
                );
                // draw high score name
                let temp_str = self.high_scores.names[iu].clone();
                self.label(
                    score_left + 30,
                    y,
                    temp_str,
                    if last { gfx::WHITE } else { gfx::YELLOW },
                    0,
                );
                // draw level number
                let temp_str = self.high_scores.levels[iu].to_string();
                self.label(
                    score_left + 160,
                    y,
                    temp_str,
                    if last { gfx::WHITE } else { gfx::YELLOW },
                    0,
                );
                // draw word "rooms"
                let temp_str = if self.high_scores.levels[iu] == 1 {
                    banner::get_localized_string(6)
                } else {
                    banner::get_localized_string(7)
                };
                self.label(score_left + 192, y, temp_str, gfx::CYAN, 0);
                // draw high score points
                let temp_str = self.high_scores.scores[iu].to_string();
                self.label(
                    score_left + 290,
                    y,
                    temp_str,
                    if last { gfx::WHITE } else { gfx::YELLOW },
                    0,
                );
            }
        }

        // TextSize(9)
        self.label(
            score_left + 80,
            drop_it - 1 + (10 * SCORE_SPACING),
            banner::get_localized_string(8),
            gfx::BLUE,
            0,
        );
    }

    //--------------------------------------------------------------  SortHighScores
    // This does a simple sort of the high scores.

    pub fn sort_high_scores(&mut self) {
        let mut temp_scores = Scores::default();

        for h in 0..MAX_SCORES as usize {
            let mut greatest = -1;
            let mut which = -1;
            for i in 0..MAX_SCORES as usize {
                if self.high_scores.scores[i] > greatest {
                    greatest = self.high_scores.scores[i];
                    which = i as i32;
                }
            }
            if which != -1 {
                let w = which as usize;
                temp_scores.names[h] = self.high_scores.names[w].clone();
                temp_scores.scores[h] = self.high_scores.scores[w];
                temp_scores.time_stamps[h] = self.high_scores.time_stamps[w];
                temp_scores.levels[h] = self.high_scores.levels[w];
                self.high_scores.scores[w] = -1;
            }
        }
        temp_scores.banner = self.high_scores.banner.clone();
        self.high_scores = temp_scores;
    }

    //--------------------------------------------------------------  ZeroHighScores
    // This funciton goes through and resets or "zeros" all high scores.

    pub fn zero_high_scores(&mut self) {
        self.high_scores.banner = self.this_house_name.clone();
        for i in 0..MAX_SCORES as usize {
            self.high_scores.names[i] = "--------------".to_string();
            self.high_scores.scores[i] = 0;
            self.high_scores.time_stamps[i] = 0;
            self.high_scores.levels[i] = 0;
        }
    }

    //--------------------------------------------------------------  ZeroAllButHighestScore
    // Like the above, but this function preserves the highest score.

    pub fn zero_all_but_highest_score(&mut self) {
        for i in 1..MAX_SCORES as usize {
            self.high_scores.names[i] = "--------------".to_string();
            self.high_scores.scores[i] = 0;
            self.high_scores.time_stamps[i] = 0;
            self.high_scores.levels[i] = 0;
        }
    }

    //--------------------------------------------------------------  TestHighScore
    // This function is called after a game ends in order to test the
    // current high score against the high score list.  It returns true
    // if the player is on the high score list now.

    pub fn test_high_score(&mut self) -> bool {
        if self.resumed_saved_game {
            return false;
        }

        self.last_high_score = -1;
        let mut placing = -1;

        for i in 0..MAX_SCORES {
            if self.the_score > self.high_scores.scores[i as usize] {
                placing = i;
                self.last_high_score = i;
                break;
            }
        }

        if placing != -1 {
            // FlushEvents: none.
            self.get_high_score_name(placing + 1);
            let last = (MAX_SCORES - 1) as usize;
            self.high_scores.names[last] = self.high_name.clone();
            if placing == 0 {
                self.get_high_score_banner();
                self.high_scores.banner = self.high_banner.clone();
            }
            self.high_scores.scores[last] = self.the_score;
            self.high_scores.time_stamps[last] = utilities::get_date_time();
            self.high_scores.levels[last] = self.count_rooms_visited();
            self.sort_high_scores();
            self.game_dirty = true;
        }

        if placing != -1 {
            self.do_high_scores();
            true
        } else {
            false
        }
    }

    //--------------------------------------------------------------  GetHighScoreName
    // Brings up a dialog to get player's name (due to a high score).
    // (The dialog is the terminal's: it leaves the name in `pending_name`;
    // without one the name stays as last entered, "Your Name" at first.)

    pub fn get_high_score_name(&mut self, place: i32) {
        let _ = place; // (the dialog's "you placed #n" text)
        self.play_priority_sound(ENERGIZE_SOUND, ENERGIZE_PRIORITY);
        if let Some(name) = self.pending_name.take() {
            self.high_name = truncated(&name, 15);
        }
        if self.high_name.is_empty() {
            self.high_name = DEFAULT_HIGH_NAME.to_string();
        }
    }

    //--------------------------------------------------------------  GetHighScoreBanner
    // A player who gets the #1 slot gets to enter a short message (that
    // appears across the top of the high scores list).  This dialog
    // gets that message. (The terminal's, by `pending_banner`.)

    pub fn get_high_score_banner(&mut self) {
        self.play_priority_sound(ENERGIZE_SOUND, ENERGIZE_PRIORITY);
        if let Some(banner) = self.pending_banner.take() {
            self.high_banner = truncated(&banner, 31);
        }
        if self.high_banner.is_empty() {
            self.high_banner = DEFAULT_HIGH_BANNER.to_string();
        }
    }

    //--------------------------------------------------------------  WriteScoresToDisk

    /// Not yet: the C writes the table to "G-PRO Scores" in the Preferences
    /// folder, a file named after the house.
    pub fn write_scores_to_disk(&mut self) {
        log::info!(
            "high scores for {}: not written to disk (not done yet)",
            self.this_house_name
        );
    }

    //--------------------------------------------------------------  ReadScoresFromDisk

    /// Not yet (see `write_scores_to_disk`).
    pub fn read_scores_from_disk(&mut self) -> bool {
        log::info!(
            "high scores for {}: not read from disk (not done yet)",
            self.this_house_name
        );
        false
    }
}

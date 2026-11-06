//! Scoreboard.c: the strip above the rooms with the room's name, the
//! gliders left, the score and the badges (foil, bands, battery, helium).
//!
//! The C draws the three texts into small maps with DrawString; here those
//! maps get their gray ground only, and the texts go to `board_title`,
//! `board_gliders` and `board_score` for the terminal to show.

use super::*;
use crate::gfx;

pub const GRAY_BACKGROUND_COLOR: i32 = 251;
pub const GRAY_BACKGROUND_COLOR4: i32 = 10;
pub const FOIL_BADGE: usize = 0;
pub const BANDS_BADGE: usize = 1;
pub const BATTERY_BADGE: usize = 2;
pub const HELIUM_BADGE: usize = 3;
pub const SCORE_ROLL_AMOUNT: i32 = 13;

pub const FOIL_LOW: i32 = 2; // 25%
pub const BATTERY_LOW: i32 = 17; // 25%
pub const HELIUM_LOW: i32 = -38; // 25%
pub const BANDS_LOW: i32 = 2; // 25%

impl Game {
    //--------------------------------------------------------------  RefreshScoreboard

    pub fn refresh_scoreboard(&mut self, mode: i32) {
        self.do_roll_score = true;

        self.refresh_room_title(mode);
        self.refresh_num_gliders();
        self.refresh_points();

        self.main_window_map.copy_bits(
            &self.board_src_map,
            self.board_src_rect,
            self.board_dest_rect,
        );

        self.quick_battery_refresh(false);
        self.quick_bands_refresh(false);
        self.quick_foil_refresh(false);
    }

    //--------------------------------------------------------------  HandleDynamicScoreboard

    #[allow(clippy::if_same_then_else)]
    pub fn handle_dynamic_scoreboard(&mut self) {
        if self.the_score > self.displayed_score {
            if self.do_roll_score {
                self.displayed_score += SCORE_ROLL_AMOUNT;
                if self.displayed_score > self.the_score {
                    self.displayed_score = self.the_score;
                }
            } else {
                self.displayed_score = self.the_score;
            }

            self.play_priority_sound(SCORE_TIK_SOUND, SCORE_TIK_PRIORITY);
            self.quick_score_refresh();
        }

        let whos_turn = self.game_frame & 0x00000007;
        match whos_turn {
            0 => {
                // show foil
                if (self.foil_total > 0) && (self.foil_total < FOIL_LOW) {
                    self.quick_foil_refresh(false);
                }
            }

            1 => {
                // hide battery
                if (self.battery_total > 0) && (self.battery_total < BATTERY_LOW) {
                    self.quick_battery_refresh(true);
                } else if (self.battery_total < 0) && (self.battery_total > HELIUM_LOW) {
                    self.quick_battery_refresh(true);
                }
            }

            2 => {
                // show rubber bands
                if (self.bands_total > 0) && (self.bands_total < BANDS_LOW) {
                    self.quick_bands_refresh(false);
                }
            }

            4 => {
                // show battery
                if (self.battery_total > 0) && (self.battery_total < BATTERY_LOW) {
                    self.quick_battery_refresh(false);
                } else if (self.battery_total < 0) && (self.battery_total > HELIUM_LOW) {
                    self.quick_battery_refresh(false);
                }
            }

            5 => {
                // hide foil
                if (self.foil_total > 0) && (self.foil_total < FOIL_LOW) {
                    self.quick_foil_refresh(true);
                }
            }

            7 => {
                // hide rubber bands
                if (self.bands_total > 0) && (self.bands_total < BANDS_LOW) {
                    self.quick_bands_refresh(true);
                }
            }

            _ => {}
        }
    }

    //--------------------------------------------------------------  RefreshRoomTitle

    pub fn refresh_room_title(&mut self, mode: i32) {
        let r = self.board_t_src_rect;
        let was = self.board_t_src_map.pen;
        self.board_t_src_map.fore_index(GRAY_BACKGROUND_COLOR);
        self.board_t_src_map.paint_rect(r);
        self.board_t_src_map.pen = was;

        // DrawString at (1, 10) in black, then at (0, 9) in white.
        self.board_title = match mode {
            ESCAPED_TITLE_MODE => "Hit Delete key if unable to Follow".to_string(),

            SAVING_TITLE_MODE => "Saving Game\u{2026}".to_string(),

            _ => crate::mac::mac_roman(&self.this_room.name),
        };
        self.board_t_src_map.fore(gfx::BLACK);

        self.board_src_map.copy_bits(
            &self.board_t_src_map,
            self.board_t_src_rect,
            self.board_t_dest_rect,
        );
    }

    //--------------------------------------------------------------  RefreshNumGliders

    pub fn refresh_num_gliders(&mut self) {
        let r = self.board_g_src_rect;
        let was = self.board_g_src_map.pen;
        self.board_g_src_map.fore_index(GRAY_BACKGROUND_COLOR);
        self.board_g_src_map.paint_rect(r);
        self.board_g_src_map.pen = was;

        let mut display_mortals = self.mortals;
        if display_mortals < 0 {
            display_mortals = 0;
        }
        self.board_gliders = display_mortals.to_string();

        self.board_g_src_map.fore(gfx::BLACK);

        self.board_src_map.copy_bits(
            &self.board_g_src_map,
            self.board_g_src_rect,
            self.board_g_dest_rect,
        );
    }

    //--------------------------------------------------------------  RefreshPoints

    pub fn refresh_points(&mut self) {
        let r = self.board_p_src_rect;
        let was = self.board_p_src_map.pen;
        self.board_p_src_map.fore_index(GRAY_BACKGROUND_COLOR);
        self.board_p_src_map.paint_rect(r);
        self.board_p_src_map.pen = was;

        self.board_score = self.the_score.to_string();

        self.board_p_src_map.fore(gfx::BLACK);

        self.board_src_map.copy_bits(
            &self.board_p_src_map,
            self.board_p_src_rect,
            self.board_p_dest_rect,
        );

        self.displayed_score = self.the_score;
    }

    //--------------------------------------------------------------  QuickGlidersRefresh

    pub fn quick_gliders_refresh(&mut self) {
        let r = self.board_g_src_rect;
        let was = self.board_g_src_map.pen;
        self.board_g_src_map.fore_index(GRAY_BACKGROUND_COLOR);
        self.board_g_src_map.paint_rect(r);
        self.board_g_src_map.pen = was;

        // (unlike RefreshNumGliders, a negative count shows as it is)
        self.board_gliders = self.mortals.to_string();

        self.board_g_src_map.fore(gfx::BLACK);

        self.main_window_map.copy_bits(
            &self.board_g_src_map,
            self.board_g_src_rect,
            self.board_gq_dest_rect,
        );
    }

    //--------------------------------------------------------------  QuickScoreRefresh

    pub fn quick_score_refresh(&mut self) {
        let r = self.board_p_src_rect;
        let was = self.board_p_src_map.pen;
        self.board_p_src_map.fore_index(GRAY_BACKGROUND_COLOR);
        self.board_p_src_map.paint_rect(r);
        self.board_p_src_map.pen = was;

        self.board_score = self.displayed_score.to_string();

        self.board_p_src_map.fore(gfx::BLACK);

        self.main_window_map.copy_bits(
            &self.board_p_src_map,
            self.board_p_src_rect,
            self.board_pq_dest_rect,
        );
    }

    //--------------------------------------------------------------  QuickBatteryRefresh

    pub fn quick_battery_refresh(&mut self, flash: bool) {
        if (self.battery_total > 0) && !flash {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_badges_rects[BATTERY_BADGE],
                self.badges_dest_rects[BATTERY_BADGE],
            );
        } else if (self.battery_total < 0) && !flash {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_badges_rects[HELIUM_BADGE],
                self.badges_dest_rects[HELIUM_BADGE],
            );
        } else {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_blank_rects[BATTERY_BADGE],
                self.badges_dest_rects[BATTERY_BADGE],
            );
        }
    }

    //--------------------------------------------------------------  QuickBandsRefresh

    pub fn quick_bands_refresh(&mut self, flash: bool) {
        if (self.bands_total > 0) && !flash {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_badges_rects[BANDS_BADGE],
                self.badges_dest_rects[BANDS_BADGE],
            );
        } else {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_blank_rects[BANDS_BADGE],
                self.badges_dest_rects[BANDS_BADGE],
            );
        }
    }

    //--------------------------------------------------------------  QuickFoilRefresh

    pub fn quick_foil_refresh(&mut self, flash: bool) {
        if (self.foil_total > 0) && !flash {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_badges_rects[FOIL_BADGE],
                self.badges_dest_rects[FOIL_BADGE],
            );
        } else {
            self.main_window_map.copy_bits(
                &self.badge_src_map,
                self.badges_blank_rects[FOIL_BADGE],
                self.badges_dest_rects[FOIL_BADGE],
            );
        }
    }

    //--------------------------------------------------------------  AdjustScoreboardHeight

    pub fn adjust_scoreboard_height(&mut self) {
        let new_mode = if self.num_neighbors == 9 {
            SCOREBOARD_HIGH
        } else {
            SCOREBOARD_LOW
        };

        if self.was_scoreboard_mode != new_mode {
            let mut offset = 0;
            match new_mode {
                SCOREBOARD_HIGH => {
                    // 9 neighbors
                    offset = self.local_rooms_dest[CENTRAL_ROOM as usize].top;
                    offset = -offset;
                    self.just_rooms_rect = self.work_src_rect;
                }

                SCOREBOARD_LOW => {
                    // 1 or 3 neighbors
                    offset = self.local_rooms_dest[CENTRAL_ROOM as usize].top;
                    self.just_rooms_rect = self.work_src_rect;
                    self.just_rooms_rect.top = self.local_rooms_dest[CENTRAL_ROOM as usize].top;
                    self.just_rooms_rect.bottom =
                        self.local_rooms_dest[CENTRAL_ROOM as usize].bottom;
                }

                _ => {}
            }

            q_offset_rect(&mut self.board_dest_rect, 0, offset);
            q_offset_rect(&mut self.board_gq_dest_rect, 0, offset);
            q_offset_rect(&mut self.board_pq_dest_rect, 0, offset);
            q_offset_rect(&mut self.badges_dest_rects[BATTERY_BADGE], 0, offset);
            q_offset_rect(&mut self.badges_dest_rects[BANDS_BADGE], 0, offset);
            q_offset_rect(&mut self.badges_dest_rects[FOIL_BADGE], 0, offset);
            q_offset_rect(&mut self.badges_dest_rects[HELIUM_BADGE], 0, offset);

            self.was_scoreboard_mode = new_mode;
        }
    }

    //--------------------------------------------------------------  BlackenScoreboard

    pub fn blacken_scoreboard(&mut self) {
        // UpdateMenuBarWindow: covers the menu bar on a second monitor;
        // there is none here.
    }
}

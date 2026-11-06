//! Banner.c: the house's opening banner (a sheet of notebook paper) and the
//! "stars remaining" message.

use super::*;
use crate::gfx;

pub const BANNER_PAGE_TOP_PICT: i32 = 1993;
pub const BANNER_PAGE_BOTTOM_PICT: i32 = 1992;
pub const BANNER_PAGE_BOTTOM_MASK: i32 = 1991;
pub const STARS_REMAINING_PICT: i32 = 1017;
pub const STAR_REMAINING_PICT: i32 = 1018;

/// Glider PRO's 'STR#' 150 ("Localized Strings"), the entries play uses.
pub fn get_localized_string(index: i32) -> &'static str {
    match index {
        1 => "There are ",
        2 => "There is ",
        3 => " stars in the house.",
        4 => " star in the house.",
        5 => "Get every star to win.",
        6 => "room",
        7 => "rooms",
        8 => "Click Mouse or Hit a Key to Exit",
        9 => "Name for New House:",
        10 => "Untitled House",
        _ => "",
    }
}

/// StringUtils.c's GetLineOfText: line `index` of a text whose lines end
/// in returns (the line keeps its return, as in the C).
pub fn get_line_of_text(src_str: &[u8], index: i32) -> Vec<u8> {
    // The C's Pascal string is 1-based: srcStr[i] is src_str[i - 1].
    let at = |i: i32| -> u8 {
        if i >= 1 {
            src_str.get((i - 1) as usize).copied().unwrap_or(0)
        } else {
            0
        }
    };
    let src_length = src_str.len().min(255) as i32;
    let mut text_line = Vec::new();

    let start;
    if index == 0 {
        // walk through to "index"
        start = 1;
    } else {
        let mut s = 0;
        let mut count = 0;
        let mut i = 0;
        let mut found_it = false;
        loop {
            i += 1;
            if at(i) == 0x0D {
                count += 1;
                if count == index {
                    s = i + 1;
                    found_it = true;
                }
            }
            if i >= src_length || found_it {
                break;
            }
        }
        start = s;
    }

    if start != 0 {
        let mut start = start;
        let mut i = start;
        let mut stop = 0;

        let mut found_it = false;
        loop {
            if at(i) == 0x0D {
                stop = i;
                found_it = true;
            }
            i += 1;
            if i >= src_length || found_it {
                break;
            }
        }

        if !found_it {
            if start > src_length {
                start = src_length;
                stop = src_length - 1;
            } else {
                stop = i;
            }
        }

        // (stop may run one past the text, as in the C; nothing is there)
        for i in start..=stop.min(src_length) {
            text_line.push(at(i));
        }
    }
    text_line
}

/// A line of Mac text for the terminal: Mac Roman, without its return.
pub fn line_text(line: &[u8]) -> String {
    let line: Vec<u8> = line.iter().copied().filter(|&c| c != 0x0D).collect();
    crate::mac::mac_roman(&line)
}

impl Game {
    //--------------------------------------------------------------  DrawBanner
    // Displays opening banner (when a new game is begun).  The banner looks
    // like a sheet of notebook paper.  The text printed on it is specified
    // by the author of the house.

    pub fn draw_banner(&mut self, top_left: &mut Point) {
        let mut whole_page = Rect::default();
        q_set_rect(&mut whole_page, 0, 0, 330, 220);
        let mut map_bounds = self.screen;
        zero_rect_corner(&mut map_bounds);
        center_rect_in_rect(&mut whole_page, &map_bounds);
        top_left.h = whole_page.left;
        top_left.v = whole_page.top;
        let mut part_page = whole_page;
        part_page.bottom = part_page.top + 190;
        if let Some(img) = self.picture(BANNER_PAGE_TOP_PICT) {
            self.work_src_map.draw_picture(&img, part_page);
        } else {
            log::warn!("PICT {BANNER_PAGE_TOP_PICT} missing");
        }

        part_page = whole_page;
        part_page.top = part_page.bottom - 30;
        map_bounds = part_page;
        zero_rect_corner(&mut map_bounds);
        let temp_map = self.load_map(BANNER_PAGE_BOTTOM_PICT, map_bounds);

        let temp_mask = self.load_map(BANNER_PAGE_BOTTOM_MASK, map_bounds);

        self.work_src_map
            .copy_mask(&temp_map, &temp_mask, map_bounds, map_bounds, part_page);
    }

    //--------------------------------------------------------------  CountStarsInHouse
    // Goes through the current house and counts the total number of stars within.

    pub fn count_stars_in_house(&self) -> i32 {
        let mut num_stars = 0;

        let num_rooms = self.house.rooms.len();
        for i in 0..num_rooms {
            if self.house.rooms[i].suite != ROOM_IS_EMPTY {
                for h in 0..MAX_ROOM_OBS as usize {
                    if self.house.rooms[i].objects[h].what == STAR {
                        num_stars += 1;
                    }
                }
            }
        }

        num_stars
    }

    //--------------------------------------------------------------  DrawBannerMessage

    // This function prints the author's message acorss the notebook paper banner.
    // (Terminal text: labels at the C's pen positions, in work map coordinates.)

    pub fn draw_banner_message(&mut self, top_left: Point) {
        let banner_str = self.house.banner.clone();

        let mut count = 0;
        loop {
            let sub_str = get_line_of_text(&banner_str, count);
            let text = line_text(&sub_str);
            if !text.is_empty() {
                self.label(
                    top_left.h + 16,
                    top_left.v + 32 + (count * 20),
                    text,
                    gfx::BLACK,
                    0,
                );
            }
            count += 1;
            if sub_str.is_empty() {
                break;
            }
        }

        if self.banner_star_count_on {
            let mut banner_str = String::from(if self.num_stars_remaining != 1 {
                get_localized_string(1)
            } else {
                get_localized_string(2)
            });

            banner_str.push_str(&self.num_stars_remaining.to_string());

            if self.num_stars_remaining != 1 {
                banner_str.push_str(get_localized_string(3));
            } else {
                banner_str.push_str(get_localized_string(4));
            }

            self.label(top_left.h + 16, top_left.v + 164, banner_str, gfx::RED, 0);
            self.label(
                top_left.h + 16,
                top_left.v + 180,
                get_localized_string(5),
                gfx::RED,
                0,
            );
        }
    }

    //--------------------------------------------------------------  BringUpBanner
    // Handles bringing up displaying and disposing of the banner.

    pub fn bring_up_banner(&mut self) {
        let mut top_left = Point::default();
        let labels_were = self.labels.len();

        self.draw_banner(&mut top_left);
        self.draw_banner_message(top_left);
        //	if (quickerTransitions)
        //		DissBitsChunky(&justRoomsRect);		// was workSrcRect
        //	else
        //		DissBits(&justRoomsRect);
        // (The C has the DissBits commented out, so the banner never
        // reached the screen; it is shown here, as it was meant to be.)
        let just_rooms_rect = self.just_rooms_rect;
        self.dump_screen_on(&just_rooms_rect);
        let mut whole_page = Rect::default();
        q_set_rect(&mut whole_page, 0, 0, 330, 220);
        q_offset_rect(&mut whole_page, top_left.h, top_left.v);

        self.work_src_map
            .copy_bits(&self.back_src_map, whole_page, whole_page);

        if self.demo_going {
            self.wait_for_input_event(4);
        } else {
            self.wait_for_input_event(15);
        }
        self.labels.truncate(labels_were);

        //	if (quickerTransitions)
        //		DissBitsChunky(&justRoomsRect);
        //	else
        //		DissBits(&justRoomsRect);
    }

    //--------------------------------------------------------------  DisplayStarsRemaining
    // This brings up a small message indicating the number of stars remaining
    // in a house.  It comes up when the player gets a star (the game is paused
    // and the user informed as to how many remain).

    pub fn display_stars_remaining(&mut self) {
        let labels_were = self.labels.len();
        let mut bounds = Rect::default();
        q_set_rect(&mut bounds, 0, 0, 256, 64);
        let screen = self.screen;
        center_rect_in_rect(&mut bounds, &screen);
        q_offset_rect(&mut bounds, -screen.left, -screen.top);
        let mut src = bounds;
        inset_rect(&mut src, 64, 32);

        let the_str = self.num_stars_remaining.to_string();

        q_offset_rect(&mut bounds, 0, -20);
        if self.num_stars_remaining < 2 {
            if let Some(img) = self.picture(STAR_REMAINING_PICT) {
                self.main_window_map.draw_picture(&img, bounds);
            }
        } else {
            if let Some(img) = self.picture(STARS_REMAINING_PICT) {
                self.main_window_map.draw_picture(&img, bounds);
            }
            // MoveTo(bounds.left + 102 - (StringWidth(theStr) / 2), ...):
            // about 7 pixels a digit in 12-point bold.
            let x = bounds.left + 102 - (the_str.len() as i32 * 7) / 2;
            self.label(x, bounds.top + 23, the_str, gfx::clut8(4), 0);
        }

        self.delay_ticks(60);
        if self.wait_for_input_event(30) {
            self.restore_entire_game_screen();
        }
        self.labels.truncate(labels_were);
        self.copy_rect_work_to_main(&bounds);
    }
}

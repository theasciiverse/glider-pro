//! Transit.c: moving the glider(s) from room to room (walking out a side,
//! up or down, by transport, duct or mailbox), and following the leader in
//! a two-player game.

use super::*;

impl Game {
    //--------------------------------------------------------------  WhatAreWeLinkedTo

    pub fn what_are_we_linked_to(&self, where_: i32, who: i32) -> i32 {
        // (a transport linked to no room or object: the C reads whatever is
        // there and most likely takes the default)
        let what = self
            .house
            .rooms
            .get(where_ as usize)
            .and_then(|r| r.objects.get(who as usize))
            .map_or(0, |o| o.what);

        match what {
            MAILBOX_LF => LINKED_TO_LEFT_MAILBOX,

            MAILBOX_RT => LINKED_TO_RIGHT_MAILBOX,

            CEILING_TRANS => LINKED_TO_CEILING_DUCT,

            _ => LINKED_TO_OTHER,
        }
    }

    //--------------------------------------------------------------  ReadyGliderFromTransit

    pub fn ready_glider_from_transit(&mut self, g: usize, to_what: i32) {
        if self.two_player_game
            && self.one_player_left
            && (self.gliders[g].which == self.player_dead)
        {
            return;
        }

        self.flag_glider_normal(g);

        let trans_rect = self.trans_rect;
        match to_what {
            LINKED_TO_OTHER => {
                self.start_glider_transporting_in(g); // set glider's mode
                let gl = &mut self.gliders[g];
                let mut temp_rect = gl.dest; // position glider
                center_rect_in_rect(&mut temp_rect, &trans_rect);
                gl.dest.left = temp_rect.left;
                gl.dest.right = temp_rect.right;
                gl.dest.top = temp_rect.top;
                gl.dest.bottom = temp_rect.bottom;
                gl.dest_shadow.left = temp_rect.left;
                gl.dest_shadow.right = temp_rect.right;
                gl.whole = gl.dest;
                gl.whole_shadow = gl.dest_shadow;
                gl.entered_rect = gl.dest;
            }

            LINKED_TO_LEFT_MAILBOX => {
                self.start_glider_mailing_out(g);
                let gl = &mut self.gliders[g];
                gl.clip = trans_rect; // fix clip
                gl.clip.right -= 64;
                gl.clip.bottom -= 25;
                let temp_rect = gl.dest;
                gl.dest.left = gl.clip.right;
                gl.dest.right = gl.dest.left;
                gl.dest.bottom = gl.clip.bottom - 4;
                gl.dest.top = gl.dest.bottom - rect_tall(&temp_rect);
                gl.dest_shadow.left = gl.dest.left;
                gl.dest_shadow.right = gl.dest.right;
                gl.whole = gl.dest;
                gl.whole_shadow = gl.dest_shadow;
            }

            LINKED_TO_RIGHT_MAILBOX => {
                self.start_glider_mailing_out(g);
                let gl = &mut self.gliders[g];
                gl.clip = trans_rect; // fix clip
                gl.clip.left += 79;
                gl.clip.bottom -= 25;
                let temp_rect = gl.dest;
                gl.dest.right = gl.clip.left;
                gl.dest.left = gl.dest.right;
                gl.dest.bottom = gl.clip.bottom - 4;
                gl.dest.top = gl.dest.bottom - rect_tall(&temp_rect);
                gl.dest_shadow.left = gl.dest.left;
                gl.dest_shadow.right = gl.dest.right;
                gl.whole = gl.dest;
                gl.whole_shadow = gl.dest_shadow;
            }

            LINKED_TO_CEILING_DUCT => {
                self.start_glider_ducting_in(g);
                let gl = &mut self.gliders[g];
                let mut temp_rect = gl.dest; // set glider's position
                center_rect_in_rect(&mut temp_rect, &trans_rect);
                gl.dest.left = temp_rect.left;
                gl.dest.right = temp_rect.right;
                gl.dest.top = temp_rect.top;
                gl.dest.bottom = gl.dest.top;
                q_offset_rect(&mut gl.dest, 0, -rect_tall(&temp_rect));
                gl.dest_shadow.left = temp_rect.left;
                gl.dest_shadow.right = temp_rect.right;
                gl.whole = gl.dest;
                gl.whole_shadow = gl.dest_shadow;
            }

            LINKED_TO_FLOOR_DUCT => {}

            _ => {}
        }

        if self.two_player_game && (self.gliders[g].which != self.first_player) {
            self.tag_glider_idle(g);
        }
    }

    //--------------------------------------------------------------  MoveRoomToRoom

    pub fn move_room_to_room(&mut self, g: usize, where_: i32) {
        let mut enter_rect = Rect::default();

        self.handle_room_visitation();
        match where_ {
            TO_RIGHT => {
                self.set_musical_mode(PROD_GAME_SCORE_MODE);
                if self.two_player_game {
                    self.undo_glider_limbo(0);
                    self.undo_glider_limbo(1);
                    self.insure_glider_facing_right(0);
                    self.insure_glider_facing_right(1);
                } else {
                    self.insure_glider_facing_right(g);
                }
                let east = self.local_numbers[EAST_ROOM as usize];
                self.force_this_room(east);
                if self.two_player_game {
                    self.offset_glider(0, TO_LEFT);
                    self.offset_glider(1, TO_LEFT);
                    q_set_rect(&mut enter_rect, 0, 0, 48, 20);
                    q_offset_rect(
                        &mut enter_rect,
                        0,
                        GLIDER_STARTS_DOWN + self.this_room.left_start as i32 - 2,
                    );
                    self.gliders[0].entered_rect = enter_rect;
                    self.gliders[1].entered_rect = enter_rect;
                } else {
                    self.offset_glider(g, TO_LEFT);
                    q_set_rect(&mut enter_rect, 0, 0, 48, 20);
                    q_offset_rect(
                        &mut enter_rect,
                        0,
                        GLIDER_STARTS_DOWN + self.this_room.left_start as i32 - 2,
                    );
                    self.gliders[g].entered_rect = enter_rect;
                }
            }

            TO_LEFT => {
                self.set_musical_mode(PROD_GAME_SCORE_MODE);
                if self.two_player_game {
                    self.undo_glider_limbo(0);
                    self.undo_glider_limbo(1);
                    self.insure_glider_facing_left(0);
                    self.insure_glider_facing_left(1);
                } else {
                    self.insure_glider_facing_left(g);
                }
                let west = self.local_numbers[WEST_ROOM as usize];
                self.force_this_room(west);
                if self.two_player_game {
                    self.offset_glider(0, TO_RIGHT);
                    self.offset_glider(1, TO_RIGHT);
                    q_set_rect(&mut enter_rect, 0, 0, 48, 20);
                    q_offset_rect(
                        &mut enter_rect,
                        ROOM_WIDE - 48,
                        GLIDER_STARTS_DOWN + self.this_room.right_start as i32 - 2,
                    );
                    self.gliders[0].entered_rect = enter_rect;
                    self.gliders[1].entered_rect = enter_rect;
                } else {
                    self.offset_glider(g, TO_RIGHT);
                    q_set_rect(&mut enter_rect, 0, 0, 48, 20);
                    q_offset_rect(
                        &mut enter_rect,
                        ROOM_WIDE - 48,
                        GLIDER_STARTS_DOWN + self.this_room.right_start as i32 - 2,
                    );
                    self.gliders[g].entered_rect = enter_rect;
                }
            }

            ABOVE => {
                self.set_musical_mode(KICK_GAME_SCORE_MODE);
                let north = self.local_numbers[NORTH_ROOM as usize];
                self.force_this_room(north);
                if !self.taking_the_stairs {
                    if self.two_player_game {
                        self.undo_glider_limbo(0);
                        self.undo_glider_limbo(1);
                        self.offset_glider(0, BELOW);
                        self.offset_glider(1, BELOW);
                        self.gliders[0].entered_rect = self.gliders[0].dest;
                        self.gliders[1].entered_rect = self.gliders[1].dest;
                    } else {
                        self.offset_glider(g, BELOW);
                        self.gliders[g].entered_rect = self.gliders[g].dest;
                    }
                } else {
                    if self.two_player_game {
                        self.ready_glider_for_trip_up_stairs(0);
                        self.ready_glider_for_trip_up_stairs(1);
                    } else {
                        self.ready_glider_for_trip_up_stairs(g);
                    }
                }
            }

            BELOW => {
                self.set_musical_mode(KICK_GAME_SCORE_MODE);
                let south = self.local_numbers[SOUTH_ROOM as usize];
                self.force_this_room(south);
                if !self.taking_the_stairs {
                    if self.two_player_game {
                        self.undo_glider_limbo(0);
                        self.undo_glider_limbo(1);
                        self.offset_glider(0, ABOVE);
                        self.offset_glider(1, ABOVE);
                        self.gliders[0].entered_rect = self.gliders[0].dest;
                        self.gliders[1].entered_rect = self.gliders[1].dest;
                    } else {
                        self.offset_glider(g, ABOVE);
                        self.gliders[g].entered_rect = self.gliders[g].dest;
                    }
                } else {
                    if self.two_player_game {
                        self.ready_glider_for_trip_down_stairs(0);
                        self.ready_glider_for_trip_down_stairs(1);
                    } else {
                        self.ready_glider_for_trip_down_stairs(g);
                    }
                }
            }

            _ => {}
        }

        if self.two_player_game && !self.one_player_left {
            if self.first_player == PLAYER1 {
                self.tag_glider_idle(1);
            } else {
                self.tag_glider_idle(0);
            }
        }

        self.ready_level();
        self.refresh_scoreboard(NORMAL_TITLE_MODE);
        let just_rooms_rect = self.just_rooms_rect;
        self.wipe_screen_on(where_, &just_rooms_rect);

        // (COMPILEQT: the TV's movie; not kept)
    }

    /// The body shared by TransportRoomToRoom, MoveDuctToDuct and
    /// MoveMailToMail (identical in the C).
    fn move_through_link(&mut self, g: usize) {
        self.set_musical_mode(KICK_GAME_SCORE_MODE);
        self.handle_room_visitation();

        let same_room = self.trans_room == self.this_room_number;
        if !same_room {
            let trans_room = self.trans_room;
            self.force_this_room(trans_room);
        }

        let linked_to_what = self.linked_to_what;
        if self.two_player_game {
            self.undo_glider_limbo(0); // turn off limbo if needed
            self.undo_glider_limbo(1); // turn off limbo if needed
            self.ready_glider_from_transit(0, linked_to_what);
            self.ready_glider_from_transit(1, linked_to_what);
        } else {
            self.ready_glider_from_transit(g, linked_to_what);
        }

        if !same_room {
            self.ready_level();
        }
        self.refresh_scoreboard(NORMAL_TITLE_MODE);
        if !same_room {
            let just_rooms_rect = self.just_rooms_rect;
            self.wipe_screen_on(ABOVE, &just_rooms_rect);
        }

        // (COMPILEQT: the TV's movie; not kept)
    }

    //--------------------------------------------------------------  TransportRoomToRoom

    pub fn transport_room_to_room(&mut self, g: usize) {
        self.move_through_link(g);
    }

    //--------------------------------------------------------------  MoveDuctToDuct

    pub fn move_duct_to_duct(&mut self, g: usize) {
        self.move_through_link(g);
    }

    //--------------------------------------------------------------  MoveMailToMail

    pub fn move_mail_to_mail(&mut self, g: usize) {
        self.move_through_link(g);
    }

    //--------------------------------------------------------------  HandleRoomVisitation

    pub fn handle_room_visitation(&mut self) {
        if !self.this_room.visited {
            let central = self.local_numbers[CENTRAL_ROOM as usize];
            self.house.rooms[central as usize].visited = true;
            self.the_score += ROOM_VISIT_SCORE;
            self.this_room.visited = true;
        }
    }

    //--------------------------------------------------------------  ForceKillGlider

    pub fn force_kill_glider(&mut self) {
        if self.gliders[0].mode == GLIDER_IN_LIMBO {
            if self.gliders[1].mode != GLIDER_FADING_OUT {
                self.start_glider_fading_out(1);
                self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                self.player_suicide = true;
            }
        } else if self.gliders[1].mode == GLIDER_IN_LIMBO {
            if self.gliders[0].mode != GLIDER_FADING_OUT {
                self.start_glider_fading_out(0);
                self.play_priority_sound(FADE_OUT_SOUND, FADE_OUT_PRIORITY);
                self.player_suicide = true;
            }
        }
    }

    //--------------------------------------------------------------  FollowTheLeader

    pub fn follow_the_leader(&mut self) {
        // (oneOrTwo is uninitialized in the C if neither glider is in
        // limbo; false here)
        let mut one_or_two = false;

        self.player_suicide = false;
        let was_escaped = self.other_player_escaped;
        self.other_player_escaped = NO_ONE_ESCAPED;

        if self.gliders[0].mode == GLIDER_IN_LIMBO {
            one_or_two = true;
            self.gliders[1].dest = self.gliders[0].dest;
            self.gliders[1].dest_shadow = self.gliders[0].dest_shadow;
            self.gliders[1].whole = self.gliders[1].dest;
            self.gliders[1].whole_shadow = self.gliders[1].dest_shadow;
        } else if self.gliders[1].mode == GLIDER_IN_LIMBO {
            one_or_two = false;
            self.gliders[0].dest = self.gliders[1].dest;
            self.gliders[0].dest_shadow = self.gliders[1].dest_shadow;
            self.gliders[0].whole = self.gliders[0].dest;
            self.gliders[0].whole_shadow = self.gliders[0].dest_shadow;
        }

        // the glider that was left behind (theGlider2 if theGlider led)
        let other = if one_or_two { 1 } else { 0 };
        match was_escaped {
            PLAYER_ESCAPED_UP | PLAYER_ESCAPING_UP_STAIRS | PLAYER_ESCAPED_UP_STAIRS => {
                self.move_room_to_room(other, ABOVE);
            }

            PLAYER_ESCAPED_DOWN | PLAYER_ESCAPING_DOWN_STAIRS | PLAYER_ESCAPED_DOWN_STAIRS => {
                self.move_room_to_room(other, BELOW);
            }

            PLAYER_ESCAPED_LEFT => {
                self.move_room_to_room(other, TO_LEFT);
            }

            PLAYER_ESCAPED_RIGHT => {
                self.move_room_to_room(other, TO_RIGHT);
            }

            PLAYER_TRANSPORTED_OUT => {
                self.transport_room_to_room(other);
            }

            PLAYER_MAILED_OUT => {
                self.move_mail_to_mail(other);
            }

            PLAYER_DUCKED_OUT => {
                self.move_duct_to_duct(other);
            }

            _ => {}
        }
    }
}

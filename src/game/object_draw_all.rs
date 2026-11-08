//! ObjectDrawAll.c: drawing a room's objects into the background map and
//! setting up their animations (flames, saved maps, dynamic objects).

use super::*;

impl Game {
    /// GetObjectRect then OffsetRectRoomRelative, as every case below does.
    fn object_rect_relative(&self, obj: &mut Object, neighbor: i32) -> Rect {
        let mut its_rect = Rect::default();
        self.get_object_rect(obj, &mut its_rect);
        self.offset_rect_room_relative(&mut its_rect, neighbor);
        its_rect
    }

    /// The candles' flames (kTaper, kCandle, kStubby): `dh` is where the
    /// flame sits from the candle's left.
    fn candle_flame(&mut self, neighbor: i32, redraw: bool, i: i32, its_rect: &Rect, dh: i32) {
        let room = self.local_numbers[neighbor as usize];
        if neighbor == CENTRAL_ROOM {
            if redraw {
                self.re_back_up_flames(room, i);
            } else {
                self.add_candle_flame(room, i, its_rect.left + dh, its_rect.top + 7);
            }
        } else {
            let mut rect_a = Rect::default();
            let mut who_cares = Rect::default();
            q_set_rect(&mut rect_a, 0, 0, 16, 15);
            q_offset_rect(&mut rect_a, its_rect.left + dh - 8, its_rect.top + 7 - 15);
            let mut rect_b = self.local_rooms_dest[CENTRAL_ROOM as usize];
            rect_b.top -= FLOOR_SUPPORT_TALL;
            rect_b.bottom += FLOOR_SUPPORT_TALL;
            if !sect_rect(&rect_a, &rect_b, &mut who_cares) {
                if redraw {
                    self.re_back_up_flames(room, i);
                } else {
                    self.add_candle_flame(room, i, its_rect.left + dh, its_rect.top + 7);
                }
            }
        }
    }

    //--------------------------------------------------------------  DrawARoomsObjects

    pub fn draw_a_rooms_objects(&mut self, neighbor: i32, redraw: bool) {
        let mut who_cares = Rect::default();
        let mut floor = 0;
        let mut suite = 0;

        let room_n = self.local_numbers[neighbor as usize];
        if room_n == ROOM_IS_EMPTY {
            return;
        }

        let mut test_rect = self.house_rect;
        zero_rect_corner(&mut test_rect);
        let is_lit = self.num_lights > 0;

        for i in 0..MAX_ROOM_OBS {
            let mut dynamic_num = -1;
            let legit;

            if self.is_this_valid(room_n, i) {
                let mut this_object = self.house.rooms[room_n as usize].objects[i as usize];
                let what = this_object.what;
                match what {
                    OBJECT_IS_EMPTY => {}

                    FLOOR_VENT | CEILING_VENT | FLOOR_BLOWER | CEILING_BLOWER | SEWER_GRATE | LEFT_FAN | RIGHT_FAN
                    | GRECO_VENT | SEWER_BLOWER => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_simple_blowers(what, &its_rect);
                        }
                    }

                    TAPER | CANDLE | STUBBY => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if is_lit {
                                self.draw_simple_blowers(what, &its_rect);
                            }
                            let dh = match what {
                                TAPER => 10,
                                CANDLE => 14,
                                _ => 9, // STUBBY
                            };
                            self.candle_flame(neighbor, redraw, i, &its_rect, dh);
                        }
                    }

                    TIKI => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if is_lit {
                            let down = self.play_origin_v + self.vertical_room_offset(neighbor);
                            self.draw_tiki(&its_rect, down);
                        }
                        if redraw {
                            self.re_back_up_tiki_flames(room_n, i);
                        } else {
                            self.add_tiki_flame(room_n, i, its_rect.left + 10, its_rect.top - 9);
                        }
                    }

                    BBQ => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if is_lit {
                                self.draw_pict_sans_white_object(what, &its_rect);
                            }
                            if redraw {
                                self.re_back_up_bbq_coals(room_n, i);
                            } else {
                                self.add_bbq_coals(room_n, i, its_rect.left + 16, its_rect.top + 9);
                            }
                        }
                    }

                    INVIS_BLOWER | LIFT_AREA => {}

                    TABLE => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if is_lit {
                            let down = self.play_origin_v;
                            self.draw_table(&its_rect, down);
                        }
                    }

                    SHELF => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if is_lit {
                            self.draw_shelf(&its_rect);
                        }
                    }

                    CABINET => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_cabinet(&its_rect);
                        }
                    }

                    FILING_CABINET | OZMA => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_pict_object(what, &its_rect);
                        }
                    }

                    WASTE_BASKET | MILK_CRATE => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_simple_furniture(what, &its_rect);
                        }
                    }

                    COUNTER => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_counter(&its_rect);
                        }
                    }

                    DRESSER => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if is_lit {
                            self.draw_dresser(&its_rect);
                        }
                    }

                    DECK_TABLE => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if is_lit {
                            let down = self.play_origin_v;
                            self.draw_deck_table(&its_rect, down);
                        }
                    }

                    STOOL => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if is_lit {
                            let down = self.play_origin_v + self.vertical_room_offset(neighbor);
                            self.draw_stool(&its_rect, down);
                        }
                    }

                    INVIS_OBSTACLE => {}

                    MANHOLE => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            self.add_temp_manhole_rect(&its_rect);
                            if is_lit {
                                self.draw_pict_sans_white_object(what, &its_rect);
                            }
                        }
                    }

                    INVIS_BOUNCE => {}

                    RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | PAPER | BATTERY | BANDS | HELIUM | FOIL => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if redraw {
                                legit = self.re_back_up_saved_map(&its_rect, room_n, i);
                            } else {
                                legit = self.back_up_to_saved_map(&its_rect, room_n, i);
                            }
                            if legit != -1 {
                                match what {
                                    RED_CLOCK => self.draw_red_clock(&its_rect),
                                    BLUE_CLOCK => self.draw_blue_clock(&its_rect),
                                    YELLOW_CLOCK => self.draw_yellow_clock(&its_rect),
                                    FOIL => self.draw_foil(&its_rect),
                                    _ => self.draw_simple_prizes(what, &its_rect),
                                }
                            }
                        }
                    }

                    CUCKOO => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if redraw {
                                legit = self.re_back_up_saved_map(&its_rect, room_n, i);
                            } else {
                                legit = self.back_up_to_saved_map(&its_rect, room_n, i);
                            }
                            if legit != -1 {
                                self.draw_cuckoo(&its_rect);
                                if redraw {
                                    self.re_back_up_pendulum(room_n, i);
                                } else {
                                    self.add_pendulum(room_n, i, its_rect.left + 4, its_rect.top + 46);
                                }
                            }
                        }
                    }

                    GREASE_RT | GREASE_LF => {
                        let is_right = what == GREASE_RT;
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        let c = this_object.c();
                        if c.state {
                            // standing
                            if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                                if redraw {
                                    dynamic_num = self.re_back_up_grease(room_n, i);
                                } else {
                                    dynamic_num =
                                        self.add_grease(room_n, i, its_rect.left, its_rect.top, c.length, is_right);
                                }
                                if dynamic_num != -1 {
                                    if is_right {
                                        self.draw_grease_rt(&its_rect, c.length, true);
                                    } else {
                                        self.draw_grease_lf(&its_rect, c.length, true);
                                    }
                                }
                            }
                        } else {
                            // fallen
                            if is_right {
                                self.draw_grease_rt(&its_rect, c.length, false);
                            } else {
                                self.draw_grease_lf(&its_rect, c.length, false);
                            }
                        }
                    }

                    INVIS_BONUS | SLIDER => {}

                    STAR => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if redraw {
                                legit = self.re_back_up_saved_map(&its_rect, room_n, i);
                            } else {
                                legit = self.back_up_to_saved_map(&its_rect, room_n, i);
                            }
                            if legit != -1 {
                                if redraw {
                                    self.re_back_up_star(room_n, i);
                                } else {
                                    self.add_star(room_n, i, its_rect.left, its_rect.top);
                                }
                                self.draw_simple_prizes(what, &its_rect);
                            }
                        }
                    }

                    SPARKLE => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if !redraw && (neighbor == CENTRAL_ROOM) {
                                let mut rect_a = its_rect;
                                q_offset_rect(&mut rect_a, -self.play_origin_h, -self.play_origin_v);
                                let state = this_object.c().state;
                                dynamic_num = self.add_dynamic_object(SPARKLE, &rect_a, &this_object, room_n, i, state);
                            }
                        }
                    }

                    UP_STAIRS | DOOR_IN_LF | DOOR_IN_RT | WINDOW_IN_LF | WINDOW_IN_RT => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            self.draw_pict_sans_white_object(what, &its_rect);
                        }
                    }

                    DOWN_STAIRS | DOOR_EX_RT | DOOR_EX_LF | WINDOW_EX_RT | WINDOW_EX_LF => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            self.draw_pict_object(what, &its_rect);
                        }
                    }

                    MAILBOX_LF => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        let down = self.play_origin_v + self.vertical_room_offset(neighbor);
                        self.draw_mailbox_left(&its_rect, down);
                    }

                    MAILBOX_RT => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        let down = self.play_origin_v + self.vertical_room_offset(neighbor);
                        self.draw_mailbox_right(&its_rect, down);
                    }

                    FLOOR_TRANS | CEILING_TRANS => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            self.draw_simple_transport(what, &its_rect);
                        }
                    }

                    INVIS_TRANS | DELUXE_TRANS => {}

                    LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            let e = this_object.e();
                            self.extract_floor_suite(e.where_, &mut floor, &mut suite);
                            let room = self.get_room_number(floor, suite);
                            let obj = e.who;
                            let state = self.get_object_state(room, obj);
                            match what {
                                LIGHT_SWITCH => self.draw_light_switch(&its_rect, state),
                                MACHINE_SWITCH => self.draw_machine_switch(&its_rect, state),
                                THERMOSTAT => self.draw_thermostat(&its_rect, state),
                                POWER_SWITCH => self.draw_power_switch(&its_rect, state),
                                _ => self.draw_knife_switch(&its_rect, state),
                            }
                        }
                        // (indexed by the object's number in its room, as in the C)
                        dynamic_num = self.master_objects[i as usize].hot_num;
                    }

                    INVIS_SWITCH => {
                        dynamic_num = self.master_objects[i as usize].hot_num;
                    }

                    TRIGGER | LG_TRIGGER | SOUND_TRIGGER => {}

                    CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_simple_light(what, &its_rect);
                        }
                    }

                    TRUNK | BOOKS | HIP_LAMP | DECO_LAMP | GUITAR | CINDER_BLOCK | FLOWER_BOX | FIREPLACE | BEAR | VASE1
                    | VASE2 | RUG | CHIMES => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_pict_sans_white_object(what, &its_rect);
                        }
                    }

                    CUSTOM_PICT => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_cust_pict_sans_white(this_object.g().height, &its_rect);
                        }
                    }

                    FLOURESCENT => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_flourescent(&its_rect);
                        }
                    }

                    TRACK_LIGHT => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_track_light(&its_rect);
                        }
                    }

                    INVIS_LIGHT => {}

                    SHREDDER | C_DS => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_simple_appliance(what, &its_rect);
                        }
                    }

                    TOASTER => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            self.draw_simple_appliance(what, &its_rect);
                            if !redraw && (neighbor == CENTRAL_ROOM) {
                                let mut rect_a = its_rect;
                                q_offset_rect(&mut rect_a, -self.play_origin_h, -self.play_origin_v);
                                let state = this_object.g().state;
                                dynamic_num = self.add_dynamic_object(TOASTER, &rect_a, &this_object, room_n, i, state);
                            }
                        }
                    }

                    MAC_PLUS | TV | COFFEE | OUTLET | VCR | STEREO | MICROWAVE => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            let state = this_object.g().state;
                            match what {
                                MAC_PLUS => self.draw_mac_plus(&its_rect, state, is_lit),
                                TV => {
                                    // (the QuickTime movie on the TV is left out)
                                    self.draw_tv(&its_rect, state, is_lit)
                                }
                                COFFEE => self.draw_coffee(&its_rect, state, is_lit),
                                OUTLET => {
                                    if is_lit {
                                        self.draw_outlet(&its_rect);
                                    }
                                }
                                VCR => self.draw_vcr(&its_rect, state, is_lit),
                                STEREO => {
                                    let on = self.is_play_music_game;
                                    self.draw_stereo(&its_rect, on, is_lit)
                                }
                                _ => self.draw_microwave(&its_rect, state, is_lit),
                            }
                            if !redraw {
                                let mut rect_a = its_rect;
                                q_offset_rect(&mut rect_a, -self.play_origin_h, -self.play_origin_v);
                                dynamic_num = self.add_dynamic_object(what, &rect_a, &this_object, room_n, i, state);
                            }
                        }
                    }

                    BALLOON | COPTER_LF | COPTER_RT | DART_LF | DART_RT | BALL => {
                        if (neighbor == CENTRAL_ROOM) && !redraw {
                            let mut its_rect = self.object_rect_relative(&mut this_object, neighbor);
                            q_offset_rect(&mut its_rect, -self.play_origin_h, -self.play_origin_v);
                            let state = this_object.h().state;
                            dynamic_num = self.add_dynamic_object(what, &its_rect, &this_object, room_n, i, state);
                        }
                    }

                    DRIP | FISH => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            if what == DRIP {
                                self.draw_drip(&its_rect);
                            } else {
                                self.draw_fish(what, &its_rect);
                            }
                            if !redraw && (neighbor == CENTRAL_ROOM) {
                                let mut rect_a = its_rect;
                                q_offset_rect(&mut rect_a, -self.play_origin_h, -self.play_origin_v);
                                let state = this_object.h().state;
                                dynamic_num = self.add_dynamic_object(what, &rect_a, &this_object, room_n, i, state);
                            }
                        }
                    }

                    COBWEB | CLOUD => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_pict_with_mask_object(what, &its_rect);
                        }
                    }

                    MIRROR => {
                        let mut its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_mirror(&its_rect);
                        }
                        if (neighbor == CENTRAL_ROOM) && !redraw {
                            inset_rect(&mut its_rect, 4, 4);
                            self.add_to_mirror_region(&its_rect);
                        }
                    }

                    MOUSEHOLE | FAUCET => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_simple_clutter(what, &its_rect);
                        }
                    }

                    FLOWER => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_flower(&its_rect, this_object.i().pict);
                        }
                    }

                    WALL_WINDOW => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) {
                            self.draw_wall_window(&its_rect);
                        }
                    }

                    CALENDAR => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_calendar(&its_rect);
                        }
                    }

                    BULLETIN => {
                        let its_rect = self.object_rect_relative(&mut this_object, neighbor);
                        if sect_rect(&its_rect, &test_rect, &mut who_cares) && is_lit {
                            self.draw_bulletin(&its_rect);
                        }
                    }

                    _ => {}
                }
            }

            if !redraw {
                // set up links
                for n in 0..self.num_master_objects as usize {
                    if (self.master_objects[n].object_num == i) && (self.master_objects[n].room_num == room_n) {
                        self.master_objects[n].dyna_num = dynamic_num;
                    }
                }
            }
        }
    }
}

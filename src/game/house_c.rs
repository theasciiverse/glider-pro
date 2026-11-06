//! House.c (the parts play needs), what HouseIO.c's ReadHouse does once a
//! house is in memory, and Link.c's floor/suite codes.

use super::*;

impl Game {
    // CreateNewHouse, InitializeEmptyHouse: the editor's, left out.

    //--------------------------------------------------------------  RealRoomNumberCount

    /// Returns the real number of rooms in a house (some rooms may still be
    /// place-holders - they were deleted earlier and are flagged as deleted
    /// but still occupy space in the file).
    pub fn real_room_number_count(&self) -> i32 {
        let n_rooms = self.house.rooms.len() as i32;
        let mut real_room_count = n_rooms;
        if real_room_count != 0 {
            for i in 0..n_rooms as usize {
                if self.house.rooms[i].suite == ROOM_IS_EMPTY {
                    real_room_count -= 1;
                }
            }
        }

        real_room_count
    }

    //--------------------------------------------------------------  GetFirstRoomNumber

    /// Returns the room number (indicee into house file) of the room where
    /// the player is to begin.
    pub fn get_first_room_number(&mut self) -> i32 {
        let n_rooms = self.house.rooms.len() as i32;
        let mut first_room;
        if n_rooms <= 0 {
            first_room = -1;
            self.no_room_at_all = true;
        } else {
            first_room = self.house.first_room;
            if (first_room >= n_rooms) || (first_room < 0) {
                first_room = 0;
            }
        }

        first_room
    }

    //--------------------------------------------------------------  WhereDoesGliderBegin

    /// Returns a rectangle indicating where in the first room the player's
    /// glider is to appear.
    pub fn where_does_glider_begin(&self, the_rect: &mut Rect, mode: i32) {
        // Saved games are cut: kResumeGameMode (smallGame.where) starts
        // where a new game does.
        if mode == RESUME_GAME_MODE {
            log::warn!("WhereDoesGliderBegin: no saved game; starting as a new game");
        }
        let initial_pt = self.house.initial;

        q_set_rect(the_rect, 0, 0, GLIDER_WIDE, GLIDER_HIGH);
        q_offset_rect(the_rect, initial_pt.h, initial_pt.v);
    }

    //--------------------------------------------------------------  HouseHasOriginalPicts

    /// Returns true is the current house has custom artwork imbedded.
    pub fn house_has_original_picts(&self) -> bool {
        let n_picts = self.house.res.keys().filter(|(t, _)| t == b"PICT").count();
        n_picts > 0
    }

    //--------------------------------------------------------------  CountHouseLinks

    /// Counts up the number of linked objects in a house.
    pub fn count_house_links(&self) -> i32 {
        let mut num_links = 0;

        for room in &self.house.rooms {
            for obj in room.objects.iter() {
                match obj.what {
                    LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
                    | LG_TRIGGER => {
                        if obj.e().where_ != -1 {
                            num_links += 1;
                        }
                    }

                    MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | INVIS_TRANS | DELUXE_TRANS => {
                        if obj.d().where_ != -1 {
                            num_links += 1;
                        }
                    }

                    _ => {}
                }
            }
        }

        num_links
    }

    //--------------------------------------------------------------  GenerateLinksList

    /// Generates a list of all objects that have links and what rooms and
    /// objects they are linked to. It is called in order to preserve the
    /// links if the objects or rooms in a house are to be shuffled around.
    /// (The C fills a list its caller allocated; here `links_list` is
    /// rebuilt.)
    pub fn generate_links_list(&mut self) {
        let mut floor = 0;
        let mut suite = 0;
        let mut list = Vec::new();

        for r in 0..self.house.rooms.len() {
            for i in 0..MAX_ROOM_OBS as usize {
                let this_object = self.house.rooms[r].objects[i];
                let (where_, who) = match this_object.what {
                    LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
                    | LG_TRIGGER => (this_object.e().where_, this_object.e().who),

                    MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | INVIS_TRANS | DELUXE_TRANS => {
                        (this_object.d().where_, this_object.d().who)
                    }

                    _ => continue,
                };
                if where_ != -1 {
                    self.extract_floor_suite(where_, &mut floor, &mut suite);
                    let room_linked = self.get_room_number(floor, suite);
                    let object_linked = who;
                    list.push(Links {
                        src_room: r as i32,
                        src_obj: i as i32,
                        dest_room: room_linked,
                        dest_obj: object_linked,
                    });
                }
            }
        }

        self.links_list = list;
    }

    // SortRoomsObjects, SortHouseObjects: the editor's, left out.

    //--------------------------------------------------------------  CountRoomsVisited

    /// Goes through and counts the number of rooms a player has been to in
    /// the current game.
    pub fn count_rooms_visited(&self) -> i32 {
        self.house.rooms.iter().filter(|r| r.visited).count() as i32
    }

    //--------------------------------------------------------------  GenerateRetroLinks

    /// Walk entire house looking for objects which are linked to objects in
    /// the current room.
    pub fn generate_retro_links(&mut self) {
        let mut floor = 0;
        let mut suite = 0;

        for i in 0..MAX_ROOM_OBS as usize {
            // Initialize array.
            self.retro_link_list[i].room = -1;
        }

        for r in 0..self.house.rooms.len() {
            for i in 0..MAX_ROOM_OBS as usize {
                let this_object = self.house.rooms[r].objects[i];
                let (where_, who) = match this_object.what {
                    LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
                    | LG_TRIGGER => (this_object.e().where_, this_object.e().who),

                    MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | INVIS_TRANS | DELUXE_TRANS => {
                        (this_object.d().where_, this_object.d().who)
                    }

                    _ => continue,
                };
                if where_ != -1 {
                    self.extract_floor_suite(where_, &mut floor, &mut suite);
                    let room_linked = self.get_room_number(floor, suite);
                    if room_linked == self.this_room_number {
                        let object_linked = who as usize;
                        // (the C writes past the list for an object
                        // number of 24 or more; skipped here)
                        if object_linked < MAX_ROOM_OBS as usize && self.retro_link_list[object_linked].room == -1 {
                            self.retro_link_list[object_linked].room = r as i32;
                            self.retro_link_list[object_linked].object = i as i32;
                        }
                    }
                }
            }
        }
    }

    // UpdateGoToDialog, GoToFilter, DoGoToDialog, ConvertHouseVer1To2,
    // ShiftWholeHouse: dialogs and editor commands, left out.

    // ================================================== from Link.c

    //--------------------------------------------------------------  MergeFloorSuite

    pub fn merge_floor_suite(&self, floor: i32, suite: i32) -> i32 {
        (suite * 100) + floor
    }

    //--------------------------------------------------------------  ExtractFloorSuite

    pub fn extract_floor_suite(&self, combo: i32, floor: &mut i32, suite: &mut i32) {
        if self.house.version < 0x0200 {
            // old floor/suite combo
            *floor = (combo / 100) - NUM_UNDERGROUND_FLOORS;
            *suite = combo % 100;
        } else {
            *suite = combo / 100;
            *floor = (combo % 100) - NUM_UNDERGROUND_FLOORS;
        }
    }

    // ================================================== from HouseIO.c

    //--------------------------------------------------------------  ReadHouse

    /// What ReadHouse does once the house is read into `self.house`
    /// (OpenHouse and the file reading are `house::load`). False if the
    /// house can't be played.
    pub fn read_house(&mut self) -> bool {
        self.number_rooms = self.house.rooms.len() as i32;
        if self.number_rooms < 1 {
            self.number_rooms = 0;
            self.no_room_at_all = true;
            log::warn!("house has no rooms (kYellowNoRooms)");
            return false;
        }

        self.was_house_version = self.house.version;
        if self.was_house_version >= NEW_HOUSE_VERSION {
            log::warn!("house is of a newer version (kYellowNewerVersion)");
            return false;
        }

        self.house_unlocked = (self.house.time_stamp & 0x00000001) == 0;

        let which_room = self.house.first_room;

        self.ward_bit_set = (self.house.flags & 0x00000001) == 0x00000001;
        self.phone_bit_set = (self.house.flags & 0x00000002) == 0x00000002;
        self.banner_star_count_on = (self.house.flags & 0x00000004) == 0x00000000;

        // (*thisHouse)->highScores
        let hs = &self.house.high_scores;
        self.high_scores = Scores {
            banner: crate::mac::mac_roman(&hs.banner),
            names: std::array::from_fn(|i| crate::mac::mac_roman(&hs.names[i])),
            scores: hs.scores,
            time_stamps: hs.time_stamps,
            levels: hs.levels,
        };

        self.no_room_at_all = self.real_room_number_count() == 0;
        self.this_room_number = -1;
        self.previous_room = -1;
        if !self.no_room_at_all {
            self.copy_room_to_this_room(which_room);
        }

        // (read-only houses' scores file, the editor's selection and
        // ReflectCurrentRoom are left out)

        true
    }
}

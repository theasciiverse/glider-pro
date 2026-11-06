//! Room.c: the current room (`thisRoom`), finding rooms in the house, and
//! what a room's walls, floor and ceiling are.

use super::*;
use std::rc::Rc;

impl Game {
    // SetInitialTiles, CreateNewRoom: the editor's, left out.

    /// GetPicture(id), else the house's 'Date' resource of that number
    /// (where houses keep some backgrounds), as ReadyBackground,
    /// LoadGraphicSpecial and PictIDExists look for them.
    pub fn get_picture_or_date(&self, id: i32) -> Option<Rc<crate::mac::Image>> {
        if let Some(p) = self.picture(id) {
            return Some(p);
        }
        let v = self.house.res.get(&(*b"Date", id as i16))?;
        match crate::mac::pict(v) {
            Ok(img) => Some(Rc::new(img)),
            Err(e) => {
                log::warn!("'Date' {id}: {e}");
                None
            }
        }
    }

    //--------------------------------------------------------------  ReadyBackground

    pub fn ready_background(&mut self, the_id: i32, the_tiles: &[i32]) {
        let mut src = Rect::default();
        let mut dest = Rect::default();

        if self.no_room_at_all || !self.house_unlocked {
            // LtGrayForeColor
            self.work_src_map.fore([0xBF, 0xBF, 0xBF]);
            let r = self.work_src_rect;
            self.work_src_map.paint_rect(r);
            self.work_src_map.fore(crate::gfx::BLACK);
            if self.house_unlocked {
                self.label(10, 20, "No rooms", crate::gfx::BLACK, 0);
            } else {
                self.label(10, 20, "Nothing to show", crate::gfx::BLACK, 0);
            }

            self.back_src_map.copy_bits(&self.work_src_map, r, r);
            return;
        }

        let Some(the_picture) = self.get_picture_or_date(the_id) else {
            log::warn!("no background {the_id} (kYellowNoBackground)");
            return;
        };

        q_set_rect(&mut dest, 0, 0, the_picture.w, the_picture.h);
        self.work_src_map.draw_picture(&the_picture, dest);

        q_set_rect(&mut src, 0, 0, TILE_WIDE, TILE_HIGH);
        q_set_rect(&mut dest, 0, 0, TILE_WIDE, TILE_HIGH);
        for i in 0..NUM_TILES as usize {
            src.left = the_tiles[i] * TILE_WIDE;
            src.right = src.left + TILE_WIDE;
            self.back_src_map.copy_bits(&self.work_src_map, src, dest);
            q_offset_rect(&mut dest, TILE_WIDE, 0);
        }

        q_set_rect(&mut src, 0, 0, ROOM_WIDE, TILE_HIGH);
        q_set_rect(&mut dest, 0, 0, ROOM_WIDE, TILE_HIGH);
        self.work_src_map.copy_bits(&self.back_src_map, src, dest);
    }

    // ReflectCurrentRoom: does nothing outside the editor; left out.

    //--------------------------------------------------------------  CopyRoomToThisRoom

    pub fn copy_room_to_this_room(&mut self, room_number: i32) {
        if room_number == -1 {
            return;
        }

        self.copy_this_room_to_room(); // copy back to house
        self.force_this_room(room_number); // load new room from house
    }

    //--------------------------------------------------------------  CopyThisRoomToRoom

    pub fn copy_this_room_to_room(&mut self) {
        if self.no_room_at_all || (self.this_room_number == -1) {
            return;
        }

        // copy back to house
        self.house.rooms[self.this_room_number as usize] = self.this_room.clone();
    }

    //--------------------------------------------------------------  ForceThisRoom

    pub fn force_this_room(&mut self, room_number: i32) {
        if room_number == -1 {
            return;
        }

        if (room_number as usize) < self.house.rooms.len() {
            self.this_room = self.house.rooms[room_number as usize].clone();
        } else {
            log::warn!("illegal room number {room_number} (kYellowIllegalRoomNum)");
        }

        self.previous_room = self.this_room_number;
        self.this_room_number = room_number;
    }

    //--------------------------------------------------------------  RoomExists

    /// Pass in a suite and floor; returns true is it is a legitimate room.
    pub fn room_exists(&self, suite: i32, floor: i32, room_num: &mut i32) -> bool {
        let mut found_it = false;

        if suite < 0 {
            return found_it;
        }

        for i in 0..self.number_rooms {
            let r = &self.house.rooms[i as usize];
            if (r.floor == floor) && (r.suite == suite) {
                found_it = true;
                *room_num = i;
                break;
            }
        }

        found_it
    }

    //--------------------------------------------------------------  RoomNumExists

    pub fn room_num_exists(&self, room_num: i32) -> bool {
        let mut floor = 0;
        let mut suite = 0;
        let mut who_cares = 0;

        let mut exists = false;
        if self.get_room_floor_suite(room_num, &mut floor, &mut suite) {
            exists = self.room_exists(suite, floor, &mut who_cares);
        }

        exists
    }

    // DeleteRoom, QueryDeleteRoom, DoesNeighborRoomExist, SelectNeighborRoom:
    // the editor's, left out.

    //--------------------------------------------------------------  GetNeighborRoomNumber

    pub fn get_neighbor_room_number(&self, which: i32) -> i32 {
        let (h_delta, v_delta) = match which {
            CENTRAL_ROOM => (0, 0),
            NORTH_ROOM => (0, 1),
            NORTH_EAST_ROOM => (1, 1),
            EAST_ROOM => (1, 0),
            SOUTH_EAST_ROOM => (1, -1),
            SOUTH_ROOM => (0, -1),
            SOUTH_WEST_ROOM => (-1, -1),
            WEST_ROOM => (-1, 0),
            NORTH_WEST_ROOM => (-1, 1),
            _ => (0, 0), // (unset in the C)
        };

        let mut room_num = ROOM_IS_EMPTY;
        let this = &self.house.rooms[self.this_room_number as usize];
        let room_h = this.suite + h_delta;
        let room_v = this.floor + v_delta;

        for i in 0..self.number_rooms {
            let r = &self.house.rooms[i as usize];
            if (r.suite == room_h) && (r.floor == room_v) {
                room_num = i;
                break;
            }
        }

        room_num
    }

    // SetToNearestNeighborRoom: only DeleteRoom (editor) uses it; left out.

    //--------------------------------------------------------------  GetRoomFloorSuite

    pub fn get_room_floor_suite(&self, room: i32, floor: &mut i32, suite: &mut i32) -> bool {
        let r = &self.house.rooms[room as usize];
        if r.suite == ROOM_IS_EMPTY {
            *floor = 0;
            *suite = ROOM_IS_EMPTY;
            false
        } else {
            *suite = r.suite;
            *floor = r.floor;
            true
        }
    }

    //--------------------------------------------------------------  GetRoomNumber

    /// Pass in a floor and suite; returns the room index into the house file.
    pub fn get_room_number(&self, floor: i32, suite: i32) -> i32 {
        let mut room_num = ROOM_IS_EMPTY;

        for i in 0..self.number_rooms {
            let r = &self.house.rooms[i as usize];
            if (r.suite == suite) && (r.floor == floor) {
                room_num = i;
                break;
            }
        }

        room_num
    }

    //--------------------------------------------------------------  IsRoomAStructure

    pub fn is_room_a_structure(&self, room_num: i32) -> bool {
        if room_num == ROOM_IS_EMPTY {
            return false;
        }

        let r = &self.house.rooms[room_num as usize];
        if r.background >= USER_BACKGROUND {
            if r.bounds != 0 {
                (r.bounds & 32) == 32
            } else {
                r.background < USER_STRUCTURE_RANGE
            }
        } else {
            matches!(
                r.background,
                PANELED_ROOM
                    | SIMPLE_ROOM
                    | CHILDS_ROOM
                    | ASIAN_ROOM
                    | UNFINISHED_ROOM
                    | SWINGERS_ROOM
                    | BATHROOM
                    | LIBRARY
                    | SKYWALK
                    | ROOF
            )
        }
    }

    //--------------------------------------------------------------  DetermineRoomOpenings

    pub fn determine_room_openings(&mut self) {
        let which_back = self.this_room.background;
        let left_tile = self.this_room.tiles[0];
        let right_tile = self.this_room.tiles[NUM_TILES as usize - 1];

        if which_back >= USER_BACKGROUND {
            let bounds_code =
                if self.this_room.bounds != 0 { self.this_room.bounds >> 1 } else { self.get_original_bounding(which_back) };
            self.left_open = (bounds_code & 0x0001) == 0x0001;
            self.right_open = (bounds_code & 0x0004) == 0x0004;

            self.left_thresh = if self.left_open { NO_LEFT_WALL_LIMIT } else { LEFT_WALL_LIMIT };
            self.right_thresh = if self.right_open { NO_RIGHT_WALL_LIMIT } else { RIGHT_WALL_LIMIT };
        } else {
            match which_back {
                SIMPLE_ROOM | PANELED_ROOM | BASEMENT | CHILDS_ROOM | ASIAN_ROOM | UNFINISHED_ROOM | SWINGERS_ROOM
                | BATHROOM | LIBRARY | SKY => {
                    self.left_thresh = if left_tile == 0 { LEFT_WALL_LIMIT } else { NO_LEFT_WALL_LIMIT };
                    self.right_thresh =
                        if right_tile == (NUM_TILES - 1) { RIGHT_WALL_LIMIT } else { NO_RIGHT_WALL_LIMIT };
                    self.left_open = left_tile != 0;
                    self.right_open = right_tile != (NUM_TILES - 1);
                }

                DIRT => {
                    self.left_thresh = if left_tile == 1 { LEFT_WALL_LIMIT } else { NO_LEFT_WALL_LIMIT };
                    self.right_thresh =
                        if right_tile == (NUM_TILES - 1) { RIGHT_WALL_LIMIT } else { NO_RIGHT_WALL_LIMIT };
                    self.left_open = left_tile != 0;
                    self.right_open = right_tile != (NUM_TILES - 1);
                }

                MEADOW => {
                    self.left_thresh = if left_tile == 6 { LEFT_WALL_LIMIT } else { NO_LEFT_WALL_LIMIT };
                    self.right_thresh = if right_tile == 7 { RIGHT_WALL_LIMIT } else { NO_RIGHT_WALL_LIMIT };
                    self.left_open = left_tile != 6;
                    self.right_open = right_tile != 7;
                }

                GARDEN | SKYWALK | FIELD | STRATOSPHERE | STARS => {
                    self.left_thresh = NO_LEFT_WALL_LIMIT;
                    self.right_thresh = NO_RIGHT_WALL_LIMIT;
                    self.left_open = true;
                    self.right_open = true;
                }

                _ => {
                    self.left_thresh = if left_tile == 0 { LEFT_WALL_LIMIT } else { NO_LEFT_WALL_LIMIT };
                    self.right_thresh =
                        if right_tile == (NUM_TILES - 1) { RIGHT_WALL_LIMIT } else { NO_RIGHT_WALL_LIMIT };
                    self.left_open = left_tile != 0;
                    self.right_open = right_tile != (NUM_TILES - 1);
                }
            }
        }

        self.bottom_open = !self.does_room_have_floor();
        self.top_open = !self.does_room_have_ceiling();
    }

    //--------------------------------------------------------------  GetOriginalBounding

    /// The 'bnds' resource of a house's own background (four Booleans:
    /// left, top, right, bottom).
    pub fn get_original_bounding(&self, the_id: i32) -> i32 {
        let mut bound_code = 0;
        match self.house.res.get(&(*b"bnds", the_id as i16)).filter(|b| b.len() >= 4) {
            None => {
                if self.pict_id_exists(the_id) {
                    log::warn!("no 'bnds' {the_id} (kYellowNoBoundsRes)");
                }
            }
            Some(b) => {
                if b[0] != 0 {
                    bound_code += 1;
                }
                if b[1] != 0 {
                    bound_code += 2;
                }
                if b[2] != 0 {
                    bound_code += 4;
                }
                if b[3] != 0 {
                    bound_code += 8;
                }
            }
        }

        bound_code
    }

    //--------------------------------------------------------------  PictIDExists

    /// From RoomInfo.c (the editor's), as GetOriginalBounding needs it.
    pub fn pict_id_exists(&self, the_id: i32) -> bool {
        self.picture(the_id).is_some() || self.house.res.contains_key(&(*b"Date", the_id as i16))
    }

    //--------------------------------------------------------------  GetNumberOfLights

    pub fn get_number_of_lights(&self, where_: i32) -> i32 {
        // (the editor's branch, counting from thisRoom, is left out)
        let r = &self.house.rooms[where_ as usize];
        let mut count = match r.background {
            GARDEN | SKYWALK | MEADOW | FIELD | ROOF | SKY | STRATOSPHERE | STARS => 1,

            DIRT => {
                if r.tiles.iter().all(|&t| t == 0) {
                    1
                } else {
                    0
                }
            }

            _ => 0,
        };
        if count == 0 {
            for i in 0..MAX_ROOM_OBS as usize {
                match r.objects[i].what {
                    DOOR_IN_LF | DOOR_IN_RT | WINDOW_IN_LF | WINDOW_IN_RT | WALL_WINDOW => {
                        count += 1;
                    }

                    CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP | FLOURESCENT | TRACK_LIGHT
                    | INVIS_LIGHT => {
                        if r.objects[i].f().state {
                            count += 1;
                        }
                    }

                    _ => {}
                }
            }
        }
        count
    }

    //--------------------------------------------------------------  IsShadowVisible

    pub fn is_shadow_visible(&self) -> bool {
        if self.this_room.background >= USER_BACKGROUND {
            let bounds_code = if self.this_room.bounds != 0 {
                // is this a version 2.0 house?
                self.this_room.bounds >> 1
            } else {
                self.get_original_bounding(self.this_room.background)
            };
            (bounds_code & 0x0008) != 0x0008
        } else {
            !matches!(self.this_room.background, ROOF | SKY | STRATOSPHERE | STARS)
        }
    }

    //--------------------------------------------------------------  DoesRoomHaveFloor

    pub fn does_room_have_floor(&self) -> bool {
        if self.this_room.background >= USER_BACKGROUND {
            let bounds_code = if self.this_room.bounds != 0 {
                // is this a version 2.0 house?
                self.this_room.bounds >> 1
            } else {
                self.get_original_bounding(self.this_room.background)
            };
            (bounds_code & 0x0008) != 0x0008
        } else {
            !matches!(self.this_room.background, SKY | STRATOSPHERE | STARS)
        }
    }

    //--------------------------------------------------------------  DoesRoomHaveCeiling

    pub fn does_room_have_ceiling(&self) -> bool {
        if self.this_room.background >= USER_BACKGROUND {
            let bounds_code = if self.this_room.bounds != 0 {
                // is this a version 2.0 house?
                self.this_room.bounds >> 1
            } else {
                self.get_original_bounding(self.this_room.background)
            };
            (bounds_code & 0x0002) != 0x0002
        } else {
            !matches!(self.this_room.background, GARDEN | MEADOW | FIELD | ROOF | SKY | STRATOSPHERE | STARS)
        }
    }
}

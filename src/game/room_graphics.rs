//! RoomGraphics.c: drawing the rooms around (their backgrounds, objects
//! and floor supports) into the background map.

use super::*;

pub const MANHOLE_THRU_FLOOR: i32 = 3957;

impl Game {
    //--------------------------------------------------------------  DrawLocale

    pub fn draw_locale(&mut self) {
        // (the last room's text goes with it: a calendar's month)
        self.labels.clear();
        self.zero_flames_and_the_like();
        self.zero_dinahs();
        self.kill_all_bands();
        self.zero_mirror_region();
        self.zero_triggers();
        self.num_temp_manholes = 0;
        self.flush_any_trigger_playing();
        self.dump_trigger_sound();
        self.tv_in_room = false;
        self.tv_with_movie_number = -1;

        let room_v = self.house.rooms[self.this_room_number as usize].floor;

        for i in 0..9 {
            self.local_numbers[i] = self.get_neighbor_room_number(i as i32);
            self.is_structure[i] = self.is_room_a_structure(self.local_numbers[i]);
        }
        self.list_all_local_objects();

        let r = self.back_src_rect;
        self.back_src_map.paint_rect(r);

        if self.num_neighbors > 3 {
            for (n, v) in [
                (NORTH_WEST_ROOM, room_v + 1),
                (NORTH_EAST_ROOM, room_v + 1),
                (NORTH_ROOM, room_v + 1),
                (SOUTH_WEST_ROOM, room_v - 1),
                (SOUTH_EAST_ROOM, room_v - 1),
                (SOUTH_ROOM, room_v - 1),
            ] {
                self.num_lights = self.get_number_of_lights_or_none(self.local_numbers[n as usize]);
                self.draw_room_background(self.local_numbers[n as usize], n, v);
                self.draw_a_rooms_objects(n, false);
            }
        }

        if self.num_neighbors > 1 {
            for n in [WEST_ROOM, EAST_ROOM] {
                self.num_lights = self.get_number_of_lights_or_none(self.local_numbers[n as usize]);
                self.draw_room_background(self.local_numbers[n as usize], n, room_v);
                self.draw_a_rooms_objects(n, false);
                self.draw_lighting();
            }
        }

        self.num_lights = self.get_number_of_lights(self.local_numbers[CENTRAL_ROOM as usize]);
        self.draw_room_background(self.local_numbers[CENTRAL_ROOM as usize], CENTRAL_ROOM, room_v);
        // (for the silhouette view: the room's bare background, before its
        // objects, which the view softens wherever it still shows)
        self.room_backdrop = Some(self.back_src_map.clone());
        self.backdrop_serial += 1;
        self.draw_a_rooms_objects(CENTRAL_ROOM, false);
        self.draw_lighting();

        if self.num_neighbors > 3 {
            self.draw_floor_support();
        }
        self.restore_work_map();
        self.shadow_visible = self.is_shadow_visible();
        self.taking_the_stairs = false;
    }

    /// GetNumberOfLights for a neighbour that may be no room: the C reads
    /// rooms[-1] (whatever lies before the rooms) and the count is then
    /// unused, since DrawRoomBackground draws an empty spot regardless.
    fn get_number_of_lights_or_none(&self, room: i32) -> i32 {
        if room == ROOM_IS_EMPTY {
            1
        } else {
            self.get_number_of_lights(room)
        }
    }

    //--------------------------------------------------------------  LoadGraphicSpecial

    /// Draws the picture (the house's or Glider PRO's, else a 'Date'
    /// resource, else picture 2000) at its own size into the work map.
    pub fn load_graphic_special(&mut self, res_id: i32) {
        let the_picture = self.get_picture_or_date(res_id).or_else(|| self.picture(2000));
        let Some(the_picture) = the_picture else {
            log::warn!("graphic {res_id} failed to load (kErrFailedGraphicLoad)");
            return;
        };

        let bounds = Rect::new(0, 0, the_picture.w, the_picture.h);
        self.work_src_map.draw_picture(&the_picture, bounds);
    }

    //--------------------------------------------------------------  DrawRoomBackground

    pub fn draw_room_background(&mut self, who: i32, where_: i32, elevation: i32) {
        let mut src = Rect::default();
        let mut dest = Rect::default();
        let pict_id;
        let tiles;

        if where_ == CENTRAL_ROOM {
            let r = &self.house.rooms[who as usize];
            self.this_background = r.background;
            self.this_tiles = r.tiles;
        }

        if (self.num_lights == 0) && (who != ROOM_IS_EMPTY) {
            let r = self.local_rooms_dest[where_ as usize];
            self.back_src_map.paint_rect(r);
            return;
        }

        if who == ROOM_IS_EMPTY {
            // This call should be smarter than this
            if self.ward_bit_set {
                let r = self.local_rooms_dest[where_ as usize];
                self.back_src_map.paint_rect(r);
                return;
            }

            if elevation > 1 {
                pict_id = SKY;
                tiles = [2; NUM_TILES as usize];
            } else if elevation == 1 {
                pict_id = MEADOW;
                tiles = [0; NUM_TILES as usize];
            } else {
                pict_id = DIRT;
                tiles = [0; NUM_TILES as usize];
            }
        } else {
            let r = &self.house.rooms[who as usize];
            pict_id = r.background;
            tiles = r.tiles;
        }

        self.load_graphic_special(pict_id);

        q_set_rect(&mut src, 0, 0, TILE_WIDE, TILE_HIGH);
        q_set_rect(&mut dest, 0, 0, TILE_WIDE, TILE_HIGH);
        let d = self.local_rooms_dest[where_ as usize];
        q_offset_rect(&mut dest, d.left, d.top);
        for tile in tiles {
            src.left = tile * TILE_WIDE;
            src.right = src.left + TILE_WIDE;
            self.back_src_map.copy_bits(&self.work_src_map, src, dest);
            q_offset_rect(&mut dest, TILE_WIDE, 0);
        }
    }

    //--------------------------------------------------------------  DrawFloorSupport

    pub fn draw_floor_support(&mut self) {
        let src = self.supp_src_rect;
        let supp = self.supp_src_rect;
        let lrd = self.local_rooms_dest;
        let central = lrd[CENTRAL_ROOM as usize];

        // (structure to test, left edge, top edge), in the C's order
        let places = [
            (NORTH_WEST_ROOM, lrd[WEST_ROOM as usize].left, central.top - supp.bottom), // left room's ceiling
            (WEST_ROOM, lrd[WEST_ROOM as usize].left, central.bottom),                  // left room's floor
            (NORTH_ROOM, central.left, central.top - supp.bottom),                      // directly above main room
            (CENTRAL_ROOM, central.left, central.bottom),                               // directly below main room
            (NORTH_EAST_ROOM, lrd[EAST_ROOM as usize].left, central.top - supp.bottom),
            (EAST_ROOM, lrd[EAST_ROOM as usize].left, central.bottom),
        ];

        for (which, h, v) in places {
            if self.is_structure[which as usize] {
                let mut dest = supp;
                q_offset_rect(&mut dest, h, v);
                self.back_src_map.copy_bits(&self.supp_src_map, src, dest);

                for i in 0..self.num_temp_manholes as usize {
                    let mut who_cares = Rect::default();
                    if sect_rect(&dest, &self.temp_manholes[i], &mut who_cares) {
                        self.temp_manholes[i].top = dest.top;
                        self.temp_manholes[i].bottom = dest.bottom;
                        // LoadScaledGraphic(kManholeThruFloor, &tempManholes[i])
                        let r = self.temp_manholes[i];
                        match self.picture(MANHOLE_THRU_FLOOR) {
                            Some(p) => self.back_src_map.draw_picture(&p, r),
                            None => log::warn!("PICT {MANHOLE_THRU_FLOOR} missing"),
                        }
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  ReadyBackMap

    pub fn ready_back_map(&mut self) {
        let r = self.work_src_rect;
        self.back_src_map.copy_bits(&self.work_src_map, r, r);
    }

    //--------------------------------------------------------------  RestoreWorkMap

    pub fn restore_work_map(&mut self) {
        let r = self.back_src_rect;
        self.work_src_map.copy_bits(&self.back_src_map, r, r);
    }

    //--------------------------------------------------------------  ReadyLevel

    pub fn ready_level(&mut self) {
        self.nil_saved_maps();

        // (stopping the TV's QuickTime movie is left out)

        self.determine_room_openings();
        self.draw_locale();
        self.init_garbage_rects();
    }

    //--------------------------------------------------------------  DrawLighting

    pub fn draw_lighting(&mut self) {
        if self.num_lights == 0 {
            #[allow(clippy::needless_return)]
            return;
        } else {
            // for future construction
        }
    }

    //--------------------------------------------------------------  RedrawRoomLighting

    pub fn redraw_room_lighting(&mut self) {
        let room_v = self.house.rooms[self.this_room_number as usize].floor;

        let was_lit = self.num_lights > 0;
        self.num_lights = self.get_number_of_lights(self.local_numbers[CENTRAL_ROOM as usize]);
        let is_lit = self.num_lights > 0;
        if was_lit != is_lit {
            self.draw_room_background(self.local_numbers[CENTRAL_ROOM as usize], CENTRAL_ROOM, room_v);
            self.draw_a_rooms_objects(CENTRAL_ROOM, true);
            self.draw_lighting();
            let (central, lights) = (self.local_numbers[CENTRAL_ROOM as usize], self.num_lights);
            self.update_outlets_lighting(central, lights);

            if self.num_neighbors > 3 {
                self.draw_floor_support();
            }
            self.restore_work_map();
            let r = self.local_rooms_dest[CENTRAL_ROOM as usize];
            self.add_rect_to_work_rects(&r);
            self.shadow_visible = self.is_shadow_visible();
        }
    }
}

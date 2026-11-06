//! StructuresInit2.c: more source maps, the work and back maps, the
//! arrays, every object's source rectangle; and the part of
//! InterfaceInit.c (VariableInit) that sets up the house's rectangles.

use super::*;

pub const ANGEL_PICT_ID: i32 = 1019;
pub const SUPPORT_PICT_ID: i32 = 1999;
pub const CLUTTER_PICT_ID: i32 = 4018;

impl Game {
    //--------------------------------------------------------------  InitScreenRects
    // From InterfaceInit.c's VariableInit: houseRect, the play origin and
    // the nine rooms' places on the screen.

    pub fn init_screen_rects(&mut self) {
        // the rest of VariableInit, before the rectangles
        self.shield_pt = Point::new(0, 0);
        self.shield_rect = self.screen;
        self.quitting = false;
        self.playing = false;
        self.even_frame = false;
        self.two_player_game = false;
        self.paused = false;
        self.has_mirror = false;
        self.demo_going = false;
        self.gliders[0].which = PLAYER1;
        self.gliders[1].which = PLAYER2;
        self.the_mode = SPLASH_MODE;
        self.this_room_number = 0;
        self.previous_room = -1;
        self.last_background = BASE_BACKGROUND_ID;
        self.last_high_score = -1;
        self.increment_mode_time = self.ticks + IDLE_SPLASH_TICKS;
        self.fade_in_sequence = [4, 5, 6, 7, 5, 6, 7, 8, 6, 7, 8, 9, 7, 8, 9, 10];
        self.mirror_rgn = None;

        let screen = self.screen;
        self.house_rect = screen;
        self.house_rect.bottom -= SCOREBOARD_TALL;
        if self.house_rect.right > MAX_VIEW_WIDTH {
            self.house_rect.right = MAX_VIEW_WIDTH;
        }
        if self.house_rect.bottom > MAX_VIEW_HEIGHT {
            self.house_rect.bottom = MAX_VIEW_HEIGHT;
        }

        self.play_origin_h = (rect_wide(&screen) - ROOM_WIDE) / 2;
        self.play_origin_v = (rect_tall(&screen) - TILE_HIGH) / 2;

        for i in 0..9 {
            q_set_rect(&mut self.local_rooms_dest[i], 0, 0, ROOM_WIDE, TILE_HIGH);
            q_offset_rect(&mut self.local_rooms_dest[i], self.play_origin_h, self.play_origin_v);
        }
        let d = &mut self.local_rooms_dest;
        q_offset_rect(&mut d[NORTH_ROOM as usize], 0, -VERT_LOCAL_OFFSET);
        q_offset_rect(&mut d[NORTH_EAST_ROOM as usize], ROOM_WIDE, -VERT_LOCAL_OFFSET);
        q_offset_rect(&mut d[EAST_ROOM as usize], ROOM_WIDE, 0);
        q_offset_rect(&mut d[SOUTH_EAST_ROOM as usize], ROOM_WIDE, VERT_LOCAL_OFFSET);
        q_offset_rect(&mut d[SOUTH_ROOM as usize], 0, VERT_LOCAL_OFFSET);
        q_offset_rect(&mut d[SOUTH_WEST_ROOM as usize], -ROOM_WIDE, VERT_LOCAL_OFFSET);
        q_offset_rect(&mut d[WEST_ROOM as usize], -ROOM_WIDE, 0);
        q_offset_rect(&mut d[NORTH_WEST_ROOM as usize], -ROOM_WIDE, -VERT_LOCAL_OFFSET);
    }

    //--------------------------------------------------------------  InitClutter
    // Structures and graphics relating to clutter are loaded up.
    // Clutter includes mirrors, teddy bears, fireplaces, calendars, etc.

    pub fn init_clutter(&mut self) {
        q_set_rect(&mut self.clutter_src_rect, 0, 0, 128, 69);
        self.clutter_src_map = self.graphic_map(CLUTTER_PICT_ID, self.clutter_src_rect);
        self.clutter_mask_map = self.graphic_map(CLUTTER_PICT_ID + 1000, self.clutter_src_rect);

        let f = &mut self.flower_src;
        q_set_rect(&mut f[0], 0, 0, 10, 28);
        q_offset_rect(&mut f[0], 0, 23);

        q_set_rect(&mut f[1], 0, 0, 24, 35);
        q_offset_rect(&mut f[1], 10, 16);

        q_set_rect(&mut f[2], 0, 0, 34, 35);
        q_offset_rect(&mut f[2], 34, 16);

        q_set_rect(&mut f[3], 0, 0, 27, 23);
        q_offset_rect(&mut f[3], 68, 14);

        q_set_rect(&mut f[4], 0, 0, 27, 14);
        q_offset_rect(&mut f[4], 68, 37);

        q_set_rect(&mut f[5], 0, 0, 32, 51);
        q_offset_rect(&mut f[5], 95, 0);
    }

    //--------------------------------------------------------------  InitSupport
    // The floor support grphic is loaded up.  It is only visible when
    // playing in 9-room mode.  It is the horizontal wooden beam that
    // seperates floors from one another.

    pub fn init_support(&mut self) {
        q_set_rect(&mut self.supp_src_rect, 0, 0, ROOM_WIDE, FLOOR_SUPPORT_TALL); // 44
        self.supp_src_map = self.graphic_map(SUPPORT_PICT_ID, self.supp_src_rect);
    }

    //--------------------------------------------------------------  InitAngel
    // This loads the graphic of the girl riding the glider.  It is seen
    // only upon completing a house.  She flies across the screen dropping
    // stars below.

    pub fn init_angel(&mut self) {
        q_set_rect(&mut self.angel_src_rect, 0, 0, 96, 44);
        self.angel_src_map = self.graphic_map(ANGEL_PICT_ID, self.angel_src_rect);
        self.angel_mask_map = self.graphic_map(ANGEL_PICT_ID + 1, self.angel_src_rect);
    }

    //--------------------------------------------------------------  CreateOffscreens
    // All "utility" or "work" offscreen pix/bit maps are created here.
    // These would be offscreens that are reused throughout a game - they
    // are not static (mere repositories for graphics).  Most are used
    // to facilitate the animation when a game is in progress.

    pub fn create_offscreens(&mut self) {
        self.just_rooms_rect = self.house_rect;
        zero_rect_corner(&mut self.just_rooms_rect);

        self.work_src_rect = self.house_rect; // Set up work map
        zero_rect_corner(&mut self.work_src_rect);
        self.work_src_map = Map::new(self.work_src_rect);

        self.back_src_rect = self.house_rect; // Set up background map
        zero_rect_corner(&mut self.back_src_rect);
        self.back_src_map = Map::new(self.back_src_rect);

        self.init_scoreboard_map();
        self.init_glider_map();
        self.init_blowers();
        self.init_furniture();
        self.init_prizes();
        self.init_transports();
        self.init_switches();
        self.init_lights();
        self.init_appliances();
        self.init_enemies();
        self.init_clutter();
        self.init_support();
        self.init_angel();

        // tileSrcRect (0, 0, 128, 80) and tileSrcMap (nil): unused.
    }

    //--------------------------------------------------------------  CreatePointers
    // This function allocates other large structures.  Pointers to hold
    // large arrays, etc.

    pub fn create_pointers(&mut self) {
        // thisRoom is a `Room` held in Game; theHousesSpecs (the house
        // files) are not kept here.
        self.hot_spots = vec![HotObject::default(); MAX_HOT_SPOTS as usize];
        self.sparkles = vec![Sparkle::default(); MAX_SPARKLES as usize];
        self.flying_points = vec![FlyingPt::default(); MAX_FLYING_PTS as usize];
        self.flames = vec![Flame::default(); MAX_CANDLES as usize];
        self.tiki_flames = vec![Flame::default(); MAX_TIKIS as usize];
        self.bbq_coals = vec![Flame::default(); MAX_COALS as usize];
        self.pendulums = vec![Pendulum::default(); MAX_PENDULUMS as usize];
        self.saved_maps = vec![SavedType::default(); MAX_SAVED_MAPS as usize];
        for m in self.saved_maps.iter_mut() {
            m.map = None;
        }
        self.bands = vec![Band::default(); MAX_RUBBER_BANDS as usize];
        self.grease = vec![Grease::default(); MAX_GREASE as usize];
        self.the_stars = vec![Star::default(); MAX_STARS as usize];
        self.shreds = vec![Shred::default(); MAX_SHREDDED as usize];
        self.dinahs = vec![Dyna::default(); MAX_DYNAMIC_OBS as usize];
        self.master_objects = vec![ObjData::default(); MAX_MASTER_OBJECTS as usize];
        self.src_rects = vec![Rect::default(); NUM_SRC_RECTS as usize];

        // demoData: the 'demo' 128 resource, kDemoLength bytes of
        // demoType (long frame, char key, char padding).
        self.demo_data = Vec::new();
        let data = self.assets.as_ref().and_then(|a| a.app.get(&(*b"demo", 128)).cloned());
        match data {
            Some(v) => {
                let n = (DEMO_LENGTH as usize).min(v.len());
                for c in v[..n].chunks_exact(6) {
                    self.demo_data.push(DemoKey { frame: i32::from_be_bytes([c[0], c[1], c[2], c[3]]), key: c[4] });
                }
            }
            None => log::warn!("CreatePointers: 'demo' 128 missing"),
        }
    }

    //--------------------------------------------------------------  InitSrcRects

    // This is a nasty, ugly function that initializes all global rectangles
    // used in Glider PRO.

    pub fn init_src_rects(&mut self) {
        let s = &mut self.src_rects;
        let mut set = |what: i32, l: i32, t: i32, r: i32, b: i32| q_set_rect(&mut s[what as usize], l, t, r, b);
        // Blowers
        set(FLOOR_VENT, 0, 0, 48, 11);
        set(CEILING_VENT, 0, 0, 48, 11);
        set(FLOOR_BLOWER, 0, 0, 48, 15);
        set(CEILING_BLOWER, 0, 0, 48, 15);
        set(SEWER_GRATE, 0, 0, 48, 17);
        set(LEFT_FAN, 0, 0, 40, 55);
        set(RIGHT_FAN, 0, 0, 40, 55);
        set(TAPER, 0, 0, 20, 59);
        set(CANDLE, 0, 0, 32, 30);
        set(STUBBY, 0, 0, 20, 36);
        set(TIKI, 0, 0, 27, 28);
        set(BBQ, 0, 0, 64, 33);
        set(INVIS_BLOWER, 0, 0, 24, 24);
        set(GRECO_VENT, 0, 0, 48, 18);
        set(SEWER_BLOWER, 0, 0, 32, 12);
        set(LIFT_AREA, 0, 0, 64, 32);
        // Furniture
        set(TABLE, 0, 0, 64, TABLE_THICK);
        set(SHELF, 0, 0, 64, SHELF_THICK);
        set(CABINET, 0, 0, 64, 64);
        set(FILING_CABINET, 0, 0, 74, 107);
        set(WASTE_BASKET, 0, 0, 64, 61);
        set(MILK_CRATE, 0, 0, 64, 58);
        set(COUNTER, 0, 0, 128, 64);
        set(DRESSER, 0, 0, 128, 64);
        set(DECK_TABLE, 0, 0, 64, TABLE_THICK);
        set(STOOL, 0, 0, 48, 38);
        set(TRUNK, 0, 0, 144, 80);
        set(INVIS_OBSTACLE, 0, 0, 64, 64);
        set(MANHOLE, 0, 0, 123, 22);
        set(BOOKS, 0, 0, 64, 51);
        set(INVIS_BOUNCE, 0, 0, 64, 64);
        // Prizes
        set(RED_CLOCK, 0, 0, 28, 17);
        set(BLUE_CLOCK, 0, 0, 28, 25);
        set(YELLOW_CLOCK, 0, 0, 28, 28);
        set(CUCKOO, 0, 0, 40, 80);
        set(PAPER, 0, 0, 48, 21);
        set(BATTERY, 0, 0, 16, 25);
        set(BANDS, 0, 0, 28, 23);
        set(GREASE_RT, 0, 0, 32, 27);
        set(GREASE_LF, 0, 0, 32, 27);
        set(FOIL, 0, 0, 55, 15);
        set(INVIS_BONUS, 0, 0, 24, 24);
        set(STAR, 0, 0, 32, 31);
        set(SPARKLE, 0, 0, 20, 19);
        set(HELIUM, 0, 0, 56, 16);
        set(SLIDER, 0, 0, 64, 16);
        // Transport
        set(UP_STAIRS, 0, 0, 160, 267);
        set(DOWN_STAIRS, 0, 0, 160, 267);
        set(MAILBOX_LF, 0, 0, 94, 80);
        set(MAILBOX_RT, 0, 0, 94, 80);
        set(FLOOR_TRANS, 0, 0, 56, 15);
        set(CEILING_TRANS, 0, 0, 56, 15);
        set(DOOR_IN_LF, 0, 0, 144, 322);
        set(DOOR_IN_RT, 0, 0, 144, 322);
        set(DOOR_EX_RT, 0, 0, 16, 322);
        set(DOOR_EX_LF, 0, 0, 16, 322);
        set(WINDOW_IN_LF, 0, 0, 20, 170);
        set(WINDOW_IN_RT, 0, 0, 20, 170);
        set(WINDOW_EX_RT, 0, 0, 16, 170);
        set(WINDOW_EX_LF, 0, 0, 16, 170);
        set(INVIS_TRANS, 0, 0, 64, 32);
        set(DELUXE_TRANS, 0, 0, 64, 64);
        // Switch
        set(LIGHT_SWITCH, 0, 0, 15, 24);
        set(MACHINE_SWITCH, 0, 0, 16, 24);
        set(THERMOSTAT, 0, 0, 15, 24);
        set(POWER_SWITCH, 0, 0, 8, 8);
        set(KNIFE_SWITCH, 0, 0, 16, 24);
        set(INVIS_SWITCH, 0, 0, 12, 12);
        set(TRIGGER, 0, 0, 12, 12);
        set(LG_TRIGGER, 0, 0, 48, 48);
        set(SOUND_TRIGGER, 0, 0, 32, 32);
        // Lights
        set(CEILING_LIGHT, 0, 0, 64, 20);
        set(LIGHT_BULB, 0, 0, 16, 28);
        set(TABLE_LAMP, 0, 0, 48, 70);
        set(HIP_LAMP, 0, 0, 72, 276);
        set(DECO_LAMP, 0, 0, 64, 212);
        set(FLOURESCENT, 0, 0, 64, 12);
        set(TRACK_LIGHT, 0, 0, 64, 24);
        set(INVIS_LIGHT, 0, 0, 16, 16);
        // Appliances
        set(SHREDDER, 0, 0, 73, 22);
        set(TOASTER, 0, 0, 48, 27);
        set(MAC_PLUS, 0, 0, 48, 58);
        set(GUITAR, 0, 0, 64, 172);
        set(TV, 0, 0, 92, 77);
        set(COFFEE, 0, 0, 43, 64);
        set(OUTLET, 0, 0, 16, 24);
        set(VCR, 0, 0, 96, 22);
        set(STEREO, 0, 0, 128, 53);
        set(MICROWAVE, 0, 0, 92, 59);
        set(CINDER_BLOCK, 0, 0, 40, 62);
        set(FLOWER_BOX, 0, 0, 80, 32);
        set(C_DS, 0, 0, 16, 30);
        set(CUSTOM_PICT, 0, 0, 72, 34);
        // Enemies
        set(BALLOON, 0, 0, 24, 30);
        set(COPTER_LF, 0, 0, 32, 30);
        set(COPTER_RT, 0, 0, 32, 30);
        set(DART_LF, 0, 0, 64, 19);
        set(DART_RT, 0, 0, 64, 19);
        set(BALL, 0, 0, 32, 32);
        set(DRIP, 0, 0, 16, 12);
        set(FISH, 0, 0, 36, 33);
        set(COBWEB, 0, 0, 54, 45);
        // Clutter
        set(OZMA, 0, 0, 102, 92);
        set(MIRROR, 0, 0, 64, 64);
        set(MOUSEHOLE, 0, 0, 10, 11);
        set(FIREPLACE, 0, 0, 180, 142);
        set(WALL_WINDOW, 0, 0, 64, 80);
        set(BEAR, 0, 0, 56, 58);
        set(CALENDAR, 0, 0, 63, 92);
        set(VASE1, 0, 0, 36, 45);
        set(VASE2, 0, 0, 35, 57);
        set(BULLETIN, 0, 0, 80, 58);
        set(CLOUD, 0, 0, 128, 30);
        set(FAUCET, 0, 0, 56, 18);
        set(RUG, 0, 0, 144, 18);
        set(CHIMES, 0, 0, 28, 74);

        // The offsets, in the C's order among the sets above (each
        // QOffsetRect follows its own QSetRect, so doing them after is the
        // same).
        let mut off = |what: i32, h: i32, v: i32| q_offset_rect(&mut self.src_rects[what as usize], h, v);
        off(FLOOR_VENT, 0, 0);
        off(CEILING_VENT, 0, 11);
        off(FLOOR_BLOWER, 0, 22);
        off(CEILING_BLOWER, 0, 37);
        off(SEWER_GRATE, 0, 52);
        off(LEFT_FAN, 0, 69);
        off(RIGHT_FAN, 0, 124);
        off(TAPER, 0, 209);
        off(CANDLE, 0, 179);
        off(STUBBY, 0, 268);
        off(TIKI, 21, 268);
        off(GRECO_VENT, 0, 340);
        off(SEWER_BLOWER, 0, 390);

        off(WASTE_BASKET, 0, 43);
        off(MILK_CRATE, 0, 104);
        off(STOOL, 0, 183);

        off(BLUE_CLOCK, 0, 17);
        off(YELLOW_CLOCK, 0, 42);
        off(CUCKOO, 0, 148);
        off(PAPER, 0, 127);
        off(BATTERY, 32, 0);
        off(BANDS, 20, 70);
        off(GREASE_RT, 0, 243);
        off(GREASE_LF, 0, 324);
        off(FOIL, 0, 228);
        off(STAR, 48, 0);
        off(SPARKLE, 0, 70);
        off(HELIUM, 32, 270);

        off(FLOOR_TRANS, 0, 1);
        off(CEILING_TRANS, 0, 16);

        off(MACHINE_SWITCH, 0, 48);
        off(THERMOSTAT, 0, 48);
        off(POWER_SWITCH, 0, 72);
        off(KNIFE_SWITCH, 0, 80);

        off(CEILING_LIGHT, 0, 0);
        off(LIGHT_BULB, 0, 20);
        off(TABLE_LAMP, 16, 20);

        off(TOASTER, 0, 22);
        off(MAC_PLUS, 0, 49);
        off(COFFEE, 0, 107);
        off(OUTLET, 64, 22);
        off(C_DS, 48, 22);

        off(FAUCET, 0, 51);
    }
}

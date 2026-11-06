//! StructuresInit.c: the source maps (graphics) and their rectangles.

use super::*;

pub const SHADOW_PICT_ID: i32 = 3998;
pub const BLOWER_PICT_ID: i32 = 4000;
pub const FURNITURE_PICT_ID: i32 = 4001;
pub const BONUS_PICT_ID: i32 = 4002;
pub const SWITCH_PICT_ID: i32 = 4003;
pub const LIGHT_PICT_ID: i32 = 4004;
pub const APPLIANCE_PICT_ID: i32 = 4005;
pub const POINTS_PICT_ID: i32 = 4006;
pub const RUBBER_BANDS_PICT_ID: i32 = 4007;
pub const TRANSPORT_PICT_ID: i32 = 4008;
pub const TOAST_PICT_ID: i32 = 4009;
pub const SHREDDED_PICT_ID: i32 = 4010;
pub const BALLOON_PICT_ID: i32 = 4011;
pub const COPTER_PICT_ID: i32 = 4012;
pub const DART_PICT_ID: i32 = 4013;
pub const BALL_PICT_ID: i32 = 4014;
pub const DRIP_PICT_ID: i32 = 4015;
pub const ENEMY_PICT_ID: i32 = 4016;
pub const FISH_PICT_ID: i32 = 4017;

pub const BADGE_PICT_ID: i32 = 1996;

impl Game {
    /// CreateOffScreenGWorld(&map, &r) + LoadGraphic(id): a white map of
    /// `r` with the picture drawn at its own size from (0, 0), as
    /// LoadGraphic does (no stretching).
    pub fn graphic_map(&self, id: i32, r: Rect) -> Map {
        let mut m = Map::new(r);
        match self.picture(id) {
            Some(img) => m.draw_picture(&img, Rect::new(0, 0, img.w, img.h)),
            None => log::warn!("LoadGraphic: PICT {id} missing"),
        }
        m
    }

    //--------------------------------------------------------------  InitScoreboardMap
    // Any graphics and structures relating to the scoreboard that appears
    // across the top of the game are initialized and loaded up here.

    pub fn init_scoreboard_map(&mut self) {
        self.was_scoreboard_mode = SCOREBOARD_HIGH;
        self.board_src_rect = self.house_rect;
        zero_rect_corner(&mut self.board_src_rect);
        self.board_src_rect.bottom = SCOREBOARD_TALL;
        self.board_src_map = Map::new(self.board_src_rect);

        let mut h_offset = if self.board_src_rect.right >= 640 {
            (rect_wide(&self.board_src_rect) - MAX_VIEW_WIDTH) / 2
        } else {
            -576
        };
        match self.picture(SCOREBOARD_PICT_ID) {
            Some(the_picture) => {
                let mut bounds = Rect::new(0, 0, the_picture.w, the_picture.h);
                q_offset_rect(&mut bounds, h_offset, 0);
                self.board_src_map.draw_picture(&the_picture, bounds);
            }
            None => log::warn!("InitScoreboardMap: PICT {SCOREBOARD_PICT_ID} missing"),
        }

        q_set_rect(&mut self.badge_src_rect, 0, 0, 32, 66); // 2144 pixels
        self.badge_src_map = self.graphic_map(BADGE_PICT_ID, self.badge_src_rect);

        self.board_dest_rect = self.board_src_rect;
        q_offset_rect(&mut self.board_dest_rect, 0, -SCOREBOARD_TALL);

        h_offset = (rect_wide(&self.house_rect) - 640) / 2;
        if h_offset < 0 {
            h_offset = -128;
        }

        // (The text maps' fonts, applFont 12 bold, are not kept: text is
        // terminal text.)
        q_set_rect(&mut self.board_t_src_rect, 0, 0, 256, 12); // room title
        self.board_t_src_map = Map::new(self.board_t_src_rect);
        self.board_t_dest_rect = self.board_t_src_rect;
        q_offset_rect(&mut self.board_t_dest_rect, 137 + h_offset, 5);

        q_set_rect(&mut self.board_g_src_rect, 0, 0, 20, 10); // # gliders
        self.board_g_src_map = Map::new(self.board_g_src_rect);
        self.board_g_dest_rect = self.board_g_src_rect;
        q_offset_rect(&mut self.board_g_dest_rect, 526 + h_offset, 5);

        q_set_rect(&mut self.board_p_src_rect, 0, 0, 64, 10); // points
        self.board_p_src_map = Map::new(self.board_p_src_rect);
        self.board_p_dest_rect = self.board_p_src_rect;
        q_offset_rect(&mut self.board_p_dest_rect, 570 + h_offset, 5); // total = 6396 pixels
        self.board_pq_dest_rect = self.board_p_dest_rect;
        q_offset_rect(&mut self.board_pq_dest_rect, 0, -SCOREBOARD_TALL);
        self.board_gq_dest_rect = self.board_g_dest_rect;
        q_offset_rect(&mut self.board_gq_dest_rect, 0, -SCOREBOARD_TALL);

        let r = &mut self.badges_blank_rects;
        q_set_rect(&mut r[0], 0, 0, 16, 16); // foil
        q_offset_rect(&mut r[0], 0, 0);
        q_set_rect(&mut r[1], 0, 0, 16, 16); // rubber bands
        q_offset_rect(&mut r[1], 0, 16);
        q_set_rect(&mut r[2], 0, 0, 16, 17); // battery
        q_offset_rect(&mut r[2], 0, 32);
        q_set_rect(&mut r[3], 0, 0, 16, 17); // helium
        q_offset_rect(&mut r[3], 0, 49);

        let r = &mut self.badges_badges_rects;
        q_set_rect(&mut r[0], 0, 0, 16, 16); // foil
        q_offset_rect(&mut r[0], 16, 0);
        q_set_rect(&mut r[1], 0, 0, 16, 16); // rubber bands
        q_offset_rect(&mut r[1], 16, 16);
        q_set_rect(&mut r[2], 0, 0, 16, 17); // battery
        q_offset_rect(&mut r[2], 16, 32);
        q_set_rect(&mut r[3], 0, 0, 16, 17); // helium
        q_offset_rect(&mut r[3], 16, 49);

        let r = &mut self.badges_dest_rects;
        q_set_rect(&mut r[0], 0, 0, 16, 16); // foil
        q_offset_rect(&mut r[0], 432 + h_offset, 2 - SCOREBOARD_TALL);
        q_set_rect(&mut r[1], 0, 0, 16, 16); // rubber bands
        q_offset_rect(&mut r[1], 449 + h_offset, 2 - SCOREBOARD_TALL);
        q_set_rect(&mut r[2], 0, 0, 16, 17); // battery
        q_offset_rect(&mut r[2], 467 + h_offset, 1 - SCOREBOARD_TALL);
        q_set_rect(&mut r[3], 0, 0, 16, 17); // helium
        q_offset_rect(&mut r[3], 467 + h_offset, 1 - SCOREBOARD_TALL);
    }

    //--------------------------------------------------------------  InitGliderMap
    // Graphics and structures relating to the little paper glider (the
    // player) are cretaed, loaded up and initialized here.

    pub fn init_glider_map(&mut self) {
        q_set_rect(&mut self.glid_src_rect, 0, 0, GLIDER_WIDE, 668); // 32112 pixels
        self.glid_src_map = self.graphic_map(GLIDER_PICT_ID, self.glid_src_rect);
        self.glid2_src_map = self.graphic_map(GLIDER2_PICT_ID, self.glid_src_rect);
        self.glid_mask_map = self.graphic_map(GLIDER_PICT_ID + 1000, self.glid_src_rect);

        for i in 0..=20 {
            q_set_rect(&mut self.glider_src[i], 0, 0, GLIDER_WIDE, GLIDER_HIGH);
            q_offset_rect(&mut self.glider_src[i], 0, GLIDER_HIGH * i as i32);
        }
        for i in 21..=28 {
            q_set_rect(&mut self.glider_src[i], 0, 0, GLIDER_WIDE, GLIDER_BURNING_HIGH);
            q_offset_rect(&mut self.glider_src[i], 0, 420 + (GLIDER_BURNING_HIGH * (i as i32 - 21)));
        }

        q_set_rect(&mut self.glider_src[29], 0, 0, GLIDER_WIDE, GLIDER_HIGH);
        q_offset_rect(&mut self.glider_src[29], 0, 628);
        q_set_rect(&mut self.glider_src[30], 0, 0, GLIDER_WIDE, GLIDER_HIGH);
        q_offset_rect(&mut self.glider_src[30], 0, 648);

        q_set_rect(&mut self.shadow_src_rect, 0, 0, GLIDER_WIDE, SHADOW_HIGH * NUM_SHADOW_SRC_RECTS);
        self.shadow_src_map = self.graphic_map(SHADOW_PICT_ID, self.shadow_src_rect);
        self.shadow_mask_map = self.graphic_map(SHADOW_PICT_ID + 1000, self.shadow_src_rect);

        for i in 0..NUM_SHADOW_SRC_RECTS as usize {
            q_set_rect(&mut self.shadow_src[i], 0, 0, GLIDER_WIDE, SHADOW_HIGH);
            q_offset_rect(&mut self.shadow_src[i], 0, SHADOW_HIGH * i as i32);
        }

        q_set_rect(&mut self.bands_src_rect, 0, 0, 16, 18); // 304 pixels
        self.bands_src_map = self.graphic_map(RUBBER_BANDS_PICT_ID, self.bands_src_rect);
        self.bands_mask_map = self.graphic_map(RUBBER_BANDS_PICT_ID + 1000, self.bands_src_rect);

        for i in 0..3 {
            q_set_rect(&mut self.band_rects[i], 0, 0, 16, 6);
            q_offset_rect(&mut self.band_rects[i], 0, 6 * i as i32);
        }
    }

    //--------------------------------------------------------------  InitBlowers
    // All blower graphics and structures are loaded up and initialized here.
    // Blowers include vents, ducts, candles, fans, etc.

    pub fn init_blowers(&mut self) {
        q_set_rect(&mut self.blower_src_rect, 0, 0, 48, 402); // 19344 pixels
        self.blower_src_map = self.graphic_map(BLOWER_PICT_ID, self.blower_src_rect);
        self.blower_mask_map = self.graphic_map(BLOWER_PICT_ID + 1000, self.blower_src_rect);

        for i in 0..NUM_CANDLE_FLAMES as usize {
            q_set_rect(&mut self.flame[i], 0, 0, 16, 15);
            q_offset_rect(&mut self.flame[i], 32, 179 + (i as i32 * 15));
        }

        for i in 0..NUM_TIKI_FLAMES as usize {
            q_set_rect(&mut self.tiki_flame[i], 0, 0, 8, 10);
            q_offset_rect(&mut self.tiki_flame[i], 40, 69 + (i as i32 * 10));
        }

        for i in 0..NUM_BBQ_COALS as usize {
            q_set_rect(&mut self.coals[i], 0, 0, 32, 9);
            q_offset_rect(&mut self.coals[i], 0, 304 + (i as i32 * 9));
        }

        // leftStartGliderSrc and rightStartGliderSrc (0, 358 and 0, 374,
        // 48 x 16) are the editor's: left out.
    }

    //--------------------------------------------------------------  InitFurniture
    // Structures and graphics relating to the furniture is loaded up.
    // Furniture includes tables, cabinets, shelves, etc.

    pub fn init_furniture(&mut self) {
        q_set_rect(&mut self.furniture_src_rect, 0, 0, 64, 278); // 17856 pixels
        self.furniture_src_map = self.graphic_map(FURNITURE_PICT_ID, self.furniture_src_rect);
        self.furniture_mask_map = self.graphic_map(FURNITURE_PICT_ID + 1000, self.furniture_src_rect);

        q_set_rect(&mut self.table_src, 0, 0, 64, 22);
        q_offset_rect(&mut self.table_src, 0, 0);

        q_set_rect(&mut self.shelf_src, 0, 0, 16, 21);
        q_offset_rect(&mut self.shelf_src, 0, 22);

        q_set_rect(&mut self.hinge_src, 0, 0, 4, 16);
        q_offset_rect(&mut self.hinge_src, 16, 22);

        q_set_rect(&mut self.handle_src, 0, 0, 4, 21);
        q_offset_rect(&mut self.handle_src, 20, 22);

        q_set_rect(&mut self.knob_src, 0, 0, 8, 8);
        q_offset_rect(&mut self.knob_src, 24, 22);

        q_set_rect(&mut self.left_foot_src, 0, 0, 16, 16);
        q_offset_rect(&mut self.left_foot_src, 32, 22);

        q_set_rect(&mut self.right_foot_src, 0, 0, 16, 16);
        q_offset_rect(&mut self.right_foot_src, 48, 22);

        q_set_rect(&mut self.deck_src, 0, 0, 64, 21);
        q_offset_rect(&mut self.deck_src, 0, 162);
    }

    //--------------------------------------------------------------  InitPrizes
    // Structures and graphics relating to the prizes (bonuses) are loaded up.
    // Prizes includes clocks, rubber bands, extra gliders, etc.

    pub fn init_prizes(&mut self) {
        q_set_rect(&mut self.bonus_src_rect, 0, 0, 88, 378); // 33264 pixels
        self.bonus_src_map = self.graphic_map(BONUS_PICT_ID, self.bonus_src_rect);
        self.bonus_mask_map = self.graphic_map(BONUS_PICT_ID + 1000, self.bonus_src_rect);

        for i in 0..11 {
            q_set_rect(&mut self.digits[i], 0, 0, 4, 6);
            q_offset_rect(&mut self.digits[i], 28, i as i32 * 6);
        }

        for i in 0..3 {
            q_set_rect(&mut self.pendulum_src[i], 0, 0, 32, 28);
            q_offset_rect(&mut self.pendulum_src[i], 56, 186 + (i as i32 * 28));
        }

        let g = &mut self.grease_src_rt;
        q_set_rect(&mut g[0], 0, 0, 32, 27);
        q_offset_rect(&mut g[0], 0, 243);
        q_set_rect(&mut g[1], 0, 0, 32, 27);
        q_offset_rect(&mut g[1], 0, 270);
        q_set_rect(&mut g[2], 0, 0, 32, 27);
        q_offset_rect(&mut g[2], 0, 297);
        q_set_rect(&mut g[3], 0, 0, 32, 27);
        q_offset_rect(&mut g[3], 32, 297);

        let g = &mut self.grease_src_lf;
        q_set_rect(&mut g[0], 0, 0, 32, 27);
        q_offset_rect(&mut g[0], 0, 324);
        q_set_rect(&mut g[1], 0, 0, 32, 27);
        q_offset_rect(&mut g[1], 32, 324);
        q_set_rect(&mut g[2], 0, 0, 32, 27);
        q_offset_rect(&mut g[2], 0, 351);
        q_set_rect(&mut g[3], 0, 0, 32, 27);
        q_offset_rect(&mut g[3], 32, 351);

        for i in 0..6 {
            q_set_rect(&mut self.star_src[i], 0, 0, 32, 31);
            q_offset_rect(&mut self.star_src[i], 48, i as i32 * 31);
        }

        for i in 0..3 {
            q_set_rect(&mut self.sparkle_src[i + 2], 0, 0, 20, 19);
            q_offset_rect(&mut self.sparkle_src[i + 2], 0, 70 + (i as i32 * 19));
        }
        self.sparkle_src[0] = self.sparkle_src[4];
        self.sparkle_src[1] = self.sparkle_src[3];

        q_set_rect(&mut self.points_src_rect, 0, 0, 24, 120); // 2880 pixels
        self.points_src_map = self.graphic_map(POINTS_PICT_ID, self.points_src_rect);
        self.points_mask_map = self.graphic_map(POINTS_PICT_ID + 1000, self.points_src_rect);

        for i in 0..15 {
            q_set_rect(&mut self.points_src[i], 0, 0, 24, 8);
            q_offset_rect(&mut self.points_src[i], 0, i as i32 * 8);
        }
    }

    //--------------------------------------------------------------  InitTransports
    // Structures and graphics relating to the transports is loaded up.
    // Transports includes transport ducts, mailboxes, etc.

    pub fn init_transports(&mut self) {
        q_set_rect(&mut self.trans_src_rect, 0, 0, 56, 32); // 1848 pixels
        self.trans_src_map = self.graphic_map(TRANSPORT_PICT_ID, self.trans_src_rect);
        self.trans_mask_map = self.graphic_map(TRANSPORT_PICT_ID + 1000, self.trans_src_rect);
    }

    //--------------------------------------------------------------  InitSwitches
    // Structures and graphics relating to switches are loaded up.
    // Switches includes triggers, light switches, etc.

    pub fn init_switches(&mut self) {
        q_set_rect(&mut self.switch_src_rect, 0, 0, 32, 104); // 3360 pixels
        self.switch_src_map = self.graphic_map(SWITCH_PICT_ID, self.switch_src_rect);

        q_set_rect(&mut self.light_switch_src[0], 0, 0, 15, 24);
        q_offset_rect(&mut self.light_switch_src[0], 0, 0);
        q_set_rect(&mut self.light_switch_src[1], 0, 0, 15, 24);
        q_offset_rect(&mut self.light_switch_src[1], 16, 0);

        q_set_rect(&mut self.machine_switch_src[0], 0, 0, 16, 24);
        q_offset_rect(&mut self.machine_switch_src[0], 0, 24);
        q_set_rect(&mut self.machine_switch_src[1], 0, 0, 16, 24);
        q_offset_rect(&mut self.machine_switch_src[1], 16, 24);

        q_set_rect(&mut self.thermostat_src[0], 0, 0, 15, 24);
        q_offset_rect(&mut self.thermostat_src[0], 0, 48);
        q_set_rect(&mut self.thermostat_src[1], 0, 0, 15, 24);
        q_offset_rect(&mut self.thermostat_src[1], 16, 48);

        q_set_rect(&mut self.power_src[0], 0, 0, 8, 8);
        q_offset_rect(&mut self.power_src[0], 0, 72);
        q_set_rect(&mut self.power_src[1], 0, 0, 8, 8);
        q_offset_rect(&mut self.power_src[1], 8, 72);

        q_set_rect(&mut self.knife_switch_src[0], 0, 0, 16, 24);
        q_offset_rect(&mut self.knife_switch_src[0], 0, 80);
        q_set_rect(&mut self.knife_switch_src[1], 0, 0, 16, 24);
        q_offset_rect(&mut self.knife_switch_src[1], 16, 80);
    }

    //--------------------------------------------------------------  InitLights
    // Structures and graphics relating to lights are loaded up.
    // Lights includes table lamps, flourescent lights, track lights, etc.

    pub fn init_lights(&mut self) {
        q_set_rect(&mut self.light_src_rect, 0, 0, 72, 126); // 9144 pixels
        self.light_src_map = self.graphic_map(LIGHT_PICT_ID, self.light_src_rect);
        self.light_mask_map = self.graphic_map(LIGHT_PICT_ID + 1000, self.light_src_rect);

        q_set_rect(&mut self.flourescent_src1, 0, 0, 16, 12);
        q_offset_rect(&mut self.flourescent_src1, 0, 78);

        q_set_rect(&mut self.flourescent_src2, 0, 0, 16, 12);
        q_offset_rect(&mut self.flourescent_src2, 0, 90);

        for i in 0..NUM_TRACK_LIGHTS as usize {
            q_set_rect(&mut self.track_light_src[i], 0, 0, 24, 24);
            q_offset_rect(&mut self.track_light_src[i], 24 * i as i32, 102);
        }
    }

    //--------------------------------------------------------------  InitAppliances
    // Structures and graphics relating to appliances are loaded up.
    // Appliances includes toasters, T.V.s, etc.

    pub fn init_appliances(&mut self) {
        q_set_rect(&mut self.appliance_src_rect, 0, 0, 80, 269); // 21600 pixels
        self.appliance_src_map = self.graphic_map(APPLIANCE_PICT_ID, self.appliance_src_rect);
        self.appliance_mask_map = self.graphic_map(APPLIANCE_PICT_ID + 1000, self.appliance_src_rect);

        q_set_rect(&mut self.toast_src_rect, 0, 0, 32, 174); // 5600 pixels
        self.toast_src_map = self.graphic_map(TOAST_PICT_ID, self.toast_src_rect);
        self.toast_mask_map = self.graphic_map(TOAST_PICT_ID + 1000, self.toast_src_rect);

        q_set_rect(&mut self.shred_src_rect, 0, 0, 40, 35); // 1440 pixels
        self.shred_src_map = self.graphic_map(SHREDDED_PICT_ID, self.shred_src_rect);
        self.shred_mask_map = self.graphic_map(SHREDDED_PICT_ID + 1000, self.shred_src_rect);

        q_set_rect(&mut self.plus_screen1, 0, 0, 32, 22);
        q_offset_rect(&mut self.plus_screen1, 48, 127);
        q_set_rect(&mut self.plus_screen2, 0, 0, 32, 22);
        q_offset_rect(&mut self.plus_screen2, 48, 149);

        q_set_rect(&mut self.tv_screen1, 0, 0, 64, 49);
        q_offset_rect(&mut self.tv_screen1, 0, 171);
        q_set_rect(&mut self.tv_screen2, 0, 0, 64, 49);
        q_offset_rect(&mut self.tv_screen2, 0, 220);

        q_set_rect(&mut self.coffee_light1, 0, 0, 8, 4);
        q_offset_rect(&mut self.coffee_light1, 72, 171);
        q_set_rect(&mut self.coffee_light2, 0, 0, 8, 4);
        q_offset_rect(&mut self.coffee_light2, 72, 175);

        for i in 0..NUM_OUTLET_PICTS as usize {
            q_set_rect(&mut self.outlet_src[i], 0, 0, 16, 24);
            q_offset_rect(&mut self.outlet_src[i], 64, 22 + (i as i32 * 24));
        }

        for i in 0..NUM_BREAD_PICTS as usize {
            q_set_rect(&mut self.bread_src[i], 0, 0, 32, 29);
            q_offset_rect(&mut self.bread_src[i], 0, i as i32 * 29);
        }

        q_set_rect(&mut self.vcr_time1, 0, 0, 16, 4);
        q_offset_rect(&mut self.vcr_time1, 64, 179);
        q_set_rect(&mut self.vcr_time2, 0, 0, 16, 4);
        q_offset_rect(&mut self.vcr_time2, 64, 183);

        q_set_rect(&mut self.stereo_light1, 0, 0, 4, 1);
        q_offset_rect(&mut self.stereo_light1, 68, 171);
        q_set_rect(&mut self.stereo_light2, 0, 0, 4, 1);
        q_offset_rect(&mut self.stereo_light2, 68, 172);

        q_set_rect(&mut self.micro_on, 0, 0, 16, 35);
        q_offset_rect(&mut self.micro_on, 64, 222);
        q_set_rect(&mut self.micro_off, 0, 0, 16, 35);
        q_offset_rect(&mut self.micro_off, 64, 187);
    }

    //--------------------------------------------------------------  InitEnemies
    // Structures and graphics relating to enemies are loaded up.
    // Enemies includes darts, balloons, fish, etc.

    pub fn init_enemies(&mut self) {
        q_set_rect(&mut self.balloon_src_rect, 0, 0, 24, 30 * NUM_BALLOON_FRAMES);
        self.balloon_src_map = self.graphic_map(BALLOON_PICT_ID, self.balloon_src_rect);
        self.balloon_mask_map = self.graphic_map(BALLOON_PICT_ID + 1000, self.balloon_src_rect);

        q_set_rect(&mut self.copter_src_rect, 0, 0, 32, 30 * NUM_COPTER_FRAMES);
        self.copter_src_map = self.graphic_map(COPTER_PICT_ID, self.copter_src_rect);
        self.copter_mask_map = self.graphic_map(COPTER_PICT_ID + 1000, self.copter_src_rect);

        q_set_rect(&mut self.dart_src_rect, 0, 0, 64, 19 * NUM_DART_FRAMES);
        self.dart_src_map = self.graphic_map(DART_PICT_ID, self.dart_src_rect);
        self.dart_mask_map = self.graphic_map(DART_PICT_ID + 1000, self.dart_src_rect);

        q_set_rect(&mut self.ball_src_rect, 0, 0, 32, 32 * NUM_BALL_FRAMES);
        self.ball_src_map = self.graphic_map(BALL_PICT_ID, self.ball_src_rect);
        self.ball_mask_map = self.graphic_map(BALL_PICT_ID + 1000, self.ball_src_rect);

        q_set_rect(&mut self.drip_src_rect, 0, 0, 16, 12 * NUM_DRIP_FRAMES);
        self.drip_src_map = self.graphic_map(DRIP_PICT_ID, self.drip_src_rect);
        self.drip_mask_map = self.graphic_map(DRIP_PICT_ID + 1000, self.drip_src_rect);

        q_set_rect(&mut self.enemy_src_rect, 0, 0, 36, 33);
        self.enemy_src_map = self.graphic_map(ENEMY_PICT_ID, self.enemy_src_rect);
        self.enemy_mask_map = self.graphic_map(ENEMY_PICT_ID + 1000, self.enemy_src_rect);

        q_set_rect(&mut self.fish_src_rect, 0, 0, 16, 16 * NUM_FISH_FRAMES);
        self.fish_src_map = self.graphic_map(FISH_PICT_ID, self.fish_src_rect);
        self.fish_mask_map = self.graphic_map(FISH_PICT_ID + 1000, self.fish_src_rect);

        for i in 0..NUM_BALLOON_FRAMES as usize {
            q_set_rect(&mut self.balloon_src[i], 0, 0, 24, 30);
            q_offset_rect(&mut self.balloon_src[i], 0, 30 * i as i32);
        }

        for i in 0..NUM_COPTER_FRAMES as usize {
            q_set_rect(&mut self.copter_src[i], 0, 0, 32, 30);
            q_offset_rect(&mut self.copter_src[i], 0, 30 * i as i32);
        }

        for i in 0..NUM_DART_FRAMES as usize {
            q_set_rect(&mut self.dart_src[i], 0, 0, 64, 19);
            q_offset_rect(&mut self.dart_src[i], 0, 19 * i as i32);
        }

        for i in 0..NUM_BALL_FRAMES as usize {
            q_set_rect(&mut self.ball_src[i], 0, 0, 32, 32);
            q_offset_rect(&mut self.ball_src[i], 0, 32 * i as i32);
        }

        for i in 0..NUM_DRIP_FRAMES as usize {
            q_set_rect(&mut self.drip_src[i], 0, 0, 16, 12);
            q_offset_rect(&mut self.drip_src[i], 0, 12 * i as i32);
        }

        for i in 0..NUM_FISH_FRAMES as usize {
            q_set_rect(&mut self.fish_src[i], 0, 0, 16, 16);
            q_offset_rect(&mut self.fish_src[i], 0, 16 * i as i32);
        }
    }
}

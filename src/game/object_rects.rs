//! ObjectRects.c: each object's rectangle, and the hot spots (active
//! rects) the objects of the current room set up.

use super::*;

pub const FLOOR_COLUMN_WIDE: i32 = 4;
pub const CEILING_COLUMN_WIDE: i32 = 24;
pub const FAN_COLUMN_THICK: i32 = 16;
pub const FAN_COLUMN_DOWN: i32 = 20;
pub const DEADLY_FLAME_HEIGHT: i32 = 24;
pub const STOOL_THICK: i32 = 25;
pub const SHREDDER_ACTIVE_HIGH: i32 = 40;

impl Game {
    //--------------------------------------------------------------  GetObjectRect

    /// `who` may be changed: a custom picture that is missing gets 10000 as
    /// its picture number, as in the C.
    pub fn get_object_rect(&self, who: &mut Object, its_rect: &mut Rect) {
        let sr = |w: i32| self.src_rects[w as usize];
        match who.what {
            OBJECT_IS_EMPTY => {
                q_set_rect(its_rect, 0, 0, 0, 0);
            }

            FLOOR_VENT | CEILING_VENT | FLOOR_BLOWER | CEILING_BLOWER | SEWER_GRATE | LEFT_FAN | RIGHT_FAN | TAPER
            | CANDLE | STUBBY | TIKI | BBQ | INVIS_BLOWER | GRECO_VENT | SEWER_BLOWER => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let a = who.a();
                q_offset_rect(its_rect, a.top_left.h, a.top_left.v);
            }

            LIFT_AREA => {
                let a = who.a();
                q_set_rect(its_rect, 0, 0, a.distance, a.tall * 2);
                q_offset_rect(its_rect, a.top_left.h, a.top_left.v);
                // (the C has three unreachable lines after this break)
            }

            TABLE | SHELF | CABINET | FILING_CABINET | WASTE_BASKET | MILK_CRATE | COUNTER | DRESSER | STOOL | TRUNK
            | DECK_TABLE | INVIS_OBSTACLE | MANHOLE | BOOKS | INVIS_BOUNCE => {
                *its_rect = who.b().bounds;
            }

            RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | CUCKOO | PAPER | BATTERY | BANDS | GREASE_RT | GREASE_LF | FOIL
            | INVIS_BONUS | STAR | SPARKLE | HELIUM => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let c = who.c();
                q_offset_rect(its_rect, c.top_left.h, c.top_left.v);
            }

            SLIDER => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let c = who.c();
                q_offset_rect(its_rect, c.top_left.h, c.top_left.v);
                its_rect.right = its_rect.left + c.length;
            }

            UP_STAIRS | DOWN_STAIRS | MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | DOOR_IN_LF | DOOR_IN_RT
            | DOOR_EX_RT | DOOR_EX_LF | WINDOW_IN_LF | WINDOW_IN_RT | WINDOW_EX_RT | WINDOW_EX_LF => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let d = who.d();
                q_offset_rect(its_rect, d.top_left.h, d.top_left.v);
            }

            INVIS_TRANS => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let d = who.d();
                q_offset_rect(its_rect, d.top_left.h, d.top_left.v);
                its_rect.bottom = its_rect.top + d.tall;
                its_rect.right += d.wide;
            }

            DELUXE_TRANS => {
                let d = who.d();
                let wide = (d.tall & 0xFF00) >> 8; // Get high byte
                let tall = d.tall & 0x00FF; // Get low byte
                q_set_rect(its_rect, 0, 0, wide * 4, tall * 4); // Scale by 4
                q_offset_rect(its_rect, d.top_left.h, d.top_left.v);
            }

            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
            | LG_TRIGGER | SOUND_TRIGGER => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let e = who.e();
                q_offset_rect(its_rect, e.top_left.h, e.top_left.v);
            }

            CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP | INVIS_LIGHT => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let f = who.f();
                q_offset_rect(its_rect, f.top_left.h, f.top_left.v);
            }

            FLOURESCENT | TRACK_LIGHT => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let f = who.f();
                its_rect.right = f.length;
                q_offset_rect(its_rect, f.top_left.h, f.top_left.v);
            }

            SHREDDER | TOASTER | MAC_PLUS | GUITAR | TV | COFFEE | OUTLET | VCR | STEREO | MICROWAVE | CINDER_BLOCK
            | FLOWER_BOX | C_DS => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let g = who.g();
                q_offset_rect(its_rect, g.top_left.h, g.top_left.v);
            }

            CUSTOM_PICT => {
                let mut g = who.g();
                match self.picture(g.height) {
                    None => {
                        g.height = 10000;
                        who.set_g(g);
                        *its_rect = sr(who.what);
                    }
                    Some(the_pict) => {
                        // picFrame (its corner is zeroed below)
                        *its_rect = Rect::new(0, 0, the_pict.w, the_pict.h);
                    }
                }
                zero_rect_corner(its_rect);
                q_offset_rect(its_rect, g.top_left.h, g.top_left.v);
            }

            BALLOON | COPTER_LF | COPTER_RT | DART_LF | DART_RT | BALL | DRIP | FISH | COBWEB => {
                *its_rect = sr(who.what);
                zero_rect_corner(its_rect);
                let h = who.h();
                q_offset_rect(its_rect, h.top_left.h, h.top_left.v);
            }

            OZMA | MIRROR | MOUSEHOLE | FIREPLACE | FLOWER | WALL_WINDOW | BEAR | CALENDAR | VASE1 | VASE2 | BULLETIN
            | CLOUD | FAUCET | RUG | CHIMES => {
                *its_rect = who.i().bounds;
            }

            _ => {}
        }
    }

    //--------------------------------------------------------------  AddActiveRect

    pub fn add_active_rect(&mut self, bounds: &Rect, action: i32, who: i32, is_on: bool, do_scrutinize: bool) -> i32 {
        if self.n_hot_spots >= MAX_HOT_SPOTS {
            return -1;
        }
        if self.hot_spots.len() < MAX_HOT_SPOTS as usize {
            self.hot_spots.resize(MAX_HOT_SPOTS as usize, HotObject::default());
        }

        let h = &mut self.hot_spots[self.n_hot_spots as usize];
        h.bounds = *bounds; // the active rect
        h.action = action; // what it does
        h.who = who; // local obj. linked to
        h.is_on = is_on; // is it active?
        h.still_over = false;
        h.do_scrutinize = do_scrutinize;
        self.n_hot_spots += 1;

        self.n_hot_spots - 1
    }

    //--------------------------------------------------------------  CreateActiveRects

    #[allow(unused_assignments)] // hotSpotNumber is overwritten as in the C
    pub fn create_active_rects(&mut self, who: i32) -> i32 {
        let mut bounds = Rect::default();
        let mut hot_spot_number = -1;
        let the_object = self.master_objects[who as usize].the_object;
        let sr = |s: &Self, w: i32| s.src_rects[w as usize];

        match the_object.what {
            OBJECT_IS_EMPTY => {}

            FLOOR_VENT | FLOOR_BLOWER | SEWER_GRATE | GRECO_VENT | SEWER_BLOWER => {
                // (five identical cases in the C)
                let a = the_object.a();
                q_set_rect(&mut bounds, 0, -a.distance, FLOOR_COLUMN_WIDE, 0);
                q_offset_rect(&mut bounds, half_rect_wide(&sr(self, the_object.what)) - FLOOR_COLUMN_WIDE / 2, 0);
                q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, LIFT_IT, who, a.state, false);
            }

            CEILING_VENT | CEILING_BLOWER => {
                let a = the_object.a();
                q_set_rect(&mut bounds, 0, 0, CEILING_COLUMN_WIDE, a.distance);
                q_offset_rect(&mut bounds, half_rect_wide(&sr(self, the_object.what)) - CEILING_COLUMN_WIDE / 2, 0);
                q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, DROP_IT, who, a.state, false);
            }

            LEFT_FAN => {
                let a = the_object.a();
                q_set_rect(&mut bounds, 0, 0, 13, 43);
                q_offset_rect(&mut bounds, a.top_left.h + 16, a.top_left.v + 12);
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
                q_set_rect(&mut bounds, 0, 0, a.distance, FAN_COLUMN_THICK);
                q_offset_rect(&mut bounds, -(a.distance), FAN_COLUMN_DOWN);
                q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, PUSH_IT_LEFT, who, a.state, false);
            }

            RIGHT_FAN => {
                let a = the_object.a();
                q_set_rect(&mut bounds, 0, 0, 13, 43);
                q_offset_rect(&mut bounds, a.top_left.h + 6, a.top_left.v + 12);
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
                q_set_rect(&mut bounds, 0, 0, a.distance, FAN_COLUMN_THICK);
                q_offset_rect(&mut bounds, rect_wide(&sr(self, RIGHT_FAN)), FAN_COLUMN_DOWN);
                q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, PUSH_IT_RIGHT, who, a.state, false);
            }

            TAPER | CANDLE | STUBBY | TIKI | BBQ => {
                let a = the_object.a();
                let what = the_object.what;
                // the column of hot air
                let bottom = if what == BBQ { 8 } else { 0 };
                q_set_rect(&mut bounds, 0, -a.distance, FLOOR_COLUMN_WIDE, bottom);
                let mut dh = half_rect_wide(&sr(self, what)) - FLOOR_COLUMN_WIDE / 2;
                if what == STUBBY {
                    dh -= 1;
                }
                q_offset_rect(&mut bounds, dh, 0);
                let left = if what == CANDLE { a.top_left.h - 2 } else { a.top_left.h };
                q_offset_rect(&mut bounds, left, a.top_left.v);
                if (bounds.bottom - bounds.top) > DEADLY_FLAME_HEIGHT {
                    bounds.bottom -= DEADLY_FLAME_HEIGHT;
                    hot_spot_number = self.add_active_rect(&bounds, LIFT_IT, who, true, false);
                    bounds.bottom += DEADLY_FLAME_HEIGHT;
                    bounds.top = bounds.bottom - DEADLY_FLAME_HEIGHT + 2;
                    hot_spot_number = self.add_active_rect(&bounds, BURN_IT, who, true, false);
                } else {
                    hot_spot_number = self.add_active_rect(&bounds, BURN_IT, who, true, false);
                }
                // the candle, tiki or grill itself
                let (w, t, dx, dy) = match what {
                    TAPER => (7, 48, 6, 11),
                    CANDLE => (8, 20, 9, 11),
                    STUBBY => (15, 26, 1, 11),
                    TIKI => (15, 14, 6, 6),
                    _ => (52, 17, 6, 8), // BBQ
                };
                q_set_rect(&mut bounds, 0, 0, w, t);
                q_offset_rect(&mut bounds, a.top_left.h + dx, a.top_left.v + dy);
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
            }

            INVIS_BLOWER => {
                let a = the_object.a();
                match a.vector & 0x0F {
                    1 => {
                        // up
                        q_set_rect(&mut bounds, 0, -a.distance - 24, FLOOR_COLUMN_WIDE, 0);
                        q_offset_rect(&mut bounds, 12 - FLOOR_COLUMN_WIDE / 2, 24);
                        q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                        hot_spot_number = self.add_active_rect(&bounds, LIFT_IT, who, a.state, false);
                    }
                    2 => {
                        // right
                        q_set_rect(&mut bounds, 0, 0, a.distance + 24, FAN_COLUMN_THICK);
                        q_offset_rect(&mut bounds, 0, 12 - FAN_COLUMN_THICK / 2);
                        q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                        hot_spot_number = self.add_active_rect(&bounds, PUSH_IT_RIGHT, who, a.state, false);
                    }
                    4 => {
                        // down
                        q_set_rect(&mut bounds, 0, 0, FLOOR_COLUMN_WIDE, a.distance + 24);
                        q_offset_rect(&mut bounds, 12 - FLOOR_COLUMN_WIDE / 2, 0);
                        q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                        hot_spot_number = self.add_active_rect(&bounds, DROP_IT, who, a.state, false);
                    }
                    8 => {
                        // left
                        q_set_rect(&mut bounds, 0, 0, a.distance + 24, FAN_COLUMN_THICK);
                        q_offset_rect(&mut bounds, -(a.distance), 12 - FAN_COLUMN_THICK / 2);
                        q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                        hot_spot_number = self.add_active_rect(&bounds, PUSH_IT_LEFT, who, a.state, false);
                    }
                    _ => {}
                }
            }

            LIFT_AREA => {
                let a = the_object.a();
                q_set_rect(&mut bounds, 0, 0, a.distance, a.tall * 2);
                q_offset_rect(&mut bounds, a.top_left.h, a.top_left.v);
                match a.vector & 0x0F {
                    1 => hot_spot_number = self.add_active_rect(&bounds, LIFT_IT, who, a.state, false), // up
                    2 => hot_spot_number = self.add_active_rect(&bounds, PUSH_IT_RIGHT, who, a.state, false), // right
                    4 => hot_spot_number = self.add_active_rect(&bounds, DROP_IT, who, a.state, false), // down
                    8 => hot_spot_number = self.add_active_rect(&bounds, PUSH_IT_LEFT, who, a.state, false), // left
                    _ => {}
                }
            }

            TABLE | SHELF | CABINET | FILING_CABINET | WASTE_BASKET | MILK_CRATE | COUNTER | DRESSER | DECK_TABLE
            | TRUNK | INVIS_OBSTACLE => {
                bounds = the_object.b().bounds;
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
            }

            BOOKS => {
                bounds = the_object.b().bounds;
                bounds.right -= 2;
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
            }

            MANHOLE => {
                bounds = the_object.b().bounds;
                bounds.left += GLIDER_WIDE + 3;
                bounds.right -= GLIDER_WIDE + 3;
                bounds.top = FLOOR_LIMIT - 1;
                bounds.bottom = TILE_HIGH;
                hot_spot_number = self.add_active_rect(&bounds, IGNORE_GROUND, who, true, false);
            }

            INVIS_BOUNCE => {
                bounds = the_object.b().bounds;
                hot_spot_number = self.add_active_rect(&bounds, BOUNCE_IT, who, true, true);
            }

            STOOL => {
                bounds = the_object.b().bounds;
                inset_rect(&mut bounds, 1, 1);
                bounds.bottom = bounds.top + STOOL_THICK;
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
            }

            RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | CUCKOO | PAPER | BATTERY | BANDS | FOIL | INVIS_BONUS | STAR
            | HELIUM => {
                let c = the_object.c();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, c.top_left.h, c.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, REWARD_IT, who, c.state, false);
            }

            GREASE_RT => {
                let c = the_object.c();
                if c.state {
                    bounds = sr(self, the_object.what);
                    zero_rect_corner(&mut bounds);
                    q_offset_rect(&mut bounds, c.top_left.h, c.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, REWARD_IT, who, true, false);
                } else {
                    q_set_rect(&mut bounds, 0, -2, c.length - 5, 0);
                    q_offset_rect(&mut bounds, 32 - 1, 27);
                    q_offset_rect(&mut bounds, c.top_left.h, c.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, SLIDE_IT, who, true, false);
                }
            }

            GREASE_LF => {
                let c = the_object.c();
                if c.state {
                    bounds = sr(self, the_object.what);
                    zero_rect_corner(&mut bounds);
                    q_offset_rect(&mut bounds, c.top_left.h, c.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, REWARD_IT, who, true, false);
                } else {
                    q_set_rect(&mut bounds, -c.length + 5, -2, 0, 0);
                    q_offset_rect(&mut bounds, 1, 27);
                    q_offset_rect(&mut bounds, c.top_left.h, c.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, SLIDE_IT, who, true, false);
                }
            }

            SPARKLE => {
                // (the C works out the bounds but adds no hot spot)
            }

            SLIDER => {
                let c = the_object.c();
                q_set_rect(&mut bounds, 0, 0, c.length, 16);
                q_offset_rect(&mut bounds, c.top_left.h, c.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, SLIDE_IT, who, true, false);
            }

            UP_STAIRS => {
                let d = the_object.d();
                q_set_rect(&mut bounds, 0, 0, 112, 32);
                q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, MOVE_IT_UP, who, true, false);
            }

            DOWN_STAIRS => {
                let d = the_object.d();
                q_set_rect(&mut bounds, -80, -56, 0, 0);
                q_offset_rect(&mut bounds, sr(self, DOWN_STAIRS).right, 170);
                q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, MOVE_IT_DOWN, who, true, false);
            }

            MAILBOX_LF => {
                let d = the_object.d();
                if d.who != 255 {
                    q_set_rect(&mut bounds, -72, 0, 0, 40);
                    q_offset_rect(&mut bounds, 30, 16);
                    q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, MAIL_IT_LEFT, who, true, false);
                }
            }

            MAILBOX_RT => {
                let d = the_object.d();
                if d.who != 255 {
                    q_set_rect(&mut bounds, 0, 0, 72, 40);
                    q_offset_rect(&mut bounds, 79, 16);
                    q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, MAIL_IT_RIGHT, who, true, false);
                }
            }

            FLOOR_TRANS => {
                let d = the_object.d();
                if d.who != 255 {
                    q_set_rect(&mut bounds, 0, -48, 76, 0);
                    q_offset_rect(&mut bounds, -8, rect_tall(&sr(self, FLOOR_TRANS)));
                    q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, DUCT_IT_DOWN, who, true, false);
                }
            }

            CEILING_TRANS => {
                let d = the_object.d();
                if d.who != 255 {
                    q_set_rect(&mut bounds, 0, 0, 76, 48);
                    q_offset_rect(&mut bounds, -8, 0);
                    q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                    hot_spot_number = self.add_active_rect(&bounds, DUCT_IT_UP, who, true, false);
                }
            }

            DOOR_IN_LF | DOOR_IN_RT | DOOR_EX_RT | DOOR_EX_LF => {
                let d = the_object.d();
                let (dx, action) = match the_object.what {
                    DOOR_IN_LF => (0, IGNORE_LEFT_WALL),
                    DOOR_IN_RT => (128, IGNORE_RIGHT_WALL),
                    DOOR_EX_RT => (0, IGNORE_RIGHT_WALL),
                    _ => (0, IGNORE_LEFT_WALL), // DOOR_EX_LF
                };
                q_set_rect(&mut bounds, 0, 0, 16, 240);
                q_offset_rect(&mut bounds, dx, 52);
                q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, action, who, true, false);
            }

            WINDOW_IN_LF | WINDOW_IN_RT | WINDOW_EX_RT | WINDOW_EX_LF => {
                let d = the_object.d();
                let (dx, action) = match the_object.what {
                    WINDOW_IN_LF => (0, IGNORE_LEFT_WALL),
                    WINDOW_IN_RT => (4, IGNORE_RIGHT_WALL),
                    WINDOW_EX_RT => (0, IGNORE_RIGHT_WALL),
                    _ => (0, IGNORE_LEFT_WALL), // WINDOW_EX_LF
                };
                q_set_rect(&mut bounds, 0, 0, 16, 44);
                q_offset_rect(&mut bounds, dx, 96);
                q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, action, who, true, false);
            }

            INVIS_TRANS => {
                let d = the_object.d();
                if d.who != 255 {
                    q_set_rect(&mut bounds, 0, 0, 64, 32);
                    q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                    bounds.bottom = bounds.top + d.tall;
                    bounds.right += d.wide;
                    hot_spot_number = self.add_active_rect(&bounds, TRANSPORT_IT, who, true, false);
                }
            }

            DELUXE_TRANS => {
                let d = the_object.d();
                if d.who != 255 {
                    let wide = (d.tall & 0xFF00) >> 8; // Get high byte
                    let tall = d.tall & 0x00FF; // Get low byte
                    q_set_rect(&mut bounds, 0, 0, wide * 4, tall * 4); // Scale by 4
                    q_offset_rect(&mut bounds, d.top_left.h, d.top_left.v);
                    let is_on = (d.wide & 0x0F) != 0;
                    hot_spot_number = self.add_active_rect(&bounds, TRANSPORT_IT, who, is_on, false);
                }
            }

            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
            | LG_TRIGGER => {
                let e = the_object.e();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, e.top_left.h, e.top_left.v);
                if (the_object.what == TRIGGER) || (the_object.what == LG_TRIGGER) {
                    if e.where_ != -1 {
                        hot_spot_number = self.add_active_rect(&bounds, TRIGGER_IT, who, true, false);
                    }
                } else {
                    if e.where_ != -1 {
                        hot_spot_number = self.add_active_rect(&bounds, SWITCH_IT, who, true, false);
                    }
                }
            }

            SOUND_TRIGGER => {
                let e = the_object.e();
                q_set_rect(&mut bounds, 0, 0, 48, 48);
                q_offset_rect(&mut bounds, e.top_left.h, e.top_left.v);
                if self.load_trigger_sound(e.where_) == 0 {
                    // noErr
                    hot_spot_number = self.add_active_rect(&bounds, SOUND_IT, who, true, false);
                }
            }

            CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP | FLOURESCENT | TRACK_LIGHT | INVIS_LIGHT => {}

            SHREDDER => {
                let g = the_object.g();
                bounds = sr(self, the_object.what);
                bounds.bottom = bounds.top + SHREDDER_ACTIVE_HIGH;
                bounds.right += 48;
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, g.top_left.h, g.top_left.v);
                q_offset_rect(&mut bounds, -24, -36);
                hot_spot_number = self.add_active_rect(&bounds, SHRED_IT, who, g.state, true);
            }

            GUITAR => {
                let g = the_object.g();
                q_set_rect(&mut bounds, 0, 0, 8, 96);
                q_offset_rect(&mut bounds, g.top_left.h + 34, g.top_left.v + 32);
                hot_spot_number = self.add_active_rect(&bounds, STRUM_IT, who, true, false);
            }

            OUTLET => {
                let g = the_object.g();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, g.top_left.h, g.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, IGNORE_IT, who, g.state, false);
            }

            MICROWAVE => {
                let g = the_object.g();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, g.top_left.h, g.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
                bounds.bottom = bounds.top;
                bounds.top = 0;
                hot_spot_number = self.add_active_rect(&bounds, MICROWAVE_IT, who, true, true);
            }

            TOASTER | MAC_PLUS | TV | COFFEE | VCR | STEREO | CINDER_BLOCK | FLOWER_BOX | C_DS => {
                let g = the_object.g();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, g.top_left.h, g.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
            }

            CUSTOM_PICT => {}

            BALLOON | COPTER_LF | COPTER_RT | DART_LF | DART_RT | BALL | DRIP => {
                let h = the_object.h();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, h.top_left.h, h.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, IGNORE_IT, who, true, false);
            }

            FISH => {
                let h = the_object.h();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, h.top_left.h, h.top_left.v);
                hot_spot_number = self.add_active_rect(&bounds, DISSOLVE_IT, who, true, true);
            }

            COBWEB => {
                let h = the_object.h();
                bounds = sr(self, the_object.what);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, h.top_left.h, h.top_left.v);
                inset_rect(&mut bounds, -24, -10);
                hot_spot_number = self.add_active_rect(&bounds, WEB_IT, who, true, true);
            }

            OZMA | MIRROR | MOUSEHOLE | FIREPLACE | FLOWER | WALL_WINDOW | BEAR | CALENDAR | VASE1 | VASE2 | BULLETIN
            | CLOUD | FAUCET | RUG => {}

            CHIMES => {
                self.num_chimes += 1;
                let i = the_object.i();
                bounds = sr(self, CHIMES);
                zero_rect_corner(&mut bounds);
                q_offset_rect(&mut bounds, i.bounds.left, i.bounds.top);
                hot_spot_number = self.add_active_rect(&bounds, CHIME_IT, who, true, false);
            }

            _ => {}
        }

        hot_spot_number
    }

    //--------------------------------------------------------------  VerticalRoomOffset

    pub fn vertical_room_offset(&self, neighbor: i32) -> i32 {
        let mut offset = 0;

        match neighbor {
            NORTH_ROOM | NORTH_EAST_ROOM | NORTH_WEST_ROOM => {
                offset -= VERT_LOCAL_OFFSET;
            }
            SOUTH_EAST_ROOM | SOUTH_ROOM | SOUTH_WEST_ROOM => {
                offset += VERT_LOCAL_OFFSET;
            }
            _ => {}
        }

        offset
    }

    //--------------------------------------------------------------  OffsetRectRoomRelative

    pub fn offset_rect_room_relative(&self, the_rect: &mut Rect, neighbor: i32) {
        q_offset_rect(the_rect, self.play_origin_h, self.play_origin_v);

        match neighbor {
            NORTH_ROOM => q_offset_rect(the_rect, 0, -VERT_LOCAL_OFFSET),
            NORTH_EAST_ROOM => q_offset_rect(the_rect, ROOM_WIDE, -VERT_LOCAL_OFFSET),
            EAST_ROOM => q_offset_rect(the_rect, ROOM_WIDE, 0),
            SOUTH_EAST_ROOM => q_offset_rect(the_rect, ROOM_WIDE, VERT_LOCAL_OFFSET),
            SOUTH_ROOM => q_offset_rect(the_rect, 0, VERT_LOCAL_OFFSET),
            SOUTH_WEST_ROOM => q_offset_rect(the_rect, -ROOM_WIDE, VERT_LOCAL_OFFSET),
            WEST_ROOM => q_offset_rect(the_rect, -ROOM_WIDE, 0),
            NORTH_WEST_ROOM => q_offset_rect(the_rect, -ROOM_WIDE, -VERT_LOCAL_OFFSET),
            _ => {}
        }
    }

    //--------------------------------------------------------------  GetUpStairsRightEdge

    pub fn get_up_stairs_right_edge(&self) -> i32 {
        let mut right_edge = ROOM_WIDE;

        for i in 0..MAX_ROOM_OBS as usize {
            let this_object = self.house.rooms[self.this_room_number as usize].objects[i];
            if this_object.what == DOWN_STAIRS {
                right_edge = this_object.d().top_left.h + self.src_rects[DOWN_STAIRS as usize].right - 1;
                break;
            }
        }

        right_edge
    }

    //--------------------------------------------------------------  GetDownStairsLeftEdge

    pub fn get_down_stairs_left_edge(&self) -> i32 {
        let mut left_edge = 0;

        for i in 0..MAX_ROOM_OBS as usize {
            let this_object = self.house.rooms[self.this_room_number as usize].objects[i];
            if this_object.what == UP_STAIRS {
                left_edge = this_object.d().top_left.h + 1;
                break;
            }
        }

        left_edge
    }
}

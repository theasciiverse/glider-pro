//! Objects.c: the master list of the objects of the rooms around, their
//! links, and switching objects on and off.

use super::*;

pub const MAX_TEMP_MANHOLES: i32 = 8;

impl Game {
    //--------------------------------------------------------------  IsThisValid

    pub fn is_this_valid(&self, where_: i32, who: i32) -> bool {
        let mut its_good = true;

        let obj = &self.house.rooms[where_ as usize].objects[who as usize];
        match obj.what {
            OBJECT_IS_EMPTY => {
                its_good = false;
            }

            RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | CUCKOO | PAPER | BATTERY | BANDS | FOIL | INVIS_BONUS | STAR
            | SPARKLE | HELIUM => {
                its_good = obj.c().state;
            }

            _ => {}
        }

        its_good
    }

    //--------------------------------------------------------------  GetRoomLinked

    pub fn get_room_linked(&self, who: &Object) -> i32 {
        let compound_room_number;
        let mut floor = 0;
        let mut suite = 0;

        match who.what {
            MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | INVIS_TRANS | DELUXE_TRANS => {
                compound_room_number = who.d().where_;
                if compound_room_number != -1 {
                    // is object linked
                    self.extract_floor_suite(compound_room_number, &mut floor, &mut suite);
                    self.get_room_number(floor, suite)
                } else {
                    -1 // not linked
                }
            }

            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
            | LG_TRIGGER => {
                compound_room_number = who.e().where_;
                if compound_room_number != -1 {
                    // is object linked
                    self.extract_floor_suite(compound_room_number, &mut floor, &mut suite);
                    self.get_room_number(floor, suite)
                } else {
                    -1 // not linked
                }
            }

            _ => -1,
        }
    }

    //--------------------------------------------------------------  GetObjectLinked

    pub fn get_object_linked(&self, who: &Object) -> i32 {
        match who.what {
            MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | INVIS_TRANS | DELUXE_TRANS => {
                if who.d().who != 255 {
                    // is it linked?
                    who.d().who
                } else {
                    -1 // object not linked
                }
            }

            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER
            | LG_TRIGGER => {
                if who.e().who != 255 {
                    // is it linked?
                    who.e().who
                } else {
                    -1 // object not linked
                }
            }

            _ => -1,
        }
    }

    //--------------------------------------------------------------  ObjectIsLinkTransport

    pub fn object_is_link_transport(&self, who: &Object) -> bool {
        matches!(who.what, MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | INVIS_TRANS | DELUXE_TRANS)
    }

    //--------------------------------------------------------------  ObjectIsLinkSwitch

    pub fn object_is_link_switch(&self, who: &Object) -> bool {
        matches!(
            who.what,
            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH | INVIS_SWITCH | TRIGGER | LG_TRIGGER
        )
    }

    //--------------------------------------------------------------  ListOneRoomsObjects

    pub fn list_one_rooms_objects(&mut self, where_: i32) {
        let room_num = self.local_numbers[where_ as usize];
        if room_num == ROOM_IS_EMPTY {
            return;
        }

        for n in 0..MAX_ROOM_OBS {
            if self.num_master_objects < MAX_MASTER_OBJECTS {
                let this_object = self.house.rooms[room_num as usize].objects[n as usize];
                let k = self.num_master_objects as usize;

                self.master_objects[k].room_num = room_num;
                self.master_objects[k].object_num = n;
                self.master_objects[k].room_link = self.get_room_linked(&this_object);
                self.master_objects[k].object_link = self.get_object_linked(&this_object);
                self.master_objects[k].local_link = -1;

                self.master_objects[k].the_object = self.house.rooms[room_num as usize].objects[n as usize];

                if (where_ == CENTRAL_ROOM) && self.is_this_valid(room_num, n) {
                    let hot = self.create_active_rects(n);
                    self.master_objects[k].hot_num = hot;
                } else {
                    self.master_objects[k].hot_num = -1;
                }
                self.master_objects[k].dyna_num = -1;

                self.num_master_objects += 1;

                if where_ == CENTRAL_ROOM {
                    self.num_local_master_objects += 1;
                }
            }
        }
    }

    //--------------------------------------------------------------  ListAllLocalObjects

    pub fn list_all_local_objects(&mut self) {
        // masterObjects and hotSpots: the C allocates them at start-up.
        if self.master_objects.len() < MAX_MASTER_OBJECTS as usize {
            self.master_objects.resize(MAX_MASTER_OBJECTS as usize, ObjData::default());
        }
        if self.hot_spots.len() < MAX_HOT_SPOTS as usize {
            self.hot_spots.resize(MAX_HOT_SPOTS as usize, HotObject::default());
        }

        self.num_master_objects = 0;
        self.num_local_master_objects = 0;
        self.n_hot_spots = 0;

        self.list_one_rooms_objects(CENTRAL_ROOM);

        if self.num_neighbors > 1 {
            self.list_one_rooms_objects(EAST_ROOM);
            self.list_one_rooms_objects(WEST_ROOM);
        }

        if self.num_neighbors > 3 {
            self.list_one_rooms_objects(NORTH_ROOM);
            self.list_one_rooms_objects(NORTH_EAST_ROOM);
            self.list_one_rooms_objects(SOUTH_EAST_ROOM);
            self.list_one_rooms_objects(SOUTH_ROOM);
            self.list_one_rooms_objects(SOUTH_WEST_ROOM);
            self.list_one_rooms_objects(NORTH_WEST_ROOM);
        }

        let num = self.num_master_objects as usize;
        for i in 0..num {
            // correlate links with index into this list
            if (self.master_objects[i].room_link != -1) && (self.master_objects[i].object_link != -1) {
                // if object has a link
                for n in 0..num {
                    // search for the object linked to in this list
                    if (self.master_objects[i].room_link == self.master_objects[n].room_num)
                        && (self.master_objects[i].object_link == self.master_objects[n].object_num)
                    {
                        self.master_objects[i].local_link = n as i32; // log the index
                    }
                }
            }
        }
    }

    //--------------------------------------------------------------  AddTempManholeRect

    pub fn add_temp_manhole_rect(&mut self, man_hole: &Rect) {
        if self.num_temp_manholes < MAX_TEMP_MANHOLES {
            let mut temp_rect = *man_hole;
            temp_rect.bottom = temp_rect.top + FLOOR_SUPPORT_TALL;
            self.temp_manholes[self.num_temp_manholes as usize] = temp_rect;
            self.num_temp_manholes += 1;
        }
    }

    //--------------------------------------------------------------  SetObjectState

    pub fn set_object_state(&mut self, room: i32, object: i32, action: i32, local: i32) -> bool {
        // `changed` is left unset by the C for some kinds; false here.
        let mut changed = false;
        let (r, o) = (room as usize, object as usize);
        let what = self.house.rooms[r].objects[o].what;

        match what {
            FLOOR_VENT | CEILING_VENT | FLOOR_BLOWER | CEILING_BLOWER | LEFT_FAN | RIGHT_FAN | SEWER_GRATE
            | INVIS_BLOWER | GRECO_VENT | SEWER_BLOWER | LIFT_AREA => {
                let mut a = self.house.rooms[r].objects[o].a();
                match action {
                    TOGGLE => {
                        self.new_state = !a.state;
                        a.state = self.new_state;
                        changed = true;
                    }
                    FORCE_ON => {
                        changed = !a.state;
                        self.new_state = true;
                        a.state = self.new_state;
                    }
                    FORCE_OFF => {
                        changed = a.state;
                        self.new_state = false;
                        a.state = self.new_state;
                    }
                    _ => {}
                }
                self.house.rooms[r].objects[o].set_a(a);
                if changed && (local != -1) {
                    let ns = self.new_state;
                    let mo = &mut self.master_objects[local as usize].the_object;
                    let mut ma = mo.a();
                    ma.state = ns;
                    mo.set_a(ma);
                    if room == self.this_room_number {
                        let mut ta = self.this_room.objects[o].a();
                        ta.state = ns;
                        self.this_room.objects[o].set_a(ta);
                    }
                    if ns {
                        self.play_priority_sound(BLOWER_ON, BLOWER_ON_PRIORITY);
                    } else {
                        self.play_priority_sound(BLOWER_OFF, BLOWER_OFF_PRIORITY);
                    }
                    let hot = self.master_objects[local as usize].hot_num;
                    if hot != -1 {
                        self.hot_spots[hot as usize].is_on = ns;
                    }
                }
            }

            TAPER | CANDLE | STUBBY | TIKI | BBQ => {
                changed = false; // Cannot switch on/off these
            }

            TABLE | SHELF | CABINET | FILING_CABINET | WASTE_BASKET | MILK_CRATE | COUNTER | DRESSER | STOOL | TRUNK
            | DECK_TABLE | INVIS_OBSTACLE | MANHOLE | BOOKS | INVIS_BOUNCE => {
                changed = false; // Cannot switch on/off these
            }

            RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | CUCKOO | PAPER | BATTERY | BANDS | GREASE_RT | GREASE_LF | FOIL
            | INVIS_BONUS | STAR | SPARKLE | HELIUM => {
                let mut c = self.house.rooms[r].objects[o].c();
                changed = c.state;
                self.new_state = false;
                c.state = self.new_state;
                self.house.rooms[r].objects[o].set_c(c);
                if changed && (local != -1) {
                    // The C writes data.a.state here (byte 7, the low byte
                    // of a bonus's points), not data.c.state: kept.
                    let mo = &mut self.master_objects[local as usize].the_object;
                    let mut ma = mo.a();
                    ma.state = false;
                    mo.set_a(ma);
                    if room == self.this_room_number {
                        let mut tc = self.this_room.objects[o].c();
                        tc.state = false;
                        self.this_room.objects[o].set_c(tc);
                        let hot = self.master_objects[local as usize].hot_num;
                        if hot != -1 {
                            self.hot_spots[hot as usize].is_on = false;
                        }
                    }
                }
            }

            SLIDER => {}

            UP_STAIRS | DOWN_STAIRS | MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | DOOR_IN_LF | DOOR_IN_RT
            | DOOR_EX_RT | DOOR_EX_LF | WINDOW_IN_LF | WINDOW_IN_RT | WINDOW_EX_RT | WINDOW_EX_LF | INVIS_TRANS => {
                changed = false;
            }

            DELUXE_TRANS => {
                let mut d = self.house.rooms[r].objects[o].d();
                match action {
                    TOGGLE => {
                        self.new_state = (d.wide & 0x0F) != 0;
                        self.new_state = !self.new_state;
                        d.wide &= 0xF0;
                        d.wide += self.new_state as i32;
                        changed = true;
                    }
                    FORCE_ON => {
                        changed = (d.wide & 0x0F) == 0x00;
                        self.new_state = true;
                        d.wide &= 0xF0;
                        d.wide += self.new_state as i32;
                    }
                    FORCE_OFF => {
                        changed = (d.wide & 0x0F) != 0x00;
                        self.new_state = false;
                        d.wide &= 0xF0;
                        d.wide += self.new_state as i32;
                    }
                    _ => {}
                }
                self.house.rooms[r].objects[o].set_d(d);
                if changed && (local != -1) {
                    let wide = self.house.rooms[r].objects[o].d().wide;
                    let mo = &mut self.master_objects[local as usize].the_object;
                    let mut md = mo.d();
                    md.wide = wide;
                    mo.set_d(md);
                    if room == self.this_room_number {
                        let mut td = self.this_room.objects[o].d();
                        td.wide = wide;
                        self.this_room.objects[o].set_d(td);
                    }
                    let hot = self.master_objects[local as usize].hot_num;
                    if hot != -1 {
                        self.hot_spots[hot as usize].is_on = self.new_state;
                    }
                }
            }

            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | INVIS_SWITCH | TRIGGER | LG_TRIGGER
            | SOUND_TRIGGER => {
                changed = false;
            }

            CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP | FLOURESCENT | TRACK_LIGHT | INVIS_LIGHT => {
                let mut f = self.house.rooms[r].objects[o].f();
                match action {
                    TOGGLE => {
                        self.new_state = !f.state;
                        f.state = self.new_state;
                        changed = true;
                    }
                    FORCE_ON => {
                        changed = !f.state;
                        self.new_state = true;
                        f.state = self.new_state;
                    }
                    FORCE_OFF => {
                        changed = f.state;
                        self.new_state = false;
                        f.state = self.new_state;
                    }
                    _ => {}
                }
                self.house.rooms[r].objects[o].set_f(f);
                if changed && (local != -1) {
                    let ns = self.new_state;
                    let mo = &mut self.master_objects[local as usize].the_object;
                    let mut mf = mo.f();
                    mf.state = ns;
                    mo.set_f(mf);
                    if room == self.this_room_number {
                        let mut tf = self.this_room.objects[o].f();
                        tf.state = ns;
                        self.this_room.objects[o].set_f(tf);
                    }
                }
            }

            GUITAR => {
                // really no point to change this state
                changed = false;
            }

            STEREO => {
                self.new_state = !self.is_play_music_game;
                self.is_play_music_game = self.new_state;
                changed = true;
            }

            SHREDDER | TOASTER | MAC_PLUS | TV | COFFEE | OUTLET | VCR | MICROWAVE => {
                let mut g = self.house.rooms[r].objects[o].g();
                match action {
                    TOGGLE => {
                        self.new_state = !g.state;
                        g.state = self.new_state;
                        changed = true;
                    }
                    FORCE_ON => {
                        changed = !g.state;
                        self.new_state = true;
                        g.state = self.new_state;
                    }
                    FORCE_OFF => {
                        changed = g.state;
                        self.new_state = false;
                        g.state = self.new_state;
                    }
                    _ => {}
                }
                self.house.rooms[r].objects[o].set_g(g);
                if changed && (local != -1) {
                    let ns = self.new_state;
                    let mo = &mut self.master_objects[local as usize].the_object;
                    let mut mg = mo.g();
                    mg.state = ns;
                    mo.set_g(mg);
                    if room == self.this_room_number {
                        let mut tg = self.this_room.objects[o].g();
                        tg.state = ns;
                        self.this_room.objects[o].set_g(tg);
                        if self.house.rooms[r].objects[o].what == SHREDDER {
                            // (the C indexes hotSpots[-1] if the shredder has none)
                            let hot = self.master_objects[local as usize].hot_num;
                            if hot >= 0 {
                                self.hot_spots[hot as usize].is_on = ns;
                            }
                        }
                    }
                }
            }

            CINDER_BLOCK | FLOWER_BOX | C_DS | CUSTOM_PICT => {
                changed = false;
            }

            BALLOON | COPTER_LF | COPTER_RT | DART_LF | DART_RT | BALL | DRIP | FISH => {
                let mut h = self.house.rooms[r].objects[o].h();
                match action {
                    TOGGLE => {
                        self.new_state = !h.state;
                        h.state = self.new_state;
                        changed = true;
                    }
                    FORCE_ON => {
                        changed = !h.state;
                        self.new_state = true;
                        h.state = self.new_state;
                    }
                    FORCE_OFF => {
                        changed = h.state;
                        self.new_state = false;
                        h.state = self.new_state;
                    }
                    _ => {}
                }
                self.house.rooms[r].objects[o].set_h(h);
                if changed && (local != -1) {
                    let ns = self.new_state;
                    let mo = &mut self.master_objects[local as usize].the_object;
                    let mut mh = mo.h();
                    mh.state = ns;
                    mo.set_h(mh);
                    if room == self.this_room_number {
                        let mut th = self.this_room.objects[o].h();
                        th.state = ns;
                        self.this_room.objects[o].set_h(th);
                    }
                }
            }

            COBWEB => {
                changed = false;
            }

            OZMA | MIRROR | MOUSEHOLE | FIREPLACE | FLOWER | WALL_WINDOW | BEAR | CALENDAR | VASE1 | VASE2 | BULLETIN
            | CLOUD | FAUCET | RUG | CHIMES => {
                changed = false;
            }

            _ => {}
        }

        changed
    }

    //--------------------------------------------------------------  GetObjectState

    pub fn get_object_state(&self, room: i32, object: i32) -> bool {
        let mut the_state = true;

        // The C reads past the house for an unlinked switch (room -1 or
        // object 255); here such a lookup gives the default, true.
        let Some(obj) = (room >= 0)
            .then(|| self.house.rooms.get(room as usize))
            .flatten()
            .and_then(|rm| rm.objects.get(object as usize))
        else {
            return the_state;
        };

        match obj.what {
            FLOOR_VENT | CEILING_VENT | FLOOR_BLOWER | CEILING_BLOWER | LEFT_FAN | RIGHT_FAN | SEWER_GRATE
            | INVIS_BLOWER | GRECO_VENT | SEWER_BLOWER | LIFT_AREA => {
                the_state = obj.a().state;
            }

            TAPER | CANDLE | STUBBY | TIKI | BBQ => {}

            TABLE | SHELF | CABINET | FILING_CABINET | WASTE_BASKET | MILK_CRATE | COUNTER | DRESSER | DECK_TABLE
            | STOOL | TRUNK | INVIS_OBSTACLE | MANHOLE | BOOKS | INVIS_BOUNCE => {}

            RED_CLOCK | BLUE_CLOCK | YELLOW_CLOCK | CUCKOO | PAPER | BATTERY | BANDS | GREASE_RT | GREASE_LF | FOIL
            | INVIS_BONUS | STAR | SPARKLE | HELIUM => {
                the_state = obj.c().state;
            }

            SLIDER => {}

            UP_STAIRS | DOWN_STAIRS | MAILBOX_LF | MAILBOX_RT | FLOOR_TRANS | CEILING_TRANS | DOOR_IN_LF | DOOR_IN_RT
            | DOOR_EX_RT | DOOR_EX_LF | WINDOW_IN_LF | WINDOW_IN_RT | WINDOW_EX_RT | WINDOW_EX_LF | INVIS_TRANS => {}

            DELUXE_TRANS => {
                the_state = (obj.d().wide & 0x0F) != 0;
            }

            LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | INVIS_SWITCH | TRIGGER | LG_TRIGGER
            | SOUND_TRIGGER => {}

            CEILING_LIGHT | LIGHT_BULB | TABLE_LAMP | HIP_LAMP | DECO_LAMP | FLOURESCENT | TRACK_LIGHT | INVIS_LIGHT => {
                the_state = obj.f().state;
            }

            STEREO => {
                the_state = self.is_play_music_game;
            }

            SHREDDER | TOASTER | MAC_PLUS | GUITAR | TV | COFFEE | OUTLET | VCR | MICROWAVE => {
                the_state = obj.g().state;
            }

            CINDER_BLOCK | FLOWER_BOX | C_DS | CUSTOM_PICT => {}

            BALLOON | COPTER_LF | COPTER_RT | DART_LF | DART_RT | BALL | DRIP | FISH => {
                the_state = obj.h().state;
            }

            COBWEB => {}

            OZMA | MIRROR | MOUSEHOLE | FIREPLACE | FLOWER | WALL_WINDOW | BEAR | CALENDAR | VASE1 | VASE2 | BULLETIN
            | CLOUD | FAUCET | RUG | CHIMES => {}

            _ => {}
        }

        the_state
    }

    // BringSendFrontBack: the editor's, left out.
}

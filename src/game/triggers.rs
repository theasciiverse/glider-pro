//! Triggers.c: triggers that, after a delay, set off a linked object.

use super::*;

pub const MAX_TRIGGERS: usize = 16;

impl Game {
    //--------------------------------------------------------------  ArmTrigger

    pub fn arm_trigger(&mut self, who: usize) {
        if self.hot_spots[who].still_over {
            return;
        }

        let where_ = self.find_empty_trigger_slot();

        if where_ != -1 {
            let w = where_ as usize;
            let who_linked = self.hot_spots[who].who; // what is trigger's obj. #
            let mo = self.master_objects[who_linked as usize];
            self.triggers[w].room = mo.room_link;
            self.triggers[w].object = mo.object_link;
            self.triggers[w].index = who_linked;
            self.triggers[w].timer = mo.the_object.e().delay * 3;
            self.triggers[w].what = self.master_objects[self.triggers[w].object as usize]
                .the_object
                .what;
            self.triggers[w].armed = true;
        }

        self.hot_spots[who].still_over = true;
    }

    //--------------------------------------------------------------  FindEmptyTriggerSlot

    pub fn find_empty_trigger_slot(&self) -> i32 {
        let mut where_ = -1;

        for i in 0..MAX_TRIGGERS {
            if !self.triggers[i].armed {
                where_ = i as i32;
                break;
            }
        }

        where_
    }

    //--------------------------------------------------------------  HandleTriggers

    pub fn handle_triggers(&mut self) {
        for i in 0..MAX_TRIGGERS {
            if self.triggers[i].armed {
                self.triggers[i].timer -= 1;
                if self.triggers[i].timer <= 0 {
                    self.triggers[i].timer = 0;
                    self.triggers[i].armed = false;
                    self.fire_trigger(i as i32);
                }
            }
        }
    }

    //--------------------------------------------------------------  FireTrigger

    pub fn fire_trigger(&mut self, index: i32) {
        let t = self.triggers[index as usize];
        let trigger_is = t.index;

        if self.master_objects[trigger_is as usize].local_link != -1 {
            let triggered_is = self.master_objects[trigger_is as usize].local_link;
            let mo = self.master_objects[triggered_is as usize];
            match mo.the_object.what {
                GREASE_RT | GREASE_LF => {
                    if self.set_object_state(t.room, t.object, FORCE_ON, triggered_is) {
                        self.spill_grease(mo.dyna_num, mo.hot_num);
                    }
                }
                LIGHT_SWITCH | MACHINE_SWITCH | THERMOSTAT | POWER_SWITCH | KNIFE_SWITCH
                | INVIS_SWITCH => {
                    self.trigger_switch(mo.dyna_num);
                }
                SOUND_TRIGGER => {
                    self.play_priority_sound(CHORD_SOUND, CHORD_PRIORITY); // Change me
                }
                TOASTER => self.trigger_toast(mo.dyna_num),
                GUITAR => self.play_priority_sound(CHORD_SOUND, CHORD_PRIORITY),
                COFFEE => self.play_priority_sound(COFFEE_SOUND, COFFEE_PRIORITY),
                OUTLET => self.trigger_outlet(mo.dyna_num),
                BALLOON => self.trigger_balloon(mo.dyna_num),
                COPTER_LF | COPTER_RT => self.trigger_copter(mo.dyna_num),
                DART_LF | DART_RT => self.trigger_dart(mo.dyna_num),
                DRIP => self.trigger_drip(mo.dyna_num),
                FISH => self.trigger_fish(mo.dyna_num),
                _ => {}
            }
        } else {
            // The linked object is in another room: only grease can be
            // set off there. (triggeredIs is -1 here, as in the C.)
            let triggered_is = self.master_objects[trigger_is as usize].local_link;
            let what = self.house.rooms[t.room as usize].objects[t.object as usize].what;
            match what {
                GREASE_RT | GREASE_LF => {
                    if self.set_object_state(t.room, t.object, FORCE_ON, triggered_is) {
                        // The C reads masterObjects[-1] here (out of bounds,
                        // spilling whatever grease that garbage names); the
                        // jar's state in the house is changed all the same,
                        // so it is found spilt when its room is entered.
                        if triggered_is >= 0 {
                            let mo = self.master_objects[triggered_is as usize];
                            self.spill_grease(mo.dyna_num, mo.hot_num);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    //--------------------------------------------------------------  ZeroTriggers

    pub fn zero_triggers(&mut self) {
        for i in 0..MAX_TRIGGERS {
            self.triggers[i].armed = false;
        }
    }
}

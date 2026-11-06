//! Trip.c: switching appliances and enemies on and off, and setting them off.

use super::*;

impl Game {
    //--------------------------------------------------------------  ToggleToaster

    pub fn toggle_toaster(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleMacPlus

    pub fn toggle_mac_plus(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
        if d.active {
            d.timer = 40;
        } else {
            d.timer = 10;
        }
    }

    //--------------------------------------------------------------  ToggleTV

    pub fn toggle_tv(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
        // The QuickTime movie on the TV (hasMovie, tvOn) is not kept.
        d.timer = 4;
    }

    //--------------------------------------------------------------  ToggleCoffee

    pub fn toggle_coffee(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
        d.timer = 4;
    }

    //--------------------------------------------------------------  ToggleOutlet

    pub fn toggle_outlet(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleVCR

    pub fn toggle_vcr(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
        d.timer = 4;
    }

    //--------------------------------------------------------------  ToggleStereo

    pub fn toggle_stereos(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        if d.timer == 0 {
            d.active = !d.active;
            d.timer = 4;
        }
    }

    //--------------------------------------------------------------  ToggleMicrowave

    pub fn toggle_microwave(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
        d.timer = 4;
    }

    //--------------------------------------------------------------  ToggleBalloon

    pub fn toggle_balloon(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleCopter

    pub fn toggle_copter(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleDart

    pub fn toggle_dart(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleBall

    pub fn toggle_ball(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleDrip

    pub fn toggle_drip(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  ToggleFish

    pub fn toggle_fish(&mut self, index: i32) {
        let d = &mut self.dinahs[index as usize];
        d.active = !d.active;
    }

    //--------------------------------------------------------------  TriggerSwitch

    pub fn trigger_switch(&mut self, who: i32) {
        self.handle_switches(who as usize);
    }

    //--------------------------------------------------------------  TriggerToast

    pub fn trigger_toast(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if !d.moving {
            if d.active {
                d.v_vel = -d.count;
                d.frame = 0;
                d.moving = true;
                self.play_priority_sound(TOAST_LAUNCH_SOUND, TOAST_LAUNCH_PRIORITY);
            } else {
                d.frame = d.timer;
            }
        }
    }

    //--------------------------------------------------------------  TriggerOutlet

    pub fn trigger_outlet(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if d.position == 0 {
            if d.active {
                d.position = 1;
                d.timer = LENGTH_OF_ZAP;
                self.play_priority_sound(ZAP_SOUND, ZAP_PRIORITY);
            } else {
                d.timer = d.count;
            }
        }
    }

    //--------------------------------------------------------------  TriggerDrip

    pub fn trigger_drip(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if !d.moving && d.timer > 7 {
            d.timer = 7; // kick off drip
        }
    }

    //--------------------------------------------------------------  TriggerFish

    pub fn trigger_fish(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if d.active && !d.moving {
            d.whole = d.dest;
            d.moving = true;
            d.frame = 4;
            self.play_priority_sound(FISH_OUT_SOUND, FISH_OUT_PRIORITY);
        }
    }

    //--------------------------------------------------------------  TriggerBalloon

    pub fn trigger_balloon(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if !d.moving {
            d.timer = START_SPARKLE + 1;
        }
    }

    //--------------------------------------------------------------  TriggerCopter

    pub fn trigger_copter(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if !d.moving {
            d.timer = START_SPARKLE + 1;
        }
    }

    //--------------------------------------------------------------  TriggerDart

    pub fn trigger_dart(&mut self, who: i32) {
        let d = &mut self.dinahs[who as usize];
        if !d.moving {
            d.timer = START_SPARKLE + 1;
        }
    }

    //--------------------------------------------------------------  UpdateOutletsLighting

    pub fn update_outlets_lighting(&mut self, room: i32, n_lights: i32) {
        for i in 0..self.num_dynamics as usize {
            if self.dinahs[i].type_ == OUTLET && self.dinahs[i].room == room {
                self.dinahs[i].h_vel = n_lights;
            }
        }
    }
}

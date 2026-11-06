//! Dynamics3.c: the master handler and renderer of the dynamics, and
//! setting them up as a room is drawn.

use super::*;

const BALLOON_START: i32 = 310;
const COPTER_START: i32 = 8;
const DART_VELOCITY: i32 = 6;

//==============================================================  Functions

impl Game {
    //--------------------------------------------------------------  HandleDynamics

    // This is the master function that calls all the specific handlers above.

    pub fn handle_dynamics(&mut self) {
        for i in 0..self.num_dynamics {
            match self.dinahs[i as usize].type_ {
                SPARKLE => self.handle_sparkle_object(i),
                TOASTER => self.handle_toast(i),
                MAC_PLUS => self.handle_mac_plus(i),
                TV => self.handle_tv(i),
                COFFEE => self.handle_coffee(i),
                OUTLET => self.handle_outlet(i),
                VCR => self.handle_vcr(i),
                STEREO => self.handle_stereo(i),
                MICROWAVE => self.handle_microwave(i),
                BALLOON => self.handle_balloon(i),
                COPTER_LF | COPTER_RT => self.handle_copter(i),
                DART_LF | DART_RT => self.handle_dart(i),
                BALL => self.handle_ball(i),
                DRIP => self.handle_drip(i),
                FISH => self.handle_fish(i),
                _ => {}
            }
        }
    }

    //--------------------------------------------------------------  HandleDynamics

    // This is the master function that calls all the various rendering handlers
    // above.

    pub fn render_dynamics(&mut self) {
        for i in 0..self.num_dynamics {
            match self.dinahs[i as usize].type_ {
                TOASTER => self.render_toast(i),
                BALLOON => self.render_balloon(i),
                COPTER_LF | COPTER_RT => self.render_copter(i),
                DART_LF | DART_RT => self.render_dart(i),
                BALL => self.render_ball(i),
                DRIP => self.render_drip(i),
                FISH => self.render_fish(i),
                _ => {}
            }
        }
    }

    //--------------------------------------------------------------  ZeroDinahs

    // This clears all dynamics - zeros them all out.  Used to initialize them.

    pub fn zero_dinahs(&mut self) {
        for i in 0..MAX_DYNAMIC_OBS as usize {
            let d = &mut self.dinahs[i];
            d.type_ = OBJECT_IS_EMPTY;
            q_set_rect(&mut d.dest, 0, 0, 0, 0);
            q_set_rect(&mut d.whole, 0, 0, 0, 0);
            d.h_vel = 0;
            d.v_vel = 0;
            d.count = 0;
            d.frame = 0;
            d.timer = 0;
            d.position = 0;
            d.room = 0;
            d.byte0 = 0;
            d.active = false;
        }
        self.num_dynamics = 0;
    }

    //--------------------------------------------------------------  AddDynamicObject

    // When a room is being drawn, various dynamic objects are pointed here.
    // This function sets up the structures to handle them.

    pub fn add_dynamic_object(&mut self, what: i32, where_: &Rect, who: &Object, room: i32, index: i32, is_on: bool) -> i32 {
        if self.num_dynamics >= MAX_DYNAMIC_OBS {
            return -1;
        }

        let n = self.num_dynamics as usize;
        let (poh, pov) = (self.play_origin_h, self.play_origin_v);
        let byte0 = (index as u8) as i32; // (Byte)index
        self.dinahs[n].type_ = what;
        match what {
            SPARKLE => {
                let timer = self.random_int(60) + 15;
                let src = self.sparkle_src[0];
                let d = &mut self.dinahs[n];
                d.dest = src;
                zero_rect_corner(&mut d.dest);
                q_offset_rect(&mut d.dest, where_.left, where_.top);
                d.whole = d.dest;
                d.h_vel = 0;
                d.v_vel = 0;
                d.count = 0;
                d.frame = 0;
                d.timer = timer;
                d.position = 0;
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on;
            }

            TOASTER => {
                let src = self.bread_src[0];
                let d = &mut self.dinahs[n];
                d.dest = src;
                center_rect_in_rect(&mut d.dest, where_);
                let dv = where_.top - d.dest.top;
                v_offset_rect(&mut d.dest, dv);
                d.whole = d.dest;
                d.h_vel = where_.top + 2; // hVel used as clip
                let mut position = who.g().height; // reverse engineer init. vel.
                let mut velocity = 0;
                loop {
                    velocity += 1;
                    position -= velocity;
                    if position <= 0 {
                        break;
                    }
                }
                d.v_vel = -velocity;
                d.count = velocity; // count = initial velocity
                d.frame = who.g().delay * 3;
                d.timer = d.frame;
                d.position = 0; // launch/idle state
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on;
            }

            MAC_PLUS => {
                let src = self.plus_screen1;
                self.appliance_dinah(n, src, where_.left + poh + 10, where_.top + pov + 7, 0, room, byte0, is_on);
            }

            TV => {
                let src = self.tv_screen1;
                self.appliance_dinah(n, src, where_.left + poh + 17, where_.top + pov + 10, 0, room, byte0, is_on);
            }

            COFFEE => {
                let src = self.coffee_light1;
                let timer = if is_on { 200 } else { 0 };
                self.appliance_dinah(n, src, where_.left + poh + 32, where_.top + pov + 57, timer, room, byte0, is_on);
            }

            OUTLET => {
                let src = self.outlet_src[0];
                let num_lights = self.num_lights;
                let d = &mut self.dinahs[n];
                d.dest = src;
                zero_rect_corner(&mut d.dest);
                q_offset_rect(&mut d.dest, where_.left + poh, where_.top + pov);
                d.whole = d.dest;
                d.h_vel = num_lights;
                d.v_vel = 0;
                d.count = (who.g().delay * 6) / TICKS_PER_FRAME;
                d.frame = 0;
                d.timer = d.count;
                d.position = 0; // launch/idle state
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on;
            }

            VCR => {
                let src = self.vcr_time1;
                let timer = if is_on { 115 } else { 0 };
                self.appliance_dinah(n, src, where_.left + poh + 64, where_.top + pov + 6, timer, room, byte0, is_on);
            }

            STEREO => {
                let src = self.stereo_light1;
                self.appliance_dinah(n, src, where_.left + poh + 56, where_.top + pov + 20, 0, room, byte0, is_on);
            }

            MICROWAVE => {
                let src = self.micro_on;
                self.appliance_dinah(n, src, where_.left + poh + 14, where_.top + pov + 13, 0, room, byte0, is_on);
                let d = &mut self.dinahs[n];
                // the C sets dest.right before copying dest to whole
                d.dest.right = d.dest.left + 48;
                d.whole = d.dest;
            }

            BALLOON => {
                let src = self.balloon_src[0];
                let d = &mut self.dinahs[n];
                d.dest = src;
                zero_rect_corner(&mut d.dest);
                q_offset_rect(&mut d.dest, where_.left, 0);
                d.dest.bottom = BALLOON_START;
                d.dest.top = d.dest.bottom - rect_tall(&src);
                d.whole = d.dest;
                d.h_vel = 0;
                d.v_vel = -2;
                d.count = (who.h().delay * 6) / TICKS_PER_FRAME;
                d.frame = 0;
                d.timer = d.count;
                d.position = 0;
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on; // initially idle
            }

            COPTER_LF | COPTER_RT => {
                let src = self.copter_src[0];
                let d = &mut self.dinahs[n];
                d.dest = src;
                zero_rect_corner(&mut d.dest);
                q_offset_rect(&mut d.dest, where_.left, 0);
                d.dest.top = COPTER_START;
                d.dest.bottom = d.dest.top + rect_tall(&src);
                d.whole = d.dest;
                if what == COPTER_LF {
                    d.h_vel = -1;
                } else {
                    d.h_vel = 1;
                }
                d.v_vel = 2;
                d.count = (who.h().delay * 6) / TICKS_PER_FRAME;
                d.frame = 0;
                d.timer = d.count;
                d.position = d.dest.left;
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on; // initially idle
            }

            DART_LF | DART_RT => {
                let src = self.dart_src[0];
                let d = &mut self.dinahs[n];
                d.dest = src;
                zero_rect_corner(&mut d.dest);
                if what == DART_LF {
                    q_offset_rect(&mut d.dest, ROOM_WIDE - rect_wide(&src), where_.top);
                    d.h_vel = -DART_VELOCITY;
                    d.frame = 0;
                } else {
                    q_offset_rect(&mut d.dest, 0, where_.top);
                    d.h_vel = DART_VELOCITY;
                    d.frame = 2;
                }
                d.whole = d.dest;
                d.v_vel = 2;
                d.count = (who.h().delay * 6) / TICKS_PER_FRAME;
                d.timer = d.count;
                d.position = d.dest.top;
                // (the C does not set room for darts)
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on; // initially idle
            }

            BALL => {
                let src = self.ball_src[0];
                self.even_frame = true;
                let d = &mut self.dinahs[n];
                d.dest = src;
                zero_rect_corner(&mut d.dest);
                q_offset_rect(&mut d.dest, where_.left, where_.top);
                d.whole = d.dest;
                d.h_vel = 0;
                let mut position = who.h().length; // reverse engineer init. vel.
                let mut velocity = 0;
                let mut lil_frame = true;
                loop {
                    if lil_frame {
                        velocity += 1;
                    }
                    lil_frame = !lil_frame;
                    position -= velocity;
                    if position <= 0 {
                        break;
                    }
                }
                d.v_vel = -velocity;
                d.moving = false;
                d.count = -velocity; // count = initial velocity
                d.frame = 0;
                d.timer = 0;
                d.position = d.dest.bottom;
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.active = is_on;
            }

            DRIP => {
                let src = self.drip_src[0];
                let d = &mut self.dinahs[n];
                d.dest = src;
                center_rect_in_rect(&mut d.dest, where_);
                let dv = where_.top - d.dest.top;
                v_offset_rect(&mut d.dest, dv);
                d.whole = d.dest;
                d.h_vel = d.dest.top; // remember
                d.v_vel = 0;
                d.count = (who.h().delay * 6) / TICKS_PER_FRAME;
                d.frame = 3;
                d.timer = d.count;
                d.position = d.dest.top + who.h().length;
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on;
            }

            FISH => {
                let src = self.fish_src[0];
                self.even_frame = true;
                let d = &mut self.dinahs[n];
                d.dest = src;
                q_offset_rect(&mut d.dest, where_.left + 10, where_.top + 8);
                d.whole = d.dest;
                d.h_vel = (who.h().delay * 6) / TICKS_PER_FRAME;
                let mut position = who.g().height; // reverse engineer init. vel.
                let mut velocity = 0;
                let mut lil_frame = true;
                loop {
                    if lil_frame {
                        velocity += 1;
                    }
                    lil_frame = !lil_frame;
                    position -= velocity;
                    if position <= 0 {
                        break;
                    }
                }
                d.v_vel = -velocity;
                d.count = -velocity; // count = initial velocity
                d.frame = 0;
                d.timer = d.h_vel;
                d.position = d.dest.bottom;
                d.room = room;
                d.byte0 = byte0;
                d.byte1 = 0;
                d.moving = false;
                d.active = is_on;
            }

            _ => return -1,
        }

        self.num_dynamics += 1;

        self.num_dynamics - 1
    }

    /// The set-up AddDynamicObject repeats for the appliances (Mac Plus,
    /// TV, coffee maker, VCR, stereo, microwave): the screen or light's
    /// rect at (h, v), no motion.
    fn appliance_dinah(&mut self, n: usize, src: Rect, h: i32, v: i32, timer: i32, room: i32, byte0: i32, is_on: bool) {
        let d = &mut self.dinahs[n];
        d.dest = src;
        zero_rect_corner(&mut d.dest);
        q_offset_rect(&mut d.dest, h, v);
        d.whole = d.dest;
        d.h_vel = 0;
        d.v_vel = 0;
        d.count = 0;
        d.frame = 0;
        d.timer = timer;
        d.position = 0;
        d.room = room;
        d.byte0 = byte0;
        d.byte1 = 0;
        d.moving = false;
        d.active = is_on;
    }
}

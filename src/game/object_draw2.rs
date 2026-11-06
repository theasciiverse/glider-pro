//! ObjectDraw2.c: drawing transports, switches, lights, appliances, enemies
//! and clutter into the back map.

use super::object_draw::{
    DARK_FLESH_COLOR, INTENSE_BLUE_COLOR, INTENSE_GREEN_COLOR, K8_BROWN_COLOR, K8_DK_GRAY_COLOR, K8_DK_GRAY2_COLOR, K8_DK_RED2_COLOR,
    K8_EARTH_BLUE_COLOR, K8_GRAY2_COLOR, K8_LT_GRAY_COLOR, K8_LTST_GRAY_COLOR, K8_LTST_GRAY4_COLOR, K8_LTST_GRAY5_COLOR,
    K8_PISS_YELLOW_COLOR, K8_SKY_COLOR, K8_TAN_COLOR, K8_WHITE_COLOR, PALE_VIOLET_COLOR, get_time,
};
use super::*;

pub const INTENSE_YELLOW_COLOR: i32 = 5;

pub const BBQ_MASK_ID: i32 = 3900;
pub const UP_STAIRS_MASK_ID: i32 = 3901;
pub const TRUNK_MASK_ID: i32 = 3902;
pub const MAILBOX_RIGHT_MASK_ID: i32 = 3903;
pub const MAILBOX_LEFT_MASK_ID: i32 = 3904;
pub const DOOR_IN_LEFT_MASK_ID: i32 = 3905;
pub const DOOR_IN_RIGHT_MASK_ID: i32 = 3906;
pub const WINDOW_IN_LEFT_MASK_ID: i32 = 3907;
pub const WINDOW_IN_RIGHT_MASK_ID: i32 = 3908;
pub const HIP_LAMP_MASK_ID: i32 = 3909;
pub const DECO_LAMP_MASK_ID: i32 = 3910;
pub const GUITAR_MASK_ID: i32 = 3911;
pub const TV_MASK_ID: i32 = 3912;
pub const VCR_MASK_ID: i32 = 3913;
pub const STEREO_MASK_ID: i32 = 3914;
pub const MICROWAVE_MASK_ID: i32 = 3915;
pub const FIREPLACE_MASK_ID: i32 = 3916;
pub const BEAR_MASK_ID: i32 = 3917;
pub const VASE1_MASK_ID: i32 = 3918;
pub const VASE2_MASK_ID: i32 = 3919;
pub const MANHOLE_MASK_ID: i32 = 3920;
pub const BOOKS_MASK_ID: i32 = 3922;
pub const CLOUD_MASK_ID: i32 = 3921;
pub const RUG_MASK_ID: i32 = 3923;
pub const CHIMES_MASK_ID: i32 = 3924;
pub const CINDER_MASK_ID: i32 = 3925;
pub const FLOWER_BOX_MASK_ID: i32 = 3926;
pub const COBWEB_MASK_ID: i32 = 3927;
pub const COBWEB_PICT_ID: i32 = 3958;
pub const FLOWER_BOX_PICT_ID: i32 = 3959;
pub const CINDER_PICT_ID: i32 = 3960;
pub const CHIMES_PICT_ID: i32 = 3961;
pub const RUG_PICT_ID: i32 = 3962;
pub const BOOKS_PICT_ID: i32 = 3964;
pub const CLOUD_PICT_ID: i32 = 3965;
pub const BULLETIN_PICT_ID: i32 = 3966;
pub const MANHOLE_PICT_ID: i32 = 3967;
pub const VASE2_PICT_ID: i32 = 3968;
pub const VASE1_PICT_ID: i32 = 3969;
pub const CALENDAR_PICT_ID: i32 = 3970;
pub const MICROWAVE_PICT_ID: i32 = 3971;
pub const BEAR_PICT_ID: i32 = 3972;
pub const FIREPLACE_PICT_ID: i32 = 3973;
pub const OZMA_PICT_ID: i32 = 3975;
pub const WINDOW_EX_RIGHT_PICT_ID: i32 = 3977;
pub const WINDOW_EX_LEFT_PICT_ID: i32 = 3978;
pub const WINDOW_IN_RIGHT_PICT_ID: i32 = 3979;
pub const WINDOW_IN_LEFT_PICT_ID: i32 = 3980;
pub const DOOR_EX_LEFT_PICT_ID: i32 = 3981;
pub const DOOR_EX_RIGHT_PICT_ID: i32 = 3982;
pub const DOOR_IN_RIGHT_PICT_ID: i32 = 3983;
pub const DOOR_IN_LEFT_PICT_ID: i32 = 3984;
pub const MAILBOX_RIGHT_PICT_ID: i32 = 3985;
pub const MAILBOX_LEFT_PICT_ID: i32 = 3986;
pub const TRUNK_PICT_ID: i32 = 3987;
pub const BBQ_PICT_ID: i32 = 3988;
pub const STEREO_PICT_ID: i32 = 3989;
pub const VCR_PICT_ID: i32 = 3990;
pub const GUITAR_PICT_ID: i32 = 3991;
pub const TV_PICT_ID: i32 = 3992;
pub const DECO_LAMP_PICT_ID: i32 = 3993;
pub const HIP_LAMP_PICT_ID: i32 = 3994;
pub const FILING_CABINET_PICT_ID: i32 = 3995;
pub const DOWN_STAIRS_PICT_ID: i32 = 3996;
pub const UP_STAIRS_PICT_ID: i32 = 3997;

pub const MAILBOX_BASE: i32 = 296;
pub const MONTH_STRING_ID: i32 = 1005;

const TRACK_LIGHT_SPACING: i32 = 64;
const WINDOW_SILL_THICK: i32 = 7;

impl Game {
    /// GetIndString(str, id, index) from Glider PRO's own `STR#`
    /// resources (index from 1); empty if missing.
    pub fn get_ind_string(&self, id: i32, index: i32) -> String {
        let Some(a) = self.assets.as_ref() else { return String::new() };
        let Some(v) = a.app.get(&(*b"STR#", id as i16)) else { return String::new() };
        if v.len() < 2 || index < 1 {
            return String::new();
        }
        let count = u16::from_be_bytes([v[0], v[1]]) as i32;
        let mut at = 2usize;
        for i in 1..=count {
            let Some(&len) = v.get(at) else { break };
            let s = v.get(at + 1..at + 1 + len as usize).unwrap_or(&[]);
            if i == index {
                return crate::mac::mac_roman(s);
            }
            at += 1 + len as usize;
        }
        String::new()
    }

    /// The mailbox post shared by DrawMailboxLeft and DrawMailboxRight:
    /// columns from `left + first` on.
    fn draw_mailbox_post(&mut self, the_rect: &Rect, down: i32, first: i32) {
        let dark_gray_c = K8_DK_GRAY2_COLOR;
        let light_wood_c = K8_PISS_YELLOW_COLOR;
        let dark_wood_c = K8_BROWN_COLOR;
        let m = &mut self.back_src_map;
        let (l, b) = (the_rect.left + first, the_rect.bottom);
        if the_rect.bottom < down + MAILBOX_BASE {
            m.color_line(l, b, l, down + MAILBOX_BASE, dark_gray_c);
            m.color_line(l + 1, b, l + 1, down + MAILBOX_BASE + 1, light_wood_c);
            m.color_line(l + 2, b, l + 2, down + MAILBOX_BASE + 2, light_wood_c);
            m.color_line(l + 3, b, l + 3, down + MAILBOX_BASE + 3, light_wood_c);
            for i in 4..=12 {
                m.color_line(l + i, b, l + i, down + MAILBOX_BASE + 3, dark_wood_c);
            }
            m.color_line(l + 13, b, l + 13, down + MAILBOX_BASE + 3, dark_gray_c);
        }
    }

    /// A picture and its mask loaded into temporary maps the size of
    /// `srcRects[what]` and masked into the back map at `the_rect`.
    fn draw_temp_masked(&mut self, what: i32, pict_id: i32, mask_id: i32, the_rect: &Rect) {
        let bounds = self.src_rects[what as usize];
        let temp_map = self.graphic_map(pict_id, bounds);
        let temp_mask = self.graphic_map(mask_id, bounds);
        self.back_src_map.copy_mask(&temp_map, &temp_mask, bounds, bounds, *the_rect);
    }

    //--------------------------------------------------------------  DrawMailboxLeft

    pub fn draw_mailbox_left(&mut self, the_rect: &Rect, down: i32) {
        self.draw_mailbox_post(the_rect, down, 49);
        self.draw_temp_masked(MAILBOX_LF, MAILBOX_LEFT_PICT_ID, MAILBOX_LEFT_MASK_ID, the_rect);
    }

    //--------------------------------------------------------------  DrawMailboxRight

    pub fn draw_mailbox_right(&mut self, the_rect: &Rect, down: i32) {
        self.draw_mailbox_post(the_rect, down, 34);
        self.draw_temp_masked(MAILBOX_RT, MAILBOX_RIGHT_PICT_ID, MAILBOX_RIGHT_MASK_ID, the_rect);
    }

    //--------------------------------------------------------------  DrawSimpleTransport

    pub fn draw_simple_transport(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.trans_src_map, &self.trans_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawInvisTransport

    pub fn draw_invis_transport(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, 32);
    }

    //--------------------------------------------------------------  DrawLightSwitch

    pub fn draw_light_switch(&mut self, the_rect: &Rect, state: bool) {
        let sr = if state { self.light_switch_src[0] } else { self.light_switch_src[1] };
        self.back_src_map.copy_bits(&self.switch_src_map, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawMachineSwitch

    pub fn draw_machine_switch(&mut self, the_rect: &Rect, state: bool) {
        let sr = if state { self.machine_switch_src[0] } else { self.machine_switch_src[1] };
        self.back_src_map.copy_bits(&self.switch_src_map, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawThermostat

    pub fn draw_thermostat(&mut self, the_rect: &Rect, state: bool) {
        let sr = if state { self.thermostat_src[0] } else { self.thermostat_src[1] };
        self.back_src_map.copy_bits(&self.switch_src_map, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawPowerSwitch

    pub fn draw_power_switch(&mut self, the_rect: &Rect, state: bool) {
        let sr = if state { self.power_src[0] } else { self.power_src[1] };
        self.back_src_map.copy_bits(&self.switch_src_map, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawKnifeSwitch

    pub fn draw_knife_switch(&mut self, the_rect: &Rect, state: bool) {
        let sr = if state { self.knife_switch_src[0] } else { self.knife_switch_src[1] };
        self.back_src_map.copy_bits(&self.switch_src_map, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawInvisibleSwitch

    pub fn draw_invisible_switch(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, INTENSE_GREEN_COLOR);
    }

    //--------------------------------------------------------------  DrawTrigger

    pub fn draw_trigger(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, INTENSE_BLUE_COLOR);
    }

    //--------------------------------------------------------------  DrawSoundTrigger

    pub fn draw_sound_trigger(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_rect(*the_rect, INTENSE_YELLOW_COLOR);
    }

    //--------------------------------------------------------------  DrawSimpleLight

    pub fn draw_simple_light(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.light_src_map, &self.light_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawFlourescent

    pub fn draw_flourescent(&mut self, the_rect: &Rect) {
        let gray_c = K8_LT_GRAY_COLOR;
        let gray2_c = K8_LTST_GRAY5_COLOR;
        let gray3_c = K8_LTST_GRAY4_COLOR;
        let gray4_c = K8_LTST_GRAY_COLOR;
        let violet_c = PALE_VIOLET_COLOR;
        let r = *the_rect;
        let m = &mut self.back_src_map;

        m.color_line(r.left + 16, r.top, r.right - 17, r.top, gray_c);
        m.color_line(r.left + 16, r.top + 1, r.right - 17, r.top + 1, gray2_c);
        m.color_line(r.left + 16, r.top + 2, r.right - 17, r.top + 2, gray2_c);
        m.color_line(r.left + 16, r.top + 3, r.right - 17, r.top + 3, gray3_c);
        m.color_line(r.left + 16, r.top + 4, r.right - 17, r.top + 4, gray4_c);
        m.color_line(r.left + 16, r.top + 5, r.right - 17, r.top + 5, violet_c);
        m.color_line(r.left + 16, r.top + 6, r.right - 17, r.top + 6, K8_WHITE_COLOR);
        m.color_line(r.left + 16, r.top + 7, r.right - 17, r.top + 7, K8_WHITE_COLOR);
        m.color_line(r.left + 16, r.top + 8, r.right - 17, r.top + 8, K8_WHITE_COLOR);
        m.color_line(r.left + 16, r.top + 9, r.right - 17, r.top + 9, K8_WHITE_COLOR);
        m.color_line(r.left + 16, r.top + 10, r.right - 17, r.top + 10, K8_WHITE_COLOR);
        m.color_line(r.left + 16, r.top + 11, r.right - 17, r.top + 11, violet_c);

        let src1 = self.flourescent_src1;
        let mut part_rect = src1;
        zero_rect_corner(&mut part_rect);
        q_offset_rect(&mut part_rect, r.left, r.top);
        self.back_src_map.copy_mask(&self.light_src_map, &self.light_mask_map, src1, src1, part_rect);

        let src2 = self.flourescent_src2;
        let mut part_rect = src2;
        zero_rect_corner(&mut part_rect);
        let pr = part_rect.right;
        q_offset_rect(&mut part_rect, -pr, 0);
        q_offset_rect(&mut part_rect, r.right, r.top);
        self.back_src_map.copy_mask(&self.light_src_map, &self.light_mask_map, src2, src2, part_rect);
    }

    //--------------------------------------------------------------  DrawTrackLight

    pub fn draw_track_light(&mut self, the_rect: &Rect) {
        let gray_c = K8_LT_GRAY_COLOR;
        let gray2_c = K8_GRAY2_COLOR;
        let gray3_c = K8_LTST_GRAY4_COLOR;
        let gray4_c = K8_DK_GRAY_COLOR;
        let r = *the_rect;
        let m = &mut self.back_src_map;

        m.color_line(r.left, r.top - 3, r.right - 1, r.top - 3, gray2_c);
        m.color_line(r.left, r.top - 2, r.right - 1, r.top - 2, gray_c);
        m.color_line(r.left, r.top - 1, r.right - 1, r.top - 1, gray_c);
        m.color_line(r.left, r.top, r.right - 1, r.top, gray3_c);
        m.color_line(r.left, r.top + 1, r.right - 1, r.top + 1, gray4_c);
        m.color_line(r.left, r.top + 2, r.right - 1, r.top + 2, gray3_c);

        let src = self.track_light_src;
        let mut part_rect = src[0]; // left most track light
        zero_rect_corner(&mut part_rect);
        q_offset_rect(&mut part_rect, r.left, r.top);
        let mut which = 0usize;
        self.back_src_map.copy_mask(&self.light_src_map, &self.light_mask_map, src[which], src[which], part_rect);

        part_rect = src[0]; // right most track light
        zero_rect_corner(&mut part_rect);
        let pr = part_rect.right;
        q_offset_rect(&mut part_rect, -pr, 0);
        q_offset_rect(&mut part_rect, r.right, r.top);
        which = 2;
        self.back_src_map.copy_mask(&self.light_src_map, &self.light_mask_map, src[which], src[which], part_rect);

        let how_many = ((rect_wide(&r) - rect_wide(&src[0])) / TRACK_LIGHT_SPACING) - 1;
        if how_many > 0 {
            which = 0;
            let spread = (rect_wide(&r) - rect_wide(&src[0])) / (how_many + 1);
            for i in 0..how_many {
                part_rect = src[0]; // filler track lights
                zero_rect_corner(&mut part_rect);
                q_offset_rect(&mut part_rect, r.left, r.top);
                q_offset_rect(&mut part_rect, spread * (i + 1), 0);
                which += 1;
                if which >= NUM_TRACK_LIGHTS as usize {
                    which = 0;
                }
                self.back_src_map.copy_mask(&self.light_src_map, &self.light_mask_map, src[which], src[which], part_rect);
            }
        }
    }

    //--------------------------------------------------------------  DrawInvisLight

    pub fn draw_invis_light(&mut self, the_rect: &Rect) {
        self.back_src_map.color_frame_oval(*the_rect, 17);
    }

    //--------------------------------------------------------------  DrawSimpleAppliance

    pub fn draw_simple_appliance(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.appliance_src_map, &self.appliance_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawMacPlus

    pub fn draw_mac_plus(&mut self, the_rect: &Rect, is_on: bool, is_lit: bool) {
        if is_lit {
            let sr = self.src_rects[MAC_PLUS as usize];
            self.back_src_map.copy_mask(&self.appliance_src_map, &self.appliance_mask_map, sr, sr, *the_rect);
        }

        let mut screen = self.plus_screen1;
        zero_rect_corner(&mut screen);
        q_offset_rect(&mut screen, the_rect.left + 10, the_rect.top + 7);
        let sr = if is_on { self.plus_screen2 } else { self.plus_screen1 };
        self.back_src_map.copy_bits(&self.appliance_src_map, sr, screen);
    }

    //--------------------------------------------------------------  DrawTV

    pub fn draw_tv(&mut self, the_rect: &Rect, is_on: bool, is_lit: bool) {
        if is_lit {
            self.draw_temp_masked(TV, TV_PICT_ID, TV_MASK_ID, the_rect);
        }

        let mut bounds = self.tv_screen1;
        zero_rect_corner(&mut bounds);
        q_offset_rect(&mut bounds, the_rect.left + 17, the_rect.top + 10);
        let sr = if is_on { self.tv_screen2 } else { self.tv_screen1 };
        self.back_src_map.copy_bits(&self.appliance_src_map, sr, bounds);
    }

    //--------------------------------------------------------------  DrawCoffee

    pub fn draw_coffee(&mut self, the_rect: &Rect, is_on: bool, is_lit: bool) {
        if is_lit {
            let sr = self.src_rects[COFFEE as usize];
            self.back_src_map.copy_mask(&self.appliance_src_map, &self.appliance_mask_map, sr, sr, *the_rect);
        }

        let mut light = self.coffee_light1;
        zero_rect_corner(&mut light);
        q_offset_rect(&mut light, the_rect.left + 32, the_rect.top + 57);
        let sr = if is_on { self.coffee_light2 } else { self.coffee_light1 };
        self.back_src_map.copy_bits(&self.appliance_src_map, sr, light);
    }

    //--------------------------------------------------------------  DrawOutlet

    pub fn draw_outlet(&mut self, the_rect: &Rect) {
        let sr = self.src_rects[OUTLET as usize];
        self.back_src_map.copy_mask(&self.appliance_src_map, &self.appliance_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawVCR

    pub fn draw_vcr(&mut self, the_rect: &Rect, is_on: bool, is_lit: bool) {
        if is_lit {
            self.draw_temp_masked(VCR, VCR_PICT_ID, VCR_MASK_ID, the_rect);
        }

        let mut bounds = self.vcr_time1;
        zero_rect_corner(&mut bounds);
        q_offset_rect(&mut bounds, the_rect.left + 64, the_rect.top + 6);
        let sr = if is_on { self.vcr_time2 } else { self.vcr_time1 };
        self.back_src_map.copy_bits(&self.appliance_src_map, sr, bounds);
    }

    //--------------------------------------------------------------  DrawStereo

    pub fn draw_stereo(&mut self, the_rect: &Rect, is_on: bool, is_lit: bool) {
        if is_lit {
            self.draw_temp_masked(STEREO, STEREO_PICT_ID, STEREO_MASK_ID, the_rect);
        }

        let mut bounds = self.stereo_light1;
        zero_rect_corner(&mut bounds);
        q_offset_rect(&mut bounds, the_rect.left + 56, the_rect.top + 20);
        let sr = if is_on { self.stereo_light2 } else { self.stereo_light1 };
        self.back_src_map.copy_bits(&self.appliance_src_map, sr, bounds);
    }

    //--------------------------------------------------------------  DrawMicrowave

    pub fn draw_microwave(&mut self, the_rect: &Rect, is_on: bool, is_lit: bool) {
        if is_lit {
            self.draw_temp_masked(MICROWAVE, MICROWAVE_PICT_ID, MICROWAVE_MASK_ID, the_rect);
        }

        let mut bounds = self.micro_on;
        zero_rect_corner(&mut bounds);
        q_offset_rect(&mut bounds, the_rect.left + 14, the_rect.top + 13);
        let sr = if is_on {
            Some(self.micro_on)
        } else if is_lit {
            Some(self.micro_off)
        } else {
            None
        };
        if let Some(sr) = sr {
            self.back_src_map.copy_bits(&self.appliance_src_map, sr, bounds);
            q_offset_rect(&mut bounds, 16, 0);
            self.back_src_map.copy_bits(&self.appliance_src_map, sr, bounds);
            q_offset_rect(&mut bounds, 16, 0);
            self.back_src_map.copy_bits(&self.appliance_src_map, sr, bounds);
        }
    }

    //--------------------------------------------------------------  DrawBalloon

    pub fn draw_balloon(&mut self, the_rect: &Rect) {
        let sr = self.balloon_src[1];
        self.back_src_map.copy_mask(&self.balloon_src_map, &self.balloon_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawCopter

    pub fn draw_copter(&mut self, the_rect: &Rect) {
        let sr = self.copter_src[1];
        self.back_src_map.copy_mask(&self.copter_src_map, &self.copter_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawDart

    pub fn draw_dart(&mut self, the_rect: &Rect, which: i32) {
        let sr = if which == DART_LF { self.dart_src[0] } else { self.dart_src[2] };
        self.back_src_map.copy_mask(&self.dart_src_map, &self.dart_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawBall

    pub fn draw_ball(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.ball_src_map, &self.ball_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawFish

    pub fn draw_fish(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.enemy_src_map, &self.enemy_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawDrip

    pub fn draw_drip(&mut self, the_rect: &Rect) {
        let sr = self.drip_src[3];
        self.back_src_map.copy_mask(&self.drip_src_map, &self.drip_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawMirror

    pub fn draw_mirror(&mut self, mirror: &Rect) {
        let gray_c = K8_DK_GRAY2_COLOR;
        let m = &mut self.back_src_map;

        let mut temp_rect = *mirror;
        m.color_rect(temp_rect, K8_WHITE_COLOR);
        m.color_frame_rect(temp_rect, gray_c);
        inset_rect(&mut temp_rect, 1, 1);
        m.color_frame_rect(temp_rect, K8_EARTH_BLUE_COLOR);
        inset_rect(&mut temp_rect, 1, 1);
        m.color_frame_rect(temp_rect, K8_EARTH_BLUE_COLOR);
        inset_rect(&mut temp_rect, 1, 1);
        m.color_frame_rect(temp_rect, gray_c);
    }

    //--------------------------------------------------------------  DrawSimpleClutter

    pub fn draw_simple_clutter(&mut self, what: i32, the_rect: &Rect) {
        let sr = self.src_rects[what as usize];
        self.back_src_map.copy_mask(&self.clutter_src_map, &self.clutter_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawFlower

    pub fn draw_flower(&mut self, the_rect: &Rect, which: i32) {
        let sr = self.flower_src[which as usize];
        self.back_src_map.copy_mask(&self.clutter_src_map, &self.clutter_mask_map, sr, sr, *the_rect);
    }

    //--------------------------------------------------------------  DrawWallWindow

    pub fn draw_wall_window(&mut self, window: &Rect) {
        let brown_c = K8_BROWN_COLOR;
        let tan_c = K8_TAN_COLOR;
        let dkst_red_c = K8_DK_RED2_COLOR;
        let m = &mut self.back_src_map;

        let mut temp_rect = *window;
        inset_rect(&mut temp_rect, 3, 0);
        m.color_rect(temp_rect, brown_c);
        m.hilite_rect(temp_rect, tan_c, dkst_red_c);

        temp_rect = *window; // top sill
        temp_rect.bottom = temp_rect.top + WINDOW_SILL_THICK;
        temp_rect.left += 1;
        temp_rect.right -= 1;
        m.color_rect(temp_rect, brown_c);
        m.hilite_rect(temp_rect, tan_c, dkst_red_c);
        temp_rect.left -= 1;
        temp_rect.right += 1;
        temp_rect.top += 2;
        temp_rect.bottom -= 2;
        m.color_rect(temp_rect, brown_c);
        m.hilite_rect(temp_rect, tan_c, dkst_red_c);

        temp_rect = *window; // bottom sill
        temp_rect.top = temp_rect.bottom - WINDOW_SILL_THICK;
        q_offset_rect(&mut temp_rect, 0, -4);
        temp_rect.left += 1;
        temp_rect.right -= 1;
        m.color_rect(temp_rect, brown_c);
        m.hilite_rect(temp_rect, tan_c, dkst_red_c);
        temp_rect.left -= 1;
        temp_rect.right += 1;
        temp_rect.top += 2;
        temp_rect.bottom -= 2;
        m.color_rect(temp_rect, brown_c);
        m.hilite_rect(temp_rect, tan_c, dkst_red_c);

        temp_rect = *window; // inside frame
        temp_rect.left += 8;
        temp_rect.right -= 8;
        temp_rect.top += 11;
        temp_rect.bottom -= 15;
        m.hilite_rect(temp_rect, dkst_red_c, tan_c);

        let half_way = (temp_rect.top + temp_rect.bottom) / 2;

        let mut temp_rect2 = temp_rect; // top pane
        temp_rect2.bottom = half_way + 2;
        inset_rect(&mut temp_rect2, 5, 5);
        m.hilite_rect(temp_rect2, dkst_red_c, tan_c);
        inset_rect(&mut temp_rect2, 1, 1);
        m.color_rect(temp_rect2, K8_SKY_COLOR);

        temp_rect2 = temp_rect; // bottom pane
        temp_rect2.top = half_way - 3;
        inset_rect(&mut temp_rect2, 5, 5);
        m.hilite_rect(temp_rect2, dkst_red_c, tan_c);
        inset_rect(&mut temp_rect2, 1, 1);
        m.color_rect(temp_rect2, K8_SKY_COLOR);

        m.color_line(temp_rect2.left - 5, temp_rect2.top - 7, temp_rect2.right + 5, temp_rect2.top - 7, tan_c);
    }

    //--------------------------------------------------------------  DrawCalendar

    pub fn draw_calendar(&mut self, the_rect: &Rect) {
        let Some(the_picture) = self.picture(CALENDAR_PICT_ID) else {
            log::warn!("DrawCalendar: PICT {CALENDAR_PICT_ID} missing");
            return;
        };
        let mut bounds = Rect::new(0, 0, the_picture.w, the_picture.h);
        q_offset_rect(&mut bounds, the_rect.left, the_rect.top);
        self.back_src_map.draw_picture(&the_picture, bounds);

        // The month's name in 9-point bold applFont: terminal text,
        // StringWidth taken as 5 pixels a letter.
        let time_rec = get_time();
        let month_str = self.get_ind_string(MONTH_STRING_ID, time_rec.month);
        let width = month_str.chars().count() as i32 * 5;
        self.label(the_rect.left + ((64 - width) / 2), the_rect.top + 55, month_str, crate::gfx::clut8(DARK_FLESH_COLOR), 0);
    }

    //--------------------------------------------------------------  DrawBulletin

    pub fn draw_bulletin(&mut self, the_rect: &Rect) {
        let Some(the_picture) = self.picture(BULLETIN_PICT_ID) else {
            log::warn!("DrawBulletin: PICT {BULLETIN_PICT_ID} missing");
            return;
        };
        let mut bounds = Rect::new(0, 0, the_picture.w, the_picture.h);
        q_offset_rect(&mut bounds, the_rect.left, the_rect.top);
        self.back_src_map.draw_picture(&the_picture, bounds);
    }

    //--------------------------------------------------------------  DrawPictObject

    pub fn draw_pict_object(&mut self, what: i32, the_rect: &Rect) {
        let pict_id = match what {
            FILING_CABINET => FILING_CABINET_PICT_ID,
            DOWN_STAIRS => DOWN_STAIRS_PICT_ID,
            DOOR_EX_RT => DOOR_EX_RIGHT_PICT_ID,
            DOOR_EX_LF => DOOR_EX_LEFT_PICT_ID,
            WINDOW_EX_RT => WINDOW_EX_RIGHT_PICT_ID,
            WINDOW_EX_LF => WINDOW_EX_LEFT_PICT_ID,
            OZMA => OZMA_PICT_ID,
            _ => {
                log::warn!("DrawPictObject: no picture for object {what}");
                return;
            }
        };

        let Some(the_picture) = self.picture(pict_id) else {
            log::warn!("DrawPictObject: PICT {pict_id} missing");
            return;
        };

        let mut bounds = self.src_rects[what as usize];
        q_offset_rect(&mut bounds, the_rect.left, the_rect.top);
        self.back_src_map.draw_picture(&the_picture, bounds);
    }

    //--------------------------------------------------------------  DrawPictWithMaskObject

    pub fn draw_pict_with_mask_object(&mut self, what: i32, the_rect: &Rect) {
        let (pict_id, mask_id) = match what {
            COBWEB => (COBWEB_PICT_ID, COBWEB_MASK_ID),
            CLOUD => (CLOUD_PICT_ID, CLOUD_MASK_ID),
            _ => {
                log::warn!("DrawPictWithMaskObject: no picture for object {what}");
                return;
            }
        };
        self.draw_temp_masked(what, pict_id, mask_id, the_rect);
    }

    //--------------------------------------------------------------  DrawPictSansWhiteObject

    pub fn draw_pict_sans_white_object(&mut self, what: i32, the_rect: &Rect) {
        let pict_id = match what {
            BBQ => BBQ_PICT_ID,
            TRUNK => TRUNK_PICT_ID,
            MANHOLE => MANHOLE_PICT_ID,
            BOOKS => BOOKS_PICT_ID,
            UP_STAIRS => UP_STAIRS_PICT_ID,
            DOOR_IN_LF => DOOR_IN_LEFT_PICT_ID,
            DOOR_IN_RT => DOOR_IN_RIGHT_PICT_ID,
            WINDOW_IN_LF => WINDOW_IN_LEFT_PICT_ID,
            WINDOW_IN_RT => WINDOW_IN_RIGHT_PICT_ID,
            HIP_LAMP => HIP_LAMP_PICT_ID,
            DECO_LAMP => DECO_LAMP_PICT_ID,
            GUITAR => GUITAR_PICT_ID,
            CINDER_BLOCK => CINDER_PICT_ID,
            FLOWER_BOX => FLOWER_BOX_PICT_ID,
            FIREPLACE => FIREPLACE_PICT_ID,
            BEAR => BEAR_PICT_ID,
            VASE1 => VASE1_PICT_ID,
            VASE2 => VASE2_PICT_ID,
            RUG => RUG_PICT_ID,
            CHIMES => CHIMES_PICT_ID,
            _ => {
                log::warn!("DrawPictSansWhiteObject: no picture for object {what}");
                return;
            }
        };

        let bounds = self.src_rects[what as usize];
        let temp_map = self.graphic_map(pict_id, bounds);
        self.back_src_map.copy_bits_transparent(&temp_map, bounds, *the_rect);
    }

    //--------------------------------------------------------------  DrawCustPictSansWhite

    pub fn draw_cust_pict_sans_white(&mut self, pict_id: i32, the_rect: &Rect) {
        let mut bounds = *the_rect;
        zero_rect_corner(&mut bounds);
        let temp_map = self.graphic_map(pict_id, bounds);
        self.back_src_map.copy_bits_transparent(&temp_map, bounds, *the_rect);
    }
}

//! Glider PRO's play, translated from Calhoun's C (`Sources/`).
//!
//! Conventions (for every module here):
//! - One `Game` holds the C globals, named as in C in snake_case
//!   (`theGlider` is `gliders[0]`, `theGlider2` is `gliders[1]`).
//! - Each C file is a module of `impl Game` blocks; a C function `FooBar`
//!   is `fn foo_bar(&mut self, ...)`. A `gliderPtr` argument is the
//!   glider's index (`g: usize`); a `hotPtr` is an index into `hot_spots`.
//! - `#define`s of GliderDefines.h are in `defs` (`kFloorVent` is
//!   `FLOOR_VENT`); a file's own `#define`s are consts in its module.
//! - `short`, `long` and `Byte` are `i32`; `Boolean` is `bool`.
//! - GWorlds are `gfx::Map`s, drawn into by field:
//!   `self.back_src_map.copy_mask(&self.blower_src_map, ...)`.
//! - Text is not drawn into maps; see `label`.

#![allow(dead_code, clippy::too_many_arguments, clippy::collapsible_if, clippy::collapsible_else_if)]

pub mod defs;
pub mod rect_utils;
pub mod player;
pub mod interactions;
pub mod input;
pub mod rubber_bands;
pub mod grease;
pub mod trip;
pub mod triggers;
pub mod dynamics;
pub mod dynamics2;
pub mod dynamics3;
pub mod dynamic_maps;
pub mod render;
pub mod objects;
pub mod object_rects;
pub mod object_draw_all;
pub mod room;
pub mod house_c;
pub mod room_graphics;
pub mod object_draw;
pub mod object_draw2;
pub mod structures_init;
pub mod structures_init2;
pub mod play;
pub mod transit;
pub mod transitions;
pub mod banner;
pub mod scoreboard;
pub mod game_over;
pub mod high_scores;
pub mod modes;
pub mod music;
pub mod sound;
pub mod utilities;

use crate::assets::Assets;
use crate::gfx::{Map, Pattern, Rect, Region};
use crate::house::{House, Object, Point, Room};
pub use defs::*;
pub use rect_utils::*;

// ---------------------------------------------------------------- records

/// `gliderType`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Glider {
    pub src: Rect,
    pub mask: Rect,
    pub dest: Rect,
    pub whole: Rect,
    pub dest_shadow: Rect,
    pub whole_shadow: Rect,
    pub clip: Rect,
    pub entered_rect: Rect,
    /// Key codes, as `input::Key` values.
    pub left_key: i32,
    pub right_key: i32,
    pub batt_key: i32,
    pub band_key: i32,
    pub h_vel: i32,
    pub v_vel: i32,
    pub was_h_vel: i32,
    pub was_v_vel: i32,
    pub v_desired_vel: i32,
    pub h_desired_vel: i32,
    pub mode: i32,
    pub frame: i32,
    pub was_mode: i32,
    pub facing: bool,
    pub tipped: bool,
    pub sliding: bool,
    pub ignore_left: bool,
    pub ignore_right: bool,
    pub fire_held: bool,
    /// `which`: true for player one (kPlayer1).
    pub which: bool,
    pub held_left: bool,
    pub held_right: bool,
    pub dont_draw: bool,
    pub ignore_ground: bool,
}

/// `hotObject`.
#[derive(Clone, Copy, Debug, Default)]
pub struct HotObject {
    pub bounds: Rect,
    pub action: i32,
    pub who: i32,
    pub is_on: bool,
    pub still_over: bool,
    pub do_scrutinize: bool,
}

/// `savedType`: the background saved under an animated object.
#[derive(Clone, Default)]
pub struct SavedType {
    pub dest: Rect,
    pub map: Option<Map>,
    pub where_: i32,
    pub who: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sparkle {
    pub bounds: Rect,
    pub mode: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FlyingPt {
    pub dest: Rect,
    pub whole: Rect,
    pub start: i32,
    pub stop: i32,
    pub mode: i32,
    pub loops: i32,
    pub h_vel: i32,
    pub v_vel: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Flame {
    pub dest: Rect,
    pub src: Rect,
    pub mode: i32,
    pub who: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Pendulum {
    pub dest: Rect,
    pub src: Rect,
    pub mode: i32,
    pub where_: i32,
    pub who: i32,
    pub link: i32,
    pub to_or_fro: bool,
    pub active: bool,
}

/// `boundsType`: which walls a room has.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundsType {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Band {
    pub dest: Rect,
    pub mode: i32,
    pub count: i32,
    pub h_vel: i32,
    pub v_vel: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Links {
    pub src_room: i32,
    pub src_obj: i32,
    pub dest_room: i32,
    pub dest_obj: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Grease {
    pub dest: Rect,
    pub map_num: i32,
    pub mode: i32,
    pub who: i32,
    pub where_: i32,
    pub start: i32,
    pub stop: i32,
    pub frame: i32,
    pub hot_num: i32,
    pub is_right: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Star {
    pub dest: Rect,
    pub src: Rect,
    pub mode: i32,
    pub who: i32,
    pub link: i32,
    pub where_: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Shred {
    pub bounds: Rect,
    pub frame: i32,
}

/// `dynaType`: a moving thing (toast, balloon, dart, ...).
#[derive(Clone, Copy, Debug, Default)]
pub struct Dyna {
    pub dest: Rect,
    pub whole: Rect,
    pub h_vel: i32,
    pub v_vel: i32,
    pub type_: i32,
    pub count: i32,
    pub frame: i32,
    pub timer: i32,
    pub position: i32,
    pub room: i32,
    pub byte0: i32,
    pub byte1: i32,
    pub moving: bool,
    pub active: bool,
}

/// `objDataType`: an object of the rooms around, in the master list.
#[derive(Clone, Copy, Debug, Default)]
pub struct ObjData {
    pub room_num: i32,
    pub object_num: i32,
    pub room_link: i32,
    pub object_link: i32,
    pub local_link: i32,
    pub hot_num: i32,
    pub dyna_num: i32,
    pub the_object: Object,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DemoKey {
    pub frame: i32,
    pub key: u8,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RetroLink {
    pub room: i32,
    pub object: i32,
}

/// Play.c's `phoneType` (the telephone and the chimes).
#[derive(Clone, Copy, Debug, Default)]
pub struct Phone {
    pub next_ring: i32,
    pub rings: i32,
    pub delay: i32,
}

/// Triggers.c's `trigType`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Trig {
    pub object: i32,
    pub room: i32,
    pub index: i32,
    pub timer: i32,
    pub what: i32,
    pub armed: bool,
}

/// GameOver.c's `pageType`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Page {
    pub dest: Rect,
    pub was: Rect,
    pub frame: i32,
    pub counter: i32,
    pub stuck: bool,
}

/// Marquee.h's `marquee` (the editor's selection; kept for GameOver etc.).
#[derive(Clone, Copy, Debug, Default)]
pub struct Marquee {
    pub pats: [Pattern; 7],
    pub bounds: Rect,
    pub handle: Rect,
    pub index: i32,
    pub direction: i32,
    pub dist: i32,
    pub active: bool,
    pub paused: bool,
    pub handled: bool,
}

/// A high score table (`scoresType`).
#[derive(Clone, Debug, Default)]
pub struct Scores {
    pub banner: String,
    pub names: [String; MAX_SCORES as usize],
    pub scores: [i32; MAX_SCORES as usize],
    pub time_stamps: [u32; MAX_SCORES as usize],
    pub levels: [i32; MAX_SCORES as usize],
}

/// Text that the C draws with DrawString during play: shown as terminal
/// text over the picture at the room position (x, baseline y).
#[derive(Clone, Debug)]
pub struct Label {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub color: crate::mac::Rgb,
    /// Frames left to show it (0: until the next room is drawn).
    pub frames: i32,
}

/// Keys held this frame, as Input.c reads them with GetKeys.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyState {
    pub left: bool,
    pub right: bool,
    pub batt: bool,
    pub band: bool,
    /// Second player (two-player game).
    pub left2: bool,
    pub right2: bool,
    pub batt2: bool,
    pub band2: bool,
    pub escape: bool,
    pub tab: bool,
    pub quit: bool,
    /// Delete (kDeleteKeyMap): in a two-player game, gives up the glider
    /// left waiting in limbo (ForceKillGlider).
    pub delete: bool,
}

/// What the game needs from the terminal while the C code waits inside a
/// function (banners, transitions, game over): show the main window and
/// let time pass, reading the keys.
pub trait ScreenIo {
    /// Show `g.main_window_map` (and labels) now; play queued sounds.
    fn present(&mut self, g: &mut Game);
    /// Let `ticks` 60ths of a second pass, refreshing `g.keys` and
    /// servicing sound; true if a key or button was pressed meanwhile.
    fn wait(&mut self, ticks: i32, g: &mut Game) -> bool;
    /// For taking the terminal back out of the game.
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
}

// ---------------------------------------------------------------- the game

/// Every global of the C files this game keeps, by the file that defines it.
#[derive(Default)]
pub struct Game {
    pub assets: Option<std::rc::Rc<Assets>>,

    // --- House.c / HouseIO.c
    /// `thisHouse`.
    pub house: House,
    pub this_house_name: String,
    pub links_list: Vec<Links>,
    pub src_locations: [i32; MAX_ROOM_OBS as usize],
    pub dest_locations: [i32; MAX_ROOM_OBS as usize],
    pub was_floor: i32,
    pub was_suite: i32,
    pub retro_link_list: [RetroLink; MAX_ROOM_OBS as usize],
    pub house_unlocked: bool,
    /// `gameDirty`: the house (its high scores) changed and wants saving.
    pub game_dirty: bool,
    /// HouseIO.c's `wasHouseVersion`.
    pub was_house_version: i32,

    // --- Room.c
    /// `thisRoom`: a copy of the current room (the C points into the
    /// house; write changes back with `copy_this_room_to_room`).
    pub this_room: Room,
    pub back_src_rect: Rect,
    pub back_src_map: Map,
    pub number_rooms: i32,
    pub this_room_number: i32,
    pub previous_room: i32,
    pub left_thresh: i32,
    pub right_thresh: i32,
    pub last_background: i32,
    pub no_room_at_all: bool,
    pub left_open: bool,
    pub right_open: bool,
    pub top_open: bool,
    pub bottom_open: bool,

    // --- RoomGraphics.c
    pub supp_src_rect: Rect,
    pub supp_src_map: Map,
    pub local_rooms_dest: [Rect; 9],
    pub house_rect: Rect,
    pub num_neighbors: i32,
    pub num_lights: i32,
    pub this_tiles: [i32; NUM_TILES as usize],
    pub local_numbers: [i32; 9],
    pub this_background: i32,
    pub is_structure: [bool; 9],
    pub ward_bit_set: bool,

    // --- MainWindow.c
    pub work_src_rect: Rect,
    pub work_src_map: Map,
    pub main_window_rect: Rect,
    pub play_origin_h: i32,
    pub play_origin_v: i32,
    pub splash_origin_h: i32,
    pub splash_origin_v: i32,
    pub the_mode: i32,

    // --- Play.c
    pub the_phone: Phone,
    pub the_chimes: Phone,
    pub glid_src_rect: Rect,
    pub just_rooms_rect: Rect,
    pub glid_src_map: Map,
    pub glid2_src_map: Map,
    pub glid_mask_map: Map,
    pub game_frame: i32,
    pub battery_total: i32,
    pub bands_total: i32,
    pub foil_total: i32,
    pub mortals: i32,
    pub playing: bool,
    pub even_frame: bool,
    pub two_player_game: bool,
    pub show_foil: bool,
    pub demo_going: bool,
    pub do_background: bool,
    pub player_suicide: bool,
    pub phone_bit_set: bool,
    pub tv_on: bool,
    /// NewGame's `wasPlayMusicPref`, kept for `play_game_finish`.
    pub was_play_music_pref: bool,

    // --- Player.c
    /// `theGlider` and `theGlider2`.
    pub gliders: [Glider; 2],
    pub shadow_src_rect: Rect,
    pub shadow_src_map: Map,
    pub shadow_mask_map: Map,
    pub shadow_src: [Rect; NUM_SHADOW_SRC_RECTS as usize],
    pub glider_src: [Rect; NUM_GLIDER_SRC_RECTS as usize],
    pub trans_rect: Rect,
    pub the_score: i32,
    pub fade_in_sequence: [i32; LAST_FADE_SEQUENCE as usize],
    pub right_clip: i32,
    pub left_clip: i32,
    pub trans_room: i32,
    pub shadow_visible: bool,
    pub one_player_left: bool,
    pub player_dead: bool,

    // --- Interactions.c
    pub other_player_escaped: i32,
    pub active_rect_escaped: i32,

    // --- Dynamics.c, Dynamics3.c
    pub bread_src: [Rect; NUM_BREAD_PICTS as usize],
    /// `dinahs`, kMaxDynamicObs long.
    pub dinahs: Vec<Dyna>,
    pub num_dynamics: i32,

    // --- DynamicMaps.c
    pub sparkles: Vec<Sparkle>,
    pub flying_points: Vec<FlyingPt>,
    pub flames: Vec<Flame>,
    pub tiki_flames: Vec<Flame>,
    pub bbq_coals: Vec<Flame>,
    pub pendulums: Vec<Pendulum>,
    pub the_stars: Vec<Star>,
    pub shreds: Vec<Shred>,
    pub points_src: [Rect; 15],
    pub num_sparkles: i32,
    pub num_flying_pts: i32,
    pub num_chimes: i32,
    pub num_flames: i32,
    pub num_saved_maps: i32,
    pub num_tiki_flames: i32,
    pub num_coals: i32,
    pub num_pendulums: i32,
    pub clock_frame: i32,
    pub num_stars: i32,
    pub num_shredded: i32,

    // --- Objects.c: source maps and rects (filled by StructuresInit)
    pub blower_src_rect: Rect,
    pub blower_src_map: Map,
    pub blower_mask_map: Map,
    pub flame: [Rect; NUM_CANDLE_FLAMES as usize],
    pub tiki_flame: [Rect; NUM_TIKI_FLAMES as usize],
    pub coals: [Rect; NUM_BBQ_COALS as usize],
    pub furniture_src_rect: Rect,
    pub furniture_src_map: Map,
    pub furniture_mask_map: Map,
    pub table_src: Rect,
    pub shelf_src: Rect,
    pub hinge_src: Rect,
    pub handle_src: Rect,
    pub knob_src: Rect,
    pub left_foot_src: Rect,
    pub right_foot_src: Rect,
    pub deck_src: Rect,
    pub bonus_src_rect: Rect,
    pub bonus_src_map: Map,
    pub bonus_mask_map: Map,
    pub points_src_rect: Rect,
    pub points_src_map: Map,
    pub points_mask_map: Map,
    pub star_src: [Rect; 6],
    pub sparkle_src: [Rect; NUM_SPARKLE_MODES as usize],
    pub digits: [Rect; 11],
    pub pendulum_src: [Rect; 3],
    pub grease_src_rt: [Rect; 4],
    pub grease_src_lf: [Rect; 4],
    pub trans_src_rect: Rect,
    pub trans_src_map: Map,
    pub trans_mask_map: Map,
    pub switch_src_rect: Rect,
    pub switch_src_map: Map,
    pub light_switch_src: [Rect; 2],
    pub machine_switch_src: [Rect; 2],
    pub thermostat_src: [Rect; 2],
    pub power_src: [Rect; 2],
    pub knife_switch_src: [Rect; 2],
    pub light_src_rect: Rect,
    pub light_src_map: Map,
    pub light_mask_map: Map,
    pub flourescent_src1: Rect,
    pub flourescent_src2: Rect,
    pub track_light_src: [Rect; NUM_TRACK_LIGHTS as usize],
    pub appliance_src_rect: Rect,
    pub toast_src_rect: Rect,
    pub shred_src_rect: Rect,
    pub appliance_src_map: Map,
    pub toast_src_map: Map,
    pub shred_src_map: Map,
    pub appliance_mask_map: Map,
    pub toast_mask_map: Map,
    pub shred_mask_map: Map,
    pub plus_screen1: Rect,
    pub plus_screen2: Rect,
    pub tv_screen1: Rect,
    pub tv_screen2: Rect,
    pub coffee_light1: Rect,
    pub coffee_light2: Rect,
    pub vcr_time1: Rect,
    pub vcr_time2: Rect,
    pub stereo_light1: Rect,
    pub stereo_light2: Rect,
    pub micro_on: Rect,
    pub micro_off: Rect,
    pub outlet_src: [Rect; NUM_OUTLET_PICTS as usize],
    pub balloon_src_rect: Rect,
    pub copter_src_rect: Rect,
    pub dart_src_rect: Rect,
    pub ball_src_rect: Rect,
    pub drip_src_rect: Rect,
    pub enemy_src_rect: Rect,
    pub fish_src_rect: Rect,
    pub balloon_src_map: Map,
    pub copter_src_map: Map,
    pub dart_src_map: Map,
    pub ball_src_map: Map,
    pub drip_src_map: Map,
    pub enemy_src_map: Map,
    pub fish_src_map: Map,
    pub balloon_mask_map: Map,
    pub copter_mask_map: Map,
    pub dart_mask_map: Map,
    pub ball_mask_map: Map,
    pub drip_mask_map: Map,
    pub enemy_mask_map: Map,
    pub fish_mask_map: Map,
    pub balloon_src: [Rect; NUM_BALLOON_FRAMES as usize],
    pub copter_src: [Rect; NUM_COPTER_FRAMES as usize],
    pub dart_src: [Rect; NUM_DART_FRAMES as usize],
    pub ball_src: [Rect; NUM_BALL_FRAMES as usize],
    pub drip_src: [Rect; NUM_DRIP_FRAMES as usize],
    pub fish_src: [Rect; NUM_FISH_FRAMES as usize],
    pub clutter_src_map: Map,
    pub clutter_mask_map: Map,
    pub clutter_src_rect: Rect,
    pub flower_src: [Rect; NUM_FLOWERS as usize],
    /// `srcRects`: each object kind's size, kNumSrcRects long.
    pub src_rects: Vec<Rect>,
    pub temp_manholes: [Rect; 8],
    pub saved_maps: Vec<SavedType>,
    /// `masterObjects`, kMaxMasterObjects long.
    pub master_objects: Vec<ObjData>,
    /// `hotSpots`, kMaxHotSpots long.
    pub hot_spots: Vec<HotObject>,
    pub n_local_obj: i32,
    pub n_hot_spots: i32,
    pub num_master_objects: i32,
    pub num_local_master_objects: i32,
    pub num_temp_manholes: i32,
    pub tv_with_movie_number: i32,
    pub new_state: bool,
    pub tv_in_room: bool,

    // --- Render.c
    pub work2_main_rects: Vec<Rect>,
    pub back2_work_rects: Vec<Rect>,
    pub shield_rect: Rect,
    pub mirror_rgn: Option<Region>,
    pub shield_pt: Point,
    pub next_frame: i32,
    pub num_work2_main: i32,
    pub num_back2_work: i32,
    pub has_mirror: bool,

    // --- Transit.c
    pub linked_to_what: i32,
    pub taking_the_stairs: bool,
    pub first_player: bool,

    // --- Triggers.c
    pub triggers: [Trig; 16],

    // --- Banner.c
    pub num_stars_remaining: i32,
    pub banner_star_count_on: bool,

    // --- Marquee.c
    pub the_marquee: Marquee,

    // --- Scoreboard.c
    pub board_src_rect: Rect,
    pub badge_src_rect: Rect,
    pub board_dest_rect: Rect,
    pub board_src_map: Map,
    pub board_t_src_map: Map,
    pub board_g_src_map: Map,
    pub board_p_src_map: Map,
    pub badge_src_map: Map,
    pub board_t_src_rect: Rect,
    pub board_t_dest_rect: Rect,
    pub board_g_src_rect: Rect,
    pub board_g_dest_rect: Rect,
    pub board_p_src_rect: Rect,
    pub board_p_dest_rect: Rect,
    pub board_pq_dest_rect: Rect,
    pub board_gq_dest_rect: Rect,
    pub badges_blank_rects: [Rect; 4],
    pub badges_badges_rects: [Rect; 4],
    pub badges_dest_rects: [Rect; 4],
    pub displayed_score: i32,
    pub was_scoreboard_mode: i32,
    pub do_roll_score: bool,
    /// The scoreboard's text (drawn with DrawString in the C), for the
    /// terminal: the room title line, the glider count, the score.
    pub board_title: String,
    pub board_gliders: String,
    pub board_score: String,

    // --- RubberBands.c
    pub bands: Vec<Band>,
    pub bands_src_rect: Rect,
    pub band_rects: [Rect; 3],
    pub bands_src_map: Map,
    pub bands_mask_map: Map,
    pub num_bands: i32,
    pub band_hit_last: i32,

    // --- Grease.c
    pub grease: Vec<Grease>,
    pub num_grease: i32,

    // --- GameOver.c
    pub pages: [Page; 8],
    pub page_src_rect: Rect,
    pub page_src: [Rect; 14],
    pub letters_src: [Rect; 8],
    pub angel_src_rect: Rect,
    pub room_rgn: Option<Region>,
    pub page_src_map: Map,
    pub game_over_src_map: Map,
    pub angel_src_map: Map,
    pub page_mask_map: Map,
    pub angel_mask_map: Map,
    pub count_down: i32,
    pub stop_pages: i32,
    pub pages_stuck: i32,
    pub game_over: bool,

    // --- HighScores.c
    pub high_banner: String,
    pub high_name: String,
    pub last_high_score: i32,
    pub key_stroke: bool,
    /// `(*thisHouse)->highScores`.
    pub high_scores: Scores,
    /// The name (and banner) the terminal got from the player for a high
    /// score; GetHighScoreName/Banner take them (they replace the dialogs).
    pub pending_name: Option<String>,
    pub pending_banner: Option<String>,
    /// SavedGames.c's `resumedSavedGame` (saved games are cut: false).
    pub resumed_saved_game: bool,

    // --- Modes.c
    pub said_follow: i32,

    // --- Music.c
    pub music_sound_id: i32,
    pub music_cursor: i32,
    pub music_score: [i32; 16],
    pub game_score: [i32; 6],
    pub music_mode: i32,
    pub is_music_on: bool,
    pub is_play_music_idle: bool,
    pub is_play_music_game: bool,
    pub failed_music: bool,
    pub dont_load_music: bool,
    /// Music for the audio layer, in order: snd ids to queue on the music
    /// channel (kBaseBufferMusicID + piece), or `music::MUSIC_STOP` to
    /// flush and quiet it. When the queue's last piece ends the audio layer
    /// asks `music_piece_finished` for the next.
    pub music_queue: Vec<i32>,

    // --- Sound.c
    pub priority0: i32,
    pub priority1: i32,
    pub priority2: i32,
    pub sound_playing0: i32,
    pub sound_playing1: i32,
    pub sound_playing2: i32,
    pub is_sound_on: bool,
    pub failed_sound: bool,
    pub dont_load_sounds: bool,
    /// `theSoundData[kMaxSounds - 1]`: the house's trigger sound (snd id),
    /// loaded by LoadTriggerSound.
    pub trigger_sound_id: Option<i32>,

    // --- Input.c
    pub demo_data: Vec<DemoKey>,
    pub demo_index: i32,
    pub battery_frame: i32,
    pub is_esc_pause_key: bool,
    pub paused: bool,
    pub battery_was_engaged: bool,
    /// What GetKeys would report this frame.
    pub keys: KeyState,

    // --- Main.c / Events.c
    pub quitting: bool,
    pub quicker_transitions: bool,
    pub do_zooms: bool,
    pub increment_mode_time: i32,

    // --- Utilities.c
    pub the_seed: u32,
    /// QuickDraw's `randSeed` (the Toolbox Random's).
    pub rand_seed: u32,

    // --- the terminal
    /// `thisMac.screen`: a 512 x 362 screen, so the work map (screen less
    /// the scoreboard) holds one room under the scoreboard's 20 rows.
    pub screen: Rect,
    /// The main window's picture (what CopyRectsQD copies the work map
    /// to); the terminal shows it.
    pub main_window_map: Map,
    /// TickCount(): 60ths of a second since start, advanced by the loop.
    pub ticks: i32,
    /// Terminal text over the room (`label`).
    pub labels: Vec<Label>,
    /// The terminal, for the C's blocking waits (`delay_ticks`,
    /// `wait_for_input_event`).
    pub screen_io: Option<Box<dyn ScreenIo>>,
    /// Sounds started this frame (`play_prioritized_sound` and friends),
    /// for the audio layer: (sound number, channel 0-2). Sound numbers are
    /// the kXSound indices (snd id `sound::BASE_BUFFER_SOUND_ID` + number;
    /// `TRIGGER_SOUND` is the house's `trigger_sound_id`). Each replaces
    /// what plays on its channel; when one ends the audio layer calls
    /// `sound_finished(channel)`. Sound -1 (`sound::NO_SOUND_PLAYING`)
    /// silences the channel (FlushAnyTriggerPlaying) without that call.
    pub sound_queue: Vec<(i32, i32)>,
}

impl Game {
    /// DelayTicks: show the screen and wait, time going on.
    pub fn delay_ticks(&mut self, n: i32) {
        if let Some(mut io) = self.screen_io.take() {
            io.present(self);
            io.wait(n, self);
            self.screen_io = Some(io);
        }
        self.ticks += n;
    }

    /// WaitForInputEvent(seconds): show the screen and wait for a key or
    /// button, at most `seconds`; true if one came.
    pub fn wait_for_input_event(&mut self, seconds: i32) -> bool {
        let mut hit = false;
        let mut waited = 0;
        if let Some(mut io) = self.screen_io.take() {
            io.present(self);
            while waited < seconds * 60 {
                waited += 2;
                if io.wait(2, self) {
                    hit = true;
                    break;
                }
            }
            self.screen_io = Some(io);
        }
        self.ticks += waited;
        hit
    }

    /// Show the main window now (the C draws to the screen directly in
    /// some animations).
    pub fn present(&mut self) {
        if let Some(mut io) = self.screen_io.take() {
            io.present(self);
            self.screen_io = Some(io);
        }
    }

    /// Show text at room position (x, baseline y), as the C's DrawString.
    pub fn label(&mut self, x: i32, y: i32, text: impl Into<String>, color: crate::mac::Rgb, frames: i32) {
        self.labels.push(Label { x, y, text: text.into(), color, frames });
    }

    pub fn assets(&self) -> std::rc::Rc<Assets> {
        self.assets.clone().expect("assets loaded")
    }

    /// A picture by resource number, the house's own first
    /// (LoadHouseResource / GetPicture).
    pub fn picture(&self, id: i32) -> Option<std::rc::Rc<crate::mac::Image>> {
        self.assets.as_ref()?.pict(Some(&self.house), id as i16)
    }

    /// A map holding a picture (CreateOffScreenGWorld + LoadGraphic), the
    /// size of `r`; white if the picture is missing.
    pub fn load_map(&self, id: i32, r: Rect) -> Map {
        let mut m = Map::new(r);
        if let Some(img) = self.picture(id) {
            m.draw_picture(&img, r);
        } else {
            log::warn!("PICT {id} missing");
        }
        m
    }
}

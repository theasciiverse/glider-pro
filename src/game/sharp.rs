//! The silhouette view's house objects drawn sharp (as `games/glider`'s
//! `draw::sharp_art`): in their own colours, without the black outline
//! around them, through the view's shape matcher instead of Braille, whose
//! brightest dot per point loses their dark detail. Chosen kind by kind
//! from renders (see CLAUDE.md, Views).

use super::Game;
use super::defs::*;
use crate::gfx::Map;
use crate::mac::Rgb;

/// How the silhouette view draws an object kind's art.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Look {
    /// Through the shape matcher, in its own colours.
    Sharp,
    /// In half blocks, each the average of the pixels under it, as the
    /// original view: for big busy pictures, which the matcher turns to
    /// glyph noise and Braille to its brightest colours.
    Picture,
}

/// Kinds drawn sharp, and kinds drawn as pictures. `GLIDER_PRO_SHARP` and
/// `GLIDER_PRO_PICTURE` (all, none, or kind numbers separated by commas)
/// override them, for comparing renders; a kind in both is a picture.
pub const SHARP: &[i32] = &[
    FLOOR_VENT,    // Demo House 0: grilles crisp, were dotted
    CEILING_LIGHT, // Demo House 0: a lit dish, was a pale smudge
    VASE2,         // Demo House 0: blue vase, was softened into the wall
    FLOWER,        // Demo House 0: stem and petals keep their colours
    WALL_WINDOW,   // Demo House 0: frame and panes clean, were dotted
    RED_CLOCK,     // Demo House 0: red case, black face, green digits; Braille showed a green face
    BLUE_CLOCK,    // Art Museum 3: blue rim, white face, hands; Braille a white square
    YELLOW_CLOCK,  // Demo House 38: bells and face read as an alarm clock
    CUCKOO,        // Demo House 5: a solid wooden house with weights, was speckle
    BATTERY,       // Demo House 1: dark top, orange body, as the art
    FOIL,          // Art Museum 19: same colours, cleaner
    STAR,          // Demo House 23: cleaner edges, same shape
    HELIUM,        // Art Museum 33: green bottle with its label
    LIGHT_SWITCH,  // CD Demo House 5: the plate shows its toggle
    MACHINE_SWITCH, // Demo House 2: clean, red lever
    THERMOSTAT,    // Art Museum 6: clean body and red mark (as Glider)
    POWER_SWITCH,  // CD Demo House 20: clean in a dark room (as Glider)
    KNIFE_SWITCH,  // Demo House 4: the box with its blades
    SHREDDER,      // California or Bust! 11: a machine with slot and light; Braille made stripes
    TOASTER,       // CD Demo House 5: clean chrome with its dial
    MAC_PLUS,      // Demo House 2: beige case, screen and slot (as Glider)
    GUITAR,        // California or Bust! 9: clean body and neck (as Glider)
    TV,            // Demo House 4: test bars and a clean cabinet
    COFFEE,        // CD Demo House 3: the machine and its pot
    OUTLET,        // CD Demo House 201: plate and two sockets; Braille a white slab
    VCR,           // California or Bust! 11: clean body, red display light kept
    MICROWAVE,     // CD Demo House 51: dark window as the art; Braille made orange speckle
    LIGHT_BULB,    // CD Demo House 5: bulb and base; Braille a white box
    TABLE_LAMP,    // CD Demo House 181: cleaner shade and base
    HIP_LAMP,      // Grand Prix 28: cleaner pole and shades
    DECO_LAMP,     // Demo House 3: the black lamp; Braille nearly lost it
    FLOURESCENT,   // Demo House 1: the tube; Braille lost it
    TRACK_LIGHT,   // Art Museum 17: cleaner rail and heads
    CEILING_VENT,  // CD Demo House 181: grille edges, as the floor vent
    FLOOR_BLOWER,  // CD Demo House 173: cleaner
    CEILING_BLOWER, // CD Demo House 173: cleaner
    SEWER_GRATE,   // Demo House 20: the bars show
    LEFT_FAN,      // CD Demo House 49: the black fan whole
    RIGHT_FAN,     // CD Demo House 5: the black fan stand, as the art
    GRECO_VENT,    // Demo House 15: the grille; Braille nearly lost it
    SEWER_BLOWER,  // CD Demo House 50: cleaner
    TAPER,         // Demo House 5: clean holders (flames are inked anyway)
    CANDLE,        // Demo House 2: white candle, orange base
    STUBBY,        // Art Museum 32: clean red and white candles
    TIKI,          // CD Demo House 76: clean pole and bowl
    BBQ,           // CD Demo House 76: the grill and its coals
    FILING_CABINET, // CD Demo House 120: drawers and handles
    WASTE_BASKET,  // CD Demo House 3: cleaner
    MILK_CRATE,    // Demo House 1: cleaner
    STOOL,         // Demo House 1: red-topped stools; Braille softened them into the wall picture
    TRUNK,         // Demo House 38: the chest and its clasp
    MANHOLE,       // Demo House 14: the dark curve
    BOOKS,         // Demo House 2: the spines keep their colours (unlike Glider's)
    CINDER_BLOCK,  // California or Bust! 6: cleaner
    FLOWER_BOX,    // CD Demo House 142: cleaner
    COBWEB,        // Art Museum 7: thin lines; Braille white noise
    MOUSEHOLE,     // CD Demo House 175: clean
    BEAR,          // Demo House 4: clean
    VASE1,         // Demo House 2: the white vase, clean
    BULLETIN,      // Demo House 4: crisp notes on cork; as a picture they blur
    FIREPLACE,     // Demo House 5: mantel and firebox; Braille softened it into the wall picture
    FAUCET,        // CD Demo House 179: the taps; Braille white blobs
    RUG,           // Art Museum 33: cleaner
    CHIMES,        // CD Demo House 2: clean tubes and hanger
    GREASE_RT,     // Demo House 1: the jar, its blue lid and label; Braille white dots
    GREASE_LF,     // Art Museum 32: as GREASE_RT
    FLOOR_TRANS,   // Art Museum 0: cleaner
    CEILING_TRANS, // Art Museum 8: cleaner
    DOOR_IN_LF,    // CD Demo House 143: crisp panels and knocker; as a picture it blurs
    DOOR_IN_RT,    // ImagineHouse PRO II 45: as DOOR_IN_LF
    DOOR_EX_RT,    // CD Demo House 142: clean frame
    DOOR_EX_LF,    // ImagineHouse PRO II 26: clean frame
    WINDOW_IN_LF,  // CD Demo House 34: clean frame and pane
    WINDOW_IN_RT,  // Demo House 5: clean frame and pane
    WINDOW_EX_RT,  // CD Demo House 33: clean frame
    WINDOW_EX_LF,  // Demo House 7: clean frame
];
// Judged and kept in Braille: PAPER (Art Museum 5: the matcher draws glyph
// noise on the sheet), BANDS (Demo House 1: sharp turns the green box
// black; Braille keeps it, as in Glider), STEREO (California or Bust! 11:
// sharp makes one black block of the speakers; Braille keeps their cones).
pub const PICTURE: &[i32] = &[
    CUSTOM_PICT, // Demo House 0: the quilt as in the original view; the matcher made glyph noise of it, Braille lost its dark diamonds
    OZMA,        // Art Museum 37: the portrait recognisable; the matcher made lines of the face
    CALENDAR,    // Demo House 2: keeps the month grid; sharp drew white bars
    UP_STAIRS,   // Demo House 4: as the original view; sharp drew a column of [[[ on the lit edge
    DOWN_STAIRS, // Demo House 5: as the original view, clean rail and steps
    MAILBOX_LF,  // CD Demo House 69: name plates read as blurred lettering, not pseudo-letters
    MAILBOX_RT,  // CD Demo House 0: as MAILBOX_LF
];

fn listed(var: &'static str, what: i32, default: &[i32]) -> bool {
    match std::env::var(var).ok().as_deref() {
        Some("all") => true,
        Some(s) => s.split(',').filter_map(|k| k.trim().parse().ok()).any(|k: i32| k == what),
        None => default.contains(&what),
    }
}

pub fn look(what: i32) -> Option<Look> {
    if what == OBJECT_IS_EMPTY {
        None
    } else if listed("GLIDER_PRO_PICTURE", what, PICTURE) {
        Some(Look::Picture)
    } else if listed("GLIDER_PRO_SHARP", what, SHARP) {
        Some(Look::Sharp)
    } else {
        None
    }
}

fn dark(c: Rgb) -> bool {
    c.iter().all(|&v| v < 64)
}

impl Game {
    /// A new room: nothing tagged yet. The tags are the tag map's `fore`:
    /// each covered pixel with the colour the object drew there.
    pub fn begin_sharp_art(&mut self) {
        let mut tag = Map::new(self.back_src_map.bounds);
        tag.enable_fore();
        self.sharp_art = Some(tag);
    }

    /// Before an object of a sharp kind is drawn: the back map as it was
    /// (for taking back the outline), and the map marking what it covers.
    pub fn begin_sharp_object(&mut self) -> Vec<Rgb> {
        self.back_src_map.cover = vec![false; self.back_src_map.px.len()];
        self.back_src_map.px.clone()
    }

    /// After it was drawn: tag what it covered (the flag: a picture). A sharp object's dark outline (covered pixels
    /// next to uncovered ones) is left out, as Glider's `Canvas::art`
    /// leaves it.
    pub fn tag_sharp_art(&mut self, before: &[Rgb], look: Look) {
        let m = &mut self.back_src_map;
        let b = m.bounds;
        let (w, h) = (b.width(), b.height());
        let covered = std::mem::take(&mut m.cover);
        let mut n = 0;
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if !covered[i] {
                    continue;
                }
                let edge = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|&(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    nx < 0 || ny < 0 || nx >= w || ny >= h || !covered[(ny * w + nx) as usize]
                });
                if look == Look::Sharp && edge && dark(m.px[i]) {
                    m.px[i] = before[i];
                    continue;
                }
                n += 1;
                if let Some(tag) = self.sharp_art.as_mut()
                    && !tag.fore.is_empty()
                {
                    tag.fore[i] = Some((m.px[i], look == Look::Picture));
                }
            }
        }
        log::debug!("sharp art: {n} pixels");
    }
}

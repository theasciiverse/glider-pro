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
];
pub const PICTURE: &[i32] = &[
    CUSTOM_PICT, // Demo House 0: the quilt as in the original view; the matcher made glyph noise of it, Braille lost its dark diamonds
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

/// What the back map holds while a sharp object is drawn, so that every
/// pixel it covers shows, even where the room's picture already had the
/// same colour there (Demo House paints some objects into its walls).
const SENTINEL: Rgb = [254, 1, 253];

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

    /// Before an object of a sharp kind is drawn: the back map as it was,
    /// and the map filled with the sentinel.
    pub fn begin_sharp_object(&mut self) -> Vec<Rgb> {
        let before = self.back_src_map.px.clone();
        self.back_src_map.px.fill(SENTINEL);
        before
    }

    /// After it was drawn: tag what it covered (the flag: a picture) and
    /// put the rest back. A sharp object's dark outline (covered pixels
    /// next to uncovered ones) is left out, as Glider's `Canvas::art`
    /// leaves it.
    pub fn tag_sharp_art(&mut self, before: &[Rgb], look: Look) {
        let m = &mut self.back_src_map;
        let b = m.bounds;
        let (w, h) = (b.width(), b.height());
        let covered: Vec<bool> = m.px.iter().map(|&c| c != SENTINEL).collect();
        let mut n = 0;
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if !covered[i] {
                    m.px[i] = before[i];
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

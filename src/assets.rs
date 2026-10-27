//! Glider PRO's own pictures and sounds, from `Glider PRO.r` (DeRez text in
//! Calhoun's repository), decoded once at start-up. A house may carry
//! pictures and sounds of its own that take the place of these
//! (`LoadHouseResource` looks in the house first): see [`Assets::pict`].

use crate::house::House;
use crate::mac::{self, Image, Resources, Sound};
use std::cell::RefCell;
use std::collections::HashMap;

const GLIDER_PRO_R: &[u8] = include_bytes!("../Glider PRO.r");

pub struct Assets {
    pub app: Resources,
    /// Decoded pictures, by (from the house?, id).
    cache: RefCell<HashMap<(bool, i16), Option<std::rc::Rc<Image>>>>,
}

impl Assets {
    pub fn load() -> Assets {
        Assets { app: mac::derez(GLIDER_PRO_R), cache: RefCell::new(HashMap::new()) }
    }

    fn decode(res: &Resources, id: i16) -> Option<std::rc::Rc<Image>> {
        let v = res.get(&(*b"PICT", id))?;
        match mac::pict(v) {
            Ok(i) => Some(std::rc::Rc::new(i)),
            Err(e) => {
                log::warn!("PICT {id}: {e}");
                None
            }
        }
    }

    /// A picture: the house's, if it has one by that number, else Glider
    /// PRO's own.
    pub fn pict(&self, house: Option<&House>, id: i16) -> Option<std::rc::Rc<Image>> {
        if let Some(h) = house
            && h.res.contains_key(&(*b"PICT", id))
        {
            return self.cache.borrow_mut().entry((true, id)).or_insert_with(|| Assets::decode(&h.res, id)).clone();
        }
        self.cache.borrow_mut().entry((false, id)).or_insert_with(|| Assets::decode(&self.app, id)).clone()
    }

    /// Forget the house's pictures (another house is loaded).
    pub fn new_house(&self) {
        self.cache.borrow_mut().retain(|k, _| !k.0);
    }

    /// A sound: the house's own first, as with pictures.
    pub fn sound(&self, house: Option<&House>, id: i16) -> Option<Sound> {
        house.and_then(|h| h.res.get(&(*b"snd ", id))).or_else(|| self.app.get(&(*b"snd ", id))).and_then(|v| mac::snd(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_picture_and_sound_decodes() {
        let a = Assets::load();
        let mut n = 0;
        for ((t, id), v) in &a.app {
            if t == b"PICT" {
                mac::pict(v).unwrap_or_else(|e| panic!("Glider PRO.r PICT {id}: {e}"));
                n += 1;
            }
            if t == b"snd " {
                assert!(mac::snd(v).is_some(), "snd {id}");
            }
        }
        assert_eq!(n, 152);
        for (name, _) in crate::house::BUILT_IN {
            let h = crate::house::load(name).unwrap();
            for ((t, id), v) in &h.res {
                if t == b"PICT" {
                    mac::pict(v).unwrap_or_else(|e| panic!("{name} PICT {id}: {e}"));
                }
            }
        }
    }
}

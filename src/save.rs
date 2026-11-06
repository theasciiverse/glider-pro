//! Files kept between runs, in `$XDG_DATA_HOME/glider-pro`
//! (`%APPDATA%\glider-pro` on Windows, `~/Library/Application
//! Support/glider-pro` on macOS; `GLIDER_PRO_DATA_DIR` overrides):
//!
//! - `scores.txt`, each house's high scores. Glider PRO wrote these into
//!   the house file itself (10 per house: name, score, time, level); here
//!   they sit beside it, one tab-separated line each, so the houses stay
//!   as shipped.
//! - `glider-pro.cfg`, settings as `key value` lines.

use std::path::PathBuf;

/// kMaxScores.
pub const NUM_SCORES: usize = 10;
/// Str15: the names Glider PRO keeps.
pub const NAME_MAX: usize = 15;

pub fn dir() -> Option<PathBuf> {
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(d) = env("GLIDER_PRO_DATA_DIR") {
        return Some(d);
    }
    let base = if cfg!(windows) {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_DATA_HOME").or_else(|| env("HOME").map(|h| h.join(".local/share")))
    };
    base.map(|b| b.join("glider-pro"))
}

fn file(name: &str) -> Option<PathBuf> {
    let d = dir()?;
    std::fs::create_dir_all(&d).ok()?;
    Some(d.join(name))
}

/// One house's high score table, as Glider PRO keeps it (`scoresType`):
/// a banner from the best player, then up to ten (name, score, time,
/// rooms visited).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub banner: String,
    pub rows: Vec<(String, i32, u32, i32)>,
}

fn table_file(house: &str) -> Option<PathBuf> {
    let d = dir()?.join("scores");
    std::fs::create_dir_all(&d).ok()?;
    let safe: String = house.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
    Some(d.join(format!("{safe}.txt")))
}

/// `scores/<house>.txt`: the banner on the first line, then one
/// tab-separated row per score.
pub fn read_table(house: &str) -> Option<Table> {
    let text = std::fs::read_to_string(table_file(house)?).ok()?;
    let mut lines = text.lines();
    let banner = lines.next()?.to_string();
    let rows = lines
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Some((f.first()?.to_string(), f.get(1)?.parse().ok()?, f.get(2)?.parse().ok()?, f.get(3)?.parse().ok()?))
        })
        .take(NUM_SCORES)
        .collect();
    Some(Table { banner, rows })
}

pub fn write_table(house: &str, t: &Table) {
    let Some(p) = table_file(house) else { return };
    let mut text = format!("{}\n", clean_long(&t.banner, 31));
    for (n, s, time, lv) in &t.rows {
        text += &format!("{}\t{s}\t{time}\t{lv}\n", clean(n));
    }
    if let Err(e) = std::fs::write(&p, text) {
        log::warn!("cannot save scores to {}: {e}", p.display());
    }
}

fn clean(s: &str) -> String {
    clean_long(s, NAME_MAX)
}

fn clean_long(s: &str, n: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(n).collect()
}

/// Settings as `key value` lines.
#[derive(Clone, Debug)]
pub struct Settings {
    pub sound: bool,
    pub music: bool,
    pub name: String,
    pub house: String,
    /// The message shown with a top score (Glider PRO's high banner).
    pub banner: String,
    /// The silhouette view rather than Calhoun's art.
    pub silhouette: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { sound: true, music: true, name: String::new(), house: "Slumberland".into(), banner: String::new(), silhouette: true }
    }
}

impl Settings {
    pub fn load() -> Settings {
        let mut s = Settings::default();
        let Some(text) = file("glider-pro.cfg").and_then(|p| std::fs::read_to_string(p).ok()) else { return s };
        for l in text.lines() {
            let (k, v) = l.split_once(' ').unwrap_or((l, ""));
            match k {
                "sound" => s.sound = v != "off",
                "music" => s.music = v != "off",
                "name" => s.name = clean(v),
                "house" => s.house = v.to_string(),
                "banner" => s.banner = v.chars().filter(|c| !c.is_control()).take(31).collect(),
                "view" => s.silhouette = v != "original",
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let Some(p) = file("glider-pro.cfg") else { return };
        let on = |b: bool| if b { "on" } else { "off" };
        let view = if self.silhouette { "silhouette" } else { "original" };
        let text = format!("sound {}\nmusic {}\nname {}\nhouse {}\nbanner {}\nview {view}\n", on(self.sound), on(self.music), clean(&self.name), self.house, self.banner);
        let _ = std::fs::write(p, text);
    }
}

//! Decodes Calhoun's files at build time, so the game carries them as
//! deflated bytes rather than as text: the 21 houses' BinHex (data and
//! resource forks) and `Glider PRO.r` (DeRez text). 50 MB of text become a
//! good deal less, and nothing is decoded but the house being played.
//!
//! Writes to OUT_DIR: `glider_pro_r.bin` (the resources, see
//! `assets::unpack`), `house<N>.bin` (u32 data fork length, data fork,
//! resource fork), and `houses.rs`, the list of built-in houses.

#[path = "src/mac.rs"]
#[allow(dead_code)]
mod mac;

use std::io::Write;
use std::path::PathBuf;

/// The houses in Calhoun's repository, as BinHex. Sampler is left out: its
/// resource fork is damaged (as Aerofoil leaves it out).
const HOUSES: &[&str] = &[
    "Demo House",
    "Art Museum",
    "CD Demo House",
    "California or Bust!",
    "Castle o' the Air",
    "Davis Station",
    "Empty House",
    "Fun House",
    "Grand Prix",
    "ImagineHouse PRO II",
    "In The Mirror",
    "Land of Illusion",
    "Leviathan",
    "Metropolis",
    "Nemo's Market",
    "Rainbow's End",
    "Slumberland",
    "SpacePods",
    "Teddy World",
    "The Asylum Pro",
    "Titanic",
];

fn deflate(b: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec(b, 9)
}

fn main() {
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=Glider PRO.r");
    println!("cargo:rerun-if-changed=src/mac.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // Glider PRO.r: count, then (type, id, length, bytes) each, by type and id
    let res = mac::derez(&std::fs::read("Glider PRO.r").expect("Glider PRO.r"));
    let mut keys: Vec<_> = res.keys().copied().collect();
    keys.sort();
    let mut raw = Vec::new();
    raw.extend_from_slice(&(keys.len() as u32).to_be_bytes());
    for k in keys {
        let v = &res[&k];
        raw.extend_from_slice(&k.0);
        raw.extend_from_slice(&k.1.to_be_bytes());
        raw.extend_from_slice(&(v.len() as u32).to_be_bytes());
        raw.extend_from_slice(v);
    }
    std::fs::write(out.join("glider_pro_r.bin"), deflate(&raw)).unwrap();

    let mut list = String::from("/// The built-in houses (from build.rs): name and deflated forks.\npub const BUILT_IN: &[(&str, &[u8])] = &[\n");
    for (i, name) in HOUSES.iter().enumerate() {
        let path = format!("Houses/{name}.binhex");
        println!("cargo:rerun-if-changed={path}");
        let text = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let (data, rsrc) = mac::binhex(&text).unwrap_or_else(|| panic!("{path}: not BinHex"));
        let mut raw = Vec::with_capacity(4 + data.len() + rsrc.len());
        raw.extend_from_slice(&(data.len() as u32).to_be_bytes());
        raw.extend_from_slice(&data);
        raw.extend_from_slice(&rsrc);
        std::fs::write(out.join(format!("house{i}.bin")), deflate(&raw)).unwrap();
        list += &format!("    ({name:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/house{i}.bin\"))),\n");
    }
    list += "];\n";
    std::fs::File::create(out.join("houses.rs")).unwrap().write_all(list.as_bytes()).unwrap();
}

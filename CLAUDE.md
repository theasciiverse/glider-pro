# Glider PRO

Asciiverse port of **Glider PRO** (John Calhoun, Casady & Greene, 1994), the sequel
to Glider 4 (`games/glider`): a house of hundreds of rooms on floors and suites,
moving enemies, links between rooms. Rust + Ratatui. Started 2026-10-27.

Name: working name "Glider PRO", the original's own (as `games/glider` is "Glider").
David suggested Aerofoil; recommended against (2026-10-27) because Aerofoil is Eric
Lasota's name for his own maintained port. Not decided. Who holds the "Glider PRO"
trademark was not found.

A terminal game **based on** Glider PRO, not a replica (David, 2026-10-27): translate
faithfully first, then change whatever stands in the way of playing well here, and
record it.

## Sources (checked 2026-10-27)

- `upstream`: github.com/softdorothy/GliderPRO, Calhoun's own repo. C (CodeWarrior),
  46k lines, CR line endings. README: "released under the GNU General Public License
  2"; the licence file is the GPLv2 text. History: AGPL, then "GPL (I think)", then
  "GPL 2 or later", then GPLv2 text (commits 3f3801f, 2fa0db4, 8b2a88d, b4aaee9).
  Holds `Glider PRO.r` (DeRez: 152 PICT, 70 `snd `, dialogs) and 22 houses as BinHex
  (several by other authors, credited in the README).
- `aerofoil`: github.com/elasota/Aerofoil, Eric Lasota's port (C++), GPLv2 licence
  file, readme "version 2 ... or (at your option) any later version". Game code in
  `GpApp/` (53.5k lines, about 25k of it gameplay; the rest editor, dialogs,
  platform). The port follows Aerofoil's code (64-bit, endian-clean, bug fixes),
  checked against Calhoun's where they differ.
- Licence field: GPL-2.0-only until it is settled whether Calhoun's "GPL 2" allows
  later versions.
- No terminal port of Glider PRO existed (searched 2026-10-27).

## Data (all read directly, nothing extracted)

- `Glider PRO.r` (15 MB DeRez text): every PICT decodes (opcode 0x99 needed), every
  `snd ` is plain 8-bit at 22254 Hz (no MACE).
- Houses: BinHex 4.0 (`mac::binhex`), data fork = `houseType` (866 bytes + 348 a room,
  big-endian; `house.rs`), resource fork = the house's own PICTs and sounds, which
  take the place of Glider PRO's (`LoadHouseResource`, `Assets::pict`). All 21 built-in
  houses' pictures decode; they need opcodes 0x1F (colour) and 0x9A (direct 16/32-bit
  pixels, Teddy World and The Asylum Pro). Sampler is left out: its resource fork is
  damaged (as Aerofoil leaves it out). The `.mov` files (QuickTime, house intros) are
  not used.
- Size: houses 34 MB BinHex, Glider PRO.r 15 MB, built into the binary for now. A
  build step converting them to raw forks would about halve that (to do).

## Choices for the terminal

- Only the current room is drawn (512x322), as large as the terminal allows. Glider
  PRO drew the 3x3 rooms around it on a big screen; at terminal sizes the room itself
  needs every cell.

## Modules

`mac.rs`, `gfx.rs`, `view.rs` are copied from `games/glider` (not a shared crate yet,
per "validate before generalizing") with Glider PRO's additions (BinHex, opcodes 0x1F
and 0x9A, 322-pixel rooms, masks as separate pictures). `house.rs`, `assets.rs`,
`main.rs` (for now a house explorer: arrows move between rooms, H next house, V view).

## Plan: translating Aerofoil's GpApp

Done: data layer, room backgrounds (DrawRoomBackground). Next, in order:

1. StructuresInit/2 (sprite sheets and source rects), ObjectDraw, ObjectDraw2,
   ObjectDrawAll, ObjectRects: the rooms with their objects.
2. Room, Objects, House (room setup, the master object list, links), Render.
3. Player, Interactions, Play, Modes, Input: the glider flying.
4. Dynamics, Dynamics2, Dynamics3, DynamicMaps (enemies), Triggers, Transit, Trip,
   Grease, RubberBands, Scoreboard, Marquee, Banner.
5. Sound, Music, GameOver, HighScores, SavedGames, SelectHouse, the title screen.
6. The silhouette view and terminal text, as in `games/glider`.

Not planned: the house editor (ObjectEdit, ObjectInfo, ObjectAdd, RoomInfo,
HouseInfo, HouseLegal, Link, Map, Tools, Scrap, SourceExport).

## Commands

```bash
cargo run --release                         # the house explorer
cargo run --release -- --house "Teddy World" --room 3
cargo run --release -- --shot --house Slumberland --room 5 --size 128x44
cargo test --release
```

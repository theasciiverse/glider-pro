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
- Licence: GPL-2.0-or-later. Calhoun's own commit 8b2a88d (2016-01-31) is titled and
  reads "GPL 2 or later"; the same day b4aaee9 ("License has full-text.") swapped in the
  full GPLv2 text and the README now says "GPL 2", read as shorthand, not a retraction.
  Aerofoil reads it the same way. So the game can be used under GPLv3 too, which
  matters for Apache-2.0 libraries such as notcurses (Apache's own page: the FSF "has
  never considered the Apache License to be compatible with GPL version 2"; it is with
  GPLv3). A confirming note to Calhoun would be a courtesy, not a blocker (David's
  call, 2026-10-27: no hair-splitting).
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

## Plan (decided 2026-11-06, replaces the Aerofoil plan of 2026-10-27)

David, 2026-11-06: the Aerofoil build (branch `asciiverse`, Aerofoil's whole C++ with a
notcurses driver) was "a bit too complete for an Asciiverse game": a graphical emulator
of the game in the terminal. Wanted instead: **a good follow-up to Glider**, built the
way `games/glider` is, and trimming parts of the game is fine.

So: **Rust on Glider's engine**, Calhoun's C translated (not Aerofoil's), reading his
files directly. Branch `main`; `asciiverse` stays on GitHub, unused.

- Kept: the whole play of the game (Play, Player, Interactions, Dynamics, Objects,
  Room, Render, Transit, Triggers, Grease, RubberBands, Banner, Scoreboard, GameOver,
  HighScores, Sound, Music), every built-in house, houses' own art and sounds, any house
  file from disk.
- Cut (David, 2026-11-06): the house editor (ObjectEdit, ObjectInfo, RoomInfo, Tools,
  Link, Coordinates, Scrap, ObjectAdd), the map window, the house checks (HouseLegal),
  house intro movies, saved games, the Mac settings dialogs (settings become a terminal
  screen as in Glider).
- Like Glider: terminal cells only (no pixel graphics), the original art in half blocks
  and a silhouette view (V switches), text shown during play as terminal text, keys
  (hold or tap), mouse, gamepad, sound.
- One room on screen: `numNeighbors` = 1, a setting Glider PRO itself offers
  (Settings: 1, 3 or 9 rooms), so play is as on a Mac set to show one room.

## Modules

- `mac.rs` (BinHex, resource forks, DeRez, PICT, snd), `house.rs`, `assets.rs`: data.
- `gfx.rs`: offscreen maps (GWorlds) of any size and the QuickDraw subset Glider PRO
  draws with (pen size, pattern and mode, regions, CopyBits, CopyMask, the 8-bit
  palette). No text: text is terminal text.
- `game/`: the translation. One `Game` struct holds Glider PRO's globals (C names in
  snake_case); each C file becomes a module of `impl Game` with the C function names
  in snake_case.
- `view.rs`: maps to terminal cells.

## Commands

```bash
# the Rust house explorer, until step 2 replaces it
cargo run --release
cargo run --release -- --house "Teddy World" --room 3
cargo run --release -- --shot --house Slumberland --room 5 --size 128x44
cargo test --release
```

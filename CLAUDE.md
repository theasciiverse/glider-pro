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
- Size: houses 34 MB BinHex, Glider PRO.r 15 MB of text. `build.rs` decodes them at
  build time (with `src/mac.rs`) and deflates each house and the resources
  (miniz_oxide); a house is inflated when it is loaded. Stripped binary 21.5 MB (was
  89 MB with the text built in), 2026-11-07. The house list is in build.rs.

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

- `mac.rs` (BinHex, resource forks, DeRez, PICT, snd), `house.rs` (house files; the
  object union as typed views, `obj.a().state` / `obj.set_a(..)`), `assets.rs`: data.
- `gfx.rs`: offscreen maps (GWorlds) of any size and the QuickDraw subset Glider PRO
  draws with (pen size, pattern and mode, regions, CopyBits, CopyMask, the 8-bit
  palette `clut8`), plus the silhouette view's ink and `fore` layer. No text.
- `game/`: the translation, 16k lines. One `Game` (mod.rs) holds Glider PRO's globals
  (C names in snake_case); each C file is a module of `impl Game` with the C function
  names in snake_case (`defs.rs`: GliderDefines.h, `kFloorVent` = `FLOOR_VENT`;
  `rect_utils.rs`: RectUtils.c and the Toolbox's rect calls). Conventions at the top of
  mod.rs. Translated 2026-11-06 by five parallel agents (groups: player/interactions,
  dynamics/render, objects/rooms, object drawing, game flow), then integrated.
- `ScreenIo` (mod.rs): what the C's blocking waits need from the terminal (DelayTicks,
  WaitForInputEvent, the high score name). `main.rs` lends the terminal to the game
  for each call that may wait (`lend`) and takes it back.
- `PlayGame`'s loop is `play_game_frame` (one frame), called every 2 ticks by
  `main.rs`; `new_game` sets up, `play_game_finish` is what follows the loop.
- `view.rs`: `games/glider`'s view, unchanged but for a local `Canvas` (one room's
  pixels and silhouettes) and `soften_backdrop`.
- `input.rs`, `audio.rs` (three effect channels and music, as Glider PRO's Sound
  Manager channels; reports channels that ended so `sound_finished` /
  `music_piece_finished` run as the C callbacks), `save.rs`.

## Screen and coordinates

`thisMac.screen` is 512 x 362, so houseRect (the work and back maps) is 512 x 342
and the one room sits at (0, 20)-(512, 342) (`localRoomsDest[kCentralRoom]`), under
the scoreboard's 20 rows; the terminal shows that rectangle of the main window. The
scoreboard's figures (room, gliders, score) are shown as the status line
(`board_title`, `board_gliders`, `board_score`), other text as `Game::label`s.

## Views (V switches, saved)

- **Original**: Calhoun's art in half blocks.
- **Silhouette** (default, as Glider): sprites drawn into the work map go through
  `copy_mask_ink` with an ink from `Game::sil` (by source map: glider white, foil, toast,
  balloons, darts, ...), recorded in the maps' `fore` layer, which copies along work ->
  main; the view draws those cells with Glider's shape matcher. The room's background
  is softened (Glider's blur and dim) wherever it still shows: `draw_locale` keeps the
  bare background (`room_backdrop`) before the objects are drawn, and the view swaps
  in its softened copy pixel by pixel. Objects keep their art; flames are inked too.
  Furniture as in Glider (David, 2026-11-07, "do the same as for what we did with
  glider"): what Calhoun drew with QuickDraw (table, deck table, shelf, cabinet,
  counter, dresser, mirror) is flat in wood and glass tones (`game/flat.rs`), and its
  edges (`room_edges`, room coordinates) are box-drawn over the picture by
  `view::draw_edges` (Glider's, skipping cells with a silhouette in them). V during
  play redraws the room for the other view (`redraw_for_view`, RedrawRoomLighting's
  way, so flames and enemies go on).

## Controls

Glider PRO's own defaults: left and right arrows, Down = battery (or helium), Up =
rubber band; also A, D, S, W and Space; gamepad as in Glider. Kitty keyboard protocol
for exact releases, else key repeats. Tab pauses (`do_pause` / `end_pause`), Q ends
the game, V view. Title: Enter, L houses, T scores (the C's DoHighScores), D demo
(Demo House from the 'demo' resource; also after two idle minutes, kIdleSplashTicks),
H help, M sound, B music.

## Saves

`$XDG_DATA_HOME/glider-pro` (`GLIDER_PRO_DATA_DIR` overrides): `glider-pro.cfg`, and
`scores/<house>.txt`, each house's table (the C kept it in the house file; houses here
stay as shipped, as the C's locked houses, whose scores went to a file).
`read_house` reads it over the house's own table; `test_high_score` writes it.

## Testing without a person

- `--frames N [HOUSE]` plays N frames headless. Env: `GLIDER_PRO_HOLD=right,batt`
  (keys held), `GLIDER_PRO_DEMO=1` (the recorded demo), `GLIDER_PRO_TRACE=k`
  (`_FROM=f`): the glider every k frames; `GLIDER_PRO_EVENTS=1`: room changes and lost
  gliders; `GLIDER_PRO_SHOT=file.ppm`: the main window at the end;
  `GLIDER_PRO_CELLS=file` (+ `GLIDER_PRO_VIEW=original`): the terminal's cells
  (120 x 39) as ANSI text; `GLIDER_PRO_MONKEY=seed`: random keys, game after game;
  `GLIDER_PRO_TWO=1`: a two-player game (`HOLD` takes `left2`, `right2`).
  Monkey run of 2026-11-07: 21 houses x 4 seeds x 30000 frames, no panic.
  `RUST_LOG=glider_pro=debug` logs every hot spot hit.
- `cargo test --release`: data decodes, every built-in house plays 1500 frames four
  ways and through a game's end (`every_house_plays`).
- tmux on a private socket (`tmux -L gptest`), `send-keys`, `capture-pane -e -p -N`
  (tmux carries colours over from the row before: a parser must too).

## Content check (2026-11-07)

`glider-pro --texts` prints every built-in house's banner, trailer and room names; all
were read. Nothing like the games skipped before; the one thing for David to decide:
The Asylum Pro (by Steve Sullivan, credited in the README) is a
mental-asylum joke throughout ("Lunatic Lounge", "Automatic For Our Patients", "You
must be crazy!"), 1990s-mild. Kept: David, 2026-11-07, "then keep it". Elsewhere: "Take me drunk, I'm
home", "Pipe Bomb" (room names). Text inside the houses' pictures was not read.

## Catalog preview (2026-11-07)

`asciiverse/glider-pro.cast`, 100x30, about 55 s, recorded by
`asciiverse/record-preview.sh`: `glider-pro --demo` (Glider PRO's own demo, Demo House
from Calhoun's recorded keys), silhouette view, the original view from about 22 s to
38 s. Not in the hub catalog: no repo release, no store page yet (David's call).

## Known gaps (2026-11-07)

- The demo replays as Calhoun recorded it, deaths included (audit of 2026-11-07: the
  glider touches the TV in room 4 at frame 1766 in the recording itself; held keys
  end exactly on the frames rooms change, before and after; the third glider is lost
  just after the last press, frame 3414). 1117 presses (kDemoLength is bytes). A
  good regression check: first glider lost at 1766 in room 4, game over at 3417 in
  room 15 (Water).
- Two-player games: 2 on the title; the second player has A, D, S, W (Glider PRO gave it the modifier keys, which a terminal cannot report alone). Without the kitty protocol only one key at a time counts as held, so two players need it.
- Not seen yet in play: every object kind's drawing and behaviour in every house.

## Commands

```bash
cargo run --release                         # title screen; house from settings
cargo run --release -- "Teddy World"
cargo run --release -- --frames 600 Slumberland
cargo run --release -- --demo                 # the demo straight away
asciiverse/record-preview.sh                  # the catalog preview cast
cargo test --release
```

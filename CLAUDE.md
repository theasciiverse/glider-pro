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

## Modules

`mac.rs`, `gfx.rs`, `view.rs` are copied from `games/glider` (not a shared crate yet,
per "validate before generalizing") with Glider PRO's additions (BinHex, opcodes 0x1F
and 0x9A, 322-pixel rooms, masks as separate pictures). `house.rs`, `assets.rs`,
`main.rs` (for now a house explorer: arrows move between rooms, H next house, V view).

## Plan (decided 2026-10-27, paused until David comes back to it)

**Build on Aerofoil's C++ and give it a terminal, instead of translating to Rust.**
Why, from the measurements of 2026-10-27:

- Aerofoil's gameplay is Calhoun's code line for line: Player 10 of 1,412 lines
  changed, Interactions 12 of 1,611, Dynamics 22 of 693, RubberBands 2 of 273 (whitespace
  normalised). The bigger diffs (ObjectDraw 363 of 1,186, Play, Render) are where
  Aerofoil swapped the Mac's drawing calls for its own.
- Calhoun's C on its own needs the classic Mac Toolbox (QuickDraw, Resource, Sound,
  Memory and File Managers, dialogs, QuickTime): that is Aerofoil's 31k-line
  PortabilityLayer. Calhoun's C + notcurses would mean rewriting it. Translating to
  Rust would be about 25k lines plus debugging every difference.
- Aerofoil keeps the platform behind small driver interfaces (`GpCommon/`):
  `IGpDisplayDriver` (about 20 methods: surfaces the game draws into, DrawSurface,
  palette, cursor), `IGpInputDriver` (ProcessInput posting `GpVOSEvent`s),
  `IGpAudioDriver` (the SDL2 one works without a window), fonts, file system.
  Linux builds through CMake (`AerofoilX` target, SDL2). SDL2 2.32, CMake, g++ and
  (from David, 2026-10-27) `libnotcurses-dev` are installed here.

Steps:

1. Build Aerofoil as it is (`cmake`, `make AerofoilX`; building external code needs
   David's approval in auto mode). Work out the data conversion on Linux: Aerofoil
   plays converted data (`.gpa` etc.) made by `ConvertResources.bat` (Windows) from
   CMake tools (MiniRez, hqx2gp, gpr2gpa, HouseTool, MergeGPF); MacPorts packages
   Aerofoil, so a non-Windows route exists. Not yet checked.
2. Rebase `games/glider-pro` onto Aerofoil's history (`aerofoil` remote), keeping
   Calhoun's as `calhoun` for reference. The Rust start (`src/`, data layer and
   explorer) is dropped; its PICT and BinHex readers live on in `games/glider`.
3. A notcurses display driver: compose the game's surfaces into a frame, show it with
   a pixel blitter (kitty graphics, Sixel) where the terminal has one, else sextants,
   octants, Braille or half blocks. Only the central room needs every cell (Glider PRO
   draws the 3x3 rooms around it on a big screen; for the terminal, show the room).
4. A notcurses input driver: keys with releases (kitty protocol), mouse; and the
   key-repeat fallback from `games/glider`.
5. Menus, dialogs and the high-score entry as terminal text (Aerofoil draws them as
   640x480 pixels, unreadable at terminal size).
6. Later: the silhouette view (needs small hooks in GpApp to know what is where),
   terminal text for in-game numbers, as in `games/glider`.

Licence for this: Aerofoil GPL-2.0-or-later, notcurses Apache-2.0 (fine with GPLv3,
see Sources).

## Commands

```bash
# the Rust house explorer, until step 2 replaces it
cargo run --release
cargo run --release -- --house "Teddy World" --room 3
cargo run --release -- --shot --house Slumberland --room 5 --size 128x44
cargo test --release
```

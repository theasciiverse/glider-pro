# Glider PRO

Asciiverse port of **Glider PRO** (John Calhoun, Casady & Greene, 1994), the sequel
to Glider 4 (`games/glider`): a house of hundreds of rooms on floors and suites,
moving enemies, links between rooms. Started 2026-10-27 in Rust; since 2026-11-06 it is
Aerofoil's C++ (Eric Lasota's port of Calhoun's code) with our notcurses terminal drivers,
on branch `asciiverse` (`main` keeps Calhoun's history and the Rust start).

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

## Layout (branch `asciiverse`)

| What | Where |
|---|---|
| Notcurses display driver: surfaces composited in software, 640x480 frame scaled to the terminal, keyboard and mouse | `AerofoilTerm/GpDisplayDriver_Notcurses.cpp` |
| Terminal `main`: SDL for sound and gamepads only, no window | `AerofoilTerm/GpMain_Term.cpp` |
| The `glider-pro` target, static notcurses via `AV_NOTCURSES_PREFIX` | `CMakeLists.txt` (after the AerofoilX block) |
| Static build: libunistring and patched notcurses 3.0.17 into `av-build/`, as Twistty | `asciiverse/build.sh`, `asciiverse/patches/` |
| Big-button main menu outside touchscreens too | `SetUseMainMenuUI` in `GpApp/MainMenuUI.cpp` |

- Data: Aerofoil's CMake converts Calhoun's `Glider PRO.r` and the BinHex houses
  (`GliderProData/`) into `Packaged/` (8.7 MB) with its own tools; no Windows step.
- Blitter: pixels where the terminal has them, else sextants, quadrants, half blocks.
  Never pixels under tmux or screen (they claim sixel whatever the terminal can do).
- Keys: exact presses and releases with the kitty protocol; elsewhere a key counts as
  held while its repeats arrive (550 ms before the first repeat, 100 ms between), as
  `games/glider`. Ctrl-C quits.
- stderr goes to /dev/null when it is the terminal (the game's own prints would land on
  the screen); `2>file` keeps it. `-diagnostics` writes a log under
  `~/.local/share/aerofoil/aerofoil/Logs`.
- Saves, prefs and high scores still go to Aerofoil's folder (`aerofoil/aerofoil`); to
  change to our own.

## Plan (decided 2026-10-27; steps 1 to 4 done 2026-11-06)

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

Done 2026-11-06: Aerofoil builds and runs here as it is (its zlib submodule must be
fetched), the `asciiverse` branch sits on Aerofoil's master, and the terminal build plays:
title, main menu by mouse, a game begun in Slumberland, the glider steered by keys into
the next room, all inside tmux in half blocks.

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

## Open: pixels or terminal cells (2026-11-06)

In a terminal with pixel graphics (David's, kitty graphics) notcurses shows the frame
as real pixels: the 1994 Mac game exactly, in a window. David: "the graphics are too
good". Recommended, not yet decided:

1. Never pixels by default: start in a cell view (sextants where the terminal has
   them, else quadrants or half blocks), V cycles views, as `games/glider`. Pixels
   either dropped or behind a flag (recommended: dropped).
2. Later (plan step 6): the silhouette view as in `games/glider` (Braille, box-drawn
   furniture), which needs small hooks in GpApp for where things are.

## Commands

A fresh Linux machine needs cmake (3.26 or later), g++, git, curl, pkg-config and the
dev packages for SDL2, FreeType, libdeflate and ncurses (Debian/Ubuntu: `libsdl2-dev
libfreetype-dev libdeflate-dev libncurses-dev`; the script installs them itself when run
as root on Ubuntu). It fetches the zlib submodule and builds libunistring and notcurses
into `av-build/` on first run.

```bash
git switch asciiverse
sh asciiverse/build.sh                        # static notcurses, data, then the game
av-build/install/bin/glider-pro               # data in av-build/install/lib/aerofoil/Packaged
av-build/install/bin/glider-pro -diagnostics  # with the log
```

Testing without a person: run it in tmux on a private socket (`tmux -L gptest`),
`send-keys` for keys and SGR mouse reports (`send-keys -l $'\e[<0;41;56M'`), and
`capture-pane -e` for the screen.

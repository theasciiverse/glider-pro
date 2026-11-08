mod assets;
#[allow(dead_code)]
mod audio;
mod game;
#[allow(dead_code)]
mod gfx;
#[allow(dead_code)]
mod house;
#[allow(dead_code)]
mod input;
#[allow(dead_code)]
mod mac;
#[allow(dead_code)]
mod save;
#[allow(dead_code)]
mod view;

use assets::Assets;
use audio::Audio;
use game::{Game, ScreenIo};
use gfx::{Map, Rect};
use input::{Cmd, Input};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event, KeyboardEnhancementFlags, MouseButton,
    MouseEventKind, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::supports_keyboard_enhancement;
use ratatui::layout::Rect as Area;
use ratatui::style::{Color, Modifier, Style};
use save::Settings;
use std::collections::{HashMap, VecDeque};
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The Mac's tick.
const TICK: Duration = Duration::from_micros(16_667);

const HELP: &str = "glider-pro: Glider PRO (John Calhoun, 1994) in the terminal

Fly a paper glider through a house of many rooms. Floor vents lift you,
nearly everything else is in the way.

usage: glider-pro [options] [HOUSE]

  HOUSE           a built-in house by name (\"Slumberland\") or a Glider PRO
                  house file: BinHex, MacBinary, or the data fork with its
                  resource fork beside it (NAME.rsrc, ._NAME)
  --houses        list the built-in houses
  --mute          no sound
  --demo          start with the demo (Demo House playing itself)
  --silhouette    the silhouette view (moving things as solid shapes)
  --original      Calhoun's art in half blocks
  --frames N      run N frames of a game with no one at the keys, no
                  terminal (a smoke test), and print where it ended
  --version       print the version

Keys: left and right arrows (or A, D) fly; Down (or S) burns battery or
helium; Up, W or Space fires a rubber band. Tab pauses, Q ends the game.

Logging: RUST_LOG=debug glider-pro 2>glider-pro.log";

/// The screen the game believes in: one room (512 x 322) under the
/// scoreboard's 20 rows, 20 more for the menu bar strip.
const SCREEN: Rect = Rect::new(0, 0, 512, 362);

fn seed_now() -> u32 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as u32 ^ d.subsec_nanos()).unwrap_or(1)
}

/// A game set up as Glider PRO's main() does before its event loop, with
/// `house` read.
fn new_game_state(a: Rc<Assets>, house: &str) -> Option<Game> {
    let mut g = Game { assets: Some(a), screen: SCREEN, num_neighbors: 1, ..Default::default() };
    g.main_window_map = Map::new(SCREEN);
    g.main_window_map.fill_rect(SCREEN, gfx::BLACK);
    g.the_seed = seed_now();
    g.init_random_long_qus();
    g.is_sound_on = true;
    g.is_play_music_game = true;
    g.is_play_music_idle = false;
    g.quicker_transitions = false;
    g.init_screen_rects();
    g.create_pointers();
    g.init_src_rects();
    g.create_offscreens();
    g.init_sound();
    g.init_music();
    if !load_house(&mut g, house) {
        return None;
    }
    Some(g)
}

fn load_house(g: &mut Game, name: &str) -> bool {
    let Some(h) = house::load(name) else { return false };
    if let Some(a) = &g.assets {
        a.new_house();
    }
    g.house = h;
    // a file from disk is named as the Mac named it, without its suffix
    let p = std::path::Path::new(name);
    g.this_house_name = if p.exists() {
        let stem = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
        stem.trim_end_matches(".binhex").trim_end_matches(".hqx").trim_end_matches(".bin").to_string()
    } else {
        name.to_string()
    };
    g.read_house()
}

// ---------------------------------------------------------------- terminal

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Screen {
    Title,
    Playing,
}

struct Term {
    terminal: ratatui::DefaultTerminal,
    input: Input,
    audio: Audio,
    a: Rc<Assets>,
    /// Sounds by snd id, house-own first; cleared with a new house.
    sounds: HashMap<i32, Option<Arc<mac::Sound>>>,
    music: VecDeque<i32>,
    music_playing: bool,
    aspect: f32,
    placed: view::Placed,
    screen: Screen,
    any: bool,
    /// Lines of terminal text shown in a box over the picture (title,
    /// help, the house list, a prompt).
    overlay: Vec<String>,
    /// The sound and music switches (Glider PRO's Sound settings).
    sound_on: bool,
    music_on: bool,
}

impl Term {
    fn sound(&mut self, g: &Game, id: i32) -> Option<Arc<mac::Sound>> {
        let a = self.a.clone();
        self.sounds.entry(id).or_insert_with(|| a.sound(Some(&g.house), id as i16).map(Arc::new)).clone()
    }

    /// Start what the game queued and react to channels that ended.
    fn service_audio(&mut self, g: &mut Game) {
        for (n, ch) in std::mem::take(&mut g.sound_queue) {
            let s = if n < 0 {
                None
            } else {
                let id = if n == game::TRIGGER_SOUND { g.trigger_sound_id.unwrap_or(-1) } else { game::sound::BASE_BUFFER_SOUND_ID + n };
                self.sound(g, id)
            };
            log::debug!("sound {n} on channel {ch}: {}", if s.is_some() { "playing" } else { "none" });
            self.audio.play(ch as usize, s);
        }
        for id in std::mem::take(&mut g.music_queue) {
            if id == game::music::MUSIC_STOP {
                self.music.clear();
                self.music_playing = false;
                self.audio.play(audio::MUSIC, None);
            } else {
                self.music.push_back(id);
            }
        }
        for ch in self.audio.finished() {
            if ch == audio::MUSIC {
                self.music_playing = false;
                if self.music.is_empty()
                    && let Some(next) = g.music_piece_finished()
                {
                    self.music.push_back(next);
                }
            } else {
                g.sound_finished(ch as i32);
            }
        }
        if !self.music_playing
            && let Some(id) = self.music.pop_front()
        {
            let s = self.sound(g, id);
            log::debug!("music piece {id}: {}", if s.is_some() { "playing" } else { "missing" });
            self.music_playing = s.is_some();
            self.audio.play(audio::MUSIC, s);
        }
    }

    fn read_events(&mut self, until: Instant) {
        loop {
            let now = Instant::now();
            let left = until.saturating_duration_since(now);
            if !event::poll(left).unwrap_or(false) {
                break;
            }
            match event::read() {
                Ok(Event::Key(k)) => self.input.on_key(k, Instant::now()),
                Ok(Event::Mouse(m)) => match m.kind {
                    MouseEventKind::Down(MouseButton::Left) => self.input.mouse_down(),
                    MouseEventKind::Up(MouseButton::Left) => self.input.mouse_up(),
                    _ => {}
                },
                Ok(Event::FocusLost) => self.input.release_all(),
                Ok(Event::Resize(..)) => {
                    let _ = self.terminal.autoresize();
                }
                _ => {}
            }
        }
        self.input.poll_pad();
    }

    /// Commands since last asked; sets `any` if a key or button came.
    fn take_cmds(&mut self, g: &mut Game) -> Vec<Cmd> {
        let cmds: Vec<Cmd> = self.input.cmds.drain(..).collect();
        for c in &cmds {
            match c {
                Cmd::Any => self.any = true,
                Cmd::Exit => {
                    g.quitting = true;
                    g.playing = false;
                }
                _ => {}
            }
        }
        cmds
    }

    fn draw(&mut self, g: &Game) {
        if let Ok(path) = std::env::var("GLIDER_PRO_DRAW_SHOT") {
            let m = &g.main_window_map;
            let mut out = format!("P6 {} {} 255\n", m.width(), m.height()).into_bytes();
            for p in &m.px {
                out.extend_from_slice(p);
            }
            let _ = std::fs::write(&path, out);
            let w = &g.work_src_map;
            let mut out = format!("P6 {} {} 255\n", w.width(), w.height()).into_bytes();
            for p in &w.px {
                out.extend_from_slice(p);
            }
            let _ = std::fs::write(format!("{path}.work"), out);
        }
        let area = self.terminal.size().map(|s| Area::new(0, 0, s.width, s.height)).unwrap_or_default();
        let pic_area = Area::new(area.x, area.y, area.width, area.height.saturating_sub(1));
        self.placed = view::place(pic_area, self.aspect);
        // the central room, out of the main window
        let room_dest = g.local_rooms_dest[0];
        let room = view::Canvas::from_map(&g.main_window_map, room_dest);
        let silhouette = g.silhouette && self.screen == Screen::Playing;
        let placed = self.placed;
        let screen = self.screen;
        let status = status_line(g, screen);
        let labels: Vec<game::Label> = g.labels.clone();
        let overlay = self.overlay.clone();
        let _ = self.terminal.draw(|f| {
            let buf: &mut Buffer = f.buffer_mut();
            if silhouette {
                view::draw_silhouette(buf, &room, &placed);
            } else {
                view::draw(buf, &room, &placed);
            }
            for l in &labels {
                let x = l.x - room_dest.left;
                let y = l.y - room_dest.top;
                if let Some((cx, cy)) = cell_at(&placed, x, y - 6) {
                    let st = Style::default().fg(Color::Rgb(l.color[0], l.color[1], l.color[2])).bg(Color::Black);
                    buf.set_stringn(cx, cy, &l.text, (area.width.saturating_sub(cx)) as usize, st);
                }
            }
            if !overlay.is_empty() {
                let w = overlay.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16 + 4;
                let h = overlay.len() as u16 + 2;
                let x0 = area.width.saturating_sub(w) / 2;
                let y0 = area.height.saturating_sub(h + 1) / 2;
                let bx = Style::default().fg(Color::White).bg(Color::Rgb(16, 16, 40));
                for yy in 0..h {
                    for xx in 0..w {
                        if x0 + xx < area.width && y0 + yy < area.height {
                            buf[(x0 + xx, y0 + yy)].set_char(' ').set_style(bx);
                        }
                    }
                }
                for (i, l) in overlay.iter().enumerate() {
                    let st = if i == 0 { bx.add_modifier(Modifier::BOLD).fg(Color::Yellow) } else { bx };
                    buf.set_stringn(x0 + 2, y0 + 1 + i as u16, l, (w - 3) as usize, st);
                }
            }
            let y = area.height.saturating_sub(1);
            let st = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
            buf.set_stringn(0, y, &status, area.width as usize, st);
        });
    }
}

/// The silhouette view on or off (V).
fn set_silhouette(g: &mut Game, on: bool) {
    g.silhouette = on;
    if on {
        g.work_src_map.enable_fore();
        g.main_window_map.enable_fore();
    } else {
        g.work_src_map.disable_fore();
        g.main_window_map.disable_fore();
    }
}

/// The terminal cell over room pixel (x, y).
fn cell_at(p: &view::Placed, x: i32, y: i32) -> Option<(u16, u16)> {
    if p.tw == 0 || p.th == 0 {
        return None;
    }
    let cx = x.clamp(0, 511) * p.tw as i32 / 512;
    let cy = y.clamp(0, 321) * (p.th as i32 / 2) / 322;
    Some((p.cells.x + cx as u16, p.cells.y + cy as u16))
}

fn status_line(g: &Game, screen: Screen) -> String {
    match screen {
        Screen::Title => format!(" {}   Enter: play   Q: quit", g.this_house_name),
        Screen::Playing => {
            let paused = if g.paused { "   PAUSED (Tab)" } else { "" };
            format!(" {}   gliders {}   score {}{paused}", g.board_title, g.board_gliders, g.board_score)
        }
    }
}

impl ScreenIo for Term {
    fn present(&mut self, g: &mut Game) {
        self.service_audio(g);
        self.draw(g);
    }

    fn wait(&mut self, ticks: i32, g: &mut Game) -> bool {
        let until = Instant::now() + TICK * ticks.max(0) as u32;
        self.any = false;
        loop {
            self.read_events(until.min(Instant::now() + TICK));
            let _ = self.take_cmds(g);
            self.input.keys(Instant::now(), &mut g.keys);
            self.service_audio(g);
            if Instant::now() >= until {
                break;
            }
        }
        self.any
    }
    fn ask(&mut self, g: &mut Game, prompt: &str, default: &str, max: usize) -> Option<String> {
        let mut text = default.to_string();
        let was = std::mem::take(&mut self.overlay);
        self.input.clear();
        let mut first = true;
        let answer = loop {
            self.overlay = vec![prompt.to_string(), String::new(), format!("> {text}_"), String::new(), "Enter: done   Esc: keep as is".into()];
            self.service_audio(g);
            self.draw(g);
            self.read_events(Instant::now() + TICK * 2);
            let mut done = None;
            for c in self.input.cmds.drain(..) {
                match c {
                    Cmd::Confirm => done = Some(Some(text.clone())),
                    Cmd::Back => done = Some(Some(default.to_string())),
                    Cmd::Exit => {
                        g.quitting = true;
                        done = Some(Some(default.to_string()));
                    }
                    Cmd::Backspace => {
                        if first {
                            text.clear();
                        }
                        text.pop();
                    }
                    Cmd::Char(ch) if !ch.is_control() => {
                        if first {
                            text.clear();
                        }
                        if text.chars().count() < max {
                            text.push(ch);
                        }
                    }
                    _ => continue,
                }
                first = false;
            }
            if let Some(d) = done {
                break d;
            }
        };
        self.overlay = was;
        self.input.clear();
        answer.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).or(Some(default.to_string()))
    }
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

/// The title: Glider PRO's splash, scaled to the room.
fn draw_title(g: &mut Game) {
    let dest = g.local_rooms_dest[0];
    g.main_window_map.fill_rect(SCREEN, gfx::BLACK);
    if let Some(img) = g.picture(game::SPLASH8_BIT_PICT) {
        g.main_window_map.draw_picture(&img, dest);
    }
    g.labels.clear();
}

const TITLE_KEYS: [&str; 2] = ["Enter: play   2: two players   D: demo   L: houses   T: scores", "H: help   M: sound   B: music   V: view   Q: quit"];

fn title_lines(g: &Game, t: &Term) -> Vec<String> {
    let on = |b: bool| if b { "on" } else { "off" };
    vec![
        "GLIDER PRO".into(),
        format!("John Calhoun, 1994   house: {}", g.this_house_name),
        String::new(),
        "Arrows fly: Down burns battery, Up fires a rubber band.".into(),
        format!("Sound {}, music {}, {} view (V).", on(t.sound_on), on(t.music_on), if g.silhouette { "silhouette" } else { "original" }),
        String::new(),
        TITLE_KEYS[0].into(),
        TITLE_KEYS[1].into(),
    ]
}

const HELP_LINES: &[&str] = &[
    "HOW TO PLAY",
    "",
    "You are a paper glider loose in a house. You only ever fall: the air",
    "from floor vents and blowers is what keeps you up. Fly from room to",
    "room, out by the doors, up and down the stairs and through the ducts.",
    "",
    "Left, Right   steer (or A, D; a gamepad's d-pad or stick)",
    "Down          burn battery for a burst of speed, or helium to rise",
    "Up, Space     fire a rubber band (if you have any)",
    "Tab           pause          Q   end the game      V   switch view",
    "",
    "Two players (2 on the title): the second flies with A and D, S for",
    "battery, W for a band. Delete gives up a glider stuck somewhere.",
    "",
    "Clocks and stars are points. Folded paper is an extra glider, foil",
    "a shield. Candles, toasters, outlets, darts and the rest are not",
    "your friends. Find all the stars in a house to finish it.",
    "",
    "Esc or Enter: back",
];

/// The house list: pick with Up/Down and Enter.
fn choose_house(term: &mut Term, g: &mut Game) -> Option<String> {
    let names: Vec<&str> = house::BUILT_IN.iter().map(|(n, _)| *n).collect();
    let mut sel = names.iter().position(|n| *n == g.this_house_name).unwrap_or(0);
    term.input.clear();
    loop {
        let top = sel.saturating_sub(6).min(names.len().saturating_sub(13));
        let mut lines = vec!["CHOOSE A HOUSE".to_string(), String::new()];
        for (i, n) in names.iter().enumerate().skip(top).take(13) {
            lines.push(format!("{} {n}", if i == sel { ">" } else { " " }));
        }
        lines.push(String::new());
        lines.push("Up, Down: choose   Enter: take it   Esc: back".into());
        term.overlay = lines;
        term.draw(g);
        term.read_events(Instant::now() + TICK * 4);
        for c in term.take_cmds(g) {
            match c {
                Cmd::Up => sel = (sel + names.len() - 1) % names.len(),
                Cmd::Down => sel = (sel + 1) % names.len(),
                Cmd::Confirm => return Some(names[sel].to_string()),
                Cmd::Back | Cmd::Quit => return None,
                _ => {}
            }
        }
        if g.quitting {
            return None;
        }
    }
}

fn run(slot: &mut Option<Box<Term>>, g: &mut Game, settings: &mut Settings, demo_first: bool) {
    if demo_first {
        if let Some(t) = slot.as_mut() {
            t.screen = Screen::Playing;
        }
        demo_game(slot, g, settings);
        if g.quitting {
            return;
        }
    }
    loop {
        let mut demo = false;
        // ---- the title
        {
            {
                let term = slot.as_mut().expect("terminal");
                term.screen = Screen::Title;
                term.input.clear();
            }
            draw_title(g);
            let mut help = false;
            let mut idle_since = Instant::now();
            loop {
                let term = slot.as_mut().expect("terminal");
                term.overlay = if help { HELP_LINES.iter().map(|l| l.to_string()).collect() } else { title_lines(g, term) };
                term.draw(g);
                term.read_events(Instant::now() + TICK * 4);
                let cmds = term.take_cmds(g);
                term.service_audio(g);
                if g.quitting {
                    return;
                }
                if help {
                    if cmds.iter().any(|c| matches!(c, Cmd::Back | Cmd::Confirm | Cmd::Help | Cmd::Quit)) {
                        help = false;
                    }
                    continue;
                }
                let mut play = false;
                let mut scores = false;
                if !cmds.is_empty() {
                    idle_since = Instant::now();
                }
                // kIdleSplashTicks: after two minutes on the title, the demo
                if idle_since.elapsed() >= TICK * game::IDLE_SPLASH_TICKS as u32 {
                    demo = true;
                    play = true;
                }
                for c in cmds {
                    match c {
                        Cmd::Quit | Cmd::Back => return,
                        Cmd::Confirm => play = true,
                        Cmd::Char('d' | 'D') => {
                            demo = true;
                            play = true;
                        }
                        Cmd::Char('2') => {
                            g.two_player_game = true;
                            play = true;
                        }
                        Cmd::Help => help = true,
                        Cmd::Sound => {
                            term.sound_on = !term.sound_on;
                            g.is_sound_on = term.sound_on;
                            settings.sound = term.sound_on;
                            settings.save();
                        }
                        Cmd::Music => {
                            term.music_on = !term.music_on;
                            g.is_play_music_game = term.music_on;
                            settings.music = term.music_on;
                            settings.save();
                        }
                        Cmd::Houses => {
                            if let Some(name) = choose_house(term, g) {
                                if load_house(g, &name) {
                                    term.sounds.clear();
                                    settings.house = name;
                                    settings.save();
                                    draw_title(g);
                                } else {
                                    log::warn!("cannot read house {name:?}");
                                }
                            }
                        }
                        Cmd::Scores => scores = true,
                        Cmd::View => {
                            settings.silhouette = !settings.silhouette;
                            set_silhouette(g, settings.silhouette);
                            settings.save();
                        }
                        _ => {}
                    }
                }
                if scores {
                    term.overlay.clear();
                    lend(slot, g, |g| g.do_high_scores());
                    draw_title(g);
                    continue;
                }
                if play {
                    term.overlay.clear();
                    term.screen = Screen::Playing;
                    term.input.clear();
                    break;
                }
                draw_title(g);
            }
        }

        // ---- a game
        if demo {
            demo_game(slot, g, settings);
        } else {
            play(slot, g, settings);
        }
        if g.quitting {
            return;
        }
    }
}

/// A game, from NewGame to its end (PlayGame and what follows it).
fn play(slot: &mut Option<Box<Term>>, g: &mut Game, settings: &mut Settings) {
    if let Some(t) = slot.as_mut() {
        t.input.two = g.two_player_game;
    }
    lend(slot, g, |g| g.new_game(game::NEW_GAME_MODE));
    let mut next = Instant::now();
    while g.playing && !g.quitting {
        {
            let term = slot.as_mut().expect("terminal");
            term.read_events(next);
            let cmds = term.take_cmds(g);
            // the demo: any key ends it (as the arcade build's), but V
            // switches the view
            let viewing = cmds.contains(&Cmd::View);
            for c in &cmds {
                match c {
                    Cmd::Any if g.demo_going && !viewing => g.playing = false,
                    Cmd::Pause => {
                        if g.paused {
                            g.end_pause();
                        } else {
                            g.do_pause();
                        }
                    }
                    Cmd::Quit => {
                        if g.paused {
                            g.end_pause();
                        }
                        g.keys.quit = true;
                    }
                    Cmd::View => {
                        settings.silhouette = !settings.silhouette;
                        set_silhouette(g, settings.silhouette);
                        settings.save();
                        // the furniture is drawn differently in each view
                        g.redraw_for_view();
                    }
                    _ => {}
                }
            }
            term.input.keys(Instant::now(), &mut g.keys);
        }
        if !g.paused && g.playing {
            g.ticks += game::TICKS_PER_FRAME;
            lend(slot, g, |g| g.play_game_frame());
            g.keys.quit = false;
        }
        let term = slot.as_mut().expect("terminal");
        term.service_audio(g);
        term.draw(g);
        next += TICK * game::TICKS_PER_FRAME as u32;
        let now = Instant::now();
        if next < now {
            next = now;
        }
    }
    if !g.quitting {
        lend(slot, g, |g| g.play_game_finish());
    }
    g.two_player_game = false;
    if let Some(t) = slot.as_mut() {
        t.input.two = false;
    }
}

/// DoDemoGame: Demo House played from the recorded keys, then back to the
/// house that was open.
fn demo_game(slot: &mut Option<Box<Term>>, g: &mut Game, settings: &mut Settings) {
    let was = g.this_house_name.clone();
    if !load_house(g, "Demo House") {
        return;
    }
    if let Some(t) = slot.as_mut() {
        t.sounds.clear();
    }
    g.demo_going = true;
    play(slot, g, settings);
    g.demo_going = false;
    load_house(g, &was);
    if let Some(t) = slot.as_mut() {
        t.sounds.clear();
    }
}

/// Run `f` on the game with the terminal lent to it, for the C code that
/// waits (banners, transitions, game over): the game shows and reads
/// through it, then hands it back.
fn lend<R>(term: &mut Option<Box<Term>>, g: &mut Game, f: impl FnOnce(&mut Game) -> R) -> R {
    let t = term.take().expect("terminal");
    g.screen_io = Some(t);
    let r = f(g);
    let io = g.screen_io.take().expect("terminal lent");
    *term = Some(io.into_any().downcast::<Term>().expect("the terminal"));
    r
}

/// No terminal: waits just pass the time.
struct NoIo;

impl ScreenIo for NoIo {
    fn present(&mut self, g: &mut Game) {
        g.sound_queue.clear();
        g.music_queue.clear();
    }
    fn wait(&mut self, _ticks: i32, g: &mut Game) -> bool {
        g.sound_queue.clear();
        g.music_queue.clear();
        false
    }
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

/// The central room as the view gets it, out of the main window.
fn room_canvas(g: &Game) -> view::Canvas {
    view::Canvas::from_map(&g.main_window_map, g.local_rooms_dest[0])
}

/// `--frames N`: a game with nobody at the keys, without a terminal.
fn smoke(a: Rc<Assets>, house: &str, frames: u32) {
    let Some(mut g) = new_game_state(a, house) else {
        eprintln!("cannot read house {house:?}");
        std::process::exit(1);
    };
    g.screen_io = Some(Box::new(NoIo));
    g.demo_going = std::env::var_os("GLIDER_PRO_DEMO").is_some();
    if std::env::var_os("GLIDER_PRO_CELLS").is_some() {
        set_silhouette(&mut g, std::env::var("GLIDER_PRO_VIEW").map_or(true, |v| v != "original"));
    }
    g.two_player_game = std::env::var_os("GLIDER_PRO_TWO").is_some();
    if let Ok(h2) = std::env::var("GLIDER_PRO_SWITCH_TO") {
        load_house(&mut g, &h2);
    }
    if std::env::var_os("GLIDER_PRO_SCORES_FIRST").is_some() {
        draw_title(&mut g);
        g.do_high_scores();
    }
    g.new_game(game::NEW_GAME_MODE);
    let mut n = 0;
    // GLIDER_PRO_HOLD: keys held throughout, e.g. "right,band"
    let hold_s = std::env::var("GLIDER_PRO_HOLD").unwrap_or_default();
    let hold: Vec<&str> = hold_s.split(',').collect();
    let (mut last_mortals, mut last_room) = (g.mortals, g.this_room_number);
    // GLIDER_PRO_MONKEY=seed: random keys, changed every 20 frames, new
    // games as they end
    let mut monkey: Option<u32> = std::env::var("GLIDER_PRO_MONKEY").ok().and_then(|v| v.parse().ok());
    let mut games = 1;
    while n < frames && (g.playing || monkey.is_some()) {
        if let Some(m) = monkey.as_mut() {
            if !g.playing {
                g.play_game_finish();
                g.new_game(game::NEW_GAME_MODE);
                games += 1;
            }
            if n % 20 == 0 {
                *m ^= *m << 13;
                *m ^= *m >> 17;
                *m ^= *m << 5;
                let r = *m;
                g.keys.left = r & 3 == 1;
                g.keys.right = r & 3 == 2;
                g.keys.batt = r & 0x30 == 0x30;
                g.keys.band = r & 0xC0 == 0xC0;
            }
            g.ticks += game::TICKS_PER_FRAME;
            g.play_game_frame();
            g.sound_queue.clear();
            g.music_queue.clear();
            n += 1;
            continue;
        }
        g.keys.left = hold.contains(&"left");
        g.keys.right = hold.contains(&"right");
        g.keys.batt = hold.contains(&"batt");
        g.keys.band = hold.contains(&"band");
        g.keys.left2 = hold.contains(&"left2");
        g.keys.right2 = hold.contains(&"right2");
        g.ticks += game::TICKS_PER_FRAME;
        g.play_game_frame();
        g.sound_queue.clear();
        g.music_queue.clear();
        n += 1;
        if std::env::var_os("GLIDER_PRO_EVENTS").is_some() && (g.mortals != last_mortals || g.this_room_number != last_room) {
            let gl = &g.gliders[0];
            eprintln!("frame {n}: room {} ({}) mortals {} mode {} dest {:?} score {}", g.this_room_number, mac::mac_roman(&g.this_room.name), g.mortals, gl.mode, gl.dest, g.the_score);
            last_mortals = g.mortals;
            last_room = g.this_room_number;
        }
        let every: u32 = std::env::var("GLIDER_PRO_TRACE").ok().and_then(|v| v.parse().ok()).unwrap_or(10);
        let from: u32 = std::env::var("GLIDER_PRO_TRACE_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if std::env::var_os("GLIDER_PRO_TRACE").is_some() && n >= from && n % every.max(1) == 0 {
            let gl = &g.gliders[0];
            eprintln!("frame {n}: room {} mode {} dest {:?} vel ({}, {})", g.this_room_number, gl.mode, gl.dest, gl.h_vel, gl.v_vel);
            if g.two_player_game {
                let g2 = &g.gliders[1];
                eprintln!("          glider 2 mode {} dest {:?} vel ({}, {})", g2.mode, g2.dest, g2.h_vel, g2.v_vel);
            }
        }
    }
    if monkey.is_some() {
        println!("{games} games");
    }
    let gl = &g.gliders[0];
    println!(
        "{n} frames: playing={} room={} ({}) glider mode={} at {:?} score={} mortals={}",
        g.playing,
        g.this_room_number,
        mac::mac_roman(&g.this_room.name),
        gl.mode,
        (gl.dest.left, gl.dest.top),
        g.the_score,
        g.mortals
    );
    // GLIDER_PRO_CELLS=file: the terminal's cells (120 x 39, as tmux's
    // capture-pane -e would give them), in the view from GLIDER_PRO_VIEW
    // (silhouette or original)
    if let Ok(path) = std::env::var("GLIDER_PRO_CELLS") {
        let sil = std::env::var("GLIDER_PRO_VIEW").map_or(true, |v| v != "original");
        let room = room_canvas(&g);
        let area = Area::new(0, 0, 120, 39);
        let placed = view::place(area, 2.0);
        let mut buf = Buffer::empty(area);
        if sil {
            view::draw_silhouette(&mut buf, &room, &placed);
        } else {
            view::draw(&mut buf, &room, &placed);
        }
        let mut out = String::new();
        for y in 0..area.height {
            for x in 0..area.width {
                let c = &buf[(x, y)];
                let rgb = |c: Color| if let Color::Rgb(r, g, b) = c { (r, g, b) } else { (0, 0, 0) };
                let (f, b) = (rgb(c.fg), rgb(c.bg));
                out += &format!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}", f.0, f.1, f.2, b.0, b.1, b.2, c.symbol());
            }
            out += "\x1b[0m\n";
        }
        let _ = std::fs::write(path, out);
    }
    if let Ok(path) = std::env::var("GLIDER_PRO_SHOT") {
        let m = &g.main_window_map;
        let mut out = format!("P6 {} {} 255\n", m.width(), m.height()).into_bytes();
        for p in &m.px {
            out.extend_from_slice(p);
        }
        let _ = std::fs::write(path, out);
    }
    if !g.playing {
        g.play_game_finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Calhoun's recorded demo plays out as recorded: the first glider is
    /// lost in room 4 (the TV, frame 1766, its fade done by 1781), the game
    /// ends in room 15 just after the last press (frame 3414).
    #[test]
    fn the_demo_replays_as_recorded() {
        let a = Rc::new(Assets::load());
        let mut g = new_game_state(a, "Demo House").unwrap();
        g.screen_io = Some(Box::new(NoIo));
        g.demo_going = true;
        g.new_game(game::NEW_GAME_MODE);
        let (mut n, mut first_loss) = (0, None);
        let start = g.mortals;
        while g.playing && n < 5000 {
            g.ticks += game::TICKS_PER_FRAME;
            g.play_game_frame();
            n += 1;
            if first_loss.is_none() && g.mortals < start {
                first_loss = Some((n, g.this_room_number));
            }
        }
        assert_eq!(first_loss, Some((1781, 4)));
        assert!(!g.playing);
        assert_eq!((n, g.this_room_number), (3432, 15));
    }

    /// The silhouette view keeps Calhoun's art (David, 2026-11-08: half
    /// blocks, no blur, no flat furniture): in Demo House's first room every
    /// pixel is as in the original view but for the silhouettes, which are
    /// all in `fore` (the glider among them).
    #[test]
    fn the_silhouette_view_keeps_the_art() {
        let a = Rc::new(Assets::load());
        let room = |sil: bool| {
            let mut g = new_game_state(a.clone(), "Demo House").unwrap();
            g.screen_io = Some(Box::new(NoIo));
            set_silhouette(&mut g, sil);
            g.new_game(game::NEW_GAME_MODE);
            for _ in 0..60 {
                g.ticks += game::TICKS_PER_FRAME;
                g.play_game_frame();
            }
            room_canvas(&g)
        };
        let (sil, orig) = (room(true), room(false));
        let inked = sil.fore.iter().filter(|f| f.is_some()).count();
        assert!(inked > 200, "glider and the rest in fore: {inked}");
        let other = (0..sil.px.len()).filter(|&i| sil.fore[i].is_none() && sil.px[i] != orig.px[i]).count();
        assert_eq!(other, 0, "pixels changed outside the silhouettes");
    }

    /// Every room of every built-in house drawn (every object kind's
    /// drawing and set-up), and a few frames played in it, in both views.
    #[test]
    fn every_room_draws() {
        let a = Rc::new(Assets::load());
        for (name, _) in house::BUILT_IN {
            let mut g = new_game_state(a.clone(), name).unwrap();
            g.screen_io = Some(Box::new(NoIo));
            g.new_game(game::NEW_GAME_MODE);
            let rooms: Vec<i32> = (0..g.house.rooms.len() as i32).filter(|&r| g.house.rooms[r as usize].suite != house::ROOM_IS_EMPTY).collect();
            for (k, r) in rooms.into_iter().enumerate() {
                set_silhouette(&mut g, k % 2 == 1);
                g.force_this_room(r);
                g.determine_room_openings();
                g.draw_locale();
                for _ in 0..8 {
                    if !g.playing {
                        break;
                    }
                    g.ticks += game::TICKS_PER_FRAME;
                    g.play_game_frame();
                }
                if !g.playing {
                    g.new_game(game::NEW_GAME_MODE);
                }
            }
        }
    }

    /// Every built-in house set up, played for a while with the keys held
    /// in a few ways, and through to the end of a game, without a panic.
    #[test]
    fn every_house_plays() {
        // SAFETY: tests here run this one test that touches the environment
        unsafe { std::env::set_var("GLIDER_PRO_DATA_DIR", std::env::temp_dir().join("glider-pro-test")) };
        let a = Rc::new(Assets::load());
        for (name, _) in house::BUILT_IN {
            for hold in [&[][..], &["right"], &["left", "batt"], &["right", "band"]] {
                let mut g = new_game_state(a.clone(), name).unwrap_or_else(|| panic!("{name}"));
                g.screen_io = Some(Box::new(NoIo));
                g.new_game(game::NEW_GAME_MODE);
                for _ in 0..1500 {
                    if !g.playing {
                        break;
                    }
                    g.keys.right = hold.contains(&"right");
                    g.keys.left = hold.contains(&"left");
                    g.keys.batt = hold.contains(&"batt");
                    g.keys.band = hold.contains(&"band");
                    g.ticks += game::TICKS_PER_FRAME;
                    g.play_game_frame();
                }
                g.keys = Default::default();
                g.keys.quit = true;
                g.play_game_frame();
                g.play_game_finish();
            }
        }
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut house = None;
    let mut mute = false;
    let mut demo_first = false;
    let mut view = None;
    let mut frames = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return;
            }
            "--version" => {
                println!("glider-pro {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--texts" => {
                // every built-in house's banner, trailer and room names, for reading
                for (n, _) in house::BUILT_IN {
                    let Some(h) = house::load(n) else { continue };
                    println!("== {n}\n banner: {}\n trailer: {}", mac::mac_roman(&h.banner), mac::mac_roman(&h.trailer));
                    let names: Vec<String> = h.rooms.iter().filter(|r| r.suite != house::ROOM_IS_EMPTY).map(|r| mac::mac_roman(&r.name)).collect();
                    println!(" rooms: {}", names.join(" | "));
                }
                return;
            }
            "--objects" if it.clone().next().is_some_and(|a| a.contains(':')) => {
                // one room's objects: --objects "HOUSE:ROOM"
                let spec = it.next().unwrap();
                let (h, r) = spec.rsplit_once(':').unwrap();
                let (Some(house), Ok(r)) = (house::load(h), r.parse::<usize>()) else { return };
                let Some(room) = house.rooms.get(r) else { return };
                println!("{} ({})", mac::mac_roman(&room.name), spec);
                for (i, o) in room.objects.iter().enumerate().filter(|(_, o)| o.what != game::defs::OBJECT_IS_EMPTY) {
                    println!("{i}\t{}\t{:?}", o.what, o.data);
                }
                return;
            }
            "--objects" => {
                // every object kind in the built-in houses: kind, count,
                // and up to six (house, room) places to see it
                let mut seen: std::collections::BTreeMap<i32, (usize, Vec<String>)> = Default::default();
                for (n, _) in house::BUILT_IN {
                    let Some(h) = house::load(n) else { continue };
                    for (i, r) in h.rooms.iter().enumerate() {
                        if r.suite == house::ROOM_IS_EMPTY {
                            continue;
                        }
                        for o in r.objects.iter().filter(|o| o.what != game::defs::OBJECT_IS_EMPTY) {
                            let e = seen.entry(o.what).or_default();
                            e.0 += 1;
                            let place = format!("{n}:{i}");
                            if e.1.len() < 6 && !e.1.contains(&place) {
                                e.1.push(place);
                            }
                        }
                    }
                }
                for (k, (count, places)) in seen {
                    println!("{k}\t{count}\t{}", places.join(" "));
                }
                return;
            }
            "--houses" => {
                for (n, _) in house::BUILT_IN {
                    println!("{n}");
                }
                return;
            }
            "--mute" => mute = true,
            "--demo" => demo_first = true,
            "--silhouette" => view = Some(true),
            "--original" => view = Some(false),
            "--frames" => frames = it.next().and_then(|n| n.parse().ok()),
            s if !s.starts_with('-') => house = Some(s.to_string()),
            s => {
                eprintln!("unknown option {s}\n\n{HELP}");
                std::process::exit(2);
            }
        }
    }
    let mut settings = Settings::load();
    if let Some(v) = view {
        settings.silhouette = v;
    }
    let house = house.unwrap_or_else(|| settings.house.clone());
    let a = Rc::new(Assets::load());
    if let Some(n) = frames {
        smoke(a, &house, n);
        return;
    }
    let Some(mut g) = new_game_state(a.clone(), &house) else {
        eprintln!("cannot read house {house:?} (see --houses)");
        std::process::exit(1);
    };
    g.is_sound_on = settings.sound;
    g.is_play_music_game = settings.music;
    g.high_name = settings.name.clone();
    g.high_banner = settings.banner.clone();
    set_silhouette(&mut g, settings.silhouette);
    settings.house = house;
    settings.save();

    let exact = supports_keyboard_enhancement().unwrap_or(false);
    let terminal = ratatui::init();
    let mut out = io::stdout();
    if exact {
        let _ = execute!(out, PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES));
    }
    let _ = execute!(out, EnableMouseCapture, EnableFocusChange);
    let mut input = Input::new(exact);
    input.open_pad();
    let term = Term {
        terminal,
        input,
        audio: Audio::new(mute),
        a,
        sounds: HashMap::new(),
        music: VecDeque::new(),
        music_playing: false,
        aspect: view::cell_aspect(),
        placed: view::Placed::default(),
        screen: Screen::Title,
        any: false,
        overlay: Vec::new(),
        sound_on: settings.sound,
        music_on: settings.music,
    };
    let mut slot = Some(Box::new(term));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut slot, &mut g, &mut settings, demo_first)));
    let mut out = io::stdout();
    if exact {
        let _ = execute!(out, PopKeyboardEnhancementFlags);
    }
    let _ = execute!(out, DisableMouseCapture, DisableFocusChange);
    ratatui::restore();
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

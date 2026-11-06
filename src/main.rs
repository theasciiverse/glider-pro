mod assets;
mod audio;
mod game;
mod gfx;
mod house;
mod input;
mod mac;
mod save;
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

  HOUSE           a built-in house by name (\"Slumberland\")
  --houses        list the built-in houses
  --mute          no sound
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
    g.this_house_name = name.to_string();
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
    /// The room's bare background and its softened copy, for the
    /// silhouette view, by `Game::backdrop_serial`.
    backdrop: Option<(u32, view::Canvas, view::Canvas)>,
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
        let mut room = view::Canvas::from_map(&g.main_window_map, room_dest);
        let silhouette = g.silhouette && self.screen == Screen::Playing;
        if silhouette && let Some(bare) = &g.room_backdrop {
            if self.backdrop.as_ref().is_none_or(|b| b.0 != g.backdrop_serial) {
                let plain = view::Canvas::from_map(bare, room_dest);
                let mut soft = view::Canvas { px: plain.px.clone(), fore: Vec::new() };
                soft.soften(4, 0.5);
                self.backdrop = Some((g.backdrop_serial, plain, soft));
            }
            if let Some((_, plain, soft)) = &self.backdrop {
                room.soften_backdrop(plain, soft);
            }
        }
        let placed = self.placed;
        let screen = self.screen;
        let status = status_line(g, screen);
        let labels: Vec<game::Label> = g.labels.clone();
        let overlay = self.overlay.clone();
        let _ = self.terminal.draw(|f| {
            let buf: &mut Buffer = f.buffer_mut();
            if silhouette {
                view::draw_braille(buf, &room, &placed);
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

const TITLE_KEYS: &str = "Enter: play   L: houses   T: scores   H: help   M: sound   B: music   V: view   Q: quit";

fn title_lines(g: &Game, t: &Term) -> Vec<String> {
    let on = |b: bool| if b { "on" } else { "off" };
    vec![
        "GLIDER PRO".into(),
        format!("John Calhoun, 1994   house: {}", g.this_house_name),
        String::new(),
        "Arrows fly: Down burns battery, Up fires a rubber band.".into(),
        format!("Sound {}, music {}, {} view (V).", on(t.sound_on), on(t.music_on), if g.silhouette { "silhouette" } else { "original" }),
        String::new(),
        TITLE_KEYS.into(),
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

fn run(slot: &mut Option<Box<Term>>, g: &mut Game, settings: &mut Settings) {
    loop {
        // ---- the title
        {
            {
                let term = slot.as_mut().expect("terminal");
                term.screen = Screen::Title;
                term.input.clear();
            }
            draw_title(g);
            let mut help = false;
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
                for c in cmds {
                    match c {
                        Cmd::Quit | Cmd::Back => return,
                        Cmd::Confirm => play = true,
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
        lend(slot, g, |g| g.new_game(game::NEW_GAME_MODE));
        let mut next = Instant::now();
        while g.playing && !g.quitting {
            {
                let term = slot.as_mut().expect("terminal");
                term.read_events(next);
                let cmds = term.take_cmds(g);
                for c in &cmds {
                    match c {
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
                        }
                        _ => {}
                    }
                }
                term.input.keys(Instant::now(), &mut g.keys);
            }
            if !g.paused {
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
        if g.quitting {
            return;
        }
        lend(slot, g, |g| g.play_game_finish());
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

/// `--frames N`: a game with nobody at the keys, without a terminal.
fn smoke(a: Rc<Assets>, house: &str, frames: u32) {
    let Some(mut g) = new_game_state(a, house) else {
        eprintln!("cannot read house {house:?}");
        std::process::exit(1);
    };
    g.screen_io = Some(Box::new(NoIo));
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
    let hold = std::env::var("GLIDER_PRO_HOLD").unwrap_or_default();
    while n < frames && g.playing {
        g.keys.left = hold.contains("left");
        g.keys.right = hold.contains("right");
        g.keys.batt = hold.contains("batt");
        g.keys.band = hold.contains("band");
        g.ticks += game::TICKS_PER_FRAME;
        g.play_game_frame();
        g.sound_queue.clear();
        g.music_queue.clear();
        n += 1;
        if std::env::var_os("GLIDER_PRO_TRACE").is_some() && n % 10 == 0 {
            let gl = &g.gliders[0];
            eprintln!("frame {n}: room {} mode {} dest {:?} vel ({}, {})", g.this_room_number, gl.mode, gl.dest, gl.h_vel, gl.v_vel);
        }
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

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut house = None;
    let mut mute = false;
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
            "--houses" => {
                for (n, _) in house::BUILT_IN {
                    println!("{n}");
                }
                return;
            }
            "--mute" => mute = true,
            "--frames" => frames = it.next().and_then(|n| n.parse().ok()),
            s if !s.starts_with('-') => house = Some(s.to_string()),
            s => {
                eprintln!("unknown option {s}\n\n{HELP}");
                std::process::exit(2);
            }
        }
    }
    let mut settings = Settings::load();
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
        backdrop: None,
    };
    let mut slot = Some(Box::new(term));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut slot, &mut g, &mut settings)));
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

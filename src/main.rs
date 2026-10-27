mod assets;
mod gfx;
mod house;
mod mac;
mod view;

use assets::Assets;
use gfx::Canvas;
use house::{House, ROOM_IS_EMPTY, Rect, TILE_HIGH, TILE_WIDE};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Rect as Area;
use ratatui::style::{Color, Style};
use std::io;

const HELP: &str = "glider-pro: Glider PRO (John Calhoun, 1994) in the terminal, based on Aerofoil

Work in progress: for now a house explorer. Arrows move between rooms
(left and right along a floor, up and down between floors), Page Up and
Page Down step through the rooms in order, H the next house, V the view,
Q quits.

usage: glider-pro [--house NAME] [--room N] [--shot [--size WxH]] [--blocks]
  --houses     list the built-in houses";

// background pictures for where there is no room (RoomGraphics.cpp)
const SKY: i32 = 2015;
const MEADOW: i32 = 2012;
const DIRT: i32 = 2011;

/// DrawRoomBackground: the room's eight tiles, each a 64-pixel column of
/// its background picture. A spot with no room shows sky above the ground
/// floor, meadow on it, dirt below.
fn room_background(a: &Assets, h: &House, room: Option<usize>, floor: i32) -> Canvas {
    let mut c = Canvas::new();
    c.fill(gfx::WHOLE, gfx::BLACK);
    let (pict, tiles) = match room {
        Some(r) => (h.rooms[r].background, h.rooms[r].tiles),
        None if floor > 1 => (SKY, [2; 8]),
        None if floor == 1 => (MEADOW, [0; 8]),
        None => (DIRT, [0; 8]),
    };
    // LoadGraphicSpecial falls back on picture 2000
    let Some(bg) = a.pict(Some(h), pict as i16).or_else(|| a.pict(Some(h), 2000)) else { return c };
    for (i, &t) in tiles.iter().enumerate() {
        let src = Rect::new(t * TILE_WIDE, 0, t * TILE_WIDE + TILE_WIDE, TILE_HIGH);
        let dst = src.at(i as i32 * TILE_WIDE, 0);
        c.copy_bits(&bg, src, dst);
    }
    c
}

struct App {
    a: Assets,
    house_ix: usize,
    house: House,
    room: usize,
    braille: bool,
    quit: bool,
}

impl App {
    fn load_house(&mut self, ix: usize) {
        let name = house::BUILT_IN[ix].0;
        if let Some(h) = house::load(name) {
            self.a.new_house();
            self.house_ix = ix;
            self.room = h.rooms.iter().position(|r| r.suite != ROOM_IS_EMPTY).unwrap_or(0);
            if (0..h.rooms.len() as i32).contains(&h.first_room) {
                self.room = h.first_room as usize;
            }
            self.house = h;
            log::info!("house {name:?}, {} rooms, first {}", self.house.rooms.len(), self.house.first_room);
        }
    }

    /// The room beside this one, if there is one.
    fn step(&mut self, df: i32, ds: i32) {
        let r = &self.house.rooms[self.room];
        if let Some(n) = self.house.room_at(r.floor + df, r.suite + ds) {
            self.room = n;
        }
    }

    fn key(&mut self, k: KeyCode) {
        let n = self.house.rooms.len();
        match k {
            KeyCode::Left => self.step(0, -1),
            KeyCode::Right => self.step(0, 1),
            KeyCode::Up => self.step(1, 0),
            KeyCode::Down => self.step(-1, 0),
            KeyCode::PageDown => {
                for d in 1..n {
                    let r = (self.room + d) % n;
                    if self.house.rooms[r].suite != ROOM_IS_EMPTY {
                        self.room = r;
                        break;
                    }
                }
            }
            KeyCode::PageUp => {
                for d in 1..n {
                    let r = (self.room + n - d) % n;
                    if self.house.rooms[r].suite != ROOM_IS_EMPTY {
                        self.room = r;
                        break;
                    }
                }
            }
            KeyCode::Char('h' | 'H') => self.load_house((self.house_ix + 1) % house::BUILT_IN.len()),
            KeyCode::Char('v' | 'V') => self.braille = !self.braille,
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => self.quit = true,
            _ => {}
        }
    }

    fn draw(&self, buf: &mut Buffer, area: Area) {
        buf.set_style(area, Style::default().bg(Color::Rgb(0, 0, 0)));
        let pic = Area::new(area.x, area.y, area.width, area.height.saturating_sub(1));
        let placed = view::place(pic, view::cell_aspect());
        let r = &self.house.rooms[self.room];
        let c = room_background(&self.a, &self.house, Some(self.room), r.floor);
        if self.braille {
            view::draw_braille(buf, &c, &placed);
        } else {
            view::draw(buf, &c, &placed);
        }
        let line = format!(
            "{}  room {} {:?}  floor {} suite {}  ({} objects)   arrows move  PgUp/PgDn  H house  V view  Q quit",
            house::BUILT_IN[self.house_ix].0,
            self.room,
            mac::mac_roman(&r.name),
            r.floor,
            r.suite,
            r.num_objects
        );
        let y = placed.cells.bottom().min(area.bottom().saturating_sub(1));
        let x = area.x + area.width.saturating_sub(line.chars().count() as u16) / 2;
        buf.set_string(x, y, line, Style::default().fg(Color::Rgb(255, 210, 90)));
    }
}

fn main() -> io::Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |f: &str| args.iter().any(|a| a == f);
    let value = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    if has("--help") || has("-h") {
        println!("{HELP}");
        return Ok(());
    }
    if has("--houses") {
        for (n, _) in house::BUILT_IN {
            println!("{n}");
        }
        return Ok(());
    }
    let ix = value("--house").and_then(|n| house::BUILT_IN.iter().position(|(h, _)| *h == n)).unwrap_or(0);
    let mut app = App { a: Assets::load(), house_ix: 0, house: house::load(house::BUILT_IN[0].0).expect("Demo House"), room: 0, braille: !has("--blocks"), quit: false };
    app.load_house(ix);
    if let Some(r) = value("--room").and_then(|v| v.parse::<usize>().ok()) {
        app.room = r.min(app.house.rooms.len().saturating_sub(1));
    }
    if has("--shot") {
        let (w, h) = value("--size").and_then(|v| v.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))).unwrap_or((128u16, 44u16));
        let area = Area::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        app.draw(&mut buf, area);
        let mut out = String::new();
        for y in 0..h {
            for x in 0..w {
                let c = &buf[(x, y)];
                for (code, col) in [(38, c.fg), (48, c.bg)] {
                    if let Color::Rgb(r, g, b) = col {
                        out.push_str(&format!("\x1b[{code};2;{r};{g};{b}m"));
                    }
                }
                out.push_str(c.symbol());
            }
            out.push_str("\x1b[0m\n");
        }
        print!("{out}");
        return Ok(());
    }
    let mut terminal = ratatui::init();
    while !app.quit {
        terminal.draw(|f| {
            let area = f.area();
            app.draw(f.buffer_mut(), area);
        })?;
        if let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            app.key(k.code);
        }
    }
    ratatui::restore();
    Ok(())
}

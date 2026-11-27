//! The intro page and the menu of instruments (D125 items 2 to 4; docs/TERMINAL.md, "The
//! command, the menu and the config"). The look is a stand-in in Loom's tokens (D125 item 7).
//!
//! SPDX-License-Identifier: MIT
use crate::host::Mode;
use crate::input::{Key, KeyEvent, Kind};
use crate::screen::{Grid, Tok};
use crate::settings::{From, INSTRUMENTS, Settings};

/// What the menu shows: the chosen line and whether `?` is open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Menu {
    pub chosen: usize,
    pub help: bool,
}

/// What a key on the menu asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Stay,
    /// Start the instrument at this place in `INSTRUMENTS`.
    Start(usize),
    Quit,
}

impl Menu {
    /// One key event. The menu acts on presses only (docs/TERMINAL.md).
    pub fn key(&mut self, e: KeyEvent) -> Choice {
        if e.kind != Kind::Press {
            return Choice::Stay;
        }
        if e.ctrl {
            return if matches!(e.key, Key::Char('q') | Key::Char('c')) { Choice::Quit } else { Choice::Stay };
        }
        let question = e.key == Key::Char('/') && e.shift;
        if self.help {
            if question || e.key == Key::Esc {
                self.help = false;
            }
            return Choice::Stay;
        }
        match e.key {
            _ if question => {
                self.help = true;
                Choice::Stay
            }
            Key::Enter => Choice::Start(self.chosen),
            Key::Char('q') if !e.shift => Choice::Quit,
            Key::Char(c) if !e.shift && c.is_ascii_digit() => {
                let n = c as usize - '0' as usize;
                if (1..=INSTRUMENTS.len()).contains(&n) {
                    self.chosen = n - 1;
                    Choice::Start(n - 1)
                } else {
                    Choice::Stay
                }
            }
            _ => Choice::Stay,
        }
    }
}

/// Everything the intro page shows besides the menu.
pub struct Intro<'a> {
    pub settings: &'a Settings,
    /// The key mode chosen at start, and why.
    pub mode: Mode,
    pub why: &'a str,
    /// The config file's path, and whether it was there.
    pub config: Option<(&'a str, bool)>,
    /// A line to say, as a sound device that would not open.
    pub note: Option<&'a str>,
    pub version: &'a str,
    /// The `--help` text, for `?`.
    pub help: &'a str,
}

const WHAT: &str = "Small calculator-format instruments, free and open source.";
const HOMAGE: &str = "An homage to Teenage Engineering's pocket operators; not affiliated.";

fn rows(i: &Intro) -> [(&'static str, String, From); 4] {
    let s = i.settings;
    let keys = match (s.keys.0, i.mode) {
        (Some(_), m) => mode_word(m).to_string(),
        (None, m) => format!("{} ({})", mode_word(m), i.why),
    };
    let audio = match s.audio.0 {
        crate::audio::Backend::Default => "default",
        crate::audio::Backend::PipeWire => "pipewire",
        crate::audio::Backend::None => "none, silent",
    };
    [
        ("keys", keys, s.keys.1),
        ("sound", audio.to_string(), s.audio.1),
        ("colour", if s.mono.0 { "off" } else { "on" }.to_string(), s.mono.1),
        ("motion", if s.reduced.0 { "reduced" } else { "full" }.to_string(), s.reduced.1),
    ]
}

fn mode_word(m: Mode) -> &'static str {
    match m {
        Mode::Hold => "hold",
        Mode::Tap => "tap",
    }
}

/// `1 or enter play   ? options   q quit`: keys in ecru, words in lilac.
fn footer(g: &mut Grid, y: usize, help: bool, pocket: bool) {
    let parts: &[(&str, &str)] = if help {
        &[("esc or ?", "back")]
    } else if pocket {
        &[("1", "play"), ("?", "options"), ("q", "quit")]
    } else {
        &[("1 or enter", "play"), ("?", "options"), ("q", "quit"), ("ctrl+q", "quit from anywhere")]
    };
    let mut x = 1;
    for (k, w) in parts {
        x = g.text(x, y, k, Tok::Ecru, Tok::Ground);
        x = g.text(x + 1, y, w, Tok::Lilac, Tok::Ground) + 3;
    }
}

fn menu_lines(g: &mut Grid, m: &Menu, x: usize, y: usize, width: usize) {
    for (n, (name, words)) in INSTRUMENTS.iter().enumerate() {
        let lit = n == m.chosen;
        let (fg, bg) = if lit { (Tok::Ground, Tok::Ecru) } else { (Tok::Ecru, Tok::Ground) };
        g.fill(x, y + n, width, 1, bg);
        let xx = g.text(x + 1, y + n, &format!("{}", n + 1), fg, bg);
        let xx = g.text(xx + 2, y + n, name, fg, bg);
        if width > 24 {
            g.text(xx.max(x + 12), y + n, words, if lit { Tok::Ground } else { Tok::Lilac }, bg);
        }
    }
}

fn help_text(g: &mut Grid, text: &str, y0: usize, h: usize) {
    let w = g.w;
    g.fill(0, y0, w, h, Tok::Display);
    let lines: Vec<&str> = text.lines().collect();
    let room = if lines.len() > h { h - 1 } else { h };
    for (r, line) in lines.iter().take(room).enumerate() {
        let line: String = line.chars().take(w - 2).collect();
        g.text(1, y0 + r, &line, Tok::Ecru, Tok::Display);
    }
    if lines.len() > h {
        g.text(1, y0 + h - 1, "the rest: pocket-instruments --help", Tok::Lilac, Tok::Display);
    }
}

/// The intro page at 80 by 24 (D125 item 2).
pub fn intro_screen(m: &Menu, i: &Intro) -> Grid {
    let mut g = Grid::new(80, 24, Tok::Ground);
    let x = g.text(1, 0, "pocket instruments", Tok::Ecru, Tok::Ground);
    g.text(x + 2, 0, i.version, Tok::Lilac, Tok::Ground);
    if m.help {
        help_text(&mut g, i.help, 1, 22);
        footer(&mut g, 23, true, false);
        return g;
    }
    g.text(2, 2, WHAT, Tok::Ecru, Tok::Ground);
    g.text(2, 3, HOMAGE, Tok::Lilac, Tok::Ground);
    g.text(2, 6, "instruments", Tok::Lilac, Tok::Ground);
    menu_lines(&mut g, m, 2, 7, 50);
    g.text(2, 11, "settings", Tok::Lilac, Tok::Ground);
    for (r, (name, value, from)) in rows(i).iter().enumerate() {
        let y = 12 + r;
        g.text(4, y, name, Tok::Lilac, Tok::Ground);
        let x = g.text(12, y, value, Tok::Ecru, Tok::Ground);
        g.text(x + 2, y, from.word(), Tok::Lilac, Tok::Ground);
    }
    if let Some(note) = i.note {
        let note: String = note.chars().take(76).collect();
        g.text(2, 18, &note, Tok::Madder, Tok::Ground);
    }
    if let Some((path, there)) = i.config {
        let x = g.text(2, 20, "config", Tok::Lilac, Tok::Ground);
        let room = 80usize.saturating_sub(x + 1 + 12);
        let shown: String = if path.chars().count() > room {
            let tail: Vec<char> = path.chars().rev().take(room.saturating_sub(1)).collect();
            std::iter::once('…').chain(tail.into_iter().rev()).collect()
        } else {
            path.to_string()
        };
        let x = g.text(x + 1, 20, &shown, Tok::Ecru, Tok::Ground);
        g.text(x + 1, 20, if there { "read" } else { "not there" }, Tok::Lilac, Tok::Ground);
    }
    footer(&mut g, 23, false, false);
    g
}

/// The pocket size, 48 by 10: the name, the menu and the keys (docs/TERMINAL.md).
pub fn intro_pocket(m: &Menu, i: &Intro) -> Grid {
    let mut g = Grid::new(48, 10, Tok::Ground);
    let x = g.text(1, 0, "pocket instruments", Tok::Ecru, Tok::Ground);
    g.text(x + 2, 0, i.version, Tok::Lilac, Tok::Ground);
    if m.help {
        help_text(&mut g, i.help, 1, 8);
        footer(&mut g, 9, true, true);
        return g;
    }
    menu_lines(&mut g, m, 1, 2, 46);
    let r = rows(i);
    let mut y = 5;
    for pair in r.chunks(2) {
        let mut x = 1;
        for (name, value, _) in pair {
            let v: String = value.split(" (").next().unwrap_or(value).to_string();
            x = g.text(x, y, name, Tok::Lilac, Tok::Ground);
            x = g.text(x + 1, y, &v, Tok::Ecru, Tok::Ground) + 3;
        }
        y += 1;
    }
    if let Some(note) = i.note {
        let note: String = note.chars().take(46).collect();
        g.text(1, 7, &note, Tok::Madder, Tok::Ground);
    }
    footer(&mut g, 9, false, true);
    g
}

/// Too small for either.
pub fn intro_tiny(w: usize, h: usize) -> Grid {
    let mut g = Grid::new(w.max(1), h.max(1), Tok::Ground);
    g.text(0, 0, "pocket instruments needs 48 by 10", Tok::Ecru, Tok::Ground);
    g
}

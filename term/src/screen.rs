//! The two screens, drawn from the view (docs/TERMINAL.md, "The screens"; `ENGINE.md` §14's
//! host table, "§14 and §17, designed 2026-11-26"; `design/DESIGN.md`, "Loom in a terminal").
//!
//! A screen is a grid of cells, each a character in two of Loom's tokens. Output turns tokens
//! into truecolour, or with no colour into normal, bold and reverse only (D115 item 5).
//!
//! SPDX-License-Identifier: MIT
use crate::ffi::*;
use crate::host::{Host, Mode, Overlay, char_of};
use std::fmt::Write as _;

/// Loom's tokens (`design/DESIGN.md`, "Tokens").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tok {
    Ground,
    Tile,
    Display,
    Table,
    Ecru,
    White,
    Ochre,
    Madder,
    Sky,
    Fern,
    Lilac,
    Thread,
}

impl Tok {
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Tok::Ground => (0x14, 0x13, 0x1f),
            Tok::Tile => (0x2b, 0x29, 0x42),
            Tok::Display => (0x0b, 0x0a, 0x12),
            Tok::Table => (0x07, 0x06, 0x0c),
            Tok::Ecru => (0xf2, 0xe8, 0xd0),
            Tok::White => (0xff, 0xff, 0xff),
            Tok::Ochre => (0xf0, 0xb4, 0x29),
            Tok::Madder => (0xf0, 0x65, 0x4a),
            Tok::Sky => (0x8f, 0xaa, 0xf7),
            Tok::Fern => (0x6f, 0xd6, 0xa0),
            Tok::Lilac => (0xc9, 0xc2, 0xdd),
            Tok::Thread => (0x6e, 0x6a, 0x92),
        }
    }

    /// With no colour a token is a level (docs/TERMINAL.md): nothing, normal or bold.
    pub fn level(self) -> Level {
        match self {
            Tok::Ground | Tok::Tile | Tok::Display | Tok::Table => Level::None,
            Tok::White | Tok::Ochre => Level::Bold,
            _ => Level::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    None,
    Normal,
    Bold,
}

/// One cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// A character in `fg` on `bg`; `under` underlines it in that colour.
    Text { ch: char, fg: Tok, bg: Tok, under: Option<Tok> },
    /// Upper half `top`, lower half `bottom` (drawn `▀`).
    Half { top: Tok, bottom: Tok },
    /// Left half `left`, right half `right` (drawn `▌`).
    Side { left: Tok, right: Tok },
}

impl Cell {
    pub const fn blank(bg: Tok) -> Cell {
        Cell::Text { ch: ' ', fg: Tok::Ecru, bg, under: None }
    }
    pub const fn text(ch: char, fg: Tok, bg: Tok) -> Cell {
        Cell::Text { ch, fg, bg, under: None }
    }
    pub fn ch(&self) -> char {
        match *self {
            Cell::Text { ch, .. } => ch,
            Cell::Half { top, bottom } if top == bottom => ' ',
            Cell::Half { .. } => '▀',
            Cell::Side { left, right } if left == right => ' ',
            Cell::Side { .. } => '▌',
        }
    }

    /// The cell with no colour: the character drawn and its attributes (D115 item 5).
    pub fn mono(&self) -> (char, Attr) {
        let halves = |a: Tok, b: Tok, both: char, first: char, second: char| {
            let (la, lb) = (a.level(), b.level());
            let bold = la.max(lb) == Level::Bold;
            match (la == Level::None, lb == Level::None) {
                (true, true) => (' ', Attr::default()),
                (false, true) => (first, Attr { bold: la == Level::Bold, ..Attr::default() }),
                (true, false) => (second, Attr { bold: lb == Level::Bold, ..Attr::default() }),
                (false, false) => (both, Attr { bold, ..Attr::default() }),
            }
        };
        match *self {
            Cell::Half { top, bottom } => halves(top, bottom, '█', '▀', '▄'),
            Cell::Side { left, right } => halves(left, right, '█', '▌', '▐'),
            Cell::Text { ch, fg, bg, under } => {
                let u = under.is_some_and(|u| u.level() != Level::None);
                if bg.level() != Level::None {
                    (ch, Attr { reverse: true, bold: bg.level() == Level::Bold, underline: u })
                } else if fg.level() == Level::None || ch == ' ' {
                    (' ', Attr { underline: u, ..Attr::default() })
                } else {
                    (ch, Attr { bold: fg.level() == Level::Bold, underline: u, ..Attr::default() })
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attr {
    pub bold: bool,
    pub reverse: bool,
    pub underline: bool,
}

pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(w: usize, h: usize, bg: Tok) -> Grid {
        Grid { w, h, cells: vec![Cell::blank(bg); w * h] }
    }
    pub fn at(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.w + x]
    }
    pub fn put(&mut self, x: usize, y: usize, c: Cell) {
        if x < self.w && y < self.h {
            self.cells[y * self.w + x] = c;
        }
    }
    pub fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, bg: Tok) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, Cell::blank(bg));
            }
        }
    }
    /// Text from (x, y), clipped at `max` cells. Returns where it ended.
    pub fn text(&mut self, x: usize, y: usize, s: &str, fg: Tok, bg: Tok) -> usize {
        let mut xx = x;
        for ch in s.chars() {
            self.put(xx, y, Cell::text(ch, fg, bg));
            xx += 1;
        }
        xx
    }
    /// The characters, a line a row, as drawn in colour.
    pub fn plain(&self) -> String {
        let mut s = String::new();
        for y in 0..self.h {
            for x in 0..self.w {
                s.push(self.at(x, y).ch());
            }
            s.push('\n');
        }
        s
    }
    /// The characters as drawn with no colour.
    pub fn plain_mono(&self) -> String {
        let mut s = String::new();
        for y in 0..self.h {
            for x in 0..self.w {
                s.push(self.at(x, y).mono().0);
            }
            s.push('\n');
        }
        s
    }
}

pub const NAMES: [&str; 16] = [
    "bass drum", "snare drum", "closed hat", "open hat", "synth snare", "sticks", "cymbal", "noise", "hand clap", "click",
    "low tom", "high tom", "cow bell", "blip", "tone", "bass tone",
];
/// Each effect in our words (`rack/src/Rhythm.cpp`, `FX_NAMES`; `ENGINE.md` §17, §18).
pub const FX_NAMES: [&str; 16] = [
    "low sample rate", "distortion", "bit crush", "delay", "lowpass", "lowpass sweep", "hipass", "hipass sweep",
    "repeat 4 a beat", "repeat 3 a beat", "repeat 8 a beat", "repeat 6 a beat", "note shuffle", "feedback", "15 (none yet)",
    "16 (none yet)",
];
const FN_WORDS: [&str; 7] = ["sound", "pattern", "bpm", "function", "fx", "play", "write"];

/// A function key's own colour: the band, and its latched face for the top three.
pub fn fn_colour(k: u8) -> Tok {
    match k {
        PI_KEY_SOUND => Tok::Madder,
        PI_KEY_PATTERN => Tok::Ochre,
        PI_KEY_BPM => Tok::Sky,
        PI_KEY_FX => Tok::Fern,
        _ => Tok::Ecru,
    }
}

/// Dark ink on a lit ground, ecru on a dark one.
pub fn ink(bg: Tok) -> Tok {
    if bg.level() == Level::None { Tok::Ecru } else { Tok::Ground }
}

/// The core's clock (§14: a firing lights the core for half a step at the view's tempo, held
/// to 40 to 120 ms). Kept by the screen, from the `fires` counters.
#[derive(Default)]
pub struct Cores {
    seen: Option<[u8; 16]>,
    until: [u64; 16],
}

impl Cores {
    /// Takes a view at `now_ms` and returns the sounds whose core is lit.
    pub fn update(&mut self, v: &[u8; PI_VIEW_SIZE], now_ms: u64) -> u16 {
        let ext = u16_at(v, PI_VIEW_EXT) as u64;
        let mbpm = if ext != 0 { ext * 100 } else { u32_at(v, PI_VIEW_TEMPO) as u64 }.max(1);
        // Half a step: 60 000 ms / (bpm × 4) / 2 = 7 500 000 / mbpm.
        let hold = (7_500_000 / mbpm).clamp(40, 120);
        let fires: [u8; 16] = v[PI_VIEW_FIRES..PI_VIEW_FIRES + 16].try_into().unwrap();
        if let Some(seen) = self.seen {
            for t in 0..16 {
                if fires[t] != seen[t] {
                    self.until[t] = now_ms + hold;
                }
            }
        }
        self.seen = Some(fires);
        (0..16).filter(|&t| now_ms < self.until[t]).fold(0, |m, t| m | 1 << t)
    }
}

/// Every light of grid key `n` (design/DESIGN.md, "A key's lights"), read from the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyLights {
    pub frame: Tok,
    pub face: Tok,
    pub pressed: bool,
    pub lit: bool,
    pub twill: bool,
    pub core: Option<Tok>,
    pub stitch: bool,
    pub bar: Option<Tok>,
    pub mark: Option<char>,
    pub legend: char,
}

pub fn key_lights(v: &[u8; PI_VIEW_SIZE], cores: u16, latched: u32, n: usize) -> KeyLights {
    let flags = v[PI_VIEW_FLAGS];
    let held = u32_at(v, PI_VIEW_HELD);
    let bit = |m: u16| m >> n & 1 != 0;
    let pressed = held >> n & 1 != 0;
    let lit = bit(u16_at(v, PI_VIEW_FIELD));
    let playhead = flags & PI_VIEW_FLAG_PLAYING != 0 && flags & PI_VIEW_FLAG_STEP != 0 && v[PI_VIEW_STEP] as usize == n;
    let face = if pressed { Tok::White } else if lit { Tok::Ecru } else { Tok::Tile };
    let frame = if pressed { Tok::Ground } else if playhead { Tok::Ochre } else { face };
    let heard = v[PI_VIEW_HEARD] as usize;
    let core = (cores >> n & 1 != 0).then(|| if heard != 0 && bit(u16_at(v, PI_VIEW_FX_SOUNDS)) { Tok::Fern } else { Tok::White });
    let bar = bit(u16_at(v, PI_VIEW_BAR)).then(|| if held >> PI_KEY_PATTERN & 1 != 0 { Tok::Ochre } else { Tok::Madder });
    // The source mark (D112 item 4; §14): while sound, or fx, is held or latched; sound's
    // first. A terminal sends no ROUTE, so only the dash can show (a sample).
    let down = |k: u8| held >> k & 1 != 0 || latched >> k & 1 != 0;
    let mark = if down(PI_KEY_SOUND) {
        bit(u16_at(v, PI_VIEW_SAMPLED)).then_some('-')
    } else {
        None
    };
    KeyLights {
        frame,
        face,
        pressed,
        lit,
        twill: heard == n + 1,
        core,
        stitch: bit(u16_at(v, PI_VIEW_STITCH)),
        bar,
        mark,
        legend: char_of(n as u8),
    }
}

/// The three cells between a key's frame sides: left (mark or twill), centre (legend or
/// core), right (stitch or twill). With the core on a lit face the two beside it go dark,
/// standing in for Loom's dark line (D115 item 4).
fn inner(k: &KeyLights, under: Option<Tok>) -> [Cell; 3] {
    let dark_flank = k.core.is_some() && k.face != Tok::Tile;
    let flank_bg = if dark_flank { Tok::Ground } else { k.face };
    let twill = Cell::Text { ch: '╱', fg: Tok::Fern, bg: Tok::Ground, under };
    let left = match k.mark {
        Some(m) => Cell::Text { ch: m, fg: ink(flank_bg), bg: flank_bg, under },
        None if k.twill => twill,
        None => Cell::Text { ch: ' ', fg: ink(flank_bg), bg: flank_bg, under },
    };
    let centre = match k.core {
        Some(c) => Cell::Text { ch: k.legend, fg: Tok::Ground, bg: c, under },
        None => Cell::Text { ch: k.legend, fg: ink(k.face), bg: k.face, under },
    };
    // The stitch is not drawn under the twill (`Lights`' terminal rule ⚠).
    let right = if k.twill {
        twill
    } else if k.stitch {
        Cell::Text { ch: '▪', fg: ink(flank_bg), bg: flank_bg, under }
    } else {
        Cell::Text { ch: ' ', fg: ink(flank_bg), bg: flank_bg, under }
    };
    [left, centre, right]
}

/// A key in 5 by 3 cells (D115 item 3), its top left at (x, y), on `ground`.
pub fn draw_key(g: &mut Grid, x: usize, y: usize, k: &KeyLights, ground: Tok) {
    for i in 0..5 {
        g.put(x + i, y, Cell::Half { top: ground, bottom: k.frame });
        g.put(x + i, y + 2, Cell::Half { top: k.frame, bottom: k.bar.unwrap_or(ground) });
    }
    g.put(x, y + 1, Cell::Side { left: k.frame, right: k.face });
    let [a, b, c] = inner(k, None);
    g.put(x + 1, y + 1, a);
    g.put(x + 2, y + 1, b);
    g.put(x + 3, y + 1, c);
    g.put(x + 4, y + 1, Cell::Side { left: k.face, right: k.frame });
}

/// A key in one row of 5 cells (the pocket size): the frame is the two side cells, the bar an
/// underline under the three between (⚠ D123 item 6).
pub fn draw_pocket_key(g: &mut Grid, x: usize, y: usize, k: &KeyLights) {
    g.put(x, y, Cell::blank(k.frame));
    let [a, b, c] = inner(k, k.bar);
    g.put(x + 1, y, a);
    g.put(x + 2, y, b);
    g.put(x + 3, y, c);
    g.put(x + 4, y, Cell::blank(k.frame));
}

/// How a function key stands (`States`): idle, pressed, latched, or lit (play, write).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FnState {
    Idle,
    Pressed,
    Latched,
    Lit,
}

pub fn fn_state(v: &[u8; PI_VIEW_SIZE], latched: u32, k: u8) -> FnState {
    let held = u32_at(v, PI_VIEW_HELD) >> k & 1 != 0;
    let flags = v[PI_VIEW_FLAGS];
    if latched >> k & 1 != 0 {
        FnState::Latched
    } else if held {
        FnState::Pressed
    } else if (k == PI_KEY_PLAY && flags & PI_VIEW_FLAG_PLAYING != 0) || (k == PI_KEY_WRITE && flags & PI_VIEW_FLAG_WRITE != 0) {
        FnState::Lit
    } else {
        FnState::Idle
    }
}

/// (band, face, cap ink, cap background, word ink) for a function key.
fn fn_look(k: u8, s: FnState) -> (Tok, Tok, Tok, Tok, Tok) {
    let colour = fn_colour(k);
    // Latched, the top three take their own colour; function, fx, play and write go ecru (the
    // page's rule: fx's band alone is fern).
    let latched_face = if k <= PI_KEY_BPM { colour } else { Tok::Ecru };
    match s {
        FnState::Idle => (colour, Tok::Tile, Tok::Lilac, Tok::Tile, Tok::Ecru),
        FnState::Pressed => (Tok::Ground, Tok::White, Tok::Ground, Tok::White, Tok::Ground),
        FnState::Latched => (colour, latched_face, latched_face, Tok::Ground, Tok::Ground),
        FnState::Lit => (colour, Tok::Ecru, Tok::Ground, Tok::Ecru, Tok::Ground),
    }
}

fn draw_fn_key(g: &mut Grid, x: usize, y: usize, w: usize, k: u8, s: FnState) {
    let (band, face, cap_ink, cap_bg, word_ink) = fn_look(k, s);
    for i in 0..w {
        g.put(x + i, y, Cell::Half { top: Tok::Ground, bottom: band });
        for r in 1..4 {
            g.put(x + i, y + r, Cell::blank(face));
        }
        g.put(x + i, y + 4, Cell::Half { top: face, bottom: Tok::Ground });
    }
    g.put(x + 1, y + 1, Cell::text(char_of(k), cap_ink, cap_bg));
    g.text(x + 1, y + 2, FN_WORDS[k as usize - 16], word_ink, face);
}

fn draw_pocket_fn(g: &mut Grid, x: usize, y: usize, w: usize, k: u8, s: FnState) {
    let (band, face, cap_ink, cap_bg, word_ink) = fn_look(k, s);
    let band = if s == FnState::Pressed { Tok::White } else { band };
    g.put(x, y, Cell::blank(band));
    g.fill(x + 1, y, w - 1, 1, face);
    g.put(x + 2, y, Cell::text(char_of(k), cap_ink, cap_bg));
    g.text(x + 4, y, FN_WORDS[k as usize - 16], word_ink, face);
}

/// What the screen knows besides the view.
pub struct Scene<'a> {
    pub view: &'a [u8; PI_VIEW_SIZE],
    pub weave: &'a [u8; PI_WEAVE_CELLS],
    pub cores: u16,
    pub host: &'a Host,
    /// A line to show in the footer for a while (saved, a refusal).
    pub message: Option<&'a str>,
}

fn weave_tok(c: u8) -> Tok {
    match c {
        1 => Tok::Thread,
        2 => Tok::Ecru,
        3 => Tok::White,
        4 => Tok::Ochre,
        5 => Tok::Tile,
        6 => Tok::Madder,
        _ => Tok::Display,
    }
}

/// The function key whose meaning the display lists (D117 item 2): held or latched; sound,
/// then bpm, pattern, function, fx.
fn shown_fn(v: &[u8; PI_VIEW_SIZE], latched: u32) -> Option<u8> {
    let held = u32_at(v, PI_VIEW_HELD) | latched;
    [PI_KEY_SOUND, PI_KEY_BPM, PI_KEY_PATTERN, PI_KEY_FUNCTION, PI_KEY_FX].into_iter().find(|&k| held >> k & 1 != 0)
}

fn tempo_text(v: &[u8; PI_VIEW_SIZE]) -> String {
    let ext = u16_at(v, PI_VIEW_EXT);
    if v[PI_VIEW_FLAGS] & PI_VIEW_FLAG_EDGES != 0 {
        if ext == 0 { "--".into() } else { format!("{}", (ext as u32 + 5) / 10) }
    } else {
        format!("{}", u32_at(v, PI_VIEW_TEMPO) / 1000)
    }
}

/// The status line: `rhythm`, the pattern; a block for each held or latched function key,
/// `fx n` while an effect is heard, the write mark, the step, the tempo.
fn status(g: &mut Grid, s: &Scene, pocket: bool) {
    let v = s.view;
    let w = g.w;
    g.fill(0, 0, w, 1, Tok::Ground);
    let x = g.text(1, 0, "rhythm", Tok::Ecru, Tok::Ground);
    if !pocket {
        g.text(x + 2, 0, "pattern 1", Tok::Lilac, Tok::Ground);
    }
    let flags = v[PI_VIEW_FLAGS];
    let step = if flags & PI_VIEW_FLAG_STEP != 0 { format!("{}", v[PI_VIEW_STEP] + 1) } else { "-".into() };
    let ext = flags & PI_VIEW_FLAG_EDGES != 0;
    let mut right: Vec<(String, Tok, Tok)> = Vec::new();
    let held = u32_at(v, PI_VIEW_HELD) | s.host.latched_mask();
    for k in [PI_KEY_SOUND, PI_KEY_PATTERN, PI_KEY_BPM, PI_KEY_FUNCTION, PI_KEY_FX] {
        if held >> k & 1 != 0 && !pocket {
            right.push((format!(" {} ", FN_WORDS[k as usize - 16]), Tok::Ground, fn_colour(k)));
        }
    }
    if v[PI_VIEW_HEARD] != 0 {
        right.push((format!(" fx {} ", v[PI_VIEW_HEARD]), Tok::Ground, Tok::Fern));
    }
    if flags & PI_VIEW_FLAG_WRITE != 0 {
        right.push((" write ".into(), Tok::Ground, Tok::Ochre));
    }
    right.push((if pocket { step.clone() } else { format!("step {step}") }, Tok::Ecru, Tok::Ground));
    right.push((format!("{} {}", tempo_text(v), if ext { "ext" } else { "bpm" }), Tok::Ecru, Tok::Ground));
    let total: usize = right.iter().map(|(t, ..)| t.chars().count() + 1).sum();
    let mut x = w.saturating_sub(total);
    for (t, fg, bg) in right {
        x = g.text(x, 0, &t, fg, bg) + 1;
    }
}

/// The weave in half blocks (78 by 15 cells in 8 rows), at (1, 1).
fn weave(g: &mut Grid, s: &Scene) {
    for r in 0..8 {
        for c in 0..PI_WEAVE_W {
            let top = weave_tok(s.weave[(2 * r) * PI_WEAVE_W + c]);
            let bottom = if 2 * r + 1 < PI_WEAVE_H { weave_tok(s.weave[(2 * r + 1) * PI_WEAVE_W + c]) } else { Tok::Display };
            g.put(1 + c, 1 + r, Cell::Half { top, bottom });
        }
    }
}

/// The lane's ticks (fern) and the sixteen step marks, each mark 3 cells with 2 between.
fn marks(g: &mut Grid, s: &Scene, ticks_row: Option<usize>, marks_row: usize, x0: usize, width: usize, gap: usize) {
    let v = s.view;
    let flags = v[PI_VIEW_FLAGS];
    let playing = flags & PI_VIEW_FLAG_PLAYING != 0 && flags & PI_VIEW_FLAG_STEP != 0;
    let lane = u16_at(v, PI_VIEW_FX_STEPS);
    for st in 0..16 {
        let x = x0 + st * (width + gap);
        let (ch, fg) = if playing && v[PI_VIEW_STEP] as usize == st {
            ('█', Tok::Ochre)
        } else if st % 4 == 0 {
            ('▄', Tok::Thread)
        } else {
            ('▂', Tok::Thread)
        };
        for i in 0..width {
            g.put(x + i, marks_row, Cell::text(ch, fg, Tok::Display));
            if let Some(t) = ticks_row {
                if lane >> st & 1 != 0 {
                    g.put(x + i, t, Cell::text('▂', Tok::Fern, Tok::Display));
                }
            }
        }
        if ticks_row.is_none() && lane >> st & 1 != 0 {
            // The pocket size has no row for the ticks: the mark's own cell takes fern above.
            g.put(x, marks_row, Cell::text(ch, if ch == '█' { Tok::Ochre } else { Tok::Fern }, Tok::Display));
        }
    }
}

/// What each key now means while a function key is held or latched (D117 item 2): the
/// display's rows from `y`, `h` of them, `w` wide from `x`.
fn meanings(g: &mut Grid, s: &Scene, k: u8, x: usize, y: usize, w: usize, h: usize) {
    let v = s.view;
    g.fill(x, y, w, h, Tok::Display);
    let (title, about): (&str, String) = match k {
        PI_KEY_SOUND => ("sound", "the grid is sixteen sounds: a key chooses one and plays it".into()),
        PI_KEY_BPM => ("bpm", format!("a key sets the volume; A swing {}, B tempo {}", v[PI_VIEW_SWING], tempo_text(v))),
        PI_KEY_PATTERN => ("pattern", "one pattern, on key 1; more come with patterns (issue 4)".into()),
        PI_KEY_FUNCTION => ("function", "with pattern: clears the pattern".into()),
        _ => ("fx", "hold a key for its effect; alone, it takes effects off the steps".into()),
    };
    let tx = g.text(x + 1, y, title, fn_colour(k), Tok::Display);
    g.text(tx + 1, y, &about[..about.len().min(w.saturating_sub(tx - x + 2))], Tok::Lilac, Tok::Display);
    if matches!(k, PI_KEY_PATTERN | PI_KEY_FUNCTION) {
        return;
    }
    let col = w / 4;
    let rows = (h - 1).min(8);
    let step = (rows / 4).max(1);
    for n in 0..16usize {
        let (cx, cy) = (x + 1 + (n % 4) * col, y + 1 + (n / 4) * step + if step > 1 { 1 } else { 0 });
        if cy >= y + h {
            continue;
        }
        let (label, on): (String, bool) = match k {
            PI_KEY_SOUND => (NAMES[n].into(), v[PI_VIEW_SEL] as usize == n),
            PI_KEY_BPM => (format!("volume {}", n + 1), (v[PI_VIEW_VOLUME] as usize) == n + 1),
            _ => (FX_NAMES[n].into(), v[PI_VIEW_HEARD] as usize == n + 1),
        };
        let colour = fn_colour(k);
        let (cap_fg, cap_bg) = if on { (Tok::Ground, colour) } else { (Tok::Ecru, Tok::Tile) };
        g.put(cx, cy, Cell::text(char_of(n as u8), cap_fg, cap_bg));
        let room = col.saturating_sub(3);
        let label: String = label.chars().take(room).collect();
        g.text(cx + 2, cy, &label, if on { colour } else { Tok::Ecru }, Tok::Display);
    }
}

fn footer(g: &mut Grid, s: &Scene, y: usize, pocket: bool) {
    let w = g.w;
    g.fill(0, y, w, 1, Tok::Ground);
    if let Overlay::Command(t) = &s.host.overlay {
        let x = g.text(1, y, ":", Tok::Ecru, Tok::Ground);
        let x = g.text(x, y, t, Tok::Ecru, Tok::Ground);
        g.put(x, y, Cell::text(' ', Tok::Ground, Tok::Ecru));
        return;
    }
    if let Some(m) = s.message {
        g.text(1, y, m, Tok::Ecru, Tok::Ground);
        return;
    }
    let v = s.view;
    let write = v[PI_VIEW_FLAGS] & PI_VIEW_FLAG_WRITE != 0;
    let playing = v[PI_VIEW_FLAGS] & PI_VIEW_FLAG_PLAYING != 0;
    let tap = s.host.mode == Mode::Tap;
    let hints: Vec<(&str, &str)> = if pocket {
        vec![("?", "keys"), (":", "command"), ("esc", "let go"), ("^q", "quit")]
    } else if shown_fn(v, s.host.latched_mask()).is_some() {
        vec![("1…v", "choose"), (if tap { "tap" } else { "shift+key" }, "latch"), ("esc", "let go"), ("?", "keys")]
    } else if write {
        vec![("1…v", "step"), ("7+key", "sound"), ("t g", "A"), ("y h", "B"), ("space", if playing { "stop" } else { "play" }), ("?", "keys"), (":", "command"), ("^q", "quit")]
    } else {
        vec![("1…v", "play"), ("7+key", "sound"), ("p", "write"), ("i+key", "effect"), ("space", if playing { "stop" } else { "play" }), ("?", "keys"), (":", "command")]
    };
    let mut x = 1;
    let room = if tap && !pocket { w - 5 } else { w };
    for (k, what) in hints {
        if x + k.chars().count() + what.len() + 1 >= room {
            break;
        }
        x = g.text(x, y, k, Tok::Ecru, Tok::Ground) + 1;
        x = g.text(x, y, what, Tok::Lilac, Tok::Ground) + 2;
    }
    if tap && !pocket {
        g.text(w - 4, y, "tap", Tok::Lilac, Tok::Ground);
    }
}

/// A slider in a column: a thread, the cap and the second mark (bytes 60 to 63).
fn slider(g: &mut Grid, x: usize, y: usize, rows: usize, width: usize, cap: u16, hit: u16, name: char) {
    let row_of = |v: u16| y + (rows - 1) - (v as usize * (rows - 1) + 32767) / 65535;
    for r in 0..rows {
        for i in 0..width {
            g.put(x + i, y + r, Cell::blank(Tok::Ground));
        }
        g.put(x + width / 2, y + r, Cell::text('│', Tok::Thread, Tok::Ground));
    }
    let hr = row_of(hit);
    for i in 0..width {
        g.put(x + i, hr, Cell::text('─', Tok::Madder, Tok::Ground));
    }
    let cr = row_of(cap);
    for i in 0..width {
        g.put(x + i, cr, Cell::text(' ', Tok::Ground, Tok::Ecru));
    }
    if width >= 3 {
        g.put(x + width / 2, cr, Cell::text(name, Tok::Ground, Tok::Ecru));
    }
}

fn helper(g: &mut Grid, y0: usize, h: usize) {
    let w = g.w;
    g.fill(0, y0, w, h, Tok::Display);
    let lines: &[(&str, &str)] = &[
        ("the keys", ""),
        ("1234 qwer asdf zxcv", "the grid, keys 1 to 16"),
        ("7 8 9", "sound, pattern, bpm"),
        ("u i o p", "function, fx, play, write"),
        ("two keys at once", ""),
        ("hold one, press the other", "or shift and a function key to latch it"),
        ("fx stays latched", "until shifted again; the others let go after the next key"),
        ("no key-up here?", "a tap latches; with fx latched a tap starts, stops, switches"),
        ("one key each", ""),
        ("t g / y h", "slider A / B up, down"),
        ("space  , .  < >", "play; tempo; swing"),
        ("- =  m", "volume; the sounds sheet"),
        ("esc  :  ^q", "let every latch go; a command (w e q); quit"),
    ];
    let mut y = y0;
    for (a, b) in lines {
        if y >= y0 + h {
            break;
        }
        if b.is_empty() {
            g.text(1, y, a, Tok::Ochre, Tok::Display);
        } else {
            let x = g.text(2, y, a, Tok::Ecru, Tok::Display);
            let x = x.max(if w >= 80 { 28 } else { x + 1 });
            let room = w.saturating_sub(x + 1);
            let b: String = b.chars().take(room).collect();
            g.text(x, y, &b, Tok::Lilac, Tok::Display);
        }
        y += 1;
    }
    g.text(w.saturating_sub(16), y0, "esc or ? closes", Tok::Lilac, Tok::Display);
}

fn sheet(g: &mut Grid, s: &Scene, y0: usize, h: usize) {
    let w = g.w;
    let v = s.view;
    g.fill(0, y0, w, h, Tok::Display);
    g.text(1, y0, "the sounds", Tok::Madder, Tok::Display);
    let help = "key: code or sample   shift+key: through fx   esc closes";
    if w >= 80 {
        g.text(14, y0, help, Tok::Lilac, Tok::Display);
    }
    let sampled = u16_at(v, PI_VIEW_SAMPLED);
    let fx = u16_at(v, PI_VIEW_FX_SOUNDS);
    let cols = if w >= 80 { 2 } else { 1 };
    let per = 16 / cols;
    for n in 0..16usize {
        let (c, r) = (n / per, n % per);
        let y = y0 + 1 + r;
        if y >= y0 + h {
            continue;
        }
        let x = 1 + c * (w / cols);
        let fired = s.cores >> n & 1 != 0;
        g.put(x, y, Cell::text(char_of(n as u8), Tok::Ecru, Tok::Tile));
        let name: String = NAMES[n].chars().take(12).collect();
        g.text(x + 2, y, &name, if fired { Tok::White } else { Tok::Ecru }, Tok::Display);
        let (vtxt, vtok) = if sampled >> n & 1 != 0 { ("sample", Tok::Ecru) } else { ("code", Tok::Lilac) };
        g.text(x + 15, y, vtxt, vtok, Tok::Display);
        if fx >> n & 1 != 0 {
            g.text(x + 23, y, "through fx", Tok::Ground, Tok::Fern);
        } else {
            g.text(x + 23, y, "dry", Tok::Lilac, Tok::Display);
        }
    }
}

/// The main screen, 80 by 24 (D117 item 1; `Main`).
pub fn main_screen(s: &Scene) -> Grid {
    let mut g = Grid::new(80, 24, Tok::Ground);
    let v = s.view;
    let latched = s.host.latched_mask();
    status(&mut g, s, false);
    match s.host.overlay {
        Overlay::Helper => helper(&mut g, 1, 22),
        Overlay::Sheet => sheet(&mut g, s, 1, 22),
        _ => {
            g.fill(0, 1, 80, 10, Tok::Ground);
            g.fill(1, 1, 78, 10, Tok::Display);
            match shown_fn(v, latched) {
                Some(k) => meanings(&mut g, s, k, 1, 1, 78, 10),
                None => {
                    weave(&mut g, s);
                    marks(&mut g, s, Some(9), 10, 1, 3, 2);
                }
            }
            for n in 0..16 {
                let k = key_lights(v, s.cores, latched, n);
                draw_key(&mut g, 2 + (n % 4) * 7, 11 + (n / 4) * 3, &k, Tok::Ground);
            }
            g.text(31, 11, "t", Tok::Lilac, Tok::Ground);
            g.text(35, 11, "y", Tok::Lilac, Tok::Ground);
            slider(&mut g, 30, 12, 10, 3, s.host.cap(0), u16_at(v, PI_VIEW_HIT_A), 'A');
            slider(&mut g, 34, 12, 10, 3, s.host.cap(1), u16_at(v, PI_VIEW_HIT_B), 'B');
            g.text(31, 22, "g", Tok::Lilac, Tok::Ground);
            g.text(35, 22, "h", Tok::Lilac, Tok::Ground);
            for (i, k) in [PI_KEY_SOUND, PI_KEY_PATTERN, PI_KEY_BPM].into_iter().enumerate() {
                draw_fn_key(&mut g, 39 + i * 14, 11, 11, k, fn_state(v, latched, k));
            }
            for (i, k) in [PI_KEY_FUNCTION, PI_KEY_FX, PI_KEY_PLAY, PI_KEY_WRITE].into_iter().enumerate() {
                draw_fn_key(&mut g, 39 + i * 10, 17, 9, k, fn_state(v, latched, k));
            }
        }
    }
    footer(&mut g, s, 23, false);
    g
}

/// The pocket size, 48 by 10 (D117 item 3; `Pocket`), with every light (D123 item 6).
pub fn pocket_screen(s: &Scene) -> Grid {
    let mut g = Grid::new(48, 10, Tok::Ground);
    let v = s.view;
    let latched = s.host.latched_mask();
    status(&mut g, s, true);
    match s.host.overlay {
        Overlay::Helper => helper(&mut g, 1, 8),
        Overlay::Sheet => sheet(&mut g, s, 1, 8),
        _ => {
            g.fill(0, 1, 48, 1, Tok::Display);
            marks(&mut g, s, None, 1, 1, 1, 2);
            match shown_fn(v, latched) {
                Some(k) if k == PI_KEY_SOUND || k == PI_KEY_FX || k == PI_KEY_BPM => {
                    // The pocket size has no display rows: the chosen name goes in the marks' row.
                    let name = match k {
                        PI_KEY_SOUND => NAMES[v[PI_VIEW_SEL] as usize].to_string(),
                        PI_KEY_BPM => format!("volume {}", v[PI_VIEW_VOLUME]),
                        _ if v[PI_VIEW_HEARD] != 0 => FX_NAMES[v[PI_VIEW_HEARD] as usize - 1].to_string(),
                        _ => "fx: hold a key".into(),
                    };
                    g.fill(0, 1, 48, 1, Tok::Display);
                    g.text(1, 1, &name, fn_colour(k), Tok::Display);
                }
                _ => {}
            }
            for n in 0..16 {
                let k = key_lights(v, s.cores, latched, n);
                draw_pocket_key(&mut g, 1 + (n % 4) * 7, 2 + (n / 4) * 2, &k);
            }
            slider(&mut g, 28, 2, 7, 1, s.host.cap(0), u16_at(v, PI_VIEW_HIT_A), 'A');
            slider(&mut g, 30, 2, 7, 1, s.host.cap(1), u16_at(v, PI_VIEW_HIT_B), 'B');
            for (i, k) in (PI_KEY_SOUND..=PI_KEY_WRITE).enumerate() {
                draw_pocket_fn(&mut g, 33, 2 + i, 14, k, fn_state(v, latched, k));
            }
        }
    }
    footer(&mut g, s, 9, true);
    g
}

/// Too small for either: one line, and the sound goes on.
pub fn tiny_screen(w: usize, h: usize) -> Grid {
    let mut g = Grid::new(w.max(1), h.max(1), Tok::Ground);
    g.text(0, 0, "rhythm needs 48 by 10", Tok::Ecru, Tok::Ground);
    g
}

/// Writes a grid to the terminal, sending only the cells that differ from `old`.
pub struct Painter {
    pub mono: bool,
    old: Option<Grid>,
}

impl Painter {
    pub fn new(mono: bool) -> Painter {
        Painter { mono, old: None }
    }

    pub fn invalidate(&mut self) {
        self.old = None;
    }

    pub fn paint(&mut self, g: Grid) -> String {
        let mut out = String::new();
        let full = self.old.as_ref().is_none_or(|o| o.w != g.w || o.h != g.h);
        if full {
            out.push_str("\x1b[0m\x1b[2J");
        }
        let mut style: Option<String> = None;
        for y in 0..g.h {
            let mut cursor_ok = false;
            for x in 0..g.w {
                let c = g.at(x, y);
                if !full && self.old.as_ref().is_some_and(|o| o.at(x, y) == c) {
                    cursor_ok = false;
                    continue;
                }
                if !cursor_ok {
                    let _ = write!(out, "\x1b[{};{}H", y + 1, x + 1);
                    cursor_ok = true;
                }
                let (ch, sgr) = if self.mono { mono_sgr(&c) } else { colour_sgr(&c) };
                if style.as_deref() != Some(sgr.as_str()) {
                    out.push_str(&sgr);
                    style = Some(sgr);
                }
                out.push(ch);
            }
        }
        out.push_str("\x1b[0m");
        self.old = Some(g);
        out
    }
}

fn colour_sgr(c: &Cell) -> (char, String) {
    let rgb = |t: Tok, base: u8| {
        let (r, g, b) = t.rgb();
        format!("{base};2;{r};{g};{b}")
    };
    match *c {
        Cell::Text { ch, fg, bg, under } => {
            let u = match under {
                Some(t) => {
                    let (r, g, b) = t.rgb();
                    format!(";4;58;2;{r};{g};{b}")
                }
                None => String::new(),
            };
            (ch, format!("\x1b[0;1;{};{}{}m", rgb(fg, 38), rgb(bg, 48), u))
        }
        Cell::Half { top, bottom } => ('▀', format!("\x1b[0;{};{}m", rgb(top, 38), rgb(bottom, 48))),
        Cell::Side { left, right } => ('▌', format!("\x1b[0;{};{}m", rgb(left, 38), rgb(right, 48))),
    }
}

fn mono_sgr(c: &Cell) -> (char, String) {
    let (ch, a) = c.mono();
    let mut s = String::from("\x1b[0");
    if a.bold {
        s.push_str(";1");
    }
    if a.underline {
        s.push_str(";4");
    }
    if a.reverse {
        s.push_str(";7");
    }
    s.push('m');
    (ch, s)
}

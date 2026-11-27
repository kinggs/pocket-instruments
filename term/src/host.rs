//! What a press does in the terminal (docs/TERMINAL.md, "Hold or tap" and "The map").
//!
//! Hold is the page's logic line for line (`web/rhythm.html`: `press`, `release`, `latch`,
//! `unlatch`, `letGo`, `shortcut`). Tap is D117 item 4 and D119 item 5. The host only says
//! what a finger did (`KEY_DOWN`, `KEY_UP`, `PARAM`, and the page's own edits for the one-key
//! shortcuts); the core decides the rest. Patterns (D135; `docs/WORKFLOW.md`, "Patterns, on every
//! way of playing") add three host rules and nothing else: Shift on a grid key keeps a latched
//! pattern latched; Shift latches write; in tap mode a tap on `p` while pattern is latched holds
//! write down until the next grid key ends. Choose, chain, copy and clear are the core's (§21).
//!
//! SPDX-License-Identifier: MIT
use crate::ffi::*;
use crate::input::{Key, KeyEvent, Kind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Key-up is real: a key is down from its press to its release.
    Hold,
    /// No key-up (or none to trust): a tap latches, D119 item 5 for fx.
    Tap,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Overlay {
    None,
    Helper,
    Sheet,
    Command(String),
}

/// Something for the program around the host to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// `:q`: leave the instrument (back to the menu when the menu started it; D125 item 3).
    Quit,
    /// Ctrl+Q (or Ctrl+C): leave the program, from anywhere (D125 item 3).
    QuitAll,
    Save(Option<String>),
    Open(String),
    Say(String),
}

/// Who holds a key down, as the page's `holders` sets: the keyboard, or a latch.
const KBD: u8 = 1;
const LATCH: u8 = 2;

/// The grid on `1234 / qwer / asdf / zxcv`, then sound, pattern, bpm on `7 8 9`, then
/// function, fx, play, write on `u i o p` (D28, D116 item 1).
pub const KEYMAP: &str = "1234qwerasdfzxcv789uiop";

pub fn key_of(c: char) -> Option<u8> {
    KEYMAP.find(c).map(|i| i as u8)
}

pub fn char_of(k: u8) -> char {
    KEYMAP.as_bytes()[k as usize] as char
}

/// The five keys Shift latches (D113), and a tap latches in tap mode.
pub fn latching(k: u8) -> bool {
    matches!(k, PI_KEY_SOUND | PI_KEY_PATTERN | PI_KEY_BPM | PI_KEY_FUNCTION | PI_KEY_FX)
}

/// The keys Shift latches since D135 item 2: the five, and write (a modifier for copy). Its
/// latch still flips write mode on the press, as write always does; a copy puts it back (§21).
pub fn shift_latching(k: u8) -> bool {
    latching(k) || k == PI_KEY_WRITE
}

/// One press of a shortcut: D122 item 2's defaults, tuned in use by D126. A slider moves a
/// 64th of its travel; a held key repeats at the terminal's rate for a long move.
pub const SLIDER_STEP: i32 = 1024;
pub const TEMPO_STEP: i32 = 1000;
pub const TEMPO_MIN: i32 = 60_000;
pub const TEMPO_MAX: i32 = 240_000;

pub struct Host {
    pub mode: Mode,
    holders: [u8; PI_KEYS],
    latched: u32,
    /// Grid keys that went down with Shift: when such a press ends, a latched pattern stays
    /// latched (D135 item 1), so a chain can be built one key at a time.
    shifted: u16,
    /// Where the hand left each slider (§20): None until it moves one.
    pub hand: [Option<u16>; 2],
    pub view: [u8; PI_VIEW_SIZE],
    pub overlay: Overlay,
    out: Vec<Cmd>,
    actions: Vec<Action>,
}

impl Host {
    pub fn new(mode: Mode) -> Host {
        Host {
            mode,
            holders: [0; PI_KEYS],
            latched: 0,
            shifted: 0,
            hand: [None, None],
            view: [0; PI_VIEW_SIZE],
            overlay: Overlay::None,
            out: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// The commands made since last asked, in order.
    pub fn take_cmds(&mut self) -> Vec<Cmd> {
        std::mem::take(&mut self.out)
    }

    pub fn take_actions(&mut self) -> Vec<Action> {
        std::mem::take(&mut self.actions)
    }

    pub fn is_latched(&self, k: u8) -> bool {
        self.latched >> k & 1 != 0
    }

    pub fn latched_mask(&self) -> u32 {
        self.latched
    }

    /// Keys this host holds down, by finger or latch.
    pub fn down_mask(&self) -> u32 {
        (0..PI_KEYS).filter(|&k| self.holders[k] != 0).fold(0, |m, k| m | 1 << k)
    }

    fn send(&mut self, c: Cmd) {
        self.out.push(c);
    }

    fn grid_down(&self) -> bool {
        self.holders[..16].iter().any(|&h| h != 0)
    }

    fn press(&mut self, k: u8, who: u8) {
        if self.holders[k as usize] & who != 0 {
            return;
        }
        // D78: while fx is latched it goes down before a grid key, so each press punches.
        if k < 16 && self.is_latched(PI_KEY_FX) {
            self.press(PI_KEY_FX, LATCH);
        }
        let was = self.holders[k as usize];
        self.holders[k as usize] |= who;
        if was == 0 {
            self.send(Cmd::new(PI_CMD_KEY_DOWN, k, 0, 0));
        }
    }

    fn release(&mut self, k: u8, who: u8) {
        if self.holders[k as usize] & who == 0 {
            return;
        }
        self.holders[k as usize] &= !who;
        if self.holders[k as usize] == 0 {
            self.send(Cmd::new(PI_CMD_KEY_UP, k, 0, 0));
        }
        if k < 16 && self.is_latched(PI_KEY_FX) && !self.grid_down() {
            self.release(PI_KEY_FX, LATCH);
        }
        // D60: any other press that ends finishes the gesture, and every latched key lets go.
        // But fx: its latch stays until it is latched again (D78, D113). And pattern, when the
        // grid key that ends went down with Shift (D135 item 1).
        if who != LATCH {
            let keep = k < 16 && self.shifted >> k & 1 != 0;
            if k < 16 {
                self.shifted &= !(1 << k);
            }
            for j in 0..PI_KEYS as u8 {
                if j != PI_KEY_FX && self.is_latched(j) && !(keep && j == PI_KEY_PATTERN) {
                    self.unlatch(j);
                }
            }
        }
    }

    fn latch(&mut self, k: u8) {
        self.latched |= 1 << k;
        if k != PI_KEY_FX || self.grid_down() {
            self.press(k, LATCH);
        }
    }

    fn unlatch(&mut self, k: u8) {
        self.latched &= !(1 << k);
        self.release(k, LATCH);
    }

    fn toggle_latch(&mut self, k: u8) {
        if self.is_latched(k) { self.unlatch(k) } else { self.latch(k) }
    }

    /// esc: every latch lets go, fx's too (D119 item 3). In tap mode the keyboard holds an
    /// effect's key down (D119 item 5), so that goes too.
    pub fn let_go(&mut self) {
        // The effect's key first, so fx comes up after it, as when a finger lifts.
        if self.mode == Mode::Tap {
            self.release_all(KBD);
        }
        for j in 0..PI_KEYS as u8 {
            if self.is_latched(j) {
                self.unlatch(j);
            }
        }
    }

    /// A key whose release this host will never see (the terminal lost focus).
    pub fn release_all(&mut self, who: u8) {
        for k in 0..PI_KEYS as u8 {
            self.release(k, who);
        }
    }

    /// Every key the keyboard holds goes up (at quit).
    pub fn release_all_keys(&mut self) {
        self.release_all(KBD);
    }

    pub fn focus_lost(&mut self) {
        if self.mode == Mode::Hold {
            self.release_all(KBD);
        }
    }

    fn held_in_view(&self, k: u8) -> bool {
        u32_at(&self.view, PI_VIEW_HELD) >> k & 1 != 0
    }

    /// A slider's cap: the view's value while bpm is held (swing and tempo) or before the hand
    /// moved it; otherwise where the hand left it (the page's `capOf`).
    pub fn cap(&self, which: usize) -> u16 {
        let shown = u16_at(&self.view, if which == 0 { PI_VIEW_A } else { PI_VIEW_B });
        if self.held_in_view(PI_KEY_BPM) { shown } else { self.hand[which].unwrap_or(shown) }
    }

    /// A slider set to `value`, as a drag would (the page's `slide`). The fixtures use it for
    /// `param`; the keys for `t g y h`.
    pub fn slide(&mut self, which: usize, value: i32) {
        let v = value.clamp(0, 65535) as u16;
        if self.holders[PI_KEY_BPM as usize] == 0 {
            self.hand[which] = Some(v);
        }
        self.send(Cmd::new(PI_CMD_PARAM, which as u8, 0, v as u32));
    }

    fn nudge(&mut self, which: usize, d: i32) {
        let from = self.cap(which) as i32;
        self.slide(which, from + d);
    }

    /// With bpm held a slider is the swing (A) or the tempo (B), and one press is one step of
    /// either (D126): the slider value that reads back as the next whole step (`ENGINE.md` §13,
    /// `PARAM` with bpm held). It stays a slider move, so bpm counts as used and letting it go
    /// keeps the tempo instead of stepping to a preset.
    fn bpm_slide(&mut self, which: usize, d: i32) {
        let v = if which == 0 {
            let s = (self.view[PI_VIEW_SWING] as i32 + d).clamp(0, PI_SWING_MAX as i32);
            (s * 65535 + 24) / 48
        } else {
            let bpm = (u32_at(&self.view, PI_VIEW_TEMPO) / 1000) as i32;
            let b = (bpm + d).clamp(TEMPO_MIN / 1000, TEMPO_MAX / 1000);
            ((b - 60) * 65535 + 90) / 180
        };
        self.slide(which, v);
    }

    /// A slider key: one step of the slider, or with bpm held one of the swing or tempo.
    fn slider_key(&mut self, which: usize, up: bool) {
        let d = if up { 1 } else { -1 };
        if self.held_in_view(PI_KEY_BPM) {
            self.bpm_slide(which, d);
        } else {
            self.nudge(which, d * SLIDER_STEP);
        }
    }

    fn tempo(&mut self, d: i32) {
        let t = u32_at(&self.view, PI_VIEW_TEMPO) as i32;
        self.send(Cmd::new(PI_CMD_TEMPO, 0, 0, (t + d).clamp(TEMPO_MIN, TEMPO_MAX) as u32));
    }

    fn swing(&mut self, d: i32) {
        let s = self.view[PI_VIEW_SWING] as i32;
        self.send(Cmd::new(PI_CMD_SWING, 0, 0, (s + d).clamp(0, PI_SWING_MAX as i32) as u32));
    }

    /// The volume has no edit of its own: bpm held and a key (§13), sent as one gesture. Not
    /// while bpm, or the key it would press, is down (the page's and Rack's rule).
    fn volume(&mut self, d: i32) {
        let n = self.view[PI_VIEW_VOLUME] as i32 + d;
        if !(1..=16).contains(&n) || self.holders[PI_KEY_BPM as usize] != 0 || self.holders[(n - 1) as usize] != 0 {
            return;
        }
        let k = (n - 1) as u8;
        self.send(Cmd::new(PI_CMD_KEY_DOWN, PI_KEY_BPM, 0, 0));
        self.send(Cmd::new(PI_CMD_KEY_DOWN, k, 0, 0));
        self.send(Cmd::new(PI_CMD_KEY_UP, k, 0, 0));
        self.send(Cmd::new(PI_CMD_KEY_UP, PI_KEY_BPM, 0, 0));
    }

    /// One key event from the terminal.
    pub fn key(&mut self, e: KeyEvent) {
        if e.ctrl {
            if e.kind != Kind::Release && matches!(e.key, Key::Char('q') | Key::Char('c')) {
                self.actions.push(Action::QuitAll);
            }
            return;
        }
        if e.alt {
            return;
        }
        match self.overlay.clone() {
            Overlay::Command(text) => return self.command_key(e, text),
            Overlay::Helper => {
                if e.kind == Kind::Press && (e.key == Key::Esc || (e.key == Key::Char('/') && e.shift)) {
                    self.overlay = Overlay::None;
                }
                return;
            }
            Overlay::Sheet => return self.sheet_key(e),
            Overlay::None => {}
        }
        if self.shortcut(e) {
            return;
        }
        let Key::Char(c) = e.key else { return };
        let Some(k) = key_of(c) else { return };
        match self.mode {
            Mode::Hold => match e.kind {
                Kind::Repeat => {}
                Kind::Press => {
                    // D113: Shift and a key latches the five; D135 item 2: and write.
                    if e.shift && shift_latching(k) {
                        self.toggle_latch(k);
                    } else {
                        if e.shift && k < 16 {
                            self.shifted |= 1 << k;
                        }
                        self.press(k, KBD);
                    }
                }
                Kind::Release => self.release(k, KBD),
            },
            Mode::Tap => {
                if e.kind != Kind::Press {
                    return; // herdr's releases are not real (R80); repeats are not presses
                }
                self.tap(k, e.shift);
            }
        }
    }

    /// Tap mode (D117 item 4, D119 item 5; D135 for patterns).
    fn tap(&mut self, k: u8, shift: bool) {
        if latching(k) {
            return self.toggle_latch(k);
        }
        // D135 item 3: with pattern latched a tap on `p` holds write down until the next grid
        // key ends (a latch of write). Shift latches it too, as in hold (D135 item 2). A tap on
        // `p` while write is so held lets it go, as a second tap lets any latch go (⚠ ours: the
        // design is silent on a second tap).
        if k == PI_KEY_WRITE && (shift || self.is_latched(PI_KEY_PATTERN) || self.is_latched(PI_KEY_WRITE)) {
            return self.toggle_latch(k);
        }
        if k >= 16 {
            // play, write: they act on the press, as everywhere.
            self.press(k, KBD);
            return self.release(k, KBD);
        }
        if self.is_latched(PI_KEY_FX) {
            if self.holders[k as usize] & KBD != 0 {
                // A second tap stops that effect.
                return self.release(k, KBD);
            }
            // A tap starts it; on another key it switches: the new key first, so fx stays down.
            let others: Vec<u8> = (0..16u8).filter(|&j| j != k && self.holders[j as usize] & KBD != 0).collect();
            self.press(k, KBD);
            for j in others {
                self.release(j, KBD);
            }
            return;
        }
        // D135 item 1: Shift on the key keeps a latched pattern latched (a chain).
        if shift {
            self.shifted |= 1 << k;
        }
        self.press(k, KBD);
        self.release(k, KBD);
    }

    /// The one-key shortcuts (D119 item 3, D122 item 2). True if the key was one.
    fn shortcut(&mut self, e: KeyEvent) -> bool {
        let Key::Char(c) = e.key else {
            if e.key == Key::Esc {
                if e.kind == Kind::Press {
                    self.let_go();
                }
                return true;
            }
            return false;
        };
        let again = e.kind != Kind::Release; // a held key repeats, as on the page and in Rack
        let press = e.kind == Kind::Press;
        match (c, e.shift) {
            ('t', _) => again.then(|| self.slider_key(0, true)),
            ('g', _) => again.then(|| self.slider_key(0, false)),
            ('y', _) => again.then(|| self.slider_key(1, true)),
            ('h', _) => again.then(|| self.slider_key(1, false)),
            (' ', _) => press.then(|| {
                self.press(PI_KEY_PLAY, KBD);
                self.release(PI_KEY_PLAY, KBD);
            }),
            (',', false) => again.then(|| self.tempo(-TEMPO_STEP)),
            ('.', false) => again.then(|| self.tempo(TEMPO_STEP)),
            (',', true) => again.then(|| self.swing(-1)),
            ('.', true) => again.then(|| self.swing(1)),
            ('-', _) => again.then(|| self.volume(-1)),
            ('=', _) => again.then(|| self.volume(1)),
            ('m', _) => press.then(|| {
                self.release_all(KBD);
                self.overlay = Overlay::Sheet;
            }),
            ('/', true) => press.then(|| {
                self.release_all(KBD);
                self.overlay = Overlay::Helper;
            }),
            (';', true) => press.then(|| {
                self.release_all(KBD);
                self.overlay = Overlay::Command(String::new());
            }),
            _ => return false,
        };
        true
    }

    /// The sounds sheet: a grid key switches its sound between code and sample and plays it;
    /// Shift and the key switches whether it goes through the effects (the page's sheet).
    fn sheet_key(&mut self, e: KeyEvent) {
        if e.kind != Kind::Press {
            return;
        }
        match e.key {
            Key::Esc => self.overlay = Overlay::None,
            Key::Char('m') => self.overlay = Overlay::None,
            Key::Char(c) => {
                if let Some(t) = key_of(c).filter(|&t| t < 16) {
                    if e.shift {
                        let on = u16_at(&self.view, PI_VIEW_FX_SOUNDS) >> t & 1 != 0;
                        self.send(Cmd::new(PI_CMD_FXSOUND, t, 0, if on { 0 } else { 1 }));
                    } else {
                        let on = u16_at(&self.view, PI_VIEW_SAMPLED) >> t & 1 != 0;
                        self.send(Cmd::new(PI_CMD_VOICE, t, 0, if on { 0 } else { 1 }));
                        self.send(Cmd::new(PI_CMD_FIRE, t, 0, 0));
                    }
                }
            }
            _ => {}
        }
    }

    /// The command line (D116 item 2, proposed ⚠): `w [file]`, `e <file>`, `q`.
    fn command_key(&mut self, e: KeyEvent, mut text: String) {
        if e.kind == Kind::Release {
            return;
        }
        match e.key {
            Key::Esc => self.overlay = Overlay::None,
            Key::Enter => {
                self.overlay = Overlay::None;
                let t = text.trim();
                let (cmd, arg) = t.split_once(' ').map(|(a, b)| (a, Some(b.trim().to_string()))).unwrap_or((t, None));
                match cmd {
                    "q" | "quit" => self.actions.push(Action::Quit),
                    "w" | "save" => self.actions.push(Action::Save(arg)),
                    "wq" | "x" => {
                        self.actions.push(Action::Save(arg));
                        self.actions.push(Action::Quit);
                    }
                    "e" | "open" => match arg {
                        Some(f) => self.actions.push(Action::Open(f)),
                        None => self.actions.push(Action::Say("open what? :e <file>".into())),
                    },
                    "" => {}
                    other => self.actions.push(Action::Say(format!("not a command: {other} (w, e, q)"))),
                }
            }
            Key::Backspace => {
                text.pop();
                self.overlay = Overlay::Command(text);
            }
            _ => {
                if let Some(c) = e.text.filter(|c| !c.is_control()) {
                    text.push(c);
                    self.overlay = Overlay::Command(text);
                }
            }
        }
    }
}

//! What the terminal sends, read as keys (docs/TERMINAL.md, "Reading them"; D123 item 3).
//!
//! With the kitty keyboard protocol a key arrives as `CSI code[:shifted[:base]] ; 1+mods[:type] u`.
//! A key is its place: the base layout key when the terminal sends one, else the unshifted
//! code, as the page reads `e.code`. Shift stays a modifier (crossterm drops it when the
//! shifted character is sent: R78). Without the protocol each byte is a tap.
//!
//! SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Press,
    Repeat,
    Release,
}

/// A key by its place on a US layout: a lowercase letter, a digit, or the unshifted symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Esc,
    Enter,
    Backspace,
    Tab,
    /// A modifier key on its own (Shift, Ctrl, ...), or anything this host does not read.
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub kind: Kind,
    /// The character the key makes, as typed (for the command line).
    pub text: Option<char>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Key(KeyEvent),
    /// The reply to `CSI ? u`: the protocol is on, with these flags.
    Flags(u32),
    /// The reply to `CSI c` (primary device attributes): every query before it is answered.
    DeviceAttributes,
    FocusIn,
    FocusOut,
}

/// What the host pushes: disambiguate, event types, alternate keys, every key as an escape.
pub const KITTY_FLAGS: u32 = 1 | 2 | 4 | 8;

/// Shifted symbols on a US layout, and the key each is on (used without the protocol). ⚠ On
/// another layout these characters come from other keys.
const US_SHIFTED: [(char, char); 21] = [
    ('!', '1'), ('@', '2'), ('#', '3'), ('$', '4'), ('%', '5'), ('^', '6'), ('&', '7'), ('*', '8'), ('(', '9'),
    (')', '0'), ('_', '-'), ('+', '='), ('<', ','), ('>', '.'), ('?', '/'), (':', ';'), ('"', '\''), ('{', '['),
    ('}', ']'), ('|', '\\'), ('~', '`'),
];

fn plain(c: char) -> KeyEvent {
    let (key, shift) = if c.is_ascii_uppercase() {
        (c.to_ascii_lowercase(), true)
    } else if let Some(&(_, k)) = US_SHIFTED.iter().find(|(s, _)| *s == c) {
        (k, true)
    } else {
        (c, false)
    };
    KeyEvent { key: Key::Char(key), shift, ctrl: false, alt: false, kind: Kind::Press, text: Some(c) }
}

fn special(key: Key) -> KeyEvent {
    KeyEvent { key, shift: false, ctrl: false, alt: false, kind: Kind::Press, text: None }
}

/// Reads bytes as they come, keeping an unfinished escape sequence for the next read.
#[derive(Default)]
pub struct Reader {
    pending: Vec<u8>,
}

impl Reader {
    pub fn new() -> Reader {
        Reader::default()
    }

    /// Everything complete in what has come so far. A lone ESC at the end of a read is esc:
    /// a terminal sends a sequence in one write, so its rest never comes later (`flush_esc`
    /// is for a caller that waits first).
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Input> {
        self.pending.extend_from_slice(bytes);
        let mut out = Vec::new();
        let mut i = 0;
        let b = std::mem::take(&mut self.pending);
        while i < b.len() {
            match b[i] {
                0x1b => {
                    if i + 1 >= b.len() {
                        out.push(Input::Key(special(Key::Esc)));
                        i += 1;
                    } else if b[i + 1] == b'[' {
                        // CSI: parameters and intermediates, then one final byte in 0x40..=0x7e.
                        let mut j = i + 2;
                        while j < b.len() && !(0x40..=0x7e).contains(&b[j]) {
                            j += 1;
                        }
                        if j >= b.len() {
                            self.pending = b[i..].to_vec();
                            return out;
                        }
                        if let Some(e) = csi(&b[i + 2..j], b[j]) {
                            out.push(e);
                        }
                        i = j + 1;
                    } else if b[i + 1] == b'O' {
                        // SS3 (F1 to F4, keypad in application mode): three bytes, not read.
                        if i + 2 >= b.len() {
                            self.pending = b[i..].to_vec();
                            return out;
                        }
                        i += 3;
                    } else if b[i + 1] == 0x1b {
                        out.push(Input::Key(special(Key::Esc)));
                        i += 1;
                    } else {
                        // Alt and a key: the key, marked.
                        let mut e = self.one(b[i + 1]);
                        if let Some(Input::Key(k)) = e.as_mut() {
                            k.alt = true;
                        }
                        out.extend(e);
                        i += 2;
                    }
                }
                c => {
                    // A UTF-8 character past ASCII: skipped whole (no key of ours makes one).
                    if c >= 0x80 {
                        let len = if c >= 0xf0 { 4 } else if c >= 0xe0 { 3 } else if c >= 0xc0 { 2 } else { 1 };
                        if i + len > b.len() {
                            self.pending = b[i..].to_vec();
                            return out;
                        }
                        i += len;
                        continue;
                    }
                    out.extend(self.one(c));
                    i += 1;
                }
            }
        }
        out
    }

    fn one(&self, c: u8) -> Option<Input> {
        let e = match c {
            b'\r' | b'\n' => special(Key::Enter),
            0x7f | 0x08 => special(Key::Backspace),
            b'\t' => special(Key::Tab),
            0x01..=0x1a => {
                // Ctrl and a letter: 0x11 is Ctrl+Q, 0x03 Ctrl+C.
                let ch = (b'a' + c - 1) as char;
                KeyEvent { key: Key::Char(ch), shift: false, ctrl: true, alt: false, kind: Kind::Press, text: None }
            }
            0x20..=0x7e => plain(c as char),
            _ => return None,
        };
        Some(Input::Key(e))
    }
}

/// One CSI sequence: `params` between `CSI` and the final byte.
fn csi(params: &[u8], fin: u8) -> Option<Input> {
    let p = std::str::from_utf8(params).ok()?;
    match fin {
        b'u' => {
            if let Some(f) = p.strip_prefix('?') {
                return Some(Input::Flags(f.parse().unwrap_or(0)));
            }
            kitty(p)
        }
        b'c' if p.starts_with('?') => Some(Input::DeviceAttributes),
        b'I' if p.is_empty() => Some(Input::FocusIn),
        b'O' if p.is_empty() => Some(Input::FocusOut),
        // Arrows, function keys and the rest: not this host's.
        _ => None,
    }
}

fn num(s: &str) -> Option<u32> {
    if s.is_empty() { None } else { s.parse().ok() }
}

/// `code[:shifted[:base]] ; mods[:type] ; text`
fn kitty(p: &str) -> Option<Input> {
    let mut fields = p.split(';');
    let codes: Vec<&str> = fields.next().unwrap_or("").split(':').collect();
    let mods_field: Vec<&str> = fields.next().unwrap_or("").split(':').collect();
    let code = num(codes[0])?;
    let shifted = codes.get(1).and_then(|s| num(s));
    let base = codes.get(2).and_then(|s| num(s));
    let mods = mods_field.first().and_then(|s| num(s)).unwrap_or(1).saturating_sub(1);
    let kind = match mods_field.get(1).and_then(|s| num(s)).unwrap_or(1) {
        2 => Kind::Repeat,
        3 => Kind::Release,
        _ => Kind::Press,
    };
    let (shift, alt, ctrl) = (mods & 1 != 0, mods & 2 != 0, mods & 4 != 0);
    let place = base.unwrap_or(code);
    let key = match place {
        27 => Key::Esc,
        13 => Key::Enter,
        9 => Key::Tab,
        127 | 8 => Key::Backspace,
        0x20..=0x7e => Key::Char((place as u8 as char).to_ascii_lowercase()),
        _ => Key::Other,
    };
    let text = match (shifted, shift) {
        (Some(s), true) => char::from_u32(s),
        _ => {
            let c = char::from_u32(code).filter(|c| !c.is_control() && (code as usize) < 0xe000);
            if shift { c.map(|c| c.to_ascii_uppercase()) } else { c }
        }
    };
    Some(Input::Key(KeyEvent { key, shift, ctrl, alt, kind, text: if ctrl || alt { None } else { text } }))
}

/// The bytes foot sends for a key going down or up (R78), for the tests and the fixtures.
pub fn kitty_bytes(key: char, shift: bool, kind: Kind) -> Vec<u8> {
    let code = key as u32;
    let mods = 1 + if shift { 1 } else { 0 };
    let shifted = if shift {
        US_SHIFTED.iter().find(|(_, k)| *k == key).map(|(s, _)| *s as u32).or_else(|| key.is_ascii_lowercase().then(|| key.to_ascii_uppercase() as u32))
    } else {
        None
    };
    let codes = match shifted {
        Some(s) => format!("{code}:{s}"),
        None => format!("{code}"),
    };
    let t = match kind {
        Kind::Press => 1,
        Kind::Repeat => 2,
        Kind::Release => 3,
    };
    format!("\x1b[{codes};{mods}:{t}u").into_bytes()
}

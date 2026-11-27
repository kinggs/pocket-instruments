//! `docs/BEAT.md`, "The form": text into snapshot version 5. Strict on purpose: every way a
//! file can be wrong is refused with its line and column and what the line takes, before any
//! byte of the caller's snapshot is written.

use crate::layout::*;
use crate::text::{Msg, crc32, put};
use crate::{BEAT_VERSION, ERR_NEWER, ERR_ROOM, ERR_TEXT, LETTERS, NAMES};

/// What a read gives: the snapshot's length, and the keys `own` named (bit k-1 for key k).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Read {
    pub len: usize,
    pub own: u16,
}

type Refusal = (i32, Msg);

/// The lines that come once: where each was given (0: not yet).
#[derive(Clone, Copy)]
enum Word {
    Name,
    Credit,
    Tempo,
    Swing,
    Volume,
    Seed,
    Chosen,
    Sliders,
    Own,
    Chain,
}
const WORDS: usize = 10;

#[derive(Clone, Copy)]
struct Sound {
    line: u32,
    sample: Option<bool>,
    fx: Option<bool>,
    a: Option<u16>,
    b: Option<u16>,
}

struct Pat {
    block: [u8; BLOCK],
    /// A step's value letter, as its index in `LETTERS` plus one (0: none), resolved at the end
    /// because a `value` line may come after the rows that use it.
    letter: [[u8; STEPS_V5]; TRACKS_V5],
    line: u32,
    rows: u16,
    fx_line: u32,
}

struct Beat {
    given: [u32; WORDS],
    name: [u8; NAME_LEN],
    credit: [u8; CREDIT_LEN],
    tempo: u32,
    swing: u8,
    volume: u8,
    seed: u32,
    chosen: u8,
    sliders: [u16; 2],
    sounds: [Sound; TRACKS_V5],
    own: u16,
    values: [[u16; 2]; 46],
    value_line: [u32; 46],
    /// Where each letter was first used in a row: the place a missing `value` is reported.
    first_use: [(u32, u32); 46],
    chain: [u8; PATTERNS_V5],
    chain_len: u8,
    pats: [Pat; PATTERNS_V5],
    current: Option<usize>,
}

const CLEARED: [u8; BLOCK] = {
    let mut b = [0u8; BLOCK];
    let mut i = B_PROB;
    while i < B_LANE {
        b[i] = 100;
        i += 1;
    }
    b[B_LENGTH] = 16;
    b
};

impl Beat {
    fn new() -> Beat {
        const PAT: Pat = Pat { block: CLEARED, letter: [[0; STEPS_V5]; TRACKS_V5], line: 0, rows: 0, fx_line: 0 };
        const SOUND: Sound = Sound { line: 0, sample: None, fx: None, a: None, b: None };
        Beat {
            given: [0; WORDS],
            name: [0; NAME_LEN],
            credit: [0; CREDIT_LEN],
            tempo: 120_000,
            swing: 0,
            volume: 16,
            seed: 1,
            chosen: 0,
            sliders: [32768; 2],
            sounds: [SOUND; TRACKS_V5],
            own: 0,
            values: [[0; 2]; 46],
            value_line: [0; 46],
            first_use: [(0, 0); 46],
            chain: [0; PATTERNS_V5],
            chain_len: 1,
            pats: [PAT; PATTERNS_V5],
            current: None,
        }
    }
}

/// One line being read: its number, its bytes (without the line end), and where it is.
struct Line<'a> {
    no: u32,
    bytes: &'a [u8],
}

impl<'a> Line<'a> {
    /// The column of byte `at`, counted in characters from 1.
    fn col(&self, at: usize) -> u32 {
        let at = at.min(self.bytes.len());
        1 + self.bytes[..at].iter().filter(|b| **b & 0xC0 != 0x80).count() as u32
    }

    fn at(&self, at: usize) -> Msg {
        Msg::new().s(b"line ").n(self.no).s(b", column ").n(self.col(at)).s(b": ")
    }

    fn whole(&self) -> Msg {
        Msg::new().s(b"line ").n(self.no).s(b": ")
    }
}

/// A word of a line and where it starts.
#[derive(Clone, Copy)]
struct Tok<'a> {
    at: usize,
    s: &'a [u8],
}

/// Up to this many words a line; a line with more is refused as too long for any statement.
const TOKS: usize = 40;

fn split<'a>(bytes: &'a [u8], toks: &mut [Tok<'a>; TOKS]) -> Option<usize> {
    let mut n = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b' ' {
            i += 1;
        }
        if n == TOKS {
            return None;
        }
        toks[n] = Tok { at: start, s: &bytes[start..i] };
        n += 1;
    }
    Some(n)
}

/// A decimal number: digits only, no sign, within a u32.
fn num(s: &[u8]) -> Option<u32> {
    if s.is_empty() || s.len() > 10 {
        return None;
    }
    let mut v: u32 = 0;
    for &c in s {
        if !c.is_ascii_digit() {
            return None;
        }
        v = v.checked_mul(10)?.checked_add((c - b'0') as u32)?;
    }
    Some(v)
}

fn num_in(s: &[u8], lo: u32, hi: u32) -> Option<u32> {
    num(s).filter(|v| (lo..=hi).contains(v))
}

/// `tempo`: a bpm with up to three decimals, in thousandths.
fn tempo(s: &[u8]) -> Option<u32> {
    let (whole, frac) = match s.iter().position(|c| *c == b'.') {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    let mut v = num(whole)?.checked_mul(1000)?;
    if let Some(f) = frac {
        if f.is_empty() || f.len() > 3 {
            return None;
        }
        let mut scale = 100;
        for &c in f {
            if !c.is_ascii_digit() {
                return None;
            }
            v = v.checked_add((c - b'0') as u32 * scale)?;
            scale /= 10;
        }
    }
    Some(v).filter(|v| (30_000..=300_000).contains(v))
}

/// `<a>=<b>`: the two halves.
fn pair(s: &[u8], sep: u8) -> Option<(&[u8], &[u8])> {
    let i = s.iter().position(|c| *c == sep)?;
    Some((&s[..i], &s[i + 1..]))
}

fn letter_index(c: u8) -> Option<usize> {
    LETTERS.iter().position(|l| *l == c)
}

fn name_index(s: &[u8]) -> Option<usize> {
    NAMES.iter().position(|n| n.as_bytes() == s)
}

const WORD_LIST: &[u8] = b"name, credit, tempo, swing, volume, seed, chosen, sliders, sound, own, value, chain, pattern, fx, or a key";

/// Reads a beat file into `out`. On success the snapshot is `out[..len]`; on a refusal
/// nothing of `out` is written.
pub fn read(text: &[u8], out: &mut [u8]) -> Result<Read, Refusal> {
    let mut b = Beat::new();
    let mut no = 0u32;
    let mut rest = text;
    let mut header = false;
    loop {
        if rest.is_empty() && no > 0 {
            break;
        }
        let (bytes, next) = match rest.iter().position(|c| *c == b'\n') {
            Some(i) => (&rest[..i], &rest[i + 1..]),
            None => (rest, &rest[rest.len()..]),
        };
        rest = next;
        no += 1;
        let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
        let line = Line { no, bytes };
        statement(&mut b, &line, &mut header)?;
        if rest.is_empty() {
            break;
        }
    }
    if !header {
        return Err((ERR_TEXT, Msg::new().s(b"line 1: a beat file starts with 'pocket-instruments beat 1'")));
    }
    finish(&mut b, out)
}

fn statement(b: &mut Beat, line: &Line, header: &mut bool) -> Result<(), Refusal> {
    if let Err(e) = core::str::from_utf8(line.bytes) {
        return Err((ERR_TEXT, line.at(e.valid_up_to()).s(b"not UTF-8 text")));
    }
    // The first word decides whether a `#` starts a comment: not in `name` and `credit`.
    let lead = line.bytes.iter().position(|c| *c != b' ').unwrap_or(line.bytes.len());
    let first_end = line.bytes[lead..].iter().position(|c| *c == b' ').map_or(line.bytes.len(), |i| lead + i);
    let first = &line.bytes[lead..first_end];
    if *header && (first == b"name" || first == b"credit") {
        return text_line(b, line, first, first_end);
    }
    let content = match line.bytes.iter().position(|c| *c == b'#') {
        Some(i) => &line.bytes[..i],
        None => line.bytes,
    };
    if let Some(i) = content.iter().position(|c| !(*c == b' ' || (0x21..0x7F).contains(c))) {
        let what: &[u8] = if content[i] == b'\t' { b"a tab: the beat file separates words with spaces" } else { b"a character the beat file does not use outside name, credit and comments" };
        return Err((ERR_TEXT, line.at(i).s(what)));
    }
    let mut toks = [Tok { at: 0, s: &[] }; TOKS];
    let Some(n) = split(content, &mut toks) else {
        return Err((ERR_TEXT, line.whole().s(b"more words than any line takes")));
    };
    let t = &toks[..n];
    if !*header {
        return header_line(line, t).map(|_| *header = true);
    }
    let Some(w) = t.first() else {
        return Ok(()); // a blank line, or a comment
    };
    match w.s {
        b"pocket-instruments" => Err((ERR_TEXT, line.at(w.at).s(b"'pocket-instruments beat 1' comes once, on line 1"))),
        b"name" | b"credit" => Err((ERR_TEXT, line.at(w.at).s(w.s).s(b" takes text after a space, to the end of the line"))),
        b"tempo" => {
            once(b, line, w, Word::Tempo)?;
            let v = one(line, t, b"tempo takes 30 to 300, up to three decimals")?;
            b.tempo = tempo(v.s).ok_or_else(|| (ERR_TEXT, line.at(v.at).s(b"tempo takes 30 to 300, up to three decimals; found ").q(v.s)))?;
            Ok(())
        }
        b"swing" => {
            once(b, line, w, Word::Swing)?;
            b.swing = one_num(line, t, 0, 48, b"swing takes 0 to 48 (of a step's 96 ticks)")? as u8;
            Ok(())
        }
        b"volume" => {
            once(b, line, w, Word::Volume)?;
            b.volume = one_num(line, t, 1, 16, b"volume takes 1 to 16")? as u8;
            Ok(())
        }
        b"seed" => {
            once(b, line, w, Word::Seed)?;
            b.seed = one_num(line, t, 1, u32::MAX, b"seed takes 1 to 4294967295")?;
            Ok(())
        }
        b"chosen" => {
            once(b, line, w, Word::Chosen)?;
            b.chosen = (one_num(line, t, 1, 16, b"chosen takes a key, 1 to 16")? - 1) as u8;
            Ok(())
        }
        b"sliders" => {
            once(b, line, w, Word::Sliders)?;
            const TAKES: &[u8] = b"sliders takes A and B, each 0 to 65535";
            if t.len() != 3 {
                return Err((ERR_TEXT, line.whole().s(TAKES)));
            }
            for i in 0..2 {
                b.sliders[i] = num_in(t[1 + i].s, 0, 65535).ok_or_else(|| (ERR_TEXT, line.at(t[1 + i].at).s(TAKES).s(b"; found ").q(t[1 + i].s)))? as u16;
            }
            Ok(())
        }
        b"sound" => sound(b, line, t),
        b"own" => {
            once(b, line, w, Word::Own)?;
            const TAKES: &[u8] = b"own takes keys, 1 to 16, each once";
            if t.len() < 2 {
                return Err((ERR_TEXT, line.whole().s(TAKES)));
            }
            for k in &t[1..] {
                let key = num_in(k.s, 1, 16).ok_or_else(|| (ERR_TEXT, line.at(k.at).s(TAKES).s(b"; found ").q(k.s)))?;
                if b.own >> (key - 1) & 1 == 1 {
                    return Err((ERR_TEXT, line.at(k.at).s(TAKES).s(b"; key ").n(key).s(b" is named twice")));
                }
                b.own |= 1 << (key - 1);
            }
            Ok(())
        }
        b"value" => value(b, line, t),
        b"chain" => {
            once(b, line, w, Word::Chain)?;
            const TAKES: &[u8] = b"chain takes one to sixteen patterns, each 1 to 16";
            if t.len() < 2 || t.len() > 17 {
                return Err((ERR_TEXT, line.whole().s(TAKES)));
            }
            for (i, p) in t[1..].iter().enumerate() {
                b.chain[i] = (num_in(p.s, 1, 16).ok_or_else(|| (ERR_TEXT, line.at(p.at).s(TAKES).s(b"; found ").q(p.s)))? - 1) as u8;
            }
            b.chain_len = (t.len() - 1) as u8;
            Ok(())
        }
        b"pattern" => pattern(b, line, t),
        b"fx" => fx(b, line, t),
        s if s[0].is_ascii_digit() => row(b, line, t),
        s => Err((ERR_TEXT, line.at(w.at).s(b"unknown word ").q(s).s(b"; a line starts with ").s(WORD_LIST))),
    }
}

fn header_line(line: &Line, t: &[Tok]) -> Result<(), Refusal> {
    const STARTS: &[u8] = b"a beat file starts with 'pocket-instruments beat 1'";
    if t.len() != 3 || t[0].s != b"pocket-instruments" || t[1].s != b"beat" {
        return Err((ERR_TEXT, line.whole().s(STARTS)));
    }
    match num(t[2].s) {
        Some(v) if v == BEAT_VERSION => Ok(()),
        Some(v) if v > BEAT_VERSION => Err((ERR_NEWER, line.whole().s(b"this is beat ").n(v).s(b"; this build reads beat ").n(BEAT_VERSION))),
        _ => Err((ERR_TEXT, line.at(t[2].at).s(STARTS).s(b"; found beat ").q(t[2].s))),
    }
}

fn once(b: &mut Beat, line: &Line, w: &Tok, word: Word) -> Result<(), Refusal> {
    let given = &mut b.given[word as usize];
    if *given != 0 {
        return Err((ERR_TEXT, line.at(w.at).s(w.s).s(b" comes once; it was given on line ").n(*given)));
    }
    *given = line.no;
    Ok(())
}

fn one<'a>(line: &Line, t: &[Tok<'a>], takes: &[u8]) -> Result<Tok<'a>, Refusal> {
    match t {
        [_, v] => Ok(*v),
        [_] => Err((ERR_TEXT, line.whole().s(takes))),
        [_, _, extra, ..] => Err((ERR_TEXT, line.at(extra.at).s(takes).s(b"; found more: ").q(extra.s))),
        [] => Err((ERR_TEXT, line.whole().s(takes))),
    }
}

fn one_num(line: &Line, t: &[Tok], lo: u32, hi: u32, takes: &[u8]) -> Result<u32, Refusal> {
    let v = one(line, t, takes)?;
    num_in(v.s, lo, hi).ok_or_else(|| (ERR_TEXT, line.at(v.at).s(takes).s(b"; found ").q(v.s)))
}

/// `name <text>` and `credit <text>`: to the end of the line, `#` included, spaces at either
/// end dropped; cut on a character boundary to the field's bytes (32 and 128).
fn text_line(b: &mut Beat, line: &Line, word: &[u8], end: usize) -> Result<(), Refusal> {
    let is_name = word == b"name";
    if b.given[if is_name { Word::Name } else { Word::Credit } as usize] != 0 {
        let w = Tok { at: end - word.len(), s: word };
        return once(b, line, &w, if is_name { Word::Name } else { Word::Credit });
    }
    let mut start = end;
    while start < line.bytes.len() && line.bytes[start] == b' ' {
        start += 1;
    }
    let mut stop = line.bytes.len();
    while stop > start && line.bytes[stop - 1] == b' ' {
        stop -= 1;
    }
    let text = &line.bytes[start..stop];
    if text.is_empty() {
        return Err((ERR_TEXT, line.whole().s(word).s(b" takes text after a space, to the end of the line")));
    }
    if let Some(i) = text.iter().position(|c| *c < 0x20 || *c == 0x7F) {
        return Err((ERR_TEXT, line.at(start + i).s(word).s(b" takes printable text: no tabs or control characters")));
    }
    let field: &mut [u8] = if is_name { &mut b.name } else { &mut b.credit };
    let mut n = crate::text::cut(text, field.len());
    while n > 0 && text[n - 1] == b' ' {
        n -= 1; // a cut can end on a space: dropped, as at the end of the line
    }
    field[..n].copy_from_slice(&text[..n]);
    b.given[if is_name { Word::Name } else { Word::Credit } as usize] = line.no;
    Ok(())
}

/// `sound <key> [sample|code] [dry|fx] [a <A>] [b <B>]`, any of them, in that order.
fn sound(b: &mut Beat, line: &Line, t: &[Tok]) -> Result<(), Refusal> {
    const TAKES: &[u8] = b"sound takes a key, then any of: sample or code, dry or fx, a <A>, b <B>, in that order";
    let Some(k) = t.get(1) else {
        return Err((ERR_TEXT, line.whole().s(TAKES)));
    };
    let key = num_in(k.s, 1, 16).ok_or_else(|| (ERR_TEXT, line.at(k.at).s(TAKES).s(b"; found ").q(k.s)))? as usize;
    let s = &mut b.sounds[key - 1];
    if s.line != 0 {
        return Err((ERR_TEXT, line.at(t[0].at).s(b"sound ").n(key as u32).s(b" comes once; it was given on line ").n(s.line)));
    }
    s.line = line.no;
    let mut stage = 0; // 1 past sample/code, 2 past dry/fx, 3 past a, 4 past b
    let mut i = 2;
    while i < t.len() {
        let w = t[i];
        let (to, is_value) = match w.s {
            b"sample" | b"code" => (1, false),
            b"dry" | b"fx" => (2, false),
            b"a" => (3, true),
            b"b" => (4, true),
            _ => return Err((ERR_TEXT, line.at(w.at).s(TAKES).s(b"; found ").q(w.s))),
        };
        if to <= stage {
            return Err((ERR_TEXT, line.at(w.at).s(TAKES).s(b"; ").q(w.s).s(b" is out of place")));
        }
        stage = to;
        match w.s {
            b"sample" => s.sample = Some(true),
            b"code" => s.sample = Some(false),
            b"dry" => s.fx = Some(false),
            b"fx" => s.fx = Some(true),
            _ => {}
        }
        if is_value {
            let Some(v) = t.get(i + 1) else {
                return Err((ERR_TEXT, line.at(w.at).s(TAKES).s(b"; ").q(w.s).s(b" takes 0 to 65535 after it")));
            };
            let v16 = num_in(v.s, 0, 65535).ok_or_else(|| (ERR_TEXT, line.at(v.at).s(b"a and b take 0 to 65535; found ").q(v.s)))? as u16;
            if to == 3 { s.a = Some(v16) } else { s.b = Some(v16) }
            i += 1;
        }
        i += 1;
    }
    Ok(())
}

/// `value <letter> <A> <B>`.
fn value(b: &mut Beat, line: &Line, t: &[Tok]) -> Result<(), Refusal> {
    const TAKES: &[u8] = b"value takes a letter (a to w or A to W) and A and B, each 0 to 65535";
    if t.len() != 4 {
        return Err((ERR_TEXT, line.whole().s(TAKES)));
    }
    let l = t[1];
    let idx = match l.s {
        [c] => letter_index(*c),
        _ => None,
    };
    let Some(idx) = idx else {
        let why: &[u8] = if l.s == b"x" || l.s == b"X" { b"; x is a step on keeping nothing, not a letter" } else { b"; found " };
        let m = line.at(l.at).s(TAKES).s(why);
        return Err((ERR_TEXT, if why == b"; found " { m.q(l.s) } else { m }));
    };
    if b.value_line[idx] != 0 {
        return Err((ERR_TEXT, line.at(l.at).s(b"value ").s(l.s).s(b" comes once; it was given on line ").n(b.value_line[idx])));
    }
    for i in 0..2 {
        b.values[idx][i] = num_in(t[2 + i].s, 0, 65535).ok_or_else(|| (ERR_TEXT, line.at(t[2 + i].at).s(TAKES).s(b"; found ").q(t[2 + i].s)))? as u16;
    }
    b.value_line[idx] = line.no;
    Ok(())
}

/// `pattern <n> [length <L>]`: the rows and lane after it are pattern n's.
fn pattern(b: &mut Beat, line: &Line, t: &[Tok]) -> Result<(), Refusal> {
    const TAKES: &[u8] = b"pattern takes a number, 1 to 16, then length 1 to 16 if not 16";
    let Some(n) = t.get(1) else {
        return Err((ERR_TEXT, line.whole().s(TAKES)));
    };
    let p = num_in(n.s, 1, 16).ok_or_else(|| (ERR_TEXT, line.at(n.at).s(TAKES).s(b"; found ").q(n.s)))? as usize - 1;
    let pat = &mut b.pats[p];
    if pat.line != 0 {
        return Err((ERR_TEXT, line.at(t[0].at).s(b"pattern ").n(p as u32 + 1).s(b" comes once; it was given on line ").n(pat.line)));
    }
    match t {
        [_, _] => {}
        [_, _, w, v] if w.s == b"length" => {
            pat.block[B_LENGTH] = num_in(v.s, 1, 16).ok_or_else(|| (ERR_TEXT, line.at(v.at).s(TAKES).s(b"; found ").q(v.s)))? as u8;
        }
        [_, _, w, ..] => return Err((ERR_TEXT, line.at(w.at).s(TAKES).s(b"; found ").q(w.s))),
        _ => return Err((ERR_TEXT, line.whole().s(TAKES))),
    }
    pat.line = line.no;
    b.current = Some(p);
    Ok(())
}

fn in_pattern(b: &Beat, line: &Line, what: &[u8]) -> Result<usize, Refusal> {
    b.current.ok_or_else(|| (ERR_TEXT, line.whole().s(what).s(b" belongs to a pattern: a 'pattern <n>' line comes first")))
}

/// `fx <step>=<effect>...`: pattern n's lane.
fn fx(b: &mut Beat, line: &Line, t: &[Tok]) -> Result<(), Refusal> {
    const TAKES: &[u8] = b"fx takes <step>=<effect>, steps 1 to 16, effects 1 to 16, each step once";
    let p = in_pattern(b, line, b"an fx line")?;
    let pat = &mut b.pats[p];
    if pat.fx_line != 0 {
        return Err((ERR_TEXT, line.at(t[0].at).s(b"fx comes once a pattern; pattern ").n(p as u32 + 1).s(b"'s was given on line ").n(pat.fx_line)));
    }
    if t.len() < 2 {
        return Err((ERR_TEXT, line.whole().s(TAKES)));
    }
    for e in &t[1..] {
        let bad = || (ERR_TEXT, line.at(e.at).s(TAKES).s(b"; found ").q(e.s));
        let (s, v) = pair(e.s, b'=').ok_or_else(bad)?;
        let s = num_in(s, 1, 16).ok_or_else(bad)? as usize - 1;
        let v = num_in(v, 1, 16).ok_or_else(bad)? as u8;
        if pat.block[B_LANE + s] != 0 {
            return Err((ERR_TEXT, line.at(e.at).s(TAKES).s(b"; step ").n(s as u32 + 1).s(b" is given twice")));
        }
        pat.block[B_LANE + s] = v;
    }
    pat.fx_line = line.no;
    Ok(())
}

/// A sound's row: `<key> [<name>] <sixteen steps> [prob <step>=<p>...] [keep <step>=<A>,<B>...]`.
fn row(b: &mut Beat, line: &Line, t: &[Tok]) -> Result<(), Refusal> {
    const TAKES: &[u8] = b"a row takes a key, its sound's name if given, sixteen steps, then prob and keep lists";
    let p = in_pattern(b, line, b"a row")?;
    let k = t[0];
    let key = num_in(k.s, 1, 16).ok_or_else(|| (ERR_TEXT, line.at(k.at).s(b"a row starts with a key, 1 to 16; found ").q(k.s)))? as usize;
    let tr = key - 1;
    if b.pats[p].rows >> tr & 1 == 1 {
        return Err((ERR_TEXT, line.at(k.at).s(b"key ").n(key as u32).s(b" has a row in pattern ").n(p as u32 + 1).s(b" already")));
    }
    let mut i = 1;
    let Some(second) = t.get(1) else {
        return Err((ERR_TEXT, line.whole().s(TAKES)));
    };
    let followed = t.get(2).is_some_and(|u| u.s != b"prob" && u.s != b"keep");
    if let Some(n) = name_index(second.s) {
        if n != tr {
            let m = line.at(second.at).s(b"key ").n(key as u32).s(b" is the ").s(NAMES[tr].as_bytes()).s(b", not the ").s(second.s);
            return Err((ERR_TEXT, m));
        }
        i = 2;
    } else if second.s.len() != 16 && followed {
        let m = line.at(second.at).s(b"no sound is called ").q(second.s).s(b"; key ").n(key as u32).s(b" is the ").s(NAMES[tr].as_bytes());
        return Err((ERR_TEXT, m));
    }
    let Some(steps) = t.get(i) else {
        return Err((ERR_TEXT, line.whole().s(b"a row takes sixteen steps after its key and name")));
    };
    for (j, &c) in steps.s.iter().enumerate() {
        if !(c == b'.' || c == b'x' || c == b'*' || letter_index(c).is_some()) {
            let m = line.at(steps.at + j).s(b"step ").n(j as u32 + 1).s(b": '").s(&[c]).s(b"' is not a step: . off, x on, a value's letter (a to w, A to W), or *");
            return Err((ERR_TEXT, m));
        }
    }
    if steps.s.len() != 16 {
        return Err((ERR_TEXT, line.at(steps.at).s(b"a row takes sixteen steps; this has ").n(steps.s.len() as u32)));
    }
    let pat = &mut b.pats[p];
    let mut on = 0u16;
    for (s, &c) in steps.s.iter().enumerate() {
        if c != b'.' {
            on |= 1 << s;
        }
        if let Some(l) = letter_index(c) {
            pat.letter[tr][s] = l as u8 + 1;
            if b.first_use[l].0 == 0 {
                b.first_use[l] = (line.no, line.col(steps.at + s));
            }
        }
    }
    put(&mut pat.block, B_STEPS + 2 * tr, &on.to_le_bytes());
    let mut keep = 0u16;
    let mut probbed = 0u16; // a probability given as 100 is the default too: "twice" needs its own mark
    let mut probs = false;
    let mut keeps = false;
    i += 1;
    while i < t.len() {
        let w = t[i];
        let is_prob = match w.s {
            b"prob" if !probs => true,
            b"keep" if !keeps => false,
            b"prob" | b"keep" => return Err((ERR_TEXT, line.at(w.at).s(w.s).s(b" comes once in a row"))),
            _ => return Err((ERR_TEXT, line.at(w.at).s(b"after the sixteen steps a row takes prob and keep lists; found ").q(w.s))),
        };
        i += 1;
        let first = i;
        while i < t.len() && t[i].s != b"prob" && t[i].s != b"keep" {
            let e = t[i];
            if is_prob {
                const TAKES: &[u8] = b"prob takes <step>=<0 to 100>, steps 1 to 16, each once";
                let bad = || (ERR_TEXT, line.at(e.at).s(TAKES).s(b"; found ").q(e.s));
                let (s, v) = pair(e.s, b'=').ok_or_else(bad)?;
                let s = num_in(s, 1, 16).ok_or_else(bad)? as usize - 1;
                let v = num_in(v, 0, 100).ok_or_else(bad)? as u8;
                let at = B_PROB + 16 * tr + s;
                if probbed >> s & 1 == 1 {
                    return Err((ERR_TEXT, line.at(e.at).s(TAKES).s(b"; step ").n(s as u32 + 1).s(b" is given twice")));
                }
                probbed |= 1 << s;
                pat.block[at] = v;
            } else {
                const TAKES: &[u8] = b"keep takes <step>=<A>,<B>, steps 1 to 16, each once, A and B 0 to 65535";
                let bad = || (ERR_TEXT, line.at(e.at).s(TAKES).s(b"; found ").q(e.s));
                let (s, ab) = pair(e.s, b'=').ok_or_else(bad)?;
                let (a, bv) = pair(ab, b',').ok_or_else(bad)?;
                let s = num_in(s, 1, 16).ok_or_else(bad)? as usize - 1;
                let a = num_in(a, 0, 65535).ok_or_else(bad)? as u16;
                let bv = num_in(bv, 0, 65535).ok_or_else(bad)? as u16;
                if keep >> s & 1 == 1 {
                    return Err((ERR_TEXT, line.at(e.at).s(TAKES).s(b"; step ").n(s as u32 + 1).s(b" is given twice")));
                }
                let c = steps.s[s];
                if c != b'*' && c != b'.' {
                    let m = line.at(e.at).s(b"step ").n(s as u32 + 1).s(b" is '").s(&[c]).s(b"': keep is for a * step or an off one");
                    return Err((ERR_TEXT, m));
                }
                keep |= 1 << s;
                let v = B_VALUES + 4 * (16 * tr + s);
                put(&mut pat.block, v, &a.to_le_bytes());
                put(&mut pat.block, v + 2, &bv.to_le_bytes());
            }
            i += 1;
        }
        if i == first {
            return Err((ERR_TEXT, line.at(w.at).s(w.s).s(b" takes at least one <step>=")));
        }
        if is_prob { probs = true } else { keeps = true }
    }
    for (s, &c) in steps.s.iter().enumerate() {
        if c == b'*' && keep >> s & 1 == 0 {
            let m = line.at(steps.at + s).s(b"step ").n(s as u32 + 1).s(b" is * and keep gives it no pair");
            return Err((ERR_TEXT, m));
        }
        if letter_index(c).is_some() {
            keep |= 1 << s;
        }
    }
    put(&mut pat.block, B_KEEPS + 2 * tr, &keep.to_le_bytes());
    pat.rows |= 1 << tr;
    Ok(())
}

/// After the last line: the letters resolved, the sounds settled, the snapshot laid out.
fn finish(b: &mut Beat, out: &mut [u8]) -> Result<Read, Refusal> {
    for p in 0..PATTERNS_V5 {
        for tr in 0..TRACKS_V5 {
            for s in 0..STEPS_V5 {
                let l = b.pats[p].letter[tr][s];
                if l == 0 {
                    continue;
                }
                let l = l as usize - 1;
                if b.value_line[l] == 0 {
                    let (no, col) = b.first_use[l];
                    let m = Msg::new().s(b"line ").n(no).s(b", column ").n(col).s(b": no value line gives the letter ").s(&[LETTERS[l]]);
                    return Err((ERR_TEXT, m));
                }
                let v = B_VALUES + 4 * (16 * tr + s);
                let [x, y] = b.values[l];
                put(&mut b.pats[p].block, v, &x.to_le_bytes());
                put(&mut b.pats[p].block, v + 2, &y.to_le_bytes());
            }
        }
    }
    let mut sampled = 0u16;
    let mut fx_sounds = 0u16;
    let mut sliders = [[0u16; 2]; TRACKS_V5];
    for (t, s) in b.sounds.iter().enumerate() {
        let own = b.own >> t & 1 == 1;
        if own && s.sample == Some(false) {
            let m = Msg::new().s(b"line ").n(b.given[Word::Own as usize]).s(b": own names key ").n(t as u32 + 1).s(b", which line ").n(s.line).s(b" says is code: own is for a key that plays a sample");
            return Err((ERR_TEXT, m));
        }
        if own || s.sample == Some(true) {
            sampled |= 1 << t;
        }
        if s.fx != Some(false) {
            fx_sounds |= 1 << t;
        }
        sliders[t] = [s.a.unwrap_or(b.sliders[0]), s.b.unwrap_or(b.sliders[1])];
    }
    // A pattern equal to a cleared one has no block (§21): named or not, it is not written.
    let mut written = 0u16;
    for (p, pat) in b.pats.iter().enumerate() {
        if pat.block != CLEARED {
            written |= 1 << p;
        }
    }
    let size = SIZE_MIN + BLOCK * written.count_ones() as usize;
    let Some(o) = out.get_mut(..size) else {
        let m = Msg::new().s(b"the snapshot takes ").n(size as u32).s(b" bytes; the room given is ").n(out.len() as u32);
        return Err((ERR_ROOM, m));
    };
    o.fill(0);
    put(o, AT_MAGIC, &MAGIC);
    put(o, AT_VERSION, &VERSION.to_le_bytes());
    put(o, AT_KIND, &KIND_RHYTHM.to_le_bytes());
    put(o, AT_PAYLOAD, &((size - HEADER - 4) as u32).to_le_bytes());
    put(o, AT_SEED, &b.seed.to_le_bytes());
    put(o, AT_TEMPO, &b.tempo.to_le_bytes());
    o[AT_SWING] = b.swing;
    o[AT_VOLUME] = b.volume;
    o[AT_CHOSEN] = b.chosen;
    o[AT_HAT_CUT] = 0;
    for (t, ab) in sliders.iter().enumerate() {
        put(o, AT_SLIDERS + 4 * t, &ab[0].to_le_bytes());
        put(o, AT_SLIDERS + 4 * t + 2, &ab[1].to_le_bytes());
    }
    put(o, AT_SAMPLED, &sampled.to_le_bytes());
    put(o, AT_FX_SOUNDS, &fx_sounds.to_le_bytes());
    put(o, AT_WRITTEN, &written.to_le_bytes());
    o[AT_CHAIN_LEN] = b.chain_len;
    put(o, AT_CHAIN, &b.chain[..b.chain_len as usize]);
    put(o, AT_NAME, &b.name);
    put(o, AT_CREDIT, &b.credit);
    let mut at = AT_BLOCKS;
    for (p, pat) in b.pats.iter().enumerate() {
        if written >> p & 1 == 1 {
            put(o, at, &pat.block);
            at += BLOCK;
        }
    }
    let crc = crc32(&o[..size - 4]);
    put(o, size - 4, &crc.to_le_bytes());
    Ok(Read { len: size, own: b.own })
}

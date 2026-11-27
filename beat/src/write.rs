//! `docs/BEAT.md`, "Canonical form": snapshot version 5 into text. A snapshot the core did not
//! write (a field outside what the core ever saves, a cleared pattern written, a name the text
//! cannot carry back) is refused, so whatever is written reads back to the same bytes.

use crate::layout::*;
use crate::text::{Sink, crc32, u16_at, u32_at};
use crate::{ERR_ROOM, ERR_SNAPSHOT, LETTERS, NAMES};

/// Writes `snap` as text into `out`. Returns the text's length; nothing is written to `out`
/// unless all of it fits.
pub fn write(snap: &[u8], own: u16, out: &mut [u8]) -> Result<usize, i32> {
    let s = check(snap).ok_or(ERR_SNAPSHOT)?;
    let mut count = Sink::count();
    emit(s, own, &mut count);
    let n = count.len;
    let Some(dst) = out.get_mut(..n) else {
        return Err(ERR_ROOM);
    };
    let mut sink = Sink::into(dst);
    emit(s, own, &mut sink);
    Ok(n)
}

fn block(s: &[u8], i: usize) -> &[u8] {
    &s[AT_BLOCKS + BLOCK * i..AT_BLOCKS + BLOCK * (i + 1)]
}

/// The name's or the credit's text: its bytes before the zero padding.
fn text_of(field: &[u8]) -> &[u8] {
    let n = field.iter().position(|c| *c == 0).unwrap_or(field.len());
    &field[..n]
}

/// A name or credit the text carries back to the same bytes: UTF-8, printable, zero-padded,
/// no space at either end (the reader drops those).
fn text_ok(field: &[u8]) -> bool {
    let t = text_of(field);
    field[t.len()..].iter().all(|c| *c == 0)
        && core::str::from_utf8(t).is_ok()
        && t.iter().all(|c| *c >= 0x20 && *c != 0x7F)
        && t.first() != Some(&b' ')
        && t.last() != Some(&b' ')
}

/// Version 5 as the core writes it, or `None`. Returns the snapshot cut to its own length:
/// bytes after the CRC are the host's, as for `pi_load`.
fn check(snap: &[u8]) -> Option<&[u8]> {
    if snap.len() < SIZE_MIN || snap[AT_MAGIC..AT_MAGIC + 4] != MAGIC {
        return None;
    }
    if u16_at(snap, AT_VERSION) != VERSION || u16_at(snap, AT_KIND) != KIND_RHYTHM {
        return None;
    }
    let written = u16_at(snap, AT_WRITTEN);
    let size = SIZE_MIN + BLOCK * written.count_ones() as usize;
    let s = snap.get(..size)?;
    if u32_at(s, AT_PAYLOAD) as usize != size - HEADER - 4 || u32_at(s, size - 4) != crc32(&s[..size - 4]) {
        return None;
    }
    let tempo = u32_at(s, AT_TEMPO);
    let chain_len = s[AT_CHAIN_LEN] as usize;
    let fields = u32_at(s, AT_SEED) != 0
        && (30_000..=300_000).contains(&tempo)
        && s[AT_SWING] <= 48
        && (1..=16).contains(&s[AT_VOLUME])
        && s[AT_CHOSEN] < 16
        && s[AT_HAT_CUT] == 0
        && (1..=PATTERNS_V5).contains(&chain_len)
        && s[AT_CHAIN_LEN + 1] == 0
        && s[AT_CHAIN..AT_CHAIN + chain_len].iter().all(|p| (*p as usize) < PATTERNS_V5)
        && s[AT_CHAIN + chain_len..AT_NAME].iter().all(|b| *b == 0)
        && text_ok(&s[AT_NAME..AT_NAME + NAME_LEN])
        && text_ok(&s[AT_CREDIT..AT_CREDIT + CREDIT_LEN]);
    if !fields {
        return None;
    }
    for i in 0..written.count_ones() as usize {
        let b = block(s, i);
        let mut cleared = b[B_LENGTH] == 16;
        for t in 0..TRACKS_V5 {
            let on = u16_at(b, B_STEPS + 2 * t);
            let keep = u16_at(b, B_KEEPS + 2 * t);
            cleared &= on == 0 && keep == 0;
            for st in 0..STEPS_V5 {
                let p = b[B_PROB + 16 * t + st];
                if p > 100 {
                    return None;
                }
                cleared &= p == 100;
                let v = B_VALUES + 4 * (16 * t + st);
                if keep >> st & 1 == 0 && b[v..v + 4] != [0; 4] {
                    return None; // §20: zero where nothing is kept
                }
            }
        }
        for st in 0..STEPS_V5 {
            if b[B_LANE + st] > 16 {
                return None;
            }
            cleared &= b[B_LANE + st] == 0;
        }
        if !(1..=16).contains(&b[B_LENGTH]) || b[B_LENGTH + 1..] != [0, 0, 0] || cleared {
            return None; // a cleared pattern is never written (§21)
        }
    }
    Some(s)
}

/// The value letters, in the order the steps first use a pair: pattern 1 to 16, key 1 to 16,
/// step 1 to 16, `a` first; at most 46, and past them `*` and a `keep` list.
struct Letters {
    pairs: [[u16; 2]; 46],
    n: usize,
}

impl Letters {
    fn of(s: &[u8]) -> Letters {
        let mut l = Letters { pairs: [[0; 2]; 46], n: 0 };
        let written = u16_at(s, AT_WRITTEN);
        for i in 0..written.count_ones() as usize {
            let b = block(s, i);
            for t in 0..TRACKS_V5 {
                let kept = u16_at(b, B_STEPS + 2 * t) & u16_at(b, B_KEEPS + 2 * t);
                for st in 0..STEPS_V5 {
                    if kept >> st & 1 == 1 {
                        let pair = value(b, t, st);
                        if l.find(pair).is_none() && l.n < 46 {
                            l.pairs[l.n] = pair;
                            l.n += 1;
                        }
                    }
                }
            }
        }
        l
    }

    fn find(&self, pair: [u16; 2]) -> Option<usize> {
        self.pairs[..self.n].iter().position(|p| *p == pair)
    }
}

fn value(b: &[u8], t: usize, st: usize) -> [u16; 2] {
    let v = B_VALUES + 4 * (16 * t + st);
    [u16_at(b, v), u16_at(b, v + 2)]
}

/// Whether a track has a row in a block: something on, kept, or a probability not 100.
fn has_row(b: &[u8], t: usize) -> bool {
    u16_at(b, B_STEPS + 2 * t) != 0 || u16_at(b, B_KEEPS + 2 * t) != 0 || b[B_PROB + 16 * t..B_PROB + 16 * (t + 1)].iter().any(|p| *p != 100)
}

/// ⚠ `docs/BEAT.md` does not say which value the `sliders` line gives when the sounds' own
/// differ. Read here: the value most sounds have, a tie going to the lowest key's; the others
/// are `sound` lines' `a` and `b`.
fn base(s: &[u8], half: usize) -> u16 {
    let v = |t: usize| u16_at(s, AT_SLIDERS + 4 * t + 2 * half);
    let mut best = (v(0), 0);
    for t in 0..TRACKS_V5 {
        let n = (0..TRACKS_V5).filter(|u| v(*u) == v(t)).count();
        if n > best.1 {
            best = (v(t), n);
        }
    }
    best.0
}

fn tempo(o: &mut Sink, mbpm: u32) {
    o.n(mbpm / 1000);
    let mut frac = mbpm % 1000;
    if frac != 0 {
        o.b(b'.');
        let mut scale = 100;
        while frac != 0 {
            o.b(b'0' + (frac / scale) as u8);
            frac %= scale;
            scale /= 10;
        }
    }
}

fn emit(s: &[u8], own: u16, o: &mut Sink) {
    o.s(b"pocket-instruments beat 1\n");
    for (word, at, len) in [(&b"name "[..], AT_NAME, NAME_LEN), (&b"credit "[..], AT_CREDIT, CREDIT_LEN)] {
        let t = text_of(&s[at..at + len]);
        if !t.is_empty() {
            o.s(word);
            o.s(t);
            o.b(b'\n');
        }
    }
    let mbpm = u32_at(s, AT_TEMPO);
    if mbpm != 120_000 {
        o.s(b"tempo ");
        tempo(o, mbpm);
        o.b(b'\n');
    }
    if s[AT_SWING] != 0 {
        o.s(b"swing ");
        o.n(s[AT_SWING] as u32);
        o.b(b'\n');
    }
    if s[AT_VOLUME] != 16 {
        o.s(b"volume ");
        o.n(s[AT_VOLUME] as u32);
        o.b(b'\n');
    }
    o.s(b"seed ");
    o.n(u32_at(s, AT_SEED));
    o.b(b'\n');
    if s[AT_CHOSEN] != 0 {
        o.s(b"chosen ");
        o.n(s[AT_CHOSEN] as u32 + 1);
        o.b(b'\n');
    }
    let (ba, bb) = (base(s, 0), base(s, 1));
    if (ba, bb) != (32768, 32768) {
        o.s(b"sliders ");
        o.n(ba as u32);
        o.b(b' ');
        o.n(bb as u32);
        o.b(b'\n');
    }
    let sampled = u16_at(s, AT_SAMPLED);
    let fx_sounds = u16_at(s, AT_FX_SOUNDS);
    for t in 0..TRACKS_V5 {
        let (a, b) = (u16_at(s, AT_SLIDERS + 4 * t), u16_at(s, AT_SLIDERS + 4 * t + 2));
        let sample = sampled >> t & 1 == 1;
        let dry = fx_sounds >> t & 1 == 0;
        if !sample && !dry && a == ba && b == bb {
            continue;
        }
        o.s(b"sound ");
        o.n(t as u32 + 1);
        if sample {
            o.s(b" sample");
        }
        if dry {
            o.s(b" dry");
        }
        if a != ba {
            o.s(b" a ");
            o.n(a as u32);
        }
        if b != bb {
            o.s(b" b ");
            o.n(b as u32);
        }
        o.b(b'\n');
    }
    // `own`: the keys that played the writer's own sample (D34); only a key that plays one.
    let own = own & sampled;
    if own != 0 {
        o.s(b"own");
        for t in (0..TRACKS_V5).filter(|t| own >> t & 1 == 1) {
            o.b(b' ');
            o.n(t as u32 + 1);
        }
        o.b(b'\n');
    }
    let letters = Letters::of(s);
    for (i, [a, b]) in letters.pairs[..letters.n].iter().enumerate() {
        o.s(b"value ");
        o.b(LETTERS[i]);
        o.b(b' ');
        o.n(*a as u32);
        o.b(b' ');
        o.n(*b as u32);
        o.b(b'\n');
    }
    let chain_len = s[AT_CHAIN_LEN] as usize;
    if chain_len != 1 || s[AT_CHAIN] != 0 {
        o.s(b"chain");
        for p in &s[AT_CHAIN..AT_CHAIN + chain_len] {
            o.b(b' ');
            o.n(*p as u32 + 1);
        }
        o.b(b'\n');
    }
    // The name column: one more than the longest name of any row in the file.
    let written = u16_at(s, AT_WRITTEN);
    let mut width = 0;
    for i in 0..written.count_ones() as usize {
        for t in (0..TRACKS_V5).filter(|t| has_row(block(s, i), *t)) {
            width = width.max(NAMES[t].len() + 1);
        }
    }
    let mut i = 0;
    for p in (0..PATTERNS_V5).filter(|p| written >> p & 1 == 1) {
        let b = block(s, i);
        i += 1;
        o.s(b"\npattern ");
        o.n(p as u32 + 1);
        if b[B_LENGTH] != 16 {
            o.s(b" length ");
            o.n(b[B_LENGTH] as u32);
        }
        o.b(b'\n');
        for t in (0..TRACKS_V5).filter(|t| has_row(b, *t)) {
            row(o, b, t, &letters, width);
        }
        if b[B_LANE..B_LANE + STEPS_V5].iter().any(|e| *e != 0) {
            o.s(b"fx");
            for st in (0..STEPS_V5).filter(|st| b[B_LANE + st] != 0) {
                o.b(b' ');
                o.n(st as u32 + 1);
                o.b(b'=');
                o.n(b[B_LANE + st] as u32);
            }
            o.b(b'\n');
        }
    }
}

fn row(o: &mut Sink, b: &[u8], t: usize, letters: &Letters, width: usize) {
    let on = u16_at(b, B_STEPS + 2 * t);
    let keep = u16_at(b, B_KEEPS + 2 * t);
    // The key in two columns and a space; the name padded to the file's column.
    o.n(t as u32 + 1);
    o.s(if t < 9 { b"  " } else { b" " });
    let name = NAMES[t].as_bytes();
    o.s(name);
    for _ in name.len()..width {
        o.b(b' ');
    }
    let mut listed = 0u16; // steps whose pair goes in the keep list
    for st in 0..STEPS_V5 {
        let (is_on, kept) = (on >> st & 1 == 1, keep >> st & 1 == 1);
        let c = match (is_on, kept) {
            (false, false) => b'.',
            (true, false) => b'x',
            (false, true) => {
                listed |= 1 << st;
                b'.'
            }
            (true, true) => match letters.find(value(b, t, st)) {
                Some(l) => LETTERS[l],
                None => {
                    listed |= 1 << st;
                    b'*'
                }
            },
        };
        o.b(c);
    }
    let probs = &b[B_PROB + 16 * t..B_PROB + 16 * (t + 1)];
    if probs.iter().any(|p| *p != 100) {
        o.s(b" prob");
        for (st, p) in probs.iter().enumerate().filter(|(_, p)| **p != 100) {
            o.b(b' ');
            o.n(st as u32 + 1);
            o.b(b'=');
            o.n(*p as u32);
        }
    }
    if listed != 0 {
        o.s(b" keep");
        for st in (0..STEPS_V5).filter(|st| listed >> st & 1 == 1) {
            let [a, v] = value(b, t, st);
            o.b(b' ');
            o.n(st as u32 + 1);
            o.b(b'=');
            o.n(a as u32);
            o.b(b',');
            o.n(v as u32);
        }
    }
    o.b(b'\n');
}

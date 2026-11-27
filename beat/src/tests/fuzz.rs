//! Property tests in the crate alone, seeded so a failure names its seed. (The fuzz the
//! design asks for, edits on a real engine saved by the core, is `core/src/tests/beat.rs`.)
//!
//! 1. A random snapshot of every field the core writes: written as text, read back, the same
//!    bytes; written again, the same text.
//! 2. Random edits of good text: never a panic; what reads, writes canonical text that reads
//!    back to the same bytes; what is refused says a line.

use super::*;
use crate::text::crc32;

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    pub fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }
    pub fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }
    pub fn chance(&mut self, in_n: u32) -> bool {
        self.below(in_n) == 0
    }
}

/// Text a name or credit can hold and read back: printable, a few multi-byte characters, no
/// space at either end, cut on a character boundary.
fn random_text(r: &mut Rng, most: usize) -> Vec<u8> {
    const PIECES: [&str; 10] = ["a", "Z", " ", "#", "é", "€", "9", ";", "ü", "𝄞"];
    let mut t = String::new();
    for _ in 0..r.below(most as u32 + 4) {
        let p = PIECES[r.below(10) as usize];
        if t.len() + p.len() > most {
            break;
        }
        t.push_str(p);
    }
    t.trim_matches(' ').as_bytes().to_vec()
}

/// A random version 5 snapshot whose every field is one the core can write.
pub fn random_snapshot(r: &mut Rng) -> Vec<u8> {
    let written = if r.chance(8) { 0 } else if r.chance(8) { 0xffff } else { r.next() as u16 & r.next() as u16 };
    let size = SIZE_MIN + BLOCK * written.count_ones() as usize;
    let mut s = vec![0u8; size];
    s[..8].copy_from_slice(b"PKIN\x05\x00\x01\x00");
    s[8..12].copy_from_slice(&((size - 16) as u32).to_le_bytes());
    s[12..16].copy_from_slice(&r.next().max(1).to_le_bytes());
    let mbpm = if r.chance(3) { 120_000 } else { 30_000 + r.below(270_001) };
    s[16..20].copy_from_slice(&mbpm.to_le_bytes());
    s[20] = if r.chance(2) { 0 } else { r.below(49) as u8 };
    s[21] = if r.chance(2) { 16 } else { 1 + r.below(16) as u8 };
    s[22] = if r.chance(2) { 0 } else { r.below(16) as u8 };
    let common = [r.next() as u16, r.next() as u16];
    for t in 0..16 {
        for h in 0..2 {
            let v = if r.chance(3) { r.next() as u16 } else if r.chance(4) { 32768 } else { common[h] };
            s[24 + 4 * t + 2 * h..26 + 4 * t + 2 * h].copy_from_slice(&v.to_le_bytes());
        }
    }
    let sampled = if r.chance(2) { 0 } else { r.next() as u16 };
    let fx = if r.chance(2) { 0xffff } else { r.next() as u16 };
    s[88..90].copy_from_slice(&sampled.to_le_bytes());
    s[90..92].copy_from_slice(&fx.to_le_bytes());
    s[92..94].copy_from_slice(&written.to_le_bytes());
    let chain_len = if r.chance(2) { 1 } else { 1 + r.below(16) as usize };
    s[94] = chain_len as u8;
    for i in 0..chain_len {
        s[96 + i] = if r.chance(2) { 0 } else { r.below(16) as u8 };
    }
    let name = random_text(r, 32);
    s[112..112 + name.len()].copy_from_slice(&name);
    let credit = random_text(r, 128);
    s[144..144 + credit.len()].copy_from_slice(&credit);
    // A small pool of pairs, so letters repeat; sometimes more than 46.
    let many = if r.chance(4) { 200 } else { 12 };
    let n = 1 + r.below(many);
    let pool: Vec<[u16; 2]> = (0..n).map(|_| [r.next() as u16, r.next() as u16]).collect();
    for i in 0..written.count_ones() as usize {
        let b = &mut s[272 + 1364 * i..272 + 1364 * (i + 1)];
        let dense = r.below(4);
        for t in 0..16 {
            if r.below(4) > dense {
                for st in 0..16 {
                    b[32 + 16 * t + st] = 100;
                }
                continue;
            }
            let on = r.next() as u16 & r.next() as u16;
            let keep = if r.chance(3) { on } else { r.next() as u16 & (on | r.next() as u16) };
            b[2 * t..2 * t + 2].copy_from_slice(&on.to_le_bytes());
            b[304 + 2 * t..306 + 2 * t].copy_from_slice(&keep.to_le_bytes());
            for st in 0..16 {
                b[32 + 16 * t + st] = if r.chance(5) { r.below(101) as u8 } else { 100 };
                if keep >> st & 1 == 1 {
                    let [x, y] = pool[r.below(pool.len() as u32) as usize];
                    let v = 336 + 4 * (16 * t + st);
                    b[v..v + 2].copy_from_slice(&x.to_le_bytes());
                    b[v + 2..v + 4].copy_from_slice(&y.to_le_bytes());
                }
            }
        }
        for st in 0..16 {
            b[288 + st] = if r.chance(4) { r.below(17) as u8 } else { 0 };
        }
        b[1360] = if r.chance(2) { 16 } else { 1 + r.below(16) as u8 };
        // A written block is never a cleared one (§21).
        let cleared = b[..32].iter().all(|x| *x == 0) && b[32..288].iter().all(|x| *x == 100) && b[288..304].iter().all(|x| *x == 0) && b[304..336].iter().all(|x| *x == 0) && b[1360] == 16;
        if cleared {
            b[1360] = 1 + r.below(15) as u8;
        }
    }
    let crc = crc32(&s[..size - 4]);
    s[size - 4..].copy_from_slice(&crc.to_le_bytes());
    s
}

#[test]
fn random_snapshots_write_and_read_back_to_the_same_bytes() {
    for seed in 1..=3000u64 {
        let mut r = Rng::new(seed);
        let s = random_snapshot(&mut r);
        let own = r.next() as u16;
        let text = text_of(&s, own);
        let mut back = vec![0u8; SIZE];
        let got = read(text.as_bytes(), &mut back).unwrap_or_else(|(_, m)| panic!("seed {seed}: {}\n{text}", String::from_utf8_lossy(m.as_bytes())));
        assert_eq!(&back[..got.len], &s[..], "seed {seed}:\n{text}");
        assert_eq!(got.own, own & u16::from_le_bytes([s[88], s[89]]), "seed {seed}: own, for the keys that play a sample");
        assert_eq!(text_of(&back[..got.len], got.own), text, "seed {seed}: the same text again");
    }
}

/// Bytes a beat file is made of, and a few it is not, for edits.
const ALPHABET: &[u8] = b" \n.x*abwAW#0123456789=,-\t\r\xc3\xa9\xff";

fn mutate(r: &mut Rng, t: &mut Vec<u8>) {
    for _ in 0..1 + r.below(4) {
        let at = if t.is_empty() { 0 } else { r.below(t.len() as u32 + 1) as usize };
        match r.below(6) {
            0 if !t.is_empty() && at < t.len() => {
                t.remove(at);
            }
            1 => t.insert(at, ALPHABET[r.below(ALPHABET.len() as u32) as usize]),
            2 if at < t.len() => t[at] = ALPHABET[r.below(ALPHABET.len() as u32) as usize],
            3 if at < t.len() && t[at].is_ascii_digit() => t[at] = b'0' + r.below(10) as u8,
            4 => {
                // a whole line again, somewhere else
                let lines: Vec<&[u8]> = t.split(|c| *c == b'\n').collect();
                let l = lines[r.below(lines.len() as u32) as usize].to_vec();
                let mut l = l;
                l.push(b'\n');
                let start = t[..at].iter().rposition(|c| *c == b'\n').map_or(0, |i| i + 1);
                t.splice(start..start, l);
            }
            _ => {
                let words: [&[u8]; 8] = [b"prob 3=50 ", b"keep 2=1,2 ", b"pattern 5\n", b"value c 1 2\n", b"own 4\n", b"sound 2 a 7\n", b"16 bass ", b"length 3 "];
                let w = words[r.below(8) as usize];
                t.splice(at..at, w.iter().copied());
            }
        }
    }
}

#[test]
fn random_edits_of_good_text_never_panic_and_what_reads_writes_back_canonical() {
    let (mut read_ok, mut refused) = (0, 0);
    for seed in 1..=20_000u64 {
        let mut r = Rng::new(seed);
        let mut text = if r.chance(2) { EXAMPLE.as_bytes().to_vec() } else { text_of(&random_snapshot(&mut r), r.next() as u16).into_bytes() };
        mutate(&mut r, &mut text);
        let mut snap = vec![0u8; SIZE];
        match read(&text, &mut snap) {
            Ok(got) => {
                read_ok += 1;
                let canon = text_of(&snap[..got.len], got.own);
                let again = bytes_of(&canon);
                assert_eq!(again, &snap[..got.len], "seed {seed}: canonical text reads back to the same bytes");
                assert_eq!(text_of(&again, got.own), canon, "seed {seed}: and writes the same text");
            }
            Err((code, m)) => {
                refused += 1;
                let m = core::str::from_utf8(m.as_bytes()).expect("a message is UTF-8");
                assert!(m.starts_with("line ") || code == crate::ERR_ROOM, "seed {seed}: {m}");
                assert!(code == crate::ERR_TEXT || code == crate::ERR_NEWER, "seed {seed}: {code} {m}");
            }
        }
    }
    println!("random edits: {read_ok} read, {refused} refused");
    assert!(read_ok > 1000 && refused > 1000, "both kinds reached: {read_ok} read, {refused} refused");
}

#[test]
fn random_bytes_never_panic_and_a_snapshot_not_the_core_s_is_refused() {
    for seed in 1..=5000u64 {
        let mut r = Rng::new(seed);
        let len = r.below(600) as usize;
        let mut text: Vec<u8> = (0..len).map(|_| r.next() as u8).collect();
        if r.chance(2) {
            text.splice(0..0, b"pocket-instruments beat 1\n".iter().copied());
        }
        let mut snap = vec![0u8; SIZE];
        let _ = read(&text, &mut snap);
        // A good snapshot with one byte changed: refused by its CRC, or by the field, or, if the
        // CRC is fixed and the field is still one the core writes, written and read back.
        let mut s = random_snapshot(&mut r);
        let at = r.below(s.len() as u32) as usize;
        s[at] ^= 1 << r.below(8);
        let mut out = vec![0u8; TEXT_MAX];
        assert_eq!(write(&s, 0, &mut out), Err(crate::ERR_SNAPSHOT), "seed {seed}: a bit flipped at {at}");
        let n = s.len() - 4;
        let crc = crc32(&s[..n]);
        s[n..].copy_from_slice(&crc.to_le_bytes());
        if let Ok(k) = write(&s, 0, &mut out) {
            let back = bytes_of(core::str::from_utf8(&out[..k]).unwrap());
            assert_eq!(back, s, "seed {seed}: a flipped bit the text can carry, at {at}");
        }
        // And every length short of the whole is refused, never read past.
        let cut = r.below(s.len() as u32) as usize;
        assert_eq!(write(&s[..cut], 0, &mut out), Err(crate::ERR_SNAPSHOT));
    }
}

//! `PIB_TEXT_MAX`: the most text any snapshot can be written as. `docs/BEAT.md` guessed under
//! 64 KiB "by arithmetic" ⚠ and left the build to measure it. Measured here: the bound below,
//! summed line by line from the most each line can hold, and a snapshot built to come near it.

use super::*;
use crate::text::crc32;

/// The most each line can be, summed: an upper bound on any text `write` gives.
fn bound() -> usize {
    let keys = |one: usize, two: usize| 9 * one + 7 * two; // keys 1 to 9, then 10 to 16
    let header = "pocket-instruments beat 1\n".len();
    let name = "name ".len() + NAME_LEN + 1;
    let credit = "credit ".len() + CREDIT_LEN + 1;
    let tempo = "tempo 299.999\n".len();
    let swing = "swing 48\n".len();
    let volume = "volume 15\n".len();
    let seed = "seed 4294967295\n".len();
    let chosen = "chosen 16\n".len();
    let sliders = "sliders 65535 65535\n".len();
    let sounds = keys("sound 9 sample dry a 65535 b 65535\n".len(), "sound 16 sample dry a 65535 b 65535\n".len());
    let own = "own".len() + keys(2, 3) + 1;
    let values = 46 * "value W 65535 65535\n".len();
    let chain = "chain".len() + 16 * " 16".len() + 1;
    let width = crate::NAMES.iter().map(|n| n.len()).max().unwrap() + 1;
    let prob = " prob".len() + keys(" 9=99".len(), " 16=99".len());
    let keep = " keep".len() + keys(" 9=65535,65535".len(), " 16=65535,65535".len());
    let row = 3 + width + 16 + prob + keep + 1;
    let fx = "fx".len() + keys(" 9=16".len(), " 16=16".len()) + 1;
    let patterns = keys("\npattern 9 length 15\n".len(), "\npattern 16 length 15\n".len()) + 16 * (16 * row + fx);
    header + name + credit + tempo + swing + volume + seed + chosen + sliders + sounds + own + values + chain + patterns
}

/// A snapshot as long in text as can be: every field at its widest, sixteen patterns with every
/// step on and its own pair, the first 46 pairs used once each (a value line is longer than the
/// keep entry it saves), every probability 99, every lane step effect 16, length 15.
fn worst() -> Vec<u8> {
    let mut s = vec![0u8; SIZE];
    s[..8].copy_from_slice(b"PKIN\x05\x00\x01\x00");
    s[8..12].copy_from_slice(&((SIZE - 16) as u32).to_le_bytes());
    s[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    s[16..20].copy_from_slice(&299_999u32.to_le_bytes());
    (s[20], s[21], s[22]) = (48, 15, 15);
    for t in 0..16 {
        // every sound apart from the rest: no value is shared, so every sound line has a and b
        s[24 + 4 * t..26 + 4 * t].copy_from_slice(&(10_000 + t as u16).to_le_bytes());
        s[26 + 4 * t..28 + 4 * t].copy_from_slice(&(20_000 + t as u16).to_le_bytes());
    }
    s[88..92].copy_from_slice(&[0xff, 0xff, 0, 0]); // every sound sampled and dry
    s[92..94].copy_from_slice(&0xffffu16.to_le_bytes());
    s[94] = 16;
    for i in 0..16 {
        s[96 + i] = 15 - (i as u8 % 7);
    }
    s[112..144].copy_from_slice(&[b'N'; 32]);
    s[144..272].copy_from_slice(&[b'C'; 128]);
    let mut pair = 0u32;
    for p in 0..16 {
        let b = &mut s[272 + 1364 * p..272 + 1364 * (p + 1)];
        for t in 0..16 {
            b[2 * t..2 * t + 2].copy_from_slice(&0xffffu16.to_le_bytes());
            b[304 + 2 * t..306 + 2 * t].copy_from_slice(&0xffffu16.to_le_bytes());
            for st in 0..16 {
                b[32 + 16 * t + st] = 99;
                let v = 336 + 4 * (16 * t + st);
                b[v..v + 2].copy_from_slice(&(10_000 + (pair % 50_000) as u16).to_le_bytes());
                b[v + 2..v + 4].copy_from_slice(&(60_000 + (pair / 50_000) as u16).to_le_bytes());
                pair += 1;
            }
        }
        b[288..304].copy_from_slice(&[16; 16]);
        b[1360] = 15;
    }
    let crc = crc32(&s[..SIZE - 4]);
    s[SIZE - 4..].copy_from_slice(&crc.to_le_bytes());
    s
}

#[test]
fn the_worst_text_is_measured_and_text_max_holds_it() {
    let s = worst();
    let text = text_of(&s, 0xffff);
    let b = bound();
    println!("the worst text measured: {} bytes; the bound: {b}; PIB_TEXT_MAX: {TEXT_MAX}", text.len());
    assert!(text.len() <= b, "the bound is a bound");
    assert!(b <= TEXT_MAX, "PIB_TEXT_MAX holds the bound");
    assert!(text.len() > 64 * 1024, "BEAT.md's 64 KiB guess is too little: the reason PIB_TEXT_MAX is larger");
    assert!(b - text.len() < 1024, "and the snapshot comes within a KiB of it: {} bytes short", b - text.len());
    assert_eq!(bytes_of(&text), s, "and it reads back");
    // One byte short of it is refused for room, writing nothing.
    let mut out = vec![0xEEu8; text.len() - 1];
    assert_eq!(write(&s, 0xffff, &mut out), Err(crate::ERR_ROOM));
    assert!(out.iter().all(|b| *b == 0xEE));
}

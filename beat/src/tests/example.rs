//! `docs/BEAT.md`'s worked example, "Four and a fill", against its table of what each line
//! fills in, typed in from the table and not from the reader.

use super::*;

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

#[test]
fn the_worked_example_fills_in_what_beat_md_s_table_says() {
    assert_eq!(EXAMPLE.len(), 428, "BEAT.md counts 428 bytes");
    let s = bytes_of(EXAMPLE);
    assert_eq!(s.len(), 3004, "276 + 2 x 1 364");
    assert_eq!(&s[0..8], b"PKIN\x05\x00\x01\x00", "magic, version 5, kind 1");
    assert_eq!(u32_at(&s, 8), 3004 - 16, "the payload");
    let mut name = [0u8; 32];
    name[..15].copy_from_slice(b"Four and a fill");
    assert_eq!(s[112..144], name, "the name, zero-padded");
    let credit = b"Pocket Instruments contributors, 2026; CC0";
    assert_eq!(&s[144..144 + credit.len()], credit);
    assert!(s[144 + credit.len()..272].iter().all(|b| *b == 0), "the credit, zero-padded");
    assert_eq!(u32_at(&s, 16), 124_000, "tempo 124");
    assert_eq!(s[20], 12, "swing 12");
    assert_eq!(s[21], 12, "volume 12");
    assert_eq!(u32_at(&s, 12), 1, "seed 1");
    assert_eq!(s[22], 0, "no chosen: the kick");
    assert_eq!(s[23], 0, "the hat cut");
    for t in 0..16 {
        assert_eq!((u16_at(&s, 24 + 4 * t), u16_at(&s, 26 + 4 * t)), (30_000, 36_000), "sliders, track {t}");
    }
    assert_eq!(u16_at(&s, 88), 0x0100, "sound 9 sample");
    assert_eq!(u16_at(&s, 90), 0xfeff, "sound 9 dry");
    assert_eq!(u16_at(&s, 92), 0x0003, "patterns 1 and 2 written");
    assert_eq!(s[94], 4, "a chain of four");
    assert_eq!(s[95], 0);
    assert_eq!(&s[96..112], &[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "chain 1 1 1 2");

    let (p1, p2) = (&s[272..272 + 1364], &s[272 + 1364..272 + 2 * 1364]);
    let steps = |p: &[u8], t: usize| u16_at(p, 2 * t);
    let keeps = |p: &[u8], t: usize| u16_at(p, 304 + 2 * t);
    let value = |p: &[u8], t: usize, st: usize| (u16_at(p, 336 + 4 * (16 * t + st)), u16_at(p, 338 + 4 * (16 * t + st)));
    // Pattern 1: kick, hat and clap, every step keeping its letter's pair.
    for (t, bits, pair) in [(0, 0x1111, (32_768, 32_768)), (2, 0x4444, (32_768, 32_768)), (8, 0x1010, (24_000, 40_000))] {
        assert_eq!((steps(p1, t), keeps(p1, t)), (bits, bits), "pattern 1, track {t}");
        for st in 0..16 {
            let want = if bits >> st & 1 == 1 { pair } else { (0, 0) };
            assert_eq!(value(p1, t, st), want, "pattern 1, track {t}, step {}", st + 1);
        }
    }
    for t in [1, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15] {
        assert_eq!((steps(p1, t), keeps(p1, t)), (0, 0), "pattern 1, track {t}");
    }
    assert!(p1[32..288].iter().all(|p| *p == 100), "pattern 1: every probability 100");
    assert!(p1[288..304].iter().all(|e| *e == 0), "pattern 1: the lane empty");
    assert_eq!(&p1[1360..], &[16, 0, 0, 0], "pattern 1: length 16");
    // Pattern 2: the kick and the hat by letter, the snare by x, step 15's 70%, the lane.
    assert_eq!((steps(p2, 0), keeps(p2, 0)), (0x0511, 0x0511));
    assert_eq!((steps(p2, 1), keeps(p2, 1)), (0xf000, 0));
    assert_eq!((steps(p2, 2), keeps(p2, 2)), (0x0044, 0x0044));
    for st in 0..16 {
        assert_eq!(p2[32 + 16 + st], if st == 14 { 70 } else { 100 }, "snare step {}", st + 1);
        assert_eq!(p2[288 + st], if st == 12 || st == 13 { 2 } else { 0 }, "the lane, step {}", st + 1);
        assert_eq!(value(p2, 1, st), (0, 0), "x keeps nothing");
    }
    assert_eq!(&p2[1360..], &[16, 0, 0, 0]);
    assert_eq!(u32_at(&s, 3000), crate::text::crc32(&s[..3000]), "the CRC");
}

#[test]
fn the_worked_example_is_canonical_its_bytes_write_back_to_its_text() {
    let s = bytes_of(EXAMPLE);
    assert_eq!(text_of(&s, 0), EXAMPLE);
}

#[test]
fn own_reads_as_sample_and_is_written_only_for_a_key_that_plays_one() {
    let with_own = EXAMPLE.replace("value a", "own 9\nvalue a");
    let mut snap = vec![0u8; SIZE];
    let r = read(with_own.as_bytes(), &mut snap).unwrap();
    assert_eq!(r.own, 1 << 8, "own 9: bit 8");
    snap.truncate(r.len);
    assert_eq!(snap, bytes_of(EXAMPLE), "own is not in the snapshot: key 9 plays the sample either way");
    assert_eq!(text_of(&snap, 1 << 8), with_own, "written back with own");
    // A key that does not play a sample has no own sample to name.
    assert_eq!(text_of(&snap, 1 << 8 | 1 << 2), with_own);
    // own alone makes a key play its sample.
    let own_only = EXAMPLE.replace("sound 9 sample dry", "sound 9 dry").replace("value a", "own 9\nvalue a");
    assert_eq!(bytes_of(&own_only), bytes_of(EXAMPLE));
    assert_eq!(text_of(&bytes_of(&own_only), 1 << 8), with_own, "and is written with sample, canonical");
}

#[test]
fn a_name_and_a_credit_too_long_are_cut_on_a_character_boundary() {
    // 31 bytes, then a two-byte character: 33 bytes, cut to 31, never inside the character.
    let name = format!("{}é", "a".repeat(31));
    let s = bytes_of(&EXAMPLE.replace("name Four and a fill", &format!("name {name}")));
    assert_eq!(&s[112..143], "a".repeat(31).as_bytes());
    assert_eq!(s[143], 0);
    // 127 bytes and a three-byte character: cut to 127.
    let credit = format!("{}€", "c".repeat(127));
    let s = bytes_of(&EXAMPLE.replace("credit Pocket Instruments contributors, 2026; CC0", &format!("credit {credit}")));
    assert_eq!(&s[144..271], "c".repeat(127).as_bytes());
    assert_eq!(s[271], 0);
    // A cut that lands after a space drops it, as the end of a line does.
    let s = bytes_of(&EXAMPLE.replace("name Four and a fill", &format!("name {} tail", "b".repeat(31))));
    assert_eq!(&s[112..143], "b".repeat(31).as_bytes());
    assert_eq!(text_of(&s, 0).lines().nth(1).unwrap(), format!("name {}", "b".repeat(31)));
    // Exactly 32 bytes of characters fits whole, and a # is the name's.
    let full = "ü".repeat(15) + "#1";
    let s = bytes_of(&EXAMPLE.replace("name Four and a fill", &format!("name {full}")));
    assert_eq!(&s[112..144], full.as_bytes());
    assert_eq!(text_of(&s, 0).lines().nth(1).unwrap(), format!("name {full}"));
}

#[test]
fn any_order_and_spacing_reads_the_same_and_writes_back_canonical() {
    let messy = "pocket-instruments beat 1   # the version\n\
        \n\
        pattern 2 length 16\n\
        3 ..a...a......... # the hat\n\
        fx 14=2   13=2\n\
           2 snare ............xxxx prob 15=70 prob\n";
    // (the trailing `prob` above takes nothing: refused, to show the strictness)
    assert!(refusal(messy).1.contains("prob comes once") || refusal(messy).1.contains("prob takes"));
    let messy = "  pocket-instruments beat 1   # the version\r\n\
        chain 1 1 1 2\n\
        \n\
        pattern 2\n\
        3 ..a...a......... # the hat\n\
        fx 14=2   13=2\n\
           2 snare ............xxxx prob 15=70\n\
        1 a...a...a.a.....\n\
        value b 24000 40000\n\
        value a 32768 32768\n\
        pattern 1 # first\n\
        9 clap ....b.......b...\n\
        3 hat ..a...a...a...a. prob 1=100\n\
        1 a...a...a...a...\n\
        sound 9 sample dry a 30000 b 36000\n\
        sliders 30000 36000\n\
        volume 12\nswing 12\ntempo 124.000\nseed 0001\n\
        credit Pocket Instruments contributors, 2026; CC0   \n\
        name    Four and a fill\n";
    let s = bytes_of(messy);
    assert_eq!(s, bytes_of(EXAMPLE));
    assert_eq!(text_of(&s, 0), EXAMPLE);
}

#[test]
fn tempo_takes_up_to_three_decimals_and_writes_none_it_does_not_need() {
    for (text, mbpm, back) in [("97.5", 97_500, "97.5"), ("97.25", 97_250, "97.25"), ("30", 30_000, "30"), ("300.000", 300_000, "300"), ("33.333", 33_333, "33.333"), ("120.05", 120_050, "120.05")] {
        let s = bytes_of(&format!("pocket-instruments beat 1\ntempo {text}\n"));
        assert_eq!(u32::from_le_bytes(s[16..20].try_into().unwrap()), mbpm, "{text}");
        assert!(text_of(&s, 0).contains(&format!("\ntempo {back}\n")), "{text}");
    }
    // 120 is the default: no line.
    let s = bytes_of("pocket-instruments beat 1\ntempo 120\n");
    assert_eq!(text_of(&s, 0), "pocket-instruments beat 1\nseed 1\n");
}

#[test]
fn the_least_file_is_the_first_line_and_gives_the_defaults() {
    let s = bytes_of("pocket-instruments beat 1");
    assert_eq!(s.len(), 276);
    assert_eq!(u32::from_le_bytes(s[12..16].try_into().unwrap()), 1, "seed 1");
    assert_eq!(u32::from_le_bytes(s[16..20].try_into().unwrap()), 120_000);
    assert_eq!((s[20], s[21], s[22]), (0, 16, 0), "swing 0, volume 16, key 1");
    assert!(s[24..88].chunks(2).all(|v| v == [0x00, 0x80]), "sliders 32768");
    assert_eq!(&s[88..96], &[0, 0, 0xff, 0xff, 0, 0, 1, 0], "all code, all through the effects, none written, a chain of one");
    assert_eq!(text_of(&s, 0), "pocket-instruments beat 1\nseed 1\n");
    // A named pattern with nothing in it is a cleared one: no block (§21).
    assert_eq!(bytes_of("pocket-instruments beat 1\npattern 3\n"), s);
    assert_eq!(bytes_of("pocket-instruments beat 1\npattern 3\n4 openhat ................ prob 2=100\n"), s);
}

#[test]
fn a_row_past_the_length_holds_its_steps_and_off_steps_keep_by_list() {
    let text = "pocket-instruments beat 1\nseed 7\n\npattern 16 length 3\n10 click  *.*.x...........  prob 1=0 2=50 keep 1=1,2 2=3,4 3=65535,0\n";
    let s = bytes_of(text);
    assert_eq!(s.len(), 1640);
    assert_eq!(u16::from_le_bytes([s[92], s[93]]), 0x8000, "pattern 16 written");
    let p = &s[272..];
    assert_eq!(u16::from_le_bytes([p[18], p[19]]), 0b10101, "track 10's steps: 1, 3, 5");
    assert_eq!(u16::from_le_bytes([p[304 + 18], p[305 + 18]]), 0b111, "keeps 1, 2 (off) and 3");
    assert_eq!(&p[32 + 9 * 16..32 + 9 * 16 + 3], &[0, 50, 100]);
    assert_eq!(p[1360], 3);
    // Canonical: the on steps that keep take letters; the off one stays in the keep list; the
    // name column is "click" and one space.
    let want = "pocket-instruments beat 1\nseed 7\nvalue a 1 2\nvalue b 65535 0\n\npattern 16 length 3\n10 click a.b.x........... prob 1=0 2=50 keep 2=3,4\n";
    assert_eq!(text_of(&s, 0), want);
    assert_eq!(bytes_of(want), s);
}

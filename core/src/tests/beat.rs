//! `docs/BEAT.md`, "How the two are kept from drifting": the beat file's reader and writer
//! (`beat/`, through its C ABI, `pib_read` and `pib_write`) with the core as the referee. The
//! core has no parser; these tests only link the beat crate.
//!
//! 1. Every fixture's version 5 snapshot, and every version 1 to 4 snapshot loaded and saved by
//!    the core: `pib_write` then `pib_read` gives the same bytes.
//! 2. Every `.beat` file in the repository: `pib_read`, `pi_load` succeeds, `pi_save` gives
//!    the same bytes, and `pib_write` gives the same text.
//! 3. (The worked example's trace and views: `fixtures.rs`, as every fixture.)
//! 4. A differential fuzz: random edits on an engine, saved, written as text, read back: the
//!    same bytes, every seed.
//!
//! And the cases every host runs again (`target/beat-cases.txt`), with this host's results in
//! `PI_BEAT`'s report: the C++ harness on the static library and Node on `pi_beat.wasm` write
//! the same report, and the Makefile compares the three.

use super::fixtures::{fixtures, hex, kept_snapshot_of, old_snapshots, snapshot_of};
use super::*;
use pi_beat::{ERR_NEWER, ERR_ROOM, ERR_SNAPSHOT, ERR_TEXT, MSG_MAX, TEXT_MAX, pib_read, pib_write};
use std::fmt::Write as _;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

/// xorshift, seeded: a failure names its seed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }
}

/// `pib_read` through the C ABI: the snapshot and `own`, or the code and the message.
pub fn read(text: &[u8]) -> Result<(Vec<u8>, u32), (i32, String)> {
    let mut snap = vec![0u8; SNAP5_MAX];
    let mut own = 0u32;
    let mut msg = [0u8; MSG_MAX];
    let n = unsafe { pib_read(text.as_ptr(), text.len() as u32, snap.as_mut_ptr(), SNAP5_MAX as u32, &mut own, msg.as_mut_ptr(), MSG_MAX as u32) };
    if n < 0 {
        let end = msg.iter().position(|b| *b == 0).expect("the message ends with a zero byte");
        return Err((n, String::from_utf8(msg[..end].to_vec()).expect("UTF-8")));
    }
    snap.truncate(n as usize);
    Ok((snap, own))
}

/// `pib_write` through the C ABI.
pub fn write(snap: &[u8], own: u32) -> Result<Vec<u8>, i32> {
    let mut text = vec![0u8; TEXT_MAX];
    let n = unsafe { pib_write(snap.as_ptr(), snap.len() as u32, own, text.as_mut_ptr(), TEXT_MAX as u32) };
    if n < 0 {
        return Err(n);
    }
    text.truncate(n as usize);
    Ok(text)
}

/// A fixture's `beat`, read by the beat reader (`fixtures.rs`).
pub fn read_fixture_beat(file: &str) -> Vec<u8> {
    let text = std::fs::read(format!("{ROOT}/fixtures/{file}")).unwrap();
    read(&text).unwrap_or_else(|(_, m)| panic!("{file}: {m}")).0
}

/// The core as referee: the bytes load, and save again as the same bytes.
fn core_saves_the_same(bytes: &[u8], label: &str) {
    let mut h = Host::new(48_000, 99);
    assert_eq!(h.load(bytes), OK, "{label}: pi_load");
    assert_eq!(h.snap(), bytes, "{label}: pi_save gives the same bytes");
}

/// Through text and back: the same bytes, the same text again, and the core agrees.
fn round_trip(s: &[u8], own: u32, label: &str) -> Vec<u8> {
    let text = write(s, own).unwrap_or_else(|e| panic!("{label}: pib_write refused the core's own snapshot: {e}"));
    let (back, got_own) = read(&text).unwrap_or_else(|(_, m)| panic!("{label}: {m}\n{}", String::from_utf8_lossy(&text)));
    assert_eq!(back, s, "{label}: text and back, the same bytes\n{}", String::from_utf8_lossy(&text));
    assert_eq!(got_own, own & u16::from_le_bytes([s[88], s[89]]) as u32, "{label}: own, for the keys that play a sample");
    assert_eq!(write(&back, got_own).unwrap(), text, "{label}: the same text again");
    core_saves_the_same(&back, label);
    text
}

/// Every fixture's version 5 snapshots (as an edit leaves them and as keys do), and every old
/// snapshot as the core loads and saves it: (label, bytes).
fn core_snapshots() -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for (name, fx) in fixtures() {
        if fx.get("beat").is_some() {
            continue; // its snapshot is its beat file's, below
        }
        out.push((format!("{name}-snapshot"), snapshot_of(&fx, 48_000)));
        out.push((format!("{name}-kept"), kept_snapshot_of(&fx, 48_000)));
    }
    for v in 1..=4 {
        for (name, bytes) in old_snapshots(&format!("snapshots-v{v}.txt")) {
            assert_eq!(write(&bytes, 0), Err(ERR_SNAPSHOT), "{name} v{v}: an old version is the core's to load first (BEAT.md)");
            let mut h = Host::new(48_000, 5);
            assert_eq!(h.load(&bytes), OK);
            out.push((format!("{name}-v{v}"), h.snap()));
        }
    }
    out
}

#[test]
fn every_fixture_s_snapshot_and_every_old_one_goes_through_text_and_back_to_the_same_bytes() {
    let all = core_snapshots();
    assert_eq!(all.len(), 2 * 17 + 7 + 7 + 9 + 13, "two a fixture, and every old snapshot");
    for (label, s) in &all {
        round_trip(s, 0, label);
    }
}

/// Every `.beat` file in the repository: the fixtures', and the bank's when it comes (issue 134).
fn beat_files() -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for dir in ["fixtures", "beats"] {
        let Ok(entries) = std::fs::read_dir(format!("{ROOT}/{dir}")) else { continue };
        for e in entries {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "beat") {
                out.push((format!("{dir}/{}", p.file_name().unwrap().to_str().unwrap()), std::fs::read(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_beat_file_reads_loads_saves_the_same_bytes_and_writes_back_the_same_text() {
    let files = beat_files();
    assert!(files.iter().any(|(n, _)| n == "fixtures/beat-four-and-a-fill.beat"));
    for (name, text) in &files {
        let (bytes, own) = read(text).unwrap_or_else(|(_, m)| panic!("{name}: {m}"));
        core_saves_the_same(&bytes, name);
        assert_eq!(write(&bytes, own).unwrap(), *text, "{name}: pib_write gives the same text, byte for byte");
    }
}

#[test]
fn the_worked_example_is_3004_bytes_through_pib_read_pi_load_and_pi_save_and_its_own_text_again() {
    let text = std::fs::read(format!("{ROOT}/fixtures/beat-four-and-a-fill.beat")).unwrap();
    assert_eq!(text.len(), 428);
    let (bytes, own) = read(&text).unwrap();
    assert_eq!((bytes.len(), own), (3004, 0));
    let mut h = Host::new(44_100, 7);
    assert_eq!(h.load(&bytes), OK);
    let saved = h.snap();
    assert_eq!(saved, bytes, "pi_save: the same 3 004 bytes");
    assert_eq!(write(&saved, 0).unwrap(), text, "and pib_write: the same text, byte for byte");
    // What the engine shows of it (§14, §21): the beat's settings, pattern 1 of a chain of four.
    let v = h.view();
    assert_eq!((v[26], v[27], u32::from_le_bytes(v[4..8].try_into().unwrap()), v[29]), (0, 12, 124_000, 12), "key 1, volume 12, tempo 124, swing 12");
    assert_eq!((v[64], v[66], &v[72..74]), (0, 4, &[0x00, 0x10][..]), "pattern 1, a chain of four: 1 1 1 2");
}

/// One random edit, of every kind a snapshot holds. A name and a credit come only from a load,
/// so some seeds load a beat file that names them first.
fn random_cmd(r: &mut Rng) -> Vec<Cmd> {
    let t = r.below(16) as u8;
    let s = r.below(16) as u8;
    let v = r.next();
    let (down, up) = (|k| cmd(CMD_KEY_DOWN, k, 0, 0), |k| cmd(CMD_KEY_UP, k, 0, 0));
    match r.below(20) {
        0..=4 => vec![cmd(CMD_STEP, t, s, r.below(2))],
        5 => vec![cmd(CMD_PROB, t, s, r.below(120))],
        6 | 7 => vec![cmd(CMD_KEEP, t, s, if r.below(2) == 0 { v } else { [0x8000_8000, 0x1234_5678, 0xffff_0000][r.below(3) as usize] })],
        8 => vec![cmd(CMD_FX, s, 0, r.below(18))],
        9 => vec![cmd(CMD_FXSOUND, t, 0, r.below(2))],
        10 => vec![cmd(CMD_SWING, 0, 0, r.below(60))],
        11 => vec![cmd(CMD_LENGTH, 0, 0, r.below(18))],
        12 => vec![cmd(CMD_PATTERN, 0, 0, r.below(17))],
        13 => vec![cmd(CMD_CHAIN, 0, 0, r.below(17))],
        14 => vec![cmd(CMD_COPY, 0, 0, r.below(17))],
        15 => vec![cmd(CMD_SOUND, t, r.below(2) as u8, if r.below(2) == 0 { v & 0xffff } else { 32768 })],
        16 => vec![cmd(CMD_VOICE, t, 0, r.below(2))],
        17 => vec![cmd(CMD_TEMPO, 0, 0, 20_000 + r.below(300_000))],
        18 => vec![down(18), down(t), up(t), up(18)], // bpm + a key: the volume
        _ => vec![down(16), down(t), up(t), up(16)],  // sound + a key: the chosen sound
    }
}

/// The fuzz's seeds: each an engine given 1 to 400 random edits, saved by the core.
const FUZZ_SEEDS: u64 = 600;

fn fuzz_snapshot(seed: u64) -> Vec<u8> {
    let mut r = Rng::new(seed);
    let mut h = Host::new(48_000, r.next());
    if r.below(4) == 0 {
        let name: String = ["Fuzz ", "é", "€", "#", "𝄞", " x"].iter().take(1 + r.below(6) as usize).copied().collect();
        let text = format!("pocket-instruments beat 1\nname {name}\ncredit seed {seed}; CC0\n");
        assert_eq!(h.load(&read(text.as_bytes()).unwrap().0), OK);
    }
    let edits = 1 + r.below(400);
    let mut cmds = Vec::new();
    for _ in 0..edits {
        cmds.extend(random_cmd(&mut r));
    }
    for chunk in cmds.chunks(200) {
        h.block(1, chunk);
    }
    h.snap()
}

#[test]
fn a_differential_fuzz_random_edits_saved_written_and_read_back_are_the_same_bytes_every_seed() {
    let mut patterns = 0;
    for seed in 1..=FUZZ_SEEDS {
        let s = fuzz_snapshot(seed);
        patterns += u16::from_le_bytes([s[92], s[93]]).count_ones();
        round_trip(&s, seed as u32 * 0x9E37, &format!("fuzz seed {seed}"));
    }
    assert!(patterns as u64 > FUZZ_SEEDS * 2, "the edits reached many patterns: {patterns}");
}

/// Texts every host reads: each beat file, the refusals the issue names and more, and good
/// text edited at random (as `beat/`'s own fuzz edits it).
fn text_cases() -> Vec<(String, Vec<u8>)> {
    let mut out = beat_files();
    let h = "pocket-instruments beat 1\n";
    let bad = [
        ("newer", "pocket-instruments beat 2\n".to_string()),
        ("unknown-word", format!("{h}bpm 120\n")),
        ("fifteen-steps", format!("{h}\npattern 1\n1 kick x..............\n")),
        ("wrong-name", format!("{h}\npattern 1\n1 snare x...............\n")),
        ("value-out-of-range", format!("{h}value a 70000 1\n")),
        ("letter-never-given", format!("{h}\npattern 1\n1 kick ..c.............\n")),
        ("control-character", format!("{h}name a\u{7f}b\n")),
        ("empty", String::new()),
        ("own-code", format!("{h}sound 3 code\nown 3\n")),
        ("quote-cut", format!("{h}chosen {}\n", "é".repeat(20))),
    ];
    out.extend(bad.into_iter().map(|(n, t)| (format!("refuse-{n}"), t.into_bytes())));
    out.push(("refuse-not-utf8".to_string(), [h.as_bytes(), b"name a\xffb\n"].concat()));
    let example = std::fs::read(format!("{ROOT}/fixtures/beat-four-and-a-fill.beat")).unwrap();
    const ALPHABET: &[u8] = b" \n.x*abwAW#0123456789=,\t\xc3\xa9";
    for seed in 1..=300u64 {
        let mut r = Rng::new(seed);
        let mut t = if seed % 2 == 0 { example.clone() } else { write(&fuzz_snapshot(seed), 0).unwrap() };
        for _ in 0..1 + r.below(3) {
            let at = r.below(t.len() as u32 + 1) as usize;
            match r.below(3) {
                0 if at < t.len() => {
                    t.remove(at);
                }
                1 if at < t.len() => t[at] = ALPHABET[r.below(ALPHABET.len() as u32) as usize],
                _ => t.insert(at, ALPHABET[r.below(ALPHABET.len() as u32) as usize]),
            }
        }
        out.push((format!("edited-{seed}"), t));
    }
    out
}

/// FNV-1a, 32 bits: a hash of a case's bytes for the report. (Not the CRC: a snapshot's CRC
/// over its own bytes, CRC included, is one constant for every snapshot.)
fn fnv(b: &[u8]) -> u32 {
    b.iter().fold(0x811C_9DC5u32, |h, x| (h ^ *x as u32).wrapping_mul(0x0100_0193))
}

/// One line of the hosts' report for a snapshot case: what `pib_write` gave, and what
/// `pib_read` gave back (the test asserts it is the same bytes; the report holds every host to
/// the same text).
fn snap_line(label: &str, s: &[u8]) -> String {
    let text = write(s, 0x0101).unwrap();
    let (back, own) = read(&text).unwrap();
    format!("{label} snap {} text {} {:08x} back {} {:08x} own {own}\n", s.len(), text.len(), fnv(&text), back.len(), fnv(&back))
}

/// For a text case: refused with its code and message, or read, loaded and saved by the core,
/// and written back.
fn text_line(label: &str, t: &[u8]) -> String {
    match read(t) {
        Err((code, m)) => format!("{label} refused {code} {m}\n"),
        Ok((snap, own)) => {
            core_saves_the_same(&snap, label);
            let text = write(&snap, own).unwrap();
            format!("{label} read {} {:08x} own {own} text {} {:08x}\n", snap.len(), fnv(&snap), text.len(), fnv(&text))
        }
    }
}

#[test]
fn the_cases_every_host_runs_and_this_host_s_report() {
    let mut cases = String::from("# The beat reader's and writer's cases (core/src/tests/beat.rs writes them; each host runs them).\n");
    let mut report = String::new();
    let mut snaps: Vec<(String, Vec<u8>)> = core_snapshots();
    snaps.extend((1..=FUZZ_SEEDS).step_by(10).map(|seed| (format!("fuzz-{seed}"), fuzz_snapshot(seed))));
    for (label, s) in &snaps {
        writeln!(cases, "snap {label} {}", hex(s)).unwrap();
        report.push_str(&snap_line(label, s));
    }
    let (mut refused, mut taken) = (0, 0);
    for (label, t) in &text_cases() {
        writeln!(cases, "text {label} {}", hex(t)).unwrap();
        let line = text_line(label, t);
        if line.contains(" refused ") { refused += 1 } else { taken += 1 }
        report.push_str(&line);
    }
    assert!(refused > 50 && taken > 20, "both kinds reached: {refused} refused, {taken} read");
    assert!(report.contains(&format!("refuse-newer refused {ERR_NEWER} line 1: this is beat 2; this build reads beat 1\n")));
    assert!(report.contains(&format!("refuse-fifteen-steps refused {ERR_TEXT} line 4, column 8: a row takes sixteen steps; this has 15\n")));
    if let Ok(path) = std::env::var("PI_BEAT") {
        std::fs::write(&path, report).unwrap();
        std::fs::write(std::path::Path::new(&path).with_file_name("beat-cases.txt"), cases).unwrap();
    }
}

#[test]
fn the_c_abi_refuses_without_writing_and_says_how() {
    let text = std::fs::read(format!("{ROOT}/fixtures/beat-four-and-a-fill.beat")).unwrap();
    let mut snap = vec![0xEEu8; 3003];
    let mut msg = [0u8; MSG_MAX];
    let n = unsafe { pib_read(text.as_ptr(), text.len() as u32, snap.as_mut_ptr(), 3003, core::ptr::null_mut(), msg.as_mut_ptr(), MSG_MAX as u32) };
    assert_eq!(n, ERR_ROOM);
    assert!(snap.iter().all(|b| *b == 0xEE), "nothing written");
    let (bytes, _) = read(&text).unwrap();
    let mut out = vec![0xEEu8; 427];
    assert_eq!(unsafe { pib_write(bytes.as_ptr(), bytes.len() as u32, 0, out.as_mut_ptr(), 427) }, ERR_ROOM);
    assert!(out.iter().all(|b| *b == 0xEE), "nothing written");
    assert_eq!(unsafe { pib_write(core::ptr::null(), 0, 0, out.as_mut_ptr(), 427) }, ERR_SNAPSHOT);
    // Bytes after the CRC are the host's, as for pi_load.
    let mut longer = bytes.clone();
    longer.extend_from_slice(&[1, 2, 3]);
    assert_eq!(write(&longer, 0).unwrap(), text);
    // Every cut short of the whole is refused.
    for n in 0..bytes.len() {
        assert_eq!(write(&bytes[..n], 0), Err(ERR_SNAPSHOT), "cut to {n}");
    }
}


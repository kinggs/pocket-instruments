//! The bank and the beat a new player opens on, in the terminal (docs/BANK.md; D141, D142,
//! proposed for Kenny to rule; issue 164). The bank compiled in is `beats/` as it is; with no
//! file at the beat's path the bank's first beat is loaded, and with one there that file is,
//! whatever it holds; `:bank <n>` and `:new` keep the beat they replace in rhythm.previous.beat,
//! and `:e` brings it back; a bank file that does not read leaves the instrument empty and says
//! why. (That the default plays `fixtures/beat-amen-break.json`'s trace is in fixtures.rs, beside
//! the runner it uses.)
//!
//! SPDX-License-Identifier: MIT
use pi_term::audio::{self, Backend, Shared};
use pi_term::bank::{self, Bank};
use pi_term::beat;
use pi_term::ffi::*;
use pi_term::settings;
use std::path::{Path, PathBuf};

fn repo(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(file)
}

fn dir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// `pib_read` of the file on `bank.txt`'s first line, read from the repository now: what a
/// new player must open on, whichever beat that line names.
fn the_default_from_disk() -> Vec<u8> {
    let list = std::fs::read_to_string(repo("beats/bank.txt")).unwrap();
    let first = list.lines().next().unwrap();
    beat::from_file(&std::fs::read(repo(&format!("beats/{first}"))).unwrap()).unwrap().snapshot
}

/// The start-up path main.rs takes: the options, the beat's path, `open_or_default`.
fn start_with(argv: &[&str]) -> (Option<PathBuf>, beat::Opened) {
    let args = settings::parse_args(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap();
    let s = settings::resolve(&args, &settings::Config::default(), false);
    let path = s.beat.clone().or_else(beat::default_path);
    let o = beat::open_or_default(path.as_deref(), &Bank::compiled());
    (path, o)
}

#[test]
fn the_bank_compiled_in_is_beats_as_it_is() {
    let b = Bank::compiled();
    let list = std::fs::read_to_string(repo("beats/bank.txt")).unwrap();
    assert_eq!(b.list, list.as_bytes(), "bank.txt");
    assert_eq!(b.order(), list.lines().collect::<Vec<_>>());
    for (f, text) in b.files {
        assert_eq!(*text, std::fs::read(repo(&format!("beats/{f}"))).unwrap().as_slice(), "{f}");
    }
    let menu = b.menu();
    assert_eq!(menu.len(), b.order().len());
    assert!(menu.iter().all(|l| !l.contains('(')), "every beat reads: {menu:?}");
    assert_eq!(b.default_beat().unwrap().1.snapshot, the_default_from_disk());
}

#[test]
fn the_empty_beat_is_the_one_pi_init_builds() {
    let empty = beat::from_file(bank::EMPTY).unwrap().snapshot;
    assert_eq!(empty, Engine::new(48_000, 1).save(), "the host's seed, 1");
}

#[test]
fn with_no_file_at_the_beat_path_a_first_start_opens_the_bank_s_first_beat() {
    let d = dir("bank-first");
    let path = d.join("rhythm.beat");
    let (got, o) = start_with(&["rhythm", "--beat", path.to_str().unwrap()]);
    assert_eq!(got.as_deref(), Some(path.as_path()));
    assert_eq!(o.snapshot.as_deref(), Some(the_default_from_disk().as_slice()), "pib_read of the file, byte for byte");
    assert!(o.writable, "written there at the next save");
    let note = o.note.unwrap();
    assert!(note.starts_with("a new beat: ") && note.chars().count() <= 78, "{note}");
    assert!(!path.exists(), "nothing is written until a save");
}

#[test]
fn a_beat_file_that_is_there_always_wins_over_the_default() {
    let d = dir("bank-saved");
    let path = d.join("rhythm.beat");
    let fill = std::fs::read(repo("fixtures/beat-four-and-a-fill.beat")).unwrap();
    // A text beat.
    std::fs::write(&path, &fill).unwrap();
    let (_, o) = start_with(&["--beat", path.to_str().unwrap()]);
    assert_eq!(o.snapshot.unwrap(), beat::from_file(&fill).unwrap().snapshot);
    assert!(o.note.is_none());
    // An old binary one (version 4).
    let v4 = std::fs::read_to_string(repo("fixtures/snapshots-v4.txt")).unwrap();
    let hex = v4.lines().find(|l| l.starts_with("straight-sixteenths ")).unwrap().split_once(' ').unwrap().1;
    let bytes: Vec<u8> = (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect();
    std::fs::write(&path, &bytes).unwrap();
    let (_, o) = start_with(&["--beat", path.to_str().unwrap()]);
    assert_eq!(o.snapshot.unwrap(), bytes);
    // A file that does not read: left alone and empty, never the default (D142: it may be a newer
    // beat from a newer build, and the default would be saved over it).
    for broken in [&b"pocket-instruments beat 2\n"[..], b"pocket-instruments beat 1\nwobble 3\n", b""] {
        std::fs::write(&path, broken).unwrap();
        let (_, o) = start_with(&["--beat", path.to_str().unwrap()]);
        assert_eq!(o.snapshot, None, "{:?}", String::from_utf8_lossy(broken));
        assert!(!o.writable && o.note.unwrap().starts_with("rhythm.beat left alone: "));
        assert_eq!(std::fs::read(&path).unwrap(), broken);
    }
}

#[test]
fn a_bank_beat_that_does_not_read_leaves_the_instrument_empty_and_says_why() {
    let d = dir("bank-broken");
    let path = d.join("rhythm.beat");
    let cases: [(&[u8], &[(&str, &[u8])], &str); 4] = [
        (b"amen-break.beat\n", &[("amen-break.beat", b"pocket-instruments beat 1\ntempo 999\n")], "amen-break.beat: line 2"),
        (b"gone.beat\n", &[], "gone.beat is not in this build's bank"),
        (b"", &[], "the bank lists no beat"),
        (b"two.beat\none.beat\n", &[("one.beat", bank::EMPTY), ("two.beat", b"not a beat\n")], "two.beat: "),
    ];
    for (list, files, why) in cases {
        let b = Bank { list, files };
        let o = beat::open_or_default(Some(&path), &b);
        assert_eq!(o.snapshot, None, "{why}");
        assert!(o.writable, "{why}: nothing is there to leave alone");
        let note = o.note.unwrap();
        assert!(note.starts_with("started empty: the bank's first beat did not open: ") && note.contains(why), "{note}");
        assert!(b.read(1).is_err() && b.menu().first().is_none_or(|l| l.contains('(')), "{why}: the menu says so too");
    }
    assert_eq!(Bank { list: b"a.beat\n", files: &[("a.beat", bank::EMPTY)] }.read(2).unwrap_err(), "the bank has beats 1 to 1, not 2");
}

/// A host on its own timer (`--audio none`), as main.rs starts it, with `start` loaded.
fn running(start: Option<&[u8]>) -> (audio::Output, std::sync::Arc<Shared>) {
    let shared = Shared::new();
    let (out, _) = audio::start(Backend::None, 1, shared.clone(), |e| {
        if let Some(b) = start {
            assert_eq!(e.load(b), PI_OK);
        }
    })
    .unwrap();
    (out, shared)
}

#[test]
fn a_bank_load_keeps_the_beat_it_replaced_and_e_brings_it_back() {
    let d = dir("bank-load");
    let path = d.join("rhythm.beat");
    let prev = bank::previous_path(&path);
    assert_eq!(prev, d.join("rhythm.previous.beat"));
    assert_eq!(bank::previous_path(&d.join("mine.beat")), d.join("mine.previous.beat"));
    let fill = std::fs::read(repo("fixtures/beat-four-and-a-fill.beat")).unwrap();
    let mine = beat::from_file(&fill).unwrap().snapshot;
    let (_out, shared) = running(Some(&mine));
    let b = Bank::compiled();
    // :bank 1: the default; the player's beat kept, as text.
    bank::replace(&shared, &prev, b.read(1).unwrap().1.snapshot).unwrap();
    assert_eq!(bank::snapshot_now(&shared).unwrap(), the_default_from_disk());
    assert_eq!(std::fs::read(&prev).unwrap(), fill, "the replaced beat, its own text");
    // :e rhythm.previous.beat: the second gesture, through :e's own pair.
    assert_eq!(bank::load(&shared, beat::from_file(&std::fs::read(&prev).unwrap()).unwrap().snapshot), Some(PI_OK));
    assert_eq!(bank::snapshot_now(&shared).unwrap(), mine, "the player's beat back");
    // :bank 3, then :new: each keeps what it replaced, once.
    bank::replace(&shared, &prev, b.read(3).unwrap().1.snapshot).unwrap();
    assert_eq!(std::fs::read(&prev).unwrap(), fill);
    bank::replace(&shared, &prev, beat::from_file(bank::EMPTY).unwrap().snapshot).unwrap();
    assert_eq!(beat::from_file(&std::fs::read(&prev).unwrap()).unwrap().snapshot, b.read(3).unwrap().1.snapshot, "the third beat, kept");
    let v = shared.view();
    assert_eq!((pi_term::ffi::u16_at(&v, PI_VIEW_USED), v[PI_VIEW_CHAIN_LEN]), (0, 1), ":new: nothing used, the chain pattern 1 alone");
    assert_eq!(bank::snapshot_now(&shared).unwrap(), Engine::new(48_000, 1).save(), ":new is pi_init's beat");
    // Nowhere to keep it: nothing is loaded.
    let before = bank::snapshot_now(&shared).unwrap();
    let nowhere = d.join("no-such-folder").join("x").join("rhythm.previous.beat");
    std::fs::write(d.join("no-such-folder"), b"a file, not a folder").unwrap();
    assert!(bank::replace(&shared, &nowhere, mine.clone()).unwrap_err().starts_with("not loaded: "));
    assert_eq!(bank::snapshot_now(&shared).unwrap(), before);
}

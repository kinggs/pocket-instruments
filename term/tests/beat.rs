//! The terminal host's beat file (docs/TERMINAL.md, "The beat"; `docs/BEAT.md`, "The terminal
//! host's beat"; D134 item 5, D136 item 8; issue 143): an old binary `rhythm.beat` still loads and
//! is written back; a file the host cannot read is left alone; `:e` reads through the same pair.
//!
//! SPDX-License-Identifier: MIT
use pi_term::beat;
use pi_term::ffi::*;
use std::path::PathBuf;

/// Every version 4 snapshot the ABI 15 core wrote before version 5 was built.
fn v4_snapshots() -> Vec<(String, Vec<u8>)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/snapshots-v4.txt");
    let text = std::fs::read_to_string(path).unwrap();
    let all: Vec<_> = text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| {
            let (name, hex) = l.split_once(' ').unwrap();
            let bytes = (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect();
            (name.to_string(), bytes)
        })
        .collect();
    assert!(all.len() >= 10, "expected the version 4 snapshots");
    all
}

/// A folder of its own for each test, under the build's target folder.
fn dir(name: &str) -> PathBuf {
    let d = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn an_old_binary_rhythm_beat_loads_and_is_written_back_as_the_same_beat() {
    let d = dir("beat-v4");
    let path = d.join("rhythm.beat");
    for (name, v4) in v4_snapshots() {
        assert_eq!(u16::from_le_bytes([v4[4], v4[5]]), 4, "{name} is version 4");
        std::fs::write(&path, &v4).unwrap();
        let o = beat::open(&path);
        assert!(o.writable && o.note.is_none(), "{name}");
        let mut e = Engine::new(48000, 1);
        assert_eq!(e.load(o.snapshot.as_ref().unwrap()), PI_OK, "{name} loads");
        // What the host writes at its next save: the engine's save through `to_file`.
        let out = beat::to_file(&e.save()).unwrap();
        beat::write(&path, &out).unwrap();
        let back = std::fs::read(&path).unwrap();
        assert!(back.starts_with(beat::MAGIC));
        if beat::WRITES_TEXT {
            panic!("the text form is written: this test then reads it back through the crate");
        }
        assert_eq!(u16::from_le_bytes([back[4], back[5]]), 5, "{name}: written as the core saves it today");
        // Read again, it is the same beat: the same bytes saved, the same trace-bearing state.
        let again = beat::open(&path);
        let mut f = Engine::new(48000, 99);
        assert_eq!(f.load(again.snapshot.as_ref().unwrap()), PI_OK);
        assert_eq!(f.save(), e.save(), "{name}: the same beat after a write and a read");
        assert_eq!(f.view()[PI_VIEW_PATTERN..], e.view()[PI_VIEW_PATTERN..], "{name}: pattern 1 alone, as version 4 loads");
    }
}

#[test]
fn a_missing_file_is_written_when_the_beat_changes() {
    let d = dir("beat-missing");
    let o = beat::open(&d.join("rhythm.beat"));
    assert_eq!(o, beat::Opened { snapshot: None, writable: true, note: None });
}

#[test]
fn a_beat_this_build_cannot_read_is_left_alone() {
    let d = dir("beat-unread");
    // The text form (docs/BEAT.md): waits for the beat reader, issue 140.
    let text = b"pocket-instruments beat 1\nname Four and a fill\ntempo 124\n\npattern 1\n1  kick  a...a...a...a...\n";
    let path = d.join("rhythm.beat");
    std::fs::write(&path, text).unwrap();
    let o = beat::open(&path);
    assert!(o.snapshot.is_none() && !o.writable, "not read, so not written over");
    let note = o.note.unwrap();
    assert!(note.contains("rhythm.beat") && note.contains("text") && note.contains("140") && note.contains("left alone"), "{note}");
    assert_eq!(std::fs::read(&path).unwrap(), text, "the file is as it was");
    // Anything else that is not a beat, and a file that cannot be read, the same.
    std::fs::write(&path, b"hello").unwrap();
    assert!(!beat::open(&path).writable);
    let folder = d.join("a-folder.beat");
    std::fs::create_dir_all(&folder).unwrap();
    assert!(!beat::open(&folder).writable);
}

#[test]
fn colon_e_and_colon_w_go_through_the_same_pair() {
    // `:e` gives `from_file` the file's bytes; `:w` writes `to_file` of the engine's save.
    let (_, v4) = v4_snapshots().into_iter().next().unwrap();
    assert_eq!(beat::from_file(&v4).unwrap(), v4, "a snapshot is given to the engine as it is");
    assert!(beat::from_file(b"pocket-instruments beat 1\n").unwrap_err().contains("140"));
    assert!(beat::from_file(b"").is_err());
    let mut e = Engine::new(48000, 1);
    assert_eq!(e.load(&v4), PI_OK);
    e.process(1, &[Cmd::new(PI_CMD_PATTERN, 0, 0, 2), Cmd::new(PI_CMD_STEP, 3, 7, 1)], &mut [Event::default(); 8], None, PI_AUDIO_NONE);
    let d = dir("beat-colon");
    let path = d.join("mine.beat");
    beat::write(&path, &beat::to_file(&e.save()).unwrap()).unwrap();
    let mut f = Engine::new(48000, 1);
    assert_eq!(f.load(&beat::from_file(&std::fs::read(&path).unwrap()).unwrap()), PI_OK);
    assert_eq!(f.save(), e.save(), "pattern 3 with its step, and pattern 1 from the old file");
    assert_eq!(f.view()[PI_VIEW_PATTERN], 2);
    assert!(beat::to_file(b"not a snapshot").is_err());
}

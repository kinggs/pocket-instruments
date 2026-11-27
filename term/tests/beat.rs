//! The terminal host's beat file (docs/TERMINAL.md, "The beat"; `docs/BEAT.md`, "The terminal
//! host's beat"; D134 item 5, D136 item 8; issue 143): the saved beat is text, read and written
//! by the beat crate; an old binary `rhythm.beat` still loads and becomes text at its next save;
//! a file the host cannot read is left alone, with the crate's line and column said.
//!
//! SPDX-License-Identifier: MIT
use pi_term::beat;
use pi_term::ffi::*;
use std::path::PathBuf;

fn repo(file: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(file)
}

/// Every version 4 snapshot the ABI 15 core wrote before version 5 was built.
fn v4_snapshots() -> Vec<(String, Vec<u8>)> {
    let text = std::fs::read_to_string(repo("fixtures/snapshots-v4.txt")).unwrap();
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

fn loaded(snapshot: &[u8]) -> Engine {
    let mut e = Engine::new(48000, 7);
    assert_eq!(e.load(snapshot), PI_OK);
    e
}

#[test]
fn a_text_beat_is_read_and_written_back_as_the_same_text() {
    // docs/BEAT.md's worked example, canonical: read, loaded, saved by the core, written again.
    let text = std::fs::read(repo("fixtures/beat-four-and-a-fill.beat")).unwrap();
    let d = dir("beat-text");
    let path = d.join("rhythm.beat");
    std::fs::write(&path, &text).unwrap();
    let o = beat::open(&path);
    assert!(o.writable && o.note.is_none(), "{:?}", o.note);
    let e = loaded(o.snapshot.as_ref().unwrap());
    assert_eq!(e.view()[PI_VIEW_CHAIN_LEN], 4, "chain 1 1 1 2");
    assert!(beat::WRITES_TEXT);
    let out = beat::to_file(&e.save()).unwrap();
    assert_eq!(String::from_utf8(out.clone()).unwrap(), String::from_utf8(text).unwrap(), "the same text");
    beat::write(&path, &out).unwrap();
    assert_eq!(loaded(&beat::open(&path).snapshot.unwrap()).save(), e.save());
}

#[test]
fn an_old_binary_rhythm_beat_loads_and_becomes_text_holding_the_same_beat() {
    let d = dir("beat-v4");
    let path = d.join("rhythm.beat");
    for (name, v4) in v4_snapshots() {
        assert_eq!(u16::from_le_bytes([v4[4], v4[5]]), 4, "{name} is version 4");
        std::fs::write(&path, &v4).unwrap();
        let o = beat::open(&path);
        assert!(o.writable, "{name}");
        let note = o.note.unwrap();
        assert!(note.contains("rhythm.beat") && note.contains("binary") && note.contains("text"), "{note}");
        let e = loaded(o.snapshot.as_ref().unwrap());
        // What the host writes at its next save: the engine's save through `to_file`, as text.
        beat::write(&path, &beat::to_file(&e.save()).unwrap()).unwrap();
        let back = std::fs::read(&path).unwrap();
        assert!(back.starts_with(beat::TEXT_FIRST), "{name}: written as text");
        // Read again, it is the same beat: the same bytes saved, and the same view.
        let again = beat::open(&path);
        assert!(again.writable && again.note.is_none(), "{name}: {:?}", again.note);
        let f = loaded(again.snapshot.as_ref().unwrap());
        assert_eq!(f.save(), e.save(), "{name}: the same beat after text and back");
        assert_eq!(f.view(), e.view(), "{name}: the same engine state, as the view shows it");
    }
}

#[test]
fn a_missing_file_is_written_when_the_beat_changes() {
    let d = dir("beat-missing");
    let o = beat::open(&d.join("rhythm.beat"));
    assert_eq!(o, beat::Opened { snapshot: None, writable: true, note: None });
}

#[test]
fn a_text_beat_the_crate_refuses_is_left_alone_and_its_line_and_column_said() {
    let d = dir("beat-refused");
    let path = d.join("rhythm.beat");
    // A row of fifteen steps: the crate refuses it with its line.
    let text = b"pocket-instruments beat 1\ntempo 124\n\npattern 1\n1  kick  a...a...a...a..\n";
    std::fs::write(&path, text).unwrap();
    let o = beat::open(&path);
    assert!(o.snapshot.is_none() && !o.writable, "not read, so not written over");
    let note = o.note.unwrap();
    assert!(note.starts_with("rhythm.beat left alone: line 5, column "), "{note}");
    assert_eq!(std::fs::read(&path).unwrap(), text, "the file is as it was");
    // A newer beat, refused by name.
    std::fs::write(&path, b"pocket-instruments beat 2\n").unwrap();
    let note = beat::open(&path).note.unwrap();
    assert!(note.contains("line 1") && note.contains("beat 2"), "{note}");
    // Anything else that is not a beat, and a file that cannot be read, the same.
    std::fs::write(&path, b"hello").unwrap();
    assert!(!beat::open(&path).writable);
    std::fs::write(&path, b"").unwrap();
    assert!(!beat::open(&path).writable);
    let folder = d.join("a-folder.beat");
    std::fs::create_dir_all(&folder).unwrap();
    assert!(!beat::open(&folder).writable);
}

#[test]
fn colon_e_and_colon_w_on_text() {
    // `:e` gives `from_file` the file's bytes; `:w` writes `to_file` of the engine's save.
    let text = std::fs::read_to_string(repo("fixtures/beat-four-and-a-fill.beat")).unwrap();
    // With an `own` line: key 9 played its writer's own sample (D34).
    let mine = text.replace("value a", "own 9\nvalue a");
    let r = beat::from_file(mine.as_bytes()).unwrap();
    assert_eq!(r.own, 1 << 8);
    assert_eq!(beat::own_note(r.own).unwrap(), "key 9 used its writer's own sample; playing the starter pack's");
    let mut e = loaded(&r.snapshot);
    // A change, then `:w`: the text says it; no `own` line, as the terminal plays the pack's.
    e.process(1, &[Cmd::new(PI_CMD_PATTERN, 0, 0, 2), Cmd::new(PI_CMD_STEP, 3, 7, 1)], &mut [Event::default(); 8], None, PI_AUDIO_NONE);
    let d = dir("beat-colon");
    let path = d.join("mine.beat");
    beat::write(&path, &beat::to_file(&e.save()).unwrap()).unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.starts_with("pocket-instruments beat 1\n") && !written.contains("own") && written.contains("pattern 3"), "{written}");
    let f = loaded(&beat::from_file(written.as_bytes()).unwrap().snapshot);
    assert_eq!(f.save(), e.save());
    // A binary file through `:e` is passed as it is.
    let (_, v4) = v4_snapshots().into_iter().next().unwrap();
    assert_eq!(beat::from_file(&v4).unwrap(), beat::Read { snapshot: v4.clone(), own: 0 });
    // Refusals carry the crate's words; a version 4 snapshot is not written as text unloaded.
    assert!(beat::from_file(b"pocket-instruments beat 1\nbogus\n").unwrap_err().starts_with("line 2"));
    assert!(beat::to_file(&v4).is_err(), "pib_write takes version 5 only");
}

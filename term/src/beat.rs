//! The beat: one file, read at start and written as it changes (docs/TERMINAL.md, "The beat";
//! `docs/BEAT.md`, "The terminal host's beat"; D134 item 5, D136 item 8).
//!
//! Everything that turns a file into the core's snapshot and back goes through one pair,
//! `from_file` and `to_file`, so the beat crate (`beat/`, issue 140: `pib_read`, `pib_write`)
//! drops in behind them and nothing else in the host changes. The host has no reader of the
//! text of its own (D134: one reader for every host).
//!
//! A file that starts with the snapshot's magic, `PKIN`, is read as a snapshot (any version the
//! core loads, 1 to 5): an old binary `rhythm.beat` still loads (D136 item 8). Anything else is
//! text, read by the beat crate's `pib_read`. The beat is written as text by `pib_write`, from
//! the version 5 snapshot the core saves, so an old binary file becomes text at its next save.
//! Both run on the terminal's thread, never on the audio thread.
//!
//! SPDX-License-Identifier: MIT
use std::path::{Path, PathBuf};

/// The snapshot's magic (`ENGINE.md` §8): a file that starts with it is binary.
pub const MAGIC: &[u8; 4] = b"PKIN";
/// The text form's first line (`docs/BEAT.md`, "The form").
pub const TEXT_FIRST: &[u8] = b"pocket-instruments beat";

/// Whether `to_file` writes the text form: since the beat crate (issue 140), yes.
pub const WRITES_TEXT: bool = true;

/// `beat/include/pi_beat.h`.
pub const PIB_TEXT_MAX: usize = 98_304;
pub const PIB_MSG_MAX: usize = 192;
pub const PIB_SNAPSHOT_MAX: usize = 22_100;
pub const PIB_ERR_TEXT: i32 = -1;
pub const PIB_ERR_ROOM: i32 = -2;
pub const PIB_ERR_NEWER: i32 = -3;
pub const PIB_ERR_SNAPSHOT: i32 = -4;

unsafe extern "C" {
    fn pib_read(text: *const u8, len: u32, snap: *mut u8, cap: u32, own: *mut u32, msg: *mut u8, msg_cap: u32) -> i32;
    fn pib_write(snap: *const u8, len: u32, own: u32, text: *mut u8, cap: u32) -> i32;
}

/// A file read: the snapshot for `pi_load`, and the keys its `own` line names (bit k-1 for key
/// k: the writer's own sample there, the starter pack's played instead; D34).
#[derive(Debug, PartialEq, Eq)]
pub struct Read {
    pub snapshot: Vec<u8>,
    pub own: u32,
}

/// What the player is told of a file's `own` keys (`docs/BEAT.md`, "What it cannot carry").
pub fn own_note(own: u32) -> Option<String> {
    let keys: Vec<String> = (0..16).filter(|k| own >> k & 1 != 0).map(|k| format!("{}", k + 1)).collect();
    match keys.len() {
        0 => None,
        1 => Some(format!("key {} used its writer's own sample; playing the starter pack's", keys[0])),
        _ => Some(format!("keys {} used their writer's own samples; playing the starter pack's", keys.join(" "))),
    }
}

/// `$XDG_DATA_HOME/pocket-instruments/rhythm.beat`, or `~/.local/share/…` when that is unset.
/// On Windows, `%APPDATA%\pocket-instruments\rhythm.beat` (not built or tried: D123).
pub fn default_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").filter(|s| !s.is_empty()).map(|h| PathBuf::from(h).join(".local/share")))
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))?;
    Some(base.join("pocket-instruments").join("rhythm.beat"))
}

/// A file's bytes as the snapshot to give `pi_load`, or why not. An old binary file is read
/// by its magic (D136 item 8), the bytes as they are; anything else by `pib_read`, whose
/// refusal ("line 12, column 5: …") is the reason given.
pub fn from_file(bytes: &[u8]) -> Result<Read, String> {
    if bytes.starts_with(MAGIC) {
        return Ok(Read { snapshot: bytes.to_vec(), own: 0 });
    }
    let mut snap = vec![0u8; PIB_SNAPSHOT_MAX];
    let mut msg = [0u8; PIB_MSG_MAX];
    let mut own = 0u32;
    let len = u32::try_from(bytes.len()).map_err(|_| "too long to be a beat".to_string())?;
    // SAFETY: every pointer is to a buffer of the length passed beside it.
    let n = unsafe { pib_read(bytes.as_ptr(), len, snap.as_mut_ptr(), snap.len() as u32, &mut own, msg.as_mut_ptr(), msg.len() as u32) };
    if n < 0 {
        let end = msg.iter().position(|b| *b == 0).unwrap_or(msg.len());
        let why = String::from_utf8_lossy(&msg[..end]).into_owned();
        return Err(match (n, why.is_empty()) {
            (_, false) => why,
            (PIB_ERR_ROOM, true) => "too large to be a beat".into(),
            _ => format!("not a beat file ({n})"),
        });
    }
    snap.truncate(n as usize);
    Ok(Read { snapshot: snap, own })
}

/// The text to write for a snapshot the core saved (version 5: the core writes no other), by
/// `pib_write`. `own` is 0: the terminal plays only the starter pack it carries, and no player's
/// own sample (docs/TERMINAL.md, "One binary, no files beside it").
pub fn to_file(snapshot: &[u8]) -> Result<Vec<u8>, String> {
    let mut text = vec![0u8; PIB_TEXT_MAX];
    // SAFETY: every pointer is to a buffer of the length passed beside it.
    let n = unsafe { pib_write(snapshot.as_ptr(), snapshot.len() as u32, 0, text.as_mut_ptr(), text.len() as u32) };
    match n {
        PIB_ERR_SNAPSHOT => Err("the engine's save is not a version 5 snapshot".into()),
        n if n < 0 => Err(format!("the beat could not be written as text ({n})")),
        n => {
            text.truncate(n as usize);
            Ok(text)
        }
    }
}

/// The beat file at start: what to load, whether the host may write it, and what to say.
#[derive(Debug, PartialEq, Eq)]
pub struct Opened {
    /// The snapshot to load, if the file was read.
    pub snapshot: Option<Vec<u8>>,
    /// Whether the host may save to the file. False when a file is there and was not read: it
    /// is left alone (docs/TERMINAL.md: "A file `pi_load` refuses is left alone"), so a beat
    /// this build cannot read is never written over.
    pub writable: bool,
    pub note: Option<String>,
}

pub fn open(path: &Path) -> Opened {
    match std::fs::read(path) {
        Ok(bytes) => match from_file(&bytes) {
            Ok(r) => {
                // An old binary beat is read as it is and written as text at the next save
                // (D136 item 8): the player is told once, as it opens. A text beat's `own` keys
                // are told as `docs/BEAT.md` says.
                let note = if bytes.starts_with(MAGIC) { Some(binary_note(path)) } else { own_note(r.own) };
                Opened { snapshot: Some(r.snapshot), writable: true, note }
            }
            Err(why) => Opened { snapshot: None, writable: false, note: Some(left_alone(path, &why)) },
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Opened { snapshot: None, writable: true, note: None },
        Err(e) => Opened { snapshot: None, writable: false, note: Some(left_alone(path, &e.to_string())) },
    }
}

/// What the host says of a file it did not read, and leaves alone (it then starts empty; `:w`
/// says where else to save). Kept short: the footer has 78 cells.
pub fn left_alone(path: &Path, why: &str) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    format!("{name} left alone: {why}")
}

/// What the host says when it opens an old binary beat, which it saves as text from then on.
pub fn binary_note(path: &Path) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    format!("{name}: an older binary beat, read; saved as text from now on")
}

pub fn read(path: &Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// Written beside itself and renamed into place, so a crash never leaves half a beat.
pub fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("beat.part");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

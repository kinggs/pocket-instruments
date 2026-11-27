//! The beat: one file, read at start and written as it changes (docs/TERMINAL.md, "The beat";
//! `docs/BEAT.md`, "The terminal host's beat"; D134 item 5, D136 item 8).
//!
//! Everything that turns a file into the core's snapshot and back goes through one pair,
//! `from_file` and `to_file`, so the beat crate (`beat/`, issue 140: `pib_read`, `pib_write`)
//! drops in behind them and nothing else in the host changes. The host has no reader of the
//! text of its own (D134: one reader for every host).
//!
//! Until the crate lands: a file that starts with the snapshot's magic, `PKIN`, is read as a
//! snapshot (any version the core loads, 1 to 5), and the beat is written as the snapshot the
//! core saves (version 5). A beat as text is not read, and the file is left alone.
//!
//! SPDX-License-Identifier: MIT
use std::path::{Path, PathBuf};

/// The snapshot's magic (`ENGINE.md` §8): a file that starts with it is binary.
pub const MAGIC: &[u8; 4] = b"PKIN";
/// The text form's first line (`docs/BEAT.md`, "The form").
pub const TEXT_FIRST: &[u8] = b"pocket-instruments beat";

/// Whether `to_file` writes the text form. False until the beat crate lands (issue 140).
pub const WRITES_TEXT: bool = false;

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
/// by its magic (D136 item 8). With the beat crate this becomes: the magic, the bytes as they
/// are; else `pib_read`.
pub fn from_file(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.starts_with(MAGIC) {
        return Ok(bytes.to_vec());
    }
    if bytes.starts_with(TEXT_FIRST) {
        return Err("a beat written as text; this build reads it once the beat reader is in (issue 140)".into());
    }
    Err("not a beat file".into())
}

/// The bytes to write for a snapshot the core saved. With the beat crate this becomes
/// `pib_write` (the terminal plays only the starter pack's samples, so `own` is 0).
pub fn to_file(snapshot: &[u8]) -> Result<Vec<u8>, String> {
    if !snapshot.starts_with(MAGIC) {
        return Err("the engine's save is not a snapshot".into());
    }
    Ok(snapshot.to_vec())
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
            Ok(s) => Opened { snapshot: Some(s), writable: true, note: None },
            Err(why) => Opened { snapshot: None, writable: false, note: Some(left_alone(path, &why)) },
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Opened { snapshot: None, writable: true, note: None },
        Err(e) => Opened { snapshot: None, writable: false, note: Some(left_alone(path, &e.to_string())) },
    }
}

/// What the host says of a file it did not read, and leaves alone.
pub fn left_alone(path: &Path, why: &str) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    format!("{name} not read ({why}); left alone, starting empty. :w FILE saves elsewhere")
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

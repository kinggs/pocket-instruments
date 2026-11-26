//! The beat: the core's snapshot in one file (docs/TERMINAL.md, "The beat").
//!
//! SPDX-License-Identifier: MIT
use std::path::{Path, PathBuf};

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

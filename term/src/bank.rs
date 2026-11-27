//! The bank of beats (`docs/BANK.md`; D141, D142, proposed for Kenny to rule): `beats/`,
//! compiled in by `build.rs` as the files are, text and all, with `bank.txt` giving the order.
//! **The first line of `bank.txt` is the beat a new player opens on.** Nothing here names which
//! beat that is, so a ruling that moves it changes `bank.txt` alone.
//!
//! A beat is turned into a snapshot when it is needed, by `beat::from_file` (the beat crate's
//! `pib_read`): the same path `:e` takes. There is no snapshot made at build time.
//!
//! What the host does with one (`docs/TERMINAL.md`, "The bank"):
//! - no file at the beat's path: the default is loaded at start (`beat::open_or_default`);
//! - `:bank` lists the bank, `:bank <n>` loads the n-th, `:new` the empty beat; each keeps the
//!   beat it replaces in `rhythm.previous.beat` beside the beat file (`replace`).
//!
//! SPDX-License-Identifier: MIT
use crate::audio::Shared;
use crate::beat;
use crate::ffi::PI_OK;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

mod compiled {
    include!(concat!(env!("OUT_DIR"), "/bank.rs"));
}

/// The empty beat, the one `pi_init` builds (§21: every pattern clear, the chain pattern 1 alone),
/// as text: every line at its default (`docs/BEAT.md`, "The form"). Read by `pib_read` like any
/// beat; `term/tests/bank.rs` holds it to the core's own `pi_init` save, byte for byte.
pub const EMPTY: &[u8] = b"pocket-instruments beat 1\n";

/// A bank: its list (`bank.txt`) and its files by name.
#[derive(Clone, Copy)]
pub struct Bank<'a> {
    pub list: &'a [u8],
    pub files: &'a [(&'a str, &'a [u8])],
}

impl Bank<'static> {
    /// The bank compiled into this binary from `beats/`.
    pub fn compiled() -> Bank<'static> {
        Bank { list: compiled::LIST, files: compiled::FILES }
    }
}

impl<'a> Bank<'a> {
    /// The files, in the menu's order: `bank.txt`'s lines.
    pub fn order(&self) -> Vec<String> {
        String::from_utf8_lossy(self.list).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
    }

    /// The n-th beat (counted from 1, as the menu numbers them), read: its file's name and what
    /// `pib_read` gave. Or why not, in words for the footer.
    pub fn read(&self, n: usize) -> Result<(String, beat::Read), String> {
        let order = self.order();
        if order.is_empty() {
            return Err("the bank lists no beat".into());
        }
        let Some(file) = n.checked_sub(1).and_then(|i| order.get(i)) else {
            return Err(format!("the bank has beats 1 to {}, not {n}", order.len()));
        };
        let Some((_, text)) = self.files.iter().find(|(f, _)| *f == file) else {
            return Err(format!("{file} is not in this build's bank"));
        };
        beat::from_file(text).map(|r| (file.clone(), r)).map_err(|why| format!("{file}: {why}"))
    }

    /// The beat a new player opens on: the first line's.
    pub fn default_beat(&self) -> Result<(String, beat::Read), String> {
        self.read(1)
    }

    /// The menu: each beat's number and name (the snapshot's bytes 112 to 143), or its file and
    /// why it cannot be read.
    pub fn menu(&self) -> Vec<String> {
        (1..=self.order().len())
            .map(|n| match self.read(n) {
                Ok((file, r)) => {
                    let name = name_of(&r.snapshot);
                    format!("{n}  {}", if name.is_empty() { file } else { name })
                }
                Err(why) => format!("{n}  ({why})"),
            })
            .collect()
    }
}

/// A snapshot's name (version 5's bytes 112 to 143, zero-padded), or "".
pub fn name_of(snap: &[u8]) -> String {
    let n = snap.get(112..144).unwrap_or(&[]);
    let end = n.iter().position(|b| *b == 0).unwrap_or(n.len());
    String::from_utf8_lossy(&n[..end]).trim().to_string()
}

/// Where the beat a load replaced is kept: `rhythm.previous.beat` beside the beat file (or beside
/// the `--beat` file: `<stem>.previous.beat`).
pub fn previous_path(beat: &Path) -> PathBuf {
    let stem = beat.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "rhythm".into());
    beat.with_file_name(format!("{stem}.previous.beat"))
}

/// The engine's snapshot as it is now, from the audio thread's next block.
pub fn snapshot_now(shared: &Shared) -> Result<Vec<u8>, String> {
    use std::sync::atomic::Ordering;
    shared.want_save.store(true, Ordering::Release);
    let until = Instant::now() + Duration::from_millis(500);
    while Instant::now() < until {
        let taken = {
            let mut s = shared.saved.lock().unwrap();
            let n = std::mem::take(&mut s.1);
            (n > 0).then(|| s.0[..n].to_vec())
        };
        if let Some(b) = taken {
            return Ok(b);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    Err("the engine did not answer the save".into())
}

/// A snapshot handed to the audio thread's ordinary load, and its result (`None`: no answer in
/// half a second).
pub fn load(shared: &Shared, bytes: Vec<u8>) -> Option<i32> {
    *shared.to_load.lock().unwrap() = Some((bytes, None));
    let until = Instant::now() + Duration::from_millis(500);
    let mut r = None;
    while Instant::now() < until && r.is_none() {
        std::thread::sleep(Duration::from_millis(2));
        r = shared.to_load.lock().unwrap().as_ref().and_then(|(_, r)| *r);
    }
    *shared.to_load.lock().unwrap() = None;
    r
}

/// `:bank <n>` and `:new`: the beat there now written to `previous` first (once: the next load
/// writes over it), then `snap` loaded. If the beat cannot be kept, nothing is loaded.
pub fn replace(shared: &Shared, previous: &Path, snap: Vec<u8>) -> Result<(), String> {
    let now = snapshot_now(shared)?;
    let text = beat::to_file(&now)?;
    let name = previous.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    beat::write(previous, &text).map_err(|e| format!("not loaded: the beat here could not be kept in {name}: {e}"))?;
    match load(shared, snap) {
        Some(PI_OK) => Ok(()),
        Some(code) => Err(format!("the engine refused it ({code})")),
        None => Err("the engine did not answer".into()),
    }
}

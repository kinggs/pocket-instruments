//! Pocket Instruments: the beat file. `docs/BEAT.md` is its specification and
//! `include/pi_beat.h` the same contract in C. A beat file is snapshot version 5
//! (`docs/ENGINE.md` §21) written as text, one to one: `read` turns the text into the
//! snapshot's bytes and `write` turns the bytes back into text, in its canonical form.
//!
//! Not the core: the core has no parser (D25, D134). Two pure functions of the caller's
//! buffers, no engine, no thread rule, never on the audio path. No standard library, no
//! allocator, no dependencies, no globals. SPDX-License-Identifier: MIT
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

/// §21's offsets: the one table the core uses too (D134 item 2). Included, not depended on.
#[allow(dead_code)]
pub mod layout {
    include!("../../core/src/snapshot_layout.rs");
}

mod read;
mod text;
mod write;

#[cfg(test)]
mod tests;

pub use read::{Read, read};
pub use text::Msg;
pub use write::write;

/// `pocket-instruments beat 1`: the one beat version this build reads and writes.
pub const BEAT_VERSION: u32 = 1;
/// The most text `write` can give for any snapshot, measured and bounded (`tests/size.rs`).
pub const TEXT_MAX: usize = 98_304;
/// The longest refusal `read` writes; a message buffer this long always holds all of it.
pub const MSG_MAX: usize = 192;

/// `pib_read`: the text is refused; the message says where and what the line takes.
pub const ERR_TEXT: i32 = -1;
/// The buffer is too short: the snapshot's (`pib_read`) or the text's (`pib_write`).
pub const ERR_ROOM: i32 = -2;
/// `pib_read`: a newer beat than this build reads (`this is beat 2; this build reads beat 1`).
pub const ERR_NEWER: i32 = -3;
/// `pib_write`: not a version 5 snapshot as the core writes it.
pub const ERR_SNAPSHOT: i32 = -4;

/// The sounds' names, for keys 1 to 16 (`docs/VOICES.md`'s roles; `docs/BEAT.md`).
pub const NAMES: [&str; 16] = [
    "kick", "snare", "hat", "openhat", "synsnare", "sticks", "cymbal", "noise", "clap", "click", "lowtom", "hitom", "cowbell",
    "blip", "tone", "bass",
];

/// The letters a `value` line can give, in the order the writer hands them out: `a` first,
/// not `x` or `X` (`x` is a step on keeping nothing).
pub const LETTERS: &[u8; 46] = b"abcdefghijklmnopqrstuvwABCDEFGHIJKLMNOPQRSTUVW";

/// Reads a beat file into snapshot version 5. Returns the snapshot's length (276 to 22 100),
/// with the keys the file names in `own` (bit k-1 for key k), or below zero with a message in
/// `msg`: `line 12, column 5: ...`, ended by a zero byte, cut to `msg_cap`. Nothing is written
/// to `snap` unless it succeeds. `own` and `msg` may be null (0 in WebAssembly).
///
/// # Safety
/// Each pointer, if not null, is valid for its length: `text` for `len` bytes to read, `snap`
/// for `cap` and `msg` for `msg_cap` bytes to write, `own` for one `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pib_read(text: *const u8, len: u32, snap: *mut u8, cap: u32, own: *mut u32, msg: *mut u8, msg_cap: u32) -> i32 {
    // SAFETY: the caller's promise, above; a null pointer stands for an empty buffer.
    let text: &[u8] = if text.is_null() || len == 0 { &[] } else { unsafe { core::slice::from_raw_parts(text, len as usize) } };
    let out: &mut [u8] = if snap.is_null() { &mut [] } else { unsafe { core::slice::from_raw_parts_mut(snap, cap as usize) } };
    let msg: &mut [u8] = if msg.is_null() { &mut [] } else { unsafe { core::slice::from_raw_parts_mut(msg, msg_cap as usize) } };
    match read(text, out) {
        Ok(r) => {
            if !own.is_null() {
                // SAFETY: as above.
                unsafe { *own = r.own as u32 };
            }
            if let Some(first) = msg.first_mut() {
                *first = 0;
            }
            r.len as i32
        }
        Err((code, m)) => {
            m.copy_to(msg);
            code
        }
    }
}

/// Writes snapshot version 5 as a beat file, in its canonical form (`docs/BEAT.md`). `own`:
/// the keys that played the writer's own sample (bit k-1 for key k), written as `own`; a bit
/// for a key that does not play a sample is not. Returns the text's length, or below zero:
/// `ERR_SNAPSHOT` (not version 5 as the core writes it: a version 1 to 4 snapshot is first
/// loaded and saved by the core), `ERR_ROOM` (`cap` under the text; `PIB_TEXT_MAX` is always
/// enough). Nothing is written unless it succeeds.
///
/// # Safety
/// Each pointer, if not null, is valid for its length: `snap` for `len` bytes to read, `text`
/// for `cap` bytes to write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pib_write(snap: *const u8, len: u32, own: u32, text: *mut u8, cap: u32) -> i32 {
    // SAFETY: the caller's promise, above.
    let snap: &[u8] = if snap.is_null() || len == 0 { &[] } else { unsafe { core::slice::from_raw_parts(snap, len as usize) } };
    let out: &mut [u8] = if text.is_null() { &mut [] } else { unsafe { core::slice::from_raw_parts_mut(text, cap as usize) } };
    match write(snap, own as u16, out) {
        Ok(n) => n as i32,
        Err(code) => code,
    }
}

/// Nothing here is meant to panic: every index is checked. If one ever does, it must not
/// unwind into C, so it stops, as the core does.
#[cfg(all(not(test), feature = "panic-handler"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();
    #[cfg(not(target_arch = "wasm32"))]
    {
        unsafe extern "C" {
            fn abort() -> !;
        }
        // SAFETY: abort takes nothing and never returns; every native host has a libc.
        unsafe { abort() }
    }
}

//! §8, §15, §17, §20 and §21. The snapshot: version 5, 276 to 22 100 bytes, little-endian.
//! Written and read only here. This core writes version 5 and loads all five versions it has
//! ever written; versions 1 to 4 load into pattern 1.

use crate::abi::{ERR_CRC, ERR_MAGIC, ERR_SIZE, ERR_VERSION, OK};
use crate::clock::{MBPM_MAX, MBPM_MIN};
use crate::crc::crc32;
use crate::engine::{CREDIT, Engine, NAME};
use crate::keys::VOLUME_MAX;
use crate::seq::{FX_MAX, PATTERNS, Pattern, STEPS, SWING_MAX, TRACKS};
use crate::voice::PARAM_DEFAULT;

/// §21: version 5 is 276 bytes and a block for each pattern written; sixteen is the most.
pub const SIZE_MIN: usize = 276;
pub const BLOCK: usize = 1364;
pub const SIZE: usize = SIZE_MIN + PATTERNS * BLOCK;
/// Versions 4 (§20), 3 (§17), 2 (§15) and 1 (§8), which it still loads.
pub const SIZE_V4: usize = 1464;
pub const SIZE_V3: usize = 408;
pub const SIZE_V2: usize = 388;
pub const SIZE_V1: usize = 316;
const MAGIC: [u8; 4] = *b"PKIN";
const VERSION: u16 = 5;
const KIND_RHYTHM: u16 = 1;
const HEADER: usize = 12;
/// Version 5: where the patterns' blocks begin, after the name (112) and the credit (144).
const BLOCKS_AT: usize = 272;
const _: () = assert!(BLOCKS_AT + 4 == SIZE_MIN && 112 + NAME == 144 && 144 + CREDIT == BLOCKS_AT);
const _: () = assert!(SIZE == 22_100);

/// The payload size field counts what is between the header and the CRC: 300, 372, 392, 1448,
/// and for version 5 260 + 1 364 a pattern written.
const fn payload(size: usize) -> u32 {
    (size - HEADER - 4) as u32
}

/// Each version this core has written: the least it can be. For versions 1 to 4 that is its
/// size; version 5 adds the blocks its "written" field names. The CRC is always the last four.
fn least_of_version(version: u16) -> Option<usize> {
    match version {
        1 => Some(SIZE_V1),
        2 => Some(SIZE_V2),
        3 => Some(SIZE_V3),
        4 => Some(SIZE_V4),
        5 => Some(SIZE_MIN),
        _ => None, // newer than this core, or 0, which no core has written
    }
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn put(b: &mut [u8], at: usize, bytes: &[u8]) {
    if let Some(dst) = b.get_mut(at..at + bytes.len()) {
        dst.copy_from_slice(bytes);
    }
}

/// §21: bit `p`, pattern `p` is not a cleared one, so its block is written.
fn written(e: &Engine) -> u16 {
    let mut w = 0u16;
    for (p, pat) in e.bank.pats.iter().enumerate() {
        if !pat.cleared() {
            w |= 1 << p;
        }
    }
    w
}

/// §21: version 5 into `b`. Returns the bytes written, 276 to 22 100, or 0, with nothing
/// written, when `b` is shorter than that.
pub fn save(e: &Engine, b: &mut [u8]) -> usize {
    let w = written(e);
    let size = SIZE_MIN + BLOCK * w.count_ones() as usize;
    let Some(b) = b.get_mut(..size) else {
        return 0;
    };
    b.fill(0);
    put(b, 0, &MAGIC);
    put(b, 4, &VERSION.to_le_bytes());
    put(b, 6, &KIND_RHYTHM.to_le_bytes());
    put(b, 8, &payload(size).to_le_bytes());
    put(b, 12, &e.seed.to_le_bytes());
    put(b, 16, &e.clock.mbpm.to_le_bytes());
    b[20] = e.swing;
    b[21] = e.keys.volume;
    b[22] = e.keys.sel;
    // 23: the hat cut, 0 until D47 has a setting.
    for t in 0..TRACKS {
        put(b, 24 + 4 * t, &e.voices.params[t][0].to_le_bytes());
        put(b, 26 + 4 * t, &e.voices.params[t][1].to_le_bytes());
    }
    put(b, 88, &e.voices.sampled.to_le_bytes());
    put(b, 90, &e.fx_sounds.to_le_bytes());
    put(b, 92, &w.to_le_bytes());
    // The chain saved is the one that will play: the next chain if one waits (§21).
    let (chain, len) = e.bank.shown();
    b[94] = len;
    put(b, 96, &chain[..(len as usize).min(PATTERNS)]); // zero past its length
    put(b, 112, &e.name);
    put(b, 144, &e.credit);
    let mut at = BLOCKS_AT;
    for (p, pat) in e.bank.pats.iter().enumerate() {
        if w >> p & 1 == 1 {
            put_block(&mut b[at..at + BLOCK], pat);
            at += BLOCK;
        }
    }
    let crc = crc32(&b[..size - 4]);
    put(b, size - 4, &crc.to_le_bytes());
    size
}

/// §21: a pattern's block, in the order versions 1 to 4 held its parts: steps, probability,
/// the lane, keeps, the values (zero where nothing is kept), the length, three zero.
fn put_block(b: &mut [u8], pat: &Pattern) {
    for t in 0..TRACKS {
        put(b, 2 * t, &pat.on[t].to_le_bytes());
        put(b, 32 + STEPS * t, &pat.prob[t]);
        put(b, 304 + 2 * t, &pat.keep[t].to_le_bytes());
        for s in 0..STEPS {
            if pat.keep[t] >> s & 1 == 1 {
                let at = 336 + 4 * (STEPS * t + s);
                put(b, at, &pat.kept[t][s][0].to_le_bytes());
                put(b, at + 2, &pat.kept[t][s][1].to_le_bytes());
            }
        }
    }
    put(b, 288, &pat.fx);
    b[1360] = pat.length;
}

/// Where a pattern's parts are in a snapshot: the steps, the probabilities, the lane (version 3
/// on), the keeps and their values (version 4 on), the length. A version 5 block's are from
/// the block's start.
struct At {
    on: usize,
    prob: usize,
    fx: Option<usize>,
    keep: Option<(usize, usize)>,
    length: usize,
}
const AT_V5: At = At { on: 0, prob: 32, fx: Some(288), keep: Some((304, 336)), length: 1360 };
const AT_V4: At = At { on: 24, prob: 56, fx: Some(384), keep: Some((404, 436)), length: 20 };

/// A pattern read back, clamped as a command would be. What the version does not hold stays as
/// a cleared pattern has it. One reader for versions 1 to 5, so they cannot drift.
#[inline(never)]
fn get_pattern(b: &[u8], at: &At, pat: &mut Pattern) {
    for t in 0..TRACKS {
        pat.on[t] = u16_at(b, at.on + 2 * t);
        for s in 0..STEPS {
            pat.prob[t][s] = b[at.prob + STEPS * t + s].min(100);
        }
        if let Some((keep, kept)) = at.keep {
            // §20: values where the keep bit is clear are not read.
            pat.keep[t] = u16_at(b, keep + 2 * t);
            for s in 0..STEPS {
                let v = kept + 4 * (STEPS * t + s);
                pat.kept[t][s] = if pat.keep[t] >> s & 1 == 1 { [u16_at(b, v), u16_at(b, v + 2)] } else { [0; 2] };
            }
        }
    }
    if let Some(fx) = at.fx {
        for s in 0..STEPS {
            let v = b[fx + s];
            pat.fx[s] = if v > FX_MAX { 0 } else { v }; // §17: past 16 loads as 0
        }
    }
    pat.length = b[at.length].clamp(1, STEPS as u8);
}

/// Checks everything before it changes anything, in §8's order, with the least size taken from
/// the version, then (version 5) the payload size against the patterns written, then the
/// length passed against the payload, then the CRC. A longer buffer is read as its first
/// `size` bytes: the rest is the host's. Values a correct core never writes (a length of 0, a
/// volume of 0) are clamped, as the same values are in a command. What an older version does
/// not hold gets `pi_init`'s values; an older version loads into pattern 1 (§21).
pub fn load(e: &mut Engine, buf: &[u8]) -> i32 {
    if buf.len() < HEADER {
        return ERR_SIZE;
    }
    if buf[..4] != MAGIC {
        return ERR_MAGIC;
    }
    let version = u16_at(buf, 4);
    let Some(least) = least_of_version(version) else {
        return ERR_VERSION;
    };
    if buf.len() < least {
        return ERR_SIZE;
    }
    if u16_at(buf, 6) != KIND_RHYTHM {
        return ERR_MAGIC;
    }
    let size = if version == 5 { SIZE_MIN + BLOCK * u16_at(buf, 92).count_ones() as usize } else { least };
    if u32_at(buf, 8) != payload(size) {
        return ERR_SIZE;
    }
    if buf.len() < size {
        return ERR_SIZE;
    }
    let b = &buf[..size];
    if u32_at(b, size - 4) != crc32(&b[..size - 4]) {
        return ERR_CRC;
    }

    e.reseed(u32_at(b, 12));
    e.clock.mbpm = u32_at(b, 16).clamp(MBPM_MIN, MBPM_MAX);
    let bank = &mut e.bank;
    for pat in &mut bank.pats {
        pat.clear();
    }
    (bank.chain, bank.chain_len, bank.pos, bank.active) = ([0; PATTERNS], 1, 0, 0);
    (bank.next, bank.next_len) = ([0; PATTERNS], 0);
    e.name = [0; NAME];
    e.credit = [0; CREDIT];
    if version == 5 {
        load_v5(e, b);
    } else {
        load_old(e, b, version);
    }
    for t in 0..TRACKS {
        // §19's addition: a sound not yet hit shows its own on the slider outputs.
        e.own_out(t);
    }
    e.generation = e.generation.wrapping_add(1);
    OK
}

/// §21: version 5. The patterns not written stay cleared; `pos` 0, no next chain, the active
/// pattern the chain's first.
fn load_v5(e: &mut Engine, b: &[u8]) {
    e.swing = b[20].min(SWING_MAX as u8);
    // The view's volume row is (1 << volume) - 1: past 16 it would overflow (§14).
    e.keys.volume = b[21].clamp(1, VOLUME_MAX);
    e.keys.sel = b[22].min(TRACKS as u8 - 1);
    // 23, the hat cut: nothing to set while no core writes anything but 0 (§15).
    for t in 0..TRACKS {
        e.voices.params[t] = [u16_at(b, 24 + 4 * t), u16_at(b, 26 + 4 * t)];
    }
    e.voices.sampled = u16_at(b, 88);
    e.fx_sounds = u16_at(b, 90);
    let w = u16_at(b, 92);
    let bank = &mut e.bank;
    bank.chain_len = b[94].clamp(1, PATTERNS as u8);
    for i in 0..bank.chain_len as usize {
        bank.chain[i] = b[96 + i].min(PATTERNS as u8 - 1);
    }
    bank.active = bank.chain[0];
    e.name.copy_from_slice(&b[112..112 + NAME]);
    e.credit.copy_from_slice(&b[144..144 + CREDIT]);
    let mut at = BLOCKS_AT;
    for p in 0..PATTERNS {
        if w >> p & 1 == 1 {
            get_pattern(&b[at..at + BLOCK], &AT_V5, &mut bank.pats[p]);
            at += BLOCK;
        }
    }
}

/// Versions 1 to 4, into pattern 1, each by its own rule, exactly as the ABI 15 core loaded
/// them; the other fifteen cleared and the chain pattern 1 alone (the caller's).
fn load_old(e: &mut Engine, b: &[u8], version: u16) {
    let pat = &mut e.bank.pats[0];
    // The lane from version 3 (§17), the keeps from version 4 (§20), at version 4's places.
    let at = At { fx: if version < 3 { None } else { AT_V4.fx }, keep: if version < 4 { None } else { AT_V4.keep }, ..AT_V4 };
    get_pattern(b, &at, pat);
    e.swing = b[21].min(SWING_MAX as u8);
    if version == 1 {
        e.voices.params = [[PARAM_DEFAULT; 2]; TRACKS];
        e.voices.sampled = 0;
        e.keys.volume = VOLUME_MAX;
        e.keys.sel = 0;
    } else {
        for t in 0..TRACKS {
            e.voices.params[t] = [u16_at(b, 312 + 4 * t), u16_at(b, 314 + 4 * t)];
        }
        e.voices.sampled = u16_at(b, 376);
        // The view's volume row is (1 << volume) - 1: past 16 it would overflow (§14).
        e.keys.volume = b[378].clamp(1, VOLUME_MAX);
        e.keys.sel = b[379].min(TRACKS as u8 - 1);
        // 380, the hat cut: nothing to set while no core writes anything but 0 (§15).
    }
    if version < 3 {
        e.fx_sounds = 0xFFFF; // and the lane empty, as cleared
    } else {
        e.fx_sounds = u16_at(b, 400);
        // 402 and 403 are not read, like 381 to 383 ⚠ (§15's reading carried on).
    }
    // §20. An older version: every step that is on keeps its sound's own A and B (D75 item
    // 6), so the hand cannot change an old beat. Version 4: what it says (`get_pattern`).
    if version < 4 {
        for t in 0..TRACKS {
            pat.keep[t] = pat.on[t];
            for s in 0..STEPS {
                pat.kept[t][s] = if pat.on[t] >> s & 1 == 1 { e.voices.params[t] } else { [0; 2] };
            }
        }
    }
}

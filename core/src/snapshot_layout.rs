// Snapshot version 5's layout (ENGINE.md §21): where each field is, and nothing else. Constants
// only, so two crates can hold the one table (docs/BEAT.md, "How the two are kept from
// drifting"; D134 item 2): the core uses it as a module (`snapshot.rs`) and the beat file's
// crate takes it with `include!` (`beat/src/lib.rs`). No dependency between the two crates.
// A field moved here moves for both, and every check below is checked in both builds.
//
// Plain `//` comments only: an inner doc comment cannot be `include!`d.
//
// SPDX-License-Identifier: MIT

/// The first four bytes of every version.
pub const MAGIC: [u8; 4] = *b"PKIN";
/// What the core writes; the beat file's `pocket-instruments beat 1` maps onto it.
pub const VERSION: u16 = 5;
/// The rhythm instrument.
pub const KIND_RHYTHM: u16 = 1;

// The header: magic, version, kind, payload size. The payload is what lies between it and the
// CRC, the last four bytes.
pub const AT_MAGIC: usize = 0;
pub const AT_VERSION: usize = 4;
pub const AT_KIND: usize = 6;
pub const AT_PAYLOAD: usize = 8;
pub const HEADER: usize = 12;

// The beat's own fields.
pub const AT_SEED: usize = 12; // u32
pub const AT_TEMPO: usize = 16; // u32, thousandths of a bpm
pub const AT_SWING: usize = 20; // u8
pub const AT_VOLUME: usize = 21; // u8
pub const AT_CHOSEN: usize = 22; // u8, the chosen sound, 0 to 15
pub const AT_HAT_CUT: usize = 23; // u8, 0
pub const AT_SLIDERS: usize = 24; // track by track, A then B, a u16 each
pub const AT_SAMPLED: usize = 88; // u16, bit t: track t plays its sample
pub const AT_FX_SOUNDS: usize = 90; // u16, bit t: track t goes through the effects
pub const AT_WRITTEN: usize = 92; // u16, bit p: pattern p's block follows
pub const AT_CHAIN_LEN: usize = 94; // u8, 1 to 16; 95 is zero
pub const AT_CHAIN: usize = 96; // a byte an entry, 0 to 15, zero past the length
pub const AT_NAME: usize = 112; // UTF-8, zero-padded (D134)
pub const NAME_LEN: usize = 32;
pub const AT_CREDIT: usize = 144; // the same
pub const CREDIT_LEN: usize = 128;
pub const AT_BLOCKS: usize = 272; // the written patterns' blocks, in rising order

/// A pattern's block, from its first byte.
pub const BLOCK: usize = 1364;
pub const B_STEPS: usize = 0; // a u16 a track, bit s: step s is on
pub const B_PROB: usize = 32; // track by track, a byte a step, 0 to 100
pub const B_LANE: usize = 288; // a byte a step, 0 or the effect 1 to 16
pub const B_KEEPS: usize = 304; // a u16 a track, bit s: step s keeps its pair
pub const B_VALUES: usize = 336; // track by track, step by step, A then B, a u16 each
pub const B_LENGTH: usize = 1360; // u8, 1 to 16; three zero after it

pub const TRACKS_V5: usize = 16;
pub const STEPS_V5: usize = 16;
pub const PATTERNS_V5: usize = 16;

/// The least version 5 can be (no pattern written), and the most (all sixteen).
pub const SIZE_MIN: usize = AT_BLOCKS + 4;
pub const SIZE: usize = SIZE_MIN + PATTERNS_V5 * BLOCK;

// The table holds together: each field ends where the next begins.
const _: () = assert!(AT_SLIDERS + 4 * TRACKS_V5 == AT_SAMPLED);
const _: () = assert!(AT_CHAIN + PATTERNS_V5 == AT_NAME);
const _: () = assert!(AT_NAME + NAME_LEN == AT_CREDIT && AT_CREDIT + CREDIT_LEN == AT_BLOCKS);
const _: () = assert!(B_STEPS + 2 * TRACKS_V5 == B_PROB && B_PROB + TRACKS_V5 * STEPS_V5 == B_LANE);
const _: () = assert!(B_LANE + STEPS_V5 == B_KEEPS && B_KEEPS + 2 * TRACKS_V5 == B_VALUES);
const _: () = assert!(B_VALUES + 4 * TRACKS_V5 * STEPS_V5 == B_LENGTH && B_LENGTH + 4 == BLOCK);
const _: () = assert!(SIZE_MIN == 276 && BLOCK == 1364 && SIZE == 22_100);

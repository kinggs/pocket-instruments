//! The starter pack, compiled in (docs/TERMINAL.md, "The crate": one binary, no files beside
//! it). Sixteen CC0 sounds as the engine reads them: 16-bit mono at 44.1 kHz, little-endian
//! (`samples/raw/*.s16`, in track order as `samples/tracks.txt`; `samples/README.md` credits
//! each). The core needs them 2-byte aligned, so each sits in an aligned wrapper.
//!
//! SPDX-License-Identifier: MIT

#[repr(C, align(2))]
struct Aligned<T: ?Sized>(T);

macro_rules! raw {
    ($name:literal) => {{
        static A: &Aligned<[u8]> = &Aligned(*include_bytes!(concat!("../../samples/raw/", $name, ".s16")));
        &A.0
    }};
}

static PACK: [&[u8]; 16] = [
    raw!("bd_haus"),
    raw!("sn_dolf"),
    raw!("hat_bdu"),
    raw!("hat_cats"),
    raw!("elec_hi_snare"),
    raw!("elec_pop"),
    raw!("elec_cymbal"),
    raw!("hat_noiz"),
    raw!("perc_snap"),
    raw!("elec_tick"),
    raw!("drum_tom_lo_hard"),
    raw!("drum_tom_hi_hard"),
    raw!("drum_cowbell"),
    raw!("elec_blip"),
    raw!("elec_ping"),
    raw!("bass_voxy_hit_c"),
];

/// Track `t`'s sample, as 16-bit frames.
pub fn sample(t: usize) -> &'static [i16] {
    let b = PACK[t];
    assert!(b.len() % 2 == 0 && (b.as_ptr() as usize) % 2 == 0);
    #[cfg(target_endian = "big")]
    compile_error!("the starter pack is little-endian on disk; this host reads it in place");
    // SAFETY: the bytes are 2-byte aligned (Aligned), an even count, 'static, and every i16 bit
    // pattern is valid; the platform is little-endian, as the files are.
    unsafe { std::slice::from_raw_parts(b.as_ptr().cast::<i16>(), b.len() / 2) }
}

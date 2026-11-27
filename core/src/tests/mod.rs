//! Tests need the standard library; the crate does not. Everything here goes through the C
//! functions, the way a host does, in memory the test owns.

mod bus;
mod clock;
mod fixtures;
mod fx;
mod json;
mod keep;
mod keys;
mod patterns;
mod route;
mod rules;
mod samples;
mod quiet;
mod snapshot;
mod sound;
mod sum;
mod tour;
mod weave;

use crate::abi::*;
use crate::engine::Engine;

pub const CAP: usize = 256;
/// §20: version 4's size, the shape `Host::save` gives (below), and versions 3's, 2's and 1's,
/// which `pi_load` still takes.
pub const SNAP: usize = 1464;
pub const SNAP_V3: usize = 408;
pub const SNAP_V2: usize = 388;
pub const SNAP_V1: usize = 316;
/// §21: what `pi_save` writes since ABI 16, version 5: 276 bytes and 1 364 a pattern written,
/// 22 100 at the most; and the view, 88 bytes.
pub const SNAP5_MIN: usize = 276;
pub const SNAP5_BLOCK: usize = 1364;
pub const SNAP5_MAX: usize = 22_100;
pub const VIEW: usize = 88;

/// A version 5 snapshot of a beat with one pattern (pattern 1 or none written, the chain
/// pattern 1 alone, no name or credit), in version 4's layout: what the ABI 15 core would have
/// written for the same engine. Panics on anything version 4 cannot hold, so no test that reads
/// version 4's offsets can lose a field to it. `fixtures.rs` holds it to bytes: every version 4
/// snapshot ever written loads, saves as version 5 and comes back here to the same bytes.
pub fn as_v4(s: &[u8]) -> [u8; SNAP] {
    let u16_at = |i: usize| u16::from_le_bytes([s[i], s[i + 1]]);
    assert_eq!(s[..8], [b'P', b'K', b'I', b'N', 5, 0, 1, 0], "a version 5 snapshot");
    let written = u16_at(92);
    assert_eq!(written & !1, 0, "only pattern 1 written");
    let n = written.count_ones() as usize;
    assert_eq!(s.len(), SNAP5_MIN + SNAP5_BLOCK * n);
    assert_eq!(u32::from_le_bytes(s[8..12].try_into().unwrap()) as usize, s.len() - 16);
    assert_eq!(s[s.len() - 4..], crate::crc::crc32(&s[..s.len() - 4]).to_le_bytes());
    assert_eq!((s[94], s[95], &s[96..112]), (1, 0, &[0u8; 16][..]), "the chain pattern 1 alone");
    assert!(s[112..272].iter().all(|b| *b == 0), "no name or credit");
    assert_eq!(s[23], 0, "the hat cut");
    let mut v = [0u8; SNAP];
    v[..4].copy_from_slice(b"PKIN");
    v[4..6].copy_from_slice(&4u16.to_le_bytes());
    v[6..8].copy_from_slice(&1u16.to_le_bytes());
    v[8..12].copy_from_slice(&((SNAP - 16) as u32).to_le_bytes());
    v[12..20].copy_from_slice(&s[12..20]); // seed, tempo
    v[21] = s[20]; // swing
    v[312..376].copy_from_slice(&s[24..88]); // sliders
    v[376..378].copy_from_slice(&s[88..90]); // sampled
    v[378] = s[21]; // volume
    v[379] = s[22]; // the chosen sound
    v[400..402].copy_from_slice(&s[90..92]); // fx sounds
    if n == 1 {
        let p = &s[272..272 + SNAP5_BLOCK];
        v[24..56].copy_from_slice(&p[..32]); // steps
        v[56..312].copy_from_slice(&p[32..288]); // probability
        v[384..400].copy_from_slice(&p[288..304]); // the lane
        v[404..436].copy_from_slice(&p[304..336]); // keeps
        v[436..1460].copy_from_slice(&p[336..1360]); // the values
        v[20] = p[1360]; // the length
        assert_eq!(p[1361..], [0, 0, 0]);
    } else {
        v[20] = 16;
        for b in &mut v[56..312] {
            *b = 100;
        }
    }
    let crc = crate::crc::crc32(&v[..SNAP - 4]);
    v[SNAP - 4..].copy_from_slice(&crc.to_le_bytes());
    v
}

/// An engine in caller-owned memory, driven through the ABI.
pub struct Host {
    _mem: Vec<u64>,
    pub e: *mut Engine,
    /// Frames processed so far: turns an event's block offset into an absolute frame.
    pub now: u64,
}

impl Host {
    pub fn new(sample_rate: u32, seed: u32) -> Host {
        let words = (pi_size() as usize).div_ceil(8);
        let mut mem = vec![0u64; words];
        let e = unsafe { pi_init(mem.as_mut_ptr().cast(), (words * 8) as u32, sample_rate, seed) };
        assert!(!e.is_null());
        Host { _mem: mem, e, now: 0 }
    }

    /// One block. Events come back with absolute frames.
    pub fn block(&mut self, frames: u32, cmds: &[Cmd]) -> Vec<Event> {
        let mut ev = [Event::default(); CAP];
        let n = unsafe {
            pi_process(self.e, frames, cmds.as_ptr(), cmds.len() as u32, ev.as_mut_ptr(), CAP as u32, core::ptr::null_mut(), 0)
        };
        let out = ev[..n as usize].iter().map(|x| Event { frame: x.frame, ..*x }).collect();
        self.now += frames as u64;
        out
    }

    /// Runs `frames` frames in blocks of `block`, with commands at absolute frames.
    /// Returns (absolute frame, event).
    pub fn run(&mut self, frames: u64, block: u32, cmds: &[(u64, Cmd)]) -> Vec<(u64, Event)> {
        let end = self.now + frames;
        let mut out = Vec::new();
        let mut next = cmds.iter().position(|c| c.0 >= self.now).unwrap_or(cmds.len());
        while self.now < end {
            let start = self.now;
            let n = (end - start).min(block as u64) as u32;
            let mut these = Vec::new();
            while next < cmds.len() && cmds[next].0 < start + n as u64 {
                these.push(Cmd { frame: (cmds[next].0 - start) as u32, ..cmds[next].1 });
                next += 1;
            }
            for ev in self.block(n, &these) {
                out.push((start + ev.frame as u64, ev));
            }
        }
        out
    }

    pub fn dropped(&self) -> u32 {
        unsafe { pi_dropped(self.e) }
    }

    pub fn generation(&self) -> u32 {
        unsafe { pi_generation(self.e) }
    }

    /// §14: the view block, as a host reads it: 88 bytes since ABI 16 (§21).
    pub fn view(&self) -> [u8; VIEW] {
        let mut b = [0u8; VIEW];
        assert_eq!(unsafe { pi_view(self.e, b.as_mut_ptr(), VIEW as u32) }, VIEW as u32);
        b
    }

    /// §21: what `pi_save` writes, version 5, at the length it returns.
    pub fn snap(&self) -> Vec<u8> {
        let mut b = vec![0u8; SNAP5_MAX];
        let n = unsafe { pi_save(self.e, b.as_mut_ptr(), SNAP5_MAX as u32) } as usize;
        assert!(n >= SNAP5_MIN && (n - SNAP5_MIN) % SNAP5_BLOCK == 0, "pi_save returned {n}");
        b.truncate(n);
        b
    }

    /// The beat as version 4 laid it out (`as_v4`), for the tests written before version 5
    /// that read its offsets. A beat that version 4 cannot hold fails here, never silently.
    pub fn save(&self) -> [u8; SNAP] {
        as_v4(&self.snap())
    }

    pub fn load(&mut self, b: &[u8]) -> i32 {
        unsafe { pi_load(self.e, b.as_ptr(), b.len() as u32) }
    }
}

pub fn cmd(kind: u8, a: u8, b: u8, value: u32) -> Cmd {
    Cmd { frame: 0, kind, a, b, slot: 0, value }
}

pub fn line(ev: &Event) -> String {
    match ev.kind {
        EV_STEP => format!("{} step {}\n", ev.tick, ev.step),
        EV_TRIG => format!("{} trig {} {}\n", ev.tick, ev.track, ev.step),
        k => panic!("unknown event type {k}"),
    }
}

pub fn trace(events: &[(u64, Event)]) -> String {
    events.iter().map(|(_, e)| line(e)).collect()
}

/// §21's text line for a view block: format 5's, which is format 4's with the patterns, bytes
/// 64 to 87, on its end: the active pattern, the chain's position, the chain and the next chain
/// as a lowercase hex digit an entry (`-` for no next chain), the used patterns.
pub fn view_text_pat(b: &[u8]) -> String {
    let nibbles = |at: usize, len: u8| -> String {
        (0..len as usize).map(|i| format!("{:x}", b[at + i / 2] >> (4 * (i & 1)) & 15)).collect()
    };
    let next = if b[67] == 0 { "-".to_string() } else { nibbles(80, b[67]) };
    format!(
        "{} pat={} pos={} chain={} next={} used={:04x}",
        view_text_keep(b), b[64], b[65], nibbles(72, b[66]), next, u16::from_le_bytes([b[68], b[69]]),
    )
}

/// §20's text line for a view block: format 4's, which is format 3's with the second mark,
/// bytes 60 to 63, on its end.
pub fn view_text_keep(b: &[u8]) -> String {
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    format!("{} ea={} eb={}", view_text_fx(b), u16_at(60), u16_at(62))
}

/// §17's text line for a view block: format 3's, which is §14's with the effects on its end.
pub fn view_text_fx(b: &[u8]) -> String {
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    format!("{} fx={} fxsteps={:04x} fxsounds={:04x}", view_text(b), b[55], u16_at(56), u16_at(58))
}

/// §14's text line for a view block: what a format 2 fixture holds. The tick and bytes 48 to
/// 63 are left out.
pub fn view_text(b: &[u8]) -> String {
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    let flag = |bit: u8| b[24] >> bit & 1;
    let step = if flag(3) == 1 { b[25].to_string() } else { "-".to_string() };
    let fires: Vec<String> = b[32..48].iter().map(|n| n.to_string()).collect();
    format!(
        "view play={} write={} edges={} sounds={} step={} sel={} vol={} tempo={} len={} swing={} held={:06x} field={:04x} stitch={:04x} bar={:04x} sampled={:04x} a={} b={} fires={}",
        flag(0), flag(1), flag(2), flag(4), step, b[26], b[27], u32_at(4), b[28], b[29],
        u32_at(8), u16_at(12), u16_at(14), u16_at(16), u16_at(18), u16_at(20), u16_at(22), fires.join(","),
    )
}

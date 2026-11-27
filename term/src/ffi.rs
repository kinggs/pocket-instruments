//! The core's C ABI (`core/include/pi_engine.h`, version 15), as this host calls it. Only what
//! the terminal uses. `pi_engine.h` is the contract; where this differs from it, it is right.
//!
//! SPDX-License-Identifier: MIT

pub const PI_ABI_VERSION: u32 = 15;
pub const PI_TRACKS: usize = 16;
pub const PI_STEPS: usize = 16;
pub const PI_VIEW_SIZE: usize = 64;
pub const PI_SNAPSHOT_SIZE: usize = 1464;
pub const PI_SWING_MAX: u32 = 48;
pub const PI_TICKS_PER_STEP: u64 = 96;

pub const PI_AUDIO_NONE: u32 = 0;
pub const PI_AUDIO_MIX: u32 = 1;

pub const PI_SOURCE_TABLE: u32 = 0;
pub const PI_SOURCE_EDGES: u32 = 1;

pub const PI_CMD_CLOCK: u8 = 1;
pub const PI_CMD_RESET: u8 = 2;
pub const PI_CMD_SOURCE: u8 = 3;
pub const PI_CMD_TEMPO: u8 = 4;
pub const PI_CMD_PLAY: u8 = 5;
pub const PI_CMD_STEP: u8 = 16;
pub const PI_CMD_PROB: u8 = 17;
pub const PI_CMD_SWING: u8 = 18;
pub const PI_CMD_LENGTH: u8 = 19;
pub const PI_CMD_FIRE: u8 = 21;
pub const PI_CMD_VOICE: u8 = 22;
pub const PI_CMD_FX: u8 = 23;
pub const PI_CMD_FXSOUND: u8 = 24;
pub const PI_CMD_KEEP: u8 = 26;
pub const PI_CMD_KEY_DOWN: u8 = 32;
pub const PI_CMD_KEY_UP: u8 = 33;
pub const PI_CMD_PARAM: u8 = 34;

pub const PI_KEY_SOUND: u8 = 16;
pub const PI_KEY_PATTERN: u8 = 17;
pub const PI_KEY_BPM: u8 = 18;
pub const PI_KEY_FUNCTION: u8 = 19;
pub const PI_KEY_FX: u8 = 20;
pub const PI_KEY_PLAY: u8 = 21;
pub const PI_KEY_WRITE: u8 = 22;
pub const PI_KEYS: usize = 23;

pub const PI_VIEW_TEMPO: usize = 4;
pub const PI_VIEW_HELD: usize = 8;
pub const PI_VIEW_FIELD: usize = 12;
pub const PI_VIEW_STITCH: usize = 14;
pub const PI_VIEW_BAR: usize = 16;
pub const PI_VIEW_SAMPLED: usize = 18;
pub const PI_VIEW_A: usize = 20;
pub const PI_VIEW_B: usize = 22;
pub const PI_VIEW_FLAGS: usize = 24;
pub const PI_VIEW_STEP: usize = 25;
pub const PI_VIEW_SEL: usize = 26;
pub const PI_VIEW_VOLUME: usize = 27;
pub const PI_VIEW_LENGTH: usize = 28;
pub const PI_VIEW_SWING: usize = 29;
pub const PI_VIEW_EXT: usize = 30;
pub const PI_VIEW_FIRES: usize = 32;
pub const PI_VIEW_HEARD: usize = 55;
pub const PI_VIEW_FX_STEPS: usize = 56;
pub const PI_VIEW_FX_SOUNDS: usize = 58;
pub const PI_VIEW_HIT_A: usize = 60;
pub const PI_VIEW_HIT_B: usize = 62;

pub const PI_VIEW_FLAG_PLAYING: u8 = 1;
pub const PI_VIEW_FLAG_WRITE: u8 = 2;
pub const PI_VIEW_FLAG_EDGES: u8 = 4;
pub const PI_VIEW_FLAG_STEP: u8 = 8;
pub const PI_VIEW_FLAG_SOUNDS: u8 = 16;

pub const PI_EV_STEP: u8 = 1;
pub const PI_EV_TRIG: u8 = 2;

pub const PI_OK: i32 = 0;
pub const PI_ERR_CRC: i32 = -4;

pub const PI_WEAVE_W: usize = 78;
pub const PI_WEAVE_H: usize = 15;
pub const PI_WEAVE_CELLS: usize = PI_WEAVE_W * PI_WEAVE_H;
pub const PI_WEAVE_REDUCED: u32 = 1;

/// An input, timestamped inside the block. 12 bytes, as `pi_cmd`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cmd {
    pub frame: u32,
    pub ty: u8,
    pub a: u8,
    pub b: u8,
    pub slot: u8,
    pub value: u32,
}

impl Cmd {
    pub const fn new(ty: u8, a: u8, b: u8, value: u32) -> Self {
        Cmd { frame: 0, ty, a, b, slot: 0, value }
    }
}

/// An output, as `pi_event`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Event {
    pub frame: u32,
    pub tick: u32,
    pub ty: u8,
    pub track: u8,
    pub step: u8,
    pub slot: u8,
}

/// The opaque engine.
#[repr(C)]
pub struct RawEngine {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn pi_abi_version() -> u32;
    pub fn pi_size() -> u32;
    pub fn pi_align() -> u32;
    pub fn pi_init(mem: *mut u8, size: u32, sample_rate: u32, seed: u32) -> *mut RawEngine;
    pub fn pi_process(
        engine: *mut RawEngine,
        frames: u32,
        cmds: *const Cmd,
        ncmds: u32,
        events: *mut Event,
        cap: u32,
        audio: *mut f32,
        channels: u32,
    ) -> u32;
    pub fn pi_view(engine: *const RawEngine, buf: *mut u8, cap: u32) -> u32;
    pub fn pi_weave(view: *const u8, len: u32, flags: u32, out: *mut u8, cap: u32) -> u32;
    pub fn pi_set_sample(engine: *mut RawEngine, track: u32, data: *const i16, frames: u32) -> u32;
    pub fn pi_delay_frames(sample_rate: u32) -> u32;
    pub fn pi_set_delay(engine: *mut RawEngine, line: *mut f32, frames: u32) -> u32;
    pub fn pi_dropped(engine: *const RawEngine) -> u32;
    pub fn pi_generation(engine: *const RawEngine) -> u32;
    pub fn pi_save(engine: *const RawEngine, buf: *mut u8, cap: u32) -> u32;
    pub fn pi_load(engine: *mut RawEngine, buf: *const u8, len: u32) -> i32;
}

/// One engine in memory this program owns, with effect 4's line lent. Everything on it is
/// called from one thread at a time (the audio thread, or a test).
pub struct Engine {
    mem: Vec<u64>,
    raw: *mut RawEngine,
    line: Vec<f32>,
    rate: u32,
}

// SAFETY: the engine is plain memory with no thread affinity; the host moves it to the audio
// thread once and calls it there only (pi_engine.h: every function from one thread).
unsafe impl Send for Engine {}

impl Engine {
    pub fn new(rate: u32, seed: u32) -> Engine {
        assert_eq!(unsafe { pi_abi_version() }, PI_ABI_VERSION, "the core's ABI is not this host's");
        let size = unsafe { pi_size() } as usize;
        assert!(unsafe { pi_align() } as usize <= 8);
        let mut mem = vec![0u64; size.div_ceil(8)];
        let raw = unsafe { pi_init(mem.as_mut_ptr().cast(), (mem.len() * 8) as u32, rate, seed) };
        assert!(!raw.is_null(), "pi_init refused good memory");
        Engine { mem, raw, line: Vec::new(), rate }
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    /// Lends effect 4 (and 14) its line (ABI 12).
    pub fn lend_delay(&mut self) -> bool {
        self.line = vec![0.0; unsafe { pi_delay_frames(self.rate) } as usize];
        unsafe { pi_set_delay(self.raw, self.line.as_mut_ptr(), self.line.len() as u32) == 1 }
    }

    /// Lends a track its sample. `data` must outlive the engine: the samples are `'static`.
    pub fn set_sample(&mut self, track: u32, data: &'static [i16]) -> bool {
        unsafe { pi_set_sample(self.raw, track, data.as_ptr(), data.len() as u32) == 1 }
    }

    pub fn process(&mut self, frames: u32, cmds: &[Cmd], events: &mut [Event], audio: Option<&mut [f32]>, channels: u32) -> usize {
        let (ptr, ch) = match audio {
            Some(a) => {
                assert!(a.len() >= frames as usize * channels as usize);
                (a.as_mut_ptr(), channels)
            }
            None => (std::ptr::null_mut(), PI_AUDIO_NONE),
        };
        unsafe {
            pi_process(self.raw, frames, cmds.as_ptr(), cmds.len() as u32, events.as_mut_ptr(), events.len() as u32, ptr, ch) as usize
        }
    }

    pub fn view(&self) -> [u8; PI_VIEW_SIZE] {
        let mut b = [0u8; PI_VIEW_SIZE];
        let n = unsafe { pi_view(self.raw, b.as_mut_ptr(), b.len() as u32) };
        assert_eq!(n as usize, PI_VIEW_SIZE);
        b
    }

    pub fn dropped(&self) -> u32 {
        unsafe { pi_dropped(self.raw) }
    }

    pub fn generation(&self) -> u32 {
        unsafe { pi_generation(self.raw) }
    }

    pub fn save(&self) -> [u8; PI_SNAPSHOT_SIZE] {
        let mut b = [0u8; PI_SNAPSHOT_SIZE];
        let n = unsafe { pi_save(self.raw, b.as_mut_ptr(), b.len() as u32) };
        assert_eq!(n as usize, PI_SNAPSHOT_SIZE);
        b
    }

    pub fn load(&mut self, bytes: &[u8]) -> i32 {
        unsafe { pi_load(self.raw, bytes.as_ptr(), bytes.len() as u32) }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Nothing to free in the core; the memory and the line go with the Vecs.
        let _ = &self.mem;
    }
}

/// The weave for a view (§16): PI_WEAVE_CELLS palette numbers.
pub fn weave(view: &[u8; PI_VIEW_SIZE], reduced: bool) -> [u8; PI_WEAVE_CELLS] {
    let mut out = [0u8; PI_WEAVE_CELLS];
    let n = unsafe { pi_weave(view.as_ptr(), view.len() as u32, if reduced { PI_WEAVE_REDUCED } else { 0 }, out.as_mut_ptr(), out.len() as u32) };
    assert_eq!(n as usize, PI_WEAVE_CELLS);
    out
}

pub fn u16_at(v: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([v[i], v[i + 1]])
}

pub fn u32_at(v: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([v[i], v[i + 1], v[i + 2], v[i + 3]])
}

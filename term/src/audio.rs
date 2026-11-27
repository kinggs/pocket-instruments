//! Sound, and the engine's thread (docs/TERMINAL.md, "Sound").
//!
//! The engine runs in cpal's callback. The callback allocates nothing and takes no lock it
//! could wait on: commands come in through a fixed ring (one producer, one consumer), the view
//! goes out through a sequence lock, and a save or a load is a `try_lock` that gives up at once.
//!
//! SPDX-License-Identifier: MIT
use crate::ffi::*;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const RING: usize = 1024;

/// What the terminal's thread and the engine's thread share.
pub struct Shared {
    ring: Box<[UnsafeCell<Cmd>]>,
    head: AtomicUsize, // written by the producer
    tail: AtomicUsize, // written by the consumer
    seq: AtomicU32,
    view: [AtomicU32; PI_VIEW_SIZE / 4],
    pub generation: AtomicU32,
    pub dropped: AtomicU32,
    pub xruns: AtomicU32,
    pub running: AtomicBool,
    pub rate: AtomicU32,
    /// A snapshot asked for (`want_save`) and its bytes once taken; a snapshot to load and its
    /// result.
    pub want_save: AtomicBool,
    /// Since ABI 16 a buffer of version 5's most and the length taken (0: none yet), so the
    /// audio thread saves without allocating.
    pub saved: Mutex<(Vec<u8>, usize)>,
    pub to_load: Mutex<Option<(Vec<u8>, Option<i32>)>>,
}

// SAFETY: the ring's cells are written only by the one producer between `head` loads and read
// only by the one consumer between `tail` loads (`push`, `drain`); everything else is atomic.
unsafe impl Sync for Shared {}

impl Shared {
    pub fn new() -> Arc<Shared> {
        Arc::new(Shared {
            ring: (0..RING).map(|_| UnsafeCell::new(Cmd::default())).collect(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            seq: AtomicU32::new(0),
            view: Default::default(),
            generation: AtomicU32::new(0),
            dropped: AtomicU32::new(0),
            xruns: AtomicU32::new(0),
            running: AtomicBool::new(false),
            rate: AtomicU32::new(0),
            want_save: AtomicBool::new(false),
            saved: Mutex::new((vec![0; PI_SNAPSHOT_SIZE], 0)),
            to_load: Mutex::new(None),
        })
    }

    /// The terminal's thread: queue a command. False if the ring is full (it never is in use:
    /// a callback drains it every few milliseconds).
    pub fn push(&self, c: Cmd) -> bool {
        let h = self.head.load(Ordering::Relaxed);
        let t = self.tail.load(Ordering::Acquire);
        if h.wrapping_sub(t) >= RING {
            return false;
        }
        // SAFETY: slot h is outside [t, h), so the consumer does not touch it.
        unsafe { *self.ring[h % RING].get() = c };
        self.head.store(h.wrapping_add(1), Ordering::Release);
        true
    }

    /// The engine's thread: take what is queued, at most `out.len()`.
    fn drain(&self, out: &mut [Cmd]) -> usize {
        let t = self.tail.load(Ordering::Relaxed);
        let h = self.head.load(Ordering::Acquire);
        let n = h.wrapping_sub(t).min(out.len());
        for (i, o) in out.iter_mut().enumerate().take(n) {
            // SAFETY: slots [t, h) are published by the producer and not written again until
            // `tail` moves past them.
            *o = unsafe { *self.ring[t.wrapping_add(i) % RING].get() };
            o.frame = 0;
        }
        self.tail.store(t.wrapping_add(n), Ordering::Release);
        n
    }

    fn publish(&self, v: &[u8; PI_VIEW_SIZE]) {
        let s = self.seq.load(Ordering::Relaxed);
        self.seq.store(s.wrapping_add(1), Ordering::Relaxed);
        std::sync::atomic::fence(Ordering::Release);
        for (i, w) in self.view.iter().enumerate() {
            w.store(u32_at(v, i * 4), Ordering::Relaxed);
        }
        self.seq.store(s.wrapping_add(2), Ordering::Release);
    }

    /// The latest view, never torn.
    pub fn view(&self) -> [u8; PI_VIEW_SIZE] {
        loop {
            let a = self.seq.load(Ordering::Acquire);
            if a % 2 == 1 {
                std::hint::spin_loop();
                continue;
            }
            let mut b = [0u8; PI_VIEW_SIZE];
            for (i, w) in self.view.iter().enumerate() {
                b[i * 4..i * 4 + 4].copy_from_slice(&w.load(Ordering::Relaxed).to_le_bytes());
            }
            std::sync::atomic::fence(Ordering::Acquire);
            if self.seq.load(Ordering::Relaxed) == a {
                return b;
            }
        }
    }
}

/// The engine's side: one block of `frames`, written to `out` with `channels` interleaved.
pub struct Runner {
    pub engine: Engine,
    shared: Arc<Shared>,
    cmds: Vec<Cmd>,
    events: Vec<Event>,
    mono: Vec<f32>,
}

impl Runner {
    pub fn new(engine: Engine, shared: Arc<Shared>) -> Runner {
        let r = Runner { engine, shared, cmds: vec![Cmd::default(); 256], events: vec![Event::default(); 256], mono: vec![0.0; 4096] };
        r.shared.publish(&r.engine.view());
        r
    }

    pub fn block(&mut self, out: Option<&mut [f32]>, frames: usize, channels: usize) {
        if frames == 0 {
            return;
        }
        let n = self.shared.drain(&mut self.cmds);
        let mut done = 0;
        let mut out = out;
        while done < frames {
            let k = (frames - done).min(self.mono.len());
            let cmds = if done == 0 { &self.cmds[..n] } else { &[][..] };
            match out.as_deref_mut() {
                Some(o) => {
                    self.engine.process(k as u32, cmds, &mut self.events, Some(&mut self.mono[..k]), PI_AUDIO_MIX);
                    for i in 0..k {
                        // The mix as the page plays it, held to ±1 (O14: each host's ceiling).
                        let s = self.mono[i].clamp(-1.0, 1.0);
                        let s = if s.is_nan() { 0.0 } else { s };
                        for c in 0..channels {
                            o[(done + i) * channels + c] = s;
                        }
                    }
                }
                None => {
                    self.engine.process(k as u32, cmds, &mut self.events, None, PI_AUDIO_NONE);
                }
            }
            done += k;
        }
        let sh = &self.shared;
        if sh.want_save.load(Ordering::Acquire) {
            if let Ok(mut s) = sh.saved.try_lock() {
                let (buf, len) = &mut *s;
                *len = self.engine.save_into(buf);
                sh.want_save.store(false, Ordering::Release);
            }
        }
        if let Ok(mut l) = sh.to_load.try_lock() {
            if let Some((bytes, res @ None)) = l.as_mut() {
                *res = Some(self.engine.load(bytes));
            }
        }
        sh.generation.store(self.engine.generation(), Ordering::Release);
        sh.dropped.store(self.engine.dropped(), Ordering::Relaxed);
        sh.publish(&self.engine.view());
    }
}

/// How sound leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// cpal's default host (ALSA on Linux, through PipeWire's or PulseAudio's plugin).
    Default,
    /// cpal's PipeWire host (the `pipewire` feature).
    PipeWire,
    /// No device: the engine runs on a timer, silent.
    None,
}

/// Keeps sound going until dropped.
pub enum Output {
    Stream(cpal::Stream),
    Timer(std::thread::JoinHandle<()>, Arc<AtomicBool>),
}

impl Drop for Output {
    fn drop(&mut self) {
        if let Output::Timer(_, stop) = self {
            stop.store(true, Ordering::Release);
        }
    }
}

/// Builds the engine at the device's rate, lends it the line and the samples, and starts it.
/// `setup` is called with the engine before it starts (to load the beat).
pub fn start(backend: Backend, seed: u32, shared: Arc<Shared>, setup: impl FnOnce(&mut Engine)) -> Result<(Output, String), String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let make = |rate: u32, setup: Box<dyn FnOnce(&mut Engine)>| {
        let mut e = Engine::new(rate, seed);
        e.lend_delay();
        for t in 0..PI_TRACKS {
            e.set_sample(t as u32, crate::samples::sample(t));
        }
        setup(&mut e);
        e
    };
    let setup: Box<dyn FnOnce(&mut Engine)> = Box::new(setup);
    if backend == Backend::None {
        let rate = 48_000;
        let mut r = Runner::new(make(rate, setup), shared.clone());
        shared.rate.store(rate, Ordering::Relaxed);
        shared.running.store(true, Ordering::Release);
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let h = std::thread::spawn(move || {
            let t0 = Instant::now();
            let mut frames: u64 = 0;
            while !stop2.load(Ordering::Acquire) {
                r.block(None, 480, 1);
                frames += 480;
                let due = t0 + Duration::from_micros(frames * 1_000_000 / rate as u64);
                if let Some(d) = due.checked_duration_since(Instant::now()) {
                    std::thread::sleep(d);
                }
            }
        });
        return Ok((Output::Timer(h, stop), "no sound (--audio none)".into()));
    }
    let host = match backend {
        #[cfg(all(feature = "pipewire", target_os = "linux"))]
        Backend::PipeWire => cpal::host_from_id(cpal::HostId::PipeWire).map_err(|e| format!("PipeWire: {e}"))?,
        #[cfg(not(all(feature = "pipewire", target_os = "linux")))]
        Backend::PipeWire => return Err("this build has no PipeWire host: build with --features pipewire, or use --audio default".into()),
        _ => cpal::default_host(),
    };
    let dev = host.default_output_device().ok_or("no sound output device (try --audio none)")?;
    let sc = dev.default_output_config().map_err(|e| format!("the output device: {e}"))?;
    let mut cfg = sc.config();
    let rate = cfg.sample_rate as u32;
    let channels = cfg.channels as usize;
    cfg.buffer_size = cpal::BufferSize::Default;
    let mut r = Runner::new(make(rate, setup), shared.clone());
    let sh = shared.clone();
    let stream = dev
        .build_output_stream::<f32, _, _>(
            cfg,
            move |out: &mut [f32], _| {
                let frames = out.len() / channels;
                r.block(Some(out), frames, channels);
            },
            move |err| {
                if err.kind() == cpal::ErrorKind::Xrun {
                    sh.xruns.fetch_add(1, Ordering::Relaxed);
                }
            },
            None,
        )
        .map_err(|e| format!("could not open sound ({e}); try --audio none"))?;
    stream.play().map_err(|e| format!("could not start sound: {e}"))?;
    shared.rate.store(rate, Ordering::Relaxed);
    shared.running.store(true, Ordering::Release);
    let name = dev.description().ok().map(|d| d.name().to_string()).unwrap_or_else(|| "?".into());
    Ok((Output::Stream(stream), format!("{:?}, {name}, {rate} Hz, {channels} ch", host.id())))
}

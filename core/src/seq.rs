//! §5. The sequencer never sees a frame: its output depends only on the order of ticks and
//! commands. Since ABI 6 (§17) it also writes the lane as steps arrive and plays the four
//! repeats.

use crate::abi::{EV_STEP, Out};
use crate::clock::{TICKS_PER_BEAT, TICKS_PER_STEP};

pub const TRACKS: usize = 16;
pub const STEPS: usize = 16;
pub const SWING_MAX: u32 = 48;
/// §17: a lane entry is 0 (none) or the effect on key 1 to 16.
pub const FX_MAX: u8 = 16;
/// §17: what the sequencer is told about the fx key each tick: fx is not held, so nothing is
/// written. Otherwise the value is what an arriving step takes: the punch, or 0.
pub const NO_WRITE: u8 = 0xFF;
/// `Seq::down` while no step is being played (Session 30).
const NO_DOWN: u8 = 0xFF;

/// §17's table: effects 9 to 12 re-fire a step every this many ticks. 0: not a repeat.
#[inline]
pub const fn repeat_period(effect: u8) -> u32 {
    match effect {
        9 => 96,
        10 => 128,
        11 => 48,
        12 => 64,
        _ => 0,
    }
}

/// §17, designed 2026-11-24 (ABI 13; D93): effect 13, "note shuffle".
pub const SHUFFLE: u8 = 13;

/// §17's draw: at 100 or over it passes with no draw; under, one draw, passing when
/// `draw % 100 < p`.
#[inline]
fn draw(p: u8, rng: &mut Rng) -> bool {
    let p = p as u32;
    p >= 100 || rng.draw() % 100 < p
}

/// §17, designed 2026-11-24 (ABI 13; D93): the step's firing while effect 13 is read. A note
/// of a sound in the effects plays as the sound twelve keys on, round the sixteen; a sound out
/// of the effects plays its own notes too. By rising track of the sound that sounds; each note
/// takes its own draw (its own first), and one `TRIG` says either passed. ⚠ The map is a
/// reading of three sounds (R70): `(m + 4) & 15` is the one line to change. Out of line: the
/// plain firing pays one comparison for it.
#[inline(never)]
fn shuffle(k: u32, cur: usize, pat: &Pattern, fx_sounds: u16, rng: &mut Rng, out: &mut Out) {
    for m in 0..TRACKS {
        let mut fire = false;
        if fx_sounds >> m & 1 == 0 && pat.on[m] >> cur & 1 == 1 {
            fire |= draw(pat.prob[m][cur], rng);
        }
        let s = (m + 4) & 15; // the sound whose note m plays: s + 12 is m
        if fx_sounds >> s & 1 == 1 && pat.on[s] >> cur & 1 == 1 {
            // Both draws are taken, whatever the first gave.
            let d = draw(pat.prob[s][cur], rng);
            fire |= d;
        }
        if fire {
            out.trig(k, m as u8, cur as u8, true);
        }
    }
}

/// §21 (D133 item 1): sixteen patterns, in the engine.
pub const PATTERNS: usize = 16;

/// §21: what a pattern holds. The swing and `fx_sounds` were here until ABI 16; they are the
/// beat's now (the engine's). A cleared pattern (`clear`, `cleared`) is one snapshot version 5
/// does not write.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pattern {
    /// One word per track; bit `s` is step `s`.
    pub on: [u16; TRACKS],
    pub prob: [[u8; STEPS]; TRACKS],
    /// §21 (D133 item 3): each pattern's own, 1 to 16.
    pub length: u8,
    /// §17: the lane, one effect a step, 0 for none. Per pattern since ABI 16 (R90).
    pub fx: [u8; STEPS],
    /// §20: bit `s` of track `t`, step `s` keeps the A and B it was written with,
    /// `kept[t][s]`. Zero where nothing is kept, so two patterns that play alike are equal.
    pub keep: [u16; TRACKS],
    pub kept: [[[u16; 2]; STEPS]; TRACKS],
}

impl Pattern {
    /// §20: step `s` of track `t` forgets what it kept. Says whether it kept anything.
    pub fn forget(&mut self, t: usize, s: usize) -> bool {
        let bit = 1u16 << (s & 15);
        let had = self.keep[t & 15] & bit != 0;
        self.keep[t & 15] &= !bit;
        self.kept[t & 15][s & 15] = [0; 2];
        had
    }

    /// A cleared pattern: every step off, every probability 100, the lane empty, nothing
    /// kept, sixteen steps long. The tests' (`clear` and `cleared` are held to it); the core
    /// builds and checks one with loops instead.
    #[cfg(test)]
    pub const fn new() -> Self {
        Pattern {
            on: [0; TRACKS],
            prob: [[100; STEPS]; TRACKS],
            length: STEPS as u8,
            fx: [0; STEPS],
            keep: [0; TRACKS],
            kept: [[[0; 2]; STEPS]; TRACKS],
        }
    }

    /// Made a cleared pattern again, as `new` is. As loops and out of line, so a bank of sixteen
    /// is not sixteen unrolled copies in the `.wasm`.
    #[inline(never)]
    pub fn clear(&mut self) {
        self.on = [0; TRACKS];
        self.keep = [0; TRACKS];
        self.kept = [[[0; 2]; STEPS]; TRACKS];
        self.fx = [0; STEPS];
        for row in &mut self.prob {
            *row = [100; STEPS];
        }
        self.length = STEPS as u8;
    }

    /// §21: equal to a cleared one (`new`), so snapshot version 5 does not write it. As loops
    /// and out of line, as `Engine::clear` is (Session 42): unrolled, a comparison of 1.3 KB
    /// grows the `.wasm` for a path that is cold.
    #[inline(never)]
    pub fn cleared(&self) -> bool {
        let mut any = self.length as u16 ^ STEPS as u16;
        for t in 0..TRACKS {
            any |= self.on[t] | self.keep[t];
            for s in 0..STEPS {
                any |= (self.prob[t][s] ^ 100) as u16 | self.kept[t][s][0] | self.kept[t][s][1];
            }
        }
        for e in &self.fx {
            any |= *e as u16;
        }
        any == 0
    }

    /// §21's view byte 68: a step on, or a lane entry, in any of its sixteen steps.
    #[inline(never)]
    pub fn used(&self) -> bool {
        let mut any = 0u16;
        for on in &self.on {
            any |= *on;
        }
        for e in &self.fx {
            any |= *e as u16;
        }
        any != 0
    }
}

/// §21: the sixteen patterns, the one playing (`active`), the chain and the next chain. One
/// source of truth: no working copy (D133 item 1).
pub struct Bank {
    pub pats: [Pattern; PATTERNS],
    pub active: u8,
    /// The chain, `chain_len` entries (1 to 16), zero past them; `pos` the one playing.
    pub chain: [u8; PATTERNS],
    pub chain_len: u8,
    pub pos: u8,
    /// The next chain, waiting to land at the end of the bar: `next_len` entries (0 for
    /// none), zero past them.
    pub next: [u8; PATTERNS],
    pub next_len: u8,
}

impl Bank {
    /// At `pi_init`: sixteen cleared patterns, the chain pattern 1 alone, no next chain. Zeroed,
    /// then each cleared in a loop (`Pattern::clear`): a constant bank of sixteen would be 21 KB
    /// of stores or of data in the `.wasm`.
    pub fn new() -> Self {
        const ZERO: Pattern = Pattern { on: [0; TRACKS], prob: [[0; STEPS]; TRACKS], length: 0, fx: [0; STEPS], keep: [0; TRACKS], kept: [[[0; 2]; STEPS]; TRACKS] };
        let mut b = Bank { pats: [ZERO; PATTERNS], active: 0, chain: [0; PATTERNS], chain_len: 1, pos: 0, next: [0; PATTERNS], next_len: 0 };
        for pat in &mut b.pats {
            pat.clear();
        }
        b
    }

    /// The active pattern: what the edits, the keys and the sequencer act on.
    #[inline]
    pub fn pat(&self) -> &Pattern {
        &self.pats[self.active as usize & 15]
    }

    #[inline]
    pub fn pat_mut(&mut self) -> &mut Pattern {
        &mut self.pats[self.active as usize & 15]
    }

    /// §21: the chain a snapshot saves and the grid shows while pattern is held: the next chain
    /// if one waits, else the chain. Zero past its length.
    pub fn shown(&self) -> ([u8; PATTERNS], u8) {
        if self.next_len > 0 { (self.next, self.next_len) } else { (self.chain, self.chain_len) }
    }

    /// §21's view byte 68: bit `p`, pattern `p` is used.
    pub fn used(&self) -> u16 {
        let mut used = 0u16;
        for (p, pat) in self.pats.iter().enumerate() {
            if pat.used() {
                used |= 1 << p;
            }
        }
        used
    }

    /// §21's `PATTERN`: a new chain of pattern `n` alone. Stopped, at once, and any next chain
    /// dropped; playing, it is the next chain. Says whether the chain saved changed.
    pub fn choose(&mut self, n: u8, playing: bool) -> bool {
        let before = self.shown();
        let mut one = [0u8; PATTERNS];
        one[0] = n & 15;
        if playing {
            (self.next, self.next_len) = (one, 1);
        } else {
            (self.chain, self.chain_len, self.pos, self.active) = (one, 1, 0, n & 15);
            (self.next, self.next_len) = ([0; PATTERNS], 0);
        }
        self.shown() != before
    }

    /// §21's `CHAIN`: `n` added to the end of the next chain if one waits, else of the chain;
    /// nothing when that chain already holds sixteen. Says whether anything was added.
    pub fn add(&mut self, n: u8) -> bool {
        let (list, len) = if self.next_len > 0 { (&mut self.next, &mut self.next_len) } else { (&mut self.chain, &mut self.chain_len) };
        if *len as usize >= PATTERNS {
            return false; // a seventeenth is ignored ⚠ (R86: "up to 16")
        }
        list[*len as usize & 15] = n & 15;
        *len += 1;
        true
    }

    /// §21, at a step boundary where the playhead wraps: a next chain lands, or play (and a
    /// reset) restarts the chain, or the chain moves on. Says whether the pattern playing
    /// changed to another, which ends a run. Out of line: once a pattern, not once a tick.
    #[inline(never)]
    fn wrap(&mut self, fresh: bool) -> bool {
        if self.next_len > 0 {
            (self.chain, self.chain_len) = (self.next, self.next_len);
            (self.next, self.next_len) = ([0; PATTERNS], 0);
            self.pos = 0;
        } else if fresh || self.pos + 1 >= self.chain_len {
            self.pos = 0; // play restarts the chain (R90); after the last, it starts over (R86)
        } else {
            self.pos += 1;
        }
        let p = self.chain[self.pos as usize & 15] & 15;
        let changed = p != self.active;
        self.active = p;
        changed
    }
}

/// xorshift32. The draw is the new state.
pub struct Rng(u32);

impl Rng {
    pub const fn new(seed: u32) -> Self {
        Rng(seed)
    }

    #[inline]
    pub fn draw(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}

pub struct Seq {
    playing: bool,
    fresh: bool,
    pending: bool,
    cur: u8,
    offset: u8,
    /// §17: the effect read at the last step boundary, and the one before it (`last`).
    eff: u8,
    last: u8,
    /// §17: the repeat in force, as its period in ticks (0: none), and the run it belongs to:
    /// the step it began on and the tick. A u16 (a period is at most 128 ticks) since Session
    /// 35, so `beats` fits the struct's padding and `pi_size` does not move.
    period: u16,
    run_step: u8,
    /// §17, Session 30 (D95 item 1, Kenny: "as close as possible to the PO"): the last
    /// `write` seen, and the step that was playing when it changed, `NO_DOWN` until a step has
    /// arrived since play or reset. A run begun by a punch still held repeats `down`: the
    /// step the key came down on, not the step after it, which the lane takes.
    wrote: u8,
    down: u8,
    run_tick: u32,
    /// §16: steps woven since play began, or since a reset while playing (D84). Held when
    /// stopped. The view's bytes 48 to 51. Not saved: the moment's.
    woven: u32,
    /// §16: bit `i`, the step woven `i` steps ago had a sound set as it arrived. Bytes 52, 53.
    wove: u16,
    /// §4, ruled 2026-11-25 (D101 item 1): the clock follows one edge a beat, so a pattern not
    /// yet started waits for a multiple of 384. Told by the engine whenever `SOURCE` is applied
    /// (`Clock::span`); not saved, as the source is not.
    beats: bool,
}

impl Seq {
    pub const fn new() -> Self {
        Seq { playing: false, fresh: false, pending: false, cur: 0, offset: 0, eff: 0, last: 0, period: 0, run_step: 0, run_tick: 0, wrote: NO_WRITE, down: NO_DOWN, woven: 0, wove: 0, beats: false }
    }

    /// §16: how many steps the cloth holds, and which of the last sixteen had a sound set.
    pub fn cloth(&self) -> (u32, u16) {
        (self.woven, self.wove)
    }

    /// §17: a new run begins at the next step that holds a repeat. And until a step has
    /// arrived there is no effect read since play: a repeat waits for that step ⚠, as a
    /// plain firing does (`pending`).
    fn end_run(&mut self) {
        self.last = 0;
        self.eff = 0;
        self.period = 0;
        self.wrote = NO_WRITE;
        self.down = NO_DOWN;
    }

    pub fn play(&mut self, on: bool) {
        if on {
            if !self.playing {
                self.playing = true;
                self.fresh = true;
                self.end_run();
                // §16: play starts a new cloth; stop leaves the old one as it is.
                self.woven = 0;
                self.wove = 0;
            }
        } else {
            self.playing = false;
            self.pending = false;
        }
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    /// A step has arrived since play, or since a reset: the playhead's step means something.
    pub fn started(&self) -> bool {
        self.playing && !self.fresh
    }

    /// The pattern step the playhead is on.
    pub fn step(&self) -> u8 {
        self.cur
    }

    /// The ticks an edge stands for, `span`, as the clock has it after a `SOURCE`: 96 or 384.
    /// Told once a command, not read every tick: given every frame, it cost every playing case
    /// about 0.3 ns a frame (Session 35, `LEARNINGS.md`).
    pub fn set_span(&mut self, span: u32) {
        self.beats = span == TICKS_PER_BEAT;
    }

    pub fn reset(&mut self) {
        self.fresh = true;
        self.pending = false;
        self.end_run();
        // §16 (D84): a reset while playing starts a new cloth, as play does, so a diamond
        // still begins on step 1. Stopped, the cloth holds still: play will start the new one.
        if self.playing {
            self.woven = 0;
            self.wove = 0;
        }
    }

    /// One tick. `write` is what an arriving step takes into the lane (`NO_WRITE` while fx
    /// is up). Returns whether the lane changed, which moves the generation. Since ABI 16
    /// (§21) it plays the bank's active pattern, and moves the chain on where it wraps; the
    /// swing and `fx_sounds` are the beat's.
    #[inline]
    pub fn tick(&mut self, k: u32, bank: &mut Bank, swing: u8, fx_sounds: u16, rng: &mut Rng, out: &mut Out, write: u8) -> bool {
        if !self.playing {
            return false;
        }
        let mut lane_changed = false;
        let g = k / TICKS_PER_STEP;
        let t = (k % TICKS_PER_STEP) as u8;
        // §4, ruled 2026-11-25 (D101 item 1): a pattern not yet started begins on a tick that is
        // a multiple of the span, so on one edge a beat play waits for the next edge. Until then
        // a step's tick 0 is any other tick of a pattern not started. With a span of 96 this is
        // `t == 0`, as it always was; a reset sets tick 0, a multiple of both.
        let boundary = t == 0 && !(self.fresh && self.beats && k % TICKS_PER_BEAT != 0);

        // §17, Session 30: where a punch came down. Before a step has arrived since play or
        // reset no step is being played, and a run there begins on its own step.
        if !boundary && write != self.wrote {
            self.wrote = write;
            self.down = if self.fresh { NO_DOWN } else { self.cur };
            // D97 (Kenny: the replay must match the push): a repeat punched mid-step is
            // recorded on the step being played too, so a replay's run begins there and
            // repeats what the hand heard. Sound effects keep §17's rule (D66 item 3): the
            // step being played is not recorded.
            let at = (self.cur & 15) as usize;
            let pat = bank.pat_mut();
            if !self.fresh && repeat_period(write) != 0 && pat.fx[at] != write {
                pat.fx[at] = write;
                lane_changed = true;
            }
        }

        if boundary {
            // §21: where the playhead wraps (or play or a reset begins), a next chain lands or
            // the chain moves on. With a chain of one and none waiting, the same pattern again.
            if self.fresh || self.cur + 1 >= bank.pat().length {
                if bank.wrap(self.fresh) {
                    // A run ends where the playing pattern changes to another, as at play.
                    self.end_run();
                    out.active = bank.active;
                }
                self.cur = 0;
                self.fresh = false;
            } else {
                self.cur += 1;
            }
            out.push(k, EV_STEP, 0, self.cur);
            self.offset = if g & 1 == 1 { swing } else { 0 };
            let pat = bank.pat_mut();
            self.pending = true;

            // §16: the step is woven as it arrives, with whether any sound is set on it now.
            // Set, not fired: a probability or a repeat does not change the cloth. Once a step.
            let mut set = 0u16;
            for on in &pat.on {
                set |= *on;
            }
            self.woven = self.woven.wrapping_add(1);
            self.wove = self.wove << 1 | (set >> (self.cur & 15) & 1);

            // A punch landing on this boundary's frame came down on the step arriving.
            if write != self.wrote {
                self.wrote = write;
                self.down = self.cur;
            }

            // §17: the lane is written by the steps that arrive, then read.
            let cur = (self.cur & 15) as usize;
            if write != NO_WRITE && pat.fx[cur] != write {
                pat.fx[cur] = write;
                lane_changed = true;
            }
            self.eff = pat.fx[cur];
            if self.eff != self.last {
                self.period = repeat_period(self.eff) as u16;
                if self.period != 0 {
                    // §17, Session 30 (D95 item 1, the hardware's rule): a run repeats the step
                    // the key came down on, so an empty step repeats as silence. Begun from the
                    // lane, with no punch held, it repeats its own first step, as built in
                    // Session 16. A new repeat is a new run, from its own key's step.
                    self.run_step = if write == self.eff && self.down != NO_DOWN { self.down } else { self.cur };
                    self.run_tick = k;
                }
                self.last = self.eff;
            }
        }

        let pat = bank.pat();
        if self.period == 0 {
            // §5, as it was before ABI 6: an empty lane costs the one comparison above.
            if self.pending && t == self.offset {
                self.pending = false;
                let cur = (self.cur & 15) as usize;
                if self.eff == SHUFFLE {
                    // §17, designed 2026-11-24: effect 13 takes the plain firing's place.
                    shuffle(k, cur, pat, fx_sounds, rng, out);
                    return lane_changed;
                }
                for track in 0..TRACKS {
                    if pat.on[track] >> cur & 1 == 1 {
                        let p = pat.prob[track][cur] as u32;
                        if p >= 100 || rng.draw() % 100 < p {
                            out.trig(k, track as u8, cur as u8, true);
                        }
                    }
                }
            }
        } else {
            self.repeat(k, t, pat, fx_sounds, rng, out);
        }
        lane_changed
    }

    /// §17's four repeats. A sound in the effects fires from the run's column on the run's
    /// own grid, unswung; one outside plays its own step as §5 says. Rising track order, one
    /// draw a firing at under 100, repeats and plain firings together.
    fn repeat(&mut self, k: u32, t: u8, pat: &Pattern, fx_sounds: u16, rng: &mut Rng, out: &mut Out) {
        let hit = k.wrapping_sub(self.run_tick) % self.period as u32 == 0;
        let plain = self.pending && t == self.offset;
        if plain {
            self.pending = false;
        }
        if !hit && !plain {
            return;
        }
        let (cur, run) = ((self.cur & 15) as usize, (self.run_step & 15) as usize);
        for track in 0..TRACKS {
            let column = if fx_sounds >> track & 1 == 1 {
                if !hit {
                    continue;
                }
                run
            } else {
                if !plain {
                    continue;
                }
                cur
            };
            if pat.on[track] >> column & 1 == 1 {
                let p = pat.prob[track][column] as u32;
                if p >= 100 || rng.draw() % 100 < p {
                    out.trig(k, track as u8, column as u8, true);
                }
            }
        }
    }
}

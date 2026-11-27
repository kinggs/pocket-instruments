//! §21 (ABI 16; D133): sixteen patterns, the chain, copy and clear, the view's bytes 64 to 87
//! and snapshot version 5. One test a row or bullet of §21, in its order. The four
//! `fixtures/patterns-*.json` walk the same rules through every harness; these hold each rule
//! alone (LEARNINGS.md, Session 10: a fixture should not be the only thing holding a rule).

use super::sound::render;
use super::*;
use crate::crc::crc32;
use crate::seq::Pattern;

const SR: u32 = 48_000; // at 120 bpm a step is 6000 frames
const STEP: u64 = 6000;
const BAR: u64 = 16 * STEP;
const PATTERN: u8 = 17;
const FUNCTION: u8 = 19;
const FX: u8 = 20;
const WRITE: u8 = 22;

fn down(key: u8) -> Cmd {
    cmd(CMD_KEY_DOWN, key, 0, 0)
}
fn up(key: u8) -> Cmd {
    cmd(CMD_KEY_UP, key, 0, 0)
}
fn pattern(p: u32) -> Cmd {
    cmd(CMD_PATTERN, 0, 0, p)
}
fn chain(p: u32) -> Cmd {
    cmd(CMD_CHAIN, 0, 0, p)
}
fn copy(p: u32) -> Cmd {
    cmd(CMD_COPY, 0, 0, p)
}
fn on(track: u8, step: u8) -> Cmd {
    cmd(CMD_STEP, track, step, 1)
}
fn play() -> Cmd {
    cmd(CMD_PLAY, 0, 0, 1)
}
fn stop() -> Cmd {
    cmd(CMD_PLAY, 0, 0, 0)
}

/// The view's patterns: (active, pos, the chain, the next chain, used), as `view_text_pat`'s
/// digits.
fn pats(h: &Host) -> (u8, u8, String, String, u16) {
    let v = h.view();
    let nib = |at: usize, len: u8| -> String { (0..len as usize).map(|i| format!("{:x}", v[at + i / 2] >> (4 * (i & 1)) & 15)).collect() };
    (v[64], v[65], nib(72, v[66]), nib(80, v[67]), u16::from_le_bytes([v[68], v[69]]))
}

/// (tick, track, step) of each firing.
fn trigs(ev: &[(u64, Event)]) -> Vec<(u32, u8, u8)> {
    ev.iter().filter(|(_, e)| e.kind == EV_TRIG).map(|(_, e)| (e.tick, e.track, e.step)).collect()
}

fn u16_at(s: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([s[i], s[i + 1]])
}

fn recrc(s: &mut [u8]) {
    let n = s.len() - 4;
    let crc = crc32(&s[..n]);
    s[n..].copy_from_slice(&crc.to_le_bytes());
}

/// Three patterns of sixteen steps, each a different track on its step 0: pattern 1 track 0,
/// pattern 2 track 1, pattern 3 track 2. Pattern 1 active again, the chain pattern 1 alone.
fn three() -> Host {
    let mut h = Host::new(SR, 1);
    h.block(1, &[on(0, 0), pattern(1), on(1, 0), pattern(2), on(2, 0), pattern(0)]);
    h
}

/// Which track fired on each step 0 the run played: the pattern playing there.
fn heads(ev: &[(u64, Event)]) -> Vec<u8> {
    trigs(ev).iter().filter(|t| t.2 == 0).map(|t| t.1).collect()
}

// ---- What a pattern holds, and what stays global ----

#[test]
fn the_edits_of_a_step_the_lane_and_the_length_act_on_the_active_pattern_and_the_rest_are_the_beat_s() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[pattern(1), on(3, 4), cmd(CMD_PROB, 3, 4, 40), cmd(CMD_FX, 4, 0, 9), cmd(CMD_KEEP, 3, 4, 7 << 16 | 8), cmd(CMD_LENGTH, 0, 0, 5)]);
    h.block(1, &[cmd(CMD_SWING, 0, 0, 20), cmd(CMD_TEMPO, 0, 0, 99_000), cmd(CMD_SOUND, 2, 0, 1234), cmd(CMD_VOICE, 5, 0, 1), cmd(CMD_FXSOUND, 6, 0, 0)]);
    let in_two = h.view();
    assert_eq!((in_two[28], in_two[29]), (5, 20), "pattern 2's length; the beat's swing");
    assert_eq!(u16_at(&in_two, 56), 1 << 4, "pattern 2's lane");
    h.block(1, &[pattern(0)]);
    let in_one = h.view();
    assert_eq!(in_one[28], 16, "the view's len is the active pattern's: pattern 1 still has sixteen");
    assert_eq!(u16_at(&in_one, 56), 0, "pattern 1's lane is empty");
    assert_eq!(u16_at(&in_one, 12), 0, "and its steps");
    // The global ones are the same from either pattern.
    for (i, what) in [(4, "tempo"), (18, "sampled"), (29, "swing"), (58, "fx sounds")] {
        assert_eq!(in_one[i..i + 2], in_two[i..i + 2], "{what}");
    }
    let s = h.snap();
    assert_eq!(u16_at(&s, 92), 1 << 1, "only pattern 2 written");
    let p = &s[272..272 + SNAP5_BLOCK];
    assert_eq!((u16_at(p, 2 * 3), p[32 + 16 * 3 + 4], p[288 + 4], u16_at(p, 304 + 6), p[1360]), (1 << 4, 40, 9, 1 << 4, 5));
    assert_eq!((u16_at(&s, 24 + 8), s[20], u16_at(&s, 88), u16_at(&s, 90)), (1234, 20, 1 << 5, !(1 << 6)), "the beat's, once");
}

#[test]
fn at_pi_init_the_chain_is_pattern_1_alone_at_its_first_with_none_waiting() {
    let h = Host::new(SR, 1);
    assert_eq!(pats(&h), (0, 0, "0".into(), "".into(), 0));
    assert_eq!(h.snap().len(), SNAP5_MIN, "nothing written");
}

// ---- The chain ----

#[test]
fn each_wrap_moves_the_chain_on_and_after_the_last_it_starts_over() {
    let mut h = three();
    h.block(1, &[pattern(1), chain(0), chain(2), chain(2), play()]); // 2 1 3 3, stopped
    let g = h.generation();
    let ev = h.run(5 * BAR, 128, &[]);
    assert_eq!(heads(&ev), [1, 0, 2, 2, 1], "2, 1, 3, 3 (a repeat), then 2 again (R86)");
    assert_eq!(h.generation(), g, "a chain advance moves no generation");
    assert_eq!(pats(&h).1, 0, "pos 0 in the fifth bar");
}

#[test]
fn a_pattern_shorter_than_sixteen_moves_the_chain_on_at_its_own_end() {
    let mut h = three();
    h.block(1, &[cmd(CMD_LENGTH, 0, 0, 3), pattern(1), cmd(CMD_LENGTH, 0, 0, 5), pattern(0), chain(1), play()]);
    let ev = h.run(16 * STEP, 128, &[]);
    let starts: Vec<(u32, u8)> = trigs(&ev).iter().filter(|t| t.2 == 0).map(|t| (t.0, t.1)).collect();
    // Play lands a frame after the first step's tick 0, so it begins on the next: tick 96.
    assert_eq!(starts, [(96, 0), (384, 1), (864, 0), (1152, 1)], "3 steps, 5 steps, and round");
    assert_eq!(h.view()[28], 5, "the view's len follows the active pattern: pattern 2's, at the end");
}

#[test]
fn a_choice_while_playing_waits_and_lands_where_the_playing_pattern_wraps() {
    let mut h = three();
    h.block(1, &[play()]);
    h.run(5 * STEP, 128, &[]);
    h.block(1, &[pattern(2)]);
    assert_eq!(pats(&h), (0, 0, "0".into(), "2".into(), 0b111), "waiting");
    let ev = h.run(2 * BAR, 128, &[]);
    assert_eq!(heads(&ev), [2, 2], "landed at the bar, and a chain of one plays on");
    assert_eq!(trigs(&ev)[0].0, 96 + 1536, "where pattern 1's bar ends (it began at tick 96)");
    assert_eq!(pats(&h), (2, 0, "2".into(), "".into(), 0b111));
}

#[test]
fn stopped_a_choice_lands_at_once_and_drops_a_next_chain() {
    let mut h = three();
    h.block(1, &[play()]);
    h.run(STEP, 128, &[]);
    h.block(1, &[pattern(2), chain(1)]);
    assert_eq!(pats(&h).3, "21");
    h.block(1, &[stop()]);
    assert_eq!(pats(&h).3, "21", "a next chain not landed stays when play stops");
    h.block(1, &[pattern(1)]);
    assert_eq!(pats(&h), (1, 0, "1".into(), "".into(), 0b111), "stopped: at once, and the next chain dropped");
}

#[test]
fn a_next_chain_not_landed_when_play_stops_lands_at_the_next_play_s_first_step() {
    let mut h = three();
    h.block(1, &[play()]);
    h.run(STEP, 128, &[]);
    h.block(1, &[pattern(2), chain(1), stop()]);
    h.run(2 * STEP, 128, &[]);
    h.block(1, &[play()]);
    let ev = h.run(2 * BAR, 128, &[]);
    assert_eq!(heads(&ev), [2, 1]);
    assert_eq!(pats(&h).2, "21");
}

#[test]
fn choosing_one_pattern_while_a_chain_runs_replaces_the_chain_at_the_bar_and_after_stop_and_play() {
    let mut h = three();
    h.block(1, &[chain(1), play()]); // 1 2
    h.run(STEP, 128, &[]);
    h.block(1, &[pattern(2)]);
    let ev = h.run(3 * BAR, 128, &[]);
    assert_eq!(heads(&ev), [2, 2, 2], "pattern 2 never plays: the chain was replaced at the end of the bar");
    h.block(1, &[stop()]);
    h.run(STEP, 128, &[]);
    h.block(1, &[play()]);
    let ev = h.run(2 * BAR, 128, &[]);
    assert_eq!(heads(&ev), [2, 2], "replaced, not paused (R90: \"Same when I stop then start\")");
}

#[test]
fn play_restarts_the_chain_from_its_first_pattern() {
    let mut h = three();
    h.block(1, &[chain(1), chain(2), play()]);
    h.run(BAR + 3 * STEP, 128, &[]);
    assert_eq!(pats(&h).0, 1, "in the chain's second");
    h.block(1, &[stop()]);
    assert_eq!(pats(&h), (1, 1, "012".into(), "".into(), 0b111), "stopped where it was");
    h.block(1, &[play()]);
    let ev = h.run(BAR, 128, &[]);
    assert_eq!(heads(&ev), [0], "from pattern 1 (R90: \"Play restarts the chain yes\")");
}

#[test]
fn a_reset_restarts_the_chain_and_a_next_chain_lands_at_its_first_step() {
    let mut h = three();
    h.block(1, &[chain(1), chain(2), play()]);
    h.run(BAR + 3 * STEP, 128, &[]);
    let reset = cmd(CMD_RESET, 0, 0, 0);
    let ev = h.run(BAR - STEP, 128, &[(h.now, reset)]);
    assert_eq!(heads(&ev), [0], "a reset starts the chain from its first, as play does");
    let ev = h.run(BAR - STEP, 128, &[(h.now, pattern(2)), (h.now, reset)]);
    assert_eq!(heads(&ev), [2], "and a next chain lands on the reset's first step");
}

#[test]
fn a_chain_holds_sixteen_and_a_seventeenth_is_ignored() {
    let mut h = Host::new(SR, 1);
    let mut keys = vec![down(PATTERN)];
    for n in 0..17u8 {
        keys.extend([down(n % 16), up(n % 16)]);
    }
    h.block(1, &keys);
    assert_eq!(pats(&h).2, "0123456789abcdef", "sixteen by keys; the seventeenth (key 1 again) ignored ⚠");
    let g = h.generation();
    h.block(1, &[chain(3)]);
    assert_eq!((pats(&h).2.len(), h.generation()), (16, g), "and by the edit");
    // Playing, the same for the next chain.
    h.block(1, &[up(PATTERN), play(), pattern(0)]);
    for p in 1..17 {
        h.block(1, &[chain(p % 16)]);
    }
    assert_eq!(pats(&h).3, "0123456789abcdef");
}

#[test]
fn the_chain_gives_the_same_trace_on_the_table_and_on_edges() {
    let run = |source: u32| {
        let mut h = three();
        let t0 = h.now;
        let mut cmds = vec![(t0, cmd(CMD_SOURCE, 0, 0, source)), (t0, chain(2)), (t0, chain(1)), (t0, cmd(CMD_LENGTH, 0, 0, 7)), (t0, play())];
        if source == 1 {
            cmds.extend((0..40).map(|m| (t0 + m * STEP, cmd(CMD_CLOCK, 0, 0, 0))));
        }
        cmds.push((t0 + 9 * STEP + 77, pattern(1)));
        cmds.sort_by_key(|c| c.0);
        h.run(40 * STEP, 128, &cmds).iter().map(|(_, e)| line(e)).collect::<String>()
    };
    let table = run(0);
    assert!(table.contains("trig 1 0"), "pattern 2 played");
    assert_eq!(table, run(1));
}

// ---- Across a change ----

#[test]
fn a_sounding_note_rings_on_across_a_change() {
    // Pattern 1: the crash cymbal (key 12, a long one) on step 15; pattern 2 empty. The same
    // sound as a beat of one pattern with that step alone, up to the next bar's end: the change
    // touched no voice (D56).
    let cmds = |chained: bool| {
        let mut c = vec![(0, on(11, 15))];
        if chained {
            c.push((0, chain(1)));
        }
        c.push((0, play()));
        c
    };
    let (mut a, mut b) = (Host::new(SR, 1), Host::new(SR, 1));
    let (x, tx) = render(&mut a, (2 * BAR) as usize, 128, 1, &cmds(true));
    let (y, ty) = render(&mut b, (2 * BAR) as usize, 128, 1, &cmds(false));
    assert_eq!(tx.len(), 1, "one hit, in pattern 1");
    assert_eq!(ty.len(), 2, "the beat of one pattern hits again in its second bar");
    let to = (BAR + 15 * STEP) as usize; // before that second hit
    assert!(x[..to].iter().zip(&y[..to]).all(|(p, q)| p.to_bits() == q.to_bits()), "the same samples");
    assert!(x[BAR as usize..BAR as usize + 100].iter().any(|s| *s != 0.0), "and still sounding after the change");
}

#[test]
fn a_held_effect_stays_heard_and_is_written_into_the_new_pattern_s_lane() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[chain(1), play()]);
    let ev = h.run(14 * STEP + 10, 128, &[(14 * STEP + 5, down(FX)), (14 * STEP + 5, down(3))]); // effect 4 punched in step 14
    assert!(ev.iter().all(|(_, e)| e.kind == EV_STEP));
    h.run(4 * STEP, 128, &[]); // into pattern 2, steps 0 to 2
    assert_eq!((h.view()[64], h.view()[55]), (1, 4), "pattern 2 plays, effect 4 heard");
    h.block(1, &[up(3), up(FX)]);
    let s = h.snap();
    assert_eq!(u16_at(&s, 92), 0b11, "both patterns written: each has lane entries");
    let lane = |p: usize| s[272 + p * SNAP5_BLOCK + 288..272 + p * SNAP5_BLOCK + 304].to_vec();
    assert_eq!(lane(0)[15], 4, "pattern 1's step 15 took it");
    assert_eq!(lane(0)[14], 0, "a sound effect does not write the step it came down in (D66 item 3)");
    assert_eq!(lane(1)[..4], [4, 4, 4, 0], "pattern 2's steps 0 to 2 took it");
}

#[test]
fn the_lane_that_plays_is_the_active_pattern_s_and_coming_back_finds_it() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[cmd(CMD_FX, 0, 0, 2), pattern(1), cmd(CMD_FX, 0, 0, 5), pattern(0), chain(1), play()]);
    h.run(STEP / 2, 128, &[]);
    assert_eq!(h.view()[55], 2, "pattern 1's step 0");
    h.run(BAR, 128, &[]);
    assert_eq!(h.view()[55], 5, "pattern 2's step 0");
    h.run(BAR, 128, &[]);
    assert_eq!(h.view()[55], 2, "pattern 1's lane as it was (R90)");
}

#[test]
fn a_run_ends_where_the_playing_pattern_changes_and_carries_on_across_the_same_one() {
    // Track 0 on every step of both patterns; effect 9 (a repeat every 96 ticks, of the run's
    // first step) on steps 15 and 0 of pattern 1 and on step 0 of pattern 2. The chain is 1 2,
    // or, the second time, 1 1. Play is in the first block, at tick 0: the wrap is at 1536.
    let run = |next: u32| {
        let mut h = Host::new(SR, 1);
        let mut setup = Vec::new();
        for p in [1u32, 0] {
            setup.push(pattern(p));
            setup.extend((0..16).map(|s| on(0, s)));
            setup.push(cmd(CMD_FX, 0, 0, 9));
        }
        setup.extend([cmd(CMD_FX, 15, 0, 9), chain(next), play()]);
        h.block(1, &setup);
        trigs(&h.run(BAR + 2 * STEP, 128, &[]))
    };
    let at = |t: &[(u32, u8, u8)], tick: u32| t.iter().find(|x| x.0 == tick).map(|x| x.2);
    let changed = run(1);
    assert_eq!(at(&changed, 1440), Some(15), "the run begun on pattern 1's step 15");
    assert_eq!(at(&changed, 1536), Some(0), "a new run on pattern 2's step 0: the run ended where the pattern changed");
    let same = run(0);
    assert_eq!(at(&same, 1536), Some(15), "a chain entry that repeats the same pattern is an ordinary wrap: the run carries on");
}

#[test]
fn a_punch_held_across_a_change_begins_a_run_on_the_new_pattern_s_step_0() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[on(0, 13), pattern(1), on(0, 0), pattern(0), chain(1), play()]);
    // Effect 9 punched on step 13 and held across the change.
    let ev = h.run(BAR + 4 * STEP, 128, &[(13 * STEP + 5, down(FX)), (13 * STEP + 5, down(8))]);
    let after: Vec<_> = trigs(&ev).into_iter().filter(|t| t.0 >= 1536).collect();
    assert!(!after.is_empty() && after.iter().all(|t| t.2 == 0), "pattern 2's step 0 repeated, not pattern 1's step 13: {after:?}");
}

#[test]
fn effect_13_and_the_sound_effects_follow_the_new_lane_at_once() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[cmd(CMD_FX, 15, 0, 13), pattern(1), cmd(CMD_FX, 0, 0, 6), pattern(0), chain(1), play()]);
    h.run(15 * STEP + 10, 128, &[]);
    assert_eq!(h.view()[55], 13);
    h.run(STEP, 128, &[]);
    assert_eq!(h.view()[55], 6, "pattern 2's step 0, at once");
}

#[test]
fn a_step_fired_just_before_a_change_on_the_same_frame_plays_what_its_own_pattern_keeps() {
    // On edges, swing 48: step 15 fires half a step late. An early edge flushes that firing and
    // the next step, the change to pattern 2, into one frame (§4). Pattern 1's step 15 keeps
    // (1000, 2000); pattern 2's step 15 keeps (5000, 6000) and its step 0 is silent. The mark
    // (the view's bytes 60 to 63) is what track 0 was last hit with: pattern 1's.
    let mut h = Host::new(SR, 1);
    let keep = |a: u32, b: u32| cmd(CMD_KEEP, 0, 15, a << 16 | b);
    h.block(1, &[cmd(CMD_SOURCE, 0, 0, 1), cmd(CMD_SWING, 0, 0, 48), on(0, 15), keep(1000, 2000), pattern(1), on(0, 15), keep(5000, 6000), pattern(0), chain(1), play()]);
    let mut cmds: Vec<(u64, Cmd)> = (0..16).map(|m| (1 + m * STEP, cmd(CMD_CLOCK, 0, 0, 0))).collect();
    cmds.push((1 + 15 * STEP + 2000, cmd(CMD_CLOCK, 0, 0, 0))); // early: before step 15's firing
    let ev = h.run(15 * STEP + 2100, 128, &cmds);
    let last: Vec<_> = ev.iter().rev().take(2).map(|(f, e)| (*f, e.kind, e.step)).collect();
    assert_eq!(last[0].0, last[1].0, "the firing and the next step on one frame: {last:?}");
    assert_eq!((last[1].1, last[0].1, last[0].2), (EV_TRIG, EV_STEP, 0));
    assert_eq!(h.view()[64], 1, "pattern 2 is playing");
    assert_eq!(h.view()[60..64], [0xe8, 0x03, 0xd0, 0x07], "hit with pattern 1's (1000, 2000)");
}

// ---- The keys ----

#[test]
fn pattern_and_write_held_a_grid_key_copies_and_puts_write_mode_back() {
    for write_before in [false, true] {
        let mut h = Host::new(SR, 1);
        if write_before {
            h.block(1, &[down(WRITE), up(WRITE)]);
        }
        h.block(1, &[on(4, 4), cmd(CMD_FX, 4, 0, 3)]);
        let g = h.generation();
        h.block(1, &[down(WRITE), down(PATTERN), down(6), up(6)]);
        assert_eq!(h.view()[24] >> 1 & 1, write_before as u8, "write mode back to what it was (D133 item 6 ⚠)");
        assert_eq!((pats(&h).0, pats(&h).4, h.generation()), (0, 0b100_0001, g + 1), "pattern 7 a copy; pattern 1 stays active");
        // A copy key counts as no chain key: the next grid key begins a new chain.
        h.block(1, &[up(WRITE), down(2), up(2), down(1), up(1)]);
        assert_eq!(pats(&h).2, "21");
        // Copying a pattern onto itself does nothing (and write mode still goes back ⚠).
        h.block(1, &[up(PATTERN), down(WRITE), down(PATTERN)]);
        let (g, s) = (h.generation(), h.snap());
        h.block(1, &[down(2), up(2)]);
        assert_eq!((h.generation(), h.snap()), (g, s), "pattern 3 onto pattern 3");
        assert_eq!(h.view()[24] >> 1 & 1, write_before as u8);
    }
}

#[test]
fn pattern_held_the_first_grid_key_chooses_and_each_further_one_chains() {
    let mut h = three();
    let g = h.generation();
    h.block(1, &[down(PATTERN), down(2), up(2)]);
    assert_eq!((pats(&h).0, pats(&h).2.as_str(), h.generation()), (2, "2", g + 1), "a new chain of pattern 3, at once (stopped)");
    h.block(1, &[down(0), up(0), down(0), up(0)]);
    assert_eq!(pats(&h).2, "200", "repeats allowed");
    // A new press of pattern begins a new chain.
    h.block(1, &[up(PATTERN), down(PATTERN), down(1), up(1), up(PATTERN)]);
    assert_eq!(pats(&h).2, "1");
    // KEY_UP on pattern: nothing else.
    let (s, g) = (h.snap(), h.generation());
    h.block(1, &[down(PATTERN), up(PATTERN)]);
    assert_eq!((h.snap(), h.generation()), (s, g));
    // Function held as well: pattern's row comes first (§21's order).
    h.block(1, &[down(FUNCTION), down(PATTERN), down(0), up(0)]);
    assert_eq!(pats(&h).2, "0");
}

#[test]
fn function_and_pattern_clear_the_active_pattern_only() {
    let mut h = three();
    h.block(1, &[pattern(1), cmd(CMD_LENGTH, 0, 0, 9), cmd(CMD_FX, 3, 0, 7), cmd(CMD_PROB, 1, 0, 30), cmd(CMD_KEEP, 1, 0, 5), cmd(CMD_SWING, 0, 0, 11)]);
    h.block(1, &[chain(2)]);
    let before = h.snap();
    let g = h.generation();
    h.block(1, &[down(FUNCTION), down(PATTERN), up(PATTERN), up(FUNCTION)]);
    assert_eq!(h.generation(), g + 1);
    let s = h.snap();
    assert_eq!((u16_at(&s, 92), s.len()), (0b111, before.len()), "pattern 2 cleared but for its length, 9, so still written");
    let p = &s[272 + SNAP5_BLOCK..272 + 2 * SNAP5_BLOCK];
    assert!(p[..32].iter().all(|b| *b == 0) && p[32..288].iter().all(|b| *b == 100) && p[288..1360].iter().all(|b| *b == 0));
    assert_eq!(p[1360], 9, "its length stays");
    assert_eq!(s[..272], before[..272], "the chain and every global setting stay");
    assert_eq!(s[272..272 + SNAP5_BLOCK], before[272..272 + SNAP5_BLOCK], "pattern 1 stays");
    let p3 = 272 + 2 * SNAP5_BLOCK;
    assert_eq!(s[p3..p3 + SNAP5_BLOCK], before[p3..p3 + SNAP5_BLOCK], "pattern 3 stays");
}

// ---- The edits ----

#[test]
fn an_edit_past_15_is_ignored() {
    let mut h = three();
    let (s, g) = (h.snap(), h.generation());
    h.block(1, &[pattern(16), chain(16), copy(16), pattern(u32::MAX), chain(255), copy(1 << 20)]);
    assert_eq!((h.snap(), h.generation(), pats(&h).0), (s, g, 0));
}

#[test]
fn the_generation_moves_when_the_chain_saved_or_a_slot_s_bytes_change_and_not_at_a_landing() {
    let mut h = three();
    let g = h.generation();
    h.block(1, &[pattern(0)]);
    assert_eq!(h.generation(), g, "the chain saved is already pattern 1 alone");
    h.block(1, &[copy(0)]);
    assert_eq!(h.generation(), g, "a copy onto itself changes no byte");
    h.block(1, &[copy(5)]);
    assert_eq!(h.generation(), g + 1);
    h.block(1, &[copy(5)]);
    assert_eq!(h.generation(), g + 1, "the slot already holds those bytes");
    h.block(1, &[play(), pattern(2)]);
    assert_eq!(h.generation(), g + 2, "a next chain is the chain saved");
    h.block(1, &[pattern(2)]);
    assert_eq!(h.generation(), g + 2, "the same next chain again");
    let s = h.snap();
    h.run(BAR + STEP, 128, &[]);
    assert_eq!(pats(&h).0, 2, "landed");
    assert_eq!((h.generation(), h.snap()), (g + 2, s), "a landing moves nothing: what is saved was already the new chain");
}

// ---- Copy ----

#[test]
fn a_copy_takes_everything_a_pattern_holds_its_lane_too_and_the_active_one_stays() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[on(2, 3), cmd(CMD_PROB, 2, 3, 61), cmd(CMD_KEEP, 2, 3, 0x0102_0304), cmd(CMD_FX, 7, 0, 12), cmd(CMD_LENGTH, 0, 0, 11), copy(9)]);
    assert_eq!(pats(&h).0, 0, "pattern 1 stays active (R86)");
    let s = h.snap();
    assert_eq!(u16_at(&s, 92), 1 | 1 << 9);
    assert_eq!(s[272..272 + SNAP5_BLOCK], s[272 + SNAP5_BLOCK..272 + 2 * SNAP5_BLOCK], "the same block, lane and length included (R90: \"Yes, effects copy\")");
    // While a next chain waits, a copy copies the pattern playing.
    h.block(1, &[play(), pattern(9), cmd(CMD_STEP, 2, 3, 0), copy(4)]);
    let s = h.snap();
    assert_eq!(u16_at(&s, 92), 1 | 1 << 4 | 1 << 9);
    assert_eq!(u16_at(&s, 272 + 2 * 2), 0, "pattern 1, edited");
    assert_eq!(s[272..272 + SNAP5_BLOCK], s[272 + SNAP5_BLOCK..272 + 2 * SNAP5_BLOCK], "pattern 5 is pattern 1 as it plays, not pattern 10");
}

// ---- The view ----

#[test]
fn the_view_s_bytes_64_to_87_say_the_pattern_the_chain_the_next_chain_and_the_used_ones() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[cmd(CMD_FX, 2, 0, 1), pattern(13), on(0, 0), pattern(4)]);
    for p in [10, 15, 1, 0, 2] {
        h.block(1, &[chain(p)]);
    }
    let v = h.view();
    assert_eq!(v[64..72], [4, 0, 6, 0, 0x01, 0x20, 0, 0], "pattern 5, at the first of six, none waiting; patterns 1 (a lane entry) and 14 (a step) used");
    assert_eq!(v[72..80], [0xa4, 0x1f, 0x20, 0, 0, 0, 0, 0], "4 a f 1 0 2, two a byte, low first");
    assert_eq!(v[80..88], [0; 8]);
    h.block(1, &[play(), pattern(7), chain(3), chain(12)]);
    let v = h.view();
    assert_eq!(v[67], 3);
    assert_eq!(v[80..88], [0x37, 0x0c, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn pattern_held_the_grid_shows_the_used_patterns_the_chain_shown_and_the_active_one() {
    let mut h = three();
    h.block(1, &[chain(2), chain(2), down(PATTERN)]);
    let v = h.view();
    assert_eq!((u16_at(&v, 12), u16_at(&v, 14), u16_at(&v, 16), v[24] >> 4 & 1), (0b111, 0b101, 1, 0), "used 1 to 3; the chain 1 3 3; pattern 1");
    h.block(1, &[up(PATTERN), play(), pattern(1), down(PATTERN)]);
    let v = h.view();
    assert_eq!((u16_at(&v, 14), u16_at(&v, 16)), (0b10, 1), "the next chain while one waits (⚠ open (chain lights): the default)");
}

#[test]
fn the_weave_takes_a_view_of_64_bytes_or_more_and_reads_only_the_first_64() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[on(0, 0), copy(3), chain(3), play()]);
    h.run(BAR + 3 * STEP, 128, &[]);
    let v = h.view();
    let mut cells = [[0u8; PI_WEAVE_CELLS]; 3];
    let mut other = v;
    other[64..].fill(0xa5);
    unsafe {
        assert_eq!(pi_weave(v.as_ptr(), VIEW as u32, 0, cells[0].as_mut_ptr(), PI_WEAVE_CELLS as u32), PI_WEAVE_CELLS as u32);
        assert_eq!(pi_weave(v.as_ptr(), 64, 0, cells[1].as_mut_ptr(), PI_WEAVE_CELLS as u32), PI_WEAVE_CELLS as u32);
        assert_eq!(pi_weave(other.as_ptr(), VIEW as u32, 0, cells[2].as_mut_ptr(), PI_WEAVE_CELLS as u32), PI_WEAVE_CELLS as u32);
        assert_eq!(pi_weave(v.as_ptr(), 63, 0, cells[2].as_mut_ptr(), PI_WEAVE_CELLS as u32), 0);
    }
    assert!(cells[0] == cells[1] && cells[1] == cells[2]);
}

// ---- Snapshot version 5 ----

const PI_WEAVE_CELLS: usize = 78 * 15;

#[test]
fn version_5_is_276_bytes_and_1364_a_pattern_written_and_a_cleared_one_is_not_written() {
    let mut h = Host::new(SR, 1);
    assert_eq!(h.snap().len(), 276);
    h.block(1, &[on(0, 0)]);
    assert_eq!(h.snap().len(), 1640, "one pattern");
    h.block(1, &[copy(1), copy(2), copy(15)]);
    assert_eq!(h.snap().len(), 5732, "four");
    h.block(1, &[cmd(CMD_STEP, 0, 0, 0)]);
    assert_eq!(h.snap().len(), 4368, "pattern 1 cleared again: three");
    h.block(1, &[cmd(CMD_LENGTH, 0, 0, 15)]);
    assert_eq!(h.snap().len(), 5732, "a length that is not sixteen is not a cleared pattern");
    for p in 0..16 {
        h.block(1, &[pattern(p), cmd(CMD_PROB, 5, 5, 99)]);
    }
    assert_eq!(h.snap().len(), SNAP5_MAX, "all sixteen: 22 100");
}

#[test]
fn cleared_and_clear_agree_with_a_new_pattern() {
    let mut p = Pattern::new();
    assert!(p.cleared());
    for poke in 0..6 {
        let mut q = Pattern::new();
        match poke {
            0 => q.on[3] = 1,
            1 => q.prob[15][15] = 99,
            2 => q.length = 1,
            3 => q.fx[9] = 16,
            4 => q.keep[0] = 1 << 15,
            _ => q.kept[15][0][1] = 1,
        }
        assert!(!q.cleared(), "poke {poke}");
        q.clear();
        assert!(q == Pattern::new(), "poke {poke}");
    }
    p.on[0] = 1;
    p.clear();
    assert!(p == Pattern::new());
}

#[test]
fn version_5_s_layout() {
    let mut h = Host::new(SR, 0x0102_0304);
    let mut c = vec![cmd(CMD_TEMPO, 0, 0, 133_700), cmd(CMD_SWING, 0, 0, 17), cmd(CMD_SOUND, 3, 0, 0xabcd), cmd(CMD_SOUND, 3, 1, 0x1234), cmd(CMD_VOICE, 2, 0, 1), cmd(CMD_FXSOUND, 1, 0, 0)];
    c.extend([down(18), down(6), up(6), up(18), down(16), down(9), up(9), up(16)]); // volume 7, sound 10
    c.extend([pattern(3), on(5, 6), cmd(CMD_PROB, 5, 6, 44), cmd(CMD_FX, 6, 0, 11), cmd(CMD_KEEP, 5, 6, 0x0506_0708), cmd(CMD_LENGTH, 0, 0, 13)]);
    c.extend([copy(12), pattern(12), chain(3), chain(12)]);
    h.block(1, &c);
    let s = h.snap();
    assert_eq!(s.len(), 276 + 2 * 1364);
    assert_eq!(s[..12], [b'P', b'K', b'I', b'N', 5, 0, 1, 0, 0xac, 0x0b, 0, 0], "magic, version 5, kind 1, payload 260 + 2 × 1364 = 2988");
    assert_eq!(s[12..16], 0x0102_0304u32.to_le_bytes());
    assert_eq!(s[16..20], 133_700u32.to_le_bytes());
    assert_eq!(s[20..24], [17, 7, 9, 0], "swing, volume, the chosen sound, the hat cut");
    assert_eq!(s[24 + 12..24 + 16], [0xcd, 0xab, 0x34, 0x12], "sound 4's sliders, A then B");
    assert_eq!(s[24..28], [0, 0x80, 0, 0x80], "sound 1's at their middles");
    assert_eq!((u16_at(&s, 88), u16_at(&s, 90), u16_at(&s, 92)), (1 << 2, !(1 << 1), 1 << 3 | 1 << 12));
    assert_eq!(s[94..100], [3, 0, 12, 3, 12, 0], "a chain of three: 13 4 13; zero past it");
    assert!(s[100..272].iter().all(|b| *b == 0), "the rest of the chain, the name and the credit: zero");
    for at in [272, 272 + 1364] {
        let p = &s[at..at + 1364];
        assert_eq!(u16_at(p, 2 * 5), 1 << 6, "steps");
        assert_eq!(p[32 + 16 * 5 + 6], 44, "probability");
        assert_eq!(p[288 + 6], 11, "the lane");
        assert_eq!(u16_at(p, 304 + 2 * 5), 1 << 6, "keeps");
        assert_eq!(p[336 + 4 * (16 * 5 + 6)..336 + 4 * (16 * 5 + 6) + 4], [0x06, 0x05, 0x08, 0x07], "the values, A then B");
        assert_eq!(p[1360..], [13, 0, 0, 0], "the length, then zero");
    }
    assert_eq!(s[s.len() - 4..], crc32(&s[..s.len() - 4]).to_le_bytes());
}

#[test]
fn the_chain_saved_is_the_one_that_will_play_and_a_load_starts_at_its_first() {
    let mut h = three();
    h.block(1, &[chain(1), play()]);
    h.run(BAR + STEP, 128, &[]);
    assert_eq!(pats(&h).1, 1, "in the chain's second");
    let s = h.snap();
    assert_eq!(s[94..98], [2, 0, 0, 1], "the chain, not where it is: pos is not saved");
    h.block(1, &[pattern(2), chain(0)]);
    let s = h.snap();
    assert_eq!(s[94..98], [2, 0, 2, 0], "the next chain while one waits");
    let mut b = Host::new(SR, 9);
    assert_eq!(b.load(&s), OK);
    assert_eq!(pats(&b), (2, 0, "20".into(), "".into(), 0b111), "pos 0, no next chain, active the chain's first");
    assert_eq!(b.view()[24] & 3, 0, "not playing, write off: not saved");
    assert_eq!(b.snap(), s);
}

#[test]
fn sixteen_patterns_save_load_and_save_to_the_same_bytes() {
    let mut h = Host::new(SR, 77);
    let mut x = 0x5eed_1234u32;
    let mut rnd = move |n: u32| {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x % n
    };
    for p in 0..16 {
        let mut c = vec![pattern(p), cmd(CMD_LENGTH, 0, 0, 1 + rnd(16))];
        for _ in 0..40 {
            let (t, s) = (rnd(16) as u8, rnd(16) as u8);
            c.push(match rnd(4) {
                0 => on(t, s),
                1 => cmd(CMD_PROB, t, s, rnd(101)),
                2 => cmd(CMD_FX, s, 0, rnd(17)),
                _ => cmd(CMD_KEEP, t, s, rnd(u32::MAX)),
            });
        }
        h.block(1, &c);
    }
    for p in [3, 3, 0, 15, 9] {
        h.block(1, &[chain(p)]);
    }
    let s = h.snap();
    assert_eq!(s.len(), SNAP5_MAX);
    // A name and a credit: copied, never read.
    let mut named = s.clone();
    named[112..117].copy_from_slice(b"Kenny");
    named[144..150].copy_from_slice("crédit".as_bytes()[..6].try_into().unwrap());
    named[271] = 0xff;
    recrc(&mut named);
    for snap in [&s, &named] {
        let mut b = Host::new(SR, 1);
        assert_eq!(b.load(snap), OK);
        assert_eq!(&b.snap(), snap, "the same bytes");
        let mut c = Host::new(SR, 1);
        assert_eq!(c.load(&b.snap()), OK);
        assert_eq!(&c.snap(), snap, "and again");
    }
}

#[test]
fn pi_save_returns_the_length_it_wrote_and_0_with_nothing_written_when_cap_is_under_it() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[on(0, 0), copy(1)]);
    let want = h.snap();
    assert_eq!(want.len(), 276 + 2 * 1364);
    let mut b = vec![0xeeu8; want.len() + 10];
    unsafe {
        assert_eq!(pi_save(h.e, b.as_mut_ptr(), want.len() as u32 - 1), 0);
        assert!(b.iter().all(|x| *x == 0xee), "nothing written");
        assert_eq!(pi_save(h.e, core::ptr::null_mut(), 30_000), 0);
        assert_eq!(pi_save(core::ptr::null(), b.as_mut_ptr(), 30_000), 0);
        assert_eq!(pi_save(h.e, b.as_mut_ptr(), b.len() as u32), want.len() as u32);
    }
    assert_eq!(b[..want.len()], want[..]);
    assert_eq!(b[want.len()..], [0xee; 10], "and nothing past it");
}

/// A version 5 with three patterns, a chain and a name, from a busy engine.
fn busy5() -> Vec<u8> {
    let mut h = three();
    h.block(1, &[cmd(CMD_SWING, 0, 0, 9), cmd(CMD_KEEP, 2, 2, 99), chain(2), chain(1)]);
    let mut s = h.snap();
    s[112] = b'x';
    recrc(&mut s);
    s
}

#[test]
fn every_load_refusal_of_version_5_comes_in_section_8_s_order_and_changes_nothing() {
    let s = busy5();
    let mut h = Host::new(SR, 5);
    h.block(1, &[on(7, 7), pattern(4)]);
    let (before, g) = (h.snap(), h.generation());
    let with = |f: &dyn Fn(&mut Vec<u8>), crc: bool| {
        let mut t = s.clone();
        f(&mut t);
        if crc {
            recrc(&mut t);
        }
        t
    };
    let cases: Vec<(&str, Vec<u8>, i32)> = vec![
        ("under 12 bytes", s[..11].to_vec(), ERR_SIZE),
        ("magic", with(&|t| t[0] = b'Q', false), ERR_MAGIC),
        ("a version no core wrote", with(&|t| t[4] = 6, true), ERR_VERSION),
        ("under version 5's least, 276", s[..275].to_vec(), ERR_SIZE),
        ("kind", with(&|t| t[6] = 2, true), ERR_MAGIC),
        ("the payload size against the patterns written", with(&|t| t[92] ^= 1 << 5, true), ERR_SIZE),
        ("the payload size itself", with(&|t| t[8] ^= 4, true), ERR_SIZE),
        ("the length passed against the payload", s[..s.len() - 1].to_vec(), ERR_SIZE),
        ("the CRC", with(&|t| t[300] ^= 1, false), ERR_CRC),
        // Each check before the next: a bad magic and a bad version is a magic.
        ("magic before version", with(&|t| { t[0] = 0; t[4] = 9 }, false), ERR_MAGIC),
        ("version before size", with(&|t| { t[4] = 0; t.truncate(200) }, false), ERR_VERSION),
        ("least size before kind", with(&|t| { t[6] = 3; t.truncate(275) }, false), ERR_SIZE),
        ("kind before payload", with(&|t| { t[6] = 3; t[8] = 0 }, true), ERR_MAGIC),
        ("payload before length", with(&|t| { t[8] = 0; t.truncate(1000) }, false), ERR_SIZE),
        ("length before CRC", with(&|t| { t[300] ^= 1; t.truncate(t.len() - 4) }, false), ERR_SIZE),
    ];
    for (what, bytes, want) in cases {
        assert_eq!(h.load(&bytes), want, "{what}");
        assert_eq!((h.snap(), h.generation()), (before.clone(), g), "{what}: nothing changes");
    }
    let mut long = s.clone();
    long.extend([7; 9]);
    assert_eq!(h.load(&long), OK, "bytes after the CRC are the host's");
    assert_eq!(h.snap(), s);
}

#[test]
fn every_corrupted_byte_of_a_version_5_is_refused_and_changes_nothing() {
    let s = busy5();
    let mut h = Host::new(SR, 5);
    h.block(1, &[on(7, 7)]);
    let (before, g) = (h.snap(), h.generation());
    for i in 0..s.len() {
        for bit in [0x01u8, 0x80] {
            let mut bad = s.clone();
            bad[i] ^= bit;
            let want = match i {
                0..4 | 6..8 => ERR_MAGIC,
                4 => if bit == 1 { ERR_SIZE } else { ERR_VERSION }, // 4 is version 4, 1 464 bytes long
                5 => ERR_VERSION,
                8..12 => ERR_SIZE,
                92..94 => ERR_SIZE, // another number of patterns written
                _ => ERR_CRC,
            };
            assert_eq!(h.load(&bad), want, "byte {i} bit {bit:#x}");
        }
    }
    assert_eq!((h.snap(), h.generation()), (before, g));
}

#[test]
fn values_no_core_writes_are_clamped_on_a_version_5_load() {
    let s = busy5();
    let mut t = s.clone();
    let (b1, b2) = (272, 272 + SNAP5_BLOCK);
    t[94] = 0; // a chain of none: 1
    t[96] = 200; // an entry past 15: 15
    t[b1 + 1360] = 0; // a length of 0: 1
    t[b2 + 1360] = 99; // past 16: 16
    t[b1 + 288] = 17; // a lane entry past 16: 0
    t[b1 + 32] = 101; // a probability past 100: 100
    t[20] = 49; // swing past 48
    t[21] = 0; // volume 0: 1
    t[22] = 16; // a sound past 15
    t[23] = 5; // the hat cut: not read
    t[95] = 3; // zero: not read
    t[b1 + 1361] = 8; // zero: not read
    t[100] = 9; // past the chain's length: not read
    recrc(&mut t);
    let mut h = Host::new(SR, 1);
    assert_eq!(h.load(&t), OK);
    let u = h.snap();
    assert_eq!((u[94], u[96], u[97]), (1, 15, 0));
    assert_eq!((u[b1 + 1360], u[b2 + 1360], u[b1 + 288], u[b1 + 32]), (1, 16, 0, 100));
    assert_eq!(u[20..24], [48, 1, 15, 0]);
    assert_eq!((u[95], u[b1 + 1361], u[100]), (0, 0, 0));
    assert_eq!(pats(&h).0, 15, "active the chain's first, clamped");
    t[94] = 40;
    recrc(&mut t);
    assert_eq!(h.load(&t), OK);
    assert_eq!(h.snap()[94], 16, "a chain past sixteen: sixteen");
}

#[test]
fn an_older_version_loads_into_pattern_1_and_clears_the_rest() {
    let mut h = Host::new(SR, 1);
    h.block(1, &[on(4, 4), cmd(CMD_FX, 1, 0, 3)]);
    let v4 = h.save(); // version 4's layout (`as_v4`)
    let mut d = Host::new(SR, 1);
    let mut dress: Vec<Cmd> = (1..16).map(|p| copy(p)).collect();
    dress.extend([on(9, 9), copy(4), pattern(6), chain(2)]);
    d.block(1, &dress);
    assert_eq!(d.load(&v4), OK);
    assert_eq!(pats(&d), (0, 0, "0".into(), "".into(), 1));
    assert_eq!(d.save(), v4, "pattern 1 as it was saved");
    assert_eq!(d.snap().len(), 1640, "the other fifteen cleared");
}

#[test]
fn not_saved_the_playhead_pos_playing_write_mode_and_the_moment_s_punches() {
    let mut a = three();
    a.block(1, &[chain(1)]);
    let s = a.snap();
    a.block(1, &[down(WRITE), up(WRITE), play()]);
    a.run(BAR + 5 * STEP, 128, &[(BAR + STEP, down(FX)), (BAR + STEP, down(2))]);
    a.block(1, &[up(2), up(FX)]);
    let t = a.snap();
    // The lane took effect 3 on the steps that arrived while it was held: that is saved. Every
    // other byte is the same.
    let lane = 272 + SNAP5_BLOCK + 288;
    let differ: Vec<usize> = (0..s.len() - 4).filter(|i| s[*i] != t[*i]).collect();
    assert!(differ.iter().all(|i| (lane..lane + 16).contains(i)), "{differ:?}");
}

// The reference patch: two Rhythm modules and only Rack's own modules (Core, and Fundamental,
// which ships with Rack). It ships in each pre-release as Showcase.vcv, so it is the first
// thing many people open: it must sound good on first play and read plainly.
//
//   node rack/patch/make-showcase.mjs      writes rack/patch/showcase.json
//   make -C rack showcase                  packs it as rack/build/Showcase.vcv
//
// Never edit showcase.json by hand: change this file and run it again. The words on the
// patch (the notes) and the steps are near the top, as plain data, for Kenny to edit.
//
// Four rows, each left to right in signal order:
//   0  DRUMS and OUT: the main note, the clock (LFO) and the play button (Push) shared by both
//      modules, the DRUMS Rhythm (one bar of the Amen break; kick and snare on their samples,
//      D50), its effect 4 (Split on FX SEND, Delay), VCA Mix (one channel a module), Audio 2.
//   1  MELODY: the MELODIC Rhythm, which plays no drums: its sounds 15 and 16 fire two voices
//      built in Rack. Split takes its SOUNDS TRIG, SLIDER A OUT and SLIDER B OUT apart; Merge
//      brings both voices back into its SOUNDS IN on channels 15 and 16. Its RETURN setting
//      is those two sounds alone, so each voice takes its sound's place in the mix, heard once,
//      and Rhythm's effects reach it (ENGINE.md §19). Then its effect 4 (Split, Delay).
//   2  LEAD (sound 15): Random takes a new voltage on each trigger, 8vert scales it to ±1 V,
//      Quantizer holds it to the minor pentatonic, VCO's saw, VCF (slider A), ADSR (slider B:
//      how long the note rings) and VCA-1.
// Effect 4, the delay, is Rack's on both modules (D87, D88): each module's setting for effect 4
// is "Rack's, beside the dry" (fxRack bit 3, fxInPlace clear), so the core's own delay is
// silent and the dry stays. FX SEND's channel 4 carries the bus only while effect 4 is heard
// (jacks.hpp, fxSend), Split takes it out, Delay's WET output (no dry, so no comb, §19) goes
// back into FX RETURN, which is always summed: the echo rings on after the step. Each Delay is
// clocked by the same LFO. FX GATES is not needed: the send is already gated.
// The lanes write effect 4 on a few steps (§17: FX), so the echoes come and go with the bar;
// hold fx and press 4 to punch it by hand.
//   3  BASS (sound 16): slider A is the note. 8vert scales it, Quantizer holds it to the same
//      notes, VCO's square two octaves down, VCF (slider B), ADSR and VCA-1. So Rhythm itself
//      is the bass's note sequencer.
// Each step of sounds 15 and 16 keeps its own slider A and B (§20, D75: KEEP), and the slider
// jacks send what a sound was last hit with, on the frame of the hit (§19). So the lead's
// filter and length change from note to note and the bass plays chosen notes.
//
// Starting together: both run on one LFO (one edge a step), and one Push drives both PLAY ins
// (issue 147, D138); see startTogether() and PRESS below.
//
// The beats are snapshots the core itself writes (pi_save on the committed web/pi_core.wasm),
// read back and checked here. Port and param numbers are Rhythm.cpp's enums and Fundamental
// 2.6.4's (its src/*.cpp, tag v2.6.4); Core's Notes keeps its text as data.text (Rack 2.6.6,
// src/core/Notes.cpp). A cable names a port by number, so a patch keeps its cables when a
// panel is redrawn.
//
// SPDX-License-Identifier: MIT
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const PAGE = 65536;

// ---- The words on the patch. Plain strings, for Kenny to edit. ----
// Core's Notes is 16 HP with a fixed display: about 31 characters a line and 21 lines show
// without scrolling ⚠ (reckoned from Notes.cpp's 81 by 105 mm display, not yet seen).

// What to press to start both modules in step: one button (startTogether()).
const PRESS = `TO START: press HOLD on the
button by the LFO. Both play,
in step. Press it again: both
stop. Each module's own PLAY
key plays that one alone.`;

const NOTE_MAIN = `POCKET INSTRUMENTS: RHYTHM
Two Rhythm modules, one clock.
${PRESS}

DRUMS: one bar of the Amen
break on our own sounds. Four
bars come with patterns.
MELODY: two voices built in
Rack, one row each below.
Delay = effect 4, Rack's own,
on set steps: hold fx, press 4.
OUT: VCA Mix > Audio 2.

https://github.com/kinggs/
pocket-instruments
Not affiliated with Teenage
Engineering.`;

const NOTE_MELODY = `MELODY: no drums.
Its sound 15 plays the LEAD,
its sound 16 the BASS.
Each step keeps its own
sliders A and B, sent out
on SLIDER A and B OUT.
Split: channels 15 and 16.
Merge: both voices back on
SOUNDS IN 15 and 16, so
Rhythm's effects reach them.
Split, Delay: effect 4 is
Rack's (menu: beside the
dry). FX SEND 4 > Delay >
FX RETURN, on the clock.`;

const NOTE_LEAD = `LEAD: sound 15
Random: a new note on each
trigger, two octaves.
Quantizer: A C D E G
(A minor pentatonic).
VCO: the saw.
VCF: slider A, per step.
ADSR: slider B, per step:
how long each note rings.
VCA-1: the envelope.`;

const NOTE_BASS = `BASS: sound 16
Slider A is the note, set
per step: A C E G.
8vert and Quantizer: the
same notes as the lead.
VCO: square, two octaves
down (A1).
VCF: slider B, per step.
ADSR, VCA-1: short.`;

// ---- The beats. Steps 0 to 15. ----
const TEMPO = 136000; // thousandths of a bpm: about the Amen's own tempo
const MID = 0.5;      // a slider in the middle: a sound as it is

// DRUMS: the first bar of the Amen break, written out on our own sounds (no recording, no
// sample of the record). One bar: the instrument holds one pattern until the patterns work
// (D110, D133) lands; the four-bar chain follows then. Each step: [step, A, B] (0 to 1);
// slider B is a drum's length (docs/VOICES.md), so the ghost snares are short.
const DRUMS = {
  sampled: [0, 1], // kick and snare on their samples (D50); the hats stay in code
  lane: [12],       // effect 4 on the backbeat snare: a throw that echoes into the next bar
  tracks: [
    [0, [[0, MID, MID], [2, MID, MID], [10, MID, MID], [11, MID, MID]]],                // 1 bass drum
    [1, [[4, MID, MID], [7, MID, 0.2], [9, MID, 0.2], [12, MID, MID], [15, MID, 0.2]]], // 2 snare, ghosts short
    [2, [0, 2, 4, 6, 8, 10, 12, 14].map((s) => [s, MID, s % 4 ? 0.35 : 0.55])],        // 3 closed hat, the ride
  ],
};

// MELODIC: sound 15 the lead, sound 16 the bass.
const LEAD = 14, BASS = 15; // tracks count from 0
// The lead: the filter opens across the bar (A); B is how long each note rings (ADSR's
// release): short, longer, short, middling, and the last note long, into the echo.
const LEAD_STEPS = [[0, 0.30, 0.20], [3, 0.40, 0.50], [6, 0.50, 0.30], [10, 0.65, 0.40], [13, 0.85, 0.90]];
// The bass: slider A is the note. Through 8vert at 0.2 slider A's full travel is 2 V, two
// octaves over the VCO's A1; a semitone is 1/24 of the travel. Root-heavy, the kick's own
// steps 2, 10 and 11 left alone, C, E and G passing. B: the filter, the down-beats brighter.
const NOTE = { A1: 0, C2: 3, E2: 7, G2: 10, A2: 12 }; // semitones over A1 (in the quantizer's C-based scale, C D# F G A#)
const BASS_STEPS = [[0, "A1", 0.55], [3, "A1", 0.35], [6, "C2", 0.45], [8, "A2", 0.60], [13, "E2", 0.40], [14, "G2", 0.50]];
const MELODIC = {
  sampled: [],
  lane: [3, 4, 13, 14], // effect 4 under the lead's second note and its last
  tracks: [
    [LEAD, LEAD_STEPS],
    [BASS, BASS_STEPS.map(([s, n, b]) => [s, NOTE[n] / 24, b])],
  ],
};

const u16 = (x) => Math.round(Math.min(1, Math.max(0, x)) * 65535);

// One beat as the core writes it, and read back. Since ABI 16 the core writes snapshot version 5
// (ENGINE.md §21): the tempo (byte 16), the sampled mask (88), the patterns written (92: these
// beats write pattern 1 alone), then pattern 1's block from byte 272: its lane (272 + 288 + s),
// and for each step the keep bit (272 + 304 + 2 t) and its A and B (272 + 336 + 4 (16 t + s)).
// Until ABI 16 the core wrote version 4 (§8, §15, §17, §20: 376, 384, 404, 436), and the
// commands, events and snapshot sat at base + 8192 to 16384: inside today's engine of about
// 24 KB, so they start past its 32 KiB room now, and the snapshot may be PI_SNAPSHOT_SIZE.
const DELAY = 4; // effect 4, the delay (Rhythm.cpp's FX_NAMES)
function snapshot({ sampled, lane, tracks }) {
  const { exports: pi } = new WebAssembly.Instance(new WebAssembly.Module(readFileSync(`${root}web/pi_core.wasm`)), {});
  const base = pi.memory.grow(2) * PAGE, CMDS = base + 32768, EV = base + 36864, SNAP = base + 40960;
  const e = pi.pi_init(base, pi.pi_size(), 48000, 1);
  const v = new DataView(pi.memory.buffer);
  const cmds = [[4, 0, 0, TEMPO]];                                         // TEMPO
  for (const t of sampled) cmds.push([22, t, 0, 1]);                       // VOICE: its sample
  for (const [t, steps] of tracks)
    for (const [s, a, b] of steps) cmds.push([16, t, s, 1], [26, t, s, (u16(a) << 16 | u16(b)) >>> 0]); // STEP, KEEP
  for (const s of lane) cmds.push([23, s, 0, DELAY]);                      // FX: the lane
  if (cmds.length > 64) throw new Error("more commands than one block takes");
  cmds.forEach(([type, a, b, value], i) => {
    const p = CMDS + 12 * i;
    v.setUint32(p, 0, true); v.setUint8(p + 4, type); v.setUint8(p + 5, a); v.setUint8(p + 6, b); v.setUint8(p + 7, 0); v.setUint32(p + 8, value, true);
  });
  pi.pi_process(e, 1, CMDS, cmds.length, EV, 64, 0, 0);
  const n = pi.pi_save(e, SNAP, 22100);
  if (!n) throw new Error("pi_save wrote nothing");
  const bytes = Buffer.from(new Uint8Array(pi.memory.buffer, SNAP, n));
  const fail = (what) => { throw new Error(`the snapshot's ${what}`); };
  if (bytes.readUInt16LE(4) !== 5) fail(`version is ${bytes.readUInt16LE(4)}, not 5`);
  if (bytes.readUInt16LE(92) !== 1 || n !== 1640) fail(`patterns written are ${bytes.readUInt16LE(92)} in ${n} bytes, not pattern 1 alone in 1640`);
  if (bytes.readUInt32LE(16) !== TEMPO) fail(`tempo is ${bytes.readUInt32LE(16)}`);
  const mask = sampled.reduce((m, t) => m | (1 << t), 0);
  if (bytes.readUInt16LE(88) !== mask) fail(`sampled mask is ${bytes.readUInt16LE(88)}, not ${mask}`);
  const P = 272; // pattern 1's block
  for (let s = 0; s < 16; s++) if (bytes[P + 288 + s] !== (lane.includes(s) ? DELAY : 0)) fail(`lane at step ${s} is ${bytes[P + 288 + s]}`);
  for (const [t, steps] of tracks) {
    const want = steps.reduce((m, [s]) => m | (1 << s), 0);
    if (bytes.readUInt16LE(P + 304 + 2 * t) !== want) fail(`keeps for track ${t} are ${bytes.readUInt16LE(P + 304 + 2 * t)}, not ${want}`);
    for (const [s, a, b] of steps) {
      const at = P + 336 + 4 * (16 * t + s);
      if (bytes.readUInt16LE(at) !== u16(a) || bytes.readUInt16LE(at + 2) !== u16(b)) fail(`track ${t} step ${s} keeps the wrong A or B`);
    }
  }
  return bytes.toString("base64");
}

// ---- The modules: [id, plugin, model, version, x in HP, row, params, data]. ----
const F = "2.6.4";
const RACK = "2.6.6";
const ID = { drums: 2, melodic: 10, drumsDelay: 8, melodicDelay: 16 };
// Each module's setting for effect 4: Rack's, beside the dry (jacks.hpp FxSettings: bit n - 1
// of fxRack, clear in fxInPlace). The rest stay Ours.
const FX_RACK = 1 << (DELAY - 1), FX_IN_PLACE = 0;
// Delay's TIME on a clock (Fundamental src/Delay.cpp): the time is 2 / (clock Hz) times
// 10000^TIME / 1000 s. One edge a step, so TIME = log10(1000 × steps / 2) / 4 gives that many
// steps: 3, a dotted eighth (0.33 s at 136 bpm), which falls between the sixteenths of the break.
const ECHO_STEPS = 3;
const DELAY_TIME = Math.log10(1000 * ECHO_STEPS / 2) / 4;
// Quantizer notes from C (Fundamental's enabledNotes). The VCOs are tuned three semitones down,
// so C's minor pentatonic (C D# F G A#) sounds as A's: A C D E G.
const PENTATONIC = [0, 3, 5, 7, 10];
const enabledNotes = (notes) => Array.from({ length: 12 }, (_, i) => notes.includes(i));

const modules = [
  // Row 0: DRUMS and OUT.
  [1, "Core", "Notes", RACK, 0, 0, [], { text: NOTE_MAIN }],
  // (the clock at x 16 and the play button at 25: clock() and startTogether())
  [ID.drums, "pocket-instruments", "Rhythm", "2.0.0", 28, 0, [], { snapshot: snapshot(DRUMS), return: -1, a: -1, b: -1, fxRack: FX_RACK, fxInPlace: FX_IN_PLACE, clockBeat: false }],
  [7, "Fundamental", "Split", F, 55, 0, []], // FX SEND
  // Delay, effect 4: WET out only. Feedback 35% on the drums: two or three repeats.
  [ID.drumsDelay, "Fundamental", "Delay", F, 60, 0, [[0, DELAY_TIME], [1, 0.35], [2, 0.5], [3, 1.0]]], // TIME, FEEDBACK, TONE, MIX
  // VCA Mix: channel 1 the drums, channel 2 the melody. Set from peaks measured headless at
  // unity: drums 5.8 V (3.9 V without their echo), melody 7.1 V, the sum about 8.2 V; a fader's
  // gain is its value squared, and the mixer does not limit, so both are brought down and the
  // drums kept on top.
  [3, "Fundamental", "VCMixer", F, 69, 0, [[0, 1.0], [1, 0.9], [2, 0.75], [3, 0.0], [4, 0.0]]],
  [4, "Core", "AudioInterface2", RACK, 78, 0, []],

  // Row 1: MELODY. RETURN: sounds 15 and 16 alone, so the Merge's other channels take nothing away.
  [9, "Core", "Notes", RACK, 0, 1, [], { text: NOTE_MELODY }],
  [ID.melodic, "pocket-instruments", "Rhythm", "2.0.0", 16, 1, [], { snapshot: snapshot(MELODIC), return: (1 << LEAD) | (1 << BASS), a: -1, b: -1, fxRack: FX_RACK, fxInPlace: FX_IN_PLACE, clockBeat: false }],
  [11, "Fundamental", "Split", F, 43, 1, []], // SOUNDS TRIG
  [12, "Fundamental", "Split", F, 48, 1, []], // SLIDER A OUT
  [13, "Fundamental", "Split", F, 53, 1, []], // SLIDER B OUT
  [14, "Fundamental", "Merge", F, 58, 1, []], // SOUNDS IN
  [15, "Fundamental", "Split", F, 63, 1, []], // FX SEND
  // Delay, effect 4: WET out only. Feedback 45%: a longer tail on the voices.
  [ID.melodicDelay, "Fundamental", "Delay", F, 68, 1, [[0, DELAY_TIME], [1, 0.45], [2, 0.5], [3, 1.0]]],

  // Row 2: LEAD.
  [20, "Core", "Notes", RACK, 0, 2, [], { text: NOTE_LEAD }],
  // Random: SHAPE 0 (the step lands at once), bipolar, PROB 1, RAND 1 (a wholly new value).
  [21, "Fundamental", "Random", F, 16, 2, [[1, 0.0], [2, 0.0], [4, 1.0], [5, 1.0]]],
  [22, "Fundamental", "8vert", F, 25, 2, [[0, 0.2]]],                                 // row 1: ±5 V to ±1 V
  [23, "Fundamental", "Quantizer", F, 33, 2, [], { enabledNotes: enabledNotes(PENTATONIC) }],
  [24, "Fundamental", "VCO", F, 36, 2, [[2, -3.0]]],                                  // FREQ: A3 at 0 V
  // VCF: cutoff about 620 Hz at A 0.3, 3.4 kHz at A 0.85.
  [25, "Fundamental", "VCF", F, 45, 2, [[0, 0.49], [2, 0.35], [3, 0.45], [4, 0.0]]], // FREQ, RES, FREQ CV, DRIVE
  // ADSR: the trigger is 1 ms, so attack then release. Slider B sets the release: RELEASE 0.4
  // plus B (0 to 10 V) times 0.4, so a time constant of 40 ms at B 0 to 1.6 s at B 1.
  [26, "Fundamental", "ADSR", F, 52, 2, [[0, 0.0], [1, 0.3], [2, 0.0], [3, 0.4], [7, 0.4]]], // attack, decay, sustain, release, release CV
  [27, "Fundamental", "VCA-1", F, 61, 2, [[0, 1.0], [1, 1.0]]],                       // level, linear

  // Row 3: BASS.
  [30, "Core", "Notes", RACK, 0, 3, [], { text: NOTE_BASS }],
  [31, "Fundamental", "8vert", F, 16, 3, [[0, 0.2]]],                                 // row 1: 10 V to 2 V, two octaves
  [32, "Fundamental", "Quantizer", F, 24, 3, [], { enabledNotes: enabledNotes(PENTATONIC) }],
  [33, "Fundamental", "VCO", F, 27, 3, [[2, -27.0]]],                                 // FREQ: A1 (55 Hz) at 0 V
  // VCF: cutoff about 100 Hz at B 0, 400 Hz at the middle, 1.6 kHz at the top.
  [34, "Fundamental", "VCF", F, 36, 3, [[0, 0.36], [2, 0.2], [3, 0.4], [4, 0.0]]],
  [35, "Fundamental", "ADSR", F, 43, 3, [[0, 0.0], [1, 0.3], [2, 0.0], [3, 0.5]]],     // about 100 ms to fall
  [36, "Fundamental", "VCA-1", F, 52, 3, [[0, 0.8], [1, 1.0]]],                       // a little under the lead
];

// Rhythm: inputs CLOCK 0, RESET 1, SOUNDS IN 2, FX RETURN 3, SLIDER A IN 4, SLIDER B IN 5, PLAY 6;
// outputs SOUNDS TRIG 0, SOUNDS OUT 1, AUDIO 2 (one mono jack), FX GATES 3, FX SEND 4,
// SLIDER A OUT 5, SLIDER B OUT 6.
// Cables: [from module, output, to module, input, colour].
const cables = [
  // OUT. AUDIO is one mono jack (Rhythm.cpp: one setVoltage), so one mixer channel a module,
  // and the mix to both ears.
  [ID.drums, 2, 3, 1, "#f2e8d0"],
  [ID.melodic, 2, 3, 2, "#f2e8d0"],
  [3, 0, 4, 0, "#f2e8d0"],
  [3, 0, 4, 1, "#f2e8d0"],
  // MELODY: the three jacks apart, the voices back.
  [ID.melodic, 0, 11, 0, "#f0654a"],
  [ID.melodic, 5, 12, 0, "#f0b429"],
  [ID.melodic, 6, 13, 0, "#6fd6a0"],
  [14, 0, ID.melodic, 2, "#8faaf7"],
  // Effect 4 on both modules: FX SEND's channel 4 into Delay, its WET into FX RETURN.
  [ID.drums, 4, 7, 0, "#c9c2dd"],
  [7, DELAY - 1, ID.drumsDelay, 4, "#c9c2dd"],
  [ID.drumsDelay, 1, ID.drums, 3, "#c9c2dd"],
  [ID.melodic, 4, 15, 0, "#c9c2dd"],
  [15, DELAY - 1, ID.melodicDelay, 4, "#c9c2dd"],
  [ID.melodicDelay, 1, ID.melodic, 3, "#c9c2dd"],
  // LEAD.
  [11, LEAD, 21, 2, "#f0654a"],  // sound 15's trigger: Random's TRIG
  [11, LEAD, 26, 4, "#f0654a"],  // and ADSR's GATE
  [21, 0, 22, 0, "#c9c2dd"],     // Random's stepped voltage into 8vert
  [22, 0, 23, 0, "#c9c2dd"],     // ±1 V into Quantizer
  [23, 0, 24, 0, "#c9c2dd"],     // the note: VCO's V/OCT
  [24, 2, 25, 3, "#8faaf7"],     // the saw into VCF
  [12, LEAD, 25, 0, "#f0b429"],  // sound 15's A: VCF's cutoff
  [25, 0, 27, 1, "#8faaf7"],     // VCF's lowpass into VCA-1
  [26, 0, 27, 0, "#f0654a"],     // the envelope: VCA-1's CV
  [13, LEAD, 26, 3, "#6fd6a0"],  // sound 15's B: ADSR's release
  [27, 0, 14, LEAD, "#8faaf7"],  // the lead: Merge's channel 15
  // BASS.
  [12, BASS, 31, 0, "#f0b429"],  // sound 16's A: the note, into 8vert
  [31, 0, 32, 0, "#c9c2dd"],     // into Quantizer
  [32, 0, 33, 0, "#c9c2dd"],     // VCO's V/OCT
  [33, 3, 34, 3, "#8faaf7"],     // the square into VCF
  [13, BASS, 34, 0, "#6fd6a0"],  // sound 16's B: VCF's cutoff
  [11, BASS, 35, 4, "#f0654a"],  // sound 16's trigger: ADSR's GATE
  [34, 0, 36, 1, "#8faaf7"],     // VCF's lowpass into VCA-1
  [35, 0, 36, 0, "#f0654a"],     // the envelope: VCA-1's CV
  [36, 0, 14, BASS, "#8faaf7"],  // the bass: Merge's channel 16
];

// ---- The clock: one LFO for everything. ----
// Its square into both CLOCK inputs, one edge a step (each module's menu "CLOCK: one edge a
// beat" off, saved as clockBeat: false), at 136 bpm's sixteenths: 9.07 Hz. And into both
// Delays' CLOCK, so the echo follows the tempo.
function clock(clocked) {
  const hz = (TEMPO / 1000) * 4 / 60; // one edge a step: four a beat
  return {
    modules: [[5, "Fundamental", "LFO", F, 16, 0, [[0, 1.0], [2, Math.log2(hz)], [5, 0.5]]]], // unipolar; FREQ in log2 Hz; PW 50%
    cables: clocked.map(([id, input]) => [5, 3, id, input, "#ffffff"]),
  };
}

// ---- Starting together (issue 147, D138). ----
// One Push, its GATE (output 1; Fundamental 2.6.4 src/Push.cpp: outputs TRIG 0, GATE 1; params
// PUSH 0, HOLD 1) into both modules' PLAY in (input 6, the end of Rhythm.cpp's InputId). A gate
// plays while high and stops when it goes low (D138 item 4, proposed: O39), so pressing Push's
// HOLD starts both stopped modules from step 1 on the same next clock edge, and pressing it
// again stops both on the same frame. Push saves nothing: HOLD opens off, as the modules open
// stopped. No RESET is wired: a stop and a start is a restart from step 1, in step.
function startTogether(drums, melodic) {
  const PLAY_IN = 6, GATE = 1;
  return {
    modules: [[6, "Fundamental", "Push", F, 25, 0, []]],
    cables: [
      [6, GATE, drums, PLAY_IN, "#ffffff"],   // Push's GATE: both PLAY ins
      [6, GATE, melodic, PLAY_IN, "#ffffff"],
    ],
  };
}

for (const part of [clock([[ID.drums, 0], [ID.melodic, 0], [ID.drumsDelay, 5], [ID.melodicDelay, 5]]), startTogether(ID.drums, ID.melodic)]) {
  modules.push(...part.modules);
  cables.unshift(...part.cables);
}

const patch = {
  version: RACK,
  zoom: 1.0,
  gridOffset: [-1.0, -0.1], // the view opens on the modules (Rack's own key, in HP and rows)
  modules: modules.sort((p, q) => p[0] - q[0]).map(([id, plugin, model, version, x, y, params, data]) => ({
    id, plugin, model, version, params: params.map(([pid, value]) => ({ id: pid, value })), pos: [x, y], ...(data ? { data } : {}),
  })),
  cables: cables.map(([o, oid, i, iid, color], n) => ({ id: n + 1, outputModuleId: o, outputId: oid, inputModuleId: i, inputId: iid, color })),
};
writeFileSync(`${root}rack/patch/showcase.json`, JSON.stringify(patch, null, 1) + "\n");
console.log(`wrote rack/patch/showcase.json: ${modules.length} modules, ${cables.length} cables`);

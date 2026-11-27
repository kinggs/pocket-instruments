// The reference patch: one Rhythm module and only Rack's own modules (Core, and Fundamental,
// which ships with Rack). It ships in each pre-release as Showcase.vcv, so it is the first
// thing many people open: it must sound good on first play and read plainly.
//
//   node rack/patch/make-showcase.mjs      writes rack/patch/showcase.json
//   make -C rack showcase                  packs it as rack/build/Showcase.vcv
//
// Never edit showcase.json by hand: change this file and run it again.
//
// What it does (open it, press play). Top row: a note, Rhythm, the audio module. Bottom row,
// left to right in signal order, sound 15 built in Rack:
//   - The kick (sound 1) plays its sample inside Rhythm, not its code voice (D50): the
//     snapshot below sets it (VOICE), so the patch shows the samples.
//   - SOUNDS TRIG's channel 15 (Split) is the only clock of the voice. Each trigger makes
//     Random take a new voltage (±5 V), 8vert scales it to ±1 V (two octaves), Quantizer
//     holds it to a minor scale, and VCO's saw plays it. The same trigger fires ADSR.
//   - SLIDER A OUT's channel 15 (Split) moves VCF's cutoff; VCA-1 applies the envelope;
//     SLIDER B OUT's channel 15 (Split) sets Delay's mix and feedback, how much echo.
//   - The voice goes back in on SOUNDS IN's channel 15 (Merge). Rhythm's RETURN setting is
//     sound 15 alone, so the voice takes sound 15's place in the mix, heard once, and
//     Rhythm's effects reach it (ENGINE.md §19). AUDIO goes to Core's Audio 2.
// The slider jacks carry 0 to 10 V, a sound's value when it was last hit (§19, §20): moving a
// slider sets all sixteen sounds' own value (§20), so sound 15's channel follows it on its
// next hit. VCF's and Delay's own CV attenuators scale it; nothing else is needed.
//
// The beat is a snapshot the core itself writes (pi_save on the committed web/pi_core.wasm).
// Port and param numbers are Rhythm.cpp's enums and Fundamental 2.6.4's (its src/*.cpp, tag
// v2.6.4); Core's Notes keeps its text as data.text (Rack 2.6.6, src/core/Notes.cpp). A cable
// names a port by number, so a patch keeps its cables when a panel is redrawn.
//
// SPDX-License-Identifier: MIT
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const PAGE = 65536;

// ---- The words on the patch. Plain strings, for Kenny to edit. ----
// Core's Notes is 16 HP with a fixed display: about 31 characters a line and 21 lines show
// without scrolling ⚠ (reckoned from Notes.cpp's 81 by 105 mm display, not yet seen).
const NOTE_MAIN = `POCKET INSTRUMENTS: RHYTHM
Sixteen sounds, sixteen steps.
Press PLAY on Rhythm.
Beat: the Amen break's first
bar, on Rhythm's own sounds.
Kick: its SAMPLE, not its code
voice (right-click > Sounds).

Sound 15 is built below. Its
trigger picks a random note in
A minor for a saw.
Slider A: filter. B: delay.
It comes back on SOUNDS IN.
AUDIO > Audio 2: choose your
sound card there.

https://github.com/kinggs/
pocket-instruments
Not affiliated with Teenage
Engineering.`;

const NOTE_VOICE = `VOICE: sound 15
Every trigger comes from
Rhythm: SOUNDS TRIG,
channel 15 (Split).

Random: a new voltage on
each trigger.
8vert: down to ±1 V, two
octaves.
Quantizer: A minor (the
VCO's tuning is A).
VCO: the saw.`;

const NOTE_SHAPE = `FILTER, ENVELOPE, DELAY
Slider A (SLIDER A OUT,
channel 15): VCF's cutoff.
ADSR and VCA-1: the pluck,
on the same trigger.
Slider B (SLIDER B OUT,
channel 15): Delay's echo,
its mix and feedback.

Merge: back into SOUNDS IN
on channel 15. Rhythm plays
it as sound 15, and its
effects reach it.`;

// ---- The beat. ----
// Steps 0 to 15. The first bar of the Amen break written out on our own sounds (no recording,
// no sample of the record), and sound 15's notes between the snares. One bar: the instrument
// holds one pattern until the patterns work (D110) lands; a chained version follows.
const TEMPO = 136000; // thousandths of a bpm: about the Amen's own tempo
const BEAT = [
  [0, [0, 2, 10, 11]],              // 1 bass drum, on its sample (VOICE below)
  [1, [4, 7, 9, 12, 15]],           // 2 snare
  [2, [0, 2, 4, 6, 8, 10, 12, 14]], // 3 closed hat, the Amen's ride
  [14, [0, 3, 6, 10, 13]],          // 15 tone: the Rack voice's triggers, three-three-four
];
const SAMPLED = [0]; // the tracks that play their sample (D50)

function snapshot() {
  const { exports: pi } = new WebAssembly.Instance(new WebAssembly.Module(readFileSync(`${root}web/pi_core.wasm`)), {});
  const base = pi.memory.grow(2) * PAGE, CMDS = base + 8192, EV = base + 12288, SNAP = base + 16384;
  const e = pi.pi_init(base, pi.pi_size(), 48000, 1);
  const v = new DataView(pi.memory.buffer);
  const cmds = [[4, 0, 0, TEMPO]];                                                     // TEMPO
  for (const t of SAMPLED) cmds.push([22, t, 0, 1]);                                   // VOICE: its sample
  for (const [t, steps] of BEAT) for (const s of steps) cmds.push([16, t, s, 1]);      // STEP
  cmds.forEach(([type, a, b, value], i) => {
    const p = CMDS + 12 * i;
    v.setUint32(p, 0, true); v.setUint8(p + 4, type); v.setUint8(p + 5, a); v.setUint8(p + 6, b); v.setUint8(p + 7, 0); v.setUint32(p + 8, value, true);
  });
  pi.pi_process(e, 1, CMDS, cmds.length, EV, 64, 0, 0);
  const n = pi.pi_save(e, SNAP, 4096);
  if (!n) throw new Error("pi_save wrote nothing");
  const bytes = Buffer.from(new Uint8Array(pi.memory.buffer, SNAP, n));
  // Held to what was asked: the tempo at byte 16 and the sampled mask at byte 376 (§8, §15).
  const mask = SAMPLED.reduce((m, t) => m | (1 << t), 0);
  if (bytes.readUInt32LE(16) !== TEMPO) throw new Error(`the snapshot's tempo is ${bytes.readUInt32LE(16)}, not ${TEMPO}`);
  if (bytes.readUInt16LE(376) !== mask) throw new Error(`the snapshot's sampled mask is ${bytes.readUInt16LE(376)}, not ${mask}`);
  return bytes.toString("base64");
}

// ---- The modules: [id, plugin, model, version, x in HP, row, params, data]. ----
const F = "2.6.4";
const RACK = "2.6.6";
const SOUND15 = 14; // a channel number counts from 0
const modules = [
  // Top row.
  [1, "Core", "Notes", RACK, 0, 0, [], { text: NOTE_MAIN }],
  // RETURN: sound 15 alone (bit 14), so the Merge's other channels take nothing away.
  [2, "pocket-instruments", "Rhythm", "2.0.0", 16, 0, [], { snapshot: snapshot(), return: 1 << SOUND15, a: -1, b: -1, fxRack: 0, fxInPlace: 0 }],
  [3, "Core", "AudioInterface2", RACK, 43, 0, []],
  // Bottom row, in signal order.
  [4, "Core", "Notes", RACK, 0, 1, [], { text: NOTE_VOICE }],
  [5, "Fundamental", "Split", F, 16, 1, []],                                          // SOUNDS TRIG
  // Random: SHAPE 0 (the step lands at once), RAND 1 (a wholly new value), PROB 1, bipolar.
  [6, "Fundamental", "Random", F, 21, 1, [[1, 0.0], [2, 0.0], [4, 1.0], [5, 1.0]]],
  [7, "Fundamental", "8vert", F, 30, 1, [[0, 0.2]]],                                   // row 1: ±5 V to ±1 V
  [8, "Fundamental", "Quantizer", F, 38, 1, [], {
    // Natural minor from C, as Fundamental's own preset "05_Aeolian (Minor)".
    enabledNotes: [true, false, true, true, false, true, false, true, true, false, true, false],
  }],
  [9, "Fundamental", "VCO", F, 41, 1, [[2, -3.0]]],                                    // FREQ: A3, so the scale is A minor
  [10, "Core", "Notes", RACK, 50, 1, [], { text: NOTE_SHAPE }],
  [11, "Fundamental", "Split", F, 66, 1, []],                                          // SLIDER A OUT
  // VCF: cutoff about 250 Hz at slider A's bottom, 1.2 kHz at its middle, 6 kHz at its top.
  [12, "Fundamental", "VCF", F, 71, 1, [[0, 0.49], [2, 0.35], [3, 0.45], [4, 0.0]]],  // FREQ, RES, FREQ CV, DRIVE
  // ADSR: a pluck. The trigger is 1 ms, so attack then release: about 160 ms to fall.
  [13, "Fundamental", "ADSR", F, 78, 1, [[0, 0.0], [1, 0.3], [2, 0.0], [3, 0.55]]],     // attack, decay, sustain, release
  [14, "Fundamental", "VCA-1", F, 87, 1, [[0, 1.0], [1, 1.0]]],                        // level, linear
  [15, "Fundamental", "Split", F, 90, 1, []],                                          // SLIDER B OUT
  // Delay: a dotted eighth at 136 bpm (0.33 s). Slider B's 0 to 10 V adds up to 45% mix and
  // 60% feedback on top of 15% each: a faint echo at the bottom, a long trail at the top.
  [16, "Fundamental", "Delay", F, 95, 1, [[0, 0.63], [1, 0.15], [2, 0.5], [3, 0.15], [5, 0.6], [7, 0.45]]],
  [17, "Fundamental", "Merge", F, 104, 1, []],                                         // SOUNDS IN
];

// Rhythm: inputs CLOCK 0, RESET 1, SOUNDS IN 2, FX RETURN 3, SLIDER A IN 4, SLIDER B IN 5;
// outputs SOUNDS TRIG 0, SOUNDS OUT 1, AUDIO 2, FX GATES 3, FX SEND 4, SLIDER A OUT 5, SLIDER B OUT 6.
// Cables: [from module, output, to module, input, colour].
const cables = [
  [2, 0, 5, 0, "#f0654a"],          // SOUNDS TRIG into Split
  [5, SOUND15, 6, 2, "#f0654a"],    // sound 15's trigger: Random's TRIG
  [5, SOUND15, 13, 4, "#f0654a"],   // and ADSR's GATE
  [6, 0, 7, 0, "#c9c2dd"],          // Random's stepped voltage into 8vert's row 1
  [7, 0, 8, 0, "#c9c2dd"],          // ±1 V into Quantizer
  [8, 0, 9, 0, "#c9c2dd"],          // the note: VCO's V/OCT
  [9, 2, 12, 3, "#8faaf7"],         // VCO's saw into VCF
  [2, 5, 11, 0, "#f0b429"],         // SLIDER A OUT into Split
  [11, SOUND15, 12, 0, "#f0b429"],  // sound 15's A: VCF's cutoff
  [12, 0, 14, 1, "#8faaf7"],        // VCF's lowpass into VCA-1
  [13, 0, 14, 0, "#f0654a"],        // the envelope: VCA-1's CV
  [2, 6, 15, 0, "#6fd6a0"],         // SLIDER B OUT into Split
  [15, SOUND15, 16, 3, "#6fd6a0"],  // sound 15's B: Delay's mix
  [15, SOUND15, 16, 1, "#6fd6a0"],  // and its feedback
  [14, 0, 16, 4, "#8faaf7"],        // the plucked saw into Delay
  [16, 0, 17, SOUND15, "#8faaf7"],  // Delay's mix: Merge's channel 15
  [17, 0, 2, 2, "#8faaf7"],         // into SOUNDS IN
  [2, 2, 3, 0, "#f2e8d0"],          // AUDIO to the left
  [2, 2, 3, 1, "#f2e8d0"],          // and the right
];

const patch = {
  version: RACK,
  zoom: 1.0,
  gridOffset: [-1.0, -0.1], // the view opens on the modules (Rack's own key, in HP and rows)
  modules: modules.map(([id, plugin, model, version, x, y, params, data]) => ({
    id, plugin, model, version, params: params.map(([pid, value]) => ({ id: pid, value })), pos: [x, y], ...(data ? { data } : {}),
  })),
  cables: cables.map(([o, oid, i, iid, color], n) => ({ id: n + 1, outputModuleId: o, outputId: oid, inputModuleId: i, inputId: iid, color })),
};
writeFileSync(`${root}rack/patch/showcase.json`, JSON.stringify(patch, null, 1) + "\n");
console.log(`wrote rack/patch/showcase.json: ${modules.length} modules, ${cables.length} cables`);

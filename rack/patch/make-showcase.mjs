// The reference patch: one Rhythm module and only Rack's own modules (Core, and Fundamental,
// which ships with Rack), patched so that every kind of jack is in use. For trying the module,
// for testing by hand, for the showcase and for screenshots.
//
//   node rack/patch/make-showcase.mjs      writes rack/patch/showcase.json
//   make -C rack showcase                  packs it as rack/build/Showcase.vcv
//
// What it does, left to right (open it, press play):
//   - SOUNDS: TRIG's channel 1 (the bass drum's trigger) fires Fundamental's ADSR; the ADSR
//     opens a VCA on a low VCO sine; that kick goes back in on IN's channel 1, so Rack's kick
//     takes sound 1's place (the menu's RETURN: sound 1). Effects still reach it.
//   - FX: SEND's sixteen channels go through Fundamental's VCF, swept by an LFO, and back in on
//     RETURN. Effect 7 is "Rack's, in place of the dry", so punching 7 is the VCF; the lane
//     holds 7 on the last four steps.
//   - SLIDER A: an LFO's triangle into IN's channel 2 moves sound 2's slider A (the snare).
//   - AUDIO through Fundamental's Delay to the audio module; AUDIO and A OUT on a Scope.
// The beat is a snapshot the core itself writes (pi_save on the committed web/pi_core.wasm).
// Port numbers are Rhythm.cpp's enums and Fundamental 2.6.4's (its src/*.cpp); a cable names a
// port by number, so a patch keeps its cables when a panel is redrawn.
//
// SPDX-License-Identifier: MIT
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const PAGE = 65536;

// The beat: track, steps (0 to 15). Then the lane: effect 7 on steps 13 to 16.
const BEAT = [
  [0, [0, 4, 8, 12]],            // bass drum (Rack's kick comes back in its place)
  [1, [4, 12]],                  // snare
  [2, [0, 2, 4, 6, 8, 10, 12]],  // closed hat
  [3, [14]],                     // open hat
  [8, [12]],                     // clap
  [10, [7, 10]],                 // low tom
  [12, [3, 11]],                 // cow bell
];
const LANE = [[12, 7], [13, 7], [14, 7], [15, 7]];

function snapshot() {
  const { exports: pi } = new WebAssembly.Instance(new WebAssembly.Module(readFileSync(`${root}web/pi_core.wasm`)), {});
  const base = pi.memory.grow(2) * PAGE, CMDS = base + 8192, EV = base + 12288, SNAP = base + 16384;
  const e = pi.pi_init(base, pi.pi_size(), 48000, 1);
  const v = new DataView(pi.memory.buffer);
  const cmds = [];
  for (const [t, steps] of BEAT) for (const s of steps) cmds.push([16, t, s, 1]); // STEP
  for (const [s, fx] of LANE) cmds.push([23, s, 0, fx]);                          // FX
  cmds.forEach(([type, a, b, value], i) => {
    const p = CMDS + 12 * i;
    v.setUint32(p, 0, true); v.setUint8(p + 4, type); v.setUint8(p + 5, a); v.setUint8(p + 6, b); v.setUint8(p + 7, 0); v.setUint32(p + 8, value, true);
  });
  pi.pi_process(e, 1, CMDS, cmds.length, EV, 64, 0, 0);
  const n = pi.pi_save(e, SNAP, 4096);
  if (!n) throw new Error("pi_save wrote nothing");
  return Buffer.from(new Uint8Array(pi.memory.buffer, SNAP, n)).toString("base64");
}

// Modules: [id, plugin, model, version, x in HP, row, params, data].
const F = "2.6.4";
const modules = [
  [1, "pocket-instruments", "Rhythm", "2.0.0", 0, 0, [], { snapshot: snapshot(), return: 1, a: 2, fxRack: 64, fxInPlace: 64 }],
  [2, "Fundamental", "Split", F, 27, 0, []],
  [3, "Fundamental", "ADSR", F, 32, 0, [[0, 0.0], [1, 0.32], [2, 0.0], [3, 0.3]]],   // attack, decay, sustain, release
  [4, "Fundamental", "VCO", F, 41, 0, [[2, -33]]],                                   // FREQ: about 55 Hz
  [5, "Fundamental", "VCA-1", F, 50, 0, [[0, 1.0]]],
  [6, "Fundamental", "Merge", F, 53, 0, []],
  [7, "Fundamental", "LFO", F, 0, 1, [[2, -2.0]]],                                   // FREQ: 0.25 Hz
  [8, "Fundamental", "VCF", F, 9, 1, [[0, 0.55], [2, 0.35], [3, 0.4]]],              // FREQ, RES, FREQ CV
  [9, "Fundamental", "Merge", F, 16, 1, []],
  [10, "Fundamental", "Delay", F, 21, 1, [[0, 0.55], [1, 0.35], [3, 0.25]]],          // TIME, FEEDBACK, MIX
  [11, "Fundamental", "Scope", F, 30, 1, []],
  [12, "Core", "AudioInterface2", "2.6.6", 43, 1, []],
];
// Rhythm: inputs CLOCK 0, RESET 1, RETURN 2, FX RETURN 3, A IN 4, B IN 5; outputs TRIGGERS 0,
// VOICES 1, AUDIO 2, FX GATES 3, FX SEND 4, A OUT 5, B OUT 6.
// Cables: [from module, output, to module, input, colour].
const cables = [
  [1, 0, 2, 0, "#f0654a"],   // TRIG into Split
  [2, 0, 3, 4, "#f0654a"],   // the bass drum's trigger: ADSR's GATE
  [3, 0, 5, 0, "#f0b429"],   // the envelope: VCA-1's CV
  [4, 0, 5, 1, "#8faaf7"],   // VCO's sine: VCA-1's IN
  [5, 0, 6, 0, "#8faaf7"],   // the kick: Merge's channel 1
  [6, 0, 1, 2, "#8faaf7"],   // into RETURN
  [1, 4, 8, 3, "#6fd6a0"],   // FX SEND into VCF
  [8, 0, 1, 3, "#6fd6a0"],   // VCF's lowpass into FX RETURN
  [7, 0, 8, 0, "#c9c2dd"],   // LFO's sine sweeps the VCF
  [7, 1, 9, 1, "#c9c2dd"],   // LFO's triangle: Merge's channel 2
  [9, 0, 1, 4, "#c9c2dd"],   // into A IN
  [1, 2, 10, 4, "#f2e8d0"],  // AUDIO into Delay
  [10, 0, 12, 0, "#f2e8d0"], // Delay to the left
  [10, 0, 12, 1, "#f2e8d0"], // and the right
  [1, 2, 11, 0, "#f2e8d0"],  // AUDIO on the Scope
  [1, 5, 11, 1, "#f0b429"],  // A OUT on the Scope
];

const patch = {
  version: "2.6.6",
  zoom: 1.0,
  gridOffset: [-1.0, -0.1], // the view opens on the modules (Rack's own key, in HP and rows)
  modules: modules.map(([id, plugin, model, version, x, y, params, data]) => ({
    id, plugin, model, version, params: params.map(([pid, value]) => ({ id: pid, value })), pos: [x, y], ...(data ? { data } : {}),
  })),
  cables: cables.map(([o, oid, i, iid, color], n) => ({ id: n + 1, outputModuleId: o, outputId: oid, inputModuleId: i, inputId: iid, color })),
};
writeFileSync(`${root}rack/patch/showcase.json`, JSON.stringify(patch, null, 1) + "\n");
console.log(`wrote rack/patch/showcase.json: ${modules.length} modules, ${cables.length} cables`);

// The core's .wasm from Node, the raw module and no glue: loading it, an engine, the starter
// pack, effect 4's line, a fixture's schedule and the loop that runs it, with or without sound.
// Moved here from trace.test.mjs unchanged (issue 163) so that the agent's command,
// beat/beat.mjs, renders with the same code the harness does; trace.test.mjs imports it.
//
// SPDX-License-Identifier: MIT
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
export const WASM = here("../target/wasm32-unknown-unknown/release/pi_core.wasm");

export const RATES = [44100, 48000, 96000];
export const CMD = { clock: 1, reset: 2, source: 3, tempo: 4, play: 5, step: 16, prob: 17, swing: 18, length: 19, sound: 20, voice: 22, fx: 23, fxsound: 24, route: 25, keep: 26, pattern: 27, chain: 28, copy: 29, key_down: 32, key_up: 33, param: 34 };
// Format 2: what a finger did. These land a frame after their step's boundary.
const KEYS = new Set(["key_down", "key_up", "param"]);
// Format 5's pattern, chain and copy are edits too: at step 0 they come from the snapshot.
export const EDITS = new Set(["step", "prob", "swing", "length", "fx", "fxsound", "keep", "pattern", "chain", "copy"]);
export const PAGE = 65536;
// Where things sit in the two pages the host adds (ENGINE.md §8). Since ABI 16 (§21; D133
// item 1) the engine gets the first 32 KiB (core/src/tests/rules.rs holds pi_size to it), and
// the snapshot room is version 5's most, 22 100 bytes; the view is 88. The audio has room for a
// 128-frame block at 50 channels. Until ABI 16: one page, the engine 4 KiB, the snapshot 1 536
// bytes (version 4's 1464), the view at 5632, the events at 5760 and the audio at 9216.
export const HOST = 2 * PAGE;
export const SNAP = 32768, SNAP_ROOM = 22100, VIEW = 55040, VIEW_SIZE = 88, EVENTS = 55296, CAP = 256, AUDIO = 58368, AUDIO_MAX = 128 * 50;
const CMDS = AUDIO + 4 * AUDIO_MAX, CMDS_MAX = Math.floor((HOST - CMDS) / 12);

export function load(path = WASM) {
  const module = new WebAssembly.Module(readFileSync(path));
  const imports = WebAssembly.Module.imports(module);
  const instance = new WebAssembly.Instance(module, {});
  const pi = instance.exports;
  // The host's memory: two pages past whatever the module asked for (one until ABI 16).
  const base = pi.memory.grow(HOST / PAGE) * PAGE;
  return { pi, base, imports, view: () => new DataView(pi.memory.buffer) };
}

export function engine(m, rate, seed) {
  const size = m.pi.pi_size();
  assert.ok(size <= SNAP, `pi_size ${size} fits before the snapshot buffer`);
  assert.equal(m.base % m.pi.pi_align(), 0);
  const e = m.pi.pi_init(m.base, size, rate, seed);
  assert.equal(e, m.base, "pi_init returns the memory it was given");
  return e;
}

// D50: the starter pack, in track order, copied once into pages of its own after the host's.
// Returns [address, frames] a track.
export function starterPack(m) {
  const dir = here("../samples");
  const names = readFileSync(`${dir}/tracks.txt`, "utf8").trim().split("\n");
  assert.equal(names.length, 16);
  const files = names.map((n) => readFileSync(`${dir}/raw/${n}.s16`));
  const total = files.reduce((a, f) => a + f.length, 0);
  let at = m.pi.memory.grow(Math.ceil(total / PAGE)) * PAGE;
  return files.map((f) => {
    new Uint8Array(m.pi.memory.buffer, at, f.length).set(f);
    const placed = [at, f.length / 2];
    at += f.length;
    return placed;
  });
}

// What `harness --render` gives an engine before it plays (P7): with `samples` the starter pack
// on every track; with `delay` effect 4 lent its line (ABI 12, §18): pages of its own after the
// samples', room for the highest rate, pi_delay_frames(rate) floats of it lent to each engine.
// Returns a function that gives an engine at a rate both, and returns the engine.
export function renderKit(m, samples, delay) {
  const pack = samples ? starterPack(m) : [];
  const line = delay ? m.pi.memory.grow(Math.ceil((4 * m.pi.pi_delay_frames(Math.max(...RATES))) / PAGE)) * PAGE : 0;
  return (e, rate) => {
    pack.forEach(([at, frames], t) => assert.equal(m.pi.pi_set_sample(e, t, at, frames), 1));
    if (line) assert.equal(m.pi.pi_set_delay(e, line, m.pi.pi_delay_frames(rate)), 1);
    return e;
  };
}

export function toCmd(c) {
  const type = CMD[c.cmd];
  assert.ok(type && c.cmd !== "clock" && c.cmd !== "source" && c.cmd !== "tempo", `fixture command ${c.cmd}`);
  // Format 3's fx names a step and nothing else: the step goes in `a` (ENGINE.md §17).
  if (c.cmd === "fx") return { type, a: c.step ?? 0, b: 0, value: c.value ?? 0 };
  if (c.cmd === "route") return { type, a: c.what, b: 0, value: c.value ?? 0 }; // P7 only (§19)
  // Format 4's keep: A in the high sixteen bits of the value, B in the low (ENGINE.md §20).
  if (c.cmd === "keep") return { type, a: c.track, b: c.step, value: ((c.a << 16) | c.b) >>> 0 };
  return { type, a: c.track ?? c.key ?? c.slider ?? 0, b: c.step ?? c.param ?? 0, value: c.value ?? 0 };
}

// What to send and when: [absolute frame, command] in order, the frame the run ends before,
// and (format 2) each view to read: [the frame its block ends on, the text expected there].
export function schedule(fx, rate, source, fromSnapshot) {
  const F = (n) => Number((BigInt(n) * BigInt(rate) * 15000n + BigInt(fx.mbpm) - 1n) / BigInt(fx.mbpm));
  let anchor = 0, anchorStep = 0;
  const cmds = [], stepFrames = [];
  if (!fromSnapshot) cmds.push([0, { type: CMD.tempo, a: 0, b: 0, value: fx.mbpm }]);
  cmds.push([0, { type: CMD.source, a: 0, b: 0, value: source }]);
  for (let m = 0; m < fx.steps; m++) {
    const at = anchor + F(m - anchorStep);
    stepFrames.push(at);
    for (const c of fx.commands) {
      if (c.at !== m || KEYS.has(c.cmd)) continue;
      if (fromSnapshot && m === 0 && EDITS.has(c.cmd)) continue; // the snapshot carries these
      cmds.push([at, toCmd(c)]);
      if (c.cmd === "reset") { anchor = at; anchorStep = m; }
    }
    if (source === 1) cmds.push([at, { type: CMD.clock, a: 0, b: 0, value: 0 }]);
    // Keys and sliders: one frame on, so the step's tick 0 is behind them on both sources.
    for (const c of fx.commands) if (c.at === m && KEYS.has(c.cmd)) cmds.push([at + 1, toCmd(c)]);
  }
  const end = anchor + F(fx.steps - anchorStep);
  // A view is read where its run step ends: on the frame the next one falls on. Written for
  // the table run; the edges run differs in that one flag.
  const views = (fx.views ?? []).map((v) => {
    assert.ok(v.at < fx.steps, "a view past the end of the run");
    return [stepFrames[v.at + 1] ?? end, source === 1 ? v.view.replace(" edges=0 ", " edges=1 ") : v.view];
  });
  return { cmds, end, views };
}

// ENGINE.md §14's text line for the view block at `p`: what a format 2 fixture holds. For
// format 3, §17's three fields on its end; for format 4, then §20's two; for format 5, §21's
// patterns, the chain and the next chain a hex digit an entry.
function viewText(v, p, format = 2) {
  const nibbles = (at, len) => Array.from({ length: len }, (_, i) => ((v.getUint8(p + at + (i >> 1)) >> (4 * (i & 1))) & 15).toString(16)).join("");
  return viewText4(v, p, format) + (format >= 5
    ? ` pat=${v.getUint8(p + 64)} pos=${v.getUint8(p + 65)} chain=${nibbles(72, v.getUint8(p + 66))}` +
      ` next=${v.getUint8(p + 67) ? nibbles(80, v.getUint8(p + 67)) : "-"} used=${v.getUint16(p + 68, true).toString(16).padStart(4, "0")}`
    : "");
}

function viewText4(v, p, format) {
  const flags = v.getUint8(p + 24), flag = (bit) => (flags >> bit) & 1;
  const hex = (n, width) => n.toString(16).padStart(width, "0");
  const fires = Array.from({ length: 16 }, (_, t) => v.getUint8(p + 32 + t)).join(",");
  return `view play=${flag(0)} write=${flag(1)} edges=${flag(2)} sounds=${flag(4)} step=${flag(3) ? v.getUint8(p + 25) : "-"}` +
    ` sel=${v.getUint8(p + 26)} vol=${v.getUint8(p + 27)} tempo=${v.getUint32(p + 4, true)} len=${v.getUint8(p + 28)} swing=${v.getUint8(p + 29)}` +
    ` held=${hex(v.getUint32(p + 8, true), 6)} field=${hex(v.getUint16(p + 12, true), 4)} stitch=${hex(v.getUint16(p + 14, true), 4)}` +
    ` bar=${hex(v.getUint16(p + 16, true), 4)} sampled=${hex(v.getUint16(p + 18, true), 4)} a=${v.getUint16(p + 20, true)} b=${v.getUint16(p + 22, true)} fires=${fires}` +
    (format >= 3 ? ` fx=${v.getUint8(p + 55)} fxsteps=${hex(v.getUint16(p + 56, true), 4)} fxsounds=${hex(v.getUint16(p + 58, true), 4)}` : "") +
    (format >= 4 ? ` ea=${v.getUint16(p + 60, true)} eb=${v.getUint16(p + 62, true)}` : "");
}

// With `audio` (a Float32Array of `end * channels` samples) sound is rendered into it as well,
// in that layout (1 the mix, 16 the tracks, 17 both).
// With `views` a block ends wherever one is to be read; each is held to its text, labelled
// `label`, and added to the lines after the trace, as the report has it.
export function run(m, e, cmds, end, block, audio, views = [], label = "", channels = 1, format = 2) {
  const lines = [], seen = [];
  assert.ok(!audio || block * channels <= AUDIO_MAX);
  let next = 0, view = 0;
  for (let start = 0; start < end; ) {
    const n = Math.min(block, (view < views.length ? views[view][0] : end) - start);
    const v = m.view();
    let count = 0;
    for (; next < cmds.length && cmds[next][0] < start + n; next++, count++) {
      assert.ok(count < CMDS_MAX);
      const [frame, c] = cmds[next];
      const p = m.base + CMDS + 12 * count;
      v.setUint32(p, frame - start, true);
      v.setUint8(p + 4, c.type);
      v.setUint8(p + 5, c.a);
      v.setUint8(p + 6, c.b);
      v.setUint8(p + 7, 0);
      v.setUint32(p + 8, c.value, true);
    }
    // §19: a loop layout is read as well as written. The same numbers harness.cpp gives it.
    if (audio && channels >= 18) {
      const io = new Float32Array(m.pi.memory.buffer, m.base + AUDIO, n * channels);
      for (let f = 0; f < n; f++) for (let c = 0; c < channels; c++) io[f * channels + c] = (((start + f) * 31 + c * 17) % 256) / 256 - 0.25;
    }
    const got = m.pi.pi_process(e, n, m.base + CMDS, count, m.base + EVENTS, CAP, audio ? m.base + AUDIO : 0, audio ? channels : 0);
    if (audio) audio.set(new Float32Array(m.pi.memory.buffer, m.base + AUDIO, n * channels), start * channels);
    for (let i = 0; i < got; i++) {
      const p = m.base + EVENTS + 12 * i;
      const tick = v.getUint32(p + 4, true), type = v.getUint8(p + 8);
      const track = v.getUint8(p + 9), step = v.getUint8(p + 10);
      if (type === 1) lines.push(`${tick} step ${step}\n`);
      else if (type === 2) lines.push(`${tick} trig ${track} ${step}\n`);
      else assert.fail(`unknown event type ${type}`);
    }
    start += n;
    for (; view < views.length && views[view][0] === start; view++) {
      assert.equal(m.pi.pi_view(e, m.base + VIEW, VIEW_SIZE), VIEW_SIZE);
      const got = viewText(m.view(), m.base + VIEW, format);
      assert.equal(got, views[view][1], `${label}: the view at frame ${start}`);
      seen.push(got + "\n");
    }
  }
  assert.equal(view, views.length, "every view was read");
  assert.equal(m.pi.pi_dropped(e), 0, "no event was dropped");
  return lines.join("") + seen.join("");
}

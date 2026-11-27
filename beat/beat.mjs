#!/usr/bin/env node
// The agent's command (docs/BEAT.md, "The agent's command"; D140, proposed, for Kenny to rule;
// issue 163). Checks a beat file, renders it to a WAV and reads every bar back as a grid, and
// packs it in canonical form, over the two committed .wasm files: Node and nothing else.
//
//   node beat/beat.mjs check <file.beat>...
//   node beat/beat.mjs render <file.beat> [-o <out.wav>] [--rate 44100|48000|96000] [--times <n>] [--trace]
//   node beat/beat.mjs pack <file.beat> [-o <out.beat>]
//   (any of them: --built takes target/'s .wasm, which `make build` just wrote)
//
// Exit 0: every file read by pib_read, loaded by pi_load (and written). 1: a file refused.
// 2: misuse, a file that cannot be read or written, or --link (issue 7, not built).
//
// The render is the Node harness's own (harness/engine.mjs): the starter pack on every key,
// effect 4 lent its line, the table clock, PLAY 1 on frame 0, blocks of 128 frames.
//
// SPDX-License-Identifier: MIT
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, isAbsolute, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { BEAT_WASM, beatRead, beatWrite, loadBeat } from "../harness/beat.mjs";
import { CMD, SNAP, SNAP_ROOM, WASM, engine, load, renderKit, run, schedule } from "../harness/engine.mjs";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const COMMITTED = { core: here("../web/pi_core.wasm"), beat: here("../web/pi_beat.wasm") };
const BUILT = { core: WASM, beat: BEAT_WASM };
const OUT_DIR = here("../target/beat/");

// docs/BEAT.md, "The form": the names a row may carry, keys 1 to 16 (docs/VOICES.md's roles).
export const NAMES = ["kick", "snare", "hat", "openhat", "synsnare", "sticks", "cymbal", "noise", "clap", "click", "lowtom", "hitom", "cowbell", "blip", "tone", "bass"];
export const RATES = [44100, 48000, 96000];
// The tail (docs/BEAT.md): until half a second in a row stays under -80 dBFS, at most 4 s.
export const QUIET = 1e-4, QUIET_SECONDS = 0.5, TAIL_MAX_SECONDS = 4;

const USAGE = `usage:
  node beat/beat.mjs check <file.beat>...
  node beat/beat.mjs render <file.beat> [-o <out.wav>] [--rate 44100|48000|96000] [--times <n>] [--trace]
  node beat/beat.mjs pack <file.beat> [-o <out.beat>]
  --built: the .wasm in target/ (make build) in place of web/'s
exit 0 good, 1 a file refused, 2 misuse or a file that cannot be read or written`;

class Misuse extends Error {}
class Unreadable extends Error {}
class Refused extends Error {}

// "5 732": a space between thousands, as the documents write numbers.
const int = (n) => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
const shown = (p) => {
  const r = relative(process.cwd(), p);
  return r && !r.startsWith("..") && !isAbsolute(r) ? r : p;
};

// What the command reads from a version 5 snapshot (core/src/snapshot_layout.rs's offsets;
// ENGINE.md section 21): the tempo, the name, the chain, the patterns written, each pattern's
// length and which steps of each track are on.
export function describe(snap) {
  const v = new DataView(snap.buffer, snap.byteOffset, snap.length);
  const mbpm = v.getUint32(16, true), written = v.getUint16(92, true);
  const chain = Array.from(snap.subarray(96, 96 + snap[94]));
  const nameBytes = snap.subarray(112, 144);
  const end = nameBytes.indexOf(0);
  const name = new TextDecoder().decode(end < 0 ? nameBytes : nameBytes.subarray(0, end));
  // A pattern with no block is a cleared one: length 16, nothing on (pi_init's).
  const patterns = Array.from({ length: 16 }, () => ({ length: 16, steps: new Array(16).fill(0) }));
  let at = 272;
  for (let p = 0; p < 16; p++) {
    if (!((written >> p) & 1)) continue;
    patterns[p] = { length: snap[at + 1360], steps: Array.from({ length: 16 }, (_, t) => v.getUint16(at + 2 * t, true)) };
    at += 1364;
  }
  const chainSteps = chain.reduce((a, p) => a + patterns[p].length, 0);
  return { mbpm, written, chain, name, patterns, chainSteps };
}

const tempo = (mbpm) => String(mbpm / 1000);
const seconds = (x) => `${x.toFixed(3)} s`;

// One file through the reader and the core: { text, snap, own, info, canonical }, or a throw.
function read(file, kit) {
  let text;
  try {
    text = readFileSync(file);
  } catch (e) {
    const why = { ENOENT: "no such file", EISDIR: "a directory, not a file", EACCES: "not allowed to read it" }[e.code] ?? e.message;
    throw new Unreadable(`${file}: cannot read: ${why}`);
  }
  const got = beatRead(kit.beat, text);
  if (got.code !== undefined) throw new Refused(`${file}: ${got.msg}`);
  // A fresh engine (pi_init), the snapshot loaded and saved again: the core is the referee.
  const m = kit.core, e = engine(m, 48000, 1);
  new Uint8Array(m.pi.memory.buffer, m.base + SNAP, got.snap.length).set(got.snap);
  const code = m.pi.pi_load(e, m.base + SNAP, got.snap.length);
  if (code !== 0) throw new Refused(`${file}: the core refused the beat's snapshot: pi_load returned ${code}`);
  const n = m.pi.pi_save(e, m.base + SNAP, SNAP_ROOM);
  const saved = new Uint8Array(m.pi.memory.buffer, m.base + SNAP, n);
  if (n !== got.snap.length || saved.some((b, i) => b !== got.snap[i])) throw new Refused(`${file}: the core saved other bytes than the reader gave it`);
  const canonical = beatWrite(kit.beat, got.snap, got.own);
  if (typeof canonical === "number") throw new Refused(`${file}: the writer refused the snapshot: pib_write returned ${canonical}`);
  return { text, snap: got.snap, own: got.own, info: describe(got.snap), canonical };
}

function check(file, kit) {
  const b = read(file, kit), i = b.info;
  const used = Array.from({ length: 16 }, (_, p) => p + 1).filter((p) => (i.written >> (p - 1)) & 1);
  let line = `${file}: ok. ${i.name ? `"${i.name}", ` : ""}${tempo(i.mbpm)} bpm, chain ${i.chain.map((p) => p + 1).join(" ")}: ` +
    `${i.chain.length} ${i.chain.length === 1 ? "bar" : "bars"}, ${seconds((i.chainSteps * 15000) / i.mbpm)}; ` +
    `patterns ${used.length ? used.join(" ") : "none"}; snapshot ${int(b.snap.length)} bytes`;
  if (!sameBytes(b.text, b.canonical)) line += " (not canonical: pack writes it so)";
  for (let k = 0; k < 16; k++) if ((b.own >> k) & 1) line += `; key ${k + 1} used its writer's own sample; playing the starter pack's`;
  return line;
}

const sameBytes = (a, b) => a.length === b.length && a.every((x, i) => x === b[i]);

// The render: { audio (the WAV's floats), trace, chainFrames, frames }.
export function renderBeat(kit, snap, rate, times) {
  const info = describe(snap), m = kit.core;
  const e = kit.lend(engine(m, rate, 1), rate);
  new Uint8Array(m.pi.memory.buffer, m.base + SNAP, snap.length).set(snap);
  if (m.pi.pi_load(e, m.base + SNAP, snap.length) !== 0) throw new Error("pi_load refused a snapshot it accepted before");
  // The chain `times` times, as a fixture with a beat runs: SOURCE table and PLAY 1 on frame 0,
  // the tempo the snapshot's; then PLAY 0 where the chain would start over, and up to the
  // longest tail after it.
  const fx = { mbpm: info.mbpm, steps: times * info.chainSteps, commands: [{ at: 0, cmd: "play", value: 1 }] };
  const { cmds, end } = schedule(fx, rate, 0, true);
  cmds.push([end, { type: CMD.play, a: 0, b: 0, value: 0 }]);
  const most = end + TAIL_MAX_SECONDS * rate;
  const audio = new Float32Array(most);
  const trace = run(m, e, cmds, most, 128, audio);
  // The tail ends on the frame that makes half a second in a row under QUIET, counted from PLAY 0.
  const half = QUIET_SECONDS * rate;
  let frames = most;
  for (let f = end, quiet = 0; f < most; f++) {
    quiet = Math.abs(audio[f]) < QUIET ? quiet + 1 : 0;
    if (quiet === half) { frames = f + 1; break; }
  }
  return { audio: audio.subarray(0, frames), trace, chainFrames: end, frames, info };
}

// RIFF WAVE, WAVE_FORMAT_IEEE_FLOAT (3), one channel, 32 bits: the fmt chunk with its cbSize
// (18 bytes, as a format other than PCM has it) and a fact chunk (the frames), then the data,
// the engine's floats as they are (little-endian, as a harness .f32).
export function wav(audio, rate) {
  const data = new Uint8Array(audio.buffer, audio.byteOffset, audio.length * 4);
  const head = new DataView(new ArrayBuffer(58));
  const tag = (at, s) => [...s].forEach((c, i) => head.setUint8(at + i, c.charCodeAt(0)));
  tag(0, "RIFF"); head.setUint32(4, 50 + data.length, true); tag(8, "WAVE");
  tag(12, "fmt "); head.setUint32(16, 18, true);
  head.setUint16(20, 3, true); head.setUint16(22, 1, true); head.setUint32(24, rate, true);
  head.setUint32(28, rate * 4, true); head.setUint16(32, 4, true); head.setUint16(34, 32, true); head.setUint16(36, 0, true);
  tag(38, "fact"); head.setUint32(42, 4, true); head.setUint32(46, audio.length, true);
  tag(50, "data"); head.setUint32(54, data.length, true);
  const out = new Uint8Array(58 + data.length);
  out.set(new Uint8Array(head.buffer), 0);
  out.set(data, 58);
  return out;
}

// Every bar of the render as a grid a sound, from the trace (docs/BEAT.md): `x` a firing, a
// digit where a step's column fired that many times (a repeat's run, ENGINE.md section 17, whose
// hits carry the step it began on), `+` past nine. A bar is one chain entry; its columns are its
// pattern's length. Its rows: the keys with a step on in that pattern, and any that fired.
export function grids(trace, info, times) {
  const bars = [];
  for (let t = 0; t < times; t++) for (const p of info.chain) bars.push({ pattern: p, length: info.patterns[p].length, count: Array.from({ length: 16 }, () => new Array(16).fill(0)) });
  const startOf = [];
  bars.reduce((at, b) => (startOf.push(at), at + b.length), 0);
  let step = -1, bar = 0;
  for (const line of trace.split("\n")) {
    const [, kind, a, b] = line.split(" ");
    if (kind === "step") {
      step++;
      while (bar + 1 < bars.length && step >= startOf[bar + 1]) bar++;
    } else if (kind === "trig" && step >= 0 && step < startOf[bars.length - 1] + bars[bars.length - 1].length) {
      bars[bar].count[+a][+b]++;
    }
  }
  const out = [];
  bars.forEach((b, n) => {
    out.push(`bar ${n + 1} (pattern ${b.pattern + 1})`);
    const steps = info.patterns[b.pattern].steps, inside = (1 << b.length) - 1;
    for (let k = 0; k < 16; k++) {
      if (!(steps[k] & inside) && !b.count[k].some((c) => c)) continue;
      let row = "|";
      for (let s = 0; s < b.length; s++) {
        const c = b.count[k][s];
        row += c === 0 ? "." : c === 1 ? "x" : c <= 9 ? String(c) : "+";
        if (s % 4 === 3 || s === b.length - 1) row += "|";
      }
      out.push(`${String(k + 1).padStart(4)} ${NAMES[k].padEnd(8)} ${row}`);
    }
  });
  return out;
}

function peakOf(audio) {
  let peak = 0;
  for (const s of audio) if (Math.abs(s) > peak) peak = Math.abs(s);
  return peak;
}

// The arguments after the verb: the files, and the flags the verb takes.
function parse(verb, args) {
  const takes = { check: ["--built"], render: ["-o", "--rate", "--times", "--trace", "--built"], pack: ["-o", "--built", "--link"] }[verb];
  const valued = new Set(["-o", "--rate", "--times"]);
  const files = [], flags = {};
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a.startsWith("-") && a !== "-") {
      if (!takes.includes(a)) throw new Misuse(`${verb}: unknown flag ${a}`);
      if (a in flags) throw new Misuse(`${verb}: ${a} given twice`);
      if (valued.has(a)) {
        if (i + 1 >= args.length) throw new Misuse(`${verb}: ${a} needs a value`);
        flags[a] = args[++i];
      } else flags[a] = true;
    } else files.push(a);
  }
  if (!files.length) throw new Misuse(`${verb}: no file given`);
  if (verb !== "check" && files.length > 1) throw new Misuse(`${verb}: one file at a time`);
  if ("--rate" in flags && !RATES.includes(+flags["--rate"])) throw new Misuse(`${verb}: --rate is 44100, 48000 or 96000`);
  const times = flags["--times"] ?? "1";
  if (!/^\d+$/.test(times) || +times < 1 || +times > 16) throw new Misuse(`${verb}: --times is 1 to 16`);
  return { files, flags, rate: +(flags["--rate"] ?? 48000), times: +times };
}

function kitOf(built) {
  const paths = built ? BUILT : COMMITTED;
  const core = (() => { try { return load(paths.core); } catch { throw new Unreadable(`cannot load ${shown(paths.core)}${built ? ": run make build" : ""}`); } })();
  const beat = (() => { try { return loadBeat(paths.beat); } catch { throw new Unreadable(`cannot load ${shown(paths.beat)}${built ? ": run make build" : ""}`); } })();
  let lend;
  // The starter pack and effect 4's line, given once to the module and lent to each engine.
  return { core, beat, get lend() { return (lend ??= renderKit(core, true, true)); } };
}

export function main(argv, out = (s) => process.stdout.write(s), err = (s) => process.stderr.write(s)) {
  const [verb, ...args] = argv;
  try {
    if (verb === "--help" || verb === "-h" || verb === "help") { out(USAGE + "\n"); return 0; }
    if (!["check", "render", "pack"].includes(verb)) throw new Misuse(verb ? `unknown command ${verb}` : "no command given");
    const { files, flags, rate, times } = parse(verb, args);
    if (flags["--link"]) throw new Misuse("pack --link: the link is issue 7's, not built yet");
    const kit = kitOf(flags["--built"]);
    if (verb === "check") {
      let worst = 0;
      for (const file of files) {
        try {
          out(check(file, kit) + "\n");
        } catch (e) {
          if (!(e instanceof Refused || e instanceof Unreadable)) throw e;
          out(e.message + "\n");
          worst = Math.max(worst, e instanceof Unreadable ? 2 : 1);
        }
      }
      return worst;
    }
    const file = files[0], b = read(file, kit);
    if (verb === "pack") {
      if (!flags["-o"]) { out(b.canonical); return 0; }
      try { writeFileSync(flags["-o"], b.canonical); } catch (e) { throw new Unreadable(`${flags["-o"]}: cannot write: ${e.code ?? e.message}`); }
      return 0;
    }
    const r = renderBeat(kit, b.snap, rate, times);
    const target = flags["-o"] ?? `${OUT_DIR}${basename(file).replace(/\.beat$/, "")}.wav`;
    try {
      if (!flags["-o"]) mkdirSync(dirname(target), { recursive: true });
      writeFileSync(target, wav(r.audio, rate));
    } catch (e) {
      throw new Unreadable(`${target}: cannot write: ${e.code ?? e.message}`);
    }
    const peak = peakOf(r.audio);
    const head = `${shown(target)}: ${int(rate)} Hz, mono, 32-bit float; ${seconds(r.chainFrames / rate)} of chain, ` +
      `${seconds((r.frames - r.chainFrames) / rate)} of tail; peak ${peak.toFixed(3)} (${peak ? (20 * Math.log10(peak)).toFixed(1) : "-inf"} dBFS)`;
    if (flags["--trace"]) { err(head + "\n"); out(r.trace); }
    else out([head, ...grids(r.trace, r.info, times)].join("\n") + "\n");
    return 0;
  } catch (e) {
    if (e instanceof Misuse) { err(`${e.message}\n${USAGE}\n`); return 2; }
    if (e instanceof Unreadable) { err(e.message + "\n"); return 2; }
    if (e instanceof Refused) { err(e.message + "\n"); return 1; }
    throw e;
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) process.exitCode = main(process.argv.slice(2));

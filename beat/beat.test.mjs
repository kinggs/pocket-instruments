// The agent's command, beat/beat.mjs, held to docs/BEAT.md's "The agent's command" (D140,
// proposed; issue 163). `make test` runs it (`node --test beat/`), after `make build`, so the
// .wasm in target/ is there for --built. Each case runs the command as an agent would, a Node
// process of its own, and reads its exit code, its output and the files it wrote.
//
// SPDX-License-Identifier: MIT
import { after, test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { beatRead, loadBeat } from "../harness/beat.mjs";
import { SNAP, engine, load, renderKit, run, schedule } from "../harness/engine.mjs";
import { describe } from "./beat.mjs";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const COMMAND = here("beat.mjs");
const BEATS = here("../beats/"), FIXTURES = here("../fixtures/");
const bank = readdirSync(BEATS).filter((n) => n.endsWith(".beat")).sort();
const scratch = mkdtempSync(join(tmpdir(), "pi-beat-command-"));
after(() => rmSync(scratch, { recursive: true, force: true }));

// The command, as an agent runs it: { code, out, err }, the output as bytes and as text.
function beat(...args) {
  const r = spawnSync(process.execPath, [COMMAND, ...args], { cwd: scratch, maxBuffer: 1 << 26 });
  return { code: r.status, outBytes: r.stdout, out: r.stdout.toString("utf8"), err: r.stderr.toString("utf8") };
}

// A WAV the command wrote, read by its bytes: the header's every field, then the floats.
function readWav(path, rate) {
  const b = readFileSync(path), v = new DataView(b.buffer, b.byteOffset, b.length);
  const tag = (at) => b.subarray(at, at + 4).toString("latin1");
  assert.equal(tag(0), "RIFF");
  assert.equal(v.getUint32(4, true), b.length - 8, "RIFF's length is the file's less eight");
  assert.equal(tag(8), "WAVE");
  assert.equal(tag(12), "fmt ");
  assert.equal(v.getUint32(16, true), 18, "fmt is 18 bytes, with cbSize");
  assert.equal(v.getUint16(20, true), 3, "WAVE_FORMAT_IEEE_FLOAT");
  assert.equal(v.getUint16(22, true), 1, "mono");
  assert.equal(v.getUint32(24, true), rate);
  assert.equal(v.getUint32(28, true), rate * 4, "bytes a second");
  assert.equal(v.getUint16(32, true), 4, "block align");
  assert.equal(v.getUint16(34, true), 32, "bits");
  assert.equal(v.getUint16(36, true), 0, "cbSize");
  assert.equal(tag(38), "fact");
  assert.equal(v.getUint32(42, true), 4);
  const frames = v.getUint32(46, true);
  assert.equal(tag(50), "data");
  assert.equal(v.getUint32(54, true), frames * 4, "data is the fact's frames, four bytes each");
  assert.equal(b.length, 58 + frames * 4, "nothing after the data");
  return { bytes: b, data: b.subarray(58), audio: new Float32Array(b.buffer.slice(b.byteOffset + 58, b.byteOffset + b.length)) };
}

// docs/BEAT.md's first line of a render.
const HEAD = /^(.+): ([\d ]+) Hz, mono, 32-bit float; (\d+\.\d{3}) s of chain, (\d+\.\d{3}) s of tail; peak (\d+\.\d{3}) \((-?\d+\.\d|-inf) dBFS\)$/;

test("check: every .beat in fixtures/ and beats/ exits 0, one ok line a file", () => {
  const files = [...readdirSync(FIXTURES).filter((n) => n.endsWith(".beat")).map((n) => FIXTURES + n), ...bank.map((n) => BEATS + n)];
  assert.ok(files.length >= 8);
  for (const built of [[], ["--built"]]) {
    const r = beat("check", ...files, ...built);
    assert.equal(r.code, 0, r.out + r.err);
    const lines = r.out.trimEnd().split("\n");
    assert.equal(lines.length, files.length);
    lines.forEach((l, i) => assert.ok(l.startsWith(`${files[i]}: ok. `) && !l.includes("not canonical"), l));
  }
  // docs/BEAT.md's line for the Amen, word for word.
  assert.equal(beat("check", BEATS + "amen-break.beat").out,
    `${BEATS}amen-break.beat: ok. "Amen Break", 136 bpm, chain 1 2 3 4: 4 bars, 7.059 s; patterns 1 2 3 4; snapshot 5 732 bytes\n`);
});

test("check: a refused text exits 1 with the crate's message, byte for byte", () => {
  // Kept here, not as a .beat in fixtures/ or beats/: the core's tests read every one of those.
  const cases = [
    ["wrong-name.txt", "pocket-instruments beat 1\n\npattern 1\n1 snare x...............\n", "line 4, column 3: key 1 is the kick, not the snare"],
    ["short-row.txt", "pocket-instruments beat 1\nvalue a 1 1\n\npattern 1\n1  kick  a...a...a...a..\n", "line 5, column 10: a row takes sixteen steps; this has 15"],
    ["newer.txt", "pocket-instruments beat 2\ntempo 99\n", "line 1: this is beat 2; this build reads beat 1"],
  ];
  for (const [name, text, msg] of cases) {
    const file = join(scratch, name);
    writeFileSync(file, text);
    const r = beat("check", file);
    assert.equal(r.code, 1, name);
    assert.equal(r.out, `${file}: ${msg}\n`, name);
    // render and pack refuse it the same way, on standard error, and write nothing.
    for (const verb of ["render", "pack"]) {
      const s = beat(verb, file, "-o", join(scratch, `${name}.${verb}`));
      assert.equal(s.code, 1, `${verb} ${name}`);
      assert.equal(s.err, `${file}: ${msg}\n`);
      assert.equal(s.out, "");
      assert.throws(() => readFileSync(join(scratch, `${name}.${verb}`)), `${verb} wrote nothing`);
    }
  }
  // A good file and a refused one: the refused one sets the exit, the good one is still told.
  const r = beat("check", BEATS + "floor.beat", join(scratch, "newer.txt"));
  assert.equal(r.code, 1);
  assert.match(r.out, /floor\.beat: ok\. .*\n.*newer\.txt: line 1: this is beat 2; this build reads beat 1\n$/);
});

test("refusals and misuse: the exit codes and the messages", () => {
  mkdirSync(join(scratch, "a-folder"), { recursive: true });
  writeFileSync(join(scratch, "noise.bin"), Uint8Array.from({ length: 512 }, (_, i) => (i * 37 + 11) & 255));
  const missing = join(scratch, "nowhere.beat");
  let r = beat("check", missing);
  assert.deepEqual([r.code, r.out], [2, `${missing}: cannot read: no such file\n`]);
  r = beat("check", join(scratch, "a-folder"));
  assert.deepEqual([r.code, r.out], [2, `${join(scratch, "a-folder")}: cannot read: a directory, not a file\n`]);
  r = beat("render", missing);
  assert.deepEqual([r.code, r.err], [2, `${missing}: cannot read: no such file\n`]);
  // A binary file is read and refused by the crate, on line 1.
  r = beat("check", join(scratch, "noise.bin"));
  assert.equal(r.code, 1);
  assert.ok(r.out.startsWith(`${join(scratch, "noise.bin")}: line 1`), r.out);
  // An unreadable file outranks a refused one: 2.
  r = beat("check", join(scratch, "noise.bin"), missing);
  assert.equal(r.code, 2);
  const misuse = [
    [[], "no command given"],
    [["play", BEATS + "floor.beat"], "unknown command play"],
    [["check"], "check: no file given"],
    [["check", BEATS + "floor.beat", "--rate", "48000"], "check: unknown flag --rate"],
    [["render", BEATS + "floor.beat", "--loud"], "render: unknown flag --loud"],
    [["render", BEATS + "floor.beat", BEATS + "stutter.beat"], "render: one file at a time"],
    [["render", BEATS + "floor.beat", "--rate", "22050"], "render: --rate is 44100, 48000 or 96000"],
    [["render", BEATS + "floor.beat", "--times", "17"], "render: --times is 1 to 16"],
    [["render", BEATS + "floor.beat", "--times", "0"], "render: --times is 1 to 16"],
    [["render", BEATS + "floor.beat", "-o"], "render: -o needs a value"],
    [["render", BEATS + "floor.beat", "--trace", "--trace"], "render: --trace given twice"],
    [["pack", BEATS + "floor.beat", "--link"], "pack --link: the link is issue 7's, not built yet"],
  ];
  for (const [args, msg] of misuse) {
    r = beat(...args);
    assert.equal(r.code, 2, args.join(" "));
    assert.equal(r.out, "");
    assert.ok(r.err.startsWith(`${msg}\nusage:\n`), r.err);
  }
  r = beat("pack", BEATS + "floor.beat", "-o", join(scratch, "no-folder", "x.beat"));
  assert.deepEqual([r.code, r.err], [2, `${join(scratch, "no-folder", "x.beat")}: cannot write: ENOENT\n`]);
  r = beat("--help");
  assert.equal(r.code, 0);
  assert.ok(r.out.startsWith("usage:\n"));
});

test("pack: every bank file comes back unchanged; a file in another form comes back canonical", () => {
  for (const name of bank) {
    const r = beat("pack", BEATS + name);
    assert.equal(r.code, 0, r.err);
    assert.deepEqual(r.outBytes, readFileSync(BEATS + name), `${name} is canonical`);
  }
  const loose = join(scratch, "loose.beat");
  writeFileSync(loose, "pocket-instruments beat 1\n# a comment\npattern 1\n  1 kick x...x...x...x...   \ntempo 120.000\nseed 0001\n");
  const c = beat("check", loose);
  assert.equal(c.code, 0);
  assert.ok(c.out.endsWith(" (not canonical: pack writes it so)\n"), c.out);
  const r = beat("pack", loose, "-o", loose);
  assert.equal(r.code, 0, r.err);
  assert.equal(r.out, "");
  assert.equal(readFileSync(loose, "utf8"), "pocket-instruments beat 1\nseed 1\n\npattern 1\n1  kick x...x...x...x...\n");
  assert.ok(!beat("check", loose).out.includes("not canonical"));
  // Longer than PIB_TEXT_MAX (96 KiB) with comments: the reader says it reads, not the room.
  const long = join(scratch, "long.beat");
  writeFileSync(long, readFileSync(BEATS + "floor.beat", "utf8") + "# a comment, to make the file long\n".repeat(3000));
  const l = beat("check", long);
  assert.equal(l.code, 0, l.out + l.err);
  assert.ok(l.out.includes(": ok. \"Floor\"") && l.out.endsWith(" (not canonical: pack writes it so)\n"), l.out);
});

test("render: the grids worked by hand for the Amen Break and Stutter (beat/expected/)", () => {
  for (const name of ["amen-break", "stutter"]) {
    const expected = readFileSync(here(`expected/${name}.txt`), "utf8").split("\n").filter((l) => l && !l.startsWith("#"));
    const r = beat("render", BEATS + `${name}.beat`, "-o", join(scratch, `${name}.wav`));
    assert.equal(r.code, 0, r.err);
    const [head, ...bars] = r.out.trimEnd().split("\n");
    assert.match(head, HEAD);
    assert.deepEqual(bars, expected, name);
  }
});

test("render: every bank file exits 0, peak under 1, the chain plus a tail, the same WAV from the committed and the built .wasm", () => {
  for (const name of bank) {
    const out = (w) => join(scratch, `${name}.${w}.wav`);
    const a = beat("render", BEATS + name, "-o", out("web"));
    const b = beat("render", BEATS + name, "-o", out("built"), "--built");
    assert.equal(a.code, 0, a.err);
    assert.equal(b.code, 0, b.err);
    const wa = readWav(out("web"), 48000), wb = readWav(out("built"), 48000);
    assert.ok(wa.bytes.equals(wb.bytes), `${name}: the committed and the built .wasm render the same bytes`);
    const [, file, rate, chain, tail, peak] = a.out.split("\n")[0].match(HEAD);
    assert.equal(file, `${name}.web.wav`, "the WAV's path, from where the command ran");
    assert.equal(rate, "48 000");
    let max = 0;
    for (const s of wa.audio) max = Math.max(max, Math.abs(s));
    assert.ok(max < 1, `${name}: peak ${max}`);
    assert.equal(peak, max.toFixed(3));
    // The chain: each entry's pattern length times a step, the harness's frame for that step.
    const info = describe(beatRead(loadBeat(here("../web/pi_beat.wasm")), readFileSync(BEATS + name)).snap);
    const frames = Number((BigInt(info.chainSteps) * 48000n * 15000n + BigInt(info.mbpm) - 1n) / BigInt(info.mbpm));
    assert.equal(chain, (frames / 48000).toFixed(3));
    // The tail: at least the half second of quiet, at most 4 s, and that half second is quiet.
    const tailFrames = wa.audio.length - frames;
    assert.equal(tail, (tailFrames / 48000).toFixed(3));
    assert.ok(tailFrames >= 24000 && tailFrames <= 4 * 48000, `${name}: tail ${tailFrames}`);
    if (tailFrames < 4 * 48000) {
      for (let f = wa.audio.length - 24000; f < wa.audio.length; f++) assert.ok(Math.abs(wa.audio[f]) < 1e-4, `${name}: frame ${f} in the quiet half second`);
      assert.ok(tailFrames === 24000 || Math.abs(wa.audio[wa.audio.length - 24001]) >= 1e-4, `${name}: the tail ends on the first quiet half second`);
    }
  }
});

test("render: the worked example, through the command and through the harness, the same trace and the same samples at every rate", () => {
  const fx = JSON.parse(readFileSync(FIXTURES + "beat-four-and-a-fill.json", "utf8"));
  const first80 = fx.expect.filter((l) => +l.split(" ")[0] < 80 * 96).map((l) => l + "\n").join("");
  assert.equal(first80, fx.expect.map((l) => l + "\n").join(""), "the fixture is 80 steps");
  // The harness's own render of the fixture (trace.test.mjs's way, with P7's starter pack and line).
  const m = load(here("../web/pi_core.wasm")), kit = renderKit(m, true, true);
  const snap = beatRead(loadBeat(here("../web/pi_beat.wasm")), readFileSync(FIXTURES + fx.beat)).snap;
  for (const rate of [44100, 48000, 96000]) {
    const wavFile = join(scratch, `four-${rate}.wav`);
    const r = beat("render", FIXTURES + "beat-four-and-a-fill.beat", "--trace", "--times", "2", "--rate", String(rate), "-o", wavFile);
    assert.equal(r.code, 0, r.err);
    assert.match(r.err.trimEnd(), HEAD, "the WAV's line goes to standard error under --trace");
    const lines = r.out.split("\n").filter((l) => l && +l.split(" ")[0] < 80 * 96).map((l) => l + "\n").join("");
    assert.equal(lines, first80, `${rate}: the first 80 steps are the fixture's hand-worked trace`);
    assert.ok(r.out.startsWith(first80));
    const e = kit(engine(m, rate, fx.seed + 12345), rate);
    new Uint8Array(m.pi.memory.buffer, m.base + SNAP, snap.length).set(snap);
    assert.equal(m.pi.pi_load(e, m.base + SNAP, snap.length), 0);
    const { cmds, end } = schedule(fx, rate, 0, true);
    const audio = new Float32Array(end);
    assert.equal(run(m, e, cmds, end, 128, audio), first80);
    const w = readWav(wavFile, rate);
    assert.ok(w.audio.length > end);
    assert.ok(Buffer.from(audio.buffer).equals(w.data.subarray(0, 4 * end)), `${rate}: the WAV's first ${end} frames are the harness's floats, bit for bit`);
  }
});

test("render: without --trace the grid of every bar the chain plays, --times passes of it", () => {
  const r = beat("render", FIXTURES + "beat-four-and-a-fill.beat", "--times", "2", "-o", join(scratch, "four.wav"));
  assert.equal(r.code, 0, r.err);
  const bars = r.out.split("\n").filter((l) => l.startsWith("bar "));
  assert.deepEqual(bars, ["1 (pattern 1)", "2 (pattern 1)", "3 (pattern 1)", "4 (pattern 2)", "5 (pattern 1)", "6 (pattern 1)", "7 (pattern 1)", "8 (pattern 2)"].map((s) => `bar ${s}`));
  // Bar 4, the fill: docs/BEAT.md's worked example, step 15's 70% firing on seed 1's first draw.
  const fill = r.out.split("bar 4 (pattern 2)\n")[1].split("bar 5")[0];
  assert.equal(fill, "   1 kick     |x...|x...|x.x.|....|\n   2 snare    |....|....|....|xxxx|\n   3 hat      |..x.|..x.|....|....|\n");
});

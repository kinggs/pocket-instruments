// P2 in Node: the raw .wasm, no glue, every fixture on both clock sources at three sample
// rates and three block sizes. fixtures/README.md says how a harness runs one; the Rust
// tests (core/src/tests/fixtures.rs) and harness.cpp do the same and write the same report.
//
// Skips when the .wasm has not been built, so `node --test harness/` passes on a machine
// with no Rust. `make test` sets PI_REQUIRE_WASM so a skip is a failure.
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { beatRead, loadBeat } from "./beat.mjs";
import { CMD, EDITS, HOST, RATES, SNAP, SNAP_ROOM, WASM, engine, load, renderKit, run, schedule, toCmd } from "./engine.mjs";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const FIXTURES = here("../fixtures/");
const built = existsSync(WASM);
const skip = built || process.env.PI_REQUIRE_WASM ? false : "pi_core.wasm is not built (make build)";

const BLOCKS = [1, 128, 512];
const SOURCES = [["table", 0], ["edges", 1]];
// The loading, the engine, the starter pack, effect 4's line, the schedule and the run are in
// engine.mjs, shared with the agent's command (beat/beat.mjs; issue 163).

// A snapshot of the fixture's pattern, from an engine given the tempo and the step-0 edits only.
function snapshotOf(m, fx, rate) {
  const e = engine(m, rate, fx.seed);
  const cmds = [[0, { type: CMD.tempo, a: 0, b: 0, value: fx.mbpm }]];
  for (const c of fx.commands) if (c.at === 0 && EDITS.has(c.cmd)) cmds.push([0, toCmd(c)]);
  run(m, e, cmds, 1, 1);
  // §21: version 5, as long as pi_save says: 276 bytes and 1 364 a pattern written.
  const n = m.pi.pi_save(e, m.base + SNAP, SNAP_ROOM);
  assert.ok(n >= 276 && (n - 276) % 1364 === 0, `pi_save returned ${n}`);
  return new Uint8Array(m.pi.memory.buffer, m.base + SNAP, n).slice();
}

// The zip CRC-32, bit by bit (ENGINE.md §8).
function crc32(bytes) {
  let c = 0xffffffff;
  for (const b of bytes) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return (c ^ 0xffffffff) >>> 0;
}

// §20: the fixture's snapshot with every step that is on keeping its sound's own A and B. Since
// ABI 16 written into version 5's bytes, in each pattern written, as a KEEP there would write
// them; then loaded and saved again, so the bytes are the core's (core/src/tests/fixtures.rs).
function keptSnapshotOf(m, fx, rate, snap) {
  const kept = snap.slice();
  const u16 = (i) => kept[i] | (kept[i + 1] << 8);
  let at = 272;
  for (let p = 0; p < 16; p++) {
    if (!((u16(92) >> p) & 1)) continue;
    for (let t = 0; t < 16; t++) {
      const on = u16(at + 2 * t), has = u16(at + 304 + 2 * t);
      for (let st = 0; st < 16; st++) if ((on >> st) & 1 && !((has >> st) & 1)) kept.copyWithin(at + 336 + 4 * (16 * t + st), 24 + 4 * t, 28 + 4 * t);
      kept[at + 304 + 2 * t] = (on | has) & 0xff;
      kept[at + 305 + 2 * t] = (on | has) >> 8;
    }
    at += 1364;
  }
  new DataView(kept.buffer).setUint32(kept.length - 4, crc32(kept.subarray(0, kept.length - 4)), true);
  const e = engine(m, rate, fx.seed);
  new Uint8Array(m.pi.memory.buffer, m.base + SNAP, kept.length).set(kept);
  assert.equal(m.pi.pi_load(e, m.base + SNAP, kept.length), 0);
  assert.equal(m.pi.pi_save(e, m.base + SNAP, SNAP_ROOM), kept.length);
  assert.deepEqual(new Uint8Array(m.pi.memory.buffer, m.base + SNAP, kept.length).slice(), kept, "the core saves the same bytes");
  return kept;
}

const hex = (bytes) => Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");

test("the module imports nothing and its memory grows by a page for the host", { skip }, () => {
  const m = load();
  assert.deepEqual(m.imports, [], "no imports: no glue to write");
  assert.equal(m.pi.pi_abi_version(), 16); // ABI 16: sixteen patterns, the view at 88, snapshot version 5 (§21)
  assert.equal(m.pi.memory.buffer.byteLength, m.base + HOST);
  // Memory the module was not given is refused, not written.
  assert.equal(m.pi.pi_init(0, m.pi.pi_size(), 48000, 1), 0, "null");
  assert.equal(m.pi.pi_init(m.base + 1, m.pi.pi_size(), 48000, 1), 0, "misaligned");
});

test("every fixture gives its trace on both sources, at every rate and block size", { skip }, () => {
  const m = load();
  const names = readdirSync(FIXTURES).filter((n) => n.endsWith(".json")).sort();
  assert.ok(names.length >= 10);
  let report = "", runs = 0, beats = 0;
  for (const file of names) {
    const fx = JSON.parse(readFileSync(FIXTURES + file, "utf8"));
    const name = file.replace(/\.json$/, "");
    assert.ok([1, 2, 3, 4, 5].includes(fx.format));
    assert.ok(fx.format >= 2 || !fx.views, "views are format 2's to 5's");
    const format = fx.format; // format 3's views carry §17's fields, format 4's §20's too, format 5's §21's
    assert.equal(fx.ticks_per_step, 96);
    // A run's text is its trace, then its views: run() holds each view to its own line.
    const trace = fx.expect.map((l) => l + "\n").join("");
    const wanted = (views) => trace + views.map((v) => v[1] + "\n").join("");
    // A fixture with a beat file (fixtures/README.md, format 5): read by pib_read from
    // pi_beat.wasm and loaded before frame 0; no tempo or step-0 edits of the harness's and no
    // run from a snapshot, since the beat is one. Its bytes go in the report once.
    if (fx.beat) {
      const got = beatRead(loadBeat(), readFileSync(FIXTURES + fx.beat));
      assert.ok(got.snap, `${name}: ${got.msg}`);
      report += `# ${name} beat ${hex(got.snap)}\n`;
      for (const [sourceName, source] of SOURCES) {
        for (const rate of RATES) {
          for (const block of BLOCKS) {
            const label = `${name} ${sourceName} ${rate} ${block}`;
            const e = engine(m, rate, fx.seed + 12345);
            new Uint8Array(m.pi.memory.buffer, m.base + SNAP, got.snap.length).set(got.snap);
            assert.equal(m.pi.pi_load(e, m.base + SNAP, got.snap.length), 0, label);
            const { cmds, end, views } = schedule(fx, rate, source, true);
            const text = run(m, e, cmds, end, block, null, views, label, 1, format);
            assert.equal(text, wanted(views), label);
            report += `# ${label}\n${text}`;
            runs++;
          }
        }
      }
      beats++;
      continue;
    }
    for (const [sourceName, source] of SOURCES) {
      for (const rate of RATES) {
        for (const block of BLOCKS) {
          const label = `${name} ${sourceName} ${rate} ${block}`;
          const { cmds, end, views } = schedule(fx, rate, source, false);
          const got = run(m, engine(m, rate, fx.seed), cmds, end, block, null, views, label, 1, format);
          assert.equal(got, wanted(views), label);
          report += `# ${name} ${sourceName} ${rate} ${block}\n${got}`;
          runs++;
        }
        // Save, load into a fresh engine with another seed, same trace.
        const snap = snapshotOf(m, fx, rate);
        const e = engine(m, rate, fx.seed + 12345);
        new Uint8Array(m.pi.memory.buffer, m.base + SNAP, snap.length).set(snap);
        assert.equal(m.pi.pi_load(e, m.base + SNAP, snap.length), 0);
        const { cmds, end, views } = schedule(fx, rate, source, true);
        const got = run(m, e, cmds, end, 128, null, views, `${name} ${sourceName} ${rate} from a snapshot`, 1, format);
        assert.equal(got, wanted(views), `${name} ${sourceName} ${rate} from a snapshot`);
        report += `# ${name} ${sourceName} ${rate} snapshot ${hex(snap)}\n${got}`;
        runs++;
        // §20 (Session 21): and from the snapshot a beat entered by keys saves, every step that
        // is on keeping its sound's own A and B (core/src/tests/fixtures.rs says why).
        const kept = keptSnapshotOf(m, fx, rate, snap);
        const k = engine(m, rate, fx.seed + 12345);
        new Uint8Array(m.pi.memory.buffer, m.base + SNAP, kept.length).set(kept);
        assert.equal(m.pi.pi_load(k, m.base + SNAP, kept.length), 0);
        const gotKept = run(m, k, cmds, end, 128, null, views, `${name} ${sourceName} ${rate} from a kept snapshot`, 1, format);
        assert.equal(gotKept, wanted(views), `${name} ${sourceName} ${rate} from a kept snapshot`);
        report += `# ${name} ${sourceName} ${rate} kept snapshot ${hex(kept)}\n${gotKept}`;
        runs++;
        // One flipped bit is refused.
        const bad = new Uint8Array(m.pi.memory.buffer, m.base + SNAP, kept.length);
        bad[40] ^= 1;
        assert.equal(m.pi.pi_load(e, m.base + SNAP, kept.length), -4);
      }
    }
  }
  assert.equal(runs, (names.length - beats) * 2 * 3 * 5 + beats * 2 * 3 * 3);
  if (process.env.PI_REPORT) writeFileSync(process.env.PI_REPORT, report);
});

// §16: the weave from the .wasm, for the set of views in fixtures/weave-views.txt, as text: a
// digit a cell. The Rust tests and harness.cpp write the same file natively, and `make test`
// holds the three to the same bytes. It needs no engine: the view and the cells sit in the
// host's page.
test("the set of views weaves, from the .wasm", { skip }, () => {
  const m = load();
  const W = 78, H = 15, CELLS = W * H;
  const lines = readFileSync(here("../fixtures/weave-views.txt"), "utf8").split("\n").filter((l) => l && !l.startsWith("#"));
  assert.ok(lines.length >= 40, `the set is ${lines.length} views`);
  let report = "";
  for (const l of lines) {
    const [name, flags, hex] = l.split(" ");
    assert.equal(hex.length, 128, name);
    const mem = new Uint8Array(m.pi.memory.buffer);
    for (let i = 0; i < 64; i++) mem[m.base + i] = parseInt(hex.slice(2 * i, 2 * i + 2), 16);
    mem.fill(0xee, m.base + 64, m.base + 64 + CELLS + 1);
    // Refused, and nothing written: a short view, too little room.
    assert.equal(m.pi.pi_weave(m.base, 63, +flags, m.base + 64, CELLS), 0);
    assert.equal(m.pi.pi_weave(m.base, 64, +flags, m.base + 64, CELLS - 1), 0);
    assert.ok(mem.subarray(m.base + 64, m.base + 64 + CELLS).every((b) => b === 0xee));
    assert.equal(m.pi.pi_weave(m.base, 64, +flags, m.base + 64, CELLS), CELLS);
    assert.equal(mem[m.base + 64 + CELLS], 0xee, "nothing past the cells");
    report += `# ${name} ${flags}\n`;
    for (let y = 0; y < H; y++) report += Array.from(mem.subarray(m.base + 64 + y * W, m.base + 64 + (y + 1) * W), (c) => String(c)).join("") + "\n";
  }
  if (process.env.PI_WEAVE) writeFileSync(process.env.PI_WEAVE, report);
});

// P7: the same scenarios harness.cpp renders (every harness/render-*.json), from the .wasm.
// Written only when asked (`make test` sets PI_RENDER); compare.mjs then holds each pair to
// the same bytes.
const SCENARIOS = readdirSync(here(".")).filter((n) => /^render-.*\.json$/.test(n)).sort();
for (const file of SCENARIOS) {
  const name = file.replace(/^render-|\.json$/g, "");
  test(`P7: ${name} renders from the .wasm`, { skip }, () => {
    const m = load();
    const fx = JSON.parse(readFileSync(here(file), "utf8"));
    // "samples" gives every track the starter pack; "delay": 1 lends effect 4 its line (engine.mjs).
    const withPack = renderKit(m, fx.samples, fx.delay);
    // "layouts": the mix alone unless the scenario asks for more. A layout but the mix is
    // written with its channel count in the name (harness.cpp does the same).
    // "loops": the loop layouts (§19), each rendered with its inputs given (run() says how).
    const layouts = [...(fx.layouts ?? [1]), ...(fx.loops ?? [])];
    assert.ok(layouts.includes(1) && (fx.layouts ?? [1]).every((c) => [1, 16, 17].includes(c)) && (fx.loops ?? []).every((c) => [18, 50].includes(c)));
    for (const [sourceName, source] of SOURCES) {
      for (const rate of RATES) {
        const { cmds, end } = schedule(fx, rate, source, false);
        const quiet = run(m, withPack(engine(m, rate, fx.seed), rate), cmds, end, 128);
        const out = {};
        for (const channels of layouts) {
          const audio = new Float32Array(end * channels);
          const loud = run(m, withPack(engine(m, rate, fx.seed), rate), cmds, end, 128, audio, [], "", channels);
          assert.equal(loud, quiet, "sound changes no event");
          assert.ok(audio.some((s) => s !== 0) && audio.every(Number.isFinite), "it made a sound");
          out[channels] = new Uint32Array(audio.buffer);
          const suffix = channels === 1 ? "" : `-${channels}ch`;
          if (process.env.PI_RENDER) writeFileSync(`${process.env.PI_RENDER}-${name}-${sourceName}-${rate}${suffix}.f32`, new Uint8Array(audio.buffer));
        }
        // §15: the seventeen channels are the sixteen, then the mix, bit for bit.
        if (out[17]) {
          assert.ok(out[16], "layout 17 is held to layout 16");
          for (let f = 0; f < end; f++) {
            for (let t = 0; t < 16; t++) if (out[17][f * 17 + t] !== out[16][f * 16 + t]) assert.fail(`${sourceName} ${rate}: frame ${f} track ${t}`);
            if (out[17][f * 17 + 16] !== out[1][f]) assert.fail(`${sourceName} ${rate}: frame ${f}, the mix`);
          }
        }
      }
    }
  });
}

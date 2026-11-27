// The beat file's reader and writer from pi_beat.wasm, the way the page will call them (docs/BEAT.md;
// beat/include/pi_beat.h): every case the Rust tests wrote (target/beat-cases.txt), with the core's
// .wasm as the referee, into the same report core/src/tests/beat.rs and `harness --beat` write.
// `make test` compares the three.
//
// Skips when the .wasm has not been built, as trace.test.mjs does; `make test` sets
// PI_REQUIRE_WASM so a skip is a failure.
//
// SPDX-License-Identifier: MIT
import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { BEAT_WASM, beatRead, beatWrite, fnv, loadBeat } from "./beat.mjs";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const CORE_WASM = here("../target/wasm32-unknown-unknown/release/pi_core.wasm");
const built = existsSync(BEAT_WASM) && existsSync(CORE_WASM);
const skip = built || process.env.PI_REQUIRE_WASM ? false : "pi_beat.wasm is not built (make build)";
const PAGE = 65536;

// The core's .wasm, for the referee: an engine in the first 32 KiB of two added pages, a snapshot after it.
function core() {
  const pi = new WebAssembly.Instance(new WebAssembly.Module(readFileSync(CORE_WASM)), {}).exports;
  const base = pi.memory.grow(2) * PAGE;
  return { pi, base, snap: base + 32768 };
}

function coreSavesTheSame(c, bytes, label) {
  const e = c.pi.pi_init(c.base, c.pi.pi_size(), 48000, 99);
  assert.equal(e, c.base);
  new Uint8Array(c.pi.memory.buffer, c.snap, bytes.length).set(bytes);
  assert.equal(c.pi.pi_load(e, c.snap, bytes.length), 0, `${label}: pi_load`);
  const n = c.pi.pi_save(e, c.snap, 22100);
  assert.deepEqual(new Uint8Array(c.pi.memory.buffer, c.snap, n).slice(), bytes, `${label}: pi_save gives the same bytes`);
}

const unhex = (h) => Uint8Array.from({ length: h.length / 2 }, (_, i) => parseInt(h.slice(2 * i, 2 * i + 2), 16));

test("pi_beat.wasm imports nothing and exports its memory and the two calls", { skip }, () => {
  const b = loadBeat();
  assert.deepEqual(b.imports, [], "no imports: no glue to write");
  assert.deepEqual(b.exports.sort(), ["memory", "pib_read", "pib_write"]);
});

test("the beat cases through pi_beat.wasm, the core's .wasm the referee", { skip }, () => {
  const cases = process.env.PI_BEAT_CASES;
  if (!cases) return; // written by the Rust tests: `make test` runs them first and passes the path
  const b = loadBeat(), c = core();
  let report = "", snaps = 0, texts = 0;
  for (const line of readFileSync(cases, "utf8").split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const [kind, label, h] = line.split(" ");
    const bytes = unhex(h);
    if (kind === "snap") {
      const text = beatWrite(b, bytes, 0x0101);
      assert.ok(typeof text !== "number", `${label}: pib_write refused the core's snapshot: ${text}`);
      const back = beatRead(b, text);
      assert.ok(back.snap, `${label}: ${back.msg}`);
      assert.deepEqual(back.snap, bytes, `${label}: text and back, the same bytes`);
      assert.deepEqual(beatWrite(b, back.snap, back.own), text, `${label}: the same text again`);
      coreSavesTheSame(c, back.snap, label);
      report += `${label} snap ${bytes.length} text ${text.length} ${fnv(text)} back ${back.snap.length} ${fnv(back.snap)} own ${back.own}\n`;
      snaps++;
    } else {
      assert.equal(kind, "text");
      const got = beatRead(b, bytes);
      if (got.code !== undefined) report += `${label} refused ${got.code} ${got.msg}\n`;
      else {
        coreSavesTheSame(c, got.snap, label);
        const text = beatWrite(b, got.snap, got.own);
        report += `${label} read ${got.snap.length} ${fnv(got.snap)} own ${got.own} text ${text.length} ${fnv(text)}\n`;
      }
      texts++;
    }
  }
  assert.ok(snaps >= 50 && texts >= 100, "the cases");
  if (process.env.PI_BEAT) writeFileSync(process.env.PI_BEAT, report);
});

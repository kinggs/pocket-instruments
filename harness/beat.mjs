// The beat file's reader and writer from pi_beat.wasm (docs/BEAT.md; beat/include/pi_beat.h), the
// raw module, no glue: what a page does to open and save a beat. trace.test.mjs reads a fixture's
// beat with it; beat.test.mjs runs the Rust tests' cases through it.
//
// SPDX-License-Identifier: MIT
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
export const BEAT_WASM = here("../target/wasm32-unknown-unknown/release/pi_beat.wasm");

// pi_beat.h's numbers.
export const PIB_TEXT_MAX = 98304, PIB_MSG_MAX = 192, PIB_SNAPSHOT_MAX = 22100;
const PAGE = 65536;

// The module, and where its buffers sit in the two pages the host adds: the text, the snapshot,
// the message, own.
export function loadBeat(path = BEAT_WASM) {
  const module = new WebAssembly.Module(readFileSync(path));
  const imports = WebAssembly.Module.imports(module);
  const pib = new WebAssembly.Instance(module, {}).exports;
  const base = pib.memory.grow(2) * PAGE;
  const at = { text: base, snap: base + PIB_TEXT_MAX, msg: base + PIB_TEXT_MAX + PIB_SNAPSHOT_MAX, own: base + PIB_TEXT_MAX + PIB_SNAPSHOT_MAX + PIB_MSG_MAX };
  return { pib, imports, at, exports: WebAssembly.Module.exports(module).map((e) => e.name) };
}

// pib_read: { snap, own } or { code, msg }.
// A text longer than PIB_TEXT_MAX (a file an agent wrote with long comments, say) is put in
// pages added for it, so the reader, not the room, says whether it reads (issue 163).
export function beatRead(b, text) {
  const at = text.length > PIB_TEXT_MAX ? b.pib.memory.grow(Math.ceil(text.length / PAGE)) * PAGE : b.at.text;
  new Uint8Array(b.pib.memory.buffer, at, text.length).set(text);
  const n = b.pib.pib_read(at, text.length, b.at.snap, PIB_SNAPSHOT_MAX, b.at.own, b.at.msg, PIB_MSG_MAX);
  const mem = new Uint8Array(b.pib.memory.buffer);
  if (n < 0) {
    const end = mem.indexOf(0, b.at.msg);
    return { code: n, msg: new TextDecoder().decode(mem.subarray(b.at.msg, end)) };
  }
  return { snap: mem.slice(b.at.snap, b.at.snap + n), own: new DataView(b.pib.memory.buffer).getUint32(b.at.own, true) };
}

// pib_write: the text's bytes, or the code below zero.
export function beatWrite(b, snap, own) {
  new Uint8Array(b.pib.memory.buffer, b.at.snap, snap.length).set(snap);
  const n = b.pib.pib_write(b.at.snap, snap.length, own >>> 0, b.at.text, PIB_TEXT_MAX);
  if (n < 0) return n;
  return new Uint8Array(b.pib.memory.buffer).slice(b.at.text, b.at.text + n);
}

// FNV-1a, 32 bits, as core/src/tests/beat.rs hashes a case's bytes for the report.
export function fnv(bytes) {
  let h = 0x811c9dc5;
  for (const x of bytes) h = Math.imul(h ^ x, 0x01000193) >>> 0;
  return h.toString(16).padStart(8, "0");
}

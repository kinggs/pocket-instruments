// web/rhythm.html's own checks, loaded only under ?auto. web/play.mjs opens the page and judges
// what each phase posts back; the bar is written there. Every press here is a PointerEvent on
// the drawn key, so it goes the way a finger goes. Sliders are moved through the page's own
// slide(), which its drag calls: a drag cannot aim at an exact value such as 40000.
//
// Three phases, one page load each, in one browser profile:
//   1       stopped, enter straight-sixteenths by keys; play; the trace. Two pointers choose a
//           sound. Then P6's browser half: clear, and enter it again while it plays; the
//           snapshot the page saved.
//   reload  the beat kept in the browser is back.
//   bpm     keys-bpm-family's edits and keys; the last view, as text.
//   latch   Shift+click latches (D60, D61; Ctrl+click until D113), Ctrl+click is plain, Shift+7
//           q chooses by the computer keys, a sample's dash while sound is latched, and the
//           sounds sheet's two halves.
//   keep    §20 (D75, Session 21): the cap is the hand's and stays put when another sound
//           is chosen; a load moves it; a step keeps what it was written with, and the
//           second mark shows what the chosen sound was last hit with.
//   punch   §17 (Session 19): fx-repeat-punch (format 3) played on the page, fx and key 11
//           by touch, fx alone by the computer key; its views and its trace.
//   shuffle §17, effect 13 (Session 28): fx-shuffle (format 3) played on the page by its
//           edits; its views and its trace.
//   written keeps-what-it-was-written-with (format 4) by touch: its views.
//   fxlatch D78 by a mouse, fx + key by the computer keys, and the sounds sheet's "through
//           fx" switch (D69); each gesture held to the commands it sent.
//   keys    the one-key shortcuts and the keys helper (D116, D119 items 3 and 6).
//   still   the weave (Session 14, ENGINE.md §16): two sounds a step played as it is, then
//           with reduced motion asked for, then stopped.
//   pchoose, pchain, pcopy, pchange   (issue 141, ENGINE.md §21) the four patterns-* fixtures
//           (format 5), each in a fresh page, played by touch: their views and traces.
//   pshift  D135 by a mouse and by the computer keys (Shift on a grid key keeps a latched
//           pattern; Shift latches write for a copy), clear and esc; the flicker, stopped on an
//           empty pattern and playing on a used one, with no frame, core or twill drawn while
//           pattern is held; the display's waiting pattern and chain place; the save throttle.
//   old     a version 4 snapshot in the page's storage loads and plays as it did.
// And in every phase, every drawing's lights while pattern is held or latched (§21: field
// but the active key, stitch, bar and pressed as the view says; no frame, core or twill) and
// the display's pattern line; and every drawing's weave: the cells read back off the canvas by their
// colour and held to pi_weave for the view they were drawn from, from this file's own
// instance of the .wasm.
//
// SPDX-License-Identifier: MIT
const r = window.rhythm;
const phase = new URLSearchParams(location.search).get("auto");
const out = { phase, agent: navigator.userAgent, rate: r.ctx.sampleRate, failed: [], drawn: { checked: 0, bad: [] } };
const sleep = (ms) => new Promise((res) => setTimeout(res, ms));
const u16 = (v, i) => v[i] | (v[i + 1] << 8);
const u32 = (v, i) => (v[i] | (v[i + 1] << 8) | (v[i + 2] << 16) | (v[i + 3] << 24)) >>> 0;
const hex = (buf) => [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
// ENGINE.md §21: pattern is down while the view's held bit 17 is set or the page holds it latched.
const patternDown = (v) => !!((u32(v, 8) >> 17) & 1) || r.latched.has(17);
// What every drawing showed while pattern was down: [time, the active key's field drawn, the
// active pattern, whether it is used]; and how often a frame, a core, a twill would have shown
// and did not, and the display's chain place and waiting pattern were drawn.
let flicks = [];
out.pheld = { framesHidden: 0, playing: 0, coresHidden: 0, twillsHidden: 0, chainShown: 0, nextShown: 0 };

// ENGINE.md §14, "As text"; format 3 adds §17's three fields, format 4 §20's two.
function viewText(v, format = 2) {
  const f = v[24], h4 = (x) => x.toString(16).padStart(4, "0");
  return `view play=${f & 1} write=${(f >> 1) & 1} edges=${(f >> 2) & 1} sounds=${(f >> 4) & 1} ` +
    `step=${f & 8 ? v[25] : "-"} sel=${v[26]} vol=${v[27]} tempo=${u32(v, 4)} len=${v[28]} swing=${v[29]} ` +
    `held=${u32(v, 8).toString(16).padStart(6, "0")} field=${h4(u16(v, 12))} stitch=${h4(u16(v, 14))} bar=${h4(u16(v, 16))} ` +
    `sampled=${h4(u16(v, 18))} a=${u16(v, 20)} b=${u16(v, 22)} fires=${[...v.slice(32, 48)].join(",")}` +
    (format >= 3 ? ` fx=${v[55]} fxsteps=${h4(u16(v, 56))} fxsounds=${h4(u16(v, 58))}` : "") +
    (format >= 4 ? ` ea=${u16(v, 60)} eb=${u16(v, 62)}` : "") +
    (format >= 5 ? ` pat=${v[64]} pos=${v[65]} chain=${nibbles(v, 72, v[66]) || "-"} next=${nibbles(v, 80, v[67]) || "-"} used=${h4(u16(v, 68))}` : "");
}
// ENGINE.md §21, format 5: a chain's entries, one hex digit each; entry 2i in the low four bits
// of byte at + i, 2i + 1 in the high four.
const nibbles = (v, at, n) => Array.from({ length: n }, (_, i) => ((v[at + (i >> 1)] >> (4 * (i & 1))) & 15).toString(16)).join("");

// The weave (ENGINE.md §16). Every drawing's canvas is read back, a pixel a cell, turned to
// palette numbers by colour (design/DESIGN.md's tokens, written out here), and held to
// pi_weave for the view it was drawn from, called in this file's own instance of the .wasm.
// It shares nothing with the page's weave() but the core. Reduced motion is the browser's
// setting, or a phase asks for it (`askStill`, with the page's rhythm.reduced).
// It also holds "painted only when the cells change": from one drawing to the next the page
// painted if and only if the cells, or the canvas's size, changed.
// `travel`: the most different drawings seen for one step of the cloth with the same sounds
// fired (the shuttle travelling); `whites`: drawings with a white cell (rule 6's flash).
const WEAVE_W = 78, WEAVE_H = 15, CELLS = WEAVE_W * WEAVE_H;
const TOKENS = { "110,106,146": 1, "242,232,208": 2, "255,255,255": 3, "240,180,41": 4, "43,41,66": 5, "240,101,74": 6 };
const mine = (await WebAssembly.instantiate(await (await fetch("pi_core.wasm")).arrayBuffer(), {})).instance.exports;
const mineBase = mine.memory.grow(1) * 65536;
function weaveOf(v, flags) {
  const mem = new Uint8Array(mine.memory.buffer);
  mem.set(v, mineBase);
  if (mine.pi_weave(mineBase, 64, flags, mineBase + 64, CELLS) !== CELLS) throw new Error("pi_weave wrote nothing");
  return mem.slice(mineBase + 64, mineBase + 64 + CELLS);
}
const prefers = matchMedia("(prefers-reduced-motion: reduce)").matches;
let askStill = false;
const tally = () => ({ drawings: 0, whites: 0, travel: 0, steps: new Map() });
let seen = tally();
out.weave = { checked: 0, bad: [], prefers };
let lastWant = null, lastWidth = 0, lastPaints = 0;
function cloth(v) {
  const c = r.cloth, w = out.weave;
  const fail = (s) => { if (w.bad.length < 5) w.bad.push(`${s} (woven ${u32(v, 48)}, wove ${u16(v, 52).toString(16)}, fired ${v[54]}, tick ${u32(v, 0)}; ${viewText(v, 4)})`); };
  if (c.hidden || !c.width || !c.height) return fail("the weave is not shown");
  const want = weaveOf(v, prefers || askStill ? 1 : 0);
  const cw = c.width / WEAVE_W, px = c.getContext("2d").getImageData(0, 0, c.width, c.height).data;
  const got = new Uint8Array(CELLS);
  for (let i = 0; i < CELLS; i++) {
    const x = Math.floor((i % WEAVE_W + 0.4) * cw), y = Math.floor((Math.floor(i / WEAVE_W) + 0.4) * cw);
    const o = 4 * (y * c.width + x);
    got[i] = y >= c.height ? 98 : px[o + 3] === 0 ? 0 : TOKENS[`${px[o]},${px[o + 1]},${px[o + 2]}`] ?? 99;
  }
  w.checked++;
  const at = got.findIndex((g, i) => g !== want[i]);
  if (at >= 0) fail(`cell ${at % WEAVE_W},${Math.floor(at / WEAVE_W)} is painted ${got[at]} and pi_weave says ${want[at]}; ${got.filter((g, i) => g !== want[i]).length} cells differ`);
  // Painted if and only if something changed since the drawing before.
  const changed = !lastWant || c.width !== lastWidth || want.some((x, i) => x !== lastWant[i]);
  if (lastWant && changed !== (r.paints !== lastPaints)) fail(changed ? "the cells changed and the page did not paint" : "the page painted and no cell had changed");
  lastWant = want; lastWidth = c.width; lastPaints = r.paints;
  seen.drawings++;
  if (got.includes(3)) seen.whites++;
  if (v[24] & 1 && v[24] & 8) {
    const key = `${u32(v, 48)} ${v[54]}`, set = seen.steps.get(key) || new Set();
    seen.steps.set(key, set.add(got.join("")));
    seen.travel = Math.max(seen.travel, set.size);
  }
}

// Where a slider's cap sits for a value, from the track's height (design/DESIGN.md: the cap
// is 26 px tall and travels the rest).
const topFor = (which, value) => (1 - value / 65535) * Math.max(0, r.sliders[which].querySelector(".tr").clientHeight - 26);
let capWant = null; // where the hand is, when a phase moves it apart from the chosen sound's own

// Every drawing is read back off the face and held to the view it was drawn from. This reads
// the classes the CSS lights and rebuilds the masks; it shares nothing with draw().
r.afterDraw = (v) => {
  out.drawn.checked++;
  let field = 0, stitch = 0, bar = 0, held = 0, frames = [], latch = 0;
  r.keys.forEach((el, n) => {
    const c = el.classList;
    if (c.contains("on")) field |= 1 << n;
    if (c.contains("ghost")) stitch |= 1 << n;
    if (c.contains("bar")) bar |= 1 << n;
    if (c.contains("press")) held |= 1 << n;
    if (c.contains("play")) frames.push(n);
  });
  r.fkeys.forEach((el, i) => {
    if (el.classList.contains("press")) held |= 1 << (16 + i);
    if (el.classList.contains("latch")) latch |= 1 << (16 + i);
  });
  const f = v[24];
  // ENGINE.md §21: while pattern is held (bit 17) or latched, no frame; the active pattern's
  // key (byte 64) flickers, so its field is not held to the view here: it is recorded, and
  // flicker() judges its rhythm. Every other key's field is the view's.
  const pdown = patternDown(v), active = v[64], fieldMask = pdown ? ~(1 << active) & 0xffff : 0xffff;
  const want = {
    field: u16(v, 12) & fieldMask, stitch: u16(v, 14), bar: u16(v, 16), held: u32(v, 8),
    frames: f & 1 && f & 8 && !pdown ? String(v[25]) : "", latch: (f & 1 ? 1 << 21 : 0) | (f & 2 ? 1 << 22 : 0),
  };
  const got = { field: field & fieldMask, stitch, bar, held, frames: frames.join(), latch };
  if (pdown) {
    flicks.push([performance.now(), (field >> active) & 1, active, (u16(v, 68) >> active) & 1]);
    if (f & 1 && f & 8) out.pheld.framesHidden++;
    if (f & 1) out.pheld.playing++;
  }
  // §21, "The display": which pattern plays, the chain's place, the one waiting, as text.
  want.patline = `pattern ${active + 1}` + (v[66] > 1 ? ` ${v[65] + 1}/${v[66]}` : "") + (v[67] ? ` → ${(v[80] & 15) + 1}` : "");
  got.patline = document.getElementById("patline").textContent;
  if (v[66] > 1) out.pheld.chainShown++;
  if (v[67]) out.pheld.nextShown++;
  // D81: the effect block says byte 55, and is absent at 0; the lane's ticks are bytes 56 and
  // 57. D69: the sheet's "through fx" switches are bytes 58 and 59. Read off the face.
  const mark = document.getElementById("fxmark");
  want.fxmark = v[55] ? `fx ${v[55]}` : "none";
  got.fxmark = mark.hidden || getComputedStyle(mark).visibility === "hidden" ? "none" : mark.textContent;
  want.lane = u16(v, 56);
  got.lane = r.ticks.reduce((m, el, n) => m | (el.classList.contains("on") && getComputedStyle(el).backgroundColor !== "rgba(0, 0, 0, 0)" ? 1 << n : 0), 0);
  want.through = u16(v, 58);
  got.through = r.sounds.reduce((m, row, t) => m | (row.fx.classList.contains("through") ? 1 << t : 0), 0);
  // §20: the second mark is drawn from bytes 60 to 63, always. The cap is drawn from 20 to
  // 23 while bpm is held; otherwise it is the hand's, which equals 20 to 23 unless a phase
  // says (capWant) where the hand is. Held only on a view that holds everything sent: a cap
  // moved by the hand is ahead of the view until then.
  for (const which of [0, 1]) {
    const key = which ? "b" : "a";
    want[`hit ${key}`] = topFor(which, u16(v, 60 + 2 * which)) + 12;
    got[`hit ${key}`] = parseFloat(r.sliders[which].querySelector(".hit").style.top);
    if (r.latest.taken >= r.sent) {
      const hand = !(u32(v, 8) & (1 << 18)) && capWant ? capWant[which] : u16(v, 20 + 2 * which);
      want[`cap ${key}`] = topFor(which, hand);
      got[`cap ${key}`] = parseFloat(r.sliders[which].querySelector(".cap").style.top);
    }
  }
  const bad = Object.keys(want).filter((k) => (typeof want[k] === "number" && k.includes(" ") ? Math.abs(want[k] - got[k]) > 0.01 : want[k] !== got[k]));
  if (bad.length && out.drawn.bad.length < 5) out.drawn.bad.push(`${bad.map((k) => `${k} drawn ${got[k]}, view ${want[k]}`).join("; ")} (${viewText(v, 4)})`);
  lights(v);
  cloth(v);
};

// D120 (ENGINE.md §14 and §17, designed 2026-11-26): a key's lights read off the face by
// their colour and their display, sharing nothing with draw(). This keeps its own record of
// when each track's count in `fires` moved. The core: lit for half a step at the view's tempo
// (40 to 120 ms) after the count moves, in every row, write on or off; fern while `heard` is
// not 0 and fx sounds names the sound; a drawing within 15 ms of a core's end is not judged.
// The face is white only under a finger. The twill: key `heard` − 1 alone. The dash: a sample,
// while sound is held or latched. Fern: the fx key's band, the fx block and the lane's ticks.
// Until D120 this was white(), holding a firing's white face to the write-off row (D82 item 4).
const WHITE = "rgb(255, 255, 255)", FERN = "rgb(111, 214, 160)";
const seenFires = new Array(16).fill(null), firedAt = new Array(16).fill(-1e9);
out.core = { writeOn: 0, writeOff: 0, fern: 0, twill: 0, dash: 0, fernDrawn: 0, bad: [] };
const shown = (el) => !!el && getComputedStyle(el).display !== "none";
function lights(v) {
  const now = performance.now(), held = u32(v, 8), heard = v[55], through = u16(v, 58), sampled = u16(v, 18);
  const span = Math.max(40, Math.min(120, 7500000 / Math.max(1, u32(v, 4))));
  const soundDown = !!((held >> 16) & 1) || r.latched.has(16);
  const C = out.core, bad = (s) => { if (C.bad.length < 5) C.bad.push(`${s} (${viewText(v, 5)})`); };
  // §21: while pattern is held or latched, no core and no twill (R90: "it stops the normal
  // playing views"). Counted where one would otherwise show, so the phase proves it was seen.
  const pdown = patternDown(v);
  r.keys.forEach((el, n) => {
    const f = v[32 + n];
    if (seenFires[n] !== null && f !== seenFires[n]) firedAt[n] = now;
    seenFires[n] = f;
    const co = el.querySelector(".co"), left = firedAt[n] + span - now, lit = shown(co);
    const fern = lit && getComputedStyle(co).backgroundColor === FERN;
    if (pdown) {
      if (lit) bad(`key ${n + 1}: a core drawn while pattern is down`);
      else if (left > 15) out.pheld.coresHidden++;
    } else if (Math.abs(left) > 15) {
      if (lit !== left > 0) bad(`key ${n + 1}: core ${lit ? "drawn" : "not drawn"}, ${Math.round(left)} ms of core left`);
      else if (lit && fern !== !!(heard && (through >> n) & 1)) bad(`key ${n + 1}: core ${fern ? "fern" : "white"}, heard ${heard}, through ${through.toString(16)}`);
    }
    if (lit) { if (v[24] & 2) C.writeOn++; else C.writeOff++; if (fern) C.fern++; }
    const face = getComputedStyle(el).backgroundColor;
    if ((face === WHITE) !== !!((held >> n) & 1)) bad(`key ${n + 1}: face ${face}, held ${(held >> n) & 1}`);
    const tw = shown(el.querySelector(".tw"));
    if (tw !== (heard === n + 1 && !pdown)) bad(`key ${n + 1}: twill ${tw}, heard ${heard}, pattern down ${pdown}`);
    if (pdown && heard === n + 1) out.pheld.twillsHidden++;
    if (tw) C.twill++;
    const dash = shown(el.querySelector(".mk"));
    if (dash !== (soundDown && !!((sampled >> n) & 1))) bad(`key ${n + 1}: dash ${dash}, sound down ${soundDown}, sampled ${sampled.toString(16)}`);
    if (dash) C.dash++;
  });
  const fxKey = r.fkeys[4], mark = document.getElementById("fxmark");
  if (!fxKey.classList.contains("press") && !fxKey.classList.contains("latched") && getComputedStyle(fxKey).borderTopColor !== FERN)
    bad(`the fx key's band is ${getComputedStyle(fxKey).borderTopColor}`);
  if (heard) {
    if (getComputedStyle(mark).backgroundColor !== FERN) bad(`the fx block is ${getComputedStyle(mark).backgroundColor}`);
    else C.fernDrawn++;
  }
  r.ticks.forEach((el, n) => { if (el.classList.contains("on") && getComputedStyle(el).backgroundColor !== FERN) bad(`tick ${n + 1} is ${getComputedStyle(el).backgroundColor}`); });
}

// Pointers, as a finger gives them.
function pointer(el, type, id) {
  el.dispatchEvent(new PointerEvent(type, { pointerId: id, bubbles: true, cancelable: true, isPrimary: false, pointerType: "touch" }));
}
const hold = (el, id) => pointer(el, "pointerdown", id);
const lift = (el, id) => pointer(el, "pointerup", id);
const tap = (el, id) => { hold(el, id); lift(el, id); };
const keyEl = (k) => (k < 16 ? r.keys[k] : r.fkeys[k - 16]);

// A view that holds everything sent so far (and, if asked, says something more).
async function settle(pred = () => true, what = "a view", ms = 8000) {
  for (let t = 0; t < ms / 10; t++) {
    const v = r.latest.view;
    if (v && r.latest.taken >= r.sent && pred(v)) return v;
    await sleep(10);
  }
  throw new Error(`timed out waiting for ${what}`);
}
async function snapshot(ms = 8000) {
  for (let t = 0; t < ms / 10; t++) {
    if (r.latest.snapshot && r.latest.snapTaken >= r.sent) return r.latest.snapshot;
    await sleep(10);
  }
  throw new Error("timed out waiting for a snapshot holding every press");
}
const next = (key) => new Promise((res) => r.listeners.push(function f(d) {
  if (d[key] !== undefined) { r.listeners.splice(r.listeners.indexOf(f), 1); res(d); }
}));

// straight-sixteenths' pattern, by keys, write on: choose each sound, tap its steps.
const PATTERN = [[0, [0, 4, 8, 12]], [1, [4, 12]], [2, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]]];
let pid = 100;
async function choose(t) {
  const s = pid++, k = pid++;
  hold(r.fkeys[0], s);
  tap(r.keys[t], k);
  lift(r.fkeys[0], s);
  await settle((v) => v[26] === t, `sound ${t + 1} chosen`);
}
async function enter() {
  for (const [t, steps] of PATTERN) {
    await choose(t);
    for (const s of steps) tap(r.keys[s], pid++);
    await settle();
  }
  await choose(0);
}
const traceLines = (t) => {
  const lines = [];
  for (let i = 0; i < t.length; i += 4)
    lines.push(t[i + 1] === 1 ? `${t[i]} step ${t[i + 3]}` : t[i + 1] === 2 ? `${t[i]} trig ${t[i + 2]} ${t[i + 3]}` : `${t[i]} unknown ${t[i + 1]}`);
  return lines;
};

async function phase1() {
  try { localStorage.clear(); } catch {}
  await r.ready;
  // As a phone is before its first touch: no audio thread running. The touch must start it
  // and count as a press.
  await r.ctx.suspend();
  tap(r.fkeys[6], 1);
  const v0 = await settle((v) => v[24] & 2, "write on after the first touch");
  // Firefox can report "running" a moment after the first block has run.
  for (let i = 0; i < 50 && r.ctx.state !== "running"; i++) await sleep(10);
  out.firstTouch = { state: r.ctx.state, write: !!(v0[24] & 2) };

  await enter();
  // The audio clock against the page's from play to stop, for P6's "no dropout".
  const clocks = [];
  r.listeners.push((d) => { if (d.frames !== undefined) clocks.push([d.frames, performance.now()]); });
  r.post({ mark: true });
  tap(r.fkeys[5], pid++); // play: RESET then PLAY 1 (§13), so ticks start at 0
  await settle((v) => v[24] & 1 && u32(v, 0) >= 32 * 96 + 48, "two bars played", 15000);
  r.post({ trace: true });
  const t = (await next("trace")).trace;
  out.trace = traceLines(t).filter((l) => +l.split(" ")[0] < 32 * 96);

  // Two pointers at once: sound held, a key pressed. Then both up, and nothing stays held.
  hold(r.fkeys[0], 61);
  hold(r.keys[5], 62);
  const v2 = await settle((v) => (u32(v, 8) & 0x10020) === 0x10020, "sound and key 6 held together");
  out.two = { sel: v2[26], held: u32(v2, 8) };
  lift(r.keys[5], 62);
  lift(r.fkeys[0], 61);
  out.two.after = u32(await settle((v) => u32(v, 8) === 0, "nothing held"), 8);

  // P6's browser half: clear while it plays, enter the pattern again by keys while it plays.
  hold(r.fkeys[3], 71);
  hold(r.fkeys[1], 72);
  lift(r.fkeys[1], 72);
  lift(r.fkeys[3], 71);
  await settle((v) => u16(v, 12) === 0 && u16(v, 14) === 0 && u32(v, 8) === 0, "the pattern cleared");
  await enter();
  const v3 = await settle();
  const snap = await snapshot();
  let stored = null;
  try { stored = localStorage.getItem("pocket-instruments/rhythm/beat"); } catch {}
  out.p6 = {
    playing: !!(v3[24] & 1), hex: hex(snap), stored: stored ? hex(Uint8Array.from(atob(stored), (c) => c.charCodeAt(0))) : null,
    // How far the audio clock fell behind the page's while it played.
    behindMs: clocks.length > 1 ? Math.round((clocks.at(-1)[1] - clocks[0][1]) - ((clocks.at(-1)[0] - clocks[0][0]) / r.ctx.sampleRate) * 1000) : null,
    seconds: clocks.length > 1 ? +((clocks.at(-1)[1] - clocks[0][1]) / 1000).toFixed(1) : 0,
  };
  tap(r.fkeys[5], pid++); // stop
  await settle((v) => !(v[24] & 1), "stopped");
  sessionStorage.setItem("pi-play-hex", out.p6.stored || "");
}

async function phaseReload() {
  await r.ready;
  await r.ctx.resume();
  for (let i = 0; i < 800 && r.latest.loaded === null; i++) await sleep(10);
  const saved = next("snapshot");
  r.post({ save: true });
  out.reload = { loaded: r.latest.loaded, hex: hex((await saved).snapshot), before: sessionStorage.getItem("pi-play-hex") };
}

async function phaseBpm() {
  await r.ready;
  await r.ctx.resume();
  const fx = await (await fetch("../fixtures/keys-bpm-family.json")).json();
  const ids = {};
  for (const c of fx.commands) {
    if (c.cmd === "step") r.send([16, c.track, c.step, c.value]);
    else if (c.cmd === "key_down") hold(keyEl(c.key), (ids[c.key] = pid++));
    else if (c.cmd === "key_up") lift(keyEl(c.key), ids[c.key]);
    else if (c.cmd === "param") r.slide(c.slider, c.value);
    else throw new Error(`a command the page cannot send: ${c.cmd}`);
  }
  out.bpm = { last: viewText(await settle()), want: fx.views.at(-1).view };
}

// D61, by a mouse: Shift+click latches (Ctrl+click until D113), a left-click on a key
// finishes. Each gesture's view is recorded as text; play.mjs holds each to what ENGINE.md §13
// says it should be.
const mouse = (el, type, button, ctrlKey = false, shiftKey = false) => el.dispatchEvent(new PointerEvent(type, {
  pointerId: 1, bubbles: true, cancelable: true, isPrimary: true, pointerType: "mouse", button, ctrlKey, shiftKey,
  buttons: type === "pointerdown" ? 1 << (button === 2 ? 1 : 0) : 0,
}));
// The latched light is the page's own state, drawn on the next frame: wait two frames.
const DOUBLE_GAP = 350; // longer than the page's double-click window, so two pairs stay two
const frames2 = () => new Promise((res) => requestAnimationFrame(() => requestAnimationFrame(res)));
const shiftClick = (el) => { mouse(el, "pointerdown", 0, false, true); mouse(el, "pointerup", 0, false, true); };
const rightClick = (el) => { mouse(el, "pointerdown", 2); mouse(el, "pointerup", 2); };
const click = (el) => { mouse(el, "pointerdown", 0); mouse(el, "pointerup", 0); };
async function phaseLatch() {
  await r.ready;
  await r.ctx.resume();
  const L = (out.latch = {});
  // Ctrl+click sound: held, and lit as latched. Click key 16: chosen, and nothing held.
  shiftClick(r.fkeys[0]);
  let v = await settle((x) => u32(x, 8) & (1 << 16), "sound latched");
  L.soundHeld = u32(v, 8);
  await frames2();
  L.soundLit = r.fkeys[0].classList.contains("latched");
  click(r.keys[15]);
  v = await settle((x) => u32(x, 8) === 0, "the latch let go after key 16");
  L.chosen = v[26];
  await frames2();
  L.afterLit = r.fkeys[0].classList.contains("latched");
  // Ctrl+click bpm, two slider moves (they let nothing go), Ctrl+click bpm again: the tempo
  // is the slider's, not the next preset, because bpm was used.
  shiftClick(r.fkeys[2]);
  await settle((x) => u32(x, 8) & (1 << 18), "bpm latched");
  r.slide(1, 40000);
  r.slide(0, 20000);
  v = await settle((x) => x[29] === 15, "swing 15");
  L.bpmStillHeld = !!(u32(v, 8) & (1 << 18));
  shiftClick(r.fkeys[2]);
  v = await settle((x) => u32(x, 8) === 0, "bpm let go");
  L.tempo = u32(v, 4);
  // Clear by mouse: a step set, write on; Ctrl+click function, click pattern.
  r.send([16, 15, 0, 1]);
  click(r.fkeys[6]);
  v = await settle((x) => x[24] & 2 && u16(x, 12) === 1, "write on, step 1 of sound 16 lit");
  shiftClick(r.fkeys[3]);
  click(r.fkeys[1]);
  v = await settle((x) => u32(x, 8) === 0, "function let go");
  L.fieldAfterClear = u16(v, 12);
  // A right-click does nothing (D61): nothing held, nothing latched.
  rightClick(r.fkeys[0]);
  await sleep(50);
  v = await settle();
  await frames2();
  L.afterRight = { held: u32(v, 8), lit: r.fkeys[0].classList.contains("latched") };
  // Ctrl+click on play is a plain press: it starts. Ctrl+click again stops it.
  shiftClick(r.fkeys[5]);
  v = await settle((x) => x[24] & 1, "Ctrl+click on play started it");
  L.playAfterCtrl = { playing: v[24] & 1, held: u32(v, 8) };
  shiftClick(r.fkeys[5]);
  await settle((x) => !(x[24] & 1), "stopped");
  // D63: a double-click puts a slider back. The chosen sound's A moved, then double-clicked:
  // 32768. With bpm held, A double-clicked: swing 0; B: 120 bpm.
  const dbl = (el) => { click(el); click(el); };
  r.slide(0, 10000);
  await settle((x) => u16(x, 20) === 10000, "slider A at 10000");
  dbl(r.sliders[0]);
  v = await settle((x) => u16(x, 20) === 32768, "slider A back to the middle");
  L.dblA = u16(v, 20);
  hold(r.fkeys[2], 990);
  r.slide(0, 30000);
  r.slide(1, 50000);
  await settle((x) => x[29] > 0 && u32(x, 4) !== 120000, "swing and tempo moved");
  await sleep(DOUBLE_GAP);
  dbl(r.sliders[0]);
  await sleep(DOUBLE_GAP);
  dbl(r.sliders[1]);
  v = await settle((x) => x[29] === 0 && u32(x, 4) === 120000, "swing 0 and 120 bpm");
  L.dblBpm = { swing: v[29], tempo: u32(v, 4) };
  lift(r.fkeys[2], 990);
  await settle((x) => u32(x, 8) === 0, "bpm let go");
  // The sheet: the name plays its sound and changes nothing; the switch changes it and plays it.
  const f = (x) => x[32 + 2];
  const before = f(await settle());
  r.sounds[2].name.click();
  v = await settle((x) => f(x) !== before, "the name of 3 fired it");
  L.nameFired = (f(v) - before + 256) % 256;
  L.nameSampled = u16(v, 18);
  r.sounds[2].sw.click();
  v = await settle((x) => u16(x, 18) & 4, "3 switched to its sample");
  L.switchFired = (f(v) - before + 256) % 256;
  L.switchSampled = u16(v, 18);
  await sleep(50);
  L.switchLabel = r.sounds[2].sw.textContent;
  // D113: Ctrl+click on sound is a plain press now: down while the button is, nothing after.
  mouse(r.fkeys[0], "pointerdown", 0, true);
  v = await settle((x) => u32(x, 8) & (1 << 16), "Ctrl+click held sound");
  L.ctrl = { down: u32(v, 8) };
  mouse(r.fkeys[0], "pointerup", 0, true);
  v = await settle((x) => u32(x, 8) === 0, "Ctrl+click let sound go");
  await frames2();
  L.ctrl.after = u32(v, 8);
  L.ctrl.lit = r.fkeys[0].classList.contains("latched");
  // D112 item 4: with sound latched, key 3's sample shows as a dash, and no other key's.
  shiftClick(r.fkeys[0]);
  await settle((x) => u32(x, 8) & (1 << 16), "sound latched again");
  await frames2();
  L.dash = r.keys.reduce((m, el, n) => m | (getComputedStyle(el.querySelector(".mk")).display !== "none" ? 1 << n : 0), 0);
  shiftClick(r.fkeys[0]);
  await settle((x) => u32(x, 8) === 0, "sound let go");
  await frames2();
  L.dashAfter = r.keys.reduce((m, el, n) => m | (getComputedStyle(el.querySelector(".mk")).display !== "none" ? 1 << n : 0), 0);
  // D113: Shift+7 latches sound by the computer keys; q then chooses sound 5 and lets it go.
  key("keydown", "Digit7", { shiftKey: true });
  key("keyup", "Digit7", { shiftKey: true });
  v = await settle((x) => u32(x, 8) === 1 << 16, "Shift+7 latched sound");
  L.shiftKeys = { held: u32(v, 8) };
  key("keydown", "KeyQ");
  key("keyup", "KeyQ");
  v = await settle((x) => x[26] === 4 && u32(x, 8) === 0, "q chose sound 5");
  await frames2();
  L.shiftKeys.sel = v[26];
  L.shiftKeys.after = u32(v, 8);
  L.shiftKeys.lit = r.fkeys[0].classList.contains("latched");
}
// A computer key by its place (e.code), with what else the event says.
const key = (type, code, more = {}) => window.dispatchEvent(new KeyboardEvent(type, { code, bubbles: true, cancelable: true, ...more }));

// The one-key shortcuts (D116 item 2, D119 item 3) and the keys helper (D116 item 3, D119 item
// 6). What each sent is read from the page's own messages, as "type a b value".
async function phaseKeys() {
  await r.ready;
  await r.ctx.resume();
  const K = (out.keys = {});
  let log = [];
  r.onPost = (m) => { if (m.cmds) log.push(...m.cmds.map((c) => c.join(" "))); };
  const take = () => { const l = log; log = []; return l; };
  const press = async (code, more = {}, what = "a view") => { take(); key("keydown", code, more); key("keyup", code, more); return settle(undefined, what); };
  let v = await settle();
  take();
  await press("Space");
  v = await settle((x) => x[24] & 1, "space played");
  K.space = { sent: take(), playing: v[24] & 1 };
  await press("Space");
  v = await settle((x) => !(x[24] & 1), "space stopped");
  K.space.stopped = !(v[24] & 1);
  take();
  await press("Period");
  v = await settle((x) => u32(x, 4) === 121000, "tempo up");
  K.tempoUp = take();
  await press("Comma");
  v = await settle((x) => u32(x, 4) === 120000, "tempo down");
  K.tempoDown = take();
  await press("Period", { shiftKey: true, key: ">" });
  v = await settle((x) => x[29] === 1, "swing up");
  K.swingUp = take();
  await press("Comma", { shiftKey: true, key: "<" });
  v = await settle((x) => x[29] === 0, "swing down");
  K.swingDown = take();
  await press("Minus");
  v = await settle((x) => x[27] === 15, "volume down");
  K.volume = { down: v[27] };
  await press("Equal");
  v = await settle((x) => x[27] === 16, "volume up");
  K.volume.up = v[27];
  K.volume.held = u32(v, 8);
  K.volume.tempo = u32(v, 4); // using bpm for the volume does not step the tempo
  take();
  const a = r.sliders[0], capA = () => +a.getAttribute("aria-valuenow");
  // Not 32768: a PARAM that changes nothing posts no view while stopped, and settle waits for one.
  r.slide(0, 30000); r.slide(1, 30000);
  await settle((x) => u16(x, 20) === 30000 && u16(x, 22) === 30000, "sliders at 30000");
  take();
  K.sliders = [];
  for (const c of ["KeyT", "KeyG", "KeyY", "KeyH"]) { await press(c); K.sliders.push(...take()); }
  K.capA = capA();
  window.scrollTo(0, 0);
  await press("KeyM");
  await sleep(50);
  K.sheetTop = Math.round(document.querySelector(".sheet").getBoundingClientRect().top);
  window.scrollTo(0, 0);
  // The helper: ? opens it, esc closes it; the ? mark opens it too.
  const open = () => !document.getElementById("helper").hidden;
  key("keydown", "Slash", { shiftKey: true, key: "?" });
  K.helper = { byKey: open() };
  key("keydown", "Escape", { key: "Escape" });
  K.helper.escClosed = !open();
  const mark = document.getElementById("helpmark"), glass = document.querySelector(".display");
  const mr = mark.getBoundingClientRect(), gr = glass.getBoundingClientRect();
  K.helper.corner = Math.round(gr.right - mr.right) <= 2 && Math.round(mr.top - gr.top) <= 2 && mr.width >= 24;
  mark.click();
  K.helper.byMark = open();
  key("keydown", "Escape", { key: "Escape" });
  K.helper.escClosed2 = !open();
  // esc lets every latch go, fx's too.
  shiftClick(r.fkeys[0]);
  shiftClick(r.fkeys[4]);
  await settle((x) => u32(x, 8) & (1 << 16), "sound latched");
  await frames2();
  K.esc = { before: [...r.latched].sort() };
  key("keydown", "Escape", { key: "Escape" });
  v = await settle((x) => u32(x, 8) === 0, "esc let go");
  await frames2();
  K.esc.after = [...r.latched];
  K.esc.lit = r.fkeys.some((el) => el.classList.contains("latched"));
  r.onPost = null;
}

// §20 (D75). A beat saved before Session 21 has each sound's own sliders: here sound 1's A
// at 1000 and sound 2's at 60000, sound 2 on step 1, as a version 3 snapshot made from this
// page's own bytes. Loaded, the cap goes to sound 1's 1000 (a load moves it). Choosing
// sound 2 sounds it (§13) at its own 60000: the mark goes there, the cap stays at 1000. The
// hand then moves A to 20000 (every sound), write on, key 5 writes sound 2's step 5. Played,
// step 1 sounds at the 60000 it kept from the old beat and step 5 at 20000: the mark steps
// between them while the cap stays at 20000.
const crc32 = (bytes) => {
  let c = ~0;
  for (const b of bytes) {
    c ^= b;
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c >>> 0;
};
async function phaseKeep() {
  await r.ready;
  await r.ctx.resume();
  const K = (out.keep = {});
  const saved = next("snapshot");
  r.post({ save: true });
  const now = new Uint8Array((await saved).snapshot);
  K.size = now.length;
  // Since ABI 16 the page saves version 5 (ENGINE.md §21): its fields laid out as version 3
  // had them, pattern 1's from its block if it is written (a cleared pattern otherwise).
  // Until then version 4's first 408 bytes were taken as they were.
  const v3 = new Uint8Array(408), d = new DataView(v3.buffer), n5 = new DataView(now.buffer);
  K.version = n5.getUint16(4, true);
  K.written = n5.getUint16(92, true);
  v3.set(now.subarray(0, 20));                                   // magic, kind, seed, tempo
  const p1 = n5.getUint16(92, true) & 1 ? now.subarray(272, 272 + 1364) : null;
  v3[20] = p1 ? p1[1360] : 16;                                   // length
  v3[21] = now[20];                                              // swing
  v3.fill(100, 56, 312);                                         // probabilities
  if (p1) v3.set(p1.subarray(32, 288), 56);
  d.setUint16(376, n5.getUint16(88, true), true);                // sampled
  v3[378] = now[21];                                             // volume
  if (p1) v3.set(p1.subarray(288, 304), 384);                    // the lane
  d.setUint16(400, n5.getUint16(90, true), true);                // fx sounds
  d.setUint16(4, 3, true);
  d.setUint32(8, 392, true);
  for (let t = 0; t < 16; t++) d.setUint16(24 + 2 * t, 0, true); // no steps but one
  d.setUint16(24 + 2 * 1, 1, true);                              // sound 2, step 1
  for (let t = 0; t < 16; t++) { d.setUint16(312 + 4 * t, 32768, true); d.setUint16(314 + 4 * t, 32768, true); }
  d.setUint16(312, 1000, true);                                  // sound 1's A
  d.setUint16(316, 60000, true);                                 // sound 2's A
  v3[379] = 0;                                                   // sound 1 chosen
  d.setUint32(404, crc32(v3.subarray(0, 404)), true);
  r.post({ load: v3.buffer });
  let v = await settle((x) => u16(x, 20) === 1000 && x[26] === 0, "the old beat loaded");
  await frames2();
  K.loadedCap = Math.abs(parseFloat(r.sliders[0].querySelector(".cap").style.top) - topFor(0, 1000)) < 0.01;
  capWant = [1000, 32768]; // from here the hand is not the chosen sound's own
  await choose(1);
  v = await settle((x) => x[26] === 1 && u16(x, 60) === 60000, "sound 2 chosen and sounded");
  await frames2();
  K.chosen = { a: u16(v, 20), mark: u16(v, 60), capStayed: Math.abs(parseFloat(r.sliders[0].querySelector(".cap").style.top) - topFor(0, 1000)) < 0.01 };
  capWant = [20000, 32768];
  r.slide(0, 20000);
  tap(r.fkeys[6], pid++); // write on
  tap(r.keys[4], pid++);  // sound 2's step 5
  v = await settle((x) => u16(x, 20) === 20000 && x[24] & 2 && u16(x, 12) === 0b10001, "A at 20000, step 5 written");
  tap(r.fkeys[5], pid++); // play
  const marks = new Set();
  for (let i = 0; i < 300 && marks.size < 2; i++) {
    v = await settle((x) => x[24] & 1, "playing");
    if (v[32 + 1] > 1) marks.add(u16(v, 60));
    await sleep(10);
  }
  K.marks = [...marks].sort((a, b) => a - b);
  tap(r.fkeys[5], pid++); // stop
  await settle((x) => !(x[24] & 1), "stopped");
  const after = next("snapshot");
  r.post({ save: true });
  const s = new DataView((await after).snapshot);
  // Version 5 (§21): pattern 1's block at 272, its keeps at 304 in it and the values at 336
  // (version 4's 404 and 436 until ABI 16).
  const P = 272;
  K.kept = { keep: s.getUint16(P + 304 + 2, true), step1: [s.getUint16(P + 336 + 4 * 16, true), s.getUint16(P + 338 + 4 * 16, true)],
    step5: [s.getUint16(P + 336 + 4 * 20, true), s.getUint16(P + 338 + 4 * 20, true)] };
}

// Run steps as the views show them while playing: one more each time the playhead moves,
// from the first view after play that has a step (§14's flag 3). The last view of each run
// step is kept, because a fixture's view is read where its run step ends (fixtures/README.md).
function runSteps() {
  const t = { run: -1, last: null, views: [], waiters: [] };
  const f = (d) => {
    if (!d.view) return;
    const v = new Uint8Array(d.view);
    if (!(v[24] & 1) || !(v[24] & 8)) return;
    if (v[25] !== t.last) { t.run++; t.last = v[25]; }
    t.views[t.run] = v;
    t.waiters = t.waiters.filter(([n, res]) => (t.run >= n ? (res(), false) : true));
  };
  r.listeners.push(f);
  t.at = (n) => (t.run >= n ? Promise.resolve() : new Promise((res) => t.waiters.push([n, res])));
  t.stop = () => r.listeners.splice(r.listeners.indexOf(f), 1);
  return t;
}
// A fixture's edits as the page sends them: [type, a, b, value] (ENGINE.md §6).
const edit = (c) => ({ step: [16, c.track, c.step, c.value], length: [19, 0, 0, c.value], fxsound: [24, c.track, 0, c.value],
  fx: [23, c.step, 0, c.value], play: [5, 0, 0, c.value], prob: [17, c.track, c.step, c.value], swing: [18, 0, 0, c.value] })[c.cmd];
// The computer keys (D57 item 5), by the key's number: a press as a keyboard gives it.
const CODE = { 0: "Digit1", 1: "Digit2", 2: "Digit3", 10: "KeyD", 20: "KeyI" };
const kb = (type, k) => window.dispatchEvent(new KeyboardEvent(type, { code: CODE[k], bubbles: true, cancelable: true }));

// fx-repeat-punch (format 3, ENGINE.md §17), played on the page. A key lands within its run
// step, a few milliseconds after the view says the step began, as a fixture's lands a frame
// after: nothing in the fixture depends on which frame. The first pass's fx and key 11 by
// touch, the second pass's fx alone by the computer key `i`. Every view is held to the
// fixture's, as text, and the trace (its ticks counted from the first) to its `expect`.
async function phasePunch() {
  await r.ready;
  await r.ctx.resume();
  const fx = await (await fetch("../fixtures/fx-repeat-punch.json")).json();
  const P = (out.punch = { want: fx.views.map((x) => x.view), wantTrace: fx.expect });
  await settle();
  const t = runSteps();
  r.post({ mark: true });
  r.send(...fx.commands.filter((c) => c.at === 0).map((c) => {
    const e = edit(c);
    if (!e) throw new Error(`a step-0 command the page cannot send: ${c.cmd}`);
    return e;
  }));
  const ids = {};
  for (const c of fx.commands.filter((c) => c.at > 0)) {
    await t.at(c.at);
    const byKeys = c.at >= 8; // the first pass by touch, the rest by the computer keys
    if (c.cmd === "key_down") byKeys ? kb("keydown", c.key) : hold(keyEl(c.key), (ids[c.key] = pid++));
    else if (c.cmd === "key_up") byKeys ? kb("keyup", c.key) : lift(keyEl(c.key), ids[c.key]);
    else throw new Error(`a command the page cannot send: ${c.cmd}`);
  }
  await t.at(fx.steps);
  t.stop();
  r.post({ trace: true });
  const trace = traceLines((await next("trace")).trace);
  r.send([5, 0, 0, 0]);
  await settle((v) => !(v[24] & 1), "stopped");
  const first = trace.length ? +trace[0].split(" ")[0] : 0;
  P.trace = trace.map((l) => l.replace(/^\d+/, (n) => String(+n - first))).filter((l) => +l.split(" ")[0] < fx.steps * fx.ticks_per_step);
  P.got = fx.views.map((x) => (t.views[x.at] ? viewText(t.views[x.at], 3) : `no view at run step ${x.at}`));
}

// fx-shuffle (format 3, ENGINE.md §17, designed 2026-11-24: effect 13), played on the page by
// its edits, as the page sends them. A fixture's edit at run step n lands on the frame of the
// tick that begins step n, before the clock moves (fixtures/README.md): the page cannot aim a
// message at a frame, so it sends each one during run step n − 1, a few milliseconds after the
// view says that step began. ⚠ Nothing in fx-shuffle reads the lane or the switches between
// those two moments: step n − 1's notes were read at its own boundary. Every view is held to
// the fixture's, as text, and the trace (its ticks counted from the first) to its `expect`.
async function phaseShuffle() {
  await r.ready;
  await r.ctx.resume();
  const fx = await (await fetch("../fixtures/fx-shuffle.json")).json();
  const Q = (out.shuffle = { want: fx.views.map((x) => x.view), wantTrace: fx.expect });
  await settle();
  const t = runSteps();
  r.post({ mark: true });
  const sendable = (c) => {
    const e = edit(c);
    if (!e) throw new Error(`a command the page cannot send: ${c.cmd}`);
    return e;
  };
  // A reset first: the fixture's engine is fresh, at tick 0, and the page's clock has counted
  // since the page loaded. Its swing is by the grid step's parity (§5), so without this the
  // pattern's step 1 can land on an odd one and play 32 ticks late (found in Session 28).
  r.send([2, 0, 0, 0], ...fx.commands.filter((c) => c.at === 0).map(sendable));
  for (const c of fx.commands.filter((c) => c.at > 0)) {
    await t.at(c.at - 1);
    r.send(sendable(c));
  }
  await t.at(fx.steps);
  t.stop();
  r.post({ trace: true });
  const trace = traceLines((await next("trace")).trace);
  r.send([5, 0, 0, 0]);
  await settle((v) => !(v[24] & 1), "stopped");
  const first = trace.length ? +trace[0].split(" ")[0] : 0;
  Q.trace = trace.map((l) => l.replace(/^\d+/, (n) => String(+n - first))).filter((l) => +l.split(" ")[0] < fx.steps * fx.ticks_per_step);
  Q.got = fx.views.map((x) => (t.views[x.at] ? viewText(t.views[x.at], 3) : `no view at run step ${x.at}`));
}

// keeps-what-it-was-written-with (format 4, §20), by touch and the page's slide(): the three
// views while stopped, then play (the edit) and the views at the play's first and fifth steps,
// which are the fixture's run steps 4 and 8 (it plays from run step 4).
async function phaseWritten() {
  await r.ready;
  await r.ctx.resume();
  const fx = await (await fetch("../fixtures/keeps-what-it-was-written-with.json")).json();
  const W = (out.written = { want: fx.views.map((x) => x.view), got: [] });
  if (fx.views.map((x) => x.at).join() !== "0,1,2,4,8" || fx.commands.at(-1).cmd !== "play" || fx.commands.at(-1).at !== 4)
    throw new Error("keeps-what-it-was-written-with is not the fixture this phase was written for");
  await settle();
  const ids = {};
  for (const at of [0, 1, 2]) {
    for (const c of fx.commands.filter((c) => c.at === at)) {
      if (c.cmd === "key_down") hold(keyEl(c.key), (ids[c.key] = pid++));
      else if (c.cmd === "key_up") lift(keyEl(c.key), ids[c.key]);
      else if (c.cmd === "param") r.slide(c.slider, c.value);
      else throw new Error(`a command the page cannot send: ${c.cmd}`);
    }
    W.got.push(viewText(await settle(), 4));
  }
  const t = runSteps();
  r.send([5, 0, 0, 1]);
  await t.at(5);
  t.stop();
  W.got.push(viewText(t.views[0], 4), viewText(t.views[4], 4));
  r.send([5, 0, 0, 0]);
  await settle((v) => !(v[24] & 1), "stopped");
}

// The four patterns-* fixtures (format 5, ENGINE.md §21), each in a page of its own (a fresh
// engine, as a fixture's), played by touch: every key press a pointer on the drawn key, a
// slider by the page's slide(), an edit as the page sends it. Stopped, a run step's commands go
// in and the view that holds them all is read. Playing, a key lands within its run step, a few
// milliseconds after the view says the step began (nothing in these fixtures depends on which
// frame: a choice or a chain waits for the wrap); a stop is sent during the run step before
// its own, as fx-shuffle's edits are, so that no extra step is emitted; a view is the last of
// its run step. The trace is taken for each stretch of play and its ticks counted from that
// stretch's first event, as the fixture's from the same moment (the page cannot aim at a frame,
// and while stopped its clock runs on for as long as the page takes).
const pedit = (c) => edit(c) || { pattern: [27, 0, 0, c.value], chain: [28, 0, 0, c.value], copy: [29, 0, 0, c.value] }[c.cmd];
async function playFixture(name) {
  await r.ready;
  await r.ctx.resume();
  const fx = await (await fetch(`../fixtures/${name}.json`)).json();
  if (fx.format !== 5 || fx.mbpm !== 120000 || fx.seed !== 1) throw new Error(`${name} is not the format 5 fixture this phase was written for`);
  const F = { want: fx.views.map((x) => `${x.at} ${x.view}`), got: [], wantTrace: [], trace: [] };
  const viewAt = new Map(fx.views.map((x) => [x.at, x.view]));
  const ids = {};
  const send = (c) => {
    if (c.cmd === "key_down") hold(keyEl(c.key), (ids[c.key] = pid++));
    else if (c.cmd === "key_up") lift(keyEl(c.key), ids[c.key]);
    else if (c.cmd === "param") r.slide(c.slider, c.value);
    else {
      const e = pedit(c);
      if (!e) throw new Error(`a command the page cannot send: ${c.cmd}`);
      r.send(e);
    }
  };
  const isPlay = (c, v) => c.cmd === "play" && c.value === v;
  await settle();
  let t = null, base = 0, stretches = []; // [first run step, last run step + 1] of each stretch of play
  // A stretch of play ends at run step `end` (its last view and event are before it): its
  // views, and its trace from the mark.
  const endStretch = async (end, wait) => {
    if (wait) await t.at(end - base);
    t.stop();
    for (const [at] of viewAt) if (at >= base && at < end) F.got[at] = `${at} ${t.views[at - base] ? viewText(t.views[at - base], 5) : "no view"}`;
    r.post({ trace: true });
    const tr = traceLines((await next("trace")).trace);
    const first = tr.length ? +tr[0].split(" ")[0] : 0;
    F.trace.push(...tr.map((l) => l.replace(/^\d+/, (n) => String(+n - first))).filter((l) => +l.split(" ")[0] < (end - base) * fx.ticks_per_step));
    stretches.push([base, end]);
    t = null;
  };
  for (let at = 0; at < fx.steps; at++) {
    const cmds = fx.commands.filter((c) => c.at === at);
    if (!t) {
      // Stopped: a play edit starts a stretch (marked just before it, in the same queue).
      for (const c of cmds) {
        if (isPlay(c, 1)) { t = runSteps(); base = at; r.post({ mark: true }); }
        send(c);
      }
      if (!t && viewAt.has(at)) F.got[at] = `${at} ${viewText(await settle(), 5)}`;
      continue;
    }
    if (!cmds.length) continue;
    if (cmds.some((c) => isPlay(c, 0))) {
      // A stop: sent during run step at − 1, so the stretch ends there.
      await t.at(at - 1 - base);
      for (const c of cmds) send(c);
      await settle((v) => !(v[24] & 1), "stopped");
      await endStretch(at, false);
      if (viewAt.has(at)) F.got[at] = `${at} ${viewText(await settle(), 5)}`;
      continue;
    }
    await t.at(at - base);
    for (const c of cmds) send(c);
  }
  if (t) await endStretch(fx.steps, true);
  r.send([5, 0, 0, 0]);
  await settle((v) => !(v[24] & 1), "stopped");
  // The fixture's expect, cut the same way: each stretch's events, counted from its first.
  for (const [a, b] of stretches) {
    const tr = fx.expect.filter((l) => { const k = +l.split(" ")[0]; return k >= a * fx.ticks_per_step && k < b * fx.ticks_per_step; });
    const first = tr.length ? +tr[0].split(" ")[0] : 0;
    F.wantTrace.push(...tr.map((l) => l.replace(/^\d+/, (n) => String(+n - first))));
  }
  F.got = F.got.filter((x) => x !== undefined);
  return F;
}
const phasePatterns = (name, key) => async () => { out[key] = await playFixture(name); };

// The flicker (§21): the drawings recorded while pattern was down, from `since`, cut into runs
// of the active key's field lit or dark. Each run between the first and the last lasts from its
// first drawing to the next run's first: a quarter of a second, give or take a frame.
function flicker(since, until = Infinity) {
  const s = flicks.filter(([t]) => t >= since && t <= until), runs = [];
  for (const [t, lit, active, used] of s) {
    if (!runs.length || runs.at(-1).lit !== lit || runs.at(-1).active !== active) runs.push({ t, lit, active, used });
  }
  const inner = runs.slice(1, -1).map((x, i) => Math.round(runs[i + 2].t - x.t));
  return { drawings: s.length, first: runs[0]?.lit, active: [...new Set(runs.map((x) => x.active))], used: [...new Set(runs.map((x) => x.used))], inner };
}

// D135 by a mouse and by the computer keys, the lights while pattern is down, playing and
// stopped, and the save throttle (§21, "Saving while playing"). What each gesture sent is read
// from the page's own messages, and what it did from the view.
async function phaseShift() {
  await r.ready;
  await r.ctx.resume();
  const D = (out.pshift = {});
  let log = [];
  r.onPost = (m) => { if (m.cmds) log.push(...m.cmds.map((c) => c.join(" "))); };
  const take = () => { const l = log; log = []; return l; };
  const lit = (i) => r.fkeys[i].classList.contains("latched");
  const chainOf = (v) => nibbles(v, 72, v[66]);
  let v = await settle();
  take();
  // 1. Chain by a mouse: Shift+click pattern; Shift+click key 2 and key 3 keep it latched; a
  // plain click on key 1 ends it: the chain 2 3 1 (stopped, it is the chain at once).
  shiftClick(r.fkeys[1]);
  v = await settle((x) => u32(x, 8) === 1 << 17, "pattern latched");
  shiftClick(r.keys[1]);
  v = await settle((x) => chainOf(x) === "1", "Shift+click key 2 chose pattern 2");
  await frames2();
  D.mouse = { afterFirst: { held: u32(v, 8), lit: lit(1), latched: [...r.latched] } };
  shiftClick(r.keys[2]);
  v = await settle((x) => chainOf(x) === "12", "Shift+click key 3 chained pattern 3");
  await frames2();
  D.mouse.afterSecond = { held: u32(v, 8), lit: lit(1) };
  click(r.keys[0]);
  v = await settle((x) => chainOf(x) === "120" && u32(x, 8) === 0, "a plain click on key 1 ended the chain");
  await frames2();
  D.mouse.end = { chain: chainOf(v), pat: v[64], held: u32(v, 8), lit: lit(1), latched: [...r.latched], sent: take() };
  // 2. The same by the computer keys: Shift+8, Shift+3 (key 3), then 4 (key 4) plain.
  key("keydown", "Digit8", { shiftKey: true }); key("keyup", "Digit8", { shiftKey: true });
  await settle((x) => u32(x, 8) === 1 << 17, "Shift+8 latched pattern");
  key("keydown", "Digit3", { shiftKey: true }); key("keyup", "Digit3", { shiftKey: true });
  v = await settle((x) => chainOf(x) === "2", "Shift+3 chose pattern 3");
  D.keys = { afterFirst: { held: u32(v, 8), latched: [...r.latched] } };
  key("keydown", "Digit4"); key("keyup", "Digit4");
  v = await settle((x) => chainOf(x) === "23" && u32(x, 8) === 0, "4 chained pattern 4 and ended it");
  await frames2();
  D.keys.end = { chain: chainOf(v), held: u32(v, 8), latched: [...r.latched] };
  // 3. Copy by a mouse: a step in the active pattern (3) by an edit, write off. Shift+click
  // write (write mode flips on, write held), Shift+click pattern, click key 6: pattern 3 copied
  // to slot 6, write mode back to off, both latches gone.
  r.send([16, 0, 0, 1]);
  v = await settle((x) => u16(x, 68) === 1 << 2 && !(x[24] & 2), "a step in pattern 3, write off");
  take();
  shiftClick(r.fkeys[6]);
  v = await settle((x) => u32(x, 8) === 1 << 22, "write latched");
  await frames2();
  D.copy = { writeLatched: { held: u32(v, 8), write: (v[24] >> 1) & 1, lit: lit(6) } };
  shiftClick(r.fkeys[1]);
  await settle((x) => u32(x, 8) === (1 << 22 | 1 << 17), "write and pattern latched");
  click(r.keys[5]);
  v = await settle((x) => u32(x, 8) === 0, "the copy let both go");
  await frames2();
  D.copy.end = { used: u16(v, 68), pat: v[64], write: (v[24] >> 1) & 1, latched: [...r.latched], lit: [lit(1), lit(6)], sent: take() };
  // 4. Copy by the computer keys, pattern first: Shift+8, Shift+p, e (key 7): slot 7.
  key("keydown", "Digit8", { shiftKey: true }); key("keyup", "Digit8", { shiftKey: true });
  key("keydown", "KeyP", { shiftKey: true }); key("keyup", "KeyP", { shiftKey: true });
  await settle((x) => u32(x, 8) === (1 << 22 | 1 << 17), "Shift+8 and Shift+p latched");
  key("keydown", "KeyE"); key("keyup", "KeyE");
  v = await settle((x) => u32(x, 8) === 0, "e copied and let go");
  await frames2();
  D.copyKeys = { used: u16(v, 68), pat: v[64], write: (v[24] >> 1) & 1, latched: [...r.latched] };
  // 5. Clear by a mouse: Shift+click function, click pattern: the active pattern (3) only.
  shiftClick(r.fkeys[3]);
  click(r.fkeys[1]);
  v = await settle((x) => u32(x, 8) === 0 && !(u16(x, 68) & 1 << 2), "function and pattern cleared pattern 3");
  D.clear = { used: u16(v, 68), pat: v[64] };
  // 6. esc lets a latched pattern go.
  shiftClick(r.fkeys[1]);
  await settle((x) => u32(x, 8) === 1 << 17, "pattern latched");
  key("keydown", "Escape", { key: "Escape" });
  v = await settle((x) => u32(x, 8) === 0, "esc let pattern go");
  await frames2();
  D.esc = { held: u32(v, 8), latched: [...r.latched] };
  // 7. The flicker, stopped: pattern 3 is active and empty (⚠ open (empty flicker): it flickers).
  // Latched by Shift+click, about 1.4 s, then let go: the field is the view's again.
  let since = performance.now();
  shiftClick(r.fkeys[1]);
  await sleep(1400);
  shiftClick(r.fkeys[1]);
  v = await settle((x) => u32(x, 8) === 0, "pattern let go");
  await frames2();
  D.flickerEmpty = flicker(since);
  D.afterFlicker = { on: r.keys[v[64]].classList.contains("on"), field: (u16(v, 12) >> v[64]) & 1 };
  // 8. Playing, pattern held by touch: choose pattern 6 (the copy, used), sound 1 on every
  // step and an effect on every step of its lane, so a frame, cores and a twill would all show.
  // Held about 1.6 s: none of them is drawn, and key 6 flickers.
  const ph = pid++;
  hold(r.fkeys[1], ph); tap(r.keys[5], pid++); lift(r.fkeys[1], ph);
  await settle((x) => x[64] === 5, "pattern 6 chosen");
  r.send(...Array.from({ length: 16 }, (_, s) => [16, 0, s, 1]), ...Array.from({ length: 16 }, (_, s) => [23, s, 0, 3]));
  tap(r.fkeys[5], pid++);
  await settle((x) => x[24] & 1 && x[24] & 8 && x[55] === 3, "playing, effect 3 heard");
  const before = { ...out.pheld };
  since = performance.now();
  const hp = pid++;
  hold(r.fkeys[1], hp);
  await settle((x) => u32(x, 8) === 1 << 17, "pattern held while playing");
  await sleep(1600);
  const until = performance.now();
  lift(r.fkeys[1], hp);
  await settle((x) => u32(x, 8) === 0, "pattern let go");
  D.flickerPlaying = flicker(since, until);
  D.hidden = Object.fromEntries(Object.keys(before).map((k) => [k, out.pheld[k] - before[k]]));
  // 9. A chain shown and a next chain waiting on the display while playing: pattern held,
  // keys 7 and 6: "→ 7" until the end of the bar, then "1/2", "2/2".
  const hc = pid++;
  hold(r.fkeys[1], hc); tap(r.keys[6], pid++); tap(r.keys[5], pid++); lift(r.fkeys[1], hc);
  v = await settle((x) => x[67] === 2, "a next chain waiting");
  await frames2(); // the page draws the view on its next frame
  D.waiting = document.getElementById("patline").textContent;
  v = await settle((x) => x[66] === 2 && x[67] === 0, "the chain landed", 6000);
  await frames2();
  D.landed = document.getElementById("patline").textContent;
  tap(r.fkeys[5], pid++);
  await settle((x) => !(x[24] & 1), "stopped");
  // 10. The save throttle (§21: at most four saves a second, and the last move always saved).
  // A real drag on slider A: about 150 pointer moves over 1.5 s, each a PARAM. The snapshots
  // the worklet posts are counted, with their arrival times, and the last must hold the drag's
  // last value (the sliders' A at byte 24 of version 5).
  take();
  const snaps = [];
  const watch = (d) => { if (d.snapshot) snaps.push([performance.now(), new DataView(d.snapshot).getUint16(24, true)]); };
  r.listeners.push(watch);
  const el = r.sliders[0], box = el.querySelector(".tr").getBoundingClientRect();
  const pm = (type, y) => el.dispatchEvent(new PointerEvent(type, { pointerId: 7, bubbles: true, cancelable: true, isPrimary: true, pointerType: "touch", clientX: box.left + 3, clientY: y }));
  const y0 = box.top + box.height / 2;
  await sleep(400); // a quarter second clear of the last save
  pm("pointerdown", y0);
  const began = performance.now();
  for (let i = 1; i <= 150; i++) { pm("pointermove", y0 - i * 0.4); await sleep(10); }
  pm("pointerup", y0);
  const params = take().filter((c) => c.startsWith("34 0 "));
  const last = +params.at(-1).split(" ")[3];
  const dragMs = performance.now() - began;
  await settle((x) => u16(x, 20) === last, "the drag's last value in the view");
  await sleep(600);
  r.listeners.splice(r.listeners.indexOf(watch), 1);
  const during = snaps.filter(([t]) => t >= began);
  D.throttle = {
    params: params.length, dragMs: Math.round(dragMs), saves: during.length, last, lastSaved: during.at(-1)?.[1],
    gaps: during.slice(1).map(([t], i) => Math.round(t - during[i][0])),
  };
  r.onPost = null;
}

// A beat saved before ABI 16 (§21: "a beat saved before this plays as it did"): the page's
// storage holds a version 4 snapshot, base64, as the ABI 15 page kept it: straight-sixteenths'
// from fixtures/snapshots-v4.txt, written by the ABI 15 core before version 5 was built. Put
// there before this page loaded (PREP below). The page loads it (result 0); the play key plays
// two bars of straight-sixteenths' trace, line for line; it is pattern 1, the chain pattern 1
// alone; and the page then saves it as version 5, one pattern written (1 640 bytes).
const OLD_NAME = "straight-sixteenths";
async function oldBeat() {
  const line = (await (await fetch("../fixtures/snapshots-v4.txt")).text()).split("\n").find((l) => l.startsWith(`${OLD_NAME} `));
  const bytes = Uint8Array.from(line.split(" ")[1].match(/../g), (h) => parseInt(h, 16));
  return btoa(String.fromCharCode(...bytes));
}
async function phaseOld() {
  await r.ready;
  await r.ctx.resume();
  const O = (out.old = {});
  for (let i = 0; i < 800 && r.latest.loaded === null; i++) await sleep(10);
  O.loaded = r.latest.loaded;
  let kept = null;
  try { kept = localStorage.getItem("pocket-instruments/rhythm/beat"); } catch {}
  O.keptVersion = kept ? new DataView(Uint8Array.from(atob(kept), (c) => c.charCodeAt(0)).buffer).getUint16(4, true) : null;
  O.keptSize = kept ? atob(kept).length : null;
  let v = await settle();
  O.view = { pat: v[64], chain: nibbles(v, 72, v[66]), used: u16(v, 68), len: v[28] };
  r.post({ mark: true });
  tap(r.fkeys[5], pid++); // play: RESET then PLAY 1 (§13), so ticks start at 0
  await settle((x) => x[24] & 1 && u32(x, 0) >= 32 * 96 + 48, "two bars played", 15000);
  r.post({ trace: true });
  O.trace = traceLines((await next("trace")).trace).filter((l) => +l.split(" ")[0] < 32 * 96);
  tap(r.fkeys[5], pid++);
  await settle((x) => !(x[24] & 1), "stopped");
  const saved = next("snapshot");
  r.post({ save: true });
  const s = new DataView((await saved).snapshot);
  O.saved = { version: s.getUint16(4, true), size: s.byteLength, written: s.getUint16(92, true) };
}
// The beat file (docs/BEAT.md; issue 141's last box), through the page's own buttons and its
// file input (a file put there as a picker would). The crate's messages are worked out here
// by this file's own instance of pi_beat.wasm, sharing nothing with the page's. A refused file
// and a newer one change nothing and show the crate's message; "Keep this one" changes
// nothing; the worked example opened plays its fixture's views and trace (a reset first: the
// fixture's swing is by the grid step's parity, as fx-shuffle's); saved, it is the file byte for
// byte; a file with `own 9` says so on opening.
const enc = new TextEncoder();
async function myBeat() {
  const pib = (await WebAssembly.instantiate(await (await fetch("pi_beat.wasm")).arrayBuffer(), {})).instance.exports;
  const base = pib.memory.grow(2) * 65536, m = base + 98304 + 22100;
  return (text) => {
    new Uint8Array(pib.memory.buffer, base, text.length).set(text);
    const n = pib.pib_read(base, text.length, base + 98304, 22100, m + 192, m, 192);
    const mem = new Uint8Array(pib.memory.buffer);
    return n < 0 ? new TextDecoder().decode(mem.subarray(m, mem.indexOf(0, m))) : null;
  };
}
function pick(name, bytes) {
  const dt = new DataTransfer();
  dt.items.add(new File([bytes], name, { type: "text/plain" }));
  const input = document.getElementById("beatFile");
  input.files = dt.files;
  input.dispatchEvent(new Event("change"));
}
const beatMsg = () => document.getElementById("beatMsg").textContent;
async function said(pred, what) {
  for (let i = 0; i < 500; i++) { if (pred(beatMsg())) return beatMsg(); await sleep(10); }
  throw new Error(`timed out waiting for ${what}: "${beatMsg()}"`);
}
async function asked() {
  for (let i = 0; i < 500 && document.getElementById("beatAsk").hidden; i++) await sleep(10);
  return !document.getElementById("beatAsk").hidden;
}
async function phaseBeat() {
  await r.ready;
  await r.ctx.resume();
  const B = (out.beat = {});
  const refuse = await myBeat();
  const file = new Uint8Array(await (await fetch("../fixtures/beat-four-and-a-fill.beat")).arrayBuffer());
  const fx = await (await fetch("../fixtures/beat-four-and-a-fill.json")).json();
  const before = viewText(await settle(), 5);
  // A refused file, and a newer one: the crate's message, and nothing changed.
  for (const [key, name, text] of [["bad", "bad.beat", "pocket-instruments beat 1\ntempo 999\n"], ["newer", "newer.beat", "pocket-instruments beat 2\n"]]) {
    const msg = refuse(enc.encode(text));
    document.getElementById("beatMsg").textContent = "";
    pick(name, enc.encode(text));
    B[key] = { want: `${name} was not opened: ${msg}`, got: await said((t) => t !== "", `the message for ${name}`), ask: !document.getElementById("beatAsk").hidden };
    await sleep(300);
    B[key].unchanged = viewText(await settle(), 5) === before && r.latest.loaded === null;
  }
  // Asked, and kept: nothing changes.
  pick("beat-four-and-a-fill.beat", file);
  B.ask = { shown: await asked(), text: document.getElementById("beatAskText").textContent };
  document.getElementById("beatNo").click();
  B.kept = { said: await said((t) => t.startsWith("Kept"), "kept"), hidden: document.getElementById("beatAsk").hidden };
  await sleep(300);
  B.kept.unchanged = viewText(await settle(), 5) === before && r.latest.loaded === null;
  // Opened.
  pick("beat-four-and-a-fill.beat", file);
  await asked();
  document.getElementById("beatYes").click();
  B.opened = await said((t) => t.startsWith("Opened") || t.includes("not opened"), "opened");
  B.loaded = r.latest.loaded;
  await settle((x) => u32(x, 4) === 124000, "the beat's tempo");
  // Saved: the file again, byte for byte.
  let saved = null;
  r.onSave = (name, text) => { saved = { name, text }; };
  document.getElementById("beatSave").click();
  B.saveSaid = await said((t) => t.startsWith("Saved"), "saved");
  B.saved = { name: saved?.name, same: !!saved && saved.text.length === file.length && saved.text.every((x, i) => x === file[i]), length: saved?.text.length, want: file.length };
  if (saved && !B.saved.same) B.saved.text = new TextDecoder().decode(saved.text);
  // Played: the fixture's views and trace.
  const t = runSteps();
  r.post({ mark: true });
  r.send([2, 0, 0, 0], [5, 0, 0, 1]);
  await t.at(fx.steps);
  t.stop();
  r.post({ trace: true });
  const tr = traceLines((await next("trace")).trace);
  r.send([5, 0, 0, 0]);
  await settle((x) => !(x[24] & 1), "stopped");
  const first = tr.length ? +tr[0].split(" ")[0] : 0;
  B.trace = tr.map((l) => l.replace(/^\d+/, (n) => String(+n - first))).filter((l) => +l.split(" ")[0] < fx.steps * fx.ticks_per_step);
  B.wantTrace = fx.expect;
  B.views = fx.views.map((x) => `${x.at} ${t.views[x.at] ? viewText(t.views[x.at], 5) : "no view"}`);
  B.wantViews = fx.views.map((x) => `${x.at} ${x.view}`);
  // `own 9`: the starter pack's sample on key 9, and the page says so.
  const ownText = new TextDecoder().decode(file).replace("sound 9 sample dry\n", "sound 9 sample dry\nown 9\n");
  pick("own.beat", enc.encode(ownText));
  await asked();
  document.getElementById("beatYes").click();
  B.own = await said((t) => (t.startsWith("Opened") && t !== B.opened) || t.includes("not opened"), "the own notice");
}

const PREP = { old: async () => { localStorage.setItem("pocket-instruments/rhythm/beat", await oldBeat()); } };

// D78 by a mouse, fx + key by the computer keys, and D69's switch. What each gesture sent is
// read from the page's own messages (rhythm.onPost), as "type a b value".
async function phaseFxLatch() {
  await r.ready;
  await r.ctx.resume();
  const X = (out.fxlatch = {});
  let log = [];
  r.onPost = (m) => { if (m.cmds) log.push(...m.cmds.map((c) => c.join(" "))); };
  const take = () => { const l = log; log = []; return l; };
  const latchedLit = (i) => r.fkeys[i].classList.contains("latched");
  // A beat to play over: sound 1 on every step. Play by the key.
  r.send(...Array.from({ length: 16 }, (_, s) => [16, 0, s, 1]));
  tap(r.fkeys[5], pid++);
  await settle((x) => x[24] & 1 && x[24] & 8, "playing");
  // fx + key by the computer keys: i, then d (key 11). Effect 11 is heard while both are down.
  take();
  kb("keydown", 20);
  kb("keydown", 10);
  let v = await settle((x) => x[55] === 11, "fx and key 11 by the computer keys");
  X.keys = { held: u32(v, 8), heard: v[55], sent: take() };
  kb("keyup", 10);
  kb("keyup", 20);
  await settle((x) => u32(x, 8) === 0, "the computer keys let go");
  X.keys.after = take();
  // An empty lane to start D78 from.
  r.send(...Array.from({ length: 16 }, (_, s) => [23, s, 0, 0]));
  await settle((x) => u16(x, 56) === 0, "the lane emptied");
  take();
  // D78 item 1: Ctrl+click fx latches it, lit, and sends nothing: fx alone would empty steps.
  shiftClick(r.fkeys[4]);
  await frames2();
  v = await settle();
  X.latched = { lit: latchedLit(4), held: u32(v, 8), sent: take() };
  // Item 2: a click on key 2 sends fx down, then the key; effect 2 is heard. Held for
  // about three and a half steps, so the steps that arrive take it.
  mouse(r.keys[1], "pointerdown", 0);
  v = await settle((x) => x[55] === 2, "a click with fx latched punched effect 2");
  X.click1 = { held: u32(v, 8), sent: take() };
  await sleep(450);
  mouse(r.keys[1], "pointerup", 0);
  v = await settle((x) => u32(x, 8) === 0, "the click let go");
  await frames2();
  X.up1 = { lane: u16(v, 56), lit: latchedLit(4), sent: take() };
  // Item 3: over a bar and a bit with the latch on and nothing clicked, the lane replays
  // (effect 2 heard on its steps) and does not change.
  const heard = new Set(), lanes = new Set();
  const watch = (d) => { if (d.view) { const x = new Uint8Array(d.view); heard.add(x[55]); lanes.add(u16(x, 56)); } };
  r.listeners.push(watch);
  await sleep(2300);
  r.listeners.splice(r.listeners.indexOf(watch), 1);
  X.bar = { heard: [...heard].sort((a, b) => a - b), lanes: [...lanes], sent: take() };
  // The latch did not go after the first click: a second click punches effect 3.
  mouse(r.keys[2], "pointerdown", 0);
  v = await settle((x) => x[55] === 3, "the second click punched effect 3");
  X.click2 = { held: u32(v, 8), sent: take() };
  mouse(r.keys[2], "pointerup", 0);
  await settle((x) => u32(x, 8) === 0, "the second click let go");
  X.up2 = { sent: take() };
  // Item 5: Ctrl+click sound, click key 5: chosen, sound's latch gone, fx's still on.
  shiftClick(r.fkeys[0]);
  click(r.keys[4]);
  v = await settle((x) => x[26] === 4 && u32(x, 8) === 0, "sound 5 chosen with fx latched");
  await frames2();
  X.sound = { sel: v[26], soundLit: latchedLit(0), fxLit: latchedLit(4) };
  take();
  // Ctrl+click fx again: off, and it sends nothing (fx was not down). A click on key 2 is
  // then a plain press: only the key goes down and up.
  shiftClick(r.fkeys[4]);
  await frames2();
  X.off = { lit: latchedLit(4), sent: take() };
  click(r.keys[1]);
  await settle();
  X.off.click = take();
  tap(r.fkeys[5], pid++);
  await settle((x) => !(x[24] & 1), "stopped");
  take();
  // D69: the sheet's switch takes sound 4 out of the effects and puts it back (FXSOUND).
  r.sounds[3].fx.click();
  v = await settle((x) => u16(x, 58) === 0xfff7, "sound 4 out of the effects");
  await frames2();
  X.through = { off: u16(v, 58), sentOff: take(), lit: r.sounds[3].fx.classList.contains("through"), pressed: r.sounds[3].fx.getAttribute("aria-pressed") };
  r.sounds[3].fx.click();
  v = await settle((x) => u16(x, 58) === 0xffff, "sound 4 back through the effects");
  X.through.on = u16(v, 58);
  X.through.sentOn = take();
  r.onPost = null;
}

// The weave moving (ENGINE.md §16; design/DESIGN.md, "The animation"). Two sounds on every
// step, played: the shuttle travels (several drawings a step) and the newest rows flash white.
// Then with reduced motion asked for: no white, and one drawing a step. Then stopped: nothing
// is painted. What each drawing shows is held to pi_weave by cloth(), as in every phase.
async function phaseStill() {
  await r.ready;
  await r.ctx.resume();
  const S = (out.still = {});
  const take = () => { const t = seen; seen = tally(); return { drawings: t.drawings, whites: t.whites, travel: t.travel }; };
  r.send(...Array.from({ length: 16 }, (_, s) => [16, 0, s, 1]), ...Array.from({ length: 16 }, (_, s) => [16, 1, s, 1]));
  await settle();
  take();
  tap(r.fkeys[5], pid++);
  await settle((v) => v[24] & 1 && u32(v, 48) >= 20, "twenty steps woven", 15000);
  S.moving = take();
  askStill = true;
  r.reduced = () => true;
  await settle((v) => u32(v, 48) >= 22, "the page asked for reduced motion", 15000);
  take(); // the step the change fell in
  await settle((v) => u32(v, 48) >= 42, "twenty more steps woven", 15000);
  S.reduced = take();
  tap(r.fkeys[5], pid++);
  await settle((v) => !(v[24] & 1), "stopped");
  await frames2();
  await sleep(100);
  const paints = r.paints;
  await sleep(600);
  S.stoppedPaints = r.paints - paints;
}

const PHASES = { 1: [phase1, "reload"], reload: [phaseReload, "bpm"], bpm: [phaseBpm, "latch"], latch: [phaseLatch, "keep"],
  keep: [phaseKeep, "punch"], punch: [phasePunch, "shuffle"], shuffle: [phaseShuffle, "written"], written: [phaseWritten, "fxlatch"], fxlatch: [phaseFxLatch, "keys"], keys: [phaseKeys, "still"], still: [phaseStill, "pchoose"],
  pchoose: [phasePatterns("patterns-choose-and-play", "pchoose"), "pchain"], pchain: [phasePatterns("patterns-chain", "pchain"), "pcopy"],
  pcopy: [phasePatterns("patterns-copy-and-clear", "pcopy"), "pchange"], pchange: [phasePatterns("patterns-change-playing", "pchange"), "pshift"],
  pshift: [phaseShift, "old"], old: [phaseOld, "beat"], beat: [phaseBeat, null] };
const [run, then] = PHASES[phase] || [async () => { throw new Error(`no phase ${phase}`); }, null];
try {
  await run();
  await sleep(100); // the last drawing's check
} catch (e) {
  out.failed.push(String(e));
}
out.done = !then || out.failed.length > 0;
await fetch("/result", { method: "POST", body: JSON.stringify(out) });
if (!out.done) {
  try { await PREP[then]?.(); } catch (e) { console.error(e); }
  location.search = `?auto=${then}`;
}

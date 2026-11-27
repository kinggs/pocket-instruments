// The phone page's wiring (web/rhythm.html, Session 12): open it with ?auto in a real browser,
// headless, let web/play-auto.js press its keys, and judge what each phase reports.
//
//   node web/play.mjs <node report> [chromium] [firefox]     exits 0 only if every browser passed
//
// Served and launched as web/listen.mjs does: no Playwright, a throwaway profile each, a
// browser that is not installed is skipped, none at all is exit 2. The page runs three
// phases, reloading itself between them in the same profile.
//
// The bar, written before the page first ran (PLAN.md, Session 12):
//   - the first touch starts the audio thread and counts as a press (write comes on);
//   - straight-sixteenths' pattern entered by keys while stopped, then the play key: the
//     worklet's trace over the first two bars is that fixture's `expect`, line for line
//     (the play key resets, so ticks start at 0);
//   - keys-bpm-family's edits and keys pressed: the last view, as text, is the fixture's last;
//   - every drawing, read back off the face, equals the view it was drawn from (more than
//     fifty drawings, none different);
//   - two pointers held at once choose a sound: sound and key 6 both held, key 6 chosen;
//     and when both lift, nothing stays held;
//   - a reload brings the beat back: the page loads the kept snapshot (result 0) and saves
//     the same bytes;
//   - ~~Ctrl+click~~ Shift+click latches (D60, D61; added 2026-11-23; Shift since D113, issue
//     101: the gestures below are the same with Shift in place of Ctrl): Shift+click sound, click key 16:
//     sound held and lit as latched, then key 16 chosen and nothing held or lit; Ctrl+click
//     bpm, two slider moves leave it held, Ctrl+click it again: the tempo is B's 170 bpm,
//     not a preset; Ctrl+click function, click pattern: cleared; a right-click: nothing;
//     Ctrl+click on play: a plain press, it plays;
//   - a double-click puts a slider back (D63): the chosen sound's A to 32768; with bpm
//     held, A to swing 0 and B to 120 bpm;
//   - the sounds sheet (D60): a name fires its sound once and switches nothing; the switch
//     turns it to its sample and fires it once more;
//   - §20 (D75; added in Session 21): an old beat with two sounds' sliders apart loads and its
//     cap goes to the chosen sound's A; choosing the other sound sounds it at its own A
//     (the mark goes there) and the cap stays; the hand's move and a step written by key
//     keep what they were written with, and played, the mark shows both; every drawing's
//     cap and mark are held to the view (or to the hand where the phase moved it apart);
//   - the effects (added in Session 19; D69, D78, D81): every drawing's effect block, lane
//     ticks and "through fx" switches are held to the view's bytes 55 to 59, as the rest is;
//     fx-repeat-punch (format 3) played on the page, fx and key 11 by touch and fx alone by
//     the computer key: its four views, as text, are the fixture's, and its trace (ticks from
//     the first) is the fixture's `expect`; keeps-what-it-was-written-with (format 4) by
//     touch: its five views are the fixture's; fx-shuffle (format 3, effect 13; added in
//     Session 28) by its edits: its four views and its trace are the fixture's;
//   - fx + key 11 by the computer keys: fx down, then the key, heard 11, and up in reverse;
//   - D78 by a mouse (Shift since D113): Shift+click fx lights it and sends nothing; a click on key 2 sends fx
//     down then the key and is heard as effect 2; its release sends the key up then fx; a bar
//     with nothing clicked replays the lane (2 heard) and changes nothing; a second click
//     punches 3; Ctrl+click sound and a key still lets sound go, and fx stays latched;
//     Ctrl+click fx again lets it go and sends nothing, and a click is then a plain press;
//   - the sheet's "through fx" (D69): a tap sends FXSOUND 0 for that sound alone, the view
//     says so and the switch goes dark; a second tap sends 1;
//   - the weave (added in Session 14; ENGINE.md §16): in every phase, every drawing's cells,
//     read back off the canvas by colour, are pi_weave's for the view they were drawn from
//     (the checker's own instance of the .wasm), and the page painted if and only if the
//     cells changed; played with two sounds a step, the shuttle travels (four or more
//     drawings in one step) and a white flash is seen; with reduced motion asked for, no
//     white is drawn and a step is one drawing; stopped, nothing is painted. With
//     PI_REDUCED=1 the browsers are launched with reduced motion as their own setting, and
//     nothing may flash or travel in any phase;
//   - P6's browser half: the pattern entered by keys while it plays, the snapshot the page
//     saved is byte for byte one the wasm host loaded and got straight-sixteenths' trace from
//     (a "# straight-sixteenths ... snapshot <hex>" label in the Node report), and is what
//     the browser kept.
// Added for issue 101 (D113, D116, D119, D120), written before the page's code changed:
//   - D113: Ctrl+click on sound is a plain press now: held while the button is down, nothing
//     latched after; Shift+7 then q latches sound by the computer keys and chooses sound 5.
//   - D120, the core: in every drawing a grid key shows a core exactly while ENGINE.md §14's
//     host rule says (track n's `fires` moved within half a step at the view's tempo), in
//     every row, write on or off; it is fern exactly while `heard` is not 0 and fx sounds
//     names the sound; and a firing never whitens a face (white is `held` alone). Seen: a
//     core with write on, one with write off, and a fern core.
//   - D120, the twill: in every drawing the key `heard` − 1 shows the twill and no other key
//     does; seen at least once.
//   - D120, the source marks: in every drawing a key shows a dash exactly while sound is
//     held or latched and `sampled` names it; the page shows no ring and no disc (it sends no
//     ROUTE); seen at least once, with sound latched.
//   - D112 item 3: the fx key's band, the fx block and the lane's ticks are drawn in fern.
//   - D116, D119 item 3 (⚠ the step of one press is the build's reading, not ruled): space
//     sends the play key down and up; `,` and `.` send TEMPO one bpm down and up; `<` and `>`
//     SWING one down and up; `-` and `=` move the volume one down and up (the view says so);
//     `t` `g` `y` `h` send PARAM 4096 up or down from where the hand is; `m` brings the sounds
//     sheet into view; esc lets every latch go, fx's too.
//   - D116 item 3, D119 item 6: `?` opens the keys helper and esc closes it; a click on the
//     `?` mark in the display's top right corner opens it too.
// Added for issue 141 (ENGINE.md §21; D133, D135, D136):
//   - in every drawing, while pattern is held (the view's bit 17) or latched: every key's
//     field but the active pattern's, the stitch, the bar and pressed are the view's; no frame,
//     no core and no twill is drawn (counted where each would show); the display's line is
//     `pattern <64 + 1>`, then `<65 + 1>/<66>` while the chain is longer than one, then
//     `→ <80's low four bits + 1>` while a next chain waits;
//   - the four patterns-* fixtures played by touch, each in a fresh page: every view, as text
//     (format 5), is the fixture's, and each stretch of play's trace, its ticks counted from
//     the stretch's first event, is the fixture's `expect` cut the same way;
//   - D135 item 1: Shift+click pattern, Shift+click key 2, Shift+click key 3 (pattern stays
//     held and lit), a plain click on key 1: the chain 2 3 1 and nothing latched; the same by
//     Shift+8, Shift+3, 4: the chain 3 4;
//   - D135 item 2: Shift+click write latches it (held, lit) and flips write mode; Shift+click
//     pattern and click key 6: the active pattern is in slot 6, write mode is back off and
//     both latches are gone; Shift+8, Shift+p, e: slot 7 the same; Shift+click function, click
//     pattern clears the active pattern only; esc lets a latched pattern go;
//   - the flicker: the active key's field, sampled in every drawing, starts lit and turns every
//     250 ms (190 to 340 allowed, three turns or more), stopped on an empty pattern (⚠ open
//     (empty flicker)) and playing on a used one; let go, the field is the view's again;
//   - the display: `pattern 6 → 7` while a chain waits, `pattern 7 1/2` once it lands;
//   - the save throttle: a drag of 150 pointer moves over about 1.5 s sends a PARAM for each
//     and the worklet posts at most one snapshot a quarter second (no gap under 150 ms at the
//     page, at most the drag's quarter seconds plus two), the last holding the drag's last value;
//   - a version 4 snapshot (straight-sixteenths', fixtures/snapshots-v4.txt) in the page's
//     storage loads (result 0) as pattern 1, the chain pattern 1 alone; the play key plays
//     straight-sixteenths' two bars line for line; saved again it is version 5, 1 640 bytes.
// Each line fails once on a planted fault (PLAN.md, Session 12's Handover).
//
// SPDX-License-Identifier: MIT
import { createServer } from "node:http";
import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url)); // pocket-instruments/
const [reportPath, ...asked] = process.argv.slice(2);
if (!reportPath) {
  console.error("usage: play.mjs <node report> [chromium] [firefox]");
  process.exit(2);
}
const browsers = asked.length ? asked : ["chromium", "firefox"];
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".wasm": "application/wasm", ".s16": "application/octet-stream", ".txt": "text/plain" };
const report = readFileSync(reportPath, "utf8");
const sixteenths = JSON.parse(readFileSync(join(root, "fixtures/straight-sixteenths.json"), "utf8"));

let results = [], finished = null;
const server = createServer((req, res) => {
  if (req.method === "POST" && req.url === "/result") {
    let body = "";
    req.on("data", (d) => (body += d));
    req.on("end", () => {
      res.end("ok");
      const r = JSON.parse(body);
      results.push(r);
      if (r.done && finished) finished();
    });
    return;
  }
  const path = normalize(join(root, decodeURIComponent(req.url.split("?")[0])));
  if (!path.startsWith(root) || !existsSync(path) || !TYPES[extname(path)]) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { "content-type": TYPES[extname(path)], "cache-control": "no-store" }).end(readFileSync(path));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const url = `http://127.0.0.1:${server.address().port}/web/rhythm.html?auto=1`;

const REDUCED = !!process.env.PI_REDUCED; // the browser's own prefers-reduced-motion, on
function launch(name, profile) {
  if (name === "chromium")
    return spawn("chromium", ["--headless=new", "--autoplay-policy=no-user-gesture-required", "--no-first-run", "--disable-gpu",
      ...(REDUCED ? ["--force-prefers-reduced-motion"] : []), `--user-data-dir=${profile}`, url], { stdio: "ignore" });
  writeFileSync(join(profile, "user.js"),
    'user_pref("media.autoplay.default", 0);\nuser_pref("media.autoplay.blocking_policy", 0);\nuser_pref("datareporting.policy.dataSubmissionEnabled", false);\n' +
    (REDUCED ? 'user_pref("ui.prefersReducedMotion", 1);\n' : ""));
  return spawn("firefox", ["--headless", "--no-remote", "--profile", profile, url], { stdio: "ignore" });
}

function judge(phases) {
  const bad = [];
  const p = Object.fromEntries(phases.map((r) => [r.phase, r]));
  for (const r of phases) for (const f of r.failed) bad.push(`phase ${r.phase}: ${f}`);
  for (const name of ["1", "reload", "bpm", "latch", "keep", "punch", "shuffle", "written", "fxlatch", "keys", "still", "pchoose", "pchain", "pcopy", "pchange", "pshift", "old"]) if (!p[name]) bad.push(`phase ${name} never reported`);
  // Each phase reports under its own name: p.reload.reload, p.bpm.bpm. (A first draft read
  // p.bpm.last, and undefined equalled undefined: the bpm line passed with nothing checked.)
  const one = p["1"] || {}, reload = p.reload?.reload || {}, bpm = p.bpm?.bpm || {};
  if (!bpm.want) bad.push("keys-bpm-family: no view to hold");
  if (!(one.firstTouch?.state === "running" && one.firstTouch?.write)) bad.push(`first touch: ${JSON.stringify(one.firstTouch)}`);
  const want = sixteenths.expect;
  if (!one.trace || one.trace.join("\n") !== want.join("\n")) {
    const at = (one.trace || []).findIndex((l, i) => l !== want[i]);
    bad.push(`trace: ${one.trace?.length} lines, not the fixture's ${want.length}; first difference at line ${at < 0 ? Math.min(one.trace?.length ?? 0, want.length) + 1 : at + 1}: "${one.trace?.[at]}" for "${want[at]}"`);
  }
  if (bpm.last !== bpm.want) bad.push(`keys-bpm-family's last view:\n     got  ${bpm.last}\n     want ${bpm.want}`);
  const drawn = phases.reduce((n, r) => n + r.drawn.checked, 0);
  const drawnBad = phases.flatMap((r) => r.drawn.bad);
  if (drawn <= 50) bad.push(`only ${drawn} drawings checked`);
  for (const d of drawnBad) bad.push(`a drawing is not its view: ${d}`);
  // D120 (until then Session 20's white, D82 item 4): every drawing's core, twill, dash and
  // fern held to ENGINE.md §14's host rule; and each seen.
  for (const r of phases) for (const d of r.core?.bad || []) bad.push(`phase ${r.phase}: a light is not §14's: ${d}`);
  const pw = phases.reduce((t, r) => { for (const k of Object.keys(r.core || {})) if (k !== "bad") t[k] = (t[k] || 0) + r.core[k]; return t; }, {});
  for (const k of ["writeOn", "writeOff", "fern", "twill", "dash", "fernDrawn"]) if (!(pw[k] > 0)) bad.push(`D120: never seen: ${k} (${JSON.stringify(pw)})`);
  if (!(one.two?.sel === 5 && (one.two.held & 0x10020) === 0x10020 && one.two.after === 0)) bad.push(`two pointers: ${JSON.stringify(one.two)}`);
  if (!(reload.loaded === 0 && reload.hex && reload.hex === reload.before)) bad.push(`reload: loaded ${reload.loaded}; saved ${reload.hex?.slice(0, 32)}…, kept ${reload.before?.slice(0, 32)}…`);
  const L = p.latch?.latch || {};
  const latchOk = L.soundHeld === 1 << 16 && L.soundLit === true && L.chosen === 15 && L.afterLit === false &&
    L.bpmStillHeld === true && L.tempo === 170000 && L.fieldAfterClear === 0 &&
    L.afterRight?.held === 0 && L.afterRight?.lit === false && L.playAfterCtrl?.playing === 1;
  if (!latchOk) bad.push(`Shift+click latch: ${JSON.stringify({ ...L, nameFired: undefined, nameSampled: undefined, switchFired: undefined, switchSampled: undefined, switchLabel: undefined })}`);
  if (!(L.nameFired === 1 && L.nameSampled === 0 && L.switchFired === 2 && L.switchSampled === 4 && L.switchLabel === "sample"))
    bad.push(`sounds sheet: name fired ${L.nameFired} (sampled ${L.nameSampled}); switch fired ${L.switchFired} (sampled ${L.switchSampled}, "${L.switchLabel}")`);
  if (!(L.ctrl?.down === 1 << 16 && L.ctrl?.after === 0 && L.ctrl?.lit === false)) bad.push(`D113: Ctrl+click is not a plain press: ${JSON.stringify(L.ctrl)}`);
  if (!(L.dash === 1 << 2 && L.dashAfter === 0)) bad.push(`D112 item 4: the dash with sound latched: ${L.dash?.toString(16)}, after ${L.dashAfter?.toString(16)}`);
  if (!(L.shiftKeys?.held === 1 << 16 && L.shiftKeys?.sel === 4 && L.shiftKeys?.after === 0 && L.shiftKeys?.lit === false)) bad.push(`D113: Shift+7 q: ${JSON.stringify(L.shiftKeys)}`);
  if (!(L.dblA === 32768 && L.dblBpm?.swing === 0 && L.dblBpm?.tempo === 120000)) bad.push(`double-click: A ${L.dblA}; with bpm ${JSON.stringify(L.dblBpm)}`);
  const K = p.keep?.keep || {};
  // ABI 16 (§21): version 5, 276 bytes and 1 364 a pattern written (version 4's 1464 until then).
  const patternsWritten = [...Array(16).keys()].filter((p) => (K.written >> p) & 1).length;
  const keepOk = K.version === 5 && K.size === 276 + 1364 * patternsWritten && K.loadedCap === true && K.chosen?.a === 60000 && K.chosen?.mark === 60000 &&
    K.chosen?.capStayed === true && K.marks?.join() === "20000,60000" && K.kept?.keep === 0b10001 &&
    K.kept?.step1?.join() === "60000,32768" && K.kept?.step5?.join() === "20000,32768";
  if (!keepOk) bad.push(`§20, the hand and what a step keeps: ${JSON.stringify(K)}`);
  // Session 19: the effects.
  const P = p.punch?.punch || {};
  if (!P.want?.length || P.got?.join("\n") !== P.want.join("\n"))
    bad.push(`fx-repeat-punch's views:\n${(P.want || []).map((w, i) => `     got  ${P.got?.[i]}\n     want ${w}`).join("\n")}`);
  if (!P.wantTrace?.length || P.trace?.join("\n") !== P.wantTrace.join("\n")) {
    const at = (P.trace || []).findIndex((l, i) => l !== P.wantTrace?.[i]);
    bad.push(`fx-repeat-punch's trace: ${P.trace?.length} lines, not ${P.wantTrace?.length}; first difference at line ${at + 1}: "${P.trace?.[at]}" for "${P.wantTrace?.[at]}"`);
  }
  // Session 28: effect 13.
  const Q = p.shuffle?.shuffle || {};
  if (!Q.want?.length || Q.got?.join("\n") !== Q.want.join("\n"))
    bad.push(`fx-shuffle's views:\n${(Q.want || []).map((w, i) => `     got  ${Q.got?.[i]}\n     want ${w}`).join("\n")}`);
  if (!Q.wantTrace?.length || Q.trace?.join("\n") !== Q.wantTrace.join("\n")) {
    const at = (Q.trace || []).findIndex((l, i) => l !== Q.wantTrace?.[i]);
    bad.push(`fx-shuffle's trace: ${Q.trace?.length} lines, not ${Q.wantTrace?.length}; first difference at line ${at + 1}: "${Q.trace?.[at]}" for "${Q.wantTrace?.[at]}"`);
  }
  const W = p.written?.written || {};
  if (!W.want?.length || W.got?.join("\n") !== W.want.join("\n"))
    bad.push(`keeps-what-it-was-written-with's views:\n${(W.want || []).map((w, i) => `     got  ${W.got?.[i]}\n     want ${w}`).join("\n")}`);
  const X = p.fxlatch?.fxlatch || {};
  const FX = 1 << 20, sent = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  const lanes = X.up1?.lane ?? 0, written = [...Array(16).keys()].filter((s) => (lanes >> s) & 1).length;
  const fxChecks = {
    "fx + key 11 by the computer keys": X.keys?.held === (FX | 1 << 10) && X.keys?.heard === 11 &&
      sent(X.keys?.sent, ["32 20 0 0", "32 10 0 0"]) && sent(X.keys?.after, ["33 10 0 0", "33 20 0 0"]),
    "D78: Shift+click fx lights it and sends nothing": X.latched?.lit === true && X.latched?.held === 0 && sent(X.latched?.sent, []),
    "D78: a click sends fx down, then the key, and punches 2": X.click1?.held === (FX | 1 << 1) && sent(X.click1?.sent, ["32 20 0 0", "32 1 0 0"]),
    "D78: its release sends the key up, then fx; the latch stays; steps took 2": sent(X.up1?.sent, ["33 1 0 0", "33 20 0 0"]) && X.up1?.lit === true && written >= 2,
    "D78: a bar with nothing clicked replays the lane and changes nothing": X.bar?.heard?.join() === "0,2" && sent(X.bar?.lanes, [lanes]) && sent(X.bar?.sent, []),
    "D78: a second click punches 3": X.click2?.held === (FX | 1 << 2) && sent(X.click2?.sent, ["32 20 0 0", "32 2 0 0"]) && sent(X.up2?.sent, ["33 2 0 0", "33 20 0 0"]),
    "D78 item 5: sound's latch goes after a key, fx's stays": X.sound?.sel === 4 && X.sound?.soundLit === false && X.sound?.fxLit === true,
    "D78: Shift+click fx again lets it go; a click is then plain": X.off?.lit === false && sent(X.off?.sent, []) && sent(X.off?.click, ["32 1 0 0", "33 1 0 0"]),
    "D69: the through fx switch": X.through?.off === 0xfff7 && sent(X.through?.sentOff, ["24 3 0 0"]) && X.through?.lit === false &&
      X.through?.pressed === "false" && X.through?.on === 0xffff && sent(X.through?.sentOn, ["24 3 0 1"]),
  };
  for (const [name, ok] of Object.entries(fxChecks)) if (!ok) bad.push(`${name}: ${JSON.stringify(X)}`);
  // Issue 101: the one-key shortcuts and the helper (D116, D119 items 3 and 6).
  const KY = p.keys?.keys || {};
  const keyChecks = {
    "space: the play key down and up, twice": sent(KY.space?.sent, ["32 21 0 0", "33 21 0 0"]) && KY.space?.playing === 1 && KY.space?.stopped === true,
    ". and ,: TEMPO one bpm up and down": sent(KY.tempoUp, ["4 0 0 121000"]) && sent(KY.tempoDown, ["4 0 0 120000"]),
    "> and <: SWING one up and down": sent(KY.swingUp, ["18 0 0 1"]) && sent(KY.swingDown, ["18 0 0 0"]),
    "- and =: the volume one down and up, the tempo kept": KY.volume?.down === 15 && KY.volume?.up === 16 && KY.volume?.held === 0 && KY.volume?.tempo === 120000,
    "t g y h: PARAM 4096 from the hand": sent(KY.sliders, ["34 0 0 34096", "34 0 0 30000", "34 1 0 34096", "34 1 0 30000"]) && KY.capA === 30000,
    "m: the sounds sheet in view": KY.sheetTop !== undefined && KY.sheetTop < 200,
    "?: the helper opens; esc closes it; the ? mark in the corner opens it": KY.helper?.byKey === true && KY.helper?.escClosed === true &&
      KY.helper?.corner === true && KY.helper?.byMark === true && KY.helper?.escClosed2 === true,
    "esc: every latch lets go, fx's too": sent(KY.esc?.before, [16, 20]) && sent(KY.esc?.after, []) && KY.esc?.lit === false,
  };
  for (const [name, ok] of Object.entries(keyChecks)) if (!ok) bad.push(`${name}: ${JSON.stringify(KY)}`);
  // Session 14: the weave. Every drawing's cells held to pi_weave; then what moves.
  const woven = phases.reduce((n, r) => n + (r.weave?.checked ?? 0), 0);
  if (woven <= 50) bad.push(`only ${woven} drawings of the weave checked`);
  for (const r of phases) for (const d of r.weave?.bad || []) bad.push(`phase ${r.phase}: the weave is not pi_weave's: ${d}`);
  const S = p.still?.still || {}, prefers = phases.some((r) => r.weave?.prefers);
  if (prefers !== REDUCED) bad.push(`the browser's prefers-reduced-motion is ${prefers}, and this run asked for ${REDUCED}`);
  if (!(S.moving?.drawings > 20 && S.reduced?.drawings > 20)) bad.push(`the weave was not seen moving: ${JSON.stringify(S)}`);
  else {
    if (!prefers && !(S.moving.whites > 0)) bad.push(`two sounds a step and no white flash drawn: ${JSON.stringify(S.moving)}`);
    if (!prefers && !(S.moving.travel >= 4)) bad.push(`the shuttle did not travel inside a step: ${JSON.stringify(S.moving)}`);
    if (prefers && !(S.moving.whites === 0 && S.moving.travel === 1)) bad.push(`the browser asks for reduced motion and the weave moved: ${JSON.stringify(S.moving)}`);
    if (S.reduced.whites !== 0) bad.push(`reduced motion drew white rows: ${JSON.stringify(S.reduced)}`);
    if (S.reduced.travel !== 1) bad.push(`reduced motion and the weave moved inside a step: ${JSON.stringify(S.reduced)}`);
    if (S.stoppedPaints !== 0) bad.push(`stopped, the weave was painted ${S.stoppedPaints} times`);
  }
  // Issue 141 (ENGINE.md §21; D133, D135, D136), written before the page's code changed.
  // The four patterns-* fixtures by touch: their views, as text (format 5), and their traces.
  for (const [name, key] of [["patterns-choose-and-play", "pchoose"], ["patterns-chain", "pchain"], ["patterns-copy-and-clear", "pcopy"], ["patterns-change-playing", "pchange"]]) {
    const F = p[key]?.[key] || {};
    if (!F.want?.length || F.got?.join("\n") !== F.want.join("\n"))
      bad.push(`${name}'s views:\n${(F.want || []).map((w, i) => `     got  ${F.got?.[i]}\n     want ${w}`).join("\n")}`);
    if (!F.wantTrace?.length || F.trace?.join("\n") !== F.wantTrace.join("\n")) {
      const at = (F.trace || []).findIndex((l, i) => l !== F.wantTrace?.[i]);
      bad.push(`${name}'s trace: ${F.trace?.length} lines, not ${F.wantTrace?.length}; first difference at line ${at + 1}: "${F.trace?.[at]}" for "${F.wantTrace?.[at]}"`);
    }
  }
  // D135 by a mouse and the keys; the lights while pattern is down; the display; the throttle.
  const D = p.pshift?.pshift || {}, same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  const flickerOk = (F) => F?.first === 1 && F.inner?.length >= 3 && F.inner.every((d) => d >= 190 && d <= 340) && F.active?.length === 1;
  const T = D.throttle || {};
  const shiftChecks = {
    "D135 item 1: Shift+click on a grid key keeps a latched pattern": D.mouse?.afterFirst?.held === 1 << 17 && D.mouse.afterFirst.lit === true &&
      same(D.mouse.afterFirst.latched, [17]) && D.mouse.afterSecond?.held === 1 << 17 && D.mouse.afterSecond.lit === true,
    "D135 item 1: a plain click ends the chain, 2 3 1, pattern let go": D.mouse?.end?.chain === "120" && D.mouse.end.pat === 1 && D.mouse.end.held === 0 &&
      D.mouse.end.lit === false && same(D.mouse.end.latched, []),
    "D135 item 1 by the computer keys: Shift+8, Shift+3, 4": D.keys?.afterFirst?.held === 1 << 17 && same(D.keys.afterFirst.latched, [17]) &&
      D.keys?.end?.chain === "23" && D.keys.end.held === 0 && same(D.keys.end.latched, []),
    "D135 item 2: Shift+click write latches it and flips write mode": D.copy?.writeLatched?.held === 1 << 22 && D.copy.writeLatched.write === 1 && D.copy.writeLatched.lit === true,
    "D135 item 2: Shift+click write, pattern, click 6 copies; write mode back; both let go": D.copy?.end?.used === 0x24 && D.copy.end.pat === 2 &&
      D.copy.end.write === 0 && same(D.copy.end.latched, []) && same(D.copy.end.lit, [false, false]),
    "D135 item 2 by the computer keys: Shift+8, Shift+p, e copies to 7": D.copyKeys?.used === 0x64 && D.copyKeys.pat === 2 && D.copyKeys.write === 0 && same(D.copyKeys.latched, []),
    "clear by a mouse: Shift+click function, click pattern: the active pattern only": D.clear?.used === 0x60 && D.clear.pat === 2,
    "esc lets a latched pattern go": D.esc?.held === 0 && same(D.esc.latched, []),
    "§21: an empty active pattern flickers, stopped (⚠ open (empty flicker))": flickerOk(D.flickerEmpty) && same(D.flickerEmpty.used, [0]) &&
      D.afterFlicker?.on === !!D.afterFlicker?.field,
    "§21: playing, the active pattern flickers while pattern is held": flickerOk(D.flickerPlaying) && same(D.flickerPlaying.used, [1]),
    "§21: playing with pattern held, no frame, core or twill was drawn where each would show": D.hidden?.framesHidden > 10 && D.hidden?.coresHidden > 0 && D.hidden?.twillsHidden > 0,
    "§21: the display says the pattern waiting, then the chain's place": D.waiting === "pattern 6 → 7" && D.landed === "pattern 7 1/2",
    "§21: a slider drag of many changes saves at most four times a second, and its last value": T.params >= 100 && T.saves >= 2 &&
      T.saves <= Math.ceil(T.dragMs / 250) + 2 && T.gaps?.every((g) => g >= 150) && T.lastSaved === T.last,
  };
  for (const [name, ok] of Object.entries(shiftChecks)) if (!ok) bad.push(`${name}: ${JSON.stringify(D)}`);
  const PH = phases.reduce((t, r) => { for (const k of Object.keys(r.pheld || {})) t[k] = (t[k] || 0) + r.pheld[k]; return t; }, {});
  if (!(PH.chainShown > 0 && PH.nextShown > 0)) bad.push(`§21: the display never drew a chain's place or a waiting pattern: ${JSON.stringify(PH)}`);
  // A beat saved before this (§21): a version 4 snapshot in the page's storage loads and plays as it did.
  const O = p.old?.old || {};
  const oldOk = O.loaded === 0 && O.keptVersion === 4 && O.keptSize === 1464 && O.view?.pat === 0 && O.view?.chain === "0" &&
    O.saved?.version === 5 && O.saved?.size === 1640 && O.saved?.written === 1 && O.trace?.join("\n") === want.join("\n");
  if (!oldOk) bad.push(`a version 4 beat from the page's storage: ${JSON.stringify({ ...O, trace: O.trace?.length })}`);
  const p6 = one.p6 || {};
  const label =p6.hex && new RegExp(`^# straight-sixteenths .* snapshot ${p6.hex}$`, "m").test(report);
  if (!p6.playing) bad.push("P6: it was not playing while the keys went in");
  if (!label) bad.push(`P6: the page's snapshot is not one the wasm host ran straight-sixteenths from: ${p6.hex}`);
  if (p6.stored !== p6.hex) bad.push("P6: what the browser kept is not the snapshot the page was sent");
  const whites = phases.reduce((n, r) => n + (r.core ? 1 : 0), 0);
  return { bad, drawn, p6, pw, whites, woven, S, D };
}

let ran = 0, failed = 0;
for (const name of browsers) {
  if (spawnSync("sh", ["-c", `command -v ${name}`]).status !== 0) {
    console.log(`play ${name}: not installed, skipped`);
    continue;
  }
  ran++;
  results = [];
  const profile = mkdtempSync(join(tmpdir(), `pi-play-${name}-`));
  const done = new Promise((r) => (finished = r));
  const child = launch(name, profile);
  const ok = await Promise.race([done.then(() => true), new Promise((r) => setTimeout(() => r(false), 120000))]);
  finished = null;
  child.kill();
  await new Promise((r) => setTimeout(r, 500));
  rmSync(profile, { recursive: true, force: true });
  const { bad, drawn, p6, pw, whites, woven, S, D } = judge(results);
  if (!ok) bad.unshift("the page did not finish within two minutes");
  if (bad.length) failed++;
  if (process.env.PI_DEBUG) console.log(JSON.stringify(results.map((r) => ({ ...r, trace: undefined })), null, 1));
  const one = results.find((r) => r.phase === "1");
  console.log(`play ${name} at ${one?.rate} Hz: ${bad.length ? "FAILED" : "passed"}; ${results.length} phases, ${drawn} drawings checked; ` +
    `P6: ${p6.hex ? "snapshot " + p6.hex.slice(32, 48) + "…" : "no snapshot"}, audio ${p6.behindMs} ms behind the page's clock over ${p6.seconds} s; ` +
    `lights held to §14 in ${whites} phases (cores: ${pw.writeOn ?? 0} with write on, ${pw.writeOff ?? 0} off, ${pw.fern ?? 0} fern; twill ${pw.twill ?? 0}; dash ${pw.dash ?? 0}); ` +
    `the weave held to pi_weave in ${woven} drawings${REDUCED ? " (the browser asking for reduced motion)" : ""}: moving, ${S.moving?.whites} with a flash and ${S.moving?.travel} drawings in a step; ` +
    `reduced, ${S.reduced?.whites} and ${S.reduced?.travel}; stopped, ${S.stoppedPaints} paints; ` +
    `patterns: the flicker ${D?.flickerPlaying?.inner?.join("/") ?? "-"} ms, held while playing ${D?.hidden?.framesHidden ?? 0} drawings with no frame; ` +
    `a drag of ${D?.throttle?.params ?? 0} changes in ${D?.throttle?.dragMs ?? 0} ms saved ${D?.throttle?.saves ?? 0} times`);
  for (const b of bad) console.log(`  ${b}`);
}
server.close();
if (!ran) {
  console.log("play: no browser to run in");
  process.exit(2);
}
process.exit(failed ? 1 : 0);

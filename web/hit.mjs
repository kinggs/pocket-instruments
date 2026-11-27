// The phone page's keys answer where D139 puts them (issue 150): web/rhythm.html in headless
// Chromium as a phone held upright, 390 by 844 CSS px, and every target pressed by position.
//
//   node web/hit.mjs [out.png]      exits 0 only if every check passed; writes the screenshot if asked
//
// The bar, worked by hand from D139 item 7 and the page's own layout before this first ran:
//   - the edge is 8 px a side (table 2, hem 2, padding 4), so the grid starts at x 8 and, under
//     the display (124 px) and the body's 12 px gap, at y 144;
//   - a grid key is 92 px square, 2 px apart: key c, r (from 0) at (8 + 94c, 144 + 94r); its
//     numeral 30 px;
//   - document.elementFromPoint at each key's centre and 2 px inside each corner is that key; in
//     the middle of each 2 px gap between two keys, no key;
//   - the gaps the keys of 88 px left until D139 (x 104 to 106, 194 to 196 and 284 to 286; y 240
//     to 242, 330 to 332 and 420 to 422, the grid then at (16, 152)), at their middles: on a key
//     wherever a key of D139's is there. Found on the first run: the gap between the middle
//     columns (x 194 to 196, the screen's middle) and the one under the second row (y 330 to
//     332) are where they were, so 8 of the 24 middles are still gaps and 16 are on a key; a fact
//     of the ruled sizes, said, not hidden;
//   - a function key is 84 px wide in the top row and 62 in the bottom, its word 20 px; each
//     answers at its centre and 2 px inside its corners;
//   - a slider's column is 48 px; its cap 48 by 44 px, and a press on the cap's centre and 2 px
//     inside its corners lands in that slider.
// Chromium only (the Chrome DevTools Protocol, as web/shot.mjs). Served as web/shot.mjs does.
//
// SPDX-License-Identifier: MIT
import { createServer } from "node:http";
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".s16": "application/octet-stream", ".txt": "text/plain", ".beat": "text/plain" };
const server = createServer((req, res) => {
  const path = normalize(join(root, decodeURIComponent(req.url.split("?")[0])));
  if (!path.startsWith(root) || !existsSync(path) || !TYPES[extname(path)]) return res.writeHead(404).end();
  res.writeHead(200, { "content-type": TYPES[extname(path)], "cache-control": "no-store" }).end(readFileSync(path));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const profile = mkdtempSync(join(tmpdir(), "pi-hit-"));
const chrome = spawn("chromium", ["--headless=new", "--no-first-run", "--disable-gpu", "--remote-debugging-port=0",
  `--user-data-dir=${profile}`, "about:blank"], { stdio: ["ignore", "ignore", "pipe"] });
chrome.on("error", () => { console.log("hit: no chromium on the PATH"); process.exit(2); });
const wsUrl = await new Promise((res, rej) => {
  let err = "";
  chrome.stderr.on("data", (d) => { err += d; const m = err.match(/DevTools listening on (ws:\/\/\S+)/); if (m) res(m[1]); });
  setTimeout(() => rej(new Error("chromium did not start")), 15000);
});
const port = new URL(wsUrl).port;
const target = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find((t) => t.type === "page");
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0;
const pending = new Map();
ws.addEventListener("message", (m) => { const d = JSON.parse(m.data); if (pending.has(d.id)) { pending.get(d.id)(d); pending.delete(d.id); } });
const cdp = (method, params = {}) => new Promise((res) => { pending.set(++id, res); ws.send(JSON.stringify({ id, method, params })); });
const run = async (expr) => {
  const r = await cdp("Runtime.evaluate", { expression: `(async () => { ${expr} })()`, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails));
  return r.result?.result?.value;
};
await cdp("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
await cdp("Emulation.setTouchEmulationEnabled", { enabled: true });
await cdp("Page.enable");
await cdp("Page.navigate", { url: `http://127.0.0.1:${server.address().port}/web/rhythm.html` });
await new Promise((r) => setTimeout(r, 1500));

// Everything is measured inside the page and sent back; the bar is held here.
const m = await run(`
  await document.fonts.ready;
  const box = (el) => { const b = el.getBoundingClientRect(); return { x: b.left, y: b.top, w: b.width, h: b.height }; };
  const keys = [...document.querySelectorAll("#grid .k")], fkeys = [...document.querySelectorAll(".fkeys .f")];
  const sliders = [...document.querySelectorAll(".sl")];
  // What a press at (x, y) reaches: a grid key's number, a function key's word, a slider's name.
  const at = (x, y) => {
    const el = document.elementFromPoint(x, y);
    const k = el?.closest(".k"), f = el?.closest(".f"), s = el?.closest(".sl");
    return k ? "key " + (keys.indexOf(k) + 1) : f ? "fn " + fkeys.indexOf(f) : s ? s.id : "nothing";
  };
  return {
    width: innerWidth, height: innerHeight,
    keys: keys.map((el) => ({ ...box(el), font: parseFloat(getComputedStyle(el).fontSize) })),
    fkeys: fkeys.map((el) => ({ ...box(el), font: parseFloat(getComputedStyle(el).fontSize), word: el.textContent.trim() })),
    sliders: sliders.map((el) => ({ ...box(el), id: el.id, cap: box(el.querySelector(".cap")) })),
    at: (() => { window.__at = at; return true; })(),
  };`);
const at = (pts) => run(`return ${JSON.stringify(pts)}.map(([x, y]) => window.__at(x, y));`);

const bad = [];
let n = 0;
const near = (a, b) => Math.abs(a - b) < 0.01;
const hold = (what, ok) => { n++; if (!ok) bad.push(what); };
hold(`the viewport is 390 by 844, not ${m.width} by ${m.height}`, m.width === 390 && m.height === 844);
// The grid keys.
const KEY = 92, PITCH = 94, X0 = 8, Y0 = 144;
hold(`16 keys, not ${m.keys.length}`, m.keys.length === 16);
const points = [], want = [];
m.keys.forEach((k, i) => {
  const c = i % 4, r = Math.floor(i / 4), x = X0 + PITCH * c, y = Y0 + PITCH * r;
  hold(`key ${i + 1} at (${k.x}, ${k.y}) ${k.w} by ${k.h}, not (${x}, ${y}) ${KEY} by ${KEY}`, near(k.x, x) && near(k.y, y) && near(k.w, KEY) && near(k.h, KEY));
  hold(`key ${i + 1}'s numeral ${k.font} px, not 30`, k.font === 30);
  for (const [px, py] of [[x + KEY / 2, y + KEY / 2], [x + 2, y + 2], [x + KEY - 2, y + 2], [x + 2, y + KEY - 2], [x + KEY - 2, y + KEY - 2]]) {
    points.push([px, py]); want.push(`key ${i + 1}`);
  }
  if (c < 3) { points.push([x + KEY + 1, y + KEY / 2]); want.push("nothing"); }
  if (r < 3) { points.push([x + KEY / 2, y + KEY + 1]); want.push("nothing"); }
});
// The old gaps: keys of 88 px at a 90 px pitch from (16, 152), until D139.
const onKey = (x, y) => {
  for (let i = 0; i < 16; i++) {
    const kx = X0 + PITCH * (i % 4), ky = Y0 + PITCH * Math.floor(i / 4);
    if (x >= kx && x < kx + KEY && y >= ky && y < ky + KEY) return `key ${i + 1}`;
  }
  return "nothing";
};
const old = [];
for (let r = 0; r < 4; r++) for (let c = 0; c < 3; c++) old.push([16 + 88 + 1 + 90 * c, 152 + 44 + 90 * r]);
for (let r = 0; r < 3; r++) for (let c = 0; c < 4; c++) old.push([16 + 44 + 90 * c, 152 + 88 + 1 + 90 * r]);
for (const [x, y] of old) { points.push([x, y]); want.push(onKey(x, y)); }
const oldOnKey = old.filter(([x, y]) => onKey(x, y) !== "nothing").length;
// The function keys: 84 and 62 px wide, the word 20 px; centre and corners.
const fw = [84, 84, 84, 62, 62, 62, 62];
m.fkeys.forEach((f, i) => {
  hold(`${f.word} is ${f.w} px wide, not ${fw[i]}`, near(f.w, fw[i]));
  hold(`${f.word}'s word is ${f.font} px, not 20`, f.font === 20);
  for (const [px, py] of [[f.x + f.w / 2, f.y + f.h / 2], [f.x + 2, f.y + 2], [f.x + f.w - 2, f.y + 2], [f.x + 2, f.y + f.h - 2], [f.x + f.w - 2, f.y + f.h - 2]]) {
    points.push([px, py]); want.push(`fn ${i}`);
  }
});
// The sliders: a column 48 px, a cap 48 by 44; the cap's centre and corners land in its slider.
m.sliders.forEach((s) => {
  hold(`${s.id}'s column is ${s.w} px, not 48`, near(s.w, 48));
  hold(`${s.id}'s cap is ${s.cap.w} by ${s.cap.h} px, not 48 by 44`, near(s.cap.w, 48) && near(s.cap.h, 44));
  const c = s.cap;
  for (const [px, py] of [[c.x + c.w / 2, c.y + c.h / 2], [c.x + 2, c.y + 2], [c.x + c.w - 2, c.y + 2], [c.x + 2, c.y + c.h - 2], [c.x + c.w - 2, c.y + c.h - 2]]) {
    points.push([px, py]); want.push(s.id);
  }
});
const got = await at(points);
points.forEach(([x, y], i) => hold(`a press at (${x}, ${y}) reached ${got[i]}, not ${want[i]}`, got[i] === want[i]));
if (process.argv[2]) {
  const { result } = await cdp("Page.captureScreenshot", { format: "png" });
  writeFileSync(process.argv[2], Buffer.from(result.data, "base64"));
}
ws.close();
chrome.kill();
server.close();
await new Promise((r) => setTimeout(r, 300));
rmSync(profile, { recursive: true, force: true });
for (const b of bad) console.log(`  ${b}`);
console.log(`hit (Chromium, 390 by 844): ${n - bad.length} of ${n} checks; ${points.length} presses by position; ` +
  `${old.length} of the old gaps, ${oldOnKey} on a key now and ${old.length - oldOnKey} in a gap still (x 194 to 196 and y 330 to 332 are gaps in both)`);
process.exit(bad.length ? 1 : 0);

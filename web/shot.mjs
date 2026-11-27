// Pictures of the phone page for Kenny's eye (issue 101): a key's lights in colour and with
// the colour taken out, sound held with its marks, and the keys helper.
//
//   node web/shot.mjs <out dir>      writes page-*.png there (headless Chromium on the PATH)
//
// The page is driven by its own messages (the commands a fixture sends), then frozen: the
// audio is suspended and the page's clock stopped, so a core caught lit stays lit for the
// picture. Nothing here judges: make play does that. This only takes pictures.
//
// SPDX-License-Identifier: MIT
import { createServer } from "node:http";
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const outDir = process.argv[2];
if (!outDir) { console.error("usage: shot.mjs <out dir>"); process.exit(2); }
mkdirSync(outDir, { recursive: true });
const TYPES = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".s16": "application/octet-stream", ".txt": "text/plain" };
const server = createServer((req, res) => {
  const path = normalize(join(root, decodeURIComponent(req.url.split("?")[0])));
  if (!path.startsWith(root) || !existsSync(path) || !TYPES[extname(path)]) return res.writeHead(404).end();
  res.writeHead(200, { "content-type": TYPES[extname(path)], "cache-control": "no-store" }).end(readFileSync(path));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const profile = mkdtempSync(join(tmpdir(), "pi-shot-"));
const chrome = spawn("chromium", ["--headless=new", "--autoplay-policy=no-user-gesture-required", "--no-first-run", "--disable-gpu",
  "--remote-debugging-port=0", "--window-size=430,932", "--force-device-scale-factor=2", `--user-data-dir=${profile}`, "about:blank"],
  { stdio: ["ignore", "ignore", "pipe"] });
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
const shoot = async (name, grey = false) => {
  await run(`document.documentElement.style.filter = ${grey ? '"grayscale(1)"' : '""'}; await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));`);
  const { result } = await cdp("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(outDir, name), Buffer.from(result.data, "base64"));
  console.log(`wrote ${join(outDir, name)}`);
};
await cdp("Page.enable");
await cdp("Page.navigate", { url: `http://127.0.0.1:${server.address().port}/web/rhythm.html` });
await new Promise((r) => setTimeout(r, 1500));
// Helpers inside the page: wait for a view that says something, and freeze the page.
const prelude = `
  const r = window.rhythm; await r.ready; await r.ctx.resume();
  const until = (pred) => new Promise((res) => { const f = (d) => { if (d.view && pred(new Uint8Array(d.view))) { r.listeners.splice(r.listeners.indexOf(f), 1); res(); } }; r.listeners.push(f); });
  const sleep = (ms) => new Promise((res) => setTimeout(res, ms));
`;
// 1. Playing with write off: sound 1 chosen (steps 1, 5, 9, 13 lit), sound 2 on the off-beats
// (stitch), samples on 3 and 4, effect 6 punched (the twill on key 6, cores through it fern),
// caught just after step 5 fires sound 1 and the frame is on key 5.
await run(`${prelude}
  const cmds = [];
  for (const s of [0, 4, 8, 12]) cmds.push([16, 0, s, 1]);
  for (const s of [2, 6, 10, 14]) cmds.push([16, 1, s, 1]);
  cmds.push([16, 5, 4, 1], [22, 2, 0, 1], [22, 3, 0, 1], [5, 0, 0, 1]);
  r.send(...cmds);
  await until((v) => v[24] & 1 && v[24] & 8 && v[25] === 2);
  r.send([32, 20, 0, 0], [32, 5, 0, 0]);
  await until((v) => v[55] === 6 && v[25] === 4);
  await sleep(25);
  await r.ctx.suspend();
  const t = performance.now(); performance.now = () => t;
  await sleep(100);`);
await shoot("page-lights-playing.png");
await shoot("page-lights-playing-grey.png", true);
// 2. Sound held: the fields are the sounds in use, the bar on the chosen sound, a dash on each
// sample; sounds 1 and 4 sounding (cores, white: nothing is heard stopped).
await run(`${prelude}
  delete performance.now; await r.ctx.resume();
  r.send([33, 5, 0, 0], [33, 20, 0, 0], [5, 0, 0, 0]);
  await until((v) => !(v[24] & 1));
  r.send([32, 16, 0, 0], [21, 0, 0, 0], [21, 3, 0, 0]);
  await until((v) => (v[8] | v[9] << 8 | v[10] << 16) & 1 << 16);
  await sleep(20);
  await r.ctx.suspend();
  const t = performance.now(); performance.now = () => t;
  await sleep(100);`);
await shoot("page-sound-held.png");
await shoot("page-sound-held-grey.png", true);
// 3. The keys helper.
await run(`${prelude} delete performance.now; await r.ctx.resume(); r.send([33, 16, 0, 0]); document.getElementById("helpmark").click(); await sleep(200);`);
await shoot("page-helper.png");
ws.close();
chrome.kill();
server.close();
await new Promise((r) => setTimeout(r, 300));
rmSync(profile, { recursive: true, force: true });

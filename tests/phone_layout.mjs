// Browser check for the phone layout (BAN-45). Needs a running dev server and a headless Chrome:
//   node tests/phone_layout.mjs [url] [path-to-chrome-headless-shell]
// Checks, at phone widths: no sideways scroll, and the preview stays pinned while the page scrolls.
import { spawn } from "node:child_process";

const url = process.argv[2] ?? "http://127.0.0.1:8080/";
const chrome = process.argv[3] ?? process.env.CHROME ?? "chrome-headless-shell";
const port = 9333;
const proc = spawn(chrome, ["--no-sandbox", "--headless", `--remote-debugging-port=${port}`, "about:blank"], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function target() {
  for (let i = 0; i < 50; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
      const page = list.find((t) => t.type === "page");
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(100);
  }
  throw new Error("Chrome did not start");
}

const ws = new WebSocket(await target());
await new Promise((r) => ws.addEventListener("open", r, { once: true }));
let id = 0;
const pending = new Map();
ws.addEventListener("message", (e) => {
  const msg = JSON.parse(e.data);
  if (pending.has(msg.id)) pending.get(msg.id)(msg.result), pending.delete(msg.id);
});
const send = (method, params = {}) => new Promise((r) => { pending.set(++id, r); ws.send(JSON.stringify({ id, method, params })); });
const evaluate = async (expr) => (await send("Runtime.evaluate", { expression: expr, returnByValue: true })).result.value;

const failures = [];
for (const width of [320, 375]) {
  await send("Emulation.setDeviceMetricsOverride", { width, height: 740, deviceScaleFactor: 2, mobile: true });
  await send("Page.navigate", { url });
  for (let i = 0; i < 100 && !(await evaluate("!!document.querySelector('.stage #preview svg')")); i++) await sleep(100);
  await sleep(300);
  const wide = await evaluate("[document.documentElement.scrollWidth, innerWidth]");
  if (wide[0] > wide[1]) failures.push(`${width}px: page is ${wide[0]}px wide, scrolls sideways`);
  await evaluate("window.scrollTo(0, 600)");
  await sleep(200);
  const top = await evaluate("[scrollY, Math.round(document.querySelector('.stage').getBoundingClientRect().top)]");
  if (top[0] < 100) failures.push(`${width}px: page did not scroll (scrollY ${top[0]})`);
  else if (top[1] !== 0) failures.push(`${width}px: preview not pinned after scrolling, its top is at ${top[1]}px`);
}
ws.close();
proc.kill();
if (failures.length) {
  console.error("FAIL\n" + failures.join("\n"));
  process.exit(1);
}
console.log("ok: no sideways scroll, preview pinned at 320px and 375px");

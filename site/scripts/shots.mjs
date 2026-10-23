// Screenshots of the built site, to judge it by eye: every page at 1280 and
// 390 pixels wide in both themes. Run from site/ after `bun run build`,
// inside the devenv shell, whose PLAYWRIGHT_BROWSERS_PATH holds Chromium:
//
//   devenv shell -- bun run shots <out-dir> [page ...]

import { existsSync, mkdirSync, readFileSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const dist = resolve(here, "../dist");
const base = "/tailrace";
const [out = resolve(here, "../shots"), ...only] = process.argv.slice(2);
const pages = {
  start: "/",
  "how-it-works": "/how-it-works/",
  "how-it-fits": "/how-it-fits/",
  design: "/design/",
  roadmap: "/roadmap/",
};
const widths = [1280, 390];
const themes = ["dark", "light"];

const types = {
  ".html": "text/html",
  ".css": "text/css",
  ".js": "text/javascript",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".woff2": "font/woff2",
  ".woff": "font/woff",
  ".json": "application/json",
};
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, "http://x").pathname);
  if (!path.startsWith(base)) return res.writeHead(404).end();
  let file = join(dist, path.slice(base.length));
  if (existsSync(file) && statSync(file).isDirectory()) file = join(file, "index.html");
  if (!existsSync(file)) return res.writeHead(404).end();
  res
    .writeHead(200, { "content-type": types[extname(file)] || "application/octet-stream" })
    .end(readFileSync(file));
}).listen(0);
const origin = `http://127.0.0.1:${server.address().port}${base}`;

mkdirSync(out, { recursive: true });
// The full Chromium, which is the one the devenv shell's Nix store holds.
const browser = await chromium.launch({ channel: "chromium" });
try {
  for (const [name, path] of Object.entries(pages)) {
    if (only.length && !only.includes(name)) continue;
    for (const theme of themes) {
      for (const width of widths) {
        // The family diagram is animated; the page is shot as a reader who
        // asks for less motion sees it, at the diagram's still frame.
        const ctx = await browser.newContext({
          viewport: { width, height: 900 },
          colorScheme: theme,
          deviceScaleFactor: 2,
          reducedMotion: "reduce",
        });
        const page = await ctx.newPage();
        await page.goto(origin + path, { waitUntil: "networkidle" });
        await page.evaluate(() => document.fonts.ready);
        await page.waitForTimeout(200);
        await page.screenshot({ path: join(out, `${name}-${width}-${theme}.png`), fullPage: true });
        await ctx.close();
      }
    }
  }
} finally {
  await browser.close();
  server.close();
}
console.log(`shots in ${out}`);

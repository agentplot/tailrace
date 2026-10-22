// Screenshots the mockups in mock/ into public/screenshots/, a light and a
// dark frame each, 1280×754 at 2×, lossless WebP. Run from site/ inside the
// devenv shell: devenv shell -- bash -c 'cd site && node mock/shoot.mjs'
import { mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';
import sharp from 'sharp';

const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, '../public/screenshots');
mkdirSync(out, { recursive: true });
const only = process.argv.slice(2);
const mocks = ['claude-code', 'herdr'].filter((m) => !only.length || only.includes(m));

const browser = await chromium.launch({ channel: 'chromium' });
try {
  for (const mock of mocks) {
    for (const theme of ['light', 'dark']) {
      const ctx = await browser.newContext({ viewport: { width: 1280, height: 754 }, colorScheme: theme, deviceScaleFactor: 2 });
      const page = await ctx.newPage();
      await page.goto(pathToFileURL(resolve(here, `${mock}.html`)).href);
      await page.evaluate(() => document.fonts.ready);
      await page.waitForTimeout(150);
      const png = await page.screenshot({ type: 'png' });
      await ctx.close();
      const file = resolve(out, `${mock}-${theme}.webp`);
      // Lossless: the frames are text, which lossy WebP smears.
      await sharp(png).webp({ lossless: true, effort: 6 }).toFile(file);
      console.log(file);
    }
  }
} finally {
  await browser.close();
}

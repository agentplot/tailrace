// The environments diagram, still: a channel moves, and every environment
// that follows it converges, pushed or pulling, now or after an approval.
// One claim: an environment is never told a version; it follows a channel,
// and what it runs is reported back beside what it should run. Drawn twice,
// wide and tall; Figure.astro shows the one that fits.

import type { Layout } from './hero';

const P = 'tr';
const c = (cls: string) => cls.split(/\s+/).filter(Boolean).map((x) => `${P}-${x}`).join(' ');
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const rect = (x: number, y: number, w: number, h: number, cls: string, r = 8) =>
  `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" class="${c(cls)}"/>`;
const text = (x: number, y: number, cls: string, s: string, anchor?: 'middle' | 'end') =>
  `<text x="${x}" y="${y}" class="${c(cls)}"${anchor ? ` text-anchor="${anchor}"` : ''}>${esc(s)}</text>`;
const path = (d: string, cls = 'ln') => `<path d="${d}" class="${c(cls)}"/>`;
const head = (x: number, y: number, dir: 'r' | 'd' | 'u') =>
  dir === 'r'
    ? `<path d="M${x - 7},${y - 5} L${x},${y} L${x - 7},${y + 5} Z" class="${c('hd-fill')}"/>`
    : dir === 'd'
      ? `<path d="M${x - 5},${y - 7} L${x},${y} L${x + 5},${y - 7} Z" class="${c('hd-fill')}"/>`
      : `<path d="M${x - 5},${y + 7} L${x},${y} L${x + 5},${y + 7} Z" class="${c('hd-fill')}"/>`;

const style = `
.${P} text{font-family:var(--sl-font);fill:var(--d-fg)}
.${P} .${P}-mono{font-family:var(--sl-font-mono)}
.${P} .${P}-bg{fill:var(--d-bg)}
.${P} .${P}-hd{font-size:11px;letter-spacing:.08em;fill:var(--d-muted);font-weight:600}
.${P} .${P}-sm{font-size:13px} .${P} .${P}-xs{font-size:11.5px}
.${P} .${P}-b{font-weight:600} .${P} .${P}-m{fill:var(--d-muted)} .${P} .${P}-acc{fill:var(--d-accent)}
.${P} .${P}-box{fill:var(--d-panel);stroke:var(--d-rule);stroke-width:1.5}
.${P} .${P}-wait{fill:var(--d-panel);stroke:var(--d-rule);stroke-width:1.5;stroke-dasharray:5 4}
.${P} .${P}-stable{fill:var(--d-bg);stroke:var(--d-fg);stroke-width:1.5}
.${P} .${P}-ln{fill:none;stroke:var(--d-muted);stroke-width:1.6}
.${P} .${P}-pull{fill:none;stroke:var(--d-accent);stroke-width:1.6;stroke-dasharray:4 4}
.${P} .${P}-hd-fill{fill:var(--d-muted)}
.${P} .${P}-ok{fill:var(--d-ok)} .${P} .${P}-bad{fill:var(--d-bad)}
.${P} .${P}-okt{fill:var(--d-ok);font-size:11.5px;font-weight:600}
.${P} .${P}-badt{fill:var(--d-bad);font-size:11.5px;font-weight:600}
`;

const title = 'tailrace: a channel moves and its environments converge';
const desc =
  'The stable channel moves from 1.4.1 to 1.4.2 by a signed promotion, recorded in the ledger. Three environments follow stable. A static site moves on its own: a push reconciler syncs the bundle, and it reports running 1.4.2. A CloudFormation stack waits: it shows the promotion and runs 1.4.1 until an approver says so. A Kubernetes cluster pulls: Flux watches the channel tag, verifies the gate’s signature and applies it, and a watcher reports 1.4.2. Each reports what it runs beside what it should run, and where the two differ the status says so.';

const env = (
  o: string[],
  x: number,
  y: number,
  w: number,
  name: string,
  how: string,
  mech: string,
  runs: string,
  ok: boolean,
  waits = false,
) => {
  o.push(rect(x, y, w, 96, waits ? 'wait' : 'box'));
  o.push(text(x + 14, y + 24, 'sm b', name));
  o.push(text(x + 14, y + 42, 'xs m', how));
  o.push(text(x + 14, y + 58, 'xs m', mech));
  o.push(`<circle cx="${x + 19}" cy="${y + 78}" r="4" class="${c(ok ? 'ok' : 'bad')}"/>`);
  o.push(text(x + 30, y + 82, ok ? 'okt mono' : 'badt mono', runs));
};

function wide(): string {
  const W = 900;
  const H = 300;
  const o: string[] = [];
  o.push(text(20, 38, 'hd', 'A PROMOTION'));
  o.push(text(300, 38, 'hd', 'THE CHANNEL'));
  o.push(text(520, 38, 'hd', 'ENVIRONMENTS THAT FOLLOW IT'));

  o.push(rect(20, 58, 236, 70, 'box'));
  o.push(text(36, 82, 'sm b', 'an approver promotes'));
  o.push(text(36, 100, 'xs mono m', 'stable: 1.4.1 → 1.4.2'));
  o.push(text(36, 116, 'xs m', 'recorded in the ledger first'));
  o.push(path('M256,93 H293') + head(300, 93, 'r'));

  o.push(rect(300, 58, 190, 70, 'stable'));
  o.push(text(316, 82, 'sm b', 'stable'));
  o.push(text(316, 100, 'xs mono', '→ app 1.4.2'));
  o.push(text(316, 116, 'xs m', 'one head for every follower'));

  // fan out
  o.push(path('M490,93 H505 V106 H513') + head(520, 106, 'r'));
  o.push(path('M505,93 V178 H513') + head(520, 178, 'r'));
  o.push(path('M505,178 V250 H513', 'pull') + head(520, 250, 'r'));

  env(o, 520, 58, 360, 'static site · moves on its own', 'push: the bundle is synced by digest', 'converged when the head moved', 'runs 1.4.2', true);
  o.push(text(880, 82, 'xs m', 'auto', 'end'));
  env(o, 520, 130, 360, 'CloudFormation stack · waits', 'push: one stack operation, once approved', 'the promotion shown; nothing applied yet', 'runs 1.4.1 · behind', false, true);
  o.push(text(880, 154, 'xs m', 'manual', 'end'));
  env(o, 520, 202, 360, 'Kubernetes · pulls', 'Flux watches the tag, verifies the gate’s signature', 'a watcher reports what runs', 'runs 1.4.2', true);
  o.push(text(880, 226, 'xs m', 'pull', 'end'));

  o.push(text(20, 170, 'xs m', 'no environment is told a version;'));
  o.push(text(20, 186, 'xs m', 'each follows a channel, and reports'));
  o.push(text(20, 202, 'xs m', 'what it runs beside what it should.'));
  o.push(text(20, 236, 'xs m', 'push or pull is the target’s own;'));
  o.push(text(20, 252, 'xs m', 'the ledger and the policy are the same.'));
  return svg(W, H, o, 'ew');
}

function tall(): string {
  const W = 360;
  const H = 560;
  const o: string[] = [];
  o.push(text(20, 34, 'hd', 'A PROMOTION'));
  o.push(rect(20, 48, 320, 62, 'box'));
  o.push(text(34, 70, 'sm b', 'an approver promotes'));
  o.push(text(34, 88, 'xs mono m', 'stable: 1.4.1 → 1.4.2 · in the ledger first'));
  o.push(path('M180,110 V123') + head(180, 130, 'd'));

  o.push(rect(20, 130, 320, 52, 'stable'));
  o.push(text(34, 152, 'sm b', 'stable'));
  o.push(text(34, 170, 'xs mono', '→ app 1.4.2 · one head for every follower'));
  o.push(path('M180,182 V195') + head(180, 202, 'd'));

  env(o, 20, 202, 320, 'static site · moves on its own', 'push: the bundle is synced by digest', 'converged when the head moved', 'runs 1.4.2', true);
  o.push(path('M180,298 V311') + head(180, 318, 'd'));
  env(o, 20, 318, 320, 'CloudFormation stack · waits', 'push: one stack operation, once approved', 'the promotion shown; nothing applied yet', 'runs 1.4.1 · behind', false, true);
  o.push(path('M180,414 V427', 'pull') + head(180, 434, 'd'));
  env(o, 20, 434, 320, 'Kubernetes · pulls', 'Flux verifies the gate’s signature, applies', 'a watcher reports what runs', 'runs 1.4.2', true);
  o.push(text(20, 548, 'xs m', 'each follows a channel; each reports what it runs.'));
  return svg(W, H, o, 'et');
}

function svg(W: number, H: number, body: string[], id: string): string {
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" class="${P}" role="img" aria-labelledby="${P}-${id}-ttl ${P}-${id}-dsc">` +
    `<title id="${P}-${id}-ttl">${esc(title)}</title><desc id="${P}-${id}-dsc">${esc(desc)}</desc><style>${style}</style>` +
    `<rect class="${c('bg')}" width="${W}" height="${H}" rx="10"/>` +
    body.join('') +
    '</svg>'
  );
}

export function environmentsSvg(layout: Layout): string {
  return layout === 'wide' ? wide() : tall();
}

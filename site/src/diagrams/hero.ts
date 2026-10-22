// The start page's diagram, still: the whole of tailrace in one picture.
// sluice lands a commit; tailrace publish versions what the build left from
// history, packs and signs it, and offers it into the store, in the user's
// own account, where its attestations hang off its digest as referrers. The
// gate reads the policy, in the repo, and stamps the offered artifact
// accepted; the channels point at accepted versions and move only by a
// promotion, each written to the ledger first. sync and serve read the
// store under the policy. Each environment follows a channel, a reconciler
// converges it, pushed to or pulling, now or after an approval, and it
// reports what it runs beside what it should. One claim: the bytes that
// passed the gate are the bytes that run; your hands are on the policy and
// the promotions. Drawn twice, wide and tall; Figure.astro shows the one
// that fits.

export type Layout = 'wide' | 'tall';

const P = 'tr';
const c = (cls: string) => cls.split(/\s+/).filter(Boolean).map((x) => `${P}-${x}`).join(' ');
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

const rect = (x: number, y: number, w: number, h: number, cls: string, r = 8) =>
  `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" class="${c(cls)}"/>`;
const text = (x: number, y: number, cls: string, s: string, anchor?: 'middle' | 'end') =>
  `<text x="${x}" y="${y}" class="${c(cls)}"${anchor ? ` text-anchor="${anchor}"` : ''}>${esc(s)}</text>`;
const path = (d: string, cls = 'ln') => `<path d="${d}" class="${c(cls)}"/>`;
/** An arrowhead pointing right, down, up or left, its tip at (x, y). */
const head = (x: number, y: number, dir: 'r' | 'd' | 'u' | 'l') => {
  const d =
    dir === 'r'
      ? `M${x - 7},${y - 5} L${x},${y} L${x - 7},${y + 5} Z`
      : dir === 'l'
        ? `M${x + 7},${y - 5} L${x},${y} L${x + 7},${y + 5} Z`
        : dir === 'd'
          ? `M${x - 5},${y - 7} L${x},${y} L${x + 5},${y - 7} Z`
          : `M${x - 5},${y + 7} L${x},${y} L${x + 5},${y + 7} Z`;
  return `<path d="${d}" class="${c('hd-fill')}"/>`;
};
const dot = (x: number, y: number, cls = 'ok') => `<circle cx="${x}" cy="${y}" r="4" class="${c(cls)}"/>`;

/** The gate: a hexagon with its left and right points at (x, y) and (x + w, y). */
const gate = (x: number, y: number, w: number) => {
  const h = 16;
  const d = `M${x},${y} L${x + 10},${y - h} H${x + w - 10} L${x + w},${y} L${x + w - 10},${y + h} H${x + 10} Z`;
  return `<path d="${d}" class="${c('gate')}"/>` + text(x + w / 2, y + 4, 'xs b acc', 'gate', 'middle');
};
/** The policy: a page with a folded corner, its top-left at (x, y). */
const page = (x: number, y: number) =>
  `<path d="M${x},${y} H${x + 11} L${x + 16},${y + 5} V${y + 20} H${x} Z" class="${c('page')}"/>` +
  `<path d="M${x + 11},${y} V${y + 5} H${x + 16}" class="${c('page')}"/>`;
/** What an environment runs beside what it should, in one line after a green or amber dot. */
const runs = (x: number, y: number, runs: string, should: string) =>
  dot(x, y - 4, runs === should ? 'ok' : 'bad') +
  `<text x="${x + 10}" y="${y}" class="${c('xxs mono')}"><tspan class="${c(runs === should ? 'okt' : 'badt')}">runs ${esc(runs)}</tspan>` +
  `<tspan class="${c('m')}"> · should ${esc(should)}</tspan></text>`;

const style = `
.${P} text{font-family:var(--sl-font);fill:var(--d-fg)}
.${P} .${P}-mono{font-family:var(--sl-font-mono)}
.${P} .${P}-bg{fill:var(--d-bg)}
.${P} .${P}-hd{font-size:11px;letter-spacing:.08em;fill:var(--d-muted);font-weight:600}
.${P} .${P}-sm{font-size:13px} .${P} .${P}-xs{font-size:11.5px} .${P} .${P}-xxs{font-size:10px}
.${P} .${P}-b{font-weight:600} .${P} .${P}-m{fill:var(--d-muted)} .${P} .${P}-acc{fill:var(--d-accent)}
.${P} .${P}-box{fill:var(--d-panel);stroke:var(--d-rule);stroke-width:1.5}
.${P} .${P}-panel{fill:var(--d-soft);stroke:var(--d-rule);stroke-width:1.5}
.${P} .${P}-art{fill:var(--d-bg);stroke:var(--d-accent);stroke-width:1.5}
.${P} .${P}-chip{fill:var(--d-panel);stroke:var(--d-accent);stroke-width:1.2}
.${P} .${P}-chipt{fill:var(--d-accent);font-size:10.5px;font-weight:600}
.${P} .${P}-state{fill:var(--d-bg);stroke:var(--d-muted);stroke-width:1.2}
.${P} .${P}-okpill{fill:var(--d-bg);stroke:var(--d-ok);stroke-width:1.5}
.${P} .${P}-gate{fill:var(--d-accent-soft);stroke:var(--d-accent);stroke-width:1.5}
.${P} .${P}-page{fill:var(--d-bg);stroke:var(--d-muted);stroke-width:1.2;stroke-linejoin:round}
.${P} .${P}-beta{fill:var(--d-accent)} .${P} .${P}-betat{fill:var(--d-on-accent);font-size:11.5px;font-weight:600}
.${P} .${P}-stable{fill:var(--d-bg);stroke:var(--d-fg);stroke-width:1.5}
.${P} .${P}-ln{fill:none;stroke:var(--d-muted);stroke-width:1.6}
.${P} .${P}-hd-fill{fill:var(--d-muted)}
.${P} .${P}-ref{fill:none;stroke:var(--d-accent);stroke-width:1.2}
.${P} .${P}-rep{fill:none;stroke:var(--d-accent);stroke-width:1.2;stroke-dasharray:3 3}
.${P} .${P}-ok{fill:var(--d-ok)} .${P} .${P}-bad{fill:var(--d-bad)}
.${P} .${P}-okt{fill:var(--d-ok);font-weight:600} .${P} .${P}-badt{fill:var(--d-bad);font-weight:600}
`;

const title = 'tailrace: from a landing, through the gate, to the environments';
const desc =
  'sluice lands commit 9f3e21c on main. tailrace publish derives the version from history, packs and signs what the build left, and offers it into the store, in your own account. There the artifact app 1.4.2 holds its attestations as referrers of its digest: the build provenance, sluice’s verdict and the tests. The gate reads the policy, which lives in the repo, and stamps the offered artifact accepted. Two channels point at accepted versions, beta at 1.4.2 and stable at 1.4.1; only a promotion moves a channel, and every move is written to the ledger first: stable 1.4.0 to 1.4.1 by an approver, beta 1.4.1 to 1.4.2 by the gate. tailrace sync and tailrace serve fill npm, Maven and pip layouts from the store under the policy. Three environments follow a channel: a laptop follows beta, pushed to, and runs 1.4.2; staging follows beta, pulling with Flux, and runs 1.4.2; production follows stable, pushed to after an approver says so, and runs 1.4.1. A reconciler converges each, pushed to or pulling, now or after an approval, and verifies before it applies; each reports what it runs, and tailrace status shows what it should run, what it runs, and where the two differ. The bytes that passed the gate are the bytes that run; your hands are on the policy and the promotions.';

const envs: Array<{ name: string; channel: string; how: string; runs: string; should: string }> = [
  { name: 'laptop', channel: 'beta', how: 'push · converges now', runs: '1.4.2', should: '1.4.2' },
  { name: 'staging', channel: 'beta', how: 'pull (Flux) · converges now', runs: '1.4.2', should: '1.4.2' },
  { name: 'production', channel: 'stable', how: 'push · waits for an approver', runs: '1.4.1', should: '1.4.1' },
];
const ledger = ['stable 1.4.0 → 1.4.1 · approver', 'beta 1.4.1 → 1.4.2 · gate'];
const reconciler = 'a reconciler converges each: pushed to, or pulling; now, or after an approval; it verifies before it applies';
const claim = 'The bytes that passed the gate are the bytes that run; your hands are on the policy and the promotions.';

/** One environment's card: the channel it follows, how it converges, what it runs beside what it should. */
const envCard = (o: string[], x: number, y: number, w: number, e: (typeof envs)[number]) => {
  o.push(rect(x, y, w, 66, 'box'));
  o.push(text(x + 14, y + 22, 'sm b', e.name));
  o.push(text(x + w - 14, y + 22, 'xs mono m', `follows ${e.channel}`, 'end'));
  o.push(text(x + 14, y + 40, 'xs m', e.how));
  o.push(runs(x + 14, y + 57, e.runs, e.should));
};
/** The status chip: tailrace status reads the reports. */
const statusChip = (o: string[], x: number, y: number, w: number) => {
  o.push(rect(x, y, w, 44, 'chip', 10));
  o.push(text(x + w / 2, y + 18, 'chipt mono', 'tailrace status', 'middle'));
  o.push(text(x + w / 2, y + 34, 'xxs m', 'should run · runs · differ', 'middle'));
};

function wide(): string {
  const W = 900;
  const H = 600;
  const o: string[] = [];
  // The columns.
  o.push(text(20, 38, 'hd', 'SLUICE LANDS'));
  o.push(text(170, 38, 'hd', 'PUBLISH'));
  o.push(text(340, 38, 'hd', 'THE STORE, IN YOUR ACCOUNT'));
  o.push(text(684, 38, 'hd', 'SYNC · SERVE'));

  // sluice lands a commit.
  o.push(rect(20, 58, 130, 70, 'box'));
  o.push(dot(36, 80));
  o.push(text(46, 84, 'sm b', 'sluice'));
  o.push(text(36, 104, 'xs m', 'lands on main'));
  o.push(text(36, 120, 'xs mono m', '9f3e21c'));
  o.push(path('M150,93 H163') + head(170, 93, 'r'));

  // tailrace publish.
  o.push(rect(170, 58, 150, 70, 'box'));
  o.push(text(184, 82, 'sm b mono', 'tailrace publish'));
  o.push(text(184, 100, 'xs m', 'version from history'));
  o.push(text(184, 116, 'xs m', 'pack · sign · offered'));
  o.push(path('M320,93 H333') + head(340, 93, 'r'));

  // The store: the artifact and its referrers, the gate and the policy,
  // the channels and the ledger.
  o.push(rect(340, 58, 320, 320, 'panel', 10));
  o.push(rect(356, 78, 112, 48, 'art'));
  o.push(text(368, 98, 'sm b mono', 'app 1.4.2'));
  o.push(text(368, 116, 'xxs mono m', 'sha256:9f3e…c21'));
  o.push(path('M468,102 H476 V145 M476,89 H484 M476,117 H484 M476,145 H484', 'ref'));
  for (const [i, s] of ['provenance', 'sluice’s verdict', 'tests'].entries()) {
    const y = 78 + i * 28;
    o.push(rect(484, y, 108, 22, 'chip', 11));
    o.push(text(538, y + 15, 'chipt', s, 'middle'));
  }
  o.push(text(356, 172, 'xxs m', 'its attestations hang off its digest, as referrers'));
  // offered → gate → accepted; the gate reads the policy.
  o.push(rect(356, 184, 64, 28, 'state', 14));
  o.push(text(388, 202, 'xs m', 'offered', 'middle'));
  o.push(path('M420,198 H437') + head(444, 198, 'r'));
  o.push(gate(444, 198, 56));
  o.push(path('M500,198 H517') + head(524, 198, 'r'));
  o.push(rect(524, 184, 84, 28, 'okpill', 14));
  o.push(text(566, 202, 'xs okt', 'accepted', 'middle'));
  o.push(path('M472,214 V224', 'ref'));
  o.push(page(464, 224));
  o.push(text(488, 238, 'xxs m', 'policy, in the repo'));
  // The channels.
  o.push(rect(356, 256, 104, 26, 'beta', 13));
  o.push(text(408, 273, 'betat mono', 'beta → 1.4.2', 'middle'));
  o.push(rect(470, 256, 108, 26, 'stable', 13));
  o.push(text(524, 273, 'xs b mono', 'stable → 1.4.1', 'middle'));
  // The ledger.
  o.push(rect(356, 294, 288, 44, 'box', 6));
  for (const [i, s] of ledger.entries()) o.push(text(368, 311 + i * 16, 'xxs mono', s));
  o.push(text(632, 311, 'xxs m', 'ledger', 'end'));
  o.push(text(356, 356, 'xxs m', 'only a promotion moves a channel;'));
  o.push(text(356, 370, 'xxs m', 'every move is in the ledger first'));
  o.push(path('M660,93 H677') + head(684, 93, 'r'));

  // sync and serve, under the policy.
  o.push(rect(684, 58, 196, 70, 'box'));
  o.push(text(698, 82, 'xs b mono', 'tailrace sync · serve'));
  o.push(text(698, 100, 'xs m', 'layouts: npm · Maven · pip'));
  o.push(text(698, 116, 'xs m', 'under the policy'));
  for (const [i, s] of ['a laptop or a CI host reads', 'its ecosystem’s own form,', 'only what the policy admits'].entries()) {
    o.push(text(684, 148 + i * 14, 'xxs m', s));
  }

  // From each channel to the environments that follow it.
  const cx = [126, 354, 582];
  o.push(path(`M356,269 H332 V404 H${cx[0]} V423 M332,404 H${cx[1]} V423`));
  o.push(head(cx[0], 430, 'd') + head(cx[1], 430, 'd'));
  o.push(path(`M578,269 H676 V404 H${cx[2]} V423`) + head(cx[2], 430, 'd'));

  // The environments, each on a channel, and the status that reads their reports.
  o.push(text(20, 420, 'hd', 'ENVIRONMENTS'));
  for (const [i, e] of envs.entries()) envCard(o, 20 + i * 228, 430, 212, e);
  statusChip(o, 708, 441, 172);
  o.push(path(`M${cx[0]},496 V518 H794 V492 M${cx[1]},496 V518 M${cx[2]},496 V518`, 'rep') + head(794, 485, 'u'));
  o.push(text(700, 512, 'xxs acc', 'reports', 'end'));
  o.push(text(20, 544, 'xs m', reconciler));

  // The claim.
  o.push(text(20, 576, 'sm b', claim));
  return svg(W, H, o, 'w');
}

function tall(): string {
  const W = 360;
  const H = 1064;
  const x = 20;
  const w = 320;
  const mid = x + w / 2;
  const o: string[] = [];

  // sluice lands a commit.
  o.push(text(x, 28, 'hd', 'SLUICE LANDS'));
  o.push(rect(x, 36, w, 56, 'box'));
  o.push(dot(x + 16, 58));
  o.push(text(x + 26, 62, 'sm b', 'sluice'));
  o.push(text(x + 16, 82, 'xs m', 'lands 9f3e21c on main'));
  o.push(path(`M${mid},92 V105`) + head(mid, 112, 'd'));

  // tailrace publish.
  o.push(rect(x, 112, w, 56, 'box'));
  o.push(text(x + 16, 136, 'sm b mono', 'tailrace publish'));
  o.push(text(x + 16, 156, 'xs m', 'version from history · pack · sign · offered'));
  o.push(path(`M${mid},168 V181`) + head(mid, 188, 'd'));

  // The store.
  o.push(rect(x, 188, w, 334, 'panel', 10));
  o.push(text(x + 16, 212, 'hd', 'THE STORE, IN YOUR ACCOUNT'));
  o.push(rect(x + 16, 222, 124, 48, 'art'));
  o.push(text(x + 28, 242, 'sm b mono', 'app 1.4.2'));
  o.push(text(x + 28, 260, 'xxs mono m', 'sha256:9f3e…c21'));
  o.push(path(`M${x + 140},246 H${x + 150} V289 M${x + 150},233 H${x + 158} M${x + 150},261 H${x + 158} M${x + 150},289 H${x + 158}`, 'ref'));
  for (const [i, s] of ['provenance', 'sluice’s verdict', 'tests'].entries()) {
    const y = 222 + i * 28;
    o.push(rect(x + 158, y, 116, 22, 'chip', 11));
    o.push(text(x + 216, y + 15, 'chipt', s, 'middle'));
  }
  o.push(text(x + 16, 316, 'xxs m', 'its attestations hang off its digest, as referrers'));
  // offered → gate → accepted; the gate reads the policy.
  o.push(rect(x + 16, 328, 64, 28, 'state', 14));
  o.push(text(x + 48, 346, 'xs m', 'offered', 'middle'));
  o.push(path(`M${x + 80},342 H${x + 97}`) + head(x + 104, 342, 'r'));
  o.push(gate(x + 104, 342, 56));
  o.push(path(`M${x + 160},342 H${x + 177}`) + head(x + 184, 342, 'r'));
  o.push(rect(x + 184, 328, 84, 28, 'okpill', 14));
  o.push(text(x + 226, 346, 'xs okt', 'accepted', 'middle'));
  o.push(path(`M${x + 132},358 V368`, 'ref'));
  o.push(page(x + 124, 368));
  o.push(text(x + 148, 382, 'xxs m', 'policy, in the repo'));
  // The channels, each with a rail down to the environments that follow it.
  o.push(rect(x + 16, 398, 104, 26, 'beta', 13));
  o.push(text(x + 68, 415, 'betat mono', 'beta → 1.4.2', 'middle'));
  o.push(rect(x + 130, 398, 108, 26, 'stable', 13));
  o.push(text(x + 184, 415, 'xs b mono', 'stable → 1.4.1', 'middle'));
  // The ledger.
  o.push(rect(x + 16, 436, 254, 44, 'box', 6));
  for (const [i, s] of ledger.entries()) o.push(text(x + 28, 453 + i * 16, 'xxs mono', s));
  o.push(text(x + 258, 469, 'xxs m', 'ledger', 'end'));
  o.push(text(x + 16, 498, 'xxs m', 'only a promotion moves a channel;'));
  o.push(text(x + 16, 512, 'xxs m', 'every move is in the ledger first'));
  o.push(path(`M${x + 135},522 V535`) + head(x + 135, 542, 'd'));

  // sync and serve, under the policy.
  o.push(rect(x, 542, 270, 56, 'box'));
  o.push(text(x + 16, 566, 'xs b mono', 'tailrace sync · serve'));
  o.push(text(x + 16, 584, 'xs m', 'npm, Maven, pip layouts, under the policy'));

  // The environments, each on a channel; the rails from the channels on the
  // right, the reports to the status on the left.
  const beta = x + 292;
  const stable = x + 310;
  const cxl = x + 20;
  const cw = 250;
  const cy = [640, 718, 796];
  o.push(text(x, 628, 'hd', 'ENVIRONMENTS'));
  for (const [i, e] of envs.entries()) envCard(o, cxl, cy[i], cw, e);
  // beta: out of its pill, right under the stable pill, down the outer rail.
  o.push(path(`M${x + 68},424 V430 H${beta} V${cy[1] + 33} H${cxl + cw + 7} M${beta},${cy[0] + 33} H${cxl + cw + 7}`));
  o.push(head(cxl + cw, cy[0] + 33, 'l') + head(cxl + cw, cy[1] + 33, 'l'));
  // stable: out of its pill's right edge, down the inner rail.
  o.push(path(`M${x + 238},411 H${stable} V${cy[2] + 33} H${cxl + cw + 7}`) + head(cxl + cw, cy[2] + 33, 'l'));
  // reports: each card to the status, down the left.
  statusChip(o, cxl, 880, cw);
  const rail = x + 8;
  o.push(path(`M${cxl},${cy[0] + 50} H${rail} V902 H${cxl - 7} M${cxl},${cy[1] + 50} H${rail} M${cxl},${cy[2] + 50} H${rail}`, 'rep'));
  o.push(head(cxl, 902, 'r'));
  o.push(text(cxl + 4, 874, 'xxs acc', 'reports'));
  for (const [i, s] of ['a reconciler converges each: pushed to, or pulling;', 'now, or after an approval;', 'it verifies before it applies'].entries()) {
    o.push(text(x, 948 + i * 14, 'xs m', s));
  }

  // The claim.
  o.push(text(x, 1006, 'sm b', 'The bytes that passed the gate'));
  o.push(text(x, 1024, 'sm b', 'are the bytes that run; your hands'));
  o.push(text(x, 1042, 'sm b', 'are on the policy and the promotions.'));
  return svg(W, H, o, 't');
}

/** The drawing; `id` keeps the two layouts' title and description ids apart. */
function svg(W: number, H: number, body: string[], id: string): string {
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" class="${P}" role="img" aria-labelledby="${P}-${id}-ttl ${P}-${id}-dsc">` +
    `<title id="${P}-${id}-ttl">${esc(title)}</title><desc id="${P}-${id}-dsc">${esc(desc)}</desc><style>${style}</style>` +
    `<rect class="${c('bg')}" width="${W}" height="${H}" rx="10"/>` +
    body.join('') +
    '</svg>'
  );
}

export function heroSvg(layout: Layout): string {
  return layout === 'wide' ? wide() : tall();
}

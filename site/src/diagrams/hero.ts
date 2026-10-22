// The start page's diagram, still: the pipeline from the landing to the
// environments. sluice lands a commit; tailrace versions, packages and signs
// what the build left and pushes it by digest into the store, in the user's
// own account, where its attestations hang off the digest as referrers and
// the channels point at versions; sync and serve read the store under a
// policy; and each environment subscribes to a channel. One claim: the bytes
// that passed the gate are the bytes that run. Drawn twice, wide and tall;
// Figure.astro shows the one that fits.

export type Layout = 'wide' | 'tall';

const P = 'tr';
const c = (cls: string) => cls.split(/\s+/).filter(Boolean).map((x) => `${P}-${x}`).join(' ');
const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

const rect = (x: number, y: number, w: number, h: number, cls: string, r = 8) =>
  `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" class="${c(cls)}"/>`;
const text = (x: number, y: number, cls: string, s: string, anchor?: 'middle' | 'end') =>
  `<text x="${x}" y="${y}" class="${c(cls)}"${anchor ? ` text-anchor="${anchor}"` : ''}>${esc(s)}</text>`;
const path = (d: string, cls = 'ln') => `<path d="${d}" class="${c(cls)}"/>`;
/** An arrowhead pointing right or down, its tip at (x, y). */
const head = (x: number, y: number, dir: 'r' | 'd') =>
  dir === 'r'
    ? `<path d="M${x - 7},${y - 5} L${x},${y} L${x - 7},${y + 5} Z" class="${c('hd-fill')}"/>`
    : `<path d="M${x - 5},${y - 7} L${x},${y} L${x + 5},${y - 7} Z" class="${c('hd-fill')}"/>`;

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
.${P} .${P}-beta{fill:var(--d-accent)} .${P} .${P}-betat{fill:var(--d-on-accent);font-size:11.5px;font-weight:600}
.${P} .${P}-stable{fill:var(--d-bg);stroke:var(--d-fg);stroke-width:1.5}
.${P} .${P}-ln{fill:none;stroke:var(--d-muted);stroke-width:1.6}
.${P} .${P}-hd-fill{fill:var(--d-muted)}
.${P} .${P}-ref{fill:none;stroke:var(--d-accent);stroke-width:1.2}
.${P} .${P}-ok{fill:var(--d-ok)}
`;

const title = 'tailrace: from the landing to the environments';
const desc =
  'sluice lands commit 9f3e21c on main. tailrace push versions, packages and signs what the build left and pushes it by digest into quarantine. The store, in your own account, holds the artifact app 1.4.2 with its attestations hanging off its digest as referrers: the build provenance, sluice’s verdicts and the tests. The beta channel points at 1.4.2 and stable at 1.4.1; stable moves only by a signed promotion, recorded in an append-only ledger. tailrace sync and tailrace serve read the store under a policy, so a version is served only when its attestations satisfy it. Three environments subscribe to a channel, a laptop and staging to beta and production to stable, and a reconciler converges each, push or pull. The bytes that passed the gate are the bytes that run.';

function wide(): string {
  const W = 900;
  const H = 338;
  const o: string[] = [];
  // The columns.
  o.push(text(20, 38, 'hd', 'SLUICE LANDS'));
  o.push(text(196, 38, 'hd', 'PUBLISH'));
  o.push(text(372, 38, 'hd', 'THE STORE, IN YOUR ACCOUNT'));
  o.push(text(650, 38, 'hd', 'SYNC · SERVE'));
  o.push(text(792, 38, 'hd', 'ENVIRONMENTS'));

  // sluice lands a commit.
  o.push(rect(20, 58, 146, 70, 'box'));
  o.push(`<circle cx="36" cy="80" r="4" class="${c('ok')}"/>`);
  o.push(text(46, 84, 'sm b', 'sluice'));
  o.push(text(36, 104, 'xs m', 'lands on main'));
  o.push(text(36, 120, 'xs mono m', '9f3e21c'));
  o.push(path('M166,93 H189') + head(196, 93, 'r'));

  // tailrace push.
  o.push(rect(196, 58, 146, 70, 'box'));
  o.push(text(212, 84, 'sm b mono', 'tailrace push'));
  o.push(text(212, 104, 'xs m', 'version · package · sign'));
  o.push(text(212, 120, 'xs m', 'into quarantine'));
  o.push(path('M342,93 H365') + head(372, 93, 'r'));

  // The store: the artifact, its referrers and the channels.
  o.push(rect(372, 58, 248, 216, 'panel', 10));
  o.push(rect(388, 78, 124, 48, 'art'));
  o.push(text(400, 98, 'sm b mono', 'app 1.4.2'));
  o.push(text(400, 116, 'xxs mono m', 'sha256:9f3e…c21'));
  o.push(path('M512,102 H521 V145 M521,89 H530 M521,117 H530 M521,145 H530', 'ref'));
  for (const [i, s] of ['provenance', 'sluice verdicts', 'tests'].entries()) {
    const y = 78 + i * 28;
    o.push(rect(530, y, 78, 22, 'chip', 11));
    o.push(text(569, y + 15, 'chipt', s, 'middle'));
  }
  o.push(text(388, 172, 'xxs m', 'its attestations hang off its digest'));
  o.push(text(388, 198, 'xxs m', 'channels'));
  o.push(rect(388, 204, 104, 26, 'beta', 13));
  o.push(text(440, 221, 'betat mono', 'beta → 1.4.2', 'middle'));
  o.push(rect(500, 204, 108, 26, 'stable', 13));
  o.push(text(554, 221, 'xs b mono', 'stable → 1.4.1', 'middle'));
  o.push(text(388, 250, 'xxs m', 'stable moves only by a signed promotion,'));
  o.push(text(388, 264, 'xxs m', 'a record in an append-only ledger'));
  o.push(path('M620,93 H643') + head(650, 93, 'r'));

  // sync and serve, under a policy.
  o.push(rect(650, 58, 124, 70, 'box'));
  o.push(text(662, 82, 'xs b mono', 'tailrace sync'));
  o.push(text(662, 102, 'xs b mono', 'tailrace serve'));
  o.push(text(662, 120, 'xs m', 'under a policy'));
  for (const [i, s] of ['fills npm, PyPI, Cargo,', 'Go, Maven, NuGet and', 'images from the store;', 'serves what the', 'policy allows'].entries()) {
    o.push(text(650, 150 + i * 14, 'xxs m', s));
  }
  o.push(path('M774,93 H781 V78 H785 M781,93 V126 H785 M781,126 V174 H785'));
  o.push(head(792, 78, 'r') + head(792, 126, 'r') + head(792, 174, 'r'));

  // The environments, each on a channel.
  const envs: Array<[string, string]> = [
    ['laptop', 'beta'],
    ['staging', 'beta'],
    ['production', 'stable'],
  ];
  for (const [i, [name, ch]] of envs.entries()) {
    const y = 58 + i * 48;
    o.push(rect(792, y, 92, 40, 'box'));
    o.push(text(804, y + 17, 'sm b', name));
    o.push(text(804, y + 32, 'xxs m mono', ch));
  }
  for (const [i, s] of ['each subscribes', 'to a channel; a', 'reconciler converges', 'it, push or pull'].entries()) {
    o.push(text(792, 214 + i * 14, 'xxs m', s));
  }

  // The claim.
  o.push(text(20, 304, 'sm b', 'The bytes that passed the gate are the bytes that run.'));
  o.push(text(20, 322, 'sm m', 'A cache hit, an artifact and a release are one digest at three levels of trust.'));
  return svg(W, H, o, 'w');
}

function tall(): string {
  const W = 372;
  const H = 742;
  const x = 20;
  const w = 332;
  const mid = x + w / 2;
  const o: string[] = [];

  // sluice lands a commit.
  o.push(text(x, 28, 'hd', 'SLUICE LANDS'));
  o.push(rect(x, 36, w, 56, 'box'));
  o.push(`<circle cx="${x + 16}" cy="58" r="4" class="${c('ok')}"/>`);
  o.push(text(x + 26, 62, 'sm b', 'sluice'));
  o.push(text(x + 16, 82, 'xs m', 'lands 9f3e21c on main'));
  o.push(path(`M${mid},92 V105`) + head(mid, 112, 'd'));

  // tailrace push.
  o.push(rect(x, 112, w, 56, 'box'));
  o.push(text(x + 16, 136, 'sm b mono', 'tailrace push'));
  o.push(text(x + 16, 156, 'xs m', 'version · package · sign · into quarantine'));
  o.push(path(`M${mid},168 V181`) + head(mid, 188, 'd'));

  // The store.
  o.push(rect(x, 188, w, 250, 'panel', 10));
  o.push(text(x + 16, 212, 'hd', 'THE STORE, IN YOUR ACCOUNT'));
  o.push(rect(x + 16, 222, 140, 48, 'art'));
  o.push(text(x + 28, 242, 'sm b mono', 'app 1.4.2'));
  o.push(text(x + 28, 260, 'xxs mono m', 'sha256:9f3e…c21'));
  o.push(path(`M${x + 156},246 H${x + 170} V289 M${x + 170},233 H${x + 180} M${x + 170},261 H${x + 180} M${x + 170},289 H${x + 180}`, 'ref'));
  for (const [i, s] of ['provenance', 'sluice verdicts', 'tests'].entries()) {
    const y = 222 + i * 28;
    o.push(rect(x + 180, y, 120, 22, 'chip', 11));
    o.push(text(x + 240, y + 15, 'chipt', s, 'middle'));
  }
  o.push(text(x + 16, 296, 'xxs m', 'attestations, by digest'));
  o.push(text(x + 16, 328, 'xxs m', 'channels'));
  o.push(rect(x + 16, 336, 130, 26, 'beta', 13));
  o.push(text(x + 81, 353, 'betat mono', 'beta → 1.4.2', 'middle'));
  o.push(rect(x + 162, 336, 140, 26, 'stable', 13));
  o.push(text(x + 232, 353, 'xs b mono', 'stable → 1.4.1', 'middle'));
  o.push(text(x + 16, 386, 'xxs m', 'stable moves only by a signed promotion,'));
  o.push(text(x + 16, 400, 'xxs m', 'a record in an append-only ledger'));
  o.push(path(`M${mid},438 V451`) + head(mid, 458, 'd'));

  // sync and serve, under a policy.
  o.push(rect(x, 458, w, 56, 'box'));
  o.push(text(x + 16, 482, 'sm b mono', 'tailrace sync · tailrace serve'));
  o.push(text(x + 16, 502, 'xs m', 'a version is served only when the policy allows it'));
  o.push(path(`M${mid},514 V527`) + head(mid, 534, 'd'));

  // The environments, each on a channel.
  const envs: Array<[string, string]> = [
    ['laptop', 'beta'],
    ['staging', 'beta'],
    ['production', 'stable'],
  ];
  for (const [i, [name, ch]] of envs.entries()) {
    const ex = x + i * 116;
    o.push(rect(ex, 534, 100, 44, 'box'));
    o.push(text(ex + 12, 552, 'sm b', name));
    o.push(text(ex + 12, 569, 'xxs m mono', ch));
  }
  o.push(text(x, 602, 'xxs m', 'each subscribes to a channel;'));
  o.push(text(x, 616, 'xxs m', 'a reconciler converges it, push or pull'));

  // The claim.
  o.push(text(x, 660, 'sm b', 'The bytes that passed the gate'));
  o.push(text(x, 678, 'sm b', 'are the bytes that run.'));
  o.push(text(x, 704, 'xs m', 'A cache hit, an artifact and a release are'));
  o.push(text(x, 720, 'xs m', 'one digest at three levels of trust.'));
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

# The site

tailrace's site at https://agentplot.github.io/tailrace, an
[Astro](https://astro.build) site with [Starlight](https://starlight.astro.build),
published by `.github/workflows/pages.yml` on every push to `main` that
touches `site/` or the README.

```sh
devenv shell -- site-dev                      # http://localhost:4321/tailrace/
devenv shell -- moon run site:build           # dist/, with every internal link checked
devenv shell -- bash -c 'cd site && bun run shots <dir>'   # screenshots, after a build
```

## Where the pages live

`src/content/docs/`, one file per page, in the sidebar `astro.config.mjs`
lists: the start page (`index.mdx`), how it works, how it fits, the
guides (`guides/self-hosting.mdx`, and `guides/integrations/`, one page
per kind of thing tailrace binds, listed in the sidebar by their `order`),
the design and the roadmap. The theme's banner says tailrace is designed, not built,
on every page, until its entry in the theme's family list says otherwise.

## The look and the diagram

The family's header, footer and diagram are `@agentplot/site-theme`'s,
taken from GitHub at the commit `package.json` names.

| file | what |
|---|---|
| `src/styles/diagrams.css` | the diagram's palette, the only colours it draws with: the family's tokens, tailrace's accent, `--ap-accent`, the theme's ok colour for a landing, and its warning colour for a version a policy refuses |
| `src/diagrams/environments.ts` | how it works, still: a channel moves and the environments that follow it converge, one on its own, one waiting on an approver, one pulling, each reporting what it runs |
| `src/diagrams/hero.ts` | the start page, still: the pipeline from the landing to the environments, the artifact with its referrers and the channels in a store the user owns |
| `src/components/Hero.astro` | the start page's hero: the name, the tagline, the actions and the diagram |
| `src/components/Figure.astro` | a diagram drawn wide and tall, the tall one on a phone |
| `scripts/shots.mjs` | screenshots of every page at 1280 and 390 pixels in both themes |

The diagram makes one claim, that the bytes that passed the gate are the
bytes that run, and is drawn still: it shows a shape, not a mechanism in
motion. Change it in its own file; the wide and tall layouts share the
same text.

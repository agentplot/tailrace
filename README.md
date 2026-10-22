# tailrace

**Designed, not built.** The design is being settled in
[agentplot/blueprints](https://github.com/agentplot/blueprints); no code
exists yet.

**Build once, sign everything, promote with proof, in your own account.**

tailrace delivers what the merge queue lands. After
[sluice](https://github.com/agentplot/sluice) lands a change, tailrace
versions, packages, signs and pushes what the build left as OCI artifacts
into a registry you own: ECR, GHCR, Harbor, zot, or a file-based OCI
layout on disk. To each artifact's digest it attaches attestations as OCI
referrers: the build's provenance, sluice's signed check verdicts for the
landed commit, and the test results. The same content-addressed store is
a build cache, an artifact repository and a release system, and the bytes
that passed the gate are the bytes that run. Your artifacts, in your
account.

- **The site:** https://agentplot.github.io/tailrace
- **The design book:** https://agentplot.github.io/blueprints/tailrace/
- **The statement**, tailrace's invariants as numbered clauses, is
  `design/tailrace/requirements.md` in agentplot/blueprints, being
  written now.

## What it does

- **push.** Packages what the build left, signs it, and pushes it to the
  registry by digest, into quarantine, with a machine-readable result
  that names the digest, the version and the attestations.
- **attest.** Attaches a signed statement to an artifact's digest as an
  OCI referrer: the fleet's or the host's build provenance, sluice's
  verdicts for the commit, a test run, a review.
- **publish.** Carries a verified artifact from quarantine into the
  artifact store by copying its digest, and points the `beta` channel at
  the version the landing made.
- **verify.** Checks an artifact's signatures and attestations against a
  policy and says which clauses it satisfies, without touching the store.
- **sync.** Fills each ecosystem's own local layout, npm, Maven, PyPI,
  Go, Cargo, NuGet, container images, CloudFormation templates and site
  bundles, from the store under a policy: a version is served only when
  its attestations satisfy it.
- **serve.** A localhost server speaking the read protocol of the
  ecosystems that need HTTP, npm first, over the same policy.
- **promote.** Moves a channel such as `stable` to a version by a signed
  promotion record in an append-only ledger; a rollback is a promotion to
  an older version.
- **status.** Says what each channel points at, what each environment
  runs, and what is waiting on a promotion.

One store at three levels of trust: a cache hit says someone built it, an
artifact says the merge queue built and signed it, and a release says it
was promoted with proof. Environments subscribe to a channel and a
reconciler converges them, push or pull. Trust is centralized only as a
thin layer of identity, policy and a signed ledger; execution stays on
the machines the agents run on.

The givens name Rust as its language, as sluice's. The first crate comes
with the first ADR.

tailrace is one of a family: headwater reports releases, eddy lists
meetings, millpond keeps the inbox, flume launches the work, sounding
works a question, sluice lands each change, and
[Flywheel](https://github.com/agentplot/flywheel-web) runs the teams
that make them. Headwater and eddy feed millpond, millpond feeds flume,
flume's agents submit to sluice, sluice lands, and tailrace delivers to
the environments. tailrace reads sluice's refs and moon's task hashes;
neither of them knows tailrace exists.

In a watermill the tailrace is the channel that carries water away from
the wheel after it has done its work. tailrace carries what the merge
queue landed out to where it runs.

## This repository

| path | what |
|---|---|
| `site/` | the site, Astro with Starlight (`site/README.md`) |
| `docs/architecture/core.md` | the rules every change here holds to |
| `docs/adr/` | the decisions behind the rules, one record each |
| `devenv.nix` | the shell: moon through proto, bun, and Playwright with its Chromium |
| `.config/kit.toml` | the kit standard's profile this repository follows |

```sh
devenv shell -- moon run site:build   # the site, with every internal link checked
devenv shell -- site-dev              # http://localhost:4321/tailrace/
```

## License

[MIT](LICENSE)

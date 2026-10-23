# tailrace

Delivers what the merge queue lands: versions, packages, signs and pushes
the build's output as OCI artifacts with attestations into a registry the
user owns, keeps channels and environments, and serves consumers under a
policy. The design is the statement of invariants in agentplot/blueprints,
`design/tailrace/requirements.md`; this repository cites its clauses by
number and never restates them. The design book is at
https://agentplot.github.io/blueprints/tailrace/.

The scaffold is built and nothing of the design yet (ADR 0011): five
crates as sluice's under `crates/`, the gate, the flake and the release
workflow. The language is Rust. Every cargo and moon command runs inside
`devenv shell`.

## Rules

The rules a change here must keep are in `docs/architecture/`. A design
or a commit that relies on a rule cites its id; one that departs from a
rule says which and why. Where `docs/architecture/` and any narrative
document disagree, `docs/architecture/` wins. The decisions behind the
rules are in `docs/adr/`, one record each.

@docs/architecture/core.md

## Layers

Run a repository tool through the shell per command,
`devenv shell -- <command>`.

| layer | what | about |
|---|---|---|
| `check-fast` | format, lint, the toolchain pins, the records' numbers, each crate's tests, the site's build: `moon run ':#check-fast'` | under a minute |

## The site

`site/` is the Starlight site at https://agentplot.github.io/tailrace,
published by `.github/workflows/pages.yml`. `site/README.md` says where
its pages and diagram live.

## Commits

Stage by pathspec and commit the paths you wrote: `git add <paths>`, then
`git commit -m "<subject>" -- <paths>`. Never `-a`, never `add .`.
Conventional Commits subjects, scope the part of the tree touched.

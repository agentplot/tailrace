# Proposal

## Why

The statement's givens name five surfaces of one core, among them a
herdr plugin with a pane of channels and environments, read from the
served page's own answer (section 9, 56, 132). The user said on
2026-10-26 that the host's served status was made for this pane (signal
`2026-10-26-madswan-planner-da2b5360/01`). Bolt 4 builds the host's one
serving process, which answers every repository the host knows at its
root and each repository whole at its own path (114, 132). Nothing yet
shows those answers beside a person's agents. This unit is that pane.

## What Changes

- **A herdr plugin under `plugins/herdr/`**, kept outside the core as
  sluice keeps its own herdr plugin (section 9, core.1). It is POSIX sh
  with jq programs, a herdr manifest (`agentplot.tailrace`), a README,
  and a moon project of its own (`plugin-herdr`) whose check-fast tasks
  check the manifest and run the scripts against stand-ins under sh and
  dash (core.25).
- **One pane, opened beside the person's work.** The plugin's action
  opens the pane as a split next to the focused pane. It first shows
  every repository the host knows, one row each: every channel's head
  version, and how many of its environments run what they should, are
  behind, wait on a person or cannot be read, or the answer given in
  their place. Opening a row shows that one repository: the branch and
  landed revision its policy was read at, each channel with its version
  and the chain of promotions that put it there, the environments, and
  what this host published. This is how sluice's popup moves between
  every repository and one. A key goes back to every repository. The
  pane asks again every few seconds while it is open.
- **Both views are read from the host's one serving process.** The
  plugin reads the root answer, `/status.json`, and one repository's,
  `/<owner>/<name>/status.json`, at the core's fixed name and port. No
  setting, flag or variable moves that address (114, core.36). The
  plugin renders what the process answers and computes no answer of its
  own: it never counts, never reads a head from a chain, and never reads
  a store, a ledger, a policy or git (56, core.31).
- **The pane starts the process when it finds none, as any asker
  does.** It asks the process to name itself. Where nothing answers, it
  starts `tailrace serve` once, detached and outside any checkout, and
  asks again. Where something other than tailrace answers, it says so
  and starts nothing. The process takes the host's one lock, so a second
  start ends at once. Two panes on one host find one process (114).
- **The four answers are kept apart.** A value, an absence, a refusal
  and an unreadable answer each read differently. Each carries the
  source's own words exactly as the process gave them (58, 59). In
  phase 1, every row's environment counts are unreadable, because no
  target is read yet (98). The pane says so and never shows them as
  zeros.
- **The documents the pane is tested on are the binary's own.** A test
  in the `cli` crate serves the root and a repository's answer for a
  scratch host on a machine that never published, with no port bound.
  It checks those answers against the documents the plugin's tests
  render. So the pane shows what `tailrace status --json` prints.
- **A proof against the real process at the core's address**, run by
  hand under a tag of its own. Two panes opened at once on a host where
  nothing answers find one process. It stays outside the gate, because
  the core's port is the host's own and a gate run must not take it
  from a process the host is running.
- **The site's surfaces page** marks the herdr plugin built and says
  what the pane shows.

Out of scope, and what each waits on:

- Environment counts that read something: every target is left to
  phase 2 (the register's entry for 132 names it). The pane renders
  whatever counts the process answers, unchanged.
- Each version's notes on the pages (132): the notes unit, which waits
  on the user. The pane shows notes once the answer carries them.
- Toasts, sidebar tokens, a tab-bar line or a dashboard tab, which
  sluice's plugin also has: neither the statement nor the bolt asks for
  them here.
- Naming a repository to the host from the pane: a repository appears
  once a tailrace command run in a checkout of it names it (132). The
  pane only reads.
- Promotion or any other act from the pane: the pane reads; a promotion
  is the gate's (core.14).

## Capabilities

### New Capabilities

- `herdr-plugin`: the herdr pane of every repository the host knows and
  of one repository whole, read from the host's one serving process,
  which it starts where none answers, rendering the core's answer and
  computing none.

### Modified Capabilities

None.

## Impact

- New: `plugins/herdr/`, with its manifest, scripts, jq programs,
  README, stand-ins, tests and `moon.yml`.
- `.moon/workspace.yml` gains the `plugin-herdr` project.
- `treefmt.nix`: the plugin's jq programs and fixture documents are
  excluded by name with their reasons, and its extension-less stand-ins
  are given to shellcheck and shfmt (core.25).
- `devenv.nix` pins `dash` and `curl` for the plugin's tests (core.26).
- `crates/tailrace/tests/herdr_documents.rs` (new) and
  `crates/tailrace/moon.yml`'s `test` inputs, which name the documents
  it reads.
- `site/src/content/docs/guides/integrations/surfaces.mdx`.
- The plugin reads the serving process that bolt 4 builds
  (`tailrace serve`, `/serving.json`, `/status.json`,
  `/<owner>/<name>/status.json`). The code stage needs bolt 4 landed on
  `main` and taken into this bolt.
- No crate, Rust dependency or core module is added. The plugin adds
  the bash language to the tree, as sluice's does.

## Touches

```
plugins/herdr/**
.moon/workspace.yml
treefmt.nix
devenv.nix
crates/tailrace/tests/herdr_documents.rs
crates/tailrace/tests/scratch/**
crates/tailrace/moon.yml
site/src/content/docs/guides/integrations/surfaces.mdx
```

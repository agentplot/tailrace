---
number: 26
title: The shell carries bun's own release build, whose node stand-in is the running user's
status: accepted
date: 2026-11-05
---

# The shell carries bun's own release build, whose node stand-in is the running user's

Status: accepted on the brief of the chore tailraces-shell-carries-bun-1-4-4-or-later, 2026-11-05, which decides it in its words: "tailrace's shell carries bun 1.4.4 or later as bun's own release build, fetched the way proto and sluice are, so bun run finds or makes a node stand-in of the running user's own and site:build passes under mac-studio's gate". The build's details below are the chore's own, cheap to change.
Sources: ADR 0008; rules core.25, core.26; bun's `src/install/lib.rs` and `src/runtime/cli/run_command.rs` at `bun-v1.4.2` and `bun-v1.4.4`; bun 1.4.4's `SHASUMS256.txt`, signed by bun's release key; nixpkgs' `pkgs/by-name/bu/bun/package.nix` at devenv.lock's pin; flume's and sluice's `devenv.nix` and `site/moon.yml` on main, 2026-11-05.

## Context and problem statement

The site (`site/`) builds with `bun run build`, which runs Astro's bin, a `#!/usr/bin/env node` script. Where `bun run` finds no `node` on PATH it stands one of its own in: a directory under `/private/tmp` (`/tmp` on Linux) holding a link to the running bun, put first on PATH. bun 1.4.2, the one nixpkgs holds at devenv.lock's pin, names that directory `bun-node-<sha>`, one for every user of the host, and uses it only when the running user made it; any other user's `bun run` gets no stand-in. mac-studio has a user of the gate's own, `sluice`, and on 2026-11-05 its `/private/tmp/bun-node-744846f84`, 1.4.2's directory, was the session user's, mode 0700. The brief reports `site:build` failing under mac-studio's gate; the run itself was not to be read from this chore. bun 1.4.4 names the directory `bun-node-<uid>-<sha>`, the running user's own. Which bun does the shell carry, and how is it held to one version on every host that runs the gate?

## Considered options

1. bun's own release build at 1.4.4, fetched by `devenv.nix` from bun's releases with one checksum per system, as proto, sluice and zot are; the Pages workflow's `setup-bun` pin moved with it; a `site:toolchain` check that the shell's bun is the Pages pin.
2. nixpkgs' bun, waiting for devenv.lock to reach a nixpkgs that holds 1.4.4.
3. The site's build runs Astro's bin with bun directly, as flume's site does, so no `node` is looked for.
4. The site's build runs under the shell's Node.js, as sluice's site does.

## Decision outcome

Option 1. It is the brief's. Options 3 and 4 work around the stand-in, and leave `bun run` and `bun install`'s lifecycle scripts broken for a second user of a host. Option 2 waits on another repository's lock for a fix that is already released.

- The build is bun's as released: on macOS no relink and no strip, so its signature stands; on Linux its dynamic loader is nixpkgs' glibc, as zot's is. x64 takes the baseline build, which asks for no AVX2, as nixpkgs takes for Linux.
- The checksums are the release's `SHASUMS256.txt`, whose signature by bun's release key (F3DCC08A8572C0749B3E18888EAB4D40A7B22B59, the key bun's own images assert) was checked before they were written in.
- `.github/workflows/pages.yml` restates the version, so `site:toolchain` fails while `bun --version` in the gate's shell differs from the Pages pin (ADR 0008's consequence, core.26).

### Consequences

- A bump of bun changes the version and the four checksums in `devenv.nix` and the pin in `pages.yml` in one commit.
- Where node is on PATH, as the shell's Node.js is, `bun run` uses it, and nothing of this record changes what runs.
- flume's site runs Astro's bin with bun because its gate's sandbox would not let 1.4.2 write its stand-in under `/tmp`, and sluice's site runs under Node.js. Whether either takes bun 1.4.4 instead is those repositories' to decide.

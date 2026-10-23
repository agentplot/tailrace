---
number: 11
title: The scaffold is sluice's, and the binary names its revision
status: accepted
date: 2026-10-23
decision-makers:
- Chuck Swanberg
---

# The scaffold is sluice's, and the binary names its revision

Deciders: the user, 2026-10-23, asking for the first unit of phase 1's bolt 1, the scaffold that lets crew give the repository a bolt (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`).
Sources: statement clauses 24, 56, 93, C.11, section 9; ADR 0008; rules core.1, core.2, core.25, core.26, core.31, core.33; the kit standard's gate.1 to gate.4 and release.3; sluice's `Cargo.toml`, `flake.nix`, `.github/workflows/ci.yml` and `.config/sluice.toml`.

## Context and problem statement

Nothing was built. crew gives a repository a bolt only when its `.config/sluice.toml` on `origin/main` names checks and a remote, and the statement's givens say one static Rust binary as sluice is, released the way sluice is. What shape does the first code take, and what does the binary say of itself when asked its version?

## Considered options

1. One crate, split later.
2. sluice's five crates from the first commit: seams, core, fakes, conformance and the binary, one moon project each, with sluice's gate, flake and release workflow copied and renamed.
3. A scaffold of its own shape.

## Decision outcome

Option 2. The seams are one crate because the core names no tool (core.1) and reaches each seam through one trait (core.2); the fakes and the conformance suite are crates because every profile passes the one suite (93, core.33); the binary is the first surface of one core (56, core.31). Each crate is a moon project with one task per check and the tools pinned in one place (core.25, core.26; ADR 0008). The gate is sluice on `main` and `bolt/*` with the fast layer required, main and the bolts on GitHub (gate.1 to gate.4). CI builds the flake, runs the fast layer in the devenv shell, and releases each push to `main` that git-cliff finds a new version in, as sluice's does (release.3).

`tailrace --version` prints the revision the binary was built from and the compiler and target that made it, never a version (24, C.11). The release's version is its tag's, carried by the flake's package metadata and the tarball's name, and `tailrace status` will say which version a revision became.

### Consequences

- The next unit of bolt 1 adds the first trait to `tailrace-seams` with the profile that implements it, and the conformance scenarios that hold it.
- A landing on `main` from here on goes through sluice; this scaffold is the last direct push.
- The release workflow runs on the first push to `main` after this and tags `v0.1.0`, a binary that does nothing but name itself; the site's roadmap says so.

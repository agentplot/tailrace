---
number: 26
title: A Nix closure is one artifact naming its roots and the fleet's cache holds its paths
status: accepted
date: 2026-11-05
decision-makers:
- madswan-design
---

# A Nix closure is one artifact naming its roots and the fleet's cache holds its paths

Status: accepted on 2026-11-06 by madswan-design, the design agent: it answers the statement's question of how a closure enters the store by clauses already written (C.12, I3), and what is left are choices cheap to change, which are design's.

Deciders: madswan-design, taking the choices of the the-nix-cache-target unit of phase 1's bolt 6, the Nix fleet pushed (`agentplot/blueprints:design/timber/models/phase-1-bolts.md`, Bolt 6). Its number is 0026 by agreement with the clan push profile's change, bolt 6's second unit, which takes 0027.
Sources: statement clauses 15, 42, 45, 52, 54, 88 to 91, 92, 119, 136, C.10, C.12 and section 10, as the statement has them at blueprints 2287669 (`design/timber/requirements.md`); the snix and casita note (`design/timber/models/snix-and-casita.md`, points 2, 4 and 5); ADRs 0002, 0009 and 0019; rules core.2, core.4, core.16, core.21, core.29, core.43 and core.50; the change's design, D2 to D9.

## Context and problem statement

No version timber held could reach a target. The target seam (B.7) had its trait, its fake and its conformance group and nothing more: the policy answered no environment (136), no binding held a target, no core path verified a version and handed it to a target (42, core.29), and no target profile existed. Bolt 6 starts with the Nix binary cache, C.12's row "push: the version's store paths pushed to the cache", which for swancloud is attic, so that a closure built on one host is realised on another from the cache alone.

Section 10 of the statement asks a question the model must answer: "How does a Nix closure, thousands of digests with references among them, map onto artifacts and the cache without a reference-aware removal (52, 54)?" Three more were open with it: how an environment names its target and its target's settings and how the core reaches that target (136, C.10, 92); how the cache is pushed with the environment's own credential and none of timber's (45); and what a target that can hold no marker answers for what it runs (119).

## Considered options

For how a closure maps onto the store and the cache (section 10):

1. A closure is one artifact naming its roots: a file of the closure's root store paths, one per line, which a machine's build task leaves and its component declares as an output, packed by the file projection; the store holds that artifact and no store path, and the paths and the references among them are the Nix binary cache's.
2. Each store path's NAR as an artifact or a cache entry.
3. A projection of a Nix ecosystem.
4. The roots with their NAR hashes in the artifact.

For how an environment names its target and how the core reaches it (136, C.10, 92):

1. The policy answers each environment with its target's profile, the one key of its `target` table, and that key's table carried as written; the binding holds every target profile built, keyed by profile, and the environment picks one; `desired` is handed the head the core read from the channel's chain.
2. The target's settings flattened beside `channel`.
3. The core reading `target` itself.
4. Each environment's target bound before the core runs, or targets keyed by component and environment name.
5. The core answering `desired` and never asking the target; the profile reading the ledger itself; the head carried as a field of the environment.

For how the cache is pushed (45, C.12):

1. `attic push <server>:<cache> <roots…>`, attic reading its servers and the environment's token from its own configuration on the host at the push; the table names `attic` and `closures` and nothing else.
2. `nix copy --to <store address>`.
3. A `token` or `token_file` setting in the table.

For a root the host that applies does not hold (15, I3):

1. Refused naming the root and the host, before anything is pushed.
2. Realised from another cache first, then pushed.
3. The roots the host holds pushed, and the rest left.

For what current and report answer of a cache (119):

1. Unread, saying the cache holds store paths by their hash and keeps no marker, and naming it.
2. The head, where the cache holds every root of its closures.
3. A marker kept as a store path of its own.

## Decision outcome

The closure is option 1 (section 10; 52, 54; design D6). A closure is one small artifact named by its digest, so its provenance and the landing's verdicts name it as they name any output, and the store holds nothing a removal of untagged objects (54) could leave half held: no store path of it is in the store, and nothing timber names depends on a path the cache may collect, since a machine that misses one builds it, as swancloud's `fallback` does today. The bytes are trusted as the snix and casita note reads it (points 2 and 5): timber's claims say which version, the cache's signature on each path says the bytes (12), and chunking a closure is below timber, as a compilation's cache is (133). Option 2 makes thousands of objects per version, removed one at a time by a store that knows nothing of references, so a closure could be left part held. Option 3 has no row in C.11 and nothing to pack but the roots. Option 4 asks for a NAR hash where the task runs, which a closure evaluated on one host and built on another, as swancloud builds srv's, does not have, and the cache signs the bytes it serves. Rule core.70 holds it for every Nix profile.

A closure's file holds one absolute store path per line, `<store dir>/<32 characters of Nix's base32>-<name>`, as `nix build --print-out-paths` prints them, the store directory `/nix/store` unless `NIX_STORE_DIR` names another; a file with no line, an empty or repeated line, or a line that is no store path is refused naming the artifact and the line. Its form is read by one module the Nix target profiles share, `profiles/target/nix_closure.rs`, beside the family's `mod.rs`, so the clan push profile reads the same artifact without calling this profile (core.2), with the profile's own code and no crate of Nix's, so the binary links no copyleft crate (the snix and casita note, point 4). The environment's table names which of the component's outputs are closures.

The environment and its target are option 1 (136, C.10, 92, 88; design D2 to D4). C.10 puts a target's settings in the target's table and nowhere else, and the policy profile is the only reader of the file's keys (core.43), so the policy answers them as written and the target reads its own from the policy's answer; option 2 contradicts C.10 and option 3 core.43. A `target` of no key, of two, or whose key names no table is unreadable naming the environment's table, in the same pass that resolves the channels, so channels and environments refuse a broken table alike. An environment with no `target` is listed with none, since its channel stands whatever runs it. Option 4's first half would have the bindings read the policy at the landed revision, which only the core does (core.22); its second binds a profile per environment for no gain, and an environment's name is unique only within its component (125), so a map by name cannot hold swancloud's cache environments, one per machine's component. `desired` stays the target's (88): a push target answers the head it is handed, read from the chain (core.50), and a pulling target, the Nix fleet's pull profile in phase 2, answers its own reading of the channel (44). Option 5's three paths would take 88 from the target, have a profile reach a second seam and parse promotion claims, or read the head for every environment made, asked or not.

The core applies one environment in this order, and judges before it hands over (42, core.29; design D5): the governed branch's landed head and the declared component, as promote reads them; the environment at that revision, its target table and its bound profile, each absent naming what is missing; the channel's chain, an empty one absent naming the channel with the target asked nothing (31); `desired` and `current`, a desired other than the chain's head refused naming both and nothing applied (40, core.16), and no apply where current answers the head; the claims the channel's policy requires of the head's index judged at the landed revision, refused naming the first found wanting with the target asked nothing; then the version, verified, handed to `apply`, and `report` read after. Apply moves no name and makes no store. `timber apply <environment>` renders that answer (core.31).

The push is option 1 (45, core.21; design D7). The fleet's cache is attic, which takes an upload through its own client and reads its own credential at the push, so the token stays in the environment's own place, the host's attic configuration, and timber passes none on the command line or in the environment, sets none of attic's variables and reads nothing of its configuration. Any key of the table besides `attic` and `closures`, an empty `closures`, or an `attic` that is not `<server>:<cache>` is unreadable naming the environment and the key, at every operation that reads the settings, so a token can be written nowhere timber reads. attic's own words on a failure are the apply's answer, unreadable, verbatim (59). Option 2 serves any binary cache but not the fleet's, and is a later key of the same table when a fleet binds a cache attic is not. Option 3 puts a credential in the repository's file, or one timber reaches for the target.

A root the host does not hold is option 1 (15, I3; design D8). The apply reads the version's index and each closure's file through the store by digest (42, core.4) and checks every root valid in the host's Nix store before attic runs; the host that applies is a host that built, the deployer's host for the Macs' closures, srv for srv's. Option 2 fetches what the apply would push straight back, and on a fresh host builds it; option 3 leaves an environment half converged with no marker to say so.

current and report are option 1 (119, 89, 98; design D9). A Nix binary cache holds store paths by their hash and their narinfo and no name that can be moved, so there is nowhere to leave a marker of the digest applied. Apply runs whenever it is asked, and attic skips what the cache holds, so an apply of a version already pushed uploads nothing. Option 2 says what the cache holds, not what was applied, and current is handed no head; option 3 cannot be found by a name, since a store path is named by its content. The target group's rig declares whether its target holds a marker, as a store's rig declares whether its names take a condition (core.49): "converged and reported" runs where it does, and "applied where no marker is held" where it does not. Every target reads no setting but its own, held by "a table with a key the target does not read" (45, 58; design D10).

### The construction's own choices

Choices made in building it that the design does not fix, each cheap to change, with the path not taken:

- The policy file carries a target's settings into the policy's answer with a TOML date or time as its text, and a NaN or an infinity unreadable, since the answer's form has neither. Not taken: refusing every date and time.
- A chain the ledger answers as last read makes apply unreadable naming the chain, with the ledger's words, and the target is asked nothing, since an apply from a stale head could roll the environment back (60, in the spirit of core.45). Not taken: applying from the kept head.
- A desired other than the head is refused with a reason of its own, naming the channel, the head and the desired digest. An environment that already runs the head is not judged, since nothing is applied.
- The closure's form finds the file projection's artifact by the ecosystem `file` written in its own module, not imported from that projection, since no profile imports another (core.2). A store path's name is checked as current Nix checks one: at most 211 characters, only `A-Z a-z 0-9 + - . _ ? =`, and neither `.` nor `..` alone or before a `-`; a leading `.` is accepted, which Nix 2.3 refused. One trailing newline ends the last line and is no empty line, and a line that is not UTF-8 is no store path. An empty `NIX_STORE_DIR` counts as unset, a trailing `/` is dropped, and `NIX_STORE` is not read.
- The roots are checked by one `nix-store --check-validity --print-invalid` run, whose exit 0 lists the invalid paths; a non-zero exit is unreadable in nix-store's words. Not taken: a run per root, reading "not valid" out of its error text. The roots are deduplicated across closures and keep their order.
- An `attic` value is refused where its server is empty or begins with `-`, which attic would read as an option, where its cache is empty, or where it has a second `:`. A closure named twice in `closures`, an empty name, or a value that is no string is unreadable.
- The host a refusal names is read with `gethostname` in the profile's own code, duplicating the signer profile's, since no profile reaches another (core.2); where it cannot be read the refusal says so in its place.
- The index read by digest must name the version verified, and is refused otherwise.
- The proof keeps attic's configuration to attic's process alone, through a wrapper the test hands the profile that sets `XDG_CONFIG_HOME` and runs attic. Not taken: setting it in the test's own environment, which would hand it to timber too.

### Consequences

- An environment names its target by its profile, `target.nix_cache = { attic = "<server>:<cache>", closures = [<output>…] }`, and a machine's build task leaves a closure file declared as an output, packed as a file.
- `timber apply <environment>` pushes a version's closures from the host that built them, with the token in attic's configuration on that host; a host that lacks a root is refused naming it and itself.
- A cache environment's current and report are always unread, by design; whether a machine realises its closure is the clan push profile's report.
- The store holds one small artifact per closure and none of its paths, so the store's removal of untagged objects never touches a closure's paths, and the cache's collection, by age, is attic's and the fleet's; a closure it has collected is built again by the machine that misses it (54).
- The clan push profile, bolt 6's second unit, takes the environments, the binding by profile, the core's apply and the closure's form from this change and writes none of them again; it adds its fields to apply's answer.
- Another pusher for a Nix binary cache, `nix copy` to a store address, is a later key of the same table.

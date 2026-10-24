# Design

## Context

The binary's `main` calls `tailrace::run(&cli, None, &at, &mut out)`:
no binding, so `publish` and `verify` answer `unbound()`, absent naming
every seam, and `status` reads the host's record (`crates/tailrace/src/lib.rs`).
`run` takes `Option<&Binding>`, where `Binding` is `{ seams: Seams,
signer: SignerId }`; the surfaces conformance test hands it a binding of
fakes (`crates/tailrace/tests/conformance.rs`).

`Seams` (`tailrace-seams/src/bind.rs`) holds one store, build, landing,
policy and keeper, the signers, projections by ecosystem and targets by
environment. What exists to fill it:

| seam | profile | made by |
|---|---|---|
| store | `profiles::store::layout::Layout` | `Layout::create(path)` for a missing or empty directory, `Layout::open(path)` otherwise; open answers unreadable when the layout is not there |
| build | `profiles::build::moon::Moon` | `Moon::new(repository)`; it reads each output's task from the policy file at the commit |
| landing | `profiles::landing::sluice::Sluice`, `profiles::landing::branch::PlainBranch` | `::new(repository, remote: Option<String>)` |
| signer | `profiles::signer::host_key::HostKey` | `HostKey::new(signer, key_path)`; verify reads no key file, sign reads it at the moment of signing |
| policy | none but the fake | |
| projection | none but the fake | |
| keeper, target | none but the fakes | |

The core reads the components, the rule and each output's ecosystem
from `.config/tailrace.toml` itself (`tailrace-core/src/declared.rs`,
`[projections]`), and asks the policy seam only for an audience's
claims, after the landing seam's yes (`policy_at`). publish never asks
the policy seam or the keeper; verify asks the policy seam once and
every bound signer in turn for each envelope (`tailrace-core/src/verify.rs`).
A provenance carries the landing's verdicts as made; verify tells each
carried verdict by asking the same signers.

sluice keeps its host settings in `$XDG_CONFIG_HOME/sluice/config.toml`
(default `~/.config`), TOML, with `[signing] signer` and `key` naming the
host's signer and its SSH key, `~` expanded (sluice
`crates/sluice/src/instance.rs`). Its policy names each signer as
`[signers.<name>]` with one identity key, such as `ssh = "…"`, beside
the claims it may make (sluice `crates/sluice-core/src/policy.rs`).

The statement settles where each binding comes from: the operator binds
the store, the signers, the landing and the ledger for a host, and the
authors bind the build, the projections and the targets in the
repository (92); the operator's bindings are one file beside the
landing's own, in the same form (118); the policy is a file in the
repository (C.10).

## Goals / Non-Goals

**Goals:**

- `tailrace publish` and `tailrace verify` bound by the host's file and
  the repository's policy file, with every failure one of the three
  answers.
- The policy seam's profile over the file, so verify can name signers.
- The file and image projections' pack.
- S1 and S2 from the built binary over real sluice and moon, and a
  second clone verifying a copied layout, in the gate's tools layer.

**Non-Goals:**

- Choosing a binding per repository on one host: one host, one store,
  one landing kind, one signer (92). A repository whose landing differs
  is a host file away.
- A command that writes the bindings file or makes a key.
- Render and serve for the two projections; a media type per file.

## Decisions

### D1. The file: `$XDG_CONFIG_HOME/tailrace/config.toml`, one key per seam

The path is sluice's with `tailrace` for `sluice`: `$XDG_CONFIG_HOME`
when set and not empty, else `$HOME/.config`. That puts it beside
sluice's own `config.toml` under one directory, in the same form, TOML
(118). Each seam's table names its profile by its one key, the way
sluice names a signer's identity by its one key, so a later profile is a
new key and not a new shape:

```toml
[store]
layout = "~/.local/share/tailrace/layout"   # C.2

[landing]
sluice = { remote = "origin" }   # C.5; or: branch = { remote = "origin" }, C.6
                                 # remote optional; without one, local refs

[signing]                         # sluice's own table, C.8
signer = "laptop"
key = "~/.config/tailrace/signing"
```

A table with no key, or two, is unreadable naming it. Any other
top-level table is unreadable naming it: `[build]` and `[projections]`
because the repository binds them (92), `[ledger]` because no ledger
profile is built, anything else because it is unknown. A path is
absolute or begins `~/`, expanded against `$HOME`; any other is
unreadable naming the key, since a relative path would mean another
file on each working directory. The file is read with `std::fs` and
parsed with the `toml` crate already in the workspace; the parser's
words are carried verbatim (59).

*Alternatives:* a `tailrace.toml` inside sluice's directory, which is
sluice's to own; a flat `store = "layout:<path>"` string per seam, which
sluice's file does not use; `$TAILRACE_CONFIG`, which no family tool
reads and 118 does not need.

### D2. What a binding holds, and where each part comes from

`crates/tailrace/src/bindings/` reads the file and makes the `Binding`
(the module the bolt plan gives this unit):

| seam | bound to | source |
|---|---|---|
| store | `Layout` at `[store] layout` | the host's file (92) |
| landing | `Sluice` or `PlainBranch` at the repository, with the remote | the host's file (92) |
| signers | `HostKey::new(signer, key)` when `[signing]` is named, else `HostKey::verifying()` (D6) | the host's file (92) |
| build | `Moon::new(repository)` | the repository: moon is the one build profile, and the policy file names each output's task (92, C.3) |
| policy | the policy file profile at the repository (D3) | the repository (C.10) |
| projections | `file` and `image`, keyed by those ecosystem names | the repository: the policy file's `[projections]` picks each output's (92, C.10) |
| keeper | a keeper answering absent `binding of the ledger` for both operations | none until bolt 4 |
| targets | none | none until phase 2 |

`bindings/` is outside `profiles/`, so it names no tool (core.1): it
reads each table generically, its one key and value, and hands them to
a `bind` function of that seam's profile family, in
`profiles/store/mod.rs`, `profiles/landing/mod.rs` and
`profiles/signer/mod.rs`, which matches the key (`layout`, `sluice`,
`branch`) to its profile and answers unreadable naming a key it does
not know. The repository's own bindings come the same way:
`profiles::build::bind(repository)`, `profiles::policy::bind(repository)`
and `profiles::projection::bound()`, the map of every built projection
by ecosystem. The words that name tools stay under `profiles/`.

The repository is the one the command runs in (`at`). `Binding.signer`
becomes `Option<SignerId>`: publish with none answers absent naming the
`[signing]` binding before anything is read, and verify needs none.

`run` takes `&Answer<Binding>` in place of `Option<&Binding>`, so a
missing or broken file is rendered by the command that needed it, as
text or with `--json`, with its exit code (58); `status` ignores it.
`main` loads the binding for publish (may make the layout) or verify
(never makes it), and for status loads nothing, so a broken file never
breaks status. The surfaces conformance test passes `Ok(binding)`.

*Alternative:* the host file naming moon, as the bolt plan's table
reads. Rejected: 92 gives the build to the authors, and a build named by
the host would bind one repository's build for every repository on it.

### D3. The policy file profile: sluice's `[signers]`, an ordered `require` per audience

`crates/tailrace/src/profiles/policy/file/` implements `Policy` and
nothing of the core (core.2). It reads `.config/tailrace.toml` from the
revision's tree with git (`git cat-file`, through `profiles/run.rs` as
the landing profiles do), never a working tree (65, core.22), and reads
only its own tables, leaving the core's (`outputs`, `[version]`,
`[projections]`, `[[component]]`) alone:

```toml
[signers.laptop]
ssh = "ssh-ed25519 AAAA… host@laptop"   # verified_by: host-key D5's listing
kinds = ["provenance"]

[signers.box]
ssh = "ssh-ed25519 BBBB…"
kinds = ["verdict"]
checks = ["cli:test", "core:test"]       # by the landing's names (116)

[acceptance]
require = [
  { kind = "provenance", signers = ["laptop"] },
  { kind = "verdict", check = "cli:test", signers = ["box"] },
]

[layout]
require = [{ kind = "provenance", signers = ["laptop"] }]

[channels.stable]
bound_to = "beta"
require = [{ kind = "acceptance", signers = ["gate"] }]
```

A signer's verifier is the one key that is not `kinds` or `checks`, its
string value passed whole as `NamedSigner.verified_by`, so each signer
profile reads its own (a host key passes over another's listing,
host-key D5). Claim kinds are `ClaimKind`'s snake-case names. `require`
keeps its order, which fixes the first claim found wanting (58).
Absent naming the audience where the file or the audience's table is
missing (82); unreadable naming the file, the revision and what is
wrong otherwise. A `[channels.<name>]` table's keys other than
`require` and `bound_to` are left to the reader that will own them
(bolt 4), as the core leaves this profile's.

*Alternatives:* the reconstruction's YAML-like `predicate from [signer]`
lines, which no TOML reader keeps; signers listed inline in each
`require` entry, which repeats a key per audience and lets two entries
disagree about one signer.

This is the policy seam's one profile, as C.10 is its one member; ADR
0013 says why one is enough (core.3).

### D4. The file projection: one file, its own bytes

`crates/tailrace/src/profiles/projection/file/`. pack reads the
output's one file (`Output.files`, which moon reports as paths in the
checkout) and answers its bytes unchanged, metadata `file`, the
output's name and the version, `same_again: true`. An output reporting
no file or several is refused (`Why::Said`) naming the output and the
files; a read error is unreadable with the operating system's words.
The name is the output's, which the authors declare, and not the file's
own name, which the build chose.

A directory, such as a site, is published by a task that leaves an
archive: tailrace builds nothing (15, non-goals), and "a file" in C.11
is one file. render and serve answer absent naming the operation.

*Alternative:* a tar of a multi-file output written by the projection.
Rejected: it would be tailrace building, with its own determinism to
keep, for bytes no check ran over.

### D5. The image projection: an OCI image archive, its own bytes

`crates/tailrace/src/profiles/projection/image/`. "An image as pushed"
(C.11) is, from a build, an OCI image layout in a tar: what `docker save`
writes since Docker Engine 25, and what `skopeo copy … oci-archive:`
and buildah write. pack checks the one file is such a tar by reading its
member names from the POSIX ustar headers (512-byte blocks, the name in
the first 100 bytes and the `prefix` field, the size in octal) until it
has seen `oci-layout` and `index.json` at the root, and answers the
file's bytes unchanged, metadata `image`, the output's name and the
version, `same_again: true`. No crate is added: two names at the root
need no more of tar than the header. A file that is not a tar (gzip's
magic, a short block) or a tar without both names is refused naming the
output and what was missing; a legacy docker archive (`manifest.json`
alone) is refused naming `oci-layout`. render and serve answer absent.

The layout store holds the archive as one layer of an OCI artifact
whose `artifactType` names `image` (C.2, the layout store's `oci.rs`),
not as an image manifest of its own: rendering an image as an image is
the image's render, phase 2's.

*Alternative:* accept docker's legacy archive too. Rejected for now: the
family's images come from tools that write OCI archives, and a second
form is a second thing to keep same-again.

### D6. A host key that only verifies

`HostKey::verifying()` makes the profile with no signer and no key path:
`verify` is unchanged (it never read the key), and `sign` answers absent
`no identity is at hand` for every identity, touching no file (79). One
constructor and an `Option` inside the profile; nothing else of it
changes. A second machine binds it when its file names no `[signing]`.

*Alternative:* require every verifying host to name a key. Rejected: a
reader is not a signer, and 64 asks that nothing hold a key it does not
use.

### D7. The suite: a host group citing 118; the projections' pack scenarios

- A new group, `host`, in `tailrace-conformance/src/seams/host.rs`, with
  a rig that names the config directory, writes a bindings file there
  naming a store, a landing and a signer, and answers what the binding
  made from it binds. Its one scenario, "the host's bindings beside the
  landing's", cites 118: the file is `tailrace/config.toml` under the
  same directory as `sluice/config.toml`, reads as text with
  `std::fs::read_to_string`, and the binding binds the store, landing
  and signer it names. `crates/tailrace/tests/` runs it over
  `bindings/`; 118 leaves the register in the same commit (core.33).
- The policy group runs against the policy file profile over a scratch
  repository, in `check-fast` (git only).
- The projection group's two pack scenarios run against the file and
  image projections. The render and serve scenarios do not: neither
  projection renders. **This departs from core.33** ("every profile
  passes the one conformance suite"): the bolt plan gives this unit
  pack alone (C.11's file and image render are phase 2's, and render
  under a policy is bolt 5's), and a render built now would be built
  before the unit that decides where render reads bytes from. The
  departure ends with the unit that builds their render. Named for the
  design agent in the open questions.

### D8. The proofs: the built binary over real sluice and moon

`crates/tailrace/tests/laptop.rs`, in the tools layer (`--features
tools`, added to `cli:tools`), runs `env!("CARGO_BIN_EXE_tailrace")`
as a subprocess with `XDG_CONFIG_HOME` and `HOME` in the scratch
directory:

1. The scratch harness's sluice gate (`tests/scratch/gate.rs`) lands a
   revision of a repository whose moon tasks leave `dist/notes.txt` and
   `dist/image.tar` (a committed fixture archive copied by the task) and
   whose policy file declares both outputs, `[projections]` naming
   `file` and `image`, `[signers.laptop]` with the test key's public
   half, `[signers.box]` with the gate's, and `[acceptance]` requiring
   provenance from `laptop`.
2. A bindings file names a fresh layout path, `[landing] sluice` with
   the remote, and `[signing]` with a key made by `ssh-keygen` (mode
   0600). `tailrace publish <revision> --json` answers S1: the version,
   the index, two artifacts, a provenance on each, sluice's verdicts
   carried; the layout's named entries are the version's alone.
3. The same command again answers the same index, and a listing of
   every file under the layout and the record, with sizes and bytes'
   digests, is unchanged (S2).
4. A second clone of the remote, a recursive copy of the layout to
   another directory, and a second config directory whose file names
   the copy and sluice and no `[signing]`; `tailrace verify <version>
   --json` there tells every provenance as made by `laptop`, counting,
   and each carried verdict with its `carried_in`.

The same run by hand, with the binary `nix build` makes from the flake
(the derivation an install takes), against a family repository that
lands through sluice and declares a file output, and a second machine
given a copy of the layout and a clone: the gate cannot reach a second
machine or a family repository's remote.

## Risks / Trade-offs

- [The policy file's signer and audience form is chosen here, and the
  design agent may choose another] → It follows sluice's
  `[signers.<name>]` and the host-key profile's listing (D5 of that
  change); the profile is one module, so a different form changes it and
  its tests alone. Named in the open questions.
- [sluice's carried verdicts never count at verify, since no bound
  profile verifies sluice's run-record signatures] → The proof's policy
  requires provenance only, and verify reports each verdict as carried.
  The question is already with the design agent.
- [An image archive's determinism is its builder's] → The projection
  copies bytes; same-again holds for the bytes the build left, and a
  rebuild that differs is a new build identity the landing's verdicts
  must name (15).
- [Hand-read tar headers miss an exotic archive] → Only the two root
  names are looked for; a pax or GNU long-name header before them is
  skipped by its size like any member, and an archive that hides them
  is refused, never accepted.
- [A host with repositories landed differently] → One landing per host
  (92); the error names the landing bound, so the mismatch is visible.

## Open Questions

- For `madswan-design`: is the policy file's form in D3 the one C.10
  means, `[signers.<name>]` with one verifier key and an ordered
  `require` per audience? The host-key signer change asked the same of
  how a signer is listed; this change answers it provisionally.
- For `madswan-design`: core.33 and a projection whose render a later
  unit builds (D7). Either the projection group gains a way for a
  profile to say render is not yet built, or the unit that builds a
  projection's pack also builds its render.
- For `madswan-design`: C.11's "a file, with a declared media type" has
  no place in `ArtifactMeta`; the layout writes every layer as an octet
  stream until a unit changes the shared type.

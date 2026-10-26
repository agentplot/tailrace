## Context

The projection seam (B.6, `crates/tailrace-seams/src/projection.rs`) has
pack, render, serve and pinned. Two units this change comes after shape
what it implements:

- **render under a policy** adds pinned (137) and the core's render in
  `crates/tailrace-core/src/render.rs`: it reads a checkout's pins,
  resolves each with `locate` through `artifact/<ecosystem>/<name>@<version>`
  to the version's index and the artifact found in it by its metadata
  (138), judges it under the layout's policy (113, C.10), and hands the
  projection exactly the satisfied artifacts at
  `<working tree>/.tailrace/layouts/<ecosystem>/`; `tailrace sync` runs it
  and answers every refused pin naming the version and the claim found
  wanting (99).
- **npm, rendered and served** (its design, D1 to D4, D10, D11) makes
  serve answer one request:

  ```rust
  pub struct Request { pub method: String, pub path: String, pub base: String }
  pub struct Response { pub status: u16, pub content_type: String, pub body: Vec<u8> }
  pub type Miss<'a> = &'a dyn Fn(&Pinned) -> Answer<()>;
  fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response>;
  ```

  `path` is below the layout's prefix, `base` the address of that prefix
  as the client reached it with no trailing `/`, and `miss` asks the core
  to render one version into the layout under the policy. The host's one
  process routes `/<owner>/<name>/<ecosystem>/<rest>` to the projection
  bound for that ecosystem in the checkout that last named the
  repository; `tailrace sync` probes each layout's serve with `path`
  empty and a miss that answers absent, reading `Err(absent "serving")`
  as serving none, and starts the process where one serves; the process
  stays while a served layout exists (114).
  `served::layout_address(repository, ecosystem)` spells
  `http://127.0.0.1:7420/<owner>/<name>/<ecosystem>/`.

Built here already: the profile, its binding with a store of its own,
render writing a crate per version and an index file per name, the index
line, the race check after pack, and the conformance rig.

C.11's Cargo row, at blueprints `d91401e`: pack the crate file with
`cargo package --no-verify` after the component's version is set on every
crate of the component, a path dependency on a sibling rewritten to that
version's requirement as cargo does (110); a crate belongs to the
component whose paths hold its manifest (123), and a path dependency on a
crate of another component is packed as a dependency on the version
requirement the manifest writes for it, refused naming the crate where
the store holds no such version (138); render a sparse registry under the
served address (49, 114), named `tailrace` in the checkout's cargo
configuration; serve from the host's one process. 138 says the store
lists the versions a name holds, a registry by its tags and a layout by
its index's names; 99 says a sparse-index client's refusal reaches it by
omission from the listing, by the bytes' answer and by sync's words.

Vendor facts, checked against cargo 1.98.1 (the devenv's) and Cargo's
reference (registries, registry index, source replacement, `cargo
package`):

- `cargo package` writes every member with the fixed time 2006-07-23 and
  owner 0/0; two packs of one tree gave identical bytes. In a git
  checkout it writes `.cargo_vcs_info.json` with the HEAD sha1 and
  `path_in_vcs`, the package's directory in the repository, and refuses
  an edited manifest without `--allow-dirty`. It writes the crate's own
  `Cargo.lock`, naming every dependency it resolved with its source and
  checksum.
- A path dependency with no `version` is refused when packaging ("all
  dependencies must have a version requirement specified when
  packaging"); with one and no `registry`, the packed crate names it
  from crates.io.
- With `registry = "tailrace"` beside `path` and `version`, packing the
  crate offline fails to resolve the path dependency from the registry
  ("no matching package named `other` found, location searched:
  `tailrace` index"). Packing it together with a sibling, `-p a -p sib`,
  resolves the sibling from cargo's overlay of the crates being packed,
  but resolves nothing else.
- Source replacement keyed by the registry's index URL works for an
  alternative registry: `--config 'source.held.registry="<tailrace's
  index>"' --config 'source.held.replace-with="pack"' --config
  'source.pack.local-registry="<dir>"'`, where `<dir>` holds `index/` in
  the index's layout and `<name>-<version>.crate` beside it. Packed so
  with `--offline`, a crate depending on `other = { path, version = "0.3",
  registry = "tailrace" }` and on a sibling, each held in `<dir>`, packs:
  its manifest names `[dependencies.other] version = "0.3"
  registry-index = "<tailrace's index>"`, the original address and not
  the replacement, and its `Cargo.lock` names `other 0.3.1` with that
  source and the held crate's checksum. With the replacement, cargo's
  overlay of crates packed together is not consulted ("no matching
  package named `sib` found … (which is replacing registry `tailrace`)"),
  so siblings go into `<dir>` too, packed one at a time. A leaf crate
  packed alone with the replacement and without it gave the same bytes.
  `[source.tailrace]` with only `replace-with` is refused ("no source
  location specified"); the source must name the URL.
- A sparse registry: Cargo fetches `<index>/config.json` (`dl`, the
  download template with `{crate}` and `{version}`; `api` absent for a
  registry that takes no publish), then each name's index file at
  `1/<n>`, `2/<n>`, `3/<c>/<n>` or `<ab>/<cd>/<n>`, the name lower-case,
  then each crate at `dl`. A 404 on an index file is "not found". Cargo
  resolves from the index file before it asks for any crate, a locked
  build too, so a version missing from the index fails resolution
  without a download request. A non-success on a download fails the
  build printing `got <status>`, then `body:` and the body. An index
  line's dependency with `registry` null is of the registry serving the
  line. Run against a test server, `cargo build` resolved two crates so
  and wrote `source = "sparse+<index>"` and each crate's SHA-256 as
  `checksum`, and `cargo build --locked` refuses where a checksum
  differs.
- `Cargo.lock` is TOML at the workspace root: `version` (4 today) and
  one `[[package]]` per package, with `name`, `version`, `source` and
  `checksum`. `source` is `registry+<url>` or `sparse+<url>` for a
  registry, `git+<url>#<sha1>` for git, and absent for a workspace member
  or a path dependency.
- The OCI distribution specification lists a repository's tags at
  `GET /v2/<name>/tags/list`, answering `{"name", "tags"}`, paged with
  `n` and `last` and a `Link: <…>; rel="next"` header while more remain;
  a repository the registry does not know answers `404` with
  `NAME_UNKNOWN`.

## Goals / Non-Goals

**Goals:** pack, pinned, render and serve for Cargo on the seams the
units above land, held to the whole projection group; a crate's
siblings and its other components' crates told apart by the component's
paths; the store's listing on every store; `cargo build --locked` proven
against the host's serving process after `tailrace sync`.

**Non-Goals:** choosing versions, the policy, the layout's place and
`tailrace sync` (render under a policy); the process, its route and its
lifetime (npm, rendered and served); writing the
`artifact/<ecosystem>/<name>@<version>` pointer (the store-naming unit);
writing a checkout's cargo configuration; a manifest's range where no
lockfile pins a name (137).

## Decisions

### D1. The output names its package by the crate file it left

The build output a family repository declares for a crate is its
packaging task, and the output's files hold the `.crate` file the task
left. The projection takes the one file among them ending `.crate`,
reads its normalized `Cargo.toml` and its `.cargo_vcs_info.json` with
`flate2` and `tar`, and packs the package of that name whose manifest is
`<checkout>/<path_in_vcs>/Cargo.toml`, checking with `cargo metadata
--no-deps --format-version 1 --manifest-path <it>` that the package
there has that name, and reading the workspace's root and members from
the same answer. None or several crate files are refused naming the
output and its files; a crate with no `path_in_vcs`, packed outside git,
is refused naming the crate. Alternative: a `Cargo.toml` among the
output's files; refused because the design declares the output as the
packaging task that leaves the crate file.

### D2. What pack is given, and the store's listing

The seam's pack is given, where the bytes are an artifact's, what it
packs at:

```rust
/// 84, 110, 123, 138: what an output is packed at as an artifact.
pub struct PackAt<'a> {
    /// The component's one version (110).
    pub version: &'a Version,
    /// 123: whether the component's paths hold a path of the repository,
    /// matched as the version rule matches them.
    pub holds: &'a dyn Fn(&str) -> bool,
    /// 138: the artifacts of a name, in the projection's ecosystem, that
    /// the store holds, each found by its metadata in its version's index.
    pub held: &'a dyn Fn(&EcosystemName) -> Answer<Vec<Artifact>>,
}
fn pack(&self, output: &Output, at: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed>;
```

The core builds both closures: `holds` from the component's declared
`paths` with the same `glob` patterns the version rule gives git-cliff
(123), and `held` from a new `render::held(store, ecosystem, name)`
beside `locate`, which lists the names under
`artifact/<ecosystem>/<name>@`, takes each version from after the `@`,
and locates each (138). A name the store holds no version of answers an
empty list; a version whose index names no one artifact of that name is
unreadable naming it, as `locate` answers. Every projection but Cargo's
reads `version.version` alone; the cache push packs with `None` as now.
Why closures: the component's paths and the store's names are the core's
(core.2, core.22); the profile asks and matches nothing of either.
Alternative: hand pack the component and the store; refused because the
profile would then match globs and resolve pointers, which are the
core's.

The store seam gains one operation:

```rust
/// 138: every name the store holds that begins with `under`, in no
/// order a reader relies on (23); none where it holds none.
fn names(&self, under: &Name) -> Answer<Vec<Name>>;
```

- The registry profile lists its repository's tags at `tags/list`,
  following `Link` while it names a next page, reads each tag back to
  the name it holds (the inverse of `tag::of`: `_` and two hex digits a
  byte, everything else itself; a referrers tag or one that is no
  name's tag skipped), and answers those beginning with `under`. `404`
  `NAME_UNKNOWN` answers none, since the repository holds nothing; any
  other failure is unreadable with the registry's words (core.19).
- The layout store answers the ref names of `index.json`'s entries that
  begin with `under`.
- The fake store answers its names map's keys that begin with `under`.
- Every store a test wraps delegates or answers as its test needs.

138 says a registry lists the tags of `artifact/<ecosystem>/<name>`; the
built registry profile holds every name as a tag of its one declared
repository, so it lists that repository's tags and keeps those under the
prefix. Named for the design (Open questions).

### D3. The component's version on its own crates, packed one at a time

From the packed package, the projection follows each dependency in
`dependencies`, `build-dependencies` and each `target.<cfg>` table of
them that names a `path`, written in the package's manifest or
inherited from `workspace.dependencies`, and so on from each crate it
reaches through a dependency of its own component. A crate reached is a
**sibling** where `holds` answers true for its manifest's path relative
to the checkout, the component's own (123); otherwise it is **of another
component**. Dev dependencies are left as written: cargo drops a path
dev dependency with no version from the packed crate.

With `toml_edit`, each edit in the file and the table where its value is
written, the rest of every file keeping its bytes:

- the version is set on the packed package and every sibling, in
  `package.version`, or in `workspace.package.version` for one that
  writes `version.workspace = true`;
- each dependency on a sibling gets `version = "<version>"`, the
  requirement Cargo reads a bare version as, and `registry =
  "tailrace"`;
- each dependency on a crate of another component keeps the `version`
  it writes and gets `registry = "tailrace"` where it names none; one
  that writes no `version` is refused naming the crate and the
  dependency, since cargo cannot pack it and the statement packs it on
  the requirement the manifest writes (C.11).

Then the crates are packed through a scratch local registry, so the
crate's own `Cargo.lock` resolves every family dependency from what the
store holds or what this pack made:

1. For each crate of another component, `held(name)` is filtered to the
   versions within the manifest's requirement, read with `semver`'s
   `VersionReq`, Cargo's own. None within it refuses the pack naming
   the crate, the requirement and the versions the store holds (C.11).
   The crates within it are written into a fresh temporary directory by
   the profile's own render (D6), which reads each through the store and
   checks its digest; each one's own dependencies on a served Cargo
   address are added the same way, until none is missing.
2. The siblings are packed in dependency order, then the packed package,
   each by `cargo package --no-verify --allow-dirty --offline --registry
   tailrace --manifest-path <workspace root>/Cargo.toml -p <one>
   --target-dir <temp>` in the package's directory, with `--config`
   `source.tailrace-held.registry = "<tailrace's index>"`,
   `source.tailrace-held.replace-with = "tailrace-pack"` and
   `source.tailrace-pack.local-registry = "<the directory>"`; each
   sibling's crate and index line are added to the directory before the
   next is packed.
3. The answer is `<temp>/package/<name>-<version>.crate`; the siblings'
   crates are left, each answered by its own output's pack, which makes
   the same bytes (vendor facts).

`<tailrace's index>` is `registries.tailrace.index` as cargo would read
it for the package: `CARGO_REGISTRIES_TAILRACE_INDEX` where set, else
the first `.cargo/config.toml` or `.cargo/config` naming it from the
package's directory up to the checkout's root, then `$CARGO_HOME`'s. None
is refused naming the registry `tailrace` and the files read. The
replacement must name the address exactly, and the packed crate keeps
naming it.

Every edited file's original bytes are written back before pack answers,
on every path. The race check stays: the crate's own manifest is read
and a crate naming another name or version is refused naming both, since
every pack of a revision edits one checkout.

Why `--offline` and the scratch registry: the registry `tailrace` is the
host's serving process, which serves consumers' layouts under their
policies and holds nothing a publish needs; the third-party crates the
crate's lockfile names come from the cache the build just filled.
Alternatives: pack the component's crates together and let cargo's
overlay resolve the siblings; refused because the overlay is not
consulted once the registry is replaced, and the replacement is what
reaches another component's crates. Pack online against the serving
process; refused because it answers what a layout holds under a policy,
not what the store holds.

### D4. same_again

The bytes are a function of the tree, the version, the revision, the
checkout's `[registries.tailrace]` address, which is the publishing
repository's own and the same on every host (49), the cargo that ran,
and the crates the scratch registry held. A crate whose dependency graph
reaches no crate of another component answers `same_again` true. One
that does answers false: its lockfile names the newest version the store
held within the requirement when it was packed, and a later publish of
another version within it packs other bytes (87). A host with another
cargo may pack other bytes too; a repeat publish there is refused naming
the artifact (18), the clause's own remedy.

### D5. Pinned reads the lockfile and nothing else

Pinned reads `Cargo.lock` at the checkout's root with `toml_edit` and
answers, for each `[[package]]` whose `source` begins `registry+` or
`sparse+`, its `name` and `version` as written (137). A package with no
source is the checkout's own, and one from git is pinned by commit; both
are left out. crates.io's pins are answered like any other: the core
finds no name for them in the store. No `Cargo.lock` answers absent
naming it; a file that is not TOML, or a package with no name or version,
is refused naming the file and carrying the parser's words (58, 59).
Pinned runs no cargo, since `cargo metadata` may write the lockfile, and
writes nothing.

### D6. Render writes the registry's files, from the store alone

Render first checks every artifact (ecosystem `cargo`, one layer, bytes
of that digest, the crate's manifest naming the metadata's name and
version, no two digests for one name and version), then writes, under
the place given:

```
index/<path>                 each name's index file, <path> as the sparse protocol names it
<name>-<version>.crate       each crate
```

This is also Cargo's local-registry layout, which D3 points the scratch
replacement at. Crates are written beside their place and renamed onto
it; each index file is written whole and renamed. Crate and index files
of versions not given are removed, so the layout holds exactly what it
was given: a version the policy refused is never given, so no index line
lists it (99). Index lines are sorted by `semver` (23). A relative place
is under the checkout the projection is bound in.

Index line, from the normalized manifest, as built: `name`, `vers`,
`deps`, `cksum`, `features`, `features2` and `v`, `links`,
`rust_version`, `yanked` false, keys in a fixed order. A dependency's
`registry` is null where its `registry-index` is a served Cargo address,
`sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/` for any owner and
name, its host and port the core's (`served`); the dependency's
`registry-index` where it is another registry; crates.io's index where
it names none. Why null for every family repository's address: the
packed crate names its dependency at the publisher's address, a consumer
reads at its own, and its own resolves any family crate by name and
version (138), so the dependency is one of the registry serving the
line. The layout names no address: `config.json` is the server's (D7),
so one layout serves under any base.

### D7. Cargo's serve, and a refusal three ways

The request's `path`, below `/<owner>/<name>/cargo/`:

- `config.json`: `{"dl":"<base>/crates/{crate}/{version}/download"}`,
  `200` `application/json`, with no `api`, since the registry takes no
  publish.
- an index path (`1/<n>`, `2/<n>`, `3/<c>/<n>`, `<ab>/<cd>/<n>`, `<n>`
  lower-case and its prefix matching it): the layout's
  `index/<path>`, `200` `text/plain`; no such file is `404`. An index
  file never asks the core: what a layout lists is what the policy
  admitted when sync rendered it.
- `crates/<name>/<version>/download`: the layout's
  `<name>-<version>.crate`, `200` `application/octet-stream`; one the
  layout lacks calls `miss` once with `Pinned { name, version }` and, on
  `Ok`, answers the crate now in the layout.
- anything else, `path` empty among them, `404`; a method other than
  `GET` or `HEAD`, `405`.

Every answer that is not the bytes or file asked is `text/plain`, the
`Not`'s own rendering, the source's words verbatim: `404` absent, `403`
refused, `502` unreadable (58, 59, 99); cargo prints it under `body:`.
Every request is answered with a response, so sync's probe reads the
projection as serving (npm's D10); `Err` is kept for a layout whose
files cannot be read for another reason than their absence.

A refused version so reaches Cargo three ways (99): its line is absent
from the name's index file, since render is never given it (D6); a
request for its bytes, from Cargo or anyone, is a miss whose refusal is
answered `403` naming the version and the claim; and `tailrace sync`
answered the refusal when it rendered the layout (render under a
policy). Cargo itself resolves from the index before it downloads, so in
a build the words a person reads are sync's, beside Cargo's "failed to
select a version".

### D8. Bound with a store of its own

`profiles::projection::bound` takes a second store for Cargo; the
bindings bind the store once more for it, as they bind `cache` for the
moon cache, since a `Box<dyn Store>` is not shared (core.2).

### D9. Tests, and the proof as one loop

Unit tests in the profile build crates in memory with `flate2` and
`tar` for the index line, render and serve, and read lockfiles written as
text for pinned; they run in `check-fast` with no cargo needed. The
store listing has unit tests in the layout and fake stores and a
tools-layer test against the registry the tools layer runs. The
conformance group and the proofs run cargo, so they sit behind the
`tools` feature in the `tools` task (check-tools), with a sealed
`CARGO_HOME` and `HOME` per test:

- `tests/cargo_projection.rs`: the projection group over a scratch git
  repository holding a workspace of two sibling crates, its rig serving
  `crates/<name>/<version>/download` as the request for a version; and
  the components: a workspace of component `fam` (`fam-a`, `fam-b`) and
  component `base` (`base-c`), `fam-a` depending by path on `fam-b` and
  on `base-c` with `version = "0.3"`, a layout store holding `base-c`
  0.3.1 under `artifact/cargo/base-c@0.3.1`. Packing `fam-a` at a
  version names `fam-b` at that version, `base-c` at `0.3` and both in
  `tailrace`'s index, and its lockfile `base-c 0.3.1` with the held
  crate's checksum; `base-c`'s manifest is never edited; every manifest
  holds its bytes afterwards; `same_again` is false. With the store
  holding only `base-c` 0.4.0, the pack is refused naming `base-c`,
  `0.3` and `0.4.0`.
- `tests/cargo_served.rs`: the publisher's workspace, `fam-b` and
  `fam-a` depending on it by path, each sibling dependency written with
  a placeholder version and `registry = "tailrace"`, `.cargo/config.toml`
  naming `tailrace` at its served address; the build's packaging task
  stood in for by `cargo package --workspace --no-verify --offline
  --registry tailrace`; both packed at one version, put in a layout store
  with their index, claims and `artifact/cargo/<name>@<version>` names as
  npm's served test puts its package. A consumer checkout whose
  `.cargo/config.toml` names `tailrace` at
  `sparse+<the test's base>/<owner>/<name>/cargo/`, depending on `fam-a`
  in that registry, with a `Cargo.lock` written as text pinning both
  crates at that source with the packed crates' checksums; `tailrace
  sync` renders the layout and the process at the test's address serves
  it; `cargo build --locked` succeeds and leaves the lockfile as written.
  Then a version the policy refuses is pinned: sync's answer names it and
  the claim found wanting, its name's index file lists no line for it,
  the build fails, and a request for its download answers `403` carrying
  the same words.

## Risks / Trade-offs

- [The store's listing grows with every version published] → a pack
  reads it once per crate of another component it reaches, and a
  registry pages it; render under a policy never lists.
- [A pack offline on a host whose cargo cache lacks a third-party
  crate] → unreadable with cargo's words; the build has just run on the
  host, so the cache is warm.
- [A crate whose `publish` leaves `tailrace` out, or a checkout naming no
  `tailrace` registry] → cargo refuses, or pack refuses naming the
  registry and the files read.
- [Two components naming one path] → a crate there is a sibling of the
  packed one, taking this component's version, as a commit there
  versions both (124).
- [A sibling that is not a declared output] → packed into the crate's
  lockfile and never published, so a consumer cannot resolve it; the
  authors declare every crate of a component its outputs reach.
- [In-place edits of a shared checkout] → the race check refuses rather
  than offers a wrong crate.
- [A crates.io crate with a family crate's name and version] → resolved
  as the family's by name and version alone (138); Cargo still reads it
  only for a dependency naming the registry `tailrace`.

## Open Questions

For the design agent, none of which changes what this change builds:

- B.1 lists seven store operations (66 to 72) and 138 says the store
  lists the versions a name holds; this change adds the listing to the
  store seam (D2). B.1 could name it.
- B.6's pack (84) is given an output, a build identity and a version;
  C.11's Cargo row needs the component's paths and the store's versions
  of a name besides, which this change hands pack with the version (D2).
- 138 says a registry lists the tags of `artifact/<ecosystem>/<name>`,
  a repository per name; the registry profile holds every name as a tag
  of one declared repository, and lists that repository's tags (D2).
- A crate reaching another component's crate packs a lockfile naming the
  newest version the store held within the requirement, so its pack
  cannot promise the same bytes (D4); the statement may prefer a version
  that does not move, such as the oldest within the requirement.

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

- **the store names an artifact by ecosystem, name and version** (its
  design, D7, D10, D11) adds the store's listing,

  ```rust
  fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>>;
  ```

  answering every version the store names `artifact/<ecosystem>/<name>@`
  at, in no order the core relies on, or absent where it names none, on
  every store profile (138, C.1). It makes pinned answer `Vec<Pin>`,
  `Pin::Version(Pinned)` for a lockfile's pin or `Pin::Range(Ranged {
  name, range })` for a name a manifest declares and no lockfile pins,
  `Range` the manifest's text, which the core never reads; and it gives
  render

  ```rust
  Contents::Ranged { held: &[Artifact], within: &[Within] }
  pub struct Within { pub range: Ranged, pub candidates: Vec<Artifact> }
  ```

  where `held` is held as given and of each `within`'s candidates, every
  version of the range's name the policy admitted, render holds the
  newest by the ecosystem's own rules within the range, or none, its
  `Holds::Versions` saying which (137, 23). The core lists, judges and
  records; which versions fall within a range, and which is newest, are
  the projection's.

Built here already: the profile, its binding with a store of its own,
render writing a crate per version and an index file per name, the index
line, the race check after pack, and the conformance rig.

C.11's Cargo row, at blueprints `b720017`: pack the crate file with
`cargo package --no-verify` after the component's version is set on every
crate of the component, a path dependency on a sibling rewritten to that
version's requirement as cargo does (110); a crate belongs to the
component whose paths hold its manifest (123), and a path dependency on a
crate of another component is packed as a dependency on the version
requirement the manifest writes for it, refused naming the crate where
the store holds no such version (138); render a sparse registry under the
served address (49, 114), named `tailrace` in the checkout's cargo
configuration; serve from the host's one process. 84 gives pack the
component the output belongs to with its declared outputs, so a
projection knows which outputs are the component's own. 137 has pinned answer, for a name
the checkout's manifest declares and no lockfile pins yet, the manifest's
range, and render hold the newest version the layout's policy admits
within it, resolved by the store's listing (138). 99 says a
sparse-index client's refusal reaches it by omission from the listing,
by the bytes' answer and by sync's words.

Vendor facts, checked against cargo 1.98.1 (the devenv's) and Cargo's
reference (registries, registry index, `cargo package`):

- `cargo package` writes every member with the fixed time 2006-07-23 and
  owner 0/0. In a git checkout it writes `.cargo_vcs_info.json` with the
  HEAD sha1 and `path_in_vcs`, the package's directory in the
  repository, and refuses an edited manifest without `--allow-dirty`.
- A path dependency with no `version` is refused when packaging ("all
  dependencies must have a version requirement specified when
  packaging"); with one and no `registry`, the packed crate names it
  from crates.io. With `registry = "tailrace"` beside `path` and
  `version`, and `--registry tailrace`, the packed manifest names it
  `version = "<as written>"` and `registry-index = "<tailrace's index
  as the checkout's cargo configuration names it>"`, and no path.
- By default `cargo package` resolves the crate's dependencies to write
  the crate's own `Cargo.lock`, and offline it fails on a dependency of
  the registry `tailrace` that no source holds ("no matching package
  named `base-c` found, location searched: `tailrace` index").
- `--exclude-lockfile`, stable in 1.98.1, packs with no `Cargo.lock` in
  the crate and resolves nothing: a crate depending on a sibling at
  `1.2.0` and on another crate at `0.3`, both of the registry
  `tailrace`, neither held anywhere, packed offline with `--no-verify
  --allow-dirty --exclude-lockfile --registry tailrace -p fam-a`, gave
  `.cargo_vcs_info.json`, `Cargo.toml`, `Cargo.toml.orig` and the
  sources, and wrote no `Cargo.lock` in the workspace; two packs gave
  identical bytes.
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
- A dependency's requirement is Cargo's: a bare version `0.3` reads as
  `^0.3`, and Cargo parses requirements with the `semver` crate's
  `VersionReq`, whose `matches` leaves out a prerelease unless a
  comparator names a prerelease of the same major, minor and patch, as
  Cargo resolves. A dependency written with no `version` and no `path`
  or `git` reads as `*`.
- `cargo build` without `--locked`, given a `Cargo.lock` that pins no
  version of a dependency the manifest declares, resolves it from the
  registry's index file and adds its `[[package]]` with `source` and
  `checksum` to the lockfile.
- A workspace's members are the root manifest's `workspace.members`,
  glob patterns relative to the root, less `workspace.exclude`, and the
  root's own `[package]` where it has one; a member's dependency written
  `{ workspace = true }` takes the requirement and registry of the root's
  `workspace.dependencies` entry of that key, and one written with
  `package = "<name>"` depends on that name under another key.

## Goals / Non-Goals

**Goals:** pack, pinned, render and serve for Cargo on the seams the
units above land, held to the whole projection group; a crate's
siblings and its other components' crates told apart by the component's
paths, and its siblings checked against the component's outputs; a
manifest's range answered and resolved by Cargo's rules; `cargo build
--locked` proven against the host's serving process after `tailrace
sync`, and a crate added by range resolved and pinned by Cargo.

**Non-Goals:** choosing versions, the policy, the layout's place and
`tailrace sync` (render under a policy); the process, its route and its
lifetime (npm, rendered and served); making the
`artifact/<ecosystem>/<name>@<version>` pointer and its registry
repository, the store's listing, and listing and judging a range's
versions (the store-naming unit); a copy carrying those pointers;
writing a checkout's cargo configuration.

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

### D2. What pack is given

The seam's pack is given, where the bytes are an artifact's, what it
packs at (84):

```rust
/// 84, 110, 123: what an output is packed at as an artifact.
pub struct PackAt<'a> {
    /// The component's one version (110).
    pub version: &'a Version,
    /// The component the output belongs to (123).
    pub component: &'a Component,
    /// The component's declared outputs, as the build left them, the
    /// output being packed among them (75).
    pub outputs: &'a [Output],
    /// 123: whether the component's paths hold a path of the repository,
    /// matched as the version rule matches them.
    pub holds: &'a dyn Fn(&str) -> bool,
}
fn pack(&self, output: &Output, at: &BuildId, version: Option<&PackAt<'_>>) -> Answer<Packed>;
```

Publish builds it once per component after `ensure` and the reports,
where every declared output's files are known (core.12): `outputs` from
each report's files, `holds` from the component's declared `paths` with
the same `glob` patterns the version rule gives git-cliff. Every
projection but Cargo's reads `version.version` alone; the cache push
packs with `None` as now. Why a closure for the paths: matching them is
the core's, the version rule's own (core.2); the profile matches no
glob. Alternative: the paths' match as a method of `Component` in the
seams; refused because the seams crate then takes `glob` and a second
reading of the patterns the version rule already owns.

The Cargo projection asks the store it is bound with (D8), the store the
publish puts into, for a name's versions by the store-naming unit's
`versions`, and reaches the store no other way (core.2). It adds no
operation to the store seam and no listing to any store profile: the
listing is that unit's alone (138).

### D3. The component's version on its own crates, and nothing picked of another's

From the packed package, the projection follows each dependency in
`dependencies`, `build-dependencies` and each `target.<cfg>` table of
them that names a `path`, written in the package's manifest or
inherited from `workspace.dependencies`, and so on from each crate it
reaches through a dependency of its own component. A crate reached is a
**sibling** where `holds` answers true for its manifest's path relative
to the checkout, the component's own (123); otherwise it is **of another
component**. Dev dependencies are left as written: cargo drops a path
dev dependency with no version from the packed crate.

The component's outputs say which crates it publishes: each output's one
crate file, read as D1 reads the packed output's, names a package. A
sibling no output names is refused naming it and the component, since a
consumer could resolve nothing for it (C.11, 84). An output whose files hold no crate file is no crate of the
component and is passed over here; its own pack refuses it (D1).

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

Before cargo runs, each crate of another component is checked against
the store: `versions` of it in `cargo`, read with `semver`'s
`VersionReq`, Cargo's own, against the manifest's requirement. None
within it refuses the pack naming the crate, the requirement and the
versions the store holds; `versions` answering absent refuses it naming
the crate, the requirement and that the store holds no version of it;
an unreadable listing is the pack's answer as the store gave it (C.11,
138, core.19). In each case no cargo runs. Pack picks no
version of it: the packed crate names the requirement alone, and which
version a build uses is the consumer's lockfile's (137).

Then the package is packed alone, `cargo package --no-verify
--allow-dirty --offline --exclude-lockfile --registry tailrace
--manifest-path <workspace root>/Cargo.toml -p <name> --target-dir
<temp>`, in the package's directory, and the answer is
`<temp>/package/<name>-<version>.crate`. Siblings are not packed here:
each is answered by its own output's pack, at the same version.

Every edited file's original bytes are written back before pack answers,
on every path. The race check stays: the crate's own manifest is read
and a crate naming another name or version is refused naming both, since
every pack of a revision edits one checkout.

Why `--exclude-lockfile`: without it cargo resolves every dependency to
write the crate's own lockfile, which needs a source holding the
siblings and the other components' crates, and names whichever version
of another component's crate that source held newest, so the bytes would
move with the store (vendor facts). A crate's own lockfile is read by no
build that depends on it. Alternatives: a scratch local registry of the
siblings and the store's crates, replacing `tailrace`; refused because
the lockfile it yields picks a version of another component's crate,
which the statement leaves to the consumer, and the bytes then depend on
what the store held. Pack online against the serving process; refused
because it serves consumers' layouts under their policies, not what a
publish packs.

### D4. same_again

The bytes are a function of the tree at the revision, the version, the
checkout's `[registries.tailrace]` address, which is the publishing
repository's own and the same on every host (49), and the cargo that
ran, the devenv's (core.26); nothing the store holds enters them. Every
Cargo pack with a version answers `same_again` true (87). A host with
another cargo may pack other bytes; a repeat publish there is refused
naming the artifact (18), the clause's own remedy.

### D5. Pinned reads the lockfile and the manifests, and nothing else

Pinned reads `Cargo.lock` at the checkout's root with `toml_edit` and
answers `Pin::Version` for each `[[package]]` whose `source` begins
`registry+` or `sparse+`, its `name` and `version` as written (137). A
package with no source is the checkout's own, and one from git is pinned
by commit; both are left out. crates.io's pins are answered like any
other: the core finds no name for them in the store.

Then the ranges. Pinned reads the root `Cargo.toml` and each member's,
the members found as Cargo finds them (vendor facts): `workspace.members`
expanded with `glob` against the root, less `workspace.exclude`, and the
root's own `[package]`. In each member's `dependencies`,
`dev-dependencies`, `build-dependencies` and each `target.<cfg>` table of
them, a dependency is of the registry `tailrace` where it writes
`registry = "tailrace"`, or `workspace = true` and the root's
`workspace.dependencies` entry of its key writes it. Its name is its
`package` where it writes one, else its key; its range is the `version`
it, or the inherited entry, writes, as written, or `*` where it writes
none. One with a `path` or a `git` is left out: Cargo builds it from
there. For each name and range so declared whose name the lockfile pins
at no version from a registry, pinned answers `Pin::Range` once, two
ranges of one name each once. Only the registry `tailrace` is read: a
range of any other registry is resolved by Cargo against that registry,
and the store holds only family crates (138).

No `Cargo.lock` answers absent naming it, as 137 says and the seam's
contract has it, though the root manifest declares crates of
`tailrace`. A lockfile or manifest that is not TOML, or a lockfile
package with no name or version, is refused naming the file and
carrying the parser's words (58, 59). Pinned runs no cargo, since cargo
may write the lockfile, and writes nothing.

### D6. Render writes the registry's files, from the store alone

Render first checks every artifact (ecosystem `cargo`, one layer, bytes
of that digest, the crate's manifest naming the metadata's name and
version, no two digests for one name and version), then writes, under
the place given:

```
index/<path>                 each name's index file, <path> as the sparse protocol names it
<name>-<version>.crate       each crate
```

Given `Contents::Ranged`, render checks the held artifacts and every
range's candidates as above, then takes of each range the one candidate
it holds: `VersionReq::parse` of the range's text, Cargo's own parser,
and of the candidates whose metadata's version parses as a `semver`
version and `matches` the requirement, the greatest. A range whose text
does not parse is refused naming the name, the range and the parser's
words, before anything is written; a candidate whose version is not a
Cargo version is within no range; a range with no candidate within it
holds none. The crates written are then the held ones and each range's
one, and `Holds::Versions` names each, which is how the core learns
which candidate render took. A version both pinned and a range's
candidate is written once. Candidates not taken are never read from the
store, past the check, and never written, so the name's index file lists
only what the layout holds and Cargo, resolving the requirement against
it, takes the version render took and writes its pin.

Crates are written beside their place and renamed onto it; each index
file is written whole and renamed. Crate and index files of versions not
given are removed, so the layout holds exactly what it was given: a
version the policy refused is never given, so no index line lists it
(99). Index lines are sorted by `semver` (23). A relative place
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
text and workspaces written as text for pinned; they run in
`check-fast` with no cargo needed. The conformance group and the proofs run cargo, so they sit behind the
`tools` feature in the `tools` task (check-tools), with a sealed
`CARGO_HOME` and `HOME` per test:

- `tests/cargo_projection.rs`: the projection group over a scratch git
  repository holding a workspace of two sibling crates, its rig serving
  `crates/<name>/<version>/download` as the request for a version; and
  the components: a workspace of component `fam` (`fam-a`, `fam-b`, each
  a declared output) and component `base` (`base-c`), `fam-a` depending
  by path on `fam-b` and on `base-c` with `version = "0.3"`, a layout
  store holding `base-c` 0.3.1 under `artifact/cargo/base-c@0.3.1`.
  Packing `fam-a` at a version names `fam-b` at that version, `base-c` at
  `0.3` and both in `tailrace`'s index and with no path, and the crate
  holds no `Cargo.lock`; `base-c`'s manifest is never edited; every
  manifest holds its bytes afterwards; `same_again` is true. With the
  store also holding `base-c` 0.3.2, packing again gives the same bytes.
  With the store holding only `base-c` 0.4.0, the pack is refused naming
  `base-c`, `0.3` and `0.4.0`. With `fam-b` among no output of `fam`,
  the pack is refused naming `fam-b` and `fam`.
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
  the same words. Then the range: the publisher's `fam-b` is also packed
  at a second version within `1`, newer, and a third outside it, all
  three named in the store, and a second consumer, whose `Cargo.lock`
  pins only its own package, declares `fam-b = { version = "1", registry
  = "tailrace" }`; sync's answer names the range and the newer version
  within it, the served index file of `fam-b` lists that version alone,
  `cargo build` succeeds, and the consumer's `Cargo.lock` now pins
  `fam-b` at that version from the served address with its packed
  checksum. `fam-b` depends on no family crate, so the range proves
  nothing about what such a crate's own family dependencies need (Open
  Questions).

## Risks / Trade-offs

- [The store's listing grows with every version published] → a pack
  reads it once per crate of another component it reaches; a sync lists
  a name only for a range, which only a name no lockfile pins has.
- [A crate packed without its own lockfile] → a `cargo install` of a
  family binary from the registry resolves its dependencies afresh, as
  it does for any crate published without one; a library's lockfile is
  read by no build that depends on it.
- [A crate whose `publish` leaves `tailrace` out, or a checkout naming no
  `tailrace` registry] → cargo refuses, answered unreadable with cargo's
  own words.
- [Two components naming one path] → a crate there is a sibling of the
  packed one, taking this component's version, as a commit there
  versions both (124).
- [In-place edits of a shared checkout] → the race check refuses rather
  than offers a wrong crate.
- [A crates.io crate with a family crate's name and version] → resolved
  as the family's by name and version alone (138); Cargo still reads it
  only for a dependency naming the registry `tailrace`.

## Open Questions

For the design agent; this change builds as written either way:

- A first install. 137 answers absent where a checkout holds no
  lockfile, and a range only where a manifest declares a name "no
  lockfile pins yet", so a Cargo checkout with no `Cargo.lock` at all
  gets no layout, and Cargo cannot write one without the registry. This
  change follows 137 as written (D5); a consumer gets a first lockfile
  by writing one that pins only its own package, as the proof does.
  Whether a checkout with a manifest and no lockfile answers its
  manifest's ranges is the statement's to say.
- A range's crate's own family dependencies. Render holds, of a range,
  the one version taken; a family crate that version depends on is
  neither pinned nor declared by the consumer's manifest, so no layout
  holds it, and Cargo's resolution fails on its index file until a
  lockfile pins it. 137 names only the names a manifest declares; the
  statement could say whether sync follows a held version's dependencies
  in the store, and under which policy.

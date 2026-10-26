## Context

The projection seam (B.6, `crates/tailrace-seams/src/projection.rs`) has
pack, render, serve and pinned. Three units this change comes after
shape what it implements:

- **render under a policy** adds pinned (137) and the core's render in
  `crates/tailrace-core/src/render.rs`: `render` reads a checkout's pins,
  and for each, `judged` resolves it with `locate` through
  `artifact/<ecosystem>/<name>@<version>` to the version's index and the
  artifact found in it by its metadata (138), reads the claims on both,
  finds the layout's policy with `policy_for` and judges it with `judge`
  (113, C.10), all before anything is written; it then hands the
  projection exactly the admitted artifacts as `Contents::Artifacts` at
  `<working tree>/.tailrace/layouts/<ecosystem>/` and writes the
  layout's record (`LayoutRecord`, `held` and `refused`). `tailrace sync`
  runs it and answers every pin, `Refreshed.pins`, a refused one naming
  the version and the claim found wanting (99). pinned's contract there
  says a name the manifest declares and no lockfile pins is no pin until
  ranges resolve.
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
  design, D7, D10) adds the store's listing,

  ```rust
  fn versions(&self, ecosystem: &Ecosystem, name: &EcosystemName) -> Answer<Vec<Version>>;
  ```

  answering every version the store names `artifact/<ecosystem>/<name>@`
  at, in no order the core relies on, the empty list where it names none,
  and unreadable only where the read fails, on every store profile (138,
  C.1). No core path of that unit reads it, and it leaves a manifest's
  range, and the part of 137 about one in the conformance register, to
  this change. It also holds a declared component's indexes and channels
  in a registry repository of the component's own, which this change
  reads nothing of: it reaches a version only by its artifact's name.

Built here already: the profile, its binding with a store of its own,
render writing a crate per version and an index file per name, the index
line, the race check after pack, and the conformance rig.

The statement, blueprints `4c14235`:

- C.11's Cargo row: pack the crate file with `cargo package --no-verify`
  after the component's version is set on every crate of the component,
  a path dependency on a sibling rewritten to that version's requirement
  as cargo does (110); a crate belongs to the component whose paths hold
  its manifest (123); a sibling the component does not declare as an
  output is refused naming it; a path dependency on a crate of another
  component is packed as a dependency on the version requirement the
  manifest writes for it, refused naming the crate where the store holds
  no such version (138); render a sparse registry under the served
  address (49, 114), named `tailrace` in the checkout's cargo
  configuration; serve from the host's one process.
- 84 gives pack the component the output belongs to with its declared
  outputs, so a projection knows which outputs are the component's own.
- 137: pinned answers, for a name the checkout's manifest declares and no
  lockfile pins yet, the manifest's range, and the core holds the newest
  version the layout's policy admits within it, by the store's listing of
  the name's versions, so a first install resolves and the client writes
  the pin; and for every version it holds, the core asks the projection
  what that version requires of the family, lists, judges and holds
  again, through every level until nothing held requires a name the
  layout lacks, so no lockfile is written by hand. The loop is the
  core's, the listing the store's, the ranges and their order the
  projection's. The design agent's batch 27 ruling reads it so for
  Cargo: pinned answers the manifest's ranges for every name no lockfile
  pins, a checkout with no `Cargo.lock` among them, and Cargo writes the
  lockfile.
- 140, requires: given a held version's artifact, the family's names and
  ranges its manifest requires, as the manifest spells them, and whether
  one version satisfies a range by the ecosystem's own ordering (23); a
  projection whose ecosystem has no manifest answers none; the core
  never reads a manifest itself (core.1).
- 138: the store lists the versions a name holds, so a range with no pin
  resolves by that listing under the layout's policy; the planner settled
  that the listing is the store-naming unit's, the empty list for a name
  the store never held.
- 99: a sparse-index client's refusal reaches it by omission from the
  listing, by the bytes' answer and by sync's words.

Vendor facts, checked against cargo 1.98.1 (the devenv's) and Cargo's
reference (registries, registry index, `cargo package`, the manifest
format, dependency resolution):

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
- A packed crate's normalized `Cargo.toml` writes its dependencies in
  `dependencies`, `build-dependencies`, `dev-dependencies` and each
  `target.<cfg>` table of them, each with its `version` requirement, and
  `registry-index` for one of a registry other than crates.io, `optional`
  where it is, and `package` where it is renamed.
- A sparse registry: Cargo fetches `<index>/config.json` (`dl`, the
  download template with `{crate}` and `{version}`; `api` absent for a
  registry that takes no publish), then each name's index file at
  `1/<n>`, `2/<n>`, `3/<c>/<n>` or `<ab>/<cd>/<n>`, the name lower-case,
  then each crate at `dl`. A 404 on an index file is "not found". Cargo
  resolves from the index files before it asks for any crate, a locked
  build too, so a version missing from an index file fails resolution
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
  or a path dependency. Cargo writes one lockfile for every platform, so
  it pins a dependency of any target.
- `cargo build` without `--locked`, in a checkout with no `Cargo.lock`
  or one that pins no version of a dependency the manifest declares,
  resolves it from the registry's index files, its own dependencies too,
  and writes the lockfile with each `[[package]]`'s `source` and
  `checksum`.
- A dependency's requirement is Cargo's: a bare version `0.3` reads as
  `^0.3`, and Cargo parses requirements with the `semver` crate's
  `VersionReq`, whose `matches` leaves out a prerelease unless a
  comparator names a prerelease of the same major, minor and patch, as
  Cargo resolves; versions are ordered by `semver::Version`'s order. A
  dependency written with no `version` and no `path` or `git` reads as
  `*`.
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
paths, and its siblings checked against the component's outputs; ranges
and requires (140) carried by the seam, and the core's render closing a
layout over them, each version listed by the store and judged by the
core; a manifest's range and a crate's requirement resolved by
Cargo's rules; a consumer with no lockfile proven to `cargo build`
against the host's serving process after `tailrace sync`, Cargo writing
the lockfile, and `cargo build --locked` after a second sync.

**Non-Goals:** the policy, judging one version, the layout's place and
its record's form (render under a policy); the process, its route and
its lifetime (npm, rendered and served); making the
`artifact/<ecosystem>/<name>@<version>` pointer and the store's listing
(the store-naming unit); npm's ranges and what an npm version requires; a copy carrying
those pointers; writing a checkout's cargo configuration.

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

The Cargo projection asks the store it is bound with (D10), the store the
publish puts into, for a name's versions by the store-naming unit's
`versions`, and reaches the store no other way (core.2). It adds no
operation to the store seam and no listing to any store profile: the
listing is that unit's alone, the empty list where the store names no
version of the name (138).

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
versions the store holds; an empty listing, the store-naming unit's
answer for a name the store never held, refuses it naming the crate,
the requirement and that the store holds no version of it; an
unreadable or refused listing is the pack's answer as the store gave it
(C.11, 138, core.19). In each case no cargo runs. Pack picks no
version of it: the packed crate names the requirement alone, and which
version a build uses is the consumer's (137).

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

Then the ranges, with or without a lockfile. Pinned reads the root
`Cargo.toml` and each member's, the members found as Cargo finds them
(vendor facts): `workspace.members` expanded with `glob` against the
root, less `workspace.exclude`, and the root's own `[package]`. In each
member's `dependencies`, `dev-dependencies`, `build-dependencies` and
each `target.<cfg>` table of them, a dependency is of the registry
`tailrace` where it writes `registry = "tailrace"`, or `workspace = true`
and the root's `workspace.dependencies` entry of its key writes it. Its
name is its `package` where it writes one, else its key; its range is
the `version` it, or the inherited entry, writes, as written, or `*`
where it writes none. One with a `path` or a `git` is left out: Cargo
builds it from there. For each name and range so declared whose name no
lockfile pins at a version from a registry, pinned answers `Pin::Range`
once, two ranges of one name each once. Only the registry `tailrace` is
read: a range of any other registry is resolved by Cargo against that
registry, and the store holds only family crates (138).

A checkout with a root `Cargo.toml` and no `Cargo.lock` so answers its
manifests' ranges, and a first install needs no lockfile written by
hand: sync holds each range's version and what it requires (D7), and
`cargo build` writes the lockfile (137; the design agent's batch 27
ruling). Pinned answers absent naming `Cargo.lock` where the checkout
holds no `Cargo.lock` and its manifests declare nothing in `tailrace`, a
checkout with no root `Cargo.toml` among them, so a checkout that is no
Cargo consumer gets no layout, as the seam's contract has it. A checkout
with a lockfile answers the list, empty or not.

A lockfile or manifest that is not TOML, or a lockfile package with no
name or version, is refused naming the file and carrying the parser's
words (58, 59). Pinned runs no cargo, since cargo may write the
lockfile, and writes nothing.

### D6. The seam carries ranges and what a version requires

`crates/tailrace-seams/src/projection.rs` gains:

```rust
/// 137: a name a checkout's manifest, or a held version's, declares, by
/// the range the manifest writes for it, as written; the core never
/// reads the range.
pub struct Ranged { pub name: EcosystemName, pub range: String }

/// 137: what pinned answers for a checkout: a version its lockfile pins,
/// or a range its manifest declares for a name no lockfile pins yet.
pub enum Pin { Version(Pinned), Range(Ranged) }

fn pinned(&self, checkout: &str) -> Answer<Vec<Pin>>;

/// 140: the family's names and ranges the manifest of `artifact`, a
/// held version's artifact of this ecosystem, requires of the
/// ecosystem, as the manifest spells them; none from a projection whose
/// ecosystem has no manifest.
fn requires(&self, artifact: &Artifact) -> Answer<Vec<Ranged>>;

/// 140, 23: of `versions`, those that satisfy `range` by the
/// ecosystem's own rules, newest first by its own ordering; refused
/// naming the name, the range and the ecosystem's own words where the
/// range cannot be read; none from a projection whose ecosystem has no
/// manifest.
fn within(&self, range: &Ranged, versions: &[Version]) -> Answer<Vec<Version>>;
```

`Pin` and `Ranged` serialize as the seam's other types, a pin and a
range each read back as written. render is unchanged: it is given
exactly the artifacts the core holds. The file, image and moon cache
projections answer `requires` and `within` with none, as 140 says of an
ecosystem with no manifest. npm's projection answers none here too: npm
has a manifest, and what an npm version requires of the family is npm's
own later work.

Why `within` answers an ordered list rather than one version's yes or
no: 140 gives the projection whether a version satisfies a range by the
ecosystem's own ordering, and the core must take the newest, which 23
orders by the ecosystem's rules and never by spelling or listing order;
one call answers both and keeps the order out of the core (core.1).
Alternatives: a `satisfies(range, version)` and a separate comparison;
refused as two calls for one answer. The core ordering by the
publisher's version rule; refused because a consumer knows no
publisher's rule and the range is the ecosystem's.

### D7. The core closes the layout

`RenderAsk.pins` becomes `Vec<Pin>`, as pinned answers. `render` in
`crates/tailrace-core/src/render.rs` holds, in a fixed order, writing
nothing until the end:

1. each `Pin::Version`, judged as now;
2. each `Pin::Range`, by name and range, then, level by level, each
   range the held versions of the level before require, by name and
   range, each name and range once. A range is met where the layout
   already holds a version of its name that `within` answers satisfies
   it, the version held for it. Otherwise the store's `versions` of the
   name, the empty list where it names none, go through `within`, and
   each version it answers, newest first, is judged as a pin is until
   one is held; that one is held for the range, or none where none is
   admitted or none is listed within it;
3. for each version newly held, pinned or taken, `requires` of its
   artifact gives the next level's ranges, until a level holds nothing
   new, so nothing held requires a name the layout lacks (137, 140).

Then `.tailrace/` ignores itself, the projection renders
`Contents::Artifacts` of every held version at `place`, and the record
is written: `held` every held version, `refused` every refused one. A
read whose source cannot answer, the store's listing, a judgment's
reads, or a projection's `requires` or `within`, is sync's answer for
the layout, and nothing is written; a range `within` refuses is that
range's own answer, held nothing. The fixed order makes the same store
under the same policy hold the same versions (48). A version is judged
under the same policy whichever way it was reached (113), and the layout
holds only admitted versions (47, core.17).

`Refreshed` gains `asked`, the answers of the versions judged for a
range, and `ranges`, each range with what required it, none for the
checkout's manifest or the held version whose manifest requires it, and
the version held for it or that none was:

```rust
pub struct RangeAnswer { pub range: Ranged, pub by: Option<Pinned>, pub answer: Answer<Option<Version>> }
```

An absent answer for a version asked, a name the listing names and the
store no longer resolves, is answered as a pin's absence is. A refused
version asked counts against sync's exit only where its range holds no
version: a range holding an older admitted version is the consumer's
build working, and the refusal is answered all the same (99). `tailrace
sync` prints, beside the pins, each range as `range <name> <range>`,
from the checkout's manifest or from `<name> <version>`, and the version
held or that none within it was admitted, and each version asked as a
pin is printed; `--json` carries `asked` and `ranges`.

Why "met" reads the held version through `within`: 137 stops where
nothing held requires a name the layout lacks; a version of the name
that does not satisfy the range would leave the client's resolution
open, so it does not meet it, and the range is resolved by the same
rule.

### D8. Cargo's requires, within and render

`requires` reads the artifact's crate through its store as render
reads it (one layer, bytes of that digest), and its normalized
manifest, and answers each family dependency the crate requires: a
dependency in `dependencies`, `build-dependencies` or a `target.<cfg>`
table of them, optional or not, whose `registry-index` is a served Cargo
address, named by its `package` or its key, with its `version` as the
range, as written, each name and range once. A family dependency's
`registry-index` is `sparse+http://127.0.0.1:7420/<owner>/<name>/cargo/`
for any owner and name, its host and port the core's (`served`), the
same test the index line makes. Dev dependencies are a package's own and
never resolved for a consumer of it. Optional and target ones are
answered, since Cargo writes one lockfile for every platform, and a
feature a consumer turns on needs the crate it enables. A crate that
does not read is unreadable naming its digest.

`within` reads the range with `VersionReq::parse`, Cargo's own parser,
refusing one whose text does not parse naming the name, the range and
the parser's words, and answers the versions that parse as a `semver`
version and `matches` it, newest first by `semver`'s order; a version
that is not a Cargo version is within no range.

render holds exactly the crates given, checked first (ecosystem `cargo`,
one layer, bytes of that digest, the crate's manifest naming the
metadata's name and version, no two digests for one name and version),
then writes, under the place given:

```
index/<path>                 each name's index file, <path> as the sparse protocol names it
<name>-<version>.crate       each crate
```

Crates are written beside their place and renamed onto it; each index
file is written whole and renamed. Crate and index files of versions not
given are removed, so the layout holds exactly what it was given: a
version the policy refused is never given, so no index line lists it
(99). Index lines are sorted by `semver` (23). A relative place is under
the checkout the projection is bound in. A name's index file lists only
what the layout holds, so Cargo, resolving each requirement against it,
takes the version the core held and writes its pin.

Index line, from the normalized manifest, as built: `name`, `vers`,
`deps`, `cksum`, `features`, `features2` and `v`, `links`,
`rust_version`, `yanked` false, keys in a fixed order. A dependency's
`registry` is null where its `registry-index` is a served Cargo address;
the dependency's `registry-index` where it is another registry;
crates.io's index where it names none. Why null for every family
repository's address: the packed crate names its dependency at the
publisher's address, a consumer reads at its own, and its own resolves
any family crate by name and version (138), so the dependency is one of
the registry serving the line. The layout names no address:
`config.json` is the server's (D9), so one layout serves under any base.

### D9. Cargo's serve, and a refusal three ways

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
from the name's index file, since render is never given it (D7, D8); a
request for its bytes, from Cargo or anyone, is a miss whose refusal is
answered `403` naming the version and the claim; and `tailrace sync`
answered the refusal when it rendered the layout (render under a
policy). Cargo itself resolves from the index before it downloads, so in
a build the words a person reads are sync's, beside Cargo's "failed to
select a version".

### D10. Bound with a store of its own

`profiles::projection::bound` takes a second store for Cargo; the
bindings bind the store once more for it, as they bind `cache` for the
moon cache, since a `Box<dyn Store>` is not shared (core.2).


### D11. Tests, and the proof as one loop

Unit tests in the profile build crates in memory with `flate2` and
`tar` for the index line, `requires`, render and serve, read lockfiles
and workspaces written as text for pinned, and give `within` ranges and
versions as text; they run in `check-fast` with no cargo needed. The
core's render is tested over the fakes in
`crates/tailrace-core/tests/render.rs`, the fake projection answering
the pins and ranges a test sets, and `requires` and `within` from what
a test sets for each artifact and range.

The layouts group of the conformance suite gains two scenarios over the
fakes, citing 137: a range held at the newest admitted version within
it, a newer refused one answered with its claim and one outside the
range neither held nor asked; and a pinned version's family dependency
held, and that one's in turn, each answered with the version requiring
it. 137's range part leaves the register in the commit that adds them
(core.33).

The conformance group and the proofs run cargo, so they sit behind the
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
  `base-c`, `0.3` and `0.4.0`; with the store naming no `base-c` at all,
  its listing empty, refused naming `base-c`, `0.3` and that the store
  holds none. With `fam-b` among no output of `fam`, the pack is refused
  naming `fam-b` and `fam`.
- `tests/cargo_served.rs`: the publisher's workspace, `fam-b` and
  `fam-a` depending on it by path, each sibling dependency written with
  a placeholder version and `registry = "tailrace"`, `.cargo/config.toml`
  naming `tailrace` at its served address; the build's packaging task
  stood in for by `cargo package --workspace --no-verify --offline
  --registry tailrace`; both packed at 1.0.0, 1.1.0, 1.2.0 and 2.0.0 and
  put in a layout store with their index, claims and
  `artifact/cargo/<name>@<version>` names as npm's served test puts its
  package, 1.2.0's index lacking a claim the consumer's policy requires.
  A consumer checkout with no `Cargo.lock`, whose `.cargo/config.toml`
  names `tailrace` at `sparse+<the test's base>/<owner>/<name>/cargo/`
  and whose manifest declares `fam-a = { version = "1", registry =
  "tailrace" }`. `tailrace sync`, with the host's process at the test's
  address, answers the range `fam-a` `1` holding 1.1.0, 1.2.0 refused
  naming the claim found wanting, and the range `fam-b` `1.1.0` from
  `fam-a` 1.1.0 holding 1.1.0, `fam-b` 1.2.0 refused the same way; the served index files of `fam-a` and
  `fam-b` each list 1.1.0 alone; a `GET` of `fam-a` 1.2.0's download
  answers `403` carrying the same words; `cargo build` succeeds and
  writes a `Cargo.lock` pinning `fam-a` and `fam-b` at 1.1.0 from the
  served address with the packed crates' checksums, and no path. Then
  `tailrace sync` again answers both as pins held, and `cargo build
  --locked` succeeds and leaves the lockfile as Cargo wrote it.

## Risks / Trade-offs

- [The store's listing grows with every version published] → a pack
  reads it once per crate of another component it reaches; a render
  sync lists a name only for a range no held version meets, and judges
  only the versions within it, newest first, stopping at the first
  admitted.
- [A lockfile pin that no longer satisfies a held crate's requirement]
  → the range is not met, and the newest admitted version within it is
  held beside the pin, so Cargo's resolution still closes.
- [A refused newest version answered on every sync] → the exit counts it
  only where its range holds nothing (D7); the words stay, as 99 asks.
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
  only for a dependency naming the registry `tailrace`, and `requires`
  answers only a dependency naming a served address.
- [Code waits on three units] → the first code task checks their seams
  are on the branch and stops through crew where any is missing.

## Open Questions

None.

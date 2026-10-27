## Context

This unit's code starts once bolt tailrace-the-gate-and-the-channels has
landed on `main` and `main` is taken into bolt/tailrace-sync-and-serve,
and once the-projection-answers-what-an-artifact-requires and
the-store-names-an-artifact-by-ecosystem-name-and-version, which it comes
after, are on this bolt. Its ranges on pinned and the closure of a miss
(D7's second half, and the closure in D4) wait besides on
cargos-registry, which runs beside it and owns the range a pin carries
and the core's loop. It is written against them as they stand:
render-under-a-policy as merged on bolt/tailrace-sync-and-serve at
`71f8841`, the store-naming unit's change at `c41eb9c`, the planner's
proposal 154 for the requires unit, which has no change yet, and the
serving process as it is on bolt/tailrace-the-gate-and-the-channels at
`0172321`. The statement is `agentplot/blueprints` `f90f5fd`, with 137
and 140 as ruled at `4c14235`.

What it builds on:

- **The serving process** (`crates/tailrace/src/commands/serve/`, the
  `serving-process` spec): `serve_on(address, host, idle)` holds
  `serve.lock` in the state directory, answers HTTP/1.1 one request per
  connection on a thread each, `GET` only, `Connection: close`, with a
  `Response { status, content_type, body: String }`.
  `respond_with(host, bind, method, target)` routes `/serving.json`,
  `/status.json`, `/` and `/<owner>/<name>/status[.json]`.
  `host::name(host, repository, checkout)` writes the host's file of
  repositories; `host::checkout_of` reads the checkout that last named
  one; `host::find_or_start(address, start)` finds the process by its
  `/serving.json` or runs `start` once. `status::page` names the
  repository from the landing's remote with `RepositoryName::from_remote`
  and calls both. `tailrace-core/src/served.rs` holds `SERVING_HOST`
  `127.0.0.1`, `SERVING_PORT` 7420 and `RepositoryName`.
- **Render under a policy** (archived on the bolt as
  `2026-10-26-render-under-a-policy`, its code in
  `crates/tailrace-core/src/render.rs`): pinned on the seam
  (`Pinned { name, version }`), whose doc says a name the lockfile does
  not pin is no pin until ranges resolve; `render::render(seams, layouts,
  &RenderAsk { ecosystem, pins, own, trusted, tree, when }) ->
  Answer<Refreshed>` reads and judges every pin before it writes
  anything: `locate` resolves `artifact/<ecosystem>/<name>@<version>`
  to the index and the artifact, `own_policy(policy, landed)` answers
  the consumer's own policy for layouts at the landed revision,
  `policy_for` answers the policy that judges a version where the
  consumer has none, from the publisher's layout-policy claim counted
  through the signers its file names, else the host's trusted ones (118,
  C.10), and `judge` holds the version to every claim that policy
  requires; the private `judged(seams, &RenderAsk, &Pinned)` does all
  four for one pin. The projection then
  renders exactly the held artifacts at `place(tree, ecosystem)`,
  `<working tree>/.tailrace/layouts/<ecosystem>/`, and `Layouts` writes
  the record, `<git dir>/tailrace/layouts/<ecosystem>.json`, whose
  `held` entries name each version and its index and artifact digests.
  `render::layouts` and `render::sync_checkout(seams, record,
  &CheckoutAsk { branch, revision, tree, when, trusted })` stand behind
  `tailrace sync`, which takes `trusted` from `binding.trust`.
- **The projection seam** (`crates/tailrace-seams/src/projection.rs`):
  `serve(&self, &Layout) -> Answer<Serving>`, where `Serving` is an
  address. The file and image profiles answer serve absent, not built;
  the moon cache's likewise. The conformance group's scenario "served
  where the clients read no files" asserts a non-empty address.
- **The store's listing** (the store-naming unit's D7): `Store::versions(
  &Ecosystem, &EcosystemName) -> Answer<Vec<Version>>`, every version the
  store names `artifact/<ecosystem>/<name>@` at, in no order the core
  relies on, empty where it names none.
- **requires and the core's loop** (137, 140, ruled at `4c14235`;
  proposal 154): the projection seam gains one question, given a held
  version's artifact, the family's names and ranges its manifest
  requires, as it spells them, and whether a version satisfies a range
  by the ecosystem's own order; a projection with no manifest answers
  none. the-projection-answers-what-an-artifact-requires adds it to the
  trait, with the fakes' answer and the absent answer of the file, image
  and moon cache projections, and lands before this unit. cargos-registry
  owns the range a pin carries (pinned answering a pin or a name and its
  range) and the loop in `render.rs`: for each range, list the name's
  versions through the store, take those within it newest first by the
  projection's order, judge each under the layout's policy until one is
  admitted, hold it; then ask requires of every version held and do the
  same, until nothing held requires a name the layout lacks. The core
  never reads a manifest (core.1). This design names the operations by
  what they do; their Rust names and types are the requires unit's and
  cargos-registry's. As cargos-registry's change spells them today, they
  are `requires(&Artifact) -> Answer<Vec<Ranged>>` and `within(&Ranged,
  &[Version]) -> Answer<Vec<Version>>`, newest first, and npm answers
  whichever shape the requires unit lands.
- **The bindings**: `profiles::projection::bound(store, checkout)` builds
  every projection, the moon cache given a store opened with
  `profiles::store::bind(store, false)`; `Host::bind` binds a checkout
  `For::Status`.

npm's own behaviour this rests on, read from its references: `npm pack`
takes `--json`, `--pack-destination` and `--ignore-scripts` (docs.npmjs.com,
npm-pack, v11); `npm pkg set <key>=<value>` edits `package.json`; a
packument's `versions[v].dist` carries `tarball`, `shasum` and, since
2017, `integrity`, and `dist-tags.latest` is always defined
(npm/registry, package-metadata.md); npm-registry-fetch builds an HTTP
error's message as `<status> <statusText> - <METHOD> <uri> - <body.error>`
where the body is a JSON object (`lib/errors.js`); npm escapes a scoped
name's `/` as `%2f` in the packument's path. npm 7 and later install a
package's `peerDependencies` unless `peerDependenciesMeta` marks one
optional, and never a dependency's `devDependencies`; a name in
`bundleDependencies` comes inside the tarball. An install from a
complete lockfile fetches each tarball from its `resolved` address
without asking a packument. A scope's registry is the `.npmrc` line
`@<scope>:registry=<url>`; a project's `.npmrc` sits beside its
`package.json` (docs.npmjs.com, npmrc, package-json, v11). npm's version
and range rules are node-semver's; the `nodejs-semver` crate, 4.2,
ports them: `Range::parse` reads `^`, `~`, `x`, hyphen and `||` ranges
and `*` or empty as any; `Range::satisfies(&Version)` leaves out a
prerelease unless a comparator names a prerelease of the same major,
minor and patch, as npm does; `Version` orders by semver precedence.

## Goals / Non-Goals

**Goals:** the npm profile's five operations, a manifest's ranges and
requires among them; serve reshaped to answer one request inside the
host's one process; the process routing served layouts, and staying
while one is served; the core's render of one missed version, closed
over what it requires, and the layout's lock; sync naming and starting
the process; npm pinned and its tests in the tools layer.

**Non-Goals:** serving any other ecosystem; the range type on the seam
and the core's loop, cargos-registry's; requires on the trait, the
requires unit's;
`.npmrc` written by tailrace (the consumer writes its scope line once,
S4); the store's `artifact/` pointer and listing.

## Decisions

### D1. serve answers one request, and asks the core for a miss

```rust
pub struct Request { pub method: String, pub path: String, pub base: String }
pub struct Response { pub status: u16, pub content_type: String, pub body: Vec<u8> }
pub type Miss<'a> = &'a dyn Fn(&Pinned) -> Answer<()>;
fn serve(&self, layout: &Layout, request: &Request, miss: Miss<'_>) -> Answer<Response>;
```

`path` is the request's path below the layout's prefix, query dropped;
`base` the absolute address of that prefix as the client reached it,
with no trailing `/`. `miss` asks the core to render one version into
`layout`; `Ok(())` means the layout now holds it. Every answer the
client should read, a refusal's included, is an `Ok(Response)`; `Err` is
kept for a projection that serves none, `Not::absent(source, "serving")`
for every request, and for a layout the projection cannot read at all.
`Serving` leaves the seam. `Request` and `Response` round-trip through
JSON, the body as base64, as every seam type does.

Why: one process per host (114) leaves a projection nothing to bind, so
an address answers nothing; the client's protocol is the ecosystem's
(core.1), so the projection words every response, a refusal's too (99);
and the judgment is the core's (core.2, ADR 0006), so a miss is a call
back into the core rather than a store read in the profile.
Alternatives: serve answering `Lacks(Pinned)` for the process to render
and ask again, which needs a second operation to word the refusal in the
client's form, and the seam has four (B.6); a trait object answering
requests, which is a second interface for the seam (core.2).

The scenario "served where the clients read no files" sits above the
group's marker and asserts an address. It is changed to ask serve for the
rig's request of a held version and to check the response is an answer,
citing 86 as before: the operation it holds changed shape, and this is
the first unit that serves. The marker's rule keeps units of one bolt
apart; no other unit of this bolt touches the scenario.

### D2. The process routes a served layout by its ecosystem's name

`respond_with` gains one route: a path of four or more parts
`/<owner>/<name>/<ecosystem>/<rest>`, or exactly `/<owner>/<name>/<ecosystem>`
with `rest` empty, where `<ecosystem>` is any name; the status routes
keep exactly three parts and are matched first. For it the process:

1. finds the checkout with `host::checkout_of` (absent where none named
   it, as status);
2. binds it `For::Sync` (`Host::bind_for(checkout, For::Sync)`, beside
   `bind`);
3. takes the projection bound under `Ecosystem::new(<ecosystem>)`, `404`
   naming the ecosystem where none is;
4. asks `serve(&Layout { at: <working tree>/.tailrace/layouts/<ecosystem> },
   &Request { method, path: rest, base }, &miss)` where `base` is
   `served::layout_address(repository, ecosystem)` without its trailing
   `/`, and `miss` calls `render::miss` (D4) in that checkout;
5. answers the projection's `Response` as given; `Err(absent "serving")`
   is `404` naming the ecosystem; any other `Err` is the status the
   process gives every answer (`status_of`) with the words in text.

The route names no ecosystem (core.1). `Response.body` becomes `Vec<u8>`
and `Content-Length` its byte length; `reason` gains `403 Forbidden` and
`502 Bad Gateway`. The process's own answers before the projection, an
unnamed repository or an unbound checkout, stay text, as every other
path's are.

Why from the checkout that last named the repository: the path names the
repository alone, since a lockfile in git names it for every checkout
(49), and the host's file already says which checkout answers for it
(132). Alternative: a host-wide layout per repository under the state
directory, which contradicts the layout's place in the checkout that
render-under-a-policy fixes.

### D3. The served address is the core's

`served.rs` gains `origin() -> String`, `http://127.0.0.1:7420/`, and
`layout_address(&RepositoryName, &Ecosystem) -> String`,
`http://127.0.0.1:7420/<owner>/<name>/<ecosystem>/`, both from the
constants already there. The process's base, sync's answer and the tests all take
it from there, so the address is spelled once (114, core.36).

### D4. A miss renders one version into the layout

`render::miss(seams, &Layouts, &MissAsk { branch, revision, ecosystem,
tree, pin, trusted, when }) -> Answer<HeldPin>`, beside `render::render`
in `render.rs`:

1. takes the layout's lock (D5);
2. reads the record; a pin its `held` names already answers held at
   once (a second request raced the first);
3. checks the revision landed (`Landed::check`), reads the consumer's
   own policy with `own_policy`, and reads and judges the one pin with
   `judged`, exactly as `render` judges a pin; `revision` is what
   `tailrace sync` with no argument takes, `history.base(main)` in that
   checkout, and `trusted` the checkout's binding's `trust`;
4. where held, hands the projection's render the record's held
   artifacts, each `Artifact { digest, meta }` rebuilt from its entry's
   name, version and artifact digest, and this one, writes the record
   with this version added to `held` and its other entries as they were,
   and answers held; where absent, refused or unreadable, writes nothing
   and answers that.

The policy is found as `render` finds it, by the same functions, so 113's
order holds for a miss as for a sync: the consuming repository's own
where its file writes one, else the publisher's layout policy the index
carries, counted through the host's trusted signers, else the given
(C.10). Whatever render-under-a-policy builds of that order, a miss
holds a version to the same.

It reads the store for the one pin, and then for what it requires: the
held versions were judged when they were rendered, and the record names
their digests (48, 50). Once the pin is held, the miss closes the layout
over it by the same loop sync's render runs (cargos-registry's), asking
the projection what the version requires and holding the newest
admitted version of each family name the layout lacks, through every
level, before the projection renders and the record is written; a
required name none of whose versions is admitted leaves the pin held
and is answered as sync answers it (137). With a lockfile, npm fetches
each tarball it pins from its `resolved` address, so the closure seldom
finds anything to add; it holds a miss to the rule every held version
keeps. The process maps the answer to `miss`'s `Answer<()>`.

### D5. One render of a layout at a time

`Layouts` gains `lock(ecosystem) -> Answer<LayoutLock>`, an exclusive
`std::fs::File::lock` on `<git dir>/tailrace/layouts/<ecosystem>.lock`,
held by `render::render` from its first read to the record's write,
and by `render::miss` from the record's read to its write, and let go
on drop. A sync waits on a miss and the reverse.
Why a file lock: the process and a sync are two processes; the lock is
the host's, as `serve.lock` and `repositories.lock` are.

### D6. npm's pack

`NpmProjection::pack(output, build, Some(version))`, in the signature
pack has when this code starts, which cargos-registry widens with the
output's component; npm reads only the version from it:

1. the output's files' shallowest `package.json` names the package
   directory; none is refused naming the output;
2. the files the output lists under that directory are copied, with
   their relative paths, into a fresh temporary directory, so the build's
   files are never edited;
3. `npm pkg set version=<version>` there, then `npm pack --json
   --ignore-scripts --pack-destination <out>`, each with
   `npm_config_update_notifier=false` and `npm_config_fund=false`; a
   non-zero exit is refused with npm's standard error verbatim, an npm
   that cannot be run unreadable naming `npm` with the system's words;
4. the tarball named by the JSON's `filename` is the bytes; its `name` is
   the metadata's name, and a `version` other than the one given is
   refused naming both;
5. `same_again: false`: npm fixes its tarballs' times, but no reference
   promises the same bytes across npm versions (87). A repeat publish
   still compares (18).

With no version: `no_cache_entries`. `npm` is the one on `PATH`; the
devenv shell's is the pinned one (D12).

### D7. npm's pinned

The lockfile's pins: `npm-shrinkwrap.json` where it exists, else
`package-lock.json`, at the checkout's root. `lockfileVersion`
2 and 3: every entry of `packages` but `""` and those with `link: true`,
named by the entry's `name` where it has one (an alias) and otherwise by
the key's part after its last `node_modules/`, at its `version`,
deduplicated and sorted. Version 1 is unreadable naming the file and
`lockfileVersion 1`, which npm 7 and later rewrite. A parse failure is
unreadable with the parser's words.

Then the ranges (137): the checkout's root `package.json` and, where its
`workspaces` names any (an array, or an object's `packages`), each
workspace's `package.json` its globs match. From each,
`dependencies`, `devDependencies`, `optionalDependencies` and the
`peerDependencies` npm installs, every name that is the family's (D13),
is not a workspace's own name, and is not pinned by the lockfile at any
version, is answered once as that name and its range as the manifest
spells it; an `npm:<name>@<range>` alias is answered as its target's name
and range. A spec npm does not read as a range is no range: `latest` is
read as `*`, since the packument's `latest` names the newest version
held (D8); a path, `git`, URL or other tag names no version a layout
can hold and is passed over. Two manifests asking one name answer each
range they ask. A `package.json` that does not parse is unreadable with
the parser's words.

pinned is absent naming `lockfile` where the checkout holds neither
lockfile nor a family range, as Cargo's is: a checkout with no npm
lockfile and nothing of the family to ask for has nothing for a layout
to hold.

### D8. npm's layout

Under the layout's place:

```
<name>/index.json                    the packument
<name>/-/<base>-<version>.tgz        each tarball
```

`<name>` is the package name, `@scope/pkg` two directories deep, and
`<base>` its part after `/`. The projection reads each artifact's
manifest from its store (`Manifest::Artifact`, one layer) and the layer's
bytes, checks them against the layer's digest, and writes the tarball
where one of that digest is not already. The packument is
`{ name, "dist-tags": { latest }, versions: { <v>: <package.json> +
dist } }`, `dist` holding `tarball` as the path relative to the layout,
`integrity` as `sha512-<base64>` and `shasum` as hex SHA-1 of the
tarball; `package.json` is read from the tarball's `package/package.json`
with `flate2` and `tar`, never written back (C.11). `latest` is the
highest held version by `semver` order, a prerelease only where no
release is held. A refused version is never given, so no packument
lists it: a packument carries no words for a version it lacks, so the
refusal reaches npm in the tarball's response (D9) and the developer in
what sync prints (99). A name no artifact given names is removed from the
layout, and a tarball no packument names is removed, so the layout holds
what it was given (47). Files are written beside and renamed.

The packument names no address: the base is the server's (D9), so one
layout serves under any base and names none (49).

### D9. npm's serve

The request's `path`, its `%2f` and `%40` unescaped:

- `<name>` (one part, or `@scope/pkg`): the packument from the layout,
  each `dist.tarball` made `<base>/<relative path>`, `200`
  `application/json`; no packument is `404`, its words saying the layout
  holds no version of the name and that a name `package.json` declares
  is held by `tailrace sync`. A packument never asks the core: it names
  no version, so nothing is missed (50), and a name the manifest
  declares with no pin is sync's to resolve (137).
- `<name>/-/<base>-<version>.tgz`: the tarball, `200`
  `application/octet-stream`; a tarball the layout lacks calls `miss`
  once with `Pinned { name, version }` and, on `Ok`, answers the
  tarball now in the layout.
- anything else, `404`; a method other than `GET` or `HEAD`, `405`.

Every answer that is not the bytes asked is `application/json`
`{"error": <words>}`, so npm prints the words after its status line:
`404` absent, `403` refused, `502` unreadable, the words being the
`Not`'s own rendering, the source's words verbatim (58, 59, 99).

### D10. Sync names the repository and starts the process

`CheckoutAsk` gains `repository: Answer<RepositoryName>`, which the
command reads from the landing's remote as `status::page` does, that
reading moved into `host::repository_of(host, checkout)` and called by
both. For each layout rendered or kept, the core asks the projection
`serve(layout, &Request { method: "GET", path: "", base }, &|_| absent)`
once: absent naming serving means it serves none; anything else that it
serves. The layout's answer gains `served: Option<Answer<String>>`,
`layout_address` or the repository's `Not`. Then the command, where any
layout's `served` is an address, calls `host::name` and
`host::find_or_start(address, detached)` exactly as `status::page` does,
and its answer is printed after the core's as `serving`, in JSON as a
`serving` field beside the core's answer flattened. The exit counts
`served` and `serving` as it counts a layout's answer. Sync takes the
host, the address and `start` as parameters, so its tests run on an
address of their own and start nothing real.

### D11. The process stays while a layout is served

`serve_on` keeps its idle time, and where it elapses asks
`host::serves_a_layout(host, bind)` before it ends: for each checkout
`host::named` answers, bound `For::Sync`, whether any bound projection
whose serve is not absent naming serving (the probe of D10) has a
rendered layout, its place `render::place(tree, ecosystem)` present
with its record. Where one has, the idle time starts again and the
process answers on; where none has, it ends as before. A checkout gone
or unbindable counts as holding none, so removing a worktree, or the
layout's directory, lets the process end at its next idle (114). The
check reads files and opens no connection (core.32). `bind` is a
parameter, as `respond_with`'s is, so the tests run it over the fake
projection; `serve_on` passes `Host::bind_for`.

Why at idle and not on each request: the answer changes only when a
sync renders or a person removes a layout, and the process need not
know sooner than its idle time. Alternative: sync writing a mark in the
host's file that a layout is served, which a removed worktree would
leave behind, keeping the process for a layout that is gone.

### D12. npm pinned, and where its tests run

`devenv.nix` adds `pkgs.nodejs` to `packages`, pinned by `devenv.lock`
(core.26). Tests:

- `crates/tailrace/src/profiles/projection/npm/` unit tests for pinned, requires,
  render and serve over tarballs a test makes with `tar` and `flate2`, no
  npm needed, in `cli:test` (`check-fast`);
- `crates/tailrace/tests/npm_projection.rs`: the projection group over
  npm; without the `tools` feature every scenario but the pack ones,
  `PATH` holding no npm for the unreachable one; with it the pack ones
  (as `projections.rs` splits the file and image groups);
- `crates/tailrace/tests/npm_served.rs` behind `tools`: a consumer
  checkout whose policy file names the publisher's host key signer for
  provenance, since the givens name no signer (116, C.10), a layout store
  holding published `@family/widget` versions, each requiring
  `@family/base`, and `@family/base` versions, with their claims and
  `artifact/` names, `[layout]` requiring provenance, `.npmrc` naming the
  served address for `@family`, `package.json` declaring
  `@family/widget` by a range and no lockfile, `serve_on` at a test
  address over a temporary host; `tailrace sync` then `npm install`
  under a home and `npm_config_*` of its own, which writes the lockfile;
  then a second sync from that lockfile and `npm ci`; a refused and an
  absent version; and two worktrees sharing one process. Added to the
  `tools` task's `--test` list in `moon.yml`.
- `crates/tailrace/tests/serve.rs` for the route (D2) and the stay
  (D11) with the fake projection, the latter at an idle of moments;
  `crates/tailrace-core/tests/render.rs` for D4 and D5.

### D13. A family package is one the checkout's `.npmrc` sends to the host

140 asks the projection for the family's names, and the statement says
no more of what makes a name the family's in an ecosystem. In npm it is
the scope the consumer's own `.npmrc` sends to the host's one process:
npm reads such a name from a layout and from nowhere else, so the layout
must hold it, and npm reads every other name from its own registry, so
the layout must not (section 8). The projection reads the `.npmrc`
beside the checkout's root `package.json` once per operation, takes each
`@<scope>:registry=<url>` line whose URL begins with the process's
address, `served::origin()`, `http://127.0.0.1:7420/`, which the binding
hands it, and reads nothing else of the file; no credential in it is
read or kept (core.21). A name of such a scope is the family's; no other
name is, and a checkout with no `.npmrc` has no family. The lockfile's
pins are answered whatever their scope, as render-under-a-policy answers
them, and the core answers a third-party pin absent.

Why the scope and not every name: a manifest names its third-party
packages too, and listing each in the store at every level would ask
the store hundreds of names it never holds; Cargo's projection answers
only dependencies of the registry `tailrace`, which is the same rule.
Alternative: a list of family scopes in the policy file, which the
statement does not name, and which the `.npmrc` line already says.

### D14. npm's requires, and its order

Given a held version's artifact, the projection reads its tarball by the
layer's digest through its own store, checked against that digest as
render does (D8), and its `package/package.json` with `flate2` and
`tar`. It answers each family name (D13) of `dependencies`,
`optionalDependencies` and the `peerDependencies` npm installs, passing
over a name `bundleDependencies` lists, with its range as written, an
alias read as in D7; a spec that is no range is passed over or read as
`*` as in D7. A tarball the store does not hold, or one with no
`package.json`, is unreadable naming the artifact.

Whether a version is within a range is `nodejs_semver::Range::parse`
then `satisfies`, a version or range that does not parse being within
nothing unless the trait's contract words a range it cannot read
otherwise; versions are ordered newest first, or the newer of two
given, by `nodejs_semver::Version`'s order (23), whichever form the
trait asks. Both are npm's own rules, so the version the core holds for
a range is the one npm would pick from what the layout offers, prereleases
left out as npm leaves them.

## Risks / Trade-offs

- [A host with a served layout keeps its process for good] → the
  clause's choice (114): npm cannot start it, so it stays while a layout
  is served, and ends once the layout's directory, or its checkout, is
  removed. The check binds each named checkout once per idle period,
  which is minutes, never per request.
- [A host that rebooted] → nothing runs the process until an asker
  starts it: the next `tailrace sync`, `tailrace status --page` or a
  pane; an `npm install` before any of them fails with npm's connection
  refused. A host may run `tailrace serve` as a service of its own (114).
- [The served layout is the last-named checkout's] → a second checkout's
  pins its layout lacks are rendered into it on the miss, and leave at
  that checkout's next sync; the lockfile is the same either way, so
  installs agree.
- [A family package named on npm's command line first] → `npm install
  @family/new`, where `package.json` does not yet declare it, answers
  `404`, since npm asks the packument first and the layout holds none for
  the name until a sync reads the manifest's range. The `404`'s words say
  the layout holds no version of the name and to declare it and sync, so
  the developer sees that it is the layout's and not the public
  registry's; the site says to declare it with `npm pkg set` and sync.
- [A scope the `.npmrc` sends elsewhere] → its names are not the
  family's, so no range or requirement of them is held, and npm reads
  them where the line sends it. A consumer who forgets the line installs
  from the public registry, as before this unit.
- [npm retries a `502`] → npm's fetch retries 5xx with backoff before it
  fails; the words still reach the user, later.
- [A miss judged under a newer landed policy than the layout's] → the
  miss's judgment is the binding one (107's reasoning); the next sync
  brings the rest of the layout under it.
- [`flate2` and `tar` are new] → used only to read `package.json` from a
  tarball the store holds; nothing is written with them.

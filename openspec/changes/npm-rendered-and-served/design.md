## Context

This unit's code starts once bolt tailrace-the-gate-and-the-channels has
landed on `main` and `main` is taken into bolt/tailrace-sync-and-serve,
and once render-under-a-policy and
the-store-names-an-artifact-by-ecosystem-name-and-version are on this
bolt. It is written against them as they stand: render-under-a-policy as
its revised change says at `34627b0`, whose code is on its branch, and
the serving process as it is on bolt/tailrace-the-gate-and-the-channels
at `0172321`. The statement is `agentplot/blueprints` `d91401e`.

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
- **Render under a policy** (`render-under-a-policy` design and its
  code in `crates/tailrace-core/src/render.rs`): pinned on the seam
  (`Pinned { name, version }`); `render::render(seams, layouts,
  &RenderAsk { ecosystem, pins, own, trusted, tree, when }) ->
  Answer<Refreshed>` reads and judges every pin before it writes
  anything: `locate` resolves `artifact/<ecosystem>/<name>@<version>`
  to the index and the artifact, `own_policy(policy, landed)` answers
  the consumer's own policy for layouts at the landed revision,
  `policy_for` answers the policy that judges a version where the
  consumer has none, from the publisher's layout-policy claim counted
  through the signers the host's bindings trust (`[trust]`, 118, C.10),
  and `judge` holds the version to every claim that policy requires; the
  private `judged` does all four for one pin. The projection then
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
name's `/` as `%2f` in the packument's path.

## Goals / Non-Goals

**Goals:** the npm profile's four operations; serve reshaped to answer
one request inside the host's one process; the process routing served
layouts, and staying while one is served; the core's render of one
missed version, and the layout's lock;
sync naming and starting the process; npm pinned and its tests in the
tools layer.

**Non-Goals:** serving any other ecosystem; a manifest's range answered
by pinned (137), which waits on render-under-a-policy carrying a range on
the seam's pin and resolving it by the store's listing (138, proposal);
`.npmrc` written by tailrace (the consumer writes its scope line once,
S4); the store's `artifact/` pointer.

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

`served.rs` gains `layout_address(&RepositoryName, &Ecosystem) -> String`,
`http://127.0.0.1:7420/<owner>/<name>/<ecosystem>/`, from the constants
already there. The process's base, sync's answer and the tests all take
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

It reads the store for the one pin alone: the held versions were judged
when they were rendered, and the record names their digests (48, 50).
The process maps the answer to `miss`'s `Answer<()>`.

### D5. One render of a layout at a time

`Layouts` gains `lock(ecosystem) -> Answer<LayoutLock>`, an exclusive
`std::fs::File::lock` on `<git dir>/tailrace/layouts/<ecosystem>.lock`,
held by `render::render` from its first read to the record's write,
and by `render::miss` from the record's read to its write, and let go
on drop. A sync waits on a miss and the reverse.
Why a file lock: the process and a sync are two processes; the lock is
the host's, as `serve.lock` and `repositories.lock` are.

### D6. npm's pack

`NpmProjection::pack(output, build, Some(version))`:

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

`npm-shrinkwrap.json` where it exists, else `package-lock.json`, at the
checkout's root; neither is absent naming `lockfile`. `lockfileVersion`
2 and 3: every entry of `packages` but `""` and those with `link: true`,
named by the entry's `name` where it has one (an alias) and otherwise by
the key's part after its last `node_modules/`, at its `version`,
deduplicated and sorted. Version 1 is unreadable naming the file and
`lockfileVersion 1`, which npm 7 and later rewrite. A parse failure is
unreadable with the parser's words.

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
  `application/json`; no packument is `404`. A packument never asks the
  core: it names no version, so nothing is missed (50), and a name the
  manifest declares with no pin is sync's to render (137).
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

- `crates/tailrace/src/profiles/projection/npm/` unit tests for pinned,
  render and serve over tarballs a test makes with `tar` and `flate2`, no
  npm needed, in `cli:test` (`check-fast`);
- `crates/tailrace/tests/npm_projection.rs`: the projection group over
  npm; without the `tools` feature every scenario but the pack ones,
  `PATH` holding no npm for the unreachable one; with it the pack ones
  (as `projections.rs` splits the file and image groups);
- `crates/tailrace/tests/npm_served.rs` behind `tools`: a consumer
  checkout whose policy file names the publisher's host key signer for
  provenance, since the givens name no signer (116, C.10), a layout store
  holding a published `@family/widget` with its claims and `artifact/`
  name, `[layout]` requiring its provenance, `.npmrc` naming the served address for
  `@family`, `serve_on` at a test address over a temporary host;
  `tailrace sync` then `npm install` under a home and `npm_config_*` of
  its own, a refused and an absent version, and two worktrees sharing one
  process. Added to the `tools` task's `--test` list in `moon.yml`.
- `crates/tailrace/tests/serve.rs` for the route (D2) and the stay
  (D11) with the fake projection, the latter at an idle of moments;
  `crates/tailrace-core/tests/render.rs` for D4 and D5.

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
- [A family package no lockfile pins] → `npm install @family/new` and
  `npm install @family/new@<version>` both answer `404`, since npm asks
  the packument first and the layout holds none for the name, until
  render resolves a manifest's range by the store's listing (137, 138;
  proposal). The `404`'s words say the layout holds no version of the
  name, so the developer sees that it is the layout's and not the
  public registry's.
- [npm retries a `502`] → npm's fetch retries 5xx with backoff before it
  fails; the words still reach the user, later.
- [A miss judged under a newer landed policy than the layout's] → the
  miss's judgment is the binding one (107's reasoning); the next sync
  brings the rest of the layout under it.
- [`flate2` and `tar` are new] → used only to read `package.json` from a
  tarball the store holds; nothing is written with them.

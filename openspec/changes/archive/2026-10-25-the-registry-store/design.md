# Design

## Context

The store seam is `tailrace_seams::Store`
(`crates/tailrace-seams/src/store.rs`): seven synchronous methods on
`&self`, each answering `Answer<T>`, absent, refused or unreadable
naming a `Source`. Every operation names a digest or a `Name`, never a
repository. `attach` answers the envelope's own digest,
`Envelope::digest()`, the digest of its canonical JSON.

The layout profile (`crates/tailrace/src/profiles/store/layout/`) is the
one store a host binds today. Its `oci.rs` writes an artifact as an OCI
image manifest (artifact type
`application/vnd.agentplot.tailrace.artifact.<ecosystem>`, config the
metadata as canonical JSON, layers the bytes) and an index as an OCI
image index with its component and version in annotations, each in one
canonical byte form. It keeps an attestation as a blob with an
`index.json` entry annotated with its subject (C.2). The host binds a
store through `profiles/store/mod.rs`'s `bind`, which matches the key of
the bindings file's `[store]` table.

The conformance suite's store group (`tailrace_conformance::seams::store`)
runs over a `StoreRig` (`store()`, `unreachable()`), seven scenarios
citing 1, 4, 5, 8, 10, 58, 59, 66 to 73. The register
(`crates/tailrace-conformance/src/register.rs`) names 103 and 109 with
this unit. The core groups run over `FakeWorld`, which holds a fake
store and a fake keeper. The ledger group (`core/ledger.rs`) is empty
below its marker, waiting for bolt 4.

Readings taken for this design:

- **OCI distribution specification (main).** No conditional request is
  defined for a tag put; the referrers section says only that clients
  "MAY use a conditional HTTP push for registries that support ETag
  conditions". The referrers API is `GET /v2/<name>/referrers/<digest>`,
  optionally `?artifactType=`, which a registry that applies it says in
  `OCI-Filters-Applied`, and a `Link` header pages it. A registry that
  takes a manifest with a subject answers `OCI-Subject`. Without that
  header, the client keeps the referrers tag
  (`<alg>-<hex>`) itself: pull it, start empty on 404, add the
  descriptor once (artifact type and annotations copied from the
  manifest), push it back. A tag matches
  `[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}`.
- **aws/containers-roadmap 2306** (open, last touched 2025-03-20): ECR
  rejects a manifest whose `artifactType` or layer media type carries a
  parameter (`;version=0.2`) with 405 `UNSUPPORTED` "Invalid JSON
  syntax"; AWS called it a bug in 2024. **2783** (open, 2026-03-06):
  ECR rejects an OCI 1.1 referrer manifest pushed by `oras copy -r`
  with the same 405, though ECR announced OCI 1.1 support in 2024.
  Neither says what in the manifest ECR objects to, so only a push
  answers it.
- **ECR's API reference.** `ListImageReferrers` filters by artifact
  types and lists only `ACTIVE` artifacts without one.
- **`docker_credential` 1.4.0** reads `$DOCKER_CONFIG/config.json` or
  `~/.docker/config.json`, honouring `auths`, `credsStore` and
  `credHelpers`, and answers an identity token or a user and password.
  **`ureq` 3.5** is blocking, rustls by default, and can be told not to
  turn 4xx and 5xx into errors.

## Goals / Non-Goals

**Goals:**

- The store group passing against zot and the reference distribution in
  the gate, and against a real ECR repository by hand as the bolt's
  acceptance, with no change to the seam, the suite or the fakes.
- A name at bytes' digest on a registry as on every store, so a cache
  entry named `cache/<build identity>` resolves to its bytes.
- One artifact, one digest, in a layout and in a registry, so a copy
  between them changes no digest and loses no claim.
- What ECR does with tailrace's manifests known from a push, and
  written down.

**Non-Goals:**

- A repository per ecosystem name (D2).
- Deleting anything from a registry, or pagination of tag lists: the
  store never lists tags.
- Chunked or resumable blob uploads: a blob is uploaded in one request.
- Choosing the store name a channel takes: bolt 4's. The repair takes
  the name from its caller.

## Decisions

### D1. One OCI form for both store profiles

`layout/oci.rs` moves to `profiles/store/oci.rs`, a module of the store
family that both profiles use; the layout changes only its `use` lines.
It gains the referrer manifest's form (D5). A module two profiles share
is not one profile calling another (core.2), and nothing in it names a
registry or a file.

Alternative: a second copy in the registry's directory. Rejected: a
byte of difference between the two is a digest of difference, and an
artifact copied from a layout to a registry would no longer be the same
artifact (1).

### D2. One declared repository on every registry

The binding names one repository; every blob, manifest, referrer and
tag lives there, as C.1 and ADR 0014 have it. put-bytes, read,
claims-of and resolve name only a digest or a name, and a registry
resolves a digest only within a repository, so a repository per
ecosystem name would need a map the store may not need (61, core.20)
or a catalogue search on every read.

Alternative: put every blob and manifest in the declared repository and
also mount each artifact into a repository named for its ecosystem
name. Rejected here: it is a second layout of the store that the
statement does not describe, and nothing in phase 1 reads it.

### D3. HTTP by hand over `ureq`, never through an OCI client library

The profile speaks the distribution protocol itself over `ureq`, with
4xx and 5xx answered as responses so every error body reaches a
refusal verbatim (59). It sends exactly the bytes `oci.rs` wrote and
hashes exactly the bytes it reads.

Alternatives: `oci-client`, which is async and re-serialises manifests
from its own types, so a manifest round trip could change bytes and so
digests; `oras` or `crane` as a subprocess, a further pinned tool whose
exit codes lose the registry's status and words.

The address is `https://<host>/v2/<repository>/…`; a loopback host
(`localhost`, `127.0.0.1`, `[::1]`) is spoken to over plain HTTP, as
Docker does, so the tests' zot needs no certificate and no flag exists
to turn TLS off elsewhere. Redirects on blob reads are followed (ECR
redirects to S3); the digest check is on what finally arrives.

### D4. The host's Docker credentials, reached on the registry's challenge

A request goes unauthenticated first. On a 401 the profile reads the
`WWW-Authenticate` challenge: for `Basic` it sends the host's
credential for the registry host, from `docker_credential` (ECR's
`ecr-login` helper among those it runs); for `Bearer` it asks the realm
for a token with that credential, for the scope the challenge names,
and retries the request once with it. The token is held in memory by
the one handle for the life of the process and written nowhere (64,
core.21). No credential found, or one the registry refuses, is refused
with the registry's words.

### D5. The operations

| operation | requests |
|---|---|
| put-bytes | `HEAD /blobs/<d>`: 200 is held already; else `POST /blobs/uploads/` and `PUT <location>&digest=<d>` with the bytes |
| put-artifact | each layer's size by `HEAD` (blob, or manifest for an index's artifacts), absent when not held; `oci::write`; the config by put-bytes; `HEAD /manifests/<d>`: 200 is held already; else `PUT /manifests/<d>` with the manifest's media type |
| read | `GET /manifests/<d>` accepting both manifest types, read by `oci::read` and the config blob; on 404 `GET /blobs/<d>`; 404 on both is absent; bytes whose digest is not `<d>` are refused `OtherBytes` (2) |
| attach | the subject's media type and size by `HEAD /manifests/<subject>`, absent on 404; the envelope's canonical JSON and the empty config `{}` by put-bytes; the referrer manifest by `oci::referrer`; `HEAD` it, held already on 200; else `PUT /manifests/<r>`; no `OCI-Subject` in the answer means the referrers tag (D6) |
| claims-of | the subject held, else absent; `GET /referrers/<d>?artifactType=<envelope type>`, following `Link`; 404 means the referrers tag; descriptors of other types dropped where `OCI-Filters-Applied` is missing; each referrer's manifest and its one layer read and parsed as an `Envelope` |
| resolve | `GET /manifests/<tag>`: 404 is absent; a manifest of the cache-entry type answers its one layer's digest (D14); any other answers its own digest, `Docker-Content-Digest` or the digest of the body |
| name | resolve the tag, refused `NameAt` naming what resolve answered unless that is the expected digest; `GET /manifests/<to>` for its bytes and media type; on 404, when `<to>` is held as a blob, the cache manifest of `<to>` (D14) written and put by its digest, else absent; `PUT /manifests/<tag>`; resolve again, refused `NameAt` naming what it found when that is not `<to>` (D7) |

The referrer manifest is an OCI image manifest: artifact type and its
one layer's media type the layout's envelope type
(`application/vnd.agentplot.tailrace.envelope.v1+json`), config the OCI
empty descriptor, subject the subject's descriptor, and, when the
envelope's payload type is in-toto and its statement names a predicate
type, the annotation `in-toto.io/predicate-type` holding it (C.1). A
predicate type is a URI
(`https://agentplot.github.io/tailrace/claim/verdict/v1`) and an
artifact type must be an RFC 6838 media type, which ECR enforces
(2306); typing every attestation by the envelope keeps claims-of one
list filtered by the envelope type, and the annotation keeps the
predicate type readable to other tools.

attach answers `Envelope::digest()`, which is the digest of the layer
it pushed, so the answer is the same as the layout's for the same
envelope.

### D6. The referrers tag

Where the registry answered no `OCI-Subject`, attach pulls the tag
`sha256-<hex>` for the subject (empty on 404, refused naming it if it
is not an image index), adds the referrer's descriptor unless it is
there, pushes the index back to the tag, and reads the tag again. An
entry missing on the read back is a writer that pushed over it: attach
answers unreadable naming the tag and the referrer, and pushes nothing
again (core.6). The referrer manifest is held by digest either way, so
a later attach of the same envelope adds the entry. claims-of falls
back to the tag on a 404 from the referrers API, and a tag that is
absent is no claims.

### D7. A tag checked before and read back after, with no conditional put

No registry this change binds honours a conditional tag put, and the
specification defines none, so name is a check, a put and a read back.
Two writers can each pass the check and each read back their own put
if their puts and reads interleave; the store cannot exclude that. For
a channel's name it does not matter: only the gate moves it, and only
after the keeper took its promotion at the chain's head, which two
promotions cannot both do (105, 120); the repair (D9) puts the name back
where the ledger says. For a version's name, the version's tag in the
shared history is claimed before anything is packed (22), so two
publishers never both reach the store with one version.

The profile exposes one hook for the tests, a closure run between the
put and the read back (hidden from the docs), so the tools layer can
lose a race on purpose.

### D8. A name escaped into a tag

A name's UTF-8 bytes in `[A-Za-z0-9.-]` stay; every other byte,
`_` included, becomes `_` and two lower-case hex digits; a leading `.`
or `-` is escaped the same way. The mapping is one-to-one, so two names
never share a tag, and version names (`lib-v1.3.0`) read as themselves.
An escaped name longer than 128 characters, or one in the referrers
tag's shape (`<alg>-<hex>`), is refused naming the name.

### D9. The repair from the ledger is the core's

`crates/tailrace-core/src/repair.rs` holds one function: given the
store, the keeper, a channel, the store name the channel takes and the
digest the caller expects it at, it reads the chain (absent and a fork
passed through), takes the head's claim (`Claim::from_envelope`; a head
that is not a promotion claim is unreadable naming it) and moves the
name to the head's subject. A `NameAt` refusal naming the head's
subject is answered as already there; one naming any other digest is a
lost race, answered with that digest after one move from it to the
head's subject, whose own refusal is passed through. It writes nothing
to the ledger. Bolt 4's gate calls it after its append; the ledger and
promotion unit decides the store name of a channel.

Alternative: the repair in the registry profile. Rejected: the profile
reaches no ledger, and every store, not only a registry, can lose a
race where a human or a second gate moves a tag (109, core.2).

### D10. The copy is the core's, through the seam

`crates/tailrace-core/src/copy.rs` copies one version, by its name, from
one `&dyn Store` to another: resolve the name in the source; read the
index; per artifact, read its manifest, put each layer's bytes, put the
artifact and compare digests; attach every envelope `claims-of`
answers on it; then the index, its envelopes, and the name made from
nothing. Its answer names every digest with whether the target held it
already, so a second copy is visibly a no-op. It names no profile, and
a channel is never copied (103). A command to run it is not this
change's (proposal); the tests call it.

### D11. Conformance

- A new core group, `copy` (`crates/tailrace-conformance/src/core/copy.rs`),
  over `FakeWorld`, its target a second fake store the scenario makes:
  a version published in the world copied whole with its claims, and
  copied again writing nothing. It cites 1, 8, 103, and 103 leaves the
  register.
- One scenario below the ledger group's marker: the world's keeper
  holding a promotion about one index, another writer moving the name
  to a second, and the repair reporting the lost race and moving the
  name back. It cites 109, which leaves the register.
- The store group is unchanged; the registry passes it as it stands.

### D12. The tools layer

`crates/tailrace/tests/registry/mod.rs` starts a zot and a reference
distribution on free loopback ports, each over a temporary directory
and a config the test writes, by the binaries `devenv.nix` names
(`TAILRACE_TOOLS_ZOT`, `TAILRACE_TOOLS_DISTRIBUTION`), and holds the
rig. On zot and the distribution each scenario gets a fresh repository.
On ECR the repository is the one the host names, so the rig gives each
run a prefix for every name it passes through, and the bytes the group
puts are held already from earlier runs, which the group allows.

- `tests/registry_store.rs` (the `tools` task): the group over zot and
  over the distribution; the spec's scenarios the group does not play
  (another tool's referrer, the referrers tag, a race lost after the put,
  escaped names, one digest in a layout and in zot, a credential
  helper, a missing credential); the copy from a layout to zot; the
  repair against zot with a fake keeper.
- `tests/registry_ecr.rs` (a `tools-ecr` task tagged `check-ecr`,
  outside the gate, core.38): the group over ECR, run by hand with a
  live AWS session as the bolt's acceptance, refusing to run naming `TAILRACE_TOOLS_ECR` when
  the host names no repository. The host names it, and the
  `AWS_PROFILE` ECR's helper reads, in `devenv.local.nix`, and maps the
  registry host to `ecr-login` in its Docker configuration.

Before the fallback test is written, the reference distribution at the
pin is checked to answer 404 on the referrers API; if it has gained the
API, the fallback is tested against a zot whose config disables it, or
is reported with `crew needs` if neither can be made to lack it.

### D13. One ADR

ADR 0014 already records one declared repository, the envelope type
with the predicate type annotated, and the cache manifest. This change
adds one record with the next number `adrs.toml` gives: the tag checked
before and read back after (D7), the host's Docker credentials reached
on the challenge (D4), and what a push against ECR answered on each
issue the registry-store spec lists, with the date and region, citing
ADR 0014 for the rest. The issues the store group cannot show (a
parameterised media type, tag immutability, create on push, a
lifecycle rule expiring untagged manifests) are read by hand against
the same account and recorded, since they are ECR's behaviour outside
anything the store sends. The record waits on the ECR run, the bolt's
acceptance (D15).

### D14. A digest held as bytes is named through a cache manifest

A tag names a manifest, never a blob, and a cache entry is bytes named
`cache/<build identity>` at their own digest (52, 100). When name is
asked to name a digest the repository holds only as a blob, the profile
writes the cache manifest of that digest and tags it, as C.1 and ADR
0014 decide. `oci.rs` writes it in one canonical byte form: an OCI
image manifest whose artifact type is tailrace's cache-entry type,
`application/vnd.agentplot.tailrace.cache.v1`, whose config is the OCI
empty descriptor, whose one layer is the bytes' descriptor with media
type `application/octet-stream`, and with no annotation and no subject.
The same bytes therefore always write the same manifest, which a second
naming finds held already.

Every place the profile reads a tag answers the digest the caller gave:
resolve, the check before the move, the read back after it, and the
digest a `NameAt` refusal names are all the layer's digest when the tag
is at a cache manifest. A name moved from a cache entry to an index, or
back, is checked against what the caller passed as expected, never
against the cache manifest's digest, which no caller sees.

The cache-entry type is neither the index type nor an artifact type, so
`oci::is_tailraces` is false for it and read of the cache manifest's
own digest answers bytes, never an artifact; its type is not the
envelope type, so claims-of never lists it (53, core.30). Its config
is the empty descriptor, which attach already puts, so naming bytes
puts the config with put-bytes before the manifest. A registry's
retention may remove an untagged cache manifest with its blob; nothing
accepted depends on either (54).

Alternative: refusing to name a blob. Rejected by ADR 0014: bolt 3's
replay from ECR rests on naming a cache entry, and every other store
names bytes.

### D15. ECR is the bolt's acceptance

The gate proves the profile against zot and the reference distribution,
each started for the test; ECR is proven once, by hand, as the bolt's
acceptance, because only a person's live session reaches it (core.38).
The code stage builds the `check-ecr` task and leaves the ECR group of
tasks unticked. At the bolt's acceptance the team's ops runs it with the
user's live session, on a host whose `devenv.local.nix` names the ECR
repository and the `AWS_PROFILE` its helper reads; the repository is
the host's, never the change's. What that run answers decides the rest
of the group: a form ECR refuses changed in `oci.rs`, the ADR's ECR
readings, and ECR's mark on the stores page moved from `designed` to
`built`. A run that fails is the acceptance failing, reported with
ECR's words, and the bolt waits on it.

## Risks / Trade-offs

- [ECR rejects tailrace's manifest or referrer form (2783)] → the ECR
  run is done before the site and the ADR are written; a form ECR
  refuses is changed in `oci.rs` for both profiles together, and the
  layout's tests say whether its existing digests change, which the
  ADR records.
- [ECR is proven only when someone runs it] → the gate cannot hold a
  person's AWS session (core.38), so ECR is proven by hand under
  `check-ecr` as the bolt's acceptance (D15); the task refuses to run
  naming what is missing rather than passing, and the ADR records the
  date of the run it rests on.
- [ECR refuses the cache manifest] → the same as any other form ECR
  refuses: changed in `oci.rs`; the layout writes no cache manifest,
  so no layout digest moves.
- [An ECR lifecycle rule expiring untagged images would remove
  artifacts only an index names] → the ADR names the rule an operator
  must not set on a store's repository.
- [Each ECR run leaves tags behind] → names carry the run's prefix; a
  lifecycle rule on the test repository alone may expire them by
  prefix.
- [Concurrent attaches on a registry with no referrers API can drop an
  entry] → reported unreadable, never silently lost, and repaired by
  attaching again (D6).

## Migration Plan

None: a new profile beside the layout, bound only by a host that names
it. A host bound to a layout is unchanged; the layout's bytes are
unchanged unless the ECR reading changes the shared form, which the ADR
records.

## Open Questions

None.


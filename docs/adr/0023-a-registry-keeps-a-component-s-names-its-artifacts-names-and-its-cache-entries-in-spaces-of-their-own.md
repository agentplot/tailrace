---
number: 23
title: A registry keeps a component's names, its artifacts' names and its cache entries in spaces of their own
status: accepted
date: 2026-10-27
decision-makers:
- Chuck Swanberg
---

# A registry keeps a component's names, its artifacts' names and its cache entries in spaces of their own

Status: accepted by the user on 2026-11-04, on their card.

Deciders: proposed by the the-store-names-an-artifact-by-ecosystem-name-and-version unit of phase 1's bolt 5, which lets a consumer find what its lockfile pins by ecosystem, name and version alone (`agentplot/blueprints:design/tailrace/models/phase-1-bolts.md`, Bolt 5), on the design agent's batch 25, 26, 27 and 29 rulings, for the user to accept with the change.
Sources: statement clauses 1, 3, 8, 52, 54, 58, 59, 61, 70, 109, 125, 135, 138 and C.1, as amended at blueprints f90f5fd; proposals 153 and 155; ADRs 0003, 0014 and 0015; rules core.1, core.2, core.5, core.6, core.14, core.19, core.20; the change's design, D8 to D13 and D18.

## Context and problem statement

ADR 0014 put everything a registry store holds in the one repository the host declares, each name escaped into a tag of it. Clause 138 now names every artifact a publish offers `artifact/<ecosystem>/<name>@<version>` at its version's index, and asks the store to list the versions a name holds. A registry lists the tags of one repository and no prefix across repositories, so a listing over the one repository would read every tag the store holds. C.1 at f90f5fd also asks that a credential can be scoped apart to what a component publishes, to the cache entries a push writes and to the artifacts' names: in one repository, a credential that writes cache entries writes indexes and channels too (135, core.14), and one that makes one component's versions moves another's channels. And a registry that makes repositories only by a creation template, ECR among them, refuses a put into any repository nobody made.

## Considered options

1. One declared repository, as ADR 0014 has it, the listing reading every tag and filtering.
2. Spaces of repositories under the declared one, each a place a credential can be scoped to: a declared component's own repository, a repository per artifact name, a repository per build profile's cache entries.
3. Every space at the registry's root, apart from the declared repository's path.

For a declared component's version's name, which the registry is bound with no component to place:

1. `<component>:<tag>`, the component carried in the name.
2. A reserved space, `version/<component>/<tag>`.
3. The store bound with the declared components, so the registry maps a prefix to its component.

## Decision outcome

Option 2, with the version named `<component>:<tag>`.

The declared repository is the root the spaces hang under, and it keeps what the seam reads by digest: bytes, artifacts, indexes and every claim (8, 135), and the versions' tags and the channels of a repository that declares no component, whose own repository is the declared one (125). Under it, each name C.1 places, each part written as one path component the distribution protocol allows, lowercased, a byte it does not allow written `_` and two hex digits, and a path past 255 characters refused naming it before anything is written:

- **A declared component's own repository**, `<declared>/<component>`. A declared component's version is named `<component>:<tag>` in the store, which no git tag can spell, and its channel `channel/<component>/<name>`; the registry holds them as the tags `<tag>` and `<name>` of its component's repository, moved with the digest expected and read back there as ADR 0015 has it (109, core.6). Each index naming the component is held there from the publish that puts it, and before a version's or a channel's tag is put, the index it names is held there, so an index put before this record is held there by the first tag that names it. A declared component is refused when its name holds a `/` or a `:`, is `artifact` or `cache`, or is `repository`, the undeclared one's, and two whose names are one once lowercased are refused naming both, so every declared component has a repository of its own.
- **An artifact's name**, `<declared>/artifact/<ecosystem>/<name>`, the version its tag, escaped one way and read back, so a name's versions are that repository's tag list, every page of it, a repository the registry does not hold answering none (core.19). The index and every manifest and blob it names are mounted from the declared repository before the tag is put, since a registry takes a manifest only where what it names is held; where the registry declines a mount, the bytes are uploaded under their own digest (core.5). Attestations are not mounted: the seam reads a digest's claims in the declared repository.
- **Cache entries**, `<declared>/cache/<build>`, one per build profile, the build identity the tag; the build profile's name is given by the build family and taken by the store's binding, so no binding names a tool (core.1). A push's store puts its bytes there and asks nothing of the declared repository, so a credential that writes `<declared>/cache/` alone pushes entries and can offer, accept or move nothing (135, core.14); every other binding puts bytes in the declared repository, and an entry's bytes are mounted into the cache repository before its cache manifest and tag. A read by digest asks the declared repository and then the cache repository, so a publish reads the bytes a push left with no map of digests kept (61, core.20).

The store never makes a repository by an API of its own. A registry that makes repositories on push makes each on its first mount or put; one that makes them only by a creation template refuses a put into one no template covers, and the refusal carries the registry's words and names the repository the put was for, nothing asked again (58, 59, core.6). The operator makes the declared repository, and templates applying on push to `<declared>/artifact/`, `<declared>/cache/` and each declared component's `<declared>/<component>`, or one applying to `<declared>/`, or each repository by hand.

Option 1 lists a name's versions by reading every tag the store holds, and scopes no credential apart. Option 3 puts the spaces outside the owner's namespace on a registry that names the owner first, as GitHub's does, and names a second owner. A reserved `version/` space for a component's versions would refuse prefixes the statement allows, and 138 names the reserved spaces; binding the store with the components reads them from somewhere other than a landed revision (65), per host.

### Consequences

- ADR 0014's one repository holds in part: the declared repository still holds every blob, manifest and claim, and a cache entry is still named through a cache manifest, now in its build profile's cache repository.
- A publish writes the declared repository, its component's repository and its artifacts' repositories; a promotion writes the channel's component's repository and, for its claim, the declared one. A credential scoped to one component's repository makes that component's versions and moves its channels and no other's.
- Every index of a declared component is held twice, and every artifact name's repository holds its index and each artifact it names; a mount copies no bytes where the registry allows it.
- A declared component's versions held under their bare tags are absent to the gate, a promotion, the record and verify until the next publish of their revision names them `<component>:<tag>` at the same index; its channel tags already in the declared repository are put by the operator as tags of its component's repository, at the digests they name, before the first promotion after upgrading. The self-hosting guide says both.
- Cache entries a push wrote into the declared repository before this record are read by nothing; an entry gone is a build again (54).

## What ECR answered

The store group, the cache manifest's scenarios, and the mount and the refusal this record reads, ran four times against ECR in us-west-2 on 2026-11-02, by hand with a live session (`moon run ':#check-ecr'`, core.38), at the change's commit c29475f, the declared repository `tailrace-tools` and one repository creation template applying on push to `tailrace-tools/`, its tags mutable, each run's names under a prefix of its own; the mount and the refusal ran once more with their words printed. The account is named in none of the words below.

- **The cross-repository mount**: declined. ECR answered both mounts into an artifact name's repository by opening an upload, `202 Accepted`, and the store uploaded the bytes under their own digest before the manifests and the tag, which the template's repository took. On ECR each artifact name's repository holds its own copy of the index and every artifact it names.
- **A repository no template covers**: refused. A blob put into `tailrace-tools-<run>-uncovered`, a sibling of the declared repository, was answered `{"errors":[{"code":"NAME_UNKNOWN","message":"The repository with name 'tailrace-tools-<run>-uncovered' does not exist in the registry with id '<account>'"}]}`, the store's refusal carried those words naming the repository the write was for, and ECR made no repository.
- **A template with resource tags**: a creation template naming resource tags and no custom role made no repository, and every push under it, a `docker push` among them, was answered `NAME_UNKNOWN` until the tags were removed. A template that tags the repositories it makes names a role that may tag them (`customRoleArn`).
- **A blob read through ECR's redirect**: two runs of four failed the store group's listing scenario on one blob's read, unreadable in the client's words `io: Peer disconnected`, though ECR held the blob and its redirect to its bucket answered the bytes by hand five times in five. Those are the client's words where the far side closes a connection the client kept from an earlier request as the request arrives on it. The store now sends a read whose connection closed before its whole answer arrived once more on a new connection, and never a write (core.6).

The check ran four times more on 2026-11-02 at d69f720, with the read sent once more on a new connection, the same template and settings, through the command the check names (`cargo test -p tailrace --features tools --test registry_ecr`) so no cache of an earlier run stood in for it: four runs of four passed every scenario, the store group's listing among them, with `a name moved from where it is not` the one scenario not applicable, since ECR's names take no condition (ADR 0015).

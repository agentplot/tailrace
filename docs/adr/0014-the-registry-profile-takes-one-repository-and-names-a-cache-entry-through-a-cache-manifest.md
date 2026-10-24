---
number: 14
title: The registry profile takes one repository and names a cache entry through a cache manifest
status: proposed
date: 2026-10-24
decision-makers:
- Chuck Swanberg
---

# The registry profile takes one repository and names a cache entry through a cache manifest

Status: proposed to the user on 2026-10-24, as `briefs.15` has it; accepted only on their yes.

Deciders: madswan-design, ruling on three questions batch 9's review raised from the-registry-store and moons-cache-namespace (phase 1, bolts 2 and 3), for the user to accept on this record's card.
Sources: statement clauses 1, 52, 53, 61, 66, 70, 71, 72, 73, 100, C.1, C.2, and Part C's opening line, "Parts A and B do not change to admit a profile"; rules core.2, core.6, core.20, core.30; the-registry-store's design, D2 and D6; moons-cache-namespace's design, D1.

## Context and problem statement

The store seam names a digest or a name, never a repository (66 to 72), and a cache entry is bytes named `cache/<build identity>` at their digest (moons-cache-namespace, D1), put with put-bytes and named with name, as the layout store and the fakes take it. An OCI registry breaks both in its own way. A blob's address is `/v2/<repository>/blobs/<digest>`, so a profile that spread ecosystem names over repositories, as C.1 allowed on a registry that creates repositories on push, could not find which repository holds a digest it is asked to read or list claims of without a map of its own or a catalogue search on every read. A tag names a manifest and never a blob, so the registry profile's name refuses a cache entry's digest, and bolt 3's replay from ECR waits on it. C.1 also said an attestation's referrer carries the predicate type as its artifact type; a predicate type is a URI and ECR takes only a media type there.

## Considered options

For the repository:

1. One declared repository per registry, the ecosystem name escaped into the tag.
2. A repository per ecosystem name, with a digest-to-repository map tailrace keeps.
3. Both: every object in the declared repository, each artifact also mounted into a repository named for it.

For a cache entry's name:

1. The profile writes a manifest of tailrace's cache-entry type, its one layer the entry's bytes, its config the empty descriptor, and tags that manifest; resolve answers the layer's digest.
2. The core wraps the entry itself, through put-artifact with no ecosystem metadata.
3. The seam gains a way to name a blob.
4. Cache entries stay out of registries.

## Decision outcome

One repository, and the profile writes the cache manifest.

The seam addresses by digest alone because a digest is the one name every store holds the same (1), and Parts A and B do not change to admit a profile, so what a registry cannot address by digest is the profile's to absorb, not the core's or the seam's. A map of digests to repositories is state tailrace would keep that the store can answer (61, core.20); mounting each artifact twice is a second layout nothing reads. One repository is where every blob, manifest, referrer and tag lives, and the ecosystem name is escaped into the tag, which the-registry-store already builds (D8).

A cache entry named through a manifest the profile writes keeps the core's picture whole: the core puts bytes and names their digest, and on every profile resolving the name gives back that digest, so cache.rs, the layout store and the fakes are untouched. The manifest's artifact type is tailrace's cache-entry type, it names no ecosystem and its config is empty, so by C.1's own definition it is no artifact: claims-of filters by the envelope type and never lists it, and no channel, index or policy names it (53, 100, core.30). The bytes stay one store object whether an artifact's layer or an entry's (100). Wrapping in the core would make a cache entry an artifact in the seam's own vocabulary, against 53; a seam operation for naming a blob is a profile's limit written into Part B; keeping entries out of registries forfeits the bolt.

The attestation's wording is corrected rather than chosen: the referrer's artifact type is tailrace's envelope type, the statement's predicate type goes in the `in-toto.io/predicate-type` annotation, and claims-of is one list filtered by the envelope type, which is what the-registry-store builds.

### Consequences

- C.1 of the statement reads as this record decides (`agentplot/blueprints:design/tailrace/requirements.md`, amended 2026-10-24); the per-name mode is gone from it.
- the-registry-store's name row changes: a digest held as a blob is named through its cache manifest, and a tag at a cache manifest resolves to its layer's digest; its refusal of a blob goes. The unit was approved with the refusal, so this changes an approved unit, said to its conductor and the user.
- moons-cache-namespace changes nothing in the core: `cache/<build identity>` at the bytes' digest holds on every profile.
- A registry's retention may remove a cache manifest and its blob together; nothing accepted depends on either (54).
- core.30's detail says the cache manifest is not ruled out.

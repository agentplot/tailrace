## MODIFIED Requirements

### Requirement: S1 over the fakes

The publish group SHALL play S1 over the fakes, an in-memory store and
a git history: a landed revision with two outputs and a landing
verdict naming their build is published.

#### Scenario: S1, a landed revision published

- **WHEN** a revision landed with a verdict naming its build is published, its two outputs read from the build and packed
- **THEN** both artifacts carry the version history derives, the index is put after both artifacts and their claims and names the revision, each artifact has one provenance in the host's identity naming the revision and build identity with the landing's verdict carried unchanged, each artifact's name by its ecosystem, name and version is made at the index before the version's name is, the answer names every digest and where each claim is attached, and nothing else moved: no other name, no keeper append and no target call

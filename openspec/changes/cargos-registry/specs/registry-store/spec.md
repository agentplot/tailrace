## ADDED Requirements

### Requirement: A name's versions are read from its repository's tags

The registry store SHALL answer the versions of a name in an ecosystem
from the tags of the repository it makes that name's pointers in,
`artifact/<ecosystem>/<name>` as C.1 spells it, as the distribution
protocol lists them, every page read: each tag read back to the version
it spells, a tag that spells no version (a referrers tag among them)
left out (138, C.1). A repository the registry does not know SHALL
answer none; any other failure SHALL answer unreadable carrying the
registry's own words (58, 59).

#### Scenario: Versions read back from tags

- **WHEN** a registry store that has made the names `artifact/cargo/fam-a@1.0.0`, `artifact/cargo/fam-a@1.1.0-pre.1+build.5` and `artifact/cargo/fam-b@1.0.0` through its own name is asked for the versions of `fam-a` in `cargo`
- **THEN** it answers `1.0.0` and `1.1.0-pre.1+build.5`, as they were named, and no other

#### Scenario: A name never made

- **WHEN** the same store is asked for the versions of a name it never made a pointer of
- **THEN** it answers none, not unreadable

#### Scenario: More tags than one page

- **WHEN** the registry pages the tag list and a name's versions fall on more than one page
- **THEN** every one of them is answered

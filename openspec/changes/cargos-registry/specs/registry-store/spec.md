## ADDED Requirements

### Requirement: The names under a prefix are read from the repository's tags

The registry store SHALL answer names, given a prefix, from the tags of
its one repository as the distribution protocol lists them, every page
read: each tag read back to the name it holds, a tag that holds no name
(a referrers tag among them) left out, and the names that begin with the
prefix answered (138). A repository the registry does not know SHALL
answer none; any other failure SHALL answer unreadable carrying the
registry's own words (58, 59).

#### Scenario: Names read back from tags

- **WHEN** a registry store holding the names `artifact/cargo/fam-a@1.0.0`, `artifact/cargo/fam-a@1.1.0-pre.1` and `fam-v1.0.0`, and a referrers tag, is asked for the names under `artifact/cargo/fam-a@`
- **THEN** it answers the first two names as they were moved, and no other

#### Scenario: More tags than one page

- **WHEN** the registry pages its tag list and the names under the prefix fall on more than one page
- **THEN** every one of them is answered

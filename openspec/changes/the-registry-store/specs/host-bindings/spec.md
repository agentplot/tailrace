# Spec Delta

## MODIFIED Requirements

### Requirement: The file names the store, the landing and the signer

The file SHALL bind the store, the landing and the signer for the host,
and nothing the repository's authors bind (92): `[store]` naming a
layout by its path (C.2) or a registry by its repository (C.1);
`[landing]` naming sluice or a plain branch,
each with an optional remote (C.5, C.6); and `[signing]` naming the
host's signer and the path of its key file, in sluice's `[signing]`
shape (C.8). A path SHALL be absolute or begin with `~/`.

#### Scenario: A layout, sluice and a host key

- **WHEN** the file names `[store] layout`, `[landing] sluice` with remote `origin`, and `[signing]` with a signer and a key path beginning `~/`
- **THEN** publish writes into that layout, reads sluice's records from `origin`, and signs as that signer with the key under the home directory

#### Scenario: A registry

- **WHEN** the file names `[store] registry` with a repository such as `localhost:5000/tailrace`
- **THEN** publish writes into that repository and verify reads from it, neither makes a repository, and the file names no credential

#### Scenario: A registry named by something that is not a repository

- **WHEN** `[store] registry` names a value that is not a registry host followed by a repository path
- **THEN** publish and verify answer unreadable, naming the file, `[store] registry` and the value, and nothing is read or written

#### Scenario: A plain branch

- **WHEN** the file names `[landing] branch`
- **THEN** a revision is landed when the branch's history holds it, and publish attaches no verdicts (78, C.6)

#### Scenario: A relative path

- **WHEN** the file names a layout or a key by a path that is neither absolute nor begins with `~/`
- **THEN** publish and verify answer unreadable, naming the file and the key, and nothing is read or written

## MODIFIED Requirements

### Requirement: A cache entry is never an artifact
No channel, index or policy SHALL name a cache entry, and reading one
SHALL offer nothing and accept nothing: filling the cache reads no
policy and writes nothing to the store, and a publish's index names no
digest by way of a cache name (53, rule core.30). The layouts the same
sync renders afterwards read the policy for layouts and never a cache
name.

#### Scenario: Sync writes nothing to the store
- **WHEN** a revision is synced in a checkout whose bound projections pin nothing
- **THEN** the store was asked only to resolve names and read digests, and the policy was not asked

#### Scenario: A publish after a cache entry
- **WHEN** a revision whose outputs have cache entries is published
- **THEN** the index names only artifacts the publish packed with a version, and no cache name is read or moved by the publish

#### Scenario: Layouts read no cache name
- **WHEN** a revision is synced in a checkout whose lockfile pins a version
- **THEN** the policy was asked only for layouts, and no name the layouts resolved begins `cache/`

## ADDED Requirements

### Requirement: A push before the landing writes what the checkout's run left

Asked to push what a checkout left, the push SHALL ask the build what
that checkout's run left and write the cache entry of each build
identity whose outcome is a pass, once, in the order the build names
them, packed as an output named for its first task (52, 139). It SHALL
ask nothing of the landing, so it runs before the landing is decided,
and a build identity that did not pass SHALL NOT be written.

#### Scenario: A check job's run pushed

- **WHEN** a checkout's run left two passing build identities and one failed one, and the push is asked for what the checkout left
- **THEN** the store's cache names of the two passing build identities name the bytes packed for them, nothing is packed for the third, and the landing was asked nothing

#### Scenario: A checkout that ran nothing

- **WHEN** the push is asked for what a checkout left and the build answers an empty list
- **THEN** the answer names no build identity, nothing is written, and the push is no refusal

### Requirement: A push before the landing answers as the push after one does

A push of what a checkout left SHALL answer each build identity on its
own, as written, held already at the digest the store's name is at,
absent where the archive is not there, or the refusal or unreadable
answer its write gave; one build identity's answer SHALL NOT stop the
next, and a push repeated SHALL write nothing (54, core.6). It SHALL
put no manifest, attach no claim, sign nothing, ask no policy and keep
nothing (53, 61, 100).

#### Scenario: Two jobs leave the same build identity

- **WHEN** two check jobs' checkouts each left the same passing build identity, and each pushes what it left
- **THEN** the first answers it written and the second answers it held already at the digest the first wrote, with nothing moved or retried

#### Scenario: Nothing offered and nothing kept

- **WHEN** a checkout's run is pushed
- **THEN** the store was asked to put bytes and make each entry's name and nothing else, the signer, the policy and the landing were asked nothing, and the host's record holds after it what it held before it

### Requirement: A check job's push replays on a fresh host after the landing

A run whose checks moon ran in one checkout, pushed from there before
the landing with nothing kept on that host, SHALL let a publish on a
second host that holds none of the outputs offer them from the store
(52, 135).

#### Scenario: Pushed from a job, published from another runner

- **WHEN** moon runs a candidate's check tasks in one clone, as a pool's run job does, the push of what that clone left runs there, the clone is deleted, sluice lands the candidate with verdicts naming the same task hashes (76), and a fresh clone on a second host sharing the store publishes the landed revision
- **THEN** the publish offers the version, every output's bytes are those the first clone's build wrote, and moon on the second host ran no task's command

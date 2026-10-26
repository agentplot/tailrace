# Spec Delta

## MODIFIED Requirements

### Requirement: A promotion moves a channel to an accepted version

A promotion SHALL name one channel, one version of one component and
the branch whose landed head holds the policy. It SHALL find the
channel among the channels the policy lists at that head for that
component, absent naming the channel and the component where none is
listed (125, 131); it SHALL resolve the version's name in the store to
its index before anything else is read of it, and SHALL read the
channel's policy at the branch's landed head (3, 65, core.4, core.22).
It SHALL move the channel only when the version's index carries an
acceptance claim that counts under the channel's policy (31, 9) and, in
the order the policy lists them, every claim the channel's policy
requires counts; otherwise it SHALL be refused naming the first claim
found wanting, and nothing SHALL be signed, appended or written (58,
core.7).

#### Scenario: A version with every claim the channel requires

- **WHEN** a version's index carries the gate's acceptance and a rebuild claim by a second signer, and the policy for `stable` requires both
- **THEN** the promotion moves `stable` to that version, and the answer names the claims that satisfied the policy

#### Scenario: A version not accepted

- **WHEN** a version's index carries no acceptance claim by a signer the channel's policy names for acceptance
- **THEN** the promotion is refused naming acceptance as the claim found wanting, and the ledger and the store are not written

#### Scenario: A claim the channel requires is wanting

- **WHEN** the policy for `stable` requires a rebuild claim and the index carries none that counts
- **THEN** the promotion is refused naming the rebuild claim and the signers that may make it

#### Scenario: A version never published

- **WHEN** the version's name resolves to nothing in the store
- **THEN** the answer is absent naming the version, and nothing is written

#### Scenario: A channel of another component

- **WHEN** the repository declares `lib` and `web`, only `web` has a channel `edge`, and `edge` is promoted for `lib`
- **THEN** the answer is absent naming `edge` and `lib`, and nothing is read of the version or written

### Requirement: Only the gate's signer makes a promotion

The promotion claim SHALL be signed by the host's signer, and only when
the channel's policy names that signer among its promoters: for a
given channel, or one an environment's words make, the gate's signers;
for one declared by hand, the signers it lets claim a promotion.
Otherwise the promotion SHALL be refused naming the promotion as the
claim found wanting and the promoters, before anything is signed (9,
32, core.8, core.14).

#### Scenario: A host the policy does not let promote

- **WHEN** the host's signer is named by the channel's policy for acceptance alone and not among its promoters
- **THEN** the promotion is refused naming the promotion claim and the signers that may make it, and nothing is signed or appended

#### Scenario: The gate promotes a given stable

- **WHEN** the policy writes no line under `stable`, names `gate` as a signer that may claim acceptance, and the host signing as `gate` promotes a version `beta` has held
- **THEN** `stable` moves, its claim made by `gate` and naming `gate` as who asked

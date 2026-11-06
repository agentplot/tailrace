## ADDED Requirements

### Requirement: report is a kind a signer may claim

A `[signers.<name>]` table's `kinds` SHALL take `report`, letting that
signer claim reports of any environment the file declares (116). No
signer SHALL be let claim a report by any rule of the profile's own:
a file whose `[signers]` lists no signer with `report` lets none, and
the one host 108 admits gains no report by being the gate (C.10, 116).

#### Scenario: The deployer may report

- **WHEN** `[signers.deployer]` names `kinds = ["provenance", "report"]`
- **THEN** a policy requiring a report names `deployer` as a signer that may claim one

#### Scenario: The one host and no report

- **WHEN** `[signers.laptop]` names `kinds = ["provenance"]` alone, the one host 108 admits
- **THEN** `laptop` may claim acceptance and promotion and no report

### Requirement: A require names no report

A `require` entry of kind `report` SHALL make the file unreadable naming
its table. An entry has no key to name the environment a report is of,
and a report is required only of the environments an `after` names
(C.10).

#### Scenario: A report a require names

- **WHEN** `[channels.us]` writes `require = [{ kind = "report", signers = ["deployer"] }]`
- **THEN** the policy file is unreadable naming `[channels.us]` and the report

### Requirement: A channel an after makes lists its environments

The profile SHALL list a channel an environment's `after` makes as
bound to the channel the environments it names follow, coming after
those environments, in the order `after` names them (131, C.10). A
channel bound by `bound_to`, or by a manual environment, SHALL come
after none.

#### Scenario: eu after us

- **WHEN** `us` names `stable` with `policy = "manual"`, and `eu` names `stable` with `after = ["us"]`
- **THEN** the channels answer lists `eu` bound to `us` and coming after `us`, and `us` bound to `stable` and coming after none

### Requirement: The policy for an after channel requires a report of each environment

The policy for a channel an `after` makes SHALL require, after the
gate's acceptance and any claim its line adds, one report of each
environment it comes after, in the order `after` names them. Each SHALL
be by the signers `[signers]` lets claim a report (83, 116, C.10). Where
no signer may claim a report, each requirement SHALL name no signer,
and every version SHALL be refused naming the first, so the channel
never moves on a report nobody may make.

#### Scenario: eu's policy

- **WHEN** `[signers.deployer]` may claim acceptance, promotion and report, and `eu` names `after = ["us"]`
- **THEN** the policy for `eu` requires an acceptance by `deployer`, then a report of `us` by `deployer`, and names `deployer` as its promoter

#### Scenario: Nobody may report

- **WHEN** `eu` names `after = ["us"]` and no signer's kinds hold `report`
- **THEN** the policy for `eu` requires a report of `us` by no signer, and a judge of any version under it is refused naming that report

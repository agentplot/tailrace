## MODIFIED Requirements

### Requirement: An environment's newest report

The core SHALL read an environment's newest report from its channel's
chain: the claims attached to every index the chain moved to, the report
claims of that environment that count under the policy, and of those the
one whose apply came last by the moment each names, the later in the
chain's order where two name one moment. The target's convergence time
SHALL order nothing (40, rule core.50). The reports SHALL be told newest
first, and none older than the first that counts SHALL be told. The
gate and apply SHALL read the newest report from the chain as the ledger
holds it and the store as it answers, with no copy, and a chain answered
as last read SHALL make it unreadable naming the chain (rule core.45).
Status SHALL read it across the chain its channels part read, as last
read included, with the reports fetched into the host's copy (60, 61,
98).

#### Scenario: srv rolled back

- **WHEN** `srv` reported 0.4.0 from an apply at 10:00 and 0.3.0 from one at 11:00
- **THEN** its newest report names 0.3.0

#### Scenario: The gate on a chain as last read

- **WHEN** the gate follows `srv` while the ledger answers `srv`'s channel's chain as last read
- **THEN** `srv`'s newest report is unreadable naming the chain, and no copy of the reports is read or written

#### Scenario: Status on a chain as last read

- **WHEN** status is asked while the ledger answers `srv`'s channel's chain as last read and the store answers
- **THEN** status reads `srv`'s newest report across that chain's indexes from the store

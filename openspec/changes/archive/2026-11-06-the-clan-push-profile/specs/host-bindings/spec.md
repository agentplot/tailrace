## ADDED Requirements

### Requirement: Apply binds the host's signer to sign its report

`timber apply` SHALL bind the host's signer to sign, the one `[signing]`
names or the given host key where the file names none, as publish and
promote bind it (118, C.8). It SHALL sign only the report of an apply
that converged and, where the host's own steps tell its gate, the
gate's moves on that report (11, 64). Every other signer profile SHALL
be bound verify-only.

#### Scenario: A host with no bindings file applies

- **WHEN** a host with no bindings file applies an environment that converges
- **THEN** its report is signed by the given host key, made on first use, and named as given in the answer

#### Scenario: Nothing converged

- **WHEN** apply is refused before the target is asked
- **THEN** nothing is signed

### Requirement: The gate table names what tells the gate after an apply

`[gate]` SHALL name what tells the host's gate after an apply's report,
as it names what tells it after a publish (130, C.16). `publish = {}`,
or no `[gate]`, SHALL bind this host's own steps to tell its own gate,
the apply's among them. `job = "<words>"` SHALL bind a job outside
timber, and timber SHALL run nothing for it. No key of the table SHALL
be added for apply.

#### Scenario: No gate table, and an apply

- **WHEN** the bindings file holds no `[gate]`, and an apply signs a report
- **THEN** the binding says this host's own apply tells its gate

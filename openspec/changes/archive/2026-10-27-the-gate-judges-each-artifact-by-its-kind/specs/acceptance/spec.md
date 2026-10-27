## MODIFIED Requirements

### Requirement: Each artifact is judged against each required claim in order

For each artifact the index names, in order, the gate SHALL take, in
order, those of the acceptance policy's required claims that the policy
asks of artifacts made the artifact's way (134), each satisfied only by a
claim on the artifact or the index naming the version's revision, whose
maker the signer seam tells is a signer the required claim names, within
that signer's reach (9, 13, 80, 116, core.7). A verdict SHALL count only
carried in the artifact's provenance, for the check and build required.

#### Scenario: Every required claim found

- **WHEN** acceptance requires provenance from `host` and a verdict of `test` from `landing`, and each artifact carries provenance from `host` holding a passing `test` verdict from `landing` for its build
- **THEN** the version is accepted

#### Scenario: A claim from a signer the policy does not name

- **WHEN** acceptance requires a verdict of `test` from `landing`, and the verdict each artifact carries was signed by a signer the policy does not name
- **THEN** the version is refused naming the required verdict and the first artifact, and the unnamed signer's claim stays attached

#### Scenario: A claim beyond its signer's reach

- **WHEN** a signer the policy lets claim only provenance made a verdict of `test`
- **THEN** that verdict does not count, and the version is refused naming the required verdict

#### Scenario: A claim about another revision

- **WHEN** an artifact carries a provenance from the named signer that names a revision other than the version's
- **THEN** it does not count, and the version is refused naming the required provenance

#### Scenario: A claim asked of built artifacts alone

- **WHEN** acceptance requires provenance from `host` of every artifact and a verdict of `test` from `landing` of built artifacts alone, and the version's index names two outputs, each with `host`'s provenance holding a passing `test` verdict for its build, and its notes, with `host`'s provenance naming no build and carrying no verdict
- **THEN** the version is accepted, and the acceptance names the notes' provenance among the claims that satisfied the policy and no verdict of the notes

## ADDED Requirements

### Requirement: The gate tells a rendered artifact by its publisher's provenance

The gate SHALL judge an artifact as rendered when a provenance attached to
the artifact itself, naming the version's revision, counts through a
signer the acceptance policy at the head names for a required provenance
(core.7), and no provenance that so counts names a build identity (13,
134). Every other artifact SHALL be judged as built. The gate tells each
artifact's making once per judgment, before any claim is counted, and
reads nothing but the claims attached and the policy to tell it (29).

#### Scenario: The notes judged as rendered

- **WHEN** a version's notes carry `host`'s provenance naming no build, and acceptance requires provenance from `host`
- **THEN** the notes are judged against the claims asked of every artifact alone

#### Scenario: A build-less provenance from a signer the policy does not name

- **WHEN** the only provenance an artifact carries that names no build was made by a signer the acceptance policy names for no provenance
- **THEN** the artifact is judged as built, and the version is refused naming the artifact and the first required claim it lacks

#### Scenario: An output's provenance names its build

- **WHEN** an output's artifact carries `host`'s provenance naming its build identity
- **THEN** it is judged as built, and a verdict asked of built artifacts is asked of it

#### Scenario: A policy that asks no provenance

- **WHEN** acceptance requires only a verdict of `test`, and the version's notes carry no verdict
- **THEN** the notes are judged as built, and the version is refused naming the notes and the required verdict

### Requirement: A rendered artifact still needs the claims asked of every artifact

A rendered artifact SHALL be refused, and with it the whole version (28),
where a claim the policy asks of every artifact is wanting for it, the
refusal naming the rendered artifact, the claim found wanting and why, in
the form any artifact's refusal takes (58).

#### Scenario: The outputs' claims count and the notes' provenance does not

- **WHEN** acceptance requires provenance from `host` of every artifact, each output carries it, and the store answers no claim of the notes
- **THEN** the version is refused naming the notes, the required provenance and that it is missing, and nothing is attached

#### Scenario: The outputs' verdict wanting

- **WHEN** acceptance requires a verdict of `test` of built artifacts alone, the notes carry `host`'s provenance naming no build, and the second output carries no `test` verdict
- **THEN** the version is refused naming the second output and the required verdict, never the notes

### Requirement: A following channel judges each artifact by its making

When the gate judges a version under a following channel's own policy, it
SHALL take for each artifact those of the channel's required claims asked
of artifacts made its way, the making told once under the acceptance
policy at the head, so a channel's claim asked of built artifacts alone
never holds a version for its notes (95, 134).

#### Scenario: beta moves with notes that carry no rebuild

- **WHEN** the gate, told by a publish's answer, accepts a version whose notes are rendered, and `beta` follows each publish and requires the gate's acceptance of every artifact and a rebuild of built artifacts alone, which each output carries
- **THEN** `beta` moves to the version's index

#### Scenario: beta held for an output's rebuild

- **WHEN** the same version's first output carries no rebuild
- **THEN** the version is accepted, `beta` does not move, and the answer for `beta` is refused naming the first output and the required rebuild

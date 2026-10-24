## Purpose

The build seam bound to moon (C.3): which moon task makes each declared
output, how one asking runs them, and how the task hash becomes the
build identity a landing's verdicts and tailrace's provenance share.

## ADDED Requirements

### Requirement: The repository names the task that makes each output

The moon build SHALL take the moon task that makes each output from
the repository's policy file at the revision asked about, never from a
checkout, where each output is named with the task's target (65, 92,
C.3). An output the file names no task for SHALL be refused by ensure
as one the build could not make, and answered absent by report.

#### Scenario: An output and its task

- **WHEN** the policy file at a revision names the task `web:build` for the output `site`, and ensure is asked for `site` at that revision
- **THEN** moon runs `web:build`, and report of `site` answers that task's hash

#### Scenario: An output with no task

- **WHEN** ensure is asked for an output the policy file at the revision names no task for
- **THEN** ensure is refused naming that output among those the build could not make

#### Scenario: A checkout's edit is not read

- **WHEN** the publisher's checkout changes the task named for an output, uncommitted, and ensure is asked of the landed revision
- **THEN** moon runs the task the policy file at the revision names

### Requirement: Ensure is one run of the declared tasks at the revision

Ensure SHALL run moon once over the tasks of every output asked, in a
checkout holding exactly the revision asked and nothing of any other
checkout, continuing past a task that fails, and replaying what moon
has cached (74, C.3). It SHALL answer the outputs present, or refused
naming every output whose task declared files the run left absent or
whose task did not run. tailrace SHALL never run an output's command
itself (15, core.10).

#### Scenario: Three outputs in one asking

- **WHEN** ensure is asked for three outputs whose tasks pass, pass and fail, each leaving its declared files
- **THEN** moon is run once over the three tasks, and ensure answers the outputs present

#### Scenario: Outputs the build could not make

- **WHEN** ensure is asked for three outputs of which two tasks leave none of their declared files
- **THEN** ensure is refused naming exactly those two outputs, after the third task has run

#### Scenario: The same revision ensured again

- **WHEN** ensure is asked again for outputs already made at the revision
- **THEN** moon answers each task from its cache, and report answers the same build identity as before

### Requirement: Report reads moon's own record

Report SHALL answer, for an output at a revision, the hash moon gave
its task as the build identity, a pass when moon ran the task to
success or answered it from its cache and a fail when the task failed,
and the paths of the files the task declares as its outputs, read from
what moon recorded of the task in the revision's checkout (75, C.3).
An output whose task moon has no record of at the revision SHALL be
answered absent naming the output.

#### Scenario: A failed task's report

- **WHEN** a task fails after writing its declared files, and its output is reported
- **THEN** report answers the task's hash, a fail, and where the files are

#### Scenario: Nothing ensured

- **WHEN** report is asked of an output at a revision no ensure has run moon for
- **THEN** the answer is absent naming the output

### Requirement: One build identity on every host

The same revision SHALL give the same build identity for an output on
any host and in any checkout, and the build identity SHALL be the key
sluice's verdicts carry for the same task, so a verdict names the build
whose outputs are offered (13, 76, C.3).

#### Scenario: Two checkouts

- **WHEN** the same revision is ensured in two clones of one repository in two directories
- **THEN** both report the same build identity for each output

#### Scenario: The landing's key is the build's identity

- **WHEN** sluice lands a revision whose check is the task that makes an output, and the moon build ensures and reports that output at the landed revision
- **THEN** the build identity report answers is the key sluice's verdict for that check carries

### Requirement: A build that cannot be run is unreadable

Where moon or the repository cannot be reached, every operation SHALL
answer unreadable naming the build, carrying the words the tool or the
operating system gave, verbatim, and never refused or absent (58, 59,
core.19).

#### Scenario: moon is not there

- **WHEN** the program the build runs as moon does not exist, and ensure and report are asked
- **THEN** both answer unreadable carrying the operating system's words unchanged

### Requirement: moon's build group passes

The conformance suite's build group SHALL pass over the moon build in
a scratch repository, every scenario on a repository of its own (93,
core.33).

#### Scenario: The build group over moon

- **WHEN** the suite's build group runs over a rig binding the moon build to a scratch repository
- **THEN** the report names the profile `moon` and every scenario passed

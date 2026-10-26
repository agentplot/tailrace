# tailrace rules

The rules that bind wherever a change is made in this repository,
written so a reviewer can point at a diff and say which rule it breaks.
The narrative is the statement in agentplot/blueprints,
`design/tailrace/requirements.md`; a rule here names the clause it
enforces and never restates it. There are no area pages yet; a rule
that binds only one part of the tree moves to an area page when that
part exists.

## Rules

### The core and its seams

- **[core.1]** Code outside a profile or a plugin directory names no registry, task runner, merge queue, key service, package manager, deploy target or forge.
- **[core.2]** Every seam of Part B is reached through exactly one interface, and a profile implements that interface and nothing of the core.
- **[core.3]** A seam with a single profile has an ADR saying why one is enough.

### Bytes and digests

- **[core.4]** No code path acts on a name: every operation resolves a name to a digest first and records the digest it acted on.
- **[core.5]** No code path writes bytes under any digest but their own, or rewrites bytes a store holds.
- **[core.6]** A name is moved only with the digest it is expected to be at, and a refused move is reported, never retried blind.

### Claims

- **[core.7]** A claim counts for a verdict only through a signer the policy names and a claim that signer may make, never through what the claim says of itself.
- **[core.8]** tailrace signs only as the party running it and only what that party observed; a claim another party made is carried as made, never re-signed.
- **[core.9]** Every claim tailrace writes names the revision it is about, and the build identity where it has one.

### Publishing

- **[core.10]** No code path reachable from publish offers bytes whose build identity the landing's verdicts do not name, accepts, promotes, moves a channel or reaches a target.
- **[core.11]** A publish refuses a revision the landing seam has not landed, before anything is read or offered.
- **[core.12]** Every declared output's outcome is read before any artifact is offered, and the index is offered last.
- **[core.13]** The version's name is derived from history by the authors' rule and claimed in the shared history before any artifact is packed, and no build is handed a version.
- **[core.37]** A version is derived for one declared component over the commits that touch its paths and the tags that carry its prefix, and no code path derives a version, claims a name or moves a channel without naming the component.

### Acceptance and channels

- **[core.14]** Only the gate's path accepts or promotes, and no code path holds both an offering credential and an accepting one.
- **[core.27]** An attestation is written only beside the digest it is about, and no code path keeps one apart from its digest.
- **[core.28]** A version with one refused artifact is refused whole, naming the artifact, and no code path accepts part of a version.
- **[core.29]** A reconciler verifies the claims it is told to require before any apply, and no apply path skips the verify.
- **[core.15]** A promotion is written to the ledger before the channel is read as moved, and nothing deletes or rewrites a ledger entry.
- **[core.16]** A channel moves only from the version it is at, and an environment is never handed a version except through its channel.
- **[core.34]** A promotion is appended to its channel's chain only after the claim it names as predecessor is the chain's head, and two claims naming one predecessor are reported as a fork, never chosen between.
- **[core.35]** No code path converges an environment that waits except by a promotion of the channel it follows; there is no approval that is not a promotion.

### Consumers

- **[core.17]** A consumer layout holds only versions whose verdict under its policy is satisfied, and a refused version is reported naming the claim found wanting.
- **[core.18]** An address a client records names no host and no one owner's store.
- **[core.36]** One serving process answers a host, at one fixed name and port with the repository in the path, and no checkout starts one of its own.

### Answers and records

- **[core.19]** Absent, refused and unreadable are three answers, and a refusal carries the refusing source's own words verbatim.
- **[core.20]** Every record a reader may need is readable with no tailrace process running, and everything kept but the record of claimed versions is rebuilt from the store and git.
- **[core.21]** No credential is kept: a credential is the asker's, the host's or the environment's, reached at the moment of use.
- **[core.22]** The policy is read from a landed revision of the branch it governs, never from a checkout.
- **[core.30]** No code path turns a cache entry into an artifact, names one from a channel, an index or a policy, or refuses to build because one is gone.
- **[core.31]** Every surface calls the one core and renders its answer; no surface computes a verdict, a version or a channel of its own.
- **[core.32]** A read of what tailrace holds opens no connection to a store, a signer or a target and writes nothing.
- **[core.33]** Every profile passes the one conformance suite, and every scenario of the suite cites the clauses it exercises; a clause no scenario cites is named in the suite's register with the unit of the plan that will cite it, or with the phase of the plan while that phase has no units carved, and why, and leaves the register with the commit that cites it.

### Design and records

- **[core.23]** A change that bears on a clause cites the clause by number; a clause that no longer holds is changed in the statement first, never worked around here.
- **[core.24]** Nothing commercial, no pricing, tier, hosted plan or customer, appears in this repository.

### The gate

- **[core.25]** Every file the tree tracks is owned by a formatter the gate runs, or is excluded by name with its reason; every language with a linter is linted; `site/`, and every crate and plugin directory, is a moon project of its own whose check-fast tasks validate, type-check and test it, one task per check.
- **[core.26]** Every tool a gate task runs is the version the tree pins, on every host that runs the gate.
- **[core.38]** No gate check needs a credential a person signs in for or a service only such a credential reaches; a check that does runs outside the gate, by hand with a live session or by a job that holds an identity of its own, under a tag of its own, refusing to run without its setting and naming it.

## Details

- **core.1** Rules out: the strings `oci`, `ecr`, `ghcr`, `oras`, `cosign`, `moon`, `sluice`, `kms`, `npm`, `maven`, `cargo`, `flux`, `gh` in any source module outside a profile directory or a plugin directory; an import of a profile from the core. Not ruled out: `git`, which section 9 of the statement fixes, except in the ledger's code, which reaches where the chain is held through the ledger seam (B.8) and names no git; the in-toto and DSSE forms, the givens' default for every attestation; `.moon/` and the other configuration of this repository's own checks, which are no module; a plugin, which is a surface outside the core and names the ecosystem command it refuses. Source: statement preamble, clause 92; ADRs 0002, 0003, 0005.
- **core.2** Rules out: a profile calling another profile; a core function with a profile-specific branch; a projection that reaches the store other than through the store seam. Source: clauses 92, 93; ADR 0002.
- **core.3** Rules out: a seam whose only profile is undocumented as such. Source: ADR 0008.
- **core.4** Rules out: reading a tag, a version or a channel and acting on what it names without resolving it; a record of an act that names a tag and no digest; a channel head read to decide what to deploy without the digest beside it. Source: clauses 3, 40; ADR 0003.
- **core.5** Rules out: a put under a digest computed from anything but the bytes; a copy that recompresses or rewrites; an "update" of an artifact's bytes. Not ruled out: packing the same outputs with a new version, which makes a new artifact with its own digest (24). Source: clauses 1, 2, 73; ADR 0003.
- **core.6** Rules out: a tag put without the expected digest where the store supports one; a loop that re-reads and retries a refused move; a channel moved without reading what it is at. Source: clauses 35, 70; ADR 0007.
- **core.7** Rules out: a verdict that counts a claim because its predicate type is right; a signer's claim counted beyond what the policy lets it claim; a verdict that trusts a carrier's word that it verified what it carried. Not ruled out: keeping every claim attached, counted or not (9). Source: clauses 9, 80; ADR 0007.
- **core.8** Rules out: signing a provenance for a build this host did not run; re-signing sluice's run record under tailrace's key; a claim in a reviewer's or a gate's name made on their behalf. Source: clauses 11, 12; ADR 0005.
- **core.9** Rules out: a provenance naming outputs and no build identity; a verdict carried with no revision; a test result with no build identity, which no verdict can join. Not ruled out: a promotion claim, which has no build (39). Source: clause 13; ADR 0005.
- **core.10** Rules out: a publish that runs the build's commands itself rather than asking the build seam; an output offered whose build identity no verdict of the landing names, where the landing has verdicts; a publish that moves `beta` because the policy says `beta` follows each publish, which is the gate's (95); a publish that syncs a bucket or updates a stack. Not ruled out: `ensure` through the build seam, which may run the repository's own tasks (74), since what it makes is offered only when its identity is one the verdicts name. Not ruled out: the host's publish step telling the host's own gate of the publish's answer where the host's bindings file names it, whose judgment and moves are the gate's (95, 108, 130). Source: clauses 15, 20, 95, 108, 130; ADR 0002, 0004, 0019.
- **core.11** Rules out: publishing a session's head, a candidate under test or a worktree's tree; checking the landing after artifacts are offered. Source: clause 14; ADR 0004.
- **core.12** Rules out: offering an artifact while another output's build is still running or has failed; offering the index before every artifact it names is held. Source: clauses 16, 17; ADR 0004.
- **core.13** Rules out: a version a publisher passes on the command line that the rule did not derive; packing before the name is claimed; a build task given the version in its environment. Not ruled out: a prerelease derived from a branch's name under the authors' rule. Source: clauses 21, 22, 24; ADR 0004.
- **core.37** Rules out: a version computed from the whole tree in a repository that declares components; a tag pushed without the component's prefix; a channel that names versions of two components; a publish of one component that re-versions another. Not ruled out: two components naming one path. Source: clauses 22, 110, 123 to 125; ADR 0010.
- **core.38** Rules out: a required gate task that reads an SSO profile, a key file or a token from the host; a gate task that passes or fails with the state of a person's sign-in; a tools-layer check against a hosted service in `#check-tools`. Not ruled out: the same check under its own tag, run by hand or by a job with an identity; the gate proving a profile against a service it runs itself, such as a registry in the tools layer. Source: clause 64; sluice clause 114; the gate's places formed from a cleared environment (sluice C.2).
- **core.14** Rules out: a publish path that also accepts; a host configuration holding the accepting credential beside the offering one; acceptance by anything that is not the gate. Not ruled out: the one host a policy names as both offerer and gate for a repository with one publisher (108), whose acceptance claim says it accepted what it offered. Source: clauses 25, 27, 96, 108; ADRs 0007, 0009.
- **core.15** Rules out: moving a tag and writing the ledger after; a ledger entry edited, compacted or pruned; a rollback that removes the entry it undoes. Source: clauses 33, 34; ADR 0007.
- **core.16** Rules out: a promotion without the expected starting version; a reconciler given a version to apply rather than a channel to follow. Source: clauses 35, 40; ADR 0007.
- **core.34** Rules out: an append that does not read the chain's head first; a ledger that resolves a fork by timestamp or by order of arrival; a promotion claim with no predecessor digest but the first of its channel. Source: clauses 105, 106, 120 to 122; ADR 0009.
- **core.35** Rules out: an approval claim a reconciler checks for apart from its channel's head; an environment configured to wait that follows a channel it does not own. Source: clauses 41, 97; ADR 0009.
- **core.17** Rules out: a layout rendered with no policy; a version left in a layout after its policy refuses it; a refusal swallowed into an absent version. Source: clauses 47, 51; ADR 0006.
- **core.18** Rules out: a served registry whose name differs per host; a layout path that embeds a registry host or an account; a lockfile written with a class's or an owner's host in it. Source: clause 49; ADR 0006.
- **core.36** Rules out: a port chosen at start and written into a client's configuration; a server started by a worktree's agent; two servers on one host for two checkouts of one repository. Source: clauses 114, 115; ADR 0006, interview 1.
- **core.19** Rules out: an empty list for a failed read; a 404 treated like a refusal; a refusal rewritten in tailrace's words alone; `unknown` as a verdict. Source: clauses 58, 59.
- **core.20** Rules out: state held only in a running process; a record only a server reads; a layout treated as a source of truth; a record that cannot be rebuilt and is not the version claim. Source: clauses 61 to 63.
- **core.21** Rules out: a token written into a client's configuration by tailrace; a credential cached in tailrace's own state; a reconciler that holds a target's credential outside the environment. Not ruled out: a client's own credential helper the ecosystem provides. Source: clauses 45, 64.
- **core.22** Rules out: reading `.config/tailrace.toml` from the publisher's worktree; a submission that relaxes the policy it is judged under. Source: clause 65.
- **core.23** Rules out: a comment or commit that reinterprets a clause; a behavior the statement forbids kept behind a flag. Source: ADR 0001.
- **core.24** Rules out: a pricing table, a tier name, a customer name, a hosted-plan reference in any file. Source: ADR 0001; statement section 9.
- **core.25** Rules out: a file type the formatter neither formats nor excludes; an exclude with no comment saying why; a part with no project of its own in `.moon/workspace.yml`; one task bundling several checks; a task in a change whose only verification is a command run by hand. Not ruled out: the site's build as its one check until it has tests. Source: ADR 0008.
- **core.26** Rules out: a tool a task finds outside its pin; two files pinning one tool with no check that fails when they differ. Source: ADR 0008.
- **core.27** Rules out: an attestation kept in tailrace's own state and not in the store; a copy of a version that leaves its referrers behind. Source: clauses 8, 103.
- **core.28** Rules out: accepting an index while one of its artifacts is refused; a partial acceptance. Source: clause 28.
- **core.29** Rules out: an apply that trusts the channel's word; a reconciler with no policy to verify under. Source: clause 42.
- **core.30** Rules out: a channel tag on a cache entry; an index naming a cache entry's digest without a publish; a publish that fails because the cache is empty. Not ruled out: a store profile's manifest of the cache-entry type, which only lets a tag name an entry's bytes and names no ecosystem. Source: clauses 53, 54, 100; ADR 0014.
- **core.31** Rules out: a plugin that reads the store itself; a pane that derives a channel's head from a tag listing. Source: clause 56.
- **core.32** Rules out: `status` that lists referrers from the registry; a read that refreshes a layout. Source: clause 60.
- **core.33** Rules out: a profile merged with no suite run; a scenario citing no clause; a scenario citing a clause it does not exercise; a clause neither cited by a scenario nor named in the register; a clause both cited and still in the register; a register entry naming neither a unit nor a phase, or no reason. Not ruled out: a clause named in the register with the unit that will cite it, or with a phase whose units are not yet carved, reported as pending on every run; the planner names the unit when it carves the phase. Source: clause 93, section 12; ADR 0012.

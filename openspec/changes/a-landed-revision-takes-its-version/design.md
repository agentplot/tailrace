# Design

## Context

`the-seams-and-their-fakes` lands the eight traits, the types they pass
(a `Component` already carries a name, outputs, paths and a prefix) and
`Answer<T>`, whose `Not` keeps absent, refused and unreadable apart,
each naming its `Source`. Its design leaves to the core's units the
refusals the core itself makes (D1 there). `tailrace-core` is empty and
names no tool but git, which section 9 fixes and core.1 lets it name.

Part B gives the landing seam `landed` and `verdicts-of` (77, 78) and
the policy seam `policy-for`, the claims an audience requires (82, 83).
No seam answers the components or the version rule, though 21 and 123
need both read at a landed revision and C.10 puts both in the policy
file. The version rule is git-cliff over conventional commits, run per
component (section 9, ADR 0010), and the name is claimed as a git tag
pushed before anything is packed (ADR 0004).

git-cliff-core 2.14.2 was read for this design. Its `repo` feature
gives `Repository::init(path)` (open, no parent search, bare or not) and
`Repository::commits(range, include_path, exclude_path, topo)`, which
walks a revision or an `a..b` range and keeps the commits whose changed
files match `glob::Pattern`s; it caches changed files under the git
directory. Its `tags(pattern, …)` reads local tags only and resolves
reachability from `HEAD`. The bump is public only through
`Changelog::new(releases, config, range)` then `bump_version()`, which
honours `config.bump` (`features_always_bump_minor`,
`breaking_always_bump_major`, `initial_tag`, …); `Release::
calculate_next_version()` is public but ignores the configured bump.
Under next_version 0.3.2, any commit that is not a feature or breaking
bumps the patch, so git-cliff alone never gives a prerelease.

## Goals / Non-Goals

**Goals:**

- One type that only a landing seam's yes makes, taken by every read in
  this change, so no code path reads the repository at a revision the
  branch has not landed (14, 65).
- The version a pure function of the landed revision, the component and
  the rule, with the claim the only write (21, 22).
- Every failure one of the three answers, with git's words carried
  (58, 59).

**Non-Goals:**

- The policy seam's profile over the file (signers, audiences,
  channels, environments): this change reads only the components and
  the rule.
- Retrying a refused claim or deriving the next version after one: a
  refused claim is reported and the publish ends (core.6).
- Which remote a host names as the shared history beyond the branch's
  own (118, bolt 2).

## Decisions

### D1. The core reads the components and the rule from git, not through the policy seam

`declared.rs` reads `.config/tailrace.toml` as a blob from the landed
revision's tree in the repository's object database and parses the
`[version]` table, the `[[component]]` tables and a top-level
`outputs`, leaving every other table and key to the reader that owns
it. The policy seam keeps `policy_for` for an audience's claims, asked
through `policy_at` (D2).

Alternatives: a policy seam operation answering components and the
rule, which B.5 does not name and which would change a trait the seams
unit owns (a statement change first, core.23); the C.10 profile under
`crates/tailrace/src/profiles/`, which the core cannot reach except
through a trait (core.2). The gap is named for the design agent; if B.5
gains the operation, `declared.rs` moves behind it unchanged in what it
answers.

### D2. `Landed` is made only by the landing seam's yes

`landed.rs`: `Landed { branch, revision }`, fields private, made only
by `Landed::check(&dyn Landing, &Branch, &Revision) -> Answer<Landed>`.
`landed` answering `false` is refused `Why::NotLanded { branch,
revision }`, sourced to the landing seam and its profile; unreadable
passes through as the landing gave it. `Declared::read`, `policy_at`,
`derive`, `touched` and `claim` each take `&Landed`, so the type system
holds 65. `policy_at(&dyn Policy, &Landed, &Audience) -> Answer<Required>`
is the policy seam's answer as given.

### D3. The repository's history is one struct in the core

`history.rs`: `History::open(path)` opens the repository with `git2`
(the version git-cliff-core takes, 0.21, no default features) and
git-cliff-core's `Repository::init` on the same path; failure is
unreadable naming the path with git's words. It answers:

- the blob at a path in a landed revision's tree, or absent;
- the commits in `last..revision` (or all of the revision's history)
  touching a component's paths, through git-cliff-core's `commits` with
  the paths as `include_path`, so the path filter is git-cliff's own
  (ADR 0010);
- the tags carrying a prefix, each with the revision it names (peeled),
  read from the local repository and from the shared history (D6), kept
  when the landed revision reaches it (equal, or `graph_descendant_of`);
- the shared history's remote for the branch (D6).

The tag pattern is git-cliff's form, `^<prefix>` then a semantic
version, matched with `regex`; the core applies it to local and shared
tags itself, because git-cliff-core's `tags` reads only local tags and
reaches from `HEAD`, not from the landed revision. A tag whose text
after the prefix is not a semantic version is not the component's.

### D4. The policy file's shape

```toml
outputs = ["site"]              # the one component's, when none is declared

[version]
prefix = "v"                    # the one component's prefix; default "v"
initial = "0.1.0"               # default "0.1.0"
prerelease = "pre"              # the prerelease identifier; default "pre"
features_always_bump_minor = true
breaking_always_bump_major = true

[[component]]
name = "lib"
outputs = ["lib-pack"]
paths = ["crates/lib/**"]
prefix = "lib-v"
```

A declared component requires `name`, `outputs`, `paths` (non-empty)
and `prefix`; a missing or ill-typed one is unreadable with the parser's
words. Two components sharing a name or a prefix are refused
`Why::DeclaredTwice { first, second, what }` (123). With no file, or no
`[[component]]`, the repository is one component named `repository`,
paths `["**"]`, the `[version]` prefix and the top-level `outputs`
(none when absent). Unknown keys are ignored so later readers add theirs
(`projections`, C.10) without breaking this one.

### D5. The version rule

`version.rs`: `derive(&History, &Landed, &Rule, &Component) ->
Answer<Derived>`. In order:

1. A tag carrying the prefix that names the landed revision itself
   gives its version, `claimed: true` (18, 21); a release tag over a
   prerelease one, then the highest.
2. `last` is the highest release tag (no prerelease part) by semantic
   version precedence among the reachable tags (23).
3. `since` is the commits from `last` (exclusive) to the revision that
   touch the component's paths. Empty: `Derived::Unchanged { last }`.
4. Any commit in `since` that git-cliff-core reads as a conventional
   `feat` or `fix`, or breaking: a release, numbered by git-cliff's
   bump of `last`'s bare version (prefix stripped by the core) with the
   rule's bump settings, through `Changelog::new` with
   `git.conventional_commits = true` and `remote.offline = true`, then
   `bump_version()`; with no `last`, the rule's `initial`.
5. Otherwise a prerelease: `last`'s next patch (or `initial` with no
   `last`), with prerelease `<identifier>.<n>`, `n` the length of
   `since`, so `1.2.4-pre.3` orders after `1.2.3` and before `1.2.4`.

`Derived` names the component, the `Version` (bare), the tag name
(prefix and version), release or prerelease, `last`, and `claimed`.
`touched(&History, &Landed, &Declared) -> Answer<Vec<Derived>>` derives
every component and keeps those not `Unchanged`, so a path two
components name versions both (124). Nothing here takes a build seam.

Alternatives: git-cliff's bump alone, which makes every commit a release
and gives no prerelease; counting prerelease tags since `last`, which
depends on what other publishers claimed and breaks 21.

### D6. The claim is a lightweight tag pushed without force

`claim(&History, &Landed, &Derived) -> Answer<Claimed>`:

1. Locally: a tag of that name at another revision is refused
   `Why::NameHeld { name, revision, other }`, nothing pushed. Absent,
   it is made as a lightweight tag at the revision with `git2`, without
   force. A lightweight tag points at the commit, so two publishers of
   one revision claim the same thing.
2. The shared history is the remote the branch's `branch.<name>.remote`
   names; with none, the local history is the whole of it and the claim
   ends (22).
3. `git push --porcelain <remote> refs/tags/<name>:refs/tags/<name>`,
   run as a git command so the host's own credential helper answers
   (core.21). New or up to date: claimed. Rejected: `git ls-remote
   <remote> refs/tags/<name> refs/tags/<name>^{}` names the other
   revision, the local tag made in step 1 is deleted, and the answer is
   refused `NameHeld` with git's words. Any other failure: unreadable
   naming the remote with git's words; a local tag made in step 1 stays,
   and a later claim at the same revision pushes it again.

The shared tags derivation reads (D3) come from `git ls-remote --tags
<remote> 'refs/tags/<prefix>*'`, which writes nothing; an unreachable
remote makes the derivation unreadable rather than derived from a
partial history.

### D7. The history answers as a source of its own

The seams' `Source` names one of the eight seams. The history is none
of them, so `answer.rs` gains `Source::history(<remote or "local">)`,
naming git, beside the seams; `NameHeld`, a fetch or push that fails,
and a policy file that cannot be parsed name it. `answer.rs` gains the
three refusals `NotLanded`, `DeclaredTwice` and `NameHeld`, each with
what its clause says it names.

### D8. Tests over real git and the fakes

`crates/tailrace-core/tests/` builds repositories with the git command
in temporary directories (`tempfile`), a bare repository standing in
for the shared history and two clones for two publishers; the fakes
(`tailrace-fakes` as a dev-dependency) answer the landing, the policy
and the build. Each spec scenario is one test. The core's moon project
gains `fakes` in `dependsOn` and `/crates/tailrace-fakes/**/*` in its
test inputs.

## Risks / Trade-offs

- [git2 builds libgit2 from C] → no default features, so no TLS or SSH
  library; the devenv shell and the flake's stdenv both carry a C
  compiler. The flake build is not in check-fast, so it is checked by
  hand once.
- [git-cliff-core is a large dependency, templates and all] → it is the
  rule section 9 and ADR 0010 name; only the `repo` feature is taken.
- [git-cliff-core caches changed files under the git directory] → the
  only write a derivation makes besides the claim, inside git's own
  directory; status and verify (core.32) do not derive.
- [The claim runs the git command] → the push needs the host's
  credential, which only git's own helpers reach without tailrace
  keeping one (core.21); the binary needs git on the host's path,
  which every repository tailrace serves has.
- [Two components read from one file by two readers once the policy
  profile exists] → D1's gap, named for the design agent.

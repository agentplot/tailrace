---
number: 25
title: The tool is named timber and its family Boomdrive
status: proposed
date: 2026-11-04
decision-makers:
- Chuck Swanberg
---

# The tool is named timber and its family Boomdrive

Status: proposed. The names are the user's ruling of 2026-11-04, quoted below. The record is accepted when the user says yes to it as a record.

The user's words (2026-11-04): "the ecosystem, the org, the branding, the cloud and the opensource runner tool are all boomdrive. There is also a "boom" open source tool that does the claim reconciliation."

Deciders: the user, 2026-11-04, renaming Flywheel to Boomdrive across every repository at once, through the chore tailrace-is-renamed-timber, which carries this repository's part. The mapping below is the brief's. The choices under "Considered options" are the chore's own.
Sources: the user's words above; the chore's brief; statement clauses 65, 105, C.10, C.13 and C.16, and section 9 (`agentplot/blueprints:design/timber/requirements.md`); ADRs 0005, 0012, 0016 and 0022; rules core.5, core.15, core.23 and core.47.

## Context and problem statement

The family of tools was named Flywheel. The user has renamed the product, the brand, the family, the control plane and the runner Boomdrive, and this tool, tailrace, timber. The mapping, as the brief gives it:

- **Boomdrive**, one word, is the product, the brand, the ecosystem and the family of open tools on their sites, sluice, flume, timber, weir and the rest. In prose it takes a capital B. In code, commands, packages, paths and repository names it is `boomdrive`, with `Boomdrive` where the code's case needs a capital. It is never spelled with a space, a hyphen, an underscore or a second capital.
- **Boomdrive Cloud** is the control plane, agentplot/boomdrive-cloud, formerly flywheel-cloud.
- **Boomdrive**, the runner, is agentplot/boomdrive, formerly flywheel-runner. Its command is `boomdrive`, and its crates are `boomdrive` and `boomdrive-hosted`.
- agentplot/boomdrive-web was flywheel-web.
- **timber** is this tool, in agentplot/timber: its crates, its command, its site at https://agentplot.github.io/timber and its records.
- **A drive**, plural drives, `Drive` in code, is a customer's unit of work, which was "a Flywheel".
- agentplot/flywheel, the original, is archived. A mention of it as history stays and names it as archived.
- boom is unchanged, and so is anything of a third party's that happens to say flywheel.
- The same pass spells enroll, enrolls, enrolled, enrolling and enrollment, never enrol or enrolment.

The decision changes names only. No clause's meaning and no record's decision changes with it. Some of this repository's names are written into what timber keeps: in a host's configuration, in a repository's files, in a git ref, and in a store's bytes. Some others name places in other repositories. Four questions are left to the chore:

1. Do the names timber writes and reads move to timber as well?
2. Where does a citation of the statement point?
3. What do the site's calls to the shared site theme name?
4. What happens to the words that explain the old name?

## Considered options

For the names timber writes and reads, which are the policy file, the ledger's, landing's and fetch's refs, its media types, annotations and claim kinds, the host's configuration, state and data directories, a checkout's layouts, and the environment variables:

1. Move them all to timber: `.config/timber.toml`, `refs/timber/…`, `application/vnd.agentplot.timber.*`, `io.github.agentplot.timber.*`, `https://agentplot.github.io/timber/claim/…`, `$XDG_CONFIG_HOME/timber`, `$XDG_STATE_HOME/timber`, `$XDG_DATA_HOME/timber`, `.timber/layouts/`, `.git/timber/` and `TIMBER_*`.
2. Keep the old names wherever bytes are kept, and rename only code and prose.

For a citation of the statement:

1. Cite `design/timber/` in agentplot/blueprints and the design book at https://agentplot.github.io/blueprints/timber/, which is where blueprints' own pass of the rename puts them. A path cited at a commit keeps the path that commit holds.
2. Cite `design/tailrace/` until blueprints moves, then change every citation again.

For the site theme, which the site pins at a revision where this tool's member id is `tailrace`:

1. Give the theme the id its pinned revision knows, and move it to `timber` together with the pin.
2. Write `timber` now, before the pin can move.

For the words that explain the old name, a watermill's tailrace carrying water away from the wheel:

1. Drop them.
2. Write a meaning for timber.

## Decision outcome

The names timber writes and reads move to timber, option 1. Nothing is held under the old names anywhere this chore could look on 2026-11-04:

- agentplot/timber's remote holds no ref under `refs/tailrace/`.
- The host that runs the gate has no tailrace configuration, state or data directory.
- No repository of agentplot's or afterthought's on GitHub, and no checkout on that host, holds a `.config/tailrace.toml`.

So no ledger entry is orphaned, no claim goes unread and no store is rewritten (core.5, core.15, core.47). Option 2 would put the old name into every store and repository timber touches, for as long as any of them is kept.

A citation of the statement is option 1. The statement is renamed with everything else, and a citation that names it by its new path needs no second pass. The one citation of a path at a commit, `design/tailrace/models/phase-1-bolts.md` at 251de29 in ADR 0012, keeps the path that commit holds.

The site theme is option 1. At its pinned revision the theme throws on an id it does not know, so option 2 fails the site's build. The site's title is `timber`, so until the pin moves the theme's banner, accent and family bar find no member for the site and draw their defaults.

The words that explain the old name are option 1. The user gave the name and no meaning for it, so the README and the page on how timber fits say what it does instead.

### Consequences

- The rules whose words name the tool or what it writes read with the new names and cite this record: core.8, core.9, core.19, core.20, core.21, core.22, core.27, core.32, core.43, core.52, core.54, core.57, core.58, core.61, core.62, core.66 and core.69.
- Until blueprints takes the rename, the statement names the policy file `.config/tailrace.toml` (C.10), the ledger's refs `refs/tailrace/ledger/` (C.13), and the tool in its title, its Part A and its sections. Until then this repository is ahead of the statement on those names, and on nothing else. This departs from core.23 for those names alone: the user ruled one rename for every repository at once, and a name is no clause's meaning.
- C.16's "a hosted tailrace, Flywheel" and section 9's "a Flywheel plugin" read as timber and Boomdrive here, as the site's surfaces page and roadmap say.
- A host whose `devenv.local.nix` sets `TAILRACE_TOOLS_ECR`, `TAILRACE_TOOLS_KMS_KEY` or `TAILRACE_TOOLS_KMS_STRANGER` sets the `TIMBER_` names for the hand-run checks. A host that installed the Claude Code plugin `tailrace` from the `tailrace` marketplace installs `timber` from `timber`. A host with the herdr plugin `agentplot.tailrace` installs `agentplot.timber` in its place. A host that fetches the release's `tailrace-<system>.tar.gz` fetches `timber-<system>.tar.gz`, and runs `timber`.
- The site's `Family` call names `tailrace` until site-theme's own rename lands and the site's pin moves.
- The suite reads the statement at `agentplot/blueprints` 7f60e57, as before. Its section A.11 is listed as "What timber keeps".
- ADR 0012 cites flywheel-next at fcaac6be. That repository is not renamed, so the citation stays as written.

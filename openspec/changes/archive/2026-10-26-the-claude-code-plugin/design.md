# Design

## Context

The statement names five surfaces answering from one core (section 9,
56), three of them plugins kept outside the core as sluice keeps its
plugins; this change builds the first plugin's hook. Rule core.1 already
names a plugin directory as the one place outside a profile that may
name an ecosystem's command, and says why: a plugin "names the ecosystem
command it refuses". Rule core.31 rules out a surface that computes a
verdict, a version or a channel, or reads the store itself.

sluice keeps its Claude Code plugins under `plugins/` with the
marketplace file at its root (sluice ADR 0023), each plugin a moon
project whose checks are a shell test of its hooks and the pinned Claude
Code's `plugin validate --strict` (sluice ADR 0030). Its Bash guard,
`plugins/sluice/scripts/guard.sh`, refuses a push onto a protected branch
with a PreToolUse `permissionDecision: "deny"` and a reason naming
`sluice submit`, and is tested on scratch repositories by
`plugins/sluice/tests/hooks.sh`. This change takes that shape whole.

The site already has the plugin's guide
(`site/src/content/docs/guides/claude-code.mdx`), which fixes the
marketplace's and the plugin's names through its install lines and lists
`npm publish`, `mvn deploy`, `uv publish`, `cargo publish` and `docker
push` as refused, and mockups of the pane and band this change does not
build.

## Goals / Non-Goals

**Goals:**

- The hook refuses an ecosystem's own publish command in a tailrace
  repository and names `tailrace publish`, with nothing on the host but
  git, jq and bash.
- The plugin installs from the repository's marketplace with the guide's
  two lines, and the gate checks both manifests and the hook.

**Non-Goals:**

- Any answer of tailrace's own in the session (status, the pane, a
  session note): proposal.md lists what waits on what.
- Closing every way bytes could leave: a script that publishes, a
  Makefile target or a CI job is beyond what a Bash hook sees. The hook
  keeps an agent from the obvious command and says what to run instead;
  the store's credentials and the gate are what stop the rest.

## Decisions

### A tailrace repository is one whose HEAD holds the policy file

The hook asks `git -C <dir> cat-file -e HEAD:.config/tailrace.toml`. It
never reads the file's contents, so it never reads the policy, and rule
core.22 (the policy only at a landed revision) is not reached: what it
reads is whether the repository is tailrace's at all, as sluice's guard
reads `HEAD:.config/sluice.toml`. HEAD rather than the working tree, so
an uncommitted file never changes what the hook does, and HEAD rather
than the landed branch, so a branch that adopts tailrace is guarded from
its first commit. A repository that declares nothing (123) still writes
the file, since the guide's setup ends by writing it; a repository with
no file is not tailrace's and the hook stays out of its way.

Alternative: ask the core, `tailrace status`. Rejected: status answers
what the host published, not whether the repository is tailrace's, and
the hook would then need the binary installed to refuse anything, which
a session that has not installed tailrace yet is exactly the one to
guard.

### The refused commands are the plugin's, one table in the guard

The table, by ecosystem of C.11 that has a publish command of its own:

| ecosystem | program (basename) | refused when a later word is |
|---|---|---|
| npm | `npm`, `pnpm`, `yarn`, `bun` | `publish` |
| Cargo | `cargo` | `publish` |
| Maven | `mvn`, `mvnw`, `gradle`, `gradlew` | `deploy` or `deploy:*` for Maven; `publish` for Gradle |
| Python | `uv`, `poetry`, `hatch`, `flit`; `twine` | `publish`; `upload` |
| NuGet | `dotnet`, `nuget` | `push` |
| an image | `docker`, `podman` | `push` |

Go has no publish command (a module is read from its tag), and a file,
moon's cache and a template set have none of an ecosystem's own; they
have no row. The table lives in the guard script as a `case`, because
rule core.1 puts the naming of these commands in the plugin and nowhere
else, and no core answer knows them.

A command is split on whitespace and on `&&`, `||`, `;` and `|`, and
read as words, never evaluated, as sluice's guard does: leading `VAR=value`
words are skipped, `cd <dir>` moves the directory the next segment is
judged in, the program is the next word's basename, and the segment is
refused when any later word is the program's verb. Any later word, not
the first subcommand, because Maven and Gradle take several goals in one
call (`mvn clean deploy`) and npm and yarn take options and nesting
before it (`npm --workspace ui publish`, `yarn npm publish`); the cost is
a refusal of an odd command whose argument happens to be the verb, which
is cheap to see and to rephrase. A quoted word is unquoted before
comparing; a command quoted as an argument of another (`echo "npm
publish"`) splits into `echo` and its words and is let through, since
`echo` is no program in the table. `npm run publish`, whose script is
almost always `npm publish`, is refused for the same reason.

A dry run (`--dry-run`) is refused too: one rule is easier to trust than
a list of flags that each tool spells its own way, and `npm pack` and
`cargo package` are what show a package without publishing it.

Alternatives: refuse only the ecosystems the repository declares
projections for. Rejected: it means reading the policy's components and
projections, which is the core's (core.31) and the landed revision's
(core.22), and a package manager's publish is never tailrace's way out
whichever projections are declared. Refuse only `npm publish` and `cargo
publish`. Rejected: the guide promises `mvn deploy`, `uv publish` and
`docker push`, and the statement says an ecosystem's own publish command,
not two.

### The refusal names the command and derives nothing

The reason reads, in substance: this repository publishes through
tailrace alone; land the work first, then run `tailrace publish
<revision>` for the landed revision, which derives the version, packs and
offers every component's artifacts and attaches their claims. It names
no version, component or channel, since each is the core's to derive
(core.13, core.31, core.37), and a guess in the reason would be a second
answer beside the core's. It does not run `tailrace`: the hook works on a
host that has not installed it, and refusing needs no answer of the
core's.

### The gate checks the plugin as sluice's gate checks sluice's

- `plugins/claude-code/moon.yml`: `language: 'bash'`, with `hooks`
  (`bash tests/hooks.sh`, the guard on scratch repositories) and
  `validate` (`../claude.sh plugin validate --strict .`), both
  `check-fast`.
- `plugins/claude.sh`: the pinned `claude` with a scratch `HOME`, every
  `CLAUDE*` and `ANTHROPIC_*` variable unset and
  `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`, so the check reads none of
  the user's configuration or credentials (core.38).
- `plugins/check-marketplace.sh` and a root `marketplace` task in
  `moon.yml`: the marketplace validated and every listed plugin found
  with a manifest.
- `devenv.nix`: Claude Code 2.1.296, the release sluice's plugins are
  checked with and the site's mockups draw, fetched as sluice's
  `devenv.nix` fetches it, with its four checksums copied from there
  (core.26).

Scratch directories go under `SLUICE_TMP` where sluice's sandbox sets it,
as sluice's tests do. The shell files fall under treefmt's existing
shellcheck and shfmt, the JSON under Biome, the README under the `*.md`
exclude; no new file type, so `treefmt.nix` is unchanged.

## Risks / Trade-offs

- [A command the table misses, or a publish behind a script] → the hook
  is a guide for an agent, not a wall; the guide says so, and a row is a
  one-line change with a test.
- [A false refusal of an odd command] → the reason says what was refused
  and why; rephrasing it is the agent's next step.
- [Claude Code's hook contract changes] → the pin holds the gate to one
  release; the hooks test checks the deny document, and `plugin validate
  --strict` checks the manifest, on every bump.

## Migration Plan

None: a new plugin, installed by whoever chooses to.

# Proposal

## Why

In a repository tailrace publishes, a coding agent that runs `npm
publish` or `cargo publish` puts bytes in a public registry that no
landing's verdicts named, no gate accepted and no channel records,
which is exactly what tailrace exists to prevent (15, 20, I2). The
statement fixes the answer as a surface (section 9): a Claude Code
plugin, kept outside the core as sluice keeps its plugins, whose hook
refuses an ecosystem's own publish command and names `tailrace
publish`. Bolt 5 is usable only once Claude Code refuses `npm publish`
in a tailrace repository.

## What Changes

- A Claude Code plugin, `tailrace`, under `plugins/claude-code/`, offered
  by a marketplace named `tailrace` at the repository's root, so the
  guide's two install lines (`/plugin marketplace add agentplot/tailrace`,
  `/plugin install tailrace@tailrace`) work as written.
- Its one hook, run before every Bash call: in a repository whose HEAD
  holds `.config/tailrace.toml`, a command that is an ecosystem's own
  publish command (`npm publish`, `cargo publish`, `mvn deploy`, `uv
  publish`, `docker push` and the rest the design lists) is refused, and
  the refusal names `tailrace publish` for a landed revision. Everywhere
  else, and for every other command, the hook says nothing.
- The plugin computes no verdict, version or channel and reads no store,
  record or policy of its own (56, core.31): the refusal is its only
  answer, and it names only the command to run.
- The plugin is a moon project of its own with its tests and its
  manifest's validation in `check-fast` (core.25); the marketplace is
  checked by a root task; Claude Code is pinned in `devenv.nix` at the
  release sluice's plugins are checked with (core.26).
- The Claude Code guide on the site says what the hook refuses, now that
  it is built.

Out of scope, each waiting on a later change:

- The session note that says what the repository publishes and where its
  last landing reached, the `/tailrace` pane, and the band and status
  line the guide draws: each renders an answer of `tailrace status`, and
  waits on the channels and environments that answer reads (the gate and
  the channels bolt, and the herdr plugin's status page in this bolt).
- The setup skill that proposes a `.config/tailrace.toml`: it waits on
  the policy command that shows the table in force (C.10), not yet built.
- Refusing a promotion, or a channel moved by hand: no command moves a
  channel yet; the gate and the channels bolt builds them.

## Capabilities

### New Capabilities

- `claude-code-plugin`: tailrace's surface in a Claude Code session: the
  plugin's install, the hook that refuses an ecosystem's own publish
  command in a tailrace repository and names `tailrace publish`, and the
  rule that it answers nothing the one core does not.

### Modified Capabilities

None.

## Impact

- New: `plugins/claude-code/` (manifest, hooks, scripts, tests, README,
  moon project), `.claude-plugin/marketplace.json`, `plugins/claude.sh`,
  `plugins/check-marketplace.sh`.
- Changed: `.moon/workspace.yml` (the plugin's project), `moon.yml` (the
  marketplace task), `devenv.nix` (Claude Code pinned),
  `site/src/content/docs/guides/claude-code.mdx`.
- No crate changes; nothing under `crates/` names the plugin or the
  commands it refuses (core.1).

## Touches

```
plugins/claude-code/**
plugins/claude.sh
plugins/check-marketplace.sh
.claude-plugin/marketplace.json
.moon/workspace.yml
moon.yml
devenv.nix
site/src/content/docs/guides/claude-code.mdx
```

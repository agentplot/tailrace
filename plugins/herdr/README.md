# tailrace for herdr

tailrace beside your agents in [herdr](https://herdr.dev): a pane of every
repository this host knows, each channel's version and the environments,
and one repository whole.

```sh
herdr plugin install agentplot/tailrace/plugins/herdr
```

Needs herdr 0.9.3 or later, on Linux or macOS, with a `tailrace` that has
`serve` on `PATH`, and `curl` and `jq`. It reads everything from the
host's one serving process at `http://127.0.0.1:7420`, the address no
setting moves. It never publishes, promotes or names a repository.

## What the pane shows

The `status` action opens the pane as a split beside the pane you are in,
where it stays while your agents run.

- **Every repository the host knows**, one row each, in the order the
  process gives them, from `/status.json`: every channel's component, name
  and the version it is at or that it is empty, and how many of the
  repository's environments run what they should, are behind, wait on a
  person or cannot be read. Until a target is read, the environments say
  so in tailrace's words, never as zeros. A repository appears once a
  tailrace command run in a checkout of it names it, as
  `tailrace status --page` does.
- **One repository whole**, on opening its row, from
  `/<owner>/<name>/status.json`, which is what `tailrace status --json`
  prints in its checkout: the branch and the landed revision its policy
  was read at, each channel with its version or that it is empty and the
  chain of promotions that put it there, newest first, with the version
  each moved from, who asked, when, and the claim; then the environments
  and what this host published.

An answer that is not a value leads with its own word wherever it stands,
`absent`, `refused` or `unreadable`, and the source's own words follow it
unchanged. Digests and revisions are shown short; the JSON holds them
whole. The pane renders what the process answers and works out nothing
of its own: it counts nothing and reads no store, ledger, policy or git.

It asks again every five seconds while it is open. The keys:

| key | does |
|---|---|
| ↑ ↓ or `k` `j` | move the cursor over the rows |
| Enter, or a row's number `1` to `9` | open that repository |
| `b` or Backspace | go back to every repository |
| `r` | ask again now |
| Page Up, Page Down | scroll a frame taller than the pane; ↑ ↓ scroll one repository |
| `q` or Escape | close the pane |

## Starting the process

Where nothing answers at the address, the pane starts `tailrace serve`
once, as any tailrace command that needs it does: detached from the
pane's terminal, its working directory the plugin's state directory and
never a checkout. It waits up to five seconds for it to answer. If it
never does, the pane says so and starts no second one. Where something
other than tailrace answers there, the pane shows what answered and
starts nothing. Two panes that start one at once find the same process:
tailrace's own lock keeps a host to one.

## A key for it

The plugin changes none of herdr's config. Add a key to herdr's
`config.toml`:

```toml
[[keys.command]]
key = "prefix+t"
type = "plugin_action"
command = "agentplot.tailrace.status"
description = "tailrace: every repository on this host"
```

## Its checks

The moon project `plugin-herdr`: `manifest` reads `herdr-plugin.toml` as
herdr does; `test-sh` and `test-dash` run the scripts under sh and dash
against stand-in `curl`, `tailrace`, `herdr` and `git`, rendering the
documents the `cli` crate's `herdr_documents` test makes from the binary
and comparing each frame with the one committed in `tests/docs/frames`.
`TAILRACE_HERDR_FRAMES=write sh tests/run.sh` writes those frames again.

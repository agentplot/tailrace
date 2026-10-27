# Design

## Context

The motivation is in the proposal. These are the facts the design rests on.

**The serving process, as bolt 4 builds it**
(`bolt/tailrace-the-gate-and-the-channels`:
`crates/tailrace/src/commands/serve/`, `crates/tailrace-core/src/served.rs`,
`crates/tailrace-core/src/status.rs`):

- It answers HTTP at `SERVING_HOST`:`SERVING_PORT`, `127.0.0.1:7420`.
  No setting, flag or variable moves that address (114, core.36).
- `GET /serving.json` answers `{"serving": "tailrace", "revision", "pid"}`.
- `GET /status.json` answers the core's `Repositories` in the one answer
  form (57): `{"answer": "ok", "value": {"asked_at", "repositories":
  [...]}}`. Each element of the list is itself an answer of a `Row`:
  `repository {owner, name}`, `heads` (an answer of a list of `Head
  {component, channel, at}`, `at` an answer of `{"state": "empty"}` or
  `{"state": "at", "version", "index"}`), and `environments` (an answer
  of `{converged, behind, waiting, unreadable}`).
- `GET /<owner>/<name>/status.json` answers the core's `Overview` in the
  checkout that last named the repository. It is exactly what `tailrace
  status --json` prints there: `branch`, `asked_at`, and `channels`,
  `environments` and `published`, each its own answer. `channels` holds
  `revision`, `landing` (how it was read) and, per channel, the
  declared channel and `at`, an answer of `Channel` `Empty {read}` or
  `At {head, chain, read}`.
- Any answer that is not a value is `{"answer": "absent" | "refused" |
  "unreadable", "<kind>": {...}}`. It carries the source and its words
  (58, 59). Bolt 4 serves an absence with HTTP 404 and anything else
  that is not a value with HTTP 500, each with the same JSON body.
- An asker finds the process by `/serving.json`. Where nothing answers,
  it spawns `tailrace serve` detached, with its working directory the
  state directory, never a checkout. It then asks again every 100 ms for
  up to five seconds (`host::find_or_start`, `status::detached`). `serve`
  takes the host's serving lock before it binds. One that finds the lock
  held prints where the first answers and exits 0. It ends after ten
  minutes with nothing asked.
- In phase 1 every overview's `environments` is unreadable, naming that
  no target is read (98). The register keeps 132's environments part
  pending on the target unit.

**sluice's herdr plugin** (agentplot/sluice `plugins/herdr/`) is the
model for the plugin's shape:

- POSIX sh scripts under `bin/` and `lib/`, jq programs in
  `lib/words.jq`, and `herdr-plugin.toml` with actions and panes.
- `bin/status.sh` draws a frame and redraws it each second. Its keys
  are `q`/Escape to close, `a` to switch between this repository and
  every one, and arrows and Page Up/Down to scroll. `--once` prints one
  frame.
- Its tests (`tests/run.sh`) run every script against stand-in `herdr`
  and `sluice` on `PATH`, under sh and under dash. Fixture documents are
  compared byte for byte.
- It is the moon project `plugin-herdr`, with `manifest`, `test-sh` and
  `test-dash` tagged `check-fast`. Its treefmt excludes the jq and awk
  programs and the fixture documents by name, and hands the
  extension-less stand-ins to shellcheck and shfmt.

**herdr 0.9.5's plugin API** (herdr.dev, `docs/plugins.mdx` at v0.9.5):

- A manifest pane takes `placement` `overlay`, `popup`, `split`, `tab`
  or `zoomed`.
- Commands receive `HERDR_PLUGIN_ROOT`, `HERDR_PLUGIN_STATE_DIR`,
  `HERDR_PLUGIN_CONTEXT_JSON` and `HERDR_BIN_PATH`.
- `herdr plugin pane open --plugin <id> --entrypoint <pane>` opens a
  pane.
- Durable plugin state belongs under `HERDR_PLUGIN_STATE_DIR`, never
  the plugin root.

## Goals / Non-Goals

**Goals:**

- The pane renders the two documents the process serves, and nothing of
  its own.
- Starting the process is the pane's only act. It is done the same way
  any asker does it, and the core's lock keeps it to one process.
- The tests tie the pane to the binary's real documents without binding
  the core's port in the gate.

**Non-Goals:**

- Live updates pushed from the process. The process renders on each
  request and keeps nothing (132). The pane asks again instead.
- Reading any repository the process does not know.
- A Windows build. The manifest names linux and macos, as sluice's does.

## Decisions

### D1. The plugin's shape: sluice's, smaller

`plugins/herdr/` holds the following:

- `herdr-plugin.toml`: id `agentplot.tailrace`, name `tailrace`,
  `min_herdr_version = "0.9.3"` (the version sluice's plugin needs),
  platforms linux and macos. It declares one action and one pane:
  - action `status`, contexts `global` and `pane`, which runs
    `bin/open.sh status`
  - pane `status`, `placement = "split"`, which runs `bin/status.sh`
- `bin/open.sh`: `herdr plugin pane open --plugin agentplot.tailrace
  --entrypoint "$1"` through `HERDR_BIN_PATH`.
- `bin/status.sh`: the pane: frames, keys, redraws.
- `lib/common.sh`: the address, the ask, finding or starting the
  process.
- `lib/words.jq`: the rendering of every document and answer.
- `README.md`: what the pane shows, how to install it (`herdr plugin
  install agentplot/tailrace/plugins/herdr`), the key snippet for
  herdr's `config.toml`, and what it needs (herdr 0.9.3 or later, a
  `tailrace` with `serve` on `PATH`, `curl` and `jq`).
- `tests/` and `moon.yml`.

A split, not a popup, because the pane is meant to stay beside the work
while the agents run. A popup takes all input and closes when it ends.
This choice is the user's word "a pane beside their work" and section
9's "a pane". Alternative considered: a popup and a tab, as sluice has.
More entry points than the statement asks for, and either can be added
later without changing what the pane shows.

No toasts, sidebar tokens or watcher. sluice's watcher exists to feed
those. A pane that is open can ask the process itself.

### D2. One address, two documents, asked with curl

`lib/common.sh` names the address once:
`address=127.0.0.1:7420`, with a comment citing 114. No variable or
config file is read for it. The pane asks with `curl --silent
--show-error --max-time 2 "http://$address$path"`. It keeps the body
and the HTTP status apart. It does not use `--fail`, because a 404 or
500 carries an answer to render (58).

- The overview is `GET /status.json`.
- A repository is `GET /<owner>/<name>/status.json`, with the owner and
  name taken from the row's `repository` as the process gave them.
- A body that is not JSON in the one answer form is shown verbatim as
  what the process answered, never parsed into a guess.
- A refused connection, a timeout, or anything else curl reports is
  shown with curl's own words.

The pane asks again every five seconds on its own, and at once on `r`.
Each time is one request, rendered whole. The asks also keep the
process from going idle while a pane is open.

Alternative considered: run `tailrace status --json` in each checkout.
That is the command line's surface, not the served one, and it needs a
checkout per repository. Section 9 says the pane reads the served
page's own answer.

### D3. Finding and starting the process

Before each ask of a document, `lib/common.sh` asks `GET
/serving.json`:

- **tailrace's document** (`serving == "tailrace"`): go on.
- **Connection refused or reset**: run `tailrace serve` once, as
  `status::detached` does. The command is `(cd
  "$HERDR_PLUGIN_STATE_DIR" && nohup tailrace serve </dev/null
  >/dev/null 2>&1 &)`. Its working directory is the plugin's state
  directory, never a checkout. It is detached from the pane's terminal
  and its hangup. The pane then asks `/serving.json` every 100 ms for up
  to five seconds. If nothing answers in that time, the frame says the
  process it started did not answer in five seconds. It starts no
  second process during that ask.
- **Anything else** (a non-200, or a body that is not tailrace's): the
  frame says something else answers at the address, with what it
  answered, and nothing is started.
- **`tailrace` not on `PATH`**: the frame says so and names `tailrace
  serve`.

Never two processes: the core's lock decides that. A `serve` started
while another holds the lock exits at once. Two panes that both find
nothing may each start one, and one of the two ends. Each pane then
finds the same process through `/serving.json`. The plugin adds no lock
of its own, because it cannot make the core's guarantee any stronger.

The pane never runs `tailrace status --page`, so it never names a
repository to the host. Naming a repository is what a command run in a
checkout does (132). A pane opened over a checkout that doesn't use
tailrace would otherwise add a row for it. Alternative considered:
naming the focused pane's checkout so it shows up. That is outside the
intent, and it needs a checkout to start the process at all.

### D4. Rendering: words.jq, the core's words, no arithmetic

`lib/words.jq` holds one function per document and one for an answer
in the one form. `answer(f)` renders `ok` through `f`. It renders
`absent`, `refused` and `unreadable` as three distinct leading words,
followed by the source and its words verbatim, with control characters
stripped. Every place an answer can stand goes through it.

The every-repository frame:

```
tailrace · 2 repositories · as of 2026-10-26 14:02:11Z
[1] agentplot/tailrace   tailrace beta 0.4.0 · tailrace stable empty
    environments: unreadable — no target is read
[2] agentplot/sluice     unreadable — <the source's words>
```

When the counts are a value, the second line reads `3 running what
they should · 1 behind · 1 waiting on a person · 0 cannot be read`.

The one-repository frame:

```
agentplot/tailrace · main at <revision>, landed (as last fetched <time>)
beta (tailrace) 0.4.0 <index, short>
  ← 0.3.0 by <asker> at <time> · <claim, short>
  ← …
stable (tailrace) empty
environments: unreadable — no target is read
published on this host: absent — <words>
b back · r ask again · q close
```

- Times are rendered UTC from the answer's epoch seconds with jq's
  `todate`.
- Digests are shown short (algorithm and the first 12 hex). The full
  digest is in the JSON.
- The renderer formats. It never counts rows into totals, never takes a
  head from a chain, and never compares versions or digests.
- The repository count in the heading is the length of the list the
  process gave.

### D5. Keys

The keys follow sluice's popup:

- `q` or Escape closes the pane.
- Arrows and `j`/`k` move a cursor over rows. Enter, or a row's number
  `1` to `9`, opens a row.
- `b` or Backspace goes back.
- `r` asks again.
- Arrows and Page Up/Down scroll a frame taller than the pane.
- `--once` prints one frame, of every repository or, with `--repo
  <owner>/<name>`, of one, then exits. The tests use it.

### D6. The proof in three parts

1. **The plugin's tests** (`tests/run.sh`, check-fast, under sh and
   dash). Stand-ins `curl`, `tailrace` and `herdr` sit first on `PATH`,
   in `tests/fake/`. The stand-in `curl` answers each path from files in
   a scratch directory and records every URL it was asked. A missing
   `serving.json` marker means the connection is refused. The stand-in
   `tailrace` records its arguments. For `serve`, it takes a `mkdir`
   lock, writes the marker, and exits, or exits at once where the lock
   is held. The tests:
   - render the fixture documents and compare each frame byte for byte
     with a committed `.frame`
   - cover each answer kind at each place it can stand
   - cover the start cases of D3
   - check that only the core's address was asked, that no `git` or
     tailrace command but `serve` ran, and that two `--once` runs
     started together leave one marker
2. **The documents are the binary's.** `crates/tailrace/tests/herdr_documents.rs`
   (cli, check-fast, git alone) builds a scratch host as
   `tests/serve.rs` does. It has a remote and its clone, a policy
   declaring `beta` and `stable`, two promotions of `beta` in the git
   ref ledger, no record of a publish, and a second named repository
   whose checkout is gone. It asks `serve::respond_with` for `/status.json`
   and the repository's `status.json`, with no port bound. It replaces
   each time, revision and digest with a fixed value of the same form.
   It compares the result with `plugins/herdr/tests/docs/never-published/*.json`
   and fails naming the first difference. Run with
   `TAILRACE_HERDR_DOCS=write`, it writes them instead. It also asserts
   that the repository's document equals `tailrace status --json` run
   in the clone, after the same replacement. The plugin's tests render
   those committed documents.
3. **The real process, by hand.** `tests/host.sh` is the
   `plugin-herdr:host` task, tagged `check-host` and in no gate. It
   refuses to run, naming the address, when anything answers at
   `127.0.0.1:7420`. It builds `tailrace`, seals `HOME`,
   `XDG_STATE_HOME` and `XDG_CONFIG_HOME` in a scratch directory, names
   a scratch repository with `tailrace status --page`, and stops the
   process that call started. It then runs two `status.sh --once`
   together with the real `curl`. It asserts that `/serving.json`
   answers one pid, that one `tailrace serve` runs under the scratch
   state, and that both frames are equal. Then it ends the process.

The third part is outside the gate because `serve` always binds the
core's port, and bolt 4 keeps every gate test off it. A gate run on a
host whose own process holds the port would find that process, under
another state directory. Two worktrees' gates would race for the port.

### D7. The gate

- `.moon/workspace.yml` gains `plugin-herdr: 'plugins/herdr'`.
- `plugins/herdr/moon.yml` sets `language: 'bash'`, with tasks
  `manifest`, `test-sh` and `test-dash`, tagged `check-fast`, and
  `host`, tagged `check-host`. Each task's inputs name what it reads,
  `/devenv.nix` and `/devenv.lock` among them.
- `tests/manifest.sh` reads the manifest with awk, as sluice's
  `tests/manifest.sh` does, and checks that
  every action's pane exists, every command's script exists, and the id
  is `agentplot.tailrace`.
- `treefmt.nix` excludes `plugins/herdr/lib/*.jq` (no formatter or
  linter for jq in the toolchain; the tests parse every program under
  both shells) and `plugins/herdr/tests/docs/**` (compared byte for
  byte). It adds `plugins/herdr/tests/fake/*` to shellcheck's and
  shfmt's includes. `herdr-plugin.toml` and `moon.yml` are left to taplo
  and yamlfmt as the tree already has them.
- `devenv.nix` adds `pkgs.dash` and `pkgs.curl` beside `pkgs.jq`, so
  every host runs the pinned ones (core.26).
- `crates/tailrace/moon.yml`'s `test` inputs gain
  `/plugins/herdr/tests/docs/**/*`.

## Risks / Trade-offs

- [Bolt 4's JSON changes before it lands] → The `cli` test compares
  against the plugin's committed documents and fails naming the
  difference. A change in shape cannot pass the gate unseen.
- [Two panes both start a process] → The core's lock ends the second at
  once, and both find the first. The by-hand proof shows it against the
  real binary.
- [The real-process proof is not in the gate] → It is a named task with
  its own tag, refusing to run where the port is taken. The gate still
  proves the pane's rendering and its starts against stand-ins, and the
  documents against the binary.
- [Polling every five seconds keeps the process alive] → It does so
  only while a pane is open. The process renders per request and writes
  nothing (60, 132).
- [Environment counts are unreadable everywhere in phase 1] → The pane
  shows that answer in its words, never as zeros. Counts are rendered
  as given once targets report.

## Migration Plan

None. A new plugin, installed by whoever wants it. Removing it is
`herdr plugin uninstall agentplot.tailrace`.

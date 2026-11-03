#!/bin/sh
# shellcheck disable=SC2016 # within's conditions expand where they are evaluated.
# The pane against the real serving process at the core's address, by
# hand: plugin-herdr's host task, tagged check-host and in no gate, since
# `tailrace serve` always binds the core's port, which is the host's own
# (114). It refuses to run where anything answers there. It builds
# tailrace, names a scratch repository to a scratch state with
# `tailrace status --page`, and ends the process that call started; then
# two panes opened at once, where nothing answers, find one process, and
# draw one frame. It ends the process before it exits.

set -u
here=$(cd "$(dirname "$0")" && pwd -P)
plugin=$(dirname "$here")
tree=$(cd "$plugin/../.." && pwd -P)
address=127.0.0.1:7420

# Whether nothing answers at the address: curl's connection refused.
nothing_answers() {
  curl --silent --max-time 2 --output /dev/null "http://$address/serving.json"
  [ $? = 7 ]
}

if ! nothing_answers; then
  echo "host: something answers at http://$address; this proof binds that address, so it runs only where nothing answers there" >&2
  exit 1
fi

target=$(cd "$tree" && cargo metadata --format-version 1 --no-deps | jq -r .target_directory) || exit 1
(cd "$tree" && cargo build -q -p tailrace) || exit 1
bin=$target/debug

s=$(mktemp -d "${TMPDIR:-/tmp}/tailrace-herdr-host.XXXXXX")
s=$(cd "$s" && pwd -P)
pids=
# Each process this proof started, ended, and the scratch removed.
finish() {
  for p in $pids $(serves); do kill "$p" 2>/dev/null; done
  rm -rf "$s"
}
trap finish EXIT
trap 'exit 1' INT TERM HUP

# The working directory of the process $1.
cwd_of() {
  if [ -d "/proc/$1" ]; then
    readlink "/proc/$1/cwd"
  else
    lsof -a -p "$1" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p'
  fi
}

# Every tailrace serve running in this proof's scratch directory, which
# is where each it started works: the state directory, never a checkout.
serves() {
  ps -eo pid=,args= | awk '$2 ~ /(^|\/)tailrace$/ && $3 == "serve" && NF == 3 { print $1 }' |
    while read -r p; do
      case $(cwd_of "$p") in "$s"/*) echo "$p" ;; esac
    done
}

# The pid the process at the address names, or nothing.
serving_pid() {
  curl --silent --max-time 2 "http://$address/serving.json" 2>/dev/null | jq -r '.pid // empty' 2>/dev/null
}

# Wait up to $1 seconds for the shell condition $2, checking every tenth.
within() {
  n=$(($1 * 10))
  while [ "$n" -gt 0 ]; do
    eval "$2" && return 0
    sleep 0.1
    n=$((n - 1))
  done
  eval "$2"
}

failures=0
fail() {
  failures=$((failures + 1))
  printf 'FAIL %s\n' "$*"
}

# The host sealed in the scratch directory, and git with none of the
# host's configuration.
mkdir -p "$s/home" "$s/state" "$s/config" "$s/data" "$s/panes"
export HOME="$s/home" XDG_STATE_HOME="$s/state" XDG_CONFIG_HOME="$s/config" XDG_DATA_HOME="$s/data"
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_TERMINAL_PROMPT=0
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@localhost GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@localhost
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE

# A repository whose landing's remote names agentplot/scratch, which git
# reads from a scratch remote.
git init -q --bare -b main "$s/remote.git"
git init -q -b main "$s/repo"
mkdir -p "$s/repo/.config"
printf '[channels.beta]\nrequire = []\n\n[channels.stable]\nbound_to = "beta"\nrequire = []\n' >"$s/repo/.config/tailrace.toml"
git -C "$s/repo" add -A
git -C "$s/repo" commit -q -m "feat: the channels"
named=https://example.com/agentplot/scratch.git
git -C "$s/repo" remote add origin "$named"
git -C "$s/repo" config "url.$s/remote.git.insteadOf" "$named"
git -C "$s/repo" push -q origin main

page=$(cd "$s/repo" && "$bin/tailrace" status --page) || {
  echo "host: tailrace status --page failed: $page" >&2
  exit 1
}
[ "$page" = "http://$address/agentplot/scratch/status" ] || fail "status --page printed: $page"
first=$(serving_pid)
[ -n "$first" ] || fail "status --page left no process answering"
[ "$(serves)" = "$first" ] || fail "the process answering, $first, is not the one status --page started in the scratch state"
kill "$first" 2>/dev/null
within 10 nothing_answers || fail "the process status --page started still answers"

# Two panes at once, where nothing answers, each free to start one.
export PATH="$bin:$PATH"
for n in 1 2; do
  HERDR_PLUGIN_STATE_DIR="$s/panes/$n" sh "$plugin/bin/status.sh" --once >"$s/frame$n" 2>&1 &
  pids="$pids $!"
done
for p in $pids; do wait "$p"; done
pids=

pid=$(serving_pid)
case $pid in
'' | *[!0-9]*) fail "/serving.json names no one pid: $pid" ;;
esac
# The second serve, finding the lock held, ends at once.
within 5 '[ "$(serves | wc -l | tr -d " ")" = 1 ]' || fail "not one tailrace serve runs: $(serves | tr '\n' ' ')"
[ "$(serves)" = "$pid" ] || fail "the tailrace serve running, $(serves | tr '\n' ' '), is not the one answering, $pid"
[ -f "$s/state/tailrace/serve.lock" ] || fail "the process holds no lock in the scratch state"
# Both frames drew the one process's answer; the moment each asked aside.
for n in 1 2; do sed 's/ · as of [^ ]*$//' "$s/frame$n" >"$s/drawn$n"; done
diff -u "$s/drawn1" "$s/drawn2" >"$s/diff" || fail "the two frames differ: $(cat "$s/diff")"
grep -q '^  \[1\] agentplot/scratch  repository beta empty · repository stable empty$' "$s/frame1" ||
  fail "the frame is not the scratch repository's: $(cat "$s/frame1")"

kill "$pid" 2>/dev/null
within 10 nothing_answers || fail "the process still answers once ended"

cat "$s/frame1"
echo
if [ "$failures" -gt 0 ]; then
  echo "$failures failed"
  exit 1
fi
echo "two panes, one process ($pid), one frame"

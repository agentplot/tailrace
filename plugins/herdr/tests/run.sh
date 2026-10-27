#!/bin/sh
# shellcheck disable=SC2016 # jq programs name their own $variables.
# The herdr plugin's scripts against stand-in curl, tailrace, herdr and git
# first on PATH: the documents the serving process answers, from files,
# and every URL asked and command run recorded and checked. Nothing here
# reaches a network, a running tailrace or a running herdr.

set -u
here=$(cd "$(dirname "$0")" && pwd -P)
plugin=$(dirname "$here")

# Under sluice's sandbox only SLUICE_TMP is writable beside the checkout.
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/tailrace-herdr.XXXXXX")
scratch=$(cd "$scratch" && pwd -P)
trap 'rm -rf "$scratch"' EXIT

unset HERDR_PLUGIN_CONTEXT_JSON HERDR_SOCKET_PATH HERDR_ENV LINES
# TAILRACE_HERDR_SH names the shell every script runs under as `sh`, as
# herdr runs them: dash, as Debian's /bin/sh is, or the system's.
if [ -n "${TAILRACE_HERDR_SH:-}" ]; then
  mkdir -p "$scratch/sh"
  ln -s "$(command -v "$TAILRACE_HERDR_SH")" "$scratch/sh/sh"
  export PATH="$scratch/sh:$PATH"
fi
echo "scripts run under $(command -v sh) -> $(readlink "$(command -v sh)" || echo itself)"
export PATH="$here/fake:$PATH" HERDR_BIN_PATH="$here/fake/herdr" HERDR_PLUGIN_ROOT="$plugin"

failures=0
failed_before=0
count=0
name=

# A fresh state and stand-in directory for one test, nothing answering.
fresh() {
  name=$1
  failed_before=$failures
  count=$((count + 1))
  t=$scratch/t$count
  mkdir -p "$t/state" "$t/fake/http"
  export HERDR_PLUGIN_STATE_DIR="$t/state" FAKE="$t/fake"
  for log in curl-calls tailrace-calls herdr-calls git-calls; do : >"$FAKE/$log"; done
}

fail() {
  failures=$((failures + 1))
  printf 'FAIL %s: %s\n' "$name" "$*"
}

pass() {
  [ "$failures" = "$failed_before" ] && printf 'ok   %s\n' "$name"
  return 0
}

# tailrace's process answering /serving.json.
answering() {
  printf '{"serving":"tailrace","revision":"test","pid":1}\n' >"$FAKE/http/serving.json"
}

# The documents of docs/$1, served at their paths.
documents() {
  cp -R "$here/docs/$1/." "$FAKE/http/"
}

status() {
  sh "$plugin/bin/status.sh" "$@"
}

# The lines of a log.
lines() {
  wc -l <"$FAKE/$1" | tr -d ' '
}

# PATH without the stand-in tailrace and any other: the stand-ins but
# tailrace linked into a directory of their own, then every entry of PATH
# holding no tailrace.
without_tailrace() {
  mkdir -p "$t/bin"
  for f in curl herdr git; do ln -sf "$here/fake/$f" "$t/bin/$f"; done
  p=$t/bin
  old_ifs=$IFS
  IFS=:
  for d in $PATH; do
    [ "$d" = "$here/fake" ] && continue
    [ -x "$d/tailrace" ] && continue
    p=$p:$d
  done
  IFS=$old_ifs
  echo "$p"
}

# The --once frame of the documents of docs/$1, of every repository or,
# with $2, of that one, compared byte for byte with the frame committed
# for it in docs/frames, or written there with TAILRACE_HERDR_FRAMES=write.
framed() {
  file=$here/docs/frames/$1${2:+.$(printf '%s' "$2" | tr / .)}.frame
  if [ -n "${2:-}" ]; then status --once --repo "$2" >"$t/frame"; else status --once >"$t/frame"; fi
  if [ "${TAILRACE_HERDR_FRAMES:-}" = write ]; then
    mkdir -p "$here/docs/frames"
    cp "$t/frame" "$file"
    return
  fi
  diff -u "$file" "$t/frame" >"$t/diff" || fail "the frame differs from $(basename "$file"): $(cat "$t/diff")"
}

# Frame $1 of a run off a terminal, each frame ended by a form feed.
nth() {
  awk -v n="$1" 'BEGIN { RS = "\f" } NR == n { sub(/^\n/, ""); printf "%s", $0 }' "$t/out"
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

fresh 'open: the action opens the pane through herdr'
sh "$plugin/bin/open.sh" status >/dev/null
grep -qxF 'plugin pane open --plugin agentplot.tailrace --entrypoint status' "$FAKE/herdr-calls" ||
  fail "herdr was asked: $(cat "$FAKE/herdr-calls")"
pass

fresh 'manifest: the plugin'"'"'s manifest is read as herdr reads it'
sh "$here/manifest.sh" >"$t/out" 2>&1 || fail "$(cat "$t/out")"
mkdir -p "$t/bad/bin"
cp "$plugin/bin/open.sh" "$t/bad/bin/"
sed -e 's/"agentplot.tailrace"/"agentplot.other"/' -e 's/"bin\/open.sh", "status"/"bin\/open.sh", "nowhere"/' \
  -e 's/bin\/status.sh/bin\/gone.sh/' "$plugin/herdr-plugin.toml" >"$t/bad/herdr-plugin.toml"
if sh "$here/manifest.sh" "$t/bad/herdr-plugin.toml" >"$t/out" 2>&1; then fail "a bad manifest passed"; fi
for said in 'the id is "agentplot.other"' 'opens a pane this plugin does not declare: "nowhere"' 'runs bin/gone.sh'; do
  grep -qF "$said" "$t/out" || fail "not said: $said: $(cat "$t/out")"
done
pass

fresh 'address: the scripts name the core'"'"'s address once and read nothing for it'
found=$(cd "$plugin" && grep -r '7420\|127\.0\.0\.1' bin lib)
[ "$found" = 'lib/common.sh:address=127.0.0.1:7420' ] || fail "the address is named as: $found"
grep -n 'address=' "$plugin/lib/common.sh" | grep -v ':address=127.0.0.1:7420$' && fail "the address is set again"
pass

fresh 'start: a process answering starts nothing'
answering
documents never-published
status --once >"$t/out"
[ "$(lines tailrace-calls)" = 0 ] || fail "tailrace ran: $(cat "$FAKE/tailrace-calls")"
grep -q '^tailrace · 2 repositories · as of ' "$t/out" || fail "no frame: $(cat "$t/out")"
pass

fresh 'start: none answering starts tailrace serve once, outside any checkout'
documents never-published
mkdir -p "$t/checkout/.git"
(cd "$t/checkout" && status --once) >"$t/out"
printf '%s\tserve\n' "$t/state" >"$t/want"
diff "$t/want" "$FAKE/tailrace-calls" >/dev/null || fail "tailrace ran as: $(cat "$FAKE/tailrace-calls")"
grep -q '^tailrace · 2 repositories · as of ' "$t/out" || fail "no frame once it answered: $(cat "$t/out")"
pass

fresh 'start: something else answering starts nothing and is shown with its words'
printf '<html>a\tdashboard\033[2J</html>\n' >"$FAKE/http/serving.json"
echo 200 >"$FAKE/http/serving.json.code"
status --once >"$t/out"
grep -qF "Something other than tailrace's serving process answers at http://127.0.0.1:7420." "$t/out" ||
  fail "not said: $(cat "$t/out")"
grep -qxF '<html>a dashboard[2J</html>' "$t/out" || fail "its words are not shown verbatim: $(cat "$t/out")"
[ "$(lines tailrace-calls)" = 0 ] || fail "tailrace ran: $(cat "$FAKE/tailrace-calls")"
grep -q 'status.json$' "$FAKE/curl-calls" && fail "a document was asked of it"
pass

fresh 'start: a start that never answers is said and not repeated'
: >"$FAKE/mute"
printf 'rq' | status >"$t/out"
[ "$(grep -c 'did not answer in five seconds' "$t/out")" = 2 ] || fail "not said in both frames: $(cat "$t/out")"
[ "$(lines tailrace-calls)" = 1 ] || fail "tailrace serve ran again: $(cat "$FAKE/tailrace-calls")"
pass

fresh 'start: tailrace missing from PATH is said'
documents never-published
PATH=$(without_tailrace) status --once >"$t/out"
grep -qF 'Nothing answers at http://127.0.0.1:7420, and tailrace is not on PATH to start tailrace serve.' "$t/out" ||
  fail "not said: $(cat "$t/out")"
pass

fresh 'start: two --once runs started together leave one marker'
documents never-published
status --once >"$t/one" &
first=$!
status --once >"$t/two" &
second=$!
wait "$first" "$second"
[ "$(lines served)" = 1 ] || fail "not one process took the lock: $(cat "$FAKE/served")"
diff "$t/one" "$t/two" >"$t/diff" || fail "the frames differ: $(cat "$t/diff")"
pass

fresh 'asks: only the core'"'"'s address is asked, and no command but tailrace serve runs'
documents never-published
status --once >"$t/out"
grep -v '^http://127\.0\.0\.1:7420/' "$FAKE/curl-calls" && fail "another address was asked"
[ "$(lines curl-calls)" -gt 1 ] || fail "nothing was asked: $(cat "$FAKE/curl-calls")"
cut -f2 "$FAKE/tailrace-calls" | grep -vx serve && fail "a tailrace command but serve ran"
[ "$(lines git-calls)" = 0 ] || fail "git ran: $(cat "$FAKE/git-calls")"
[ "$(lines herdr-calls)" = 0 ] || fail "herdr was asked: $(cat "$FAKE/herdr-calls")"
pass

fresh 'asks: an ask curl cannot make is said with curl'"'"'s words'
echo '28 Operation timed out after 2001 milliseconds with 0 bytes received' >"$FAKE/fail"
status --once >"$t/out"
grep -qF 'http://127.0.0.1:7420/serving.json could not be asked:' "$t/out" || fail "not said: $(cat "$t/out")"
grep -qF 'curl: (28) Operation timed out after 2001 milliseconds' "$t/out" || fail "not curl's words: $(cat "$t/out")"
[ "$(lines tailrace-calls)" = 0 ] || fail "tailrace ran: $(cat "$FAKE/tailrace-calls")"
pass

fresh 'frames: the binary'"'"'s documents, of a machine that never published'
answering
documents never-published
framed never-published
framed never-published agentplot/widget
pass

fresh 'frames: each answer kind at each place it can stand'
answering
documents kinds
framed kinds
for r in alpha beta gamma delta; do framed kinds "agentplot/$r"; done
pass

fresh 'frames: a host that knows no repository, and a root that cannot be read'
answering
documents empty
framed empty
documents unreadable
framed unreadable
pass

fresh 'every: a row shows each channel'"'"'s head and the four counts as answered'
answering
documents kinds
status --once >"$t/out"
grep -qF '[1] agentplot/alpha  app beta 1.2.0 · app stable refused · app edge empty · app nightly absent · app canary unreadable' "$t/out" ||
  fail "the heads: $(cat "$t/out")"
grep -qxF '      environments: 3 running what they should · 1 behind · 1 waiting on a person · 0 cannot be read' "$t/out" ||
  fail "the counts: $(cat "$t/out")"
pass

fresh 'every: environments not read are said, and never counted'
answering
documents never-published
status --once >"$t/out"
grep -qF 'environments: unreadable — the target (unread) could not read the environments' "$t/out" || fail "not said: $(cat "$t/out")"
grep -q 'running what they should\|[0-9] behind' "$t/out" && fail "counted: $(cat "$t/out")"
pass

fresh 'one: a chain refused is said with the ledger'"'"'s words, beside the chain that answers'
answering
documents kinds
status --once --repo agentplot/alpha >"$t/out"
grep -qxF 'beta (app) 1.2.0 sha256:aaaaaaaaaaaa, prereleases too' "$t/out" || fail "beta: $(cat "$t/out")"
grep -qxF '  ← 1.2.0 from 1.1.0, asked by gate at 2026-10-26T07:33:21Z, claim sha256:666666666666' "$t/out" || fail "the chain, newest first: $(cat "$t/out")"
grep -qF 'stable (app) refused — the keeper (git ref, origin) refused: fork: predecessor sha256:' "$t/out" || fail "stable: $(cat "$t/out")"
grep -qxF '   see the ledger'"'"'s [31mlog[0m' "$t/out" || fail "the ledger's words: $(cat "$t/out")"
grep -q '^stable.*\(empty\|absent\)' "$t/out" && fail "stable read as empty or absent"
pass

fresh 'one: nothing published is absent, never an empty list'
answering
documents never-published
status --once --repo agentplot/widget >"$t/out"
grep -qxF 'published on this host: absent — the host (/scratch/widget/repo/.git/tailrace/published) holds no version it published' "$t/out" ||
  fail "not said: $(cat "$t/out")"
pass

fresh 'one: the row'"'"'s counts and the repository'"'"'s are each as answered'
answering
documents kinds
status --once | grep -qF '3 running what they should · 1 behind' || fail "the row's counts"
status --once --repo agentplot/alpha | grep -qxF 'environments: 0 running what they should · 0 behind · 0 waiting on a person · 2 cannot be read' ||
  fail "the repository's counts"
pass

fresh 'keys: the cursor moves over the rows that open, Enter and a number open one, b goes back'
answering
documents kinds
printf 'j\nb3bkkq' | status >"$t/out"
nth 1 | grep -qxF '▸ [1] agentplot/alpha  app beta 1.2.0 · app stable refused · app edge empty · app nightly absent · app canary unreadable' ||
  fail "frame 1 has the cursor elsewhere: $(nth 1)"
nth 2 | grep -q '^▸ \[2\] agentplot/beta ' || fail "frame 2 has the cursor elsewhere: $(nth 2)"
nth 3 | head -1 | grep -qxF 'agentplot/beta · main, as of 2026-10-26T07:35:00Z' || fail "frame 3 is not beta: $(nth 3)"
nth 4 | grep -q '^▸ \[2\] agentplot/beta ' || fail "frame 4 did not go back to beta's row: $(nth 4)"
nth 5 | head -1 | grep -qxF 'agentplot/gamma · main at 111111111111, as of 2026-10-26T07:35:00Z' || fail "frame 5 is not gamma: $(nth 5)"
nth 7 | grep -q '^▸ \[2\] ' || fail "k did not move the cursor up: $(nth 7)"
nth 8 | grep -q '^▸ \[1\] ' || fail "k did not move the cursor up: $(nth 8)"
grep -qxF 'http://127.0.0.1:7420/agentplot/beta/status.json' "$FAKE/curl-calls" || fail "beta's path was not asked"
pass

fresh 'keys: a repository answered absent is said, and b goes back'
answering
documents kinds
printf 'b' | status --repo agentplot/delta >"$t/out"
nth 1 | grep -qxF 'absent — the host (/home/a/.local/state/tailrace/repositories) holds no repository agentplot/delta: no command run in a checkout of it on this host has named it' ||
  fail "not said: $(nth 1)"
nth 2 | head -1 | grep -qF 'tailrace · 6 repositories' || fail "b did not go to every repository: $(nth 2)"
pass

fresh 'keys: --repo names an owner and a name'
answering
documents kinds
status --once --repo alpha >"$t/out"
grep -qxF -- '--repo names a repository as <owner>/<name>, not alpha' "$t/out" || fail "not said: $(cat "$t/out")"
pass

fresh 'keys: r asks again, and a moved channel is shown'
answering
documents never-published
mkfifo "$t/keys"
status <"$t/keys" >"$t/out" &
pane_pid=$!
exec 3>"$t/keys"
within 10 '[ "$(grep -c "^↑↓ choose" "$t/out")" -ge 1 ]' || fail "no first frame"
jq '.value.repositories[1].value.heads.value[0].at.value.version = "0.3.0"' "$here/docs/never-published/status.json" >"$FAKE/http/status.json"
printf r >&3
within 10 '[ "$(grep -c "^↑↓ choose" "$t/out")" -ge 2 ]' || fail "no second frame"
printf q >&3
exec 3>&-
wait "$pane_pid"
nth 1 | grep -qF 'repository beta 0.2.0' || fail "frame 1: $(nth 1)"
nth 2 | grep -qF 'repository beta 0.3.0' || fail "frame 2 does not show the move: $(nth 2)"
pass

fresh 'keys: a frame taller than the pane keeps the cursor'"'"'s row drawn'
answering
documents kinds
printf 'jjq' | LINES=8 status >"$t/out"
for n in 1 2 3; do
  [ "$(nth "$n" | wc -l)" -le 8 ] || fail "frame $n is taller than 8 lines: $(nth "$n")"
done
nth 3 | grep -q '^▸ \[3\] agentplot/gamma' || fail "the cursor's row is not drawn: $(nth 3)"
pass

fresh 'every: a body that is no answer of tailrace'"'"'s is shown as it came'
answering
printf '<html>Bad Gateway</html>\n' >"$FAKE/http/status.json"
echo 502 >"$FAKE/http/status.json.code"
status --once >"$t/out"
grep -qxF "http://127.0.0.1:7420/status.json answered HTTP 502, which is no answer of tailrace's:" "$t/out" || fail "not said: $(cat "$t/out")"
grep -qxF '<html>Bad Gateway</html>' "$t/out" || fail "its words: $(cat "$t/out")"
pass

echo
if [ "$failures" -gt 0 ]; then
  echo "$failures failed of $count"
  exit 1
fi
echo "all $count passed"

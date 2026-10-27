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
grep -q 'every repository' "$t/out" || fail "no frame: $(cat "$t/out")"
pass

fresh 'start: none answering starts tailrace serve once, outside any checkout'
documents never-published
mkdir -p "$t/checkout/.git"
(cd "$t/checkout" && status --once) >"$t/out"
printf '%s\tserve\n' "$t/state" >"$t/want"
diff "$t/want" "$FAKE/tailrace-calls" >/dev/null || fail "tailrace ran as: $(cat "$FAKE/tailrace-calls")"
grep -q 'every repository' "$t/out" || fail "no frame once it answered: $(cat "$t/out")"
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

echo
if [ "$failures" -gt 0 ]; then
  echo "$failures failed of $count"
  exit 1
fi
echo "all $count passed"

# shellcheck shell=sh
# What every script of the plugin shares: where it is and where its state
# is, the one address it asks, and how it finds the host's serving
# process or starts it. Sourced, never run.

: "${HERDR_PLUGIN_ROOT:=$(cd "$(dirname "$0")/.." && pwd -P)}"
: "${HERDR_PLUGIN_STATE_DIR:=${XDG_STATE_HOME:-$HOME/.local/state}/herdr-agentplot.tailrace}"
root=$HERDR_PLUGIN_ROOT
state=$HERDR_PLUGIN_STATE_DIR
lib=$root/lib
mkdir -p "$state"

# The core's address, where the host's one serving process answers: no
# setting, flag or variable moves it (114, core.36).
address=127.0.0.1:7420

# What this pane keeps while it is open: the documents last asked, and
# whether a process it started never answered.
pane=$state/pane.$$
mkdir -p "$pane"

# jq with the plugin's words loaded.
jqw() {
  jq -L "$lib" "$@"
}

# A source's words as the frame shows them: unchanged but for control
# characters, which would move the pane's cursor; a tab is a space.
verbatim() {
  tr '\t' ' ' | tr -d '\000-\010\013-\037\177'
}

# One GET of the path $1 at the address, its body into the file $2: the
# HTTP status in $code, curl's exit in $rc and its words in $2.err. curl
# is not told to fail on a 404 or a 500, which carry an answer (58).
ask() {
  code=$(curl --silent --show-error --max-time 2 --output "$2" --write-out '%{http_code}' "http://$address$1" 2>"$2.err")
  rc=$?
}

# Whether an ask found nothing at the address: the connection refused, or
# reset while a process starts to listen.
nothing() {
  [ "$rc" = 7 ] || [ "$rc" = 56 ]
}

# Whether the last ask of /serving.json answered tailrace's document.
tailrace_answers() {
  [ "$rc" = 0 ] && [ "$code" = 200 ] &&
    jq -e '.serving == "tailrace"' "$pane/serving" >/dev/null 2>&1
}

# What answered /serving.json in place of tailrace's process, as the
# frame says it.
something_else() {
  echo "Something other than tailrace's serving process answers at http://$address."
  echo "Asked /serving.json, it answered HTTP $code:"
  verbatim <"$pane/serving"
  echo
}

# An ask that got no answer, with curl's own words.
unasked() {
  echo "http://$address$1 could not be asked:"
  verbatim <"$2.err"
}

# 114: the host's serving process, found by its /serving.json, or
# started where nothing answers, as any asker starts it: `tailrace serve`
# once, detached from the pane's terminal and its hangup, its working
# directory this plugin's state, never a checkout; then asked every
# 100 ms for up to five seconds. The core's lock keeps it to one process
# however many askers start one. Prints nothing where tailrace's process
# answers; otherwise says why not and fails. A process this pane started
# that never answered is not started again.
find_or_start() {
  ask /serving.json "$pane/serving"
  tailrace_answers && return 0
  if ! nothing; then
    if [ "$rc" = 0 ]; then something_else; else unasked /serving.json "$pane/serving"; fi
    return 1
  fi
  if [ -f "$pane/start-failed" ]; then
    echo "Nothing answers at http://$address. The tailrace serve this pane started did not answer in five seconds, and it starts no second one."
    return 1
  fi
  if ! command -v tailrace >/dev/null 2>&1; then
    echo "Nothing answers at http://$address, and tailrace is not on PATH to start tailrace serve."
    return 1
  fi
  (cd "$state" && nohup tailrace serve </dev/null >/dev/null 2>&1 &)
  tries=50
  while [ "$tries" -gt 0 ]; do
    sleep 0.1
    ask /serving.json "$pane/serving"
    tailrace_answers && return 0
    if [ "$rc" = 0 ]; then
      something_else
      return 1
    fi
    tries=$((tries - 1))
  done
  : >"$pane/start-failed"
  echo "Nothing answers at http://$address. The tailrace serve this pane started did not answer in five seconds, and it starts no second one."
  return 1
}

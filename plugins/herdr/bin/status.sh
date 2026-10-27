#!/bin/sh
# shellcheck disable=SC1091,SC2154 # common.sh, sourced, sets address, pane, rc and code.
# The pane: every repository the host's serving process knows, read at
# the core's address from the process, which it starts where none
# answers (114). r asks again, q closes. --once prints one frame and
# exits.
set -u
. "$(dirname "$0")/../lib/common.sh"
trap 'rm -rf "$pane"' EXIT

once=false
for a in "$@"; do
  case $a in
  --once) once=true ;;
  esac
done

# The document at the path $1 in $pane/doc, as the process answered it
# in the one form (57); otherwise what answered in its place, said.
document() {
  ask "$1" "$pane/doc"
  if [ "$rc" != 0 ]; then
    unasked "$1" "$pane/doc"
    return 1
  fi
  jq -e 'type == "object" and (.answer | type == "string")' "$pane/doc" >/dev/null 2>&1 && return 0
  echo "http://$address$1 answered HTTP $code, which is no answer of tailrace's:"
  verbatim <"$pane/doc"
  return 1
}

frame() {
  if find_or_start && document /status.json; then
    jqw -r 'include "words"; every' "$pane/doc"
  fi
  echo
  echo "r ask again · q close"
}

if [ "$once" = true ]; then
  frame
  exit 0
fi

frame
printf '\f\n'
while :; do
  key=$(dd bs=1 count=1 2>/dev/null)
  [ -z "$key" ] && exit 0
  case $key in
  q) exit 0 ;;
  esac
  frame
  printf '\f\n'
done

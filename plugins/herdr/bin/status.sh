#!/bin/sh
# shellcheck disable=SC1091,SC2016,SC2154 # common.sh, sourced, sets address, pane, rc and code; jq programs.
# The pane: every repository the host's serving process knows, one row
# each, and one repository whole, read at the core's address from the
# process, which it starts where none answers, and redrawn every five
# seconds and on r (114, 132). It renders what the process answers and
# computes nothing of its own (56, core.31).
#
# Keys: the arrows or j and k move the cursor over the rows, Enter or a
# row's number opens one, b or Backspace goes back, r asks again, Page Up
# and Down scroll, and the arrows scroll one repository; q or Escape
# closes. --once prints one frame, of every repository or, with
# --repo <owner>/<name>, of one, and exits.
set -u
. "$(dirname "$0")/../lib/common.sh"
trap 'rm -rf "$pane"' EXIT

once=false
repo=
while [ $# -gt 0 ]; do
  case $1 in
  --once) once=true ;;
  --repo)
    repo=${2:-}
    [ $# -gt 1 ] && shift
    ;;
  esac
  shift
done

[ -t 1 ] && tty=true || tty=false
# The row the cursor is on, numbered from 1 as the rows that open are;
# none in --once.
cursor=1
[ "$once" = true ] && cursor=0
# The first line of the frame drawn, moved by scrolling.
top=0

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

# The frame's lines, as words.jq makes them, in $pane/body, and of every
# repository, the rows that open, in their numbers' order, in $pane/rows.
body() {
  if [ -n "$repo" ]; then heading=$repo; else heading="tailrace · every repository on this host"; fi
  if [ -n "$repo" ] && ! printf '%s\n' "$repo" | grep -qx '[^/ ][^/]*/[^/ ][^/]*'; then
    echo "--repo names a repository as <owner>/<name>, not $repo" >"$pane/said"
  elif find_or_start >"$pane/said" && document "${repo:+/$repo}/status.json" >"$pane/said"; then
    if [ -n "$repo" ]; then
      jqw --arg repo "$repo" 'include "words"; one($repo)' "$pane/doc" >"$pane/body"
    else
      jqw --argjson cursor "$cursor" 'include "words"; every($cursor)' "$pane/doc" >"$pane/body"
      jqw -r 'include "words"; openable' "$pane/doc" >"$pane/rows"
    fi
    return
  fi
  [ -z "$repo" ] && : >"$pane/rows"
  jq -R -s --arg heading "$heading" '[{text: $heading}, {text: ""}] + (rtrimstr("\n") | split("\n") | map({text: .}))' \
    "$pane/said" >"$pane/body"
}

# The frame in $1 lines at most, its footer's two among them, or in as
# many as it takes when $1 is empty; the top it was drawn from in
# $pane/top.
frame() {
  body
  height=null
  [ -n "${1:-}" ] && height=$1
  focus=null
  [ -z "$repo" ] && [ "$cursor" -gt 0 ] && focus=$cursor
  jqw -c --argjson height "$height" --argjson top "$top" --argjson focus "$focus" \
    'include "words"; fit($height; $top; $focus)' "$pane/body" >"$pane/fit"
  jq -r '.top' "$pane/fit" >"$pane/top"
  jq -r '.lines[].text' "$pane/fit"
  echo
  if [ -n "$repo" ]; then
    echo "b back · r ask again · q close"
  else
    echo "↑↓ choose · enter open · r ask again · q close"
  fi
}

if [ "$once" = true ]; then
  frame
  exit 0
fi

# The terminal's height, read again at each draw since a pane is resized:
# from the terminal itself, else terminfo, else LINES, else nothing.
lines_tall() {
  h=
  if [ "$tty" = true ]; then
    h=$(stty size </dev/tty 2>/dev/null | cut -d' ' -f1)
    [ -n "$h" ] && [ "$h" -gt 0 ] 2>/dev/null || h=$(tput lines 2>/dev/null)
  fi
  [ -n "$h" ] && [ "$h" -gt 0 ] 2>/dev/null || h=${LINES:-}
  [ -n "$h" ] && [ "$h" -gt 0 ] 2>/dev/null || h=
  echo "$h"
}

# One frame drawn: over the last on a terminal, each line cleared to its
# end and the frame fitted to the height, so a redraw scrolls nothing;
# off a terminal, followed by a form feed.
draw() {
  h=$(lines_tall)
  page=$((${h:-12} - 4))
  [ "$page" -gt 0 ] || page=1
  if [ "$tty" = true ]; then
    esc=$(printf '\033')
    out=$(frame "$h" | sed "s/\$/${esc}[K/")
    printf '\033[H%s\033[J' "$out"
  else
    frame "$h"
    printf '\f\n'
  fi
  top=$(cat "$pane/top" 2>/dev/null || echo 0)
  clamp
}

# The cursor kept on a row of those every repository's last frame drew.
clamp() {
  [ -f "$pane/rows" ] || return 0
  rows=$(wc -l <"$pane/rows" | tr -d ' ')
  [ "$cursor" -le "$rows" ] || cursor=$rows
  [ "$cursor" -ge 1 ] || cursor=1
}

# The row numbered $1 opened, where there is one.
open_row() {
  named=$(sed -n "${1}p" "$pane/rows" 2>/dev/null)
  [ -n "$named" ] || return 1
  repo=$named
  top=0
}

# Five seconds with no key, then the frame is asked again (132).
wait_keys() {
  stty -icanon -echo min 0 time 50
}

if [ -t 0 ]; then
  saved=$(stty -g)
  trap 'stty "$saved" 2>/dev/null; printf "\033[?1049l\033[?7h\033[?25h"; rm -rf "$pane"' EXIT
  trap 'exit 0' INT TERM HUP
  wait_keys
  # The alternate screen, with no wrapping, so a line wider than the
  # pane is cut at its edge.
  printf '\033[?1049h\033[?7l\033[?25l\033[2J'
fi

esc=$(printf '\033')
nl='
'
cr=$(printf '\r')
del=$(printf '\177')
bs=$(printf '\010')
draw
while :; do
  # The dot keeps a newline, Enter off a terminal, from being stripped.
  key=$(
    dd bs=1 count=1 2>/dev/null
    echo .
  )
  key=${key%.}
  if [ -z "$key" ]; then
    # Five seconds without a key on a terminal; the end of input otherwise.
    [ -t 0 ] || exit 0
    draw
    continue
  fi
  case $key in
  q) exit 0 ;;
  "$esc")
    # Escape alone closes; an arrow's bytes follow at once. Off a
    # terminal, a key's two bytes after its escape are read as one.
    if [ -t 0 ]; then
      stty time 1
      more=$(dd bs=64 count=1 2>/dev/null)
      wait_keys
    else
      more=$(dd bs=1 count=2 2>/dev/null)
      case $more in "[5" | "[6") more=$more$(dd bs=1 count=1 2>/dev/null) ;; esac
    fi
    [ -z "$more" ] && exit 0
    case $more in
    *5~*) top=$((top - page)) ;;
    *6~*) top=$((top + page)) ;;
    *A*) if [ -z "$repo" ]; then cursor=$((cursor - 1)); else top=$((top - 1)); fi ;;
    *B*) if [ -z "$repo" ]; then cursor=$((cursor + 1)); else top=$((top + 1)); fi ;;
    esac
    [ "$top" -ge 0 ] || top=0
    ;;
  k) if [ -z "$repo" ]; then cursor=$((cursor - 1)); elif [ "$top" -gt 0 ]; then top=$((top - 1)); fi ;;
  j) if [ -z "$repo" ]; then cursor=$((cursor + 1)); else top=$((top + 1)); fi ;;
  "$nl" | "$cr") [ -z "$repo" ] && open_row "$cursor" ;;
  [1-9])
    if [ -z "$repo" ]; then
      open_row "$key" && cursor=$key
    fi
    ;;
  b | "$del" | "$bs")
    if [ -n "$repo" ]; then
      repo=
      top=0
    fi
    ;;
  esac
  clamp
  draw
done

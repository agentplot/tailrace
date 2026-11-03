#!/usr/bin/env bash
# PreToolUse on Bash: in a repository whose HEAD holds
# .config/tailrace.toml, refuse an ecosystem's own publish command, naming
# `tailrace publish` for a landed revision instead. The plugin is a surface
# kept outside the core (section 9), so it names the commands it refuses
# and nothing of the core does (core.1). It answers nothing the core does
# not (56, core.31): it reads only whether HEAD holds the file, never its
# contents, and the refusal names the command to run and nothing derived.
# It never runs `tailrace`, so it refuses on a host that has none.
# Anything it cannot read is let through.
#
# bash 3.2. Nothing of the command is evaluated: it is split on
# whitespace and shell operators and read as words.

set -u
command -v jq >/dev/null 2>&1 || exit 0

input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // empty' 2>/dev/null)
dir=$(printf '%s' "$input" | jq -r '.cwd // empty' 2>/dev/null)
[ -n "$cmd" ] || exit 0
[ -n "$dir" ] || dir=$PWD

# One pattern match on every command naming no program of the table, and
# out.
printf '%s' "$cmd" | grep -Eq '(^|[^[:alnum:]_-])(npm|pnpm|yarn|bun|cargo|mvnw?|gradlew?|uv|poetry|hatch|flit|twine|dotnet|nuget|docker|podman)([^[:alnum:]_-]|$)' || exit 0

# Whether word $2 is program $1's publish verb: one row per ecosystem that
# has a publish command of its own. Go has none: a module is read from its
# tag.
is_publish() {
  case $1 in
  npm | pnpm | yarn | bun) [ "$2" = publish ] ;;
  cargo) [ "$2" = publish ] ;;
  mvn | mvnw)
    case $2 in deploy | deploy:*) return 0 ;; esac
    return 1
    ;;
  gradle | gradlew) [ "$2" = publish ] ;;
  uv | poetry | hatch | flit) [ "$2" = publish ] ;;
  twine) [ "$2" = upload ] ;;
  dotnet | nuget) [ "$2" = push ] ;;
  docker | podman) [ "$2" = push ] ;;
  *) return 1 ;;
  esac
}

deny() { # the refused command, as its program and verb
  local reason
  reason="This repository publishes through tailrace alone: \`$1\` would put bytes in a registry that no landing's verdicts named and no gate accepted. Land the work first, then run \`tailrace publish <revision>\` for the landed revision; tailrace derives the version, packs and offers every component's artifacts and attaches their claims."
  jq -n --arg r "$reason" '{hookSpecificOutput: {hookEventName: "PreToolUse", permissionDecision: "deny", permissionDecisionReason: $r}}'
  exit 0
}

unquote() {
  local w=$1
  w=${w#\"}
  w=${w%\"}
  w=${w#\'}
  w=${w%\'}
  printf '%s' "$w"
}

segments=$(printf '%s\n' "$cmd" | awk '{ gsub(/&&|\|\||[;|]/, "\n"); print }')

while IFS= read -r seg; do
  # shellcheck disable=SC2206 # split into words on purpose, never evaluated
  read -r -a w <<<"$seg"
  n=${#w[@]}
  i=0
  # Leading VAR=value assignments.
  while [ $i -lt "$n" ] && [[ ${w[$i]} == [A-Za-z_]*=* ]]; do i=$((i + 1)); done
  [ $i -lt "$n" ] || continue
  prog=$(unquote "${w[$i]}")
  i=$((i + 1))

  if [ "$prog" = cd ]; then
    [ $i -lt "$n" ] || continue
    to=$(unquote "${w[$i]}")
    case $to in /*) dir=$to ;; ~*) dir=$HOME${to#\~} ;; *) dir=$dir/$to ;; esac
    continue
  fi
  prog=${prog##*/}

  # Any later word, not the first: Maven and Gradle take several goals in
  # one call, and npm and yarn take options and nesting before the verb.
  for a in "${w[@]:i}"; do
    a=$(unquote "$a")
    is_publish "$prog" "$a" || continue
    git -C "$dir" cat-file -e HEAD:.config/tailrace.toml 2>/dev/null || continue
    deny "$prog $a"
  done
done <<EOF
$segments
EOF

exit 0

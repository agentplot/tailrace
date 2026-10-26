#!/usr/bin/env bash
# The guard on scratch repositories: in a repository whose HEAD holds
# .config/tailrace.toml it refuses every publish command of its table,
# naming `tailrace publish` and nothing it derived, whether or not the host
# has `tailrace`; everything else, and everywhere else, it lets through
# with no output. git runs isolated from the user's config and
# credentials, so nothing reaches a network or asks anyone anything.

set -u
here=$(cd "$(dirname "$0")" && pwd)
guard=$here/../scripts/guard.sh

# Under sluice's sandbox only SLUICE_TMP is writable beside the checkout.
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/tailrace-plugin-hooks.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
scratch=$(cd "$scratch" && pwd -P)

export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_TERMINAL_PROMPT=0
export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@localhost
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@localhost
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_ASKPASS SSH_ASKPASS

package() { # dir
  mkdir -p "$1"
  printf '{ "name": "@acme/ui", "version": "1.4.0" }\n' >"$1/package.json"
}

# A tailrace repository: HEAD holds the policy file.
repo=$scratch/repo
git init -q -b main "$repo"
mkdir -p "$repo/.config"
printf '[store]\n' >"$repo/.config/tailrace.toml"
package "$repo"
package "$repo/packages/ui"
git -C "$repo" add -A
git -C "$repo" commit -qm policy

# A plain repository.
plain=$scratch/plain
git init -q -b main "$plain"
package "$plain"
git -C "$plain" add -A
git -C "$plain" commit -qm root

# The policy file in the working tree only, never committed.
uncommitted=$scratch/uncommitted
git init -q -b main "$uncommitted"
package "$uncommitted"
git -C "$uncommitted" add -A
git -C "$uncommitted" commit -qm root
mkdir -p "$uncommitted/.config"
printf '[store]\n' >"$uncommitted/.config/tailrace.toml"

# No repository at all.
nowhere=$scratch/nowhere
package "$nowhere"

# The tools the guard runs, and no `tailrace`; and beside them a stand-in
# `tailrace` that fails, and says so, if anything calls it.
tools=$scratch/tools
mkdir -p "$tools"
for t in bash jq git grep awk cat; do
  ln -s "$(command -v "$t")" "$tools/$t"
done
stand=$scratch/stand
mkdir -p "$stand"
cat >"$stand/tailrace" <<EOF
#!/bin/sh
touch "$scratch/tailrace-called"
exit 1
EOF
chmod +x "$stand/tailrace"

failures=0

run_guard() { # dir command
  jq -n --arg c "$2" --arg d "$1" '{tool_name: "Bash", tool_input: {command: $c}, cwd: $d}' | "$guard"
}

# A deny names `tailrace publish` and neither the package's version nor
# its name.
denied() { # output
  printf '%s' "$1" | jq -e '.hookSpecificOutput.hookEventName == "PreToolUse" and .hookSpecificOutput.permissionDecision == "deny"' >/dev/null 2>&1 &&
    printf '%s' "$1" | jq -r '.hookSpecificOutput.permissionDecisionReason' | grep -q 'tailrace publish' &&
    ! printf '%s' "$1" | grep -Eq '1\.4\.0|acme'
}

denies() { # dir command [where]
  local out
  out=$(run_guard "$1" "$2")
  if denied "$out"; then
    echo "ok    denies: $2${3:+ ($3)}"
  else
    echo "FAIL  should deny: $2 (in $1) -> ${out:-nothing}"
    failures=$((failures + 1))
  fi
}

allows() { # dir command [where]
  local out
  out=$(run_guard "$1" "$2")
  if [ -z "$out" ]; then
    echo "ok    allows: $2${3:+ ($3)}"
  else
    echo "FAIL  should allow: $2 (in $1) -> $out"
    failures=$((failures + 1))
  fi
}

# Every row of the guard's table.
denies "$repo" 'npm publish'
denies "$repo" 'npm run publish'
denies "$repo" 'pnpm publish'
denies "$repo" 'yarn publish'
denies "$repo" 'yarn npm publish'
denies "$repo" 'bun publish'
denies "$repo" 'cargo publish -p family-crate'
denies "$repo" 'mvn clean deploy'
denies "$repo" 'mvn deploy:deploy-file -Dfile=x.jar'
denies "$repo" './mvnw deploy'
denies "$repo" 'gradle publish'
denies "$repo" './gradlew publish'
denies "$repo" 'uv publish'
denies "$repo" 'poetry publish'
denies "$repo" 'hatch publish'
denies "$repo" 'flit publish'
denies "$repo" 'twine upload dist/*'
denies "$repo" 'dotnet nuget push x.nupkg'
denies "$repo" 'nuget push x.nupkg'
denies "$repo" 'docker push acme/api:1'
denies "$repo" 'podman push acme/api:1'

# Inside a longer command, and from below the root.
denies "$repo" 'cd packages/ui && NPM_CONFIG_PROVENANCE=true npm --workspace ui publish --dry-run'
denies "$repo" 'npm test; /usr/local/bin/npm "publish"'
denies "$repo/packages/ui" 'npm publish' 'below the root'

allows "$repo" 'npm install'
allows "$repo" 'npm pack'
allows "$repo" 'cargo build'
allows "$repo" 'cargo package --no-verify'
allows "$repo" 'docker build .'
allows "$repo" 'git push origin unit/x'
allows "$repo" 'tailrace publish HEAD --json'
allows "$repo" 'echo "npm publish"'
allows "$repo" 'ls -la'
allows "$plain" 'npm publish' 'HEAD holds no policy file'
allows "$uncommitted" 'npm publish' 'the policy file uncommitted'
allows "$nowhere" 'npm publish' 'no repository'

# The same refusal with no `tailrace` on the host as with one, which is
# never called.
none=$(PATH=$tools run_guard "$repo" 'npm publish')
with=$(PATH=$stand:$tools run_guard "$repo" 'npm publish')
if denied "$none" && [ "$none" = "$with" ] && [ ! -e "$scratch/tailrace-called" ]; then
  echo "ok    the refusal is the same with no tailrace on PATH, and never calls it"
else
  echo "FAIL  with no tailrace: ${none:-nothing}; with one: ${with:-nothing}; called: $([ -e "$scratch/tailrace-called" ] && echo yes || echo no)"
  failures=$((failures + 1))
fi

echo "$failures failed"
[ "$failures" -eq 0 ]

#!/bin/sh
# A merge made outside the devenv shell passes the commit hooks devenv.nix
# installs, as crew's take of one unit's work into another's place is made:
# in a scratch repository laid out as the kit is (kit/kit.git beside
# kit/main and the places), no worktree holding a .pre-commit-config.yaml
# since no devenv shell ran in any, the hooks install-hooks installs refuse
# a message convco refuses and pass a take's.
set -eu

fail() {
  echo "check-hooks: $*" >&2
  exit 1
}

command -v install-hooks >/dev/null || fail "the shell has no install-hooks"
git=$(command -v git)
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

mkdir "$scratch/home"
cat >"$scratch/home/.gitconfig" <<'CONFIG'
[user]
	name = check-hooks
	email = check-hooks@example.invalid
[init]
	defaultBranch = main
CONFIG
HOME="$scratch/home"
GIT_CONFIG_NOSYSTEM=1
export HOME GIT_CONFIG_NOSYSTEM
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE

# Outside the shell: none of its tools on PATH, none of its environment.
outside() {
  (cd "$1" && shift && env -i HOME="$HOME" GIT_CONFIG_NOSYSTEM=1 PATH=/usr/bin:/bin "$git" "$@")
}

kit="$scratch/kit"
"$git" init -q --bare "$kit/kit.git"
"$git" -C "$kit/kit.git" worktree add -q --orphan -b main "$kit/main"
cp .versionrc "$kit/main/.versionrc"
"$git" -C "$kit/main" add .versionrc
"$git" -C "$kit/main" commit -q -m "chore: the kit"
"$git" -C "$kit/kit.git" worktree add -q -b unit/a "$kit/places/a" main
"$git" -C "$kit/kit.git" worktree add -q -b unit/b "$kit/places/b" main
echo a >"$kit/places/a/a"
"$git" -C "$kit/places/a" add a
"$git" -C "$kit/places/a" commit -q -m "feat(a): a"
(cd "$kit/main" && install-hooks)

if outside "$kit/places/b" commit -q --allow-empty -m "not a conventional commit" >"$scratch/refused" 2>&1; then
  fail "a commit outside the shell passed the hooks with a message convco refuses"
fi
grep -q convco "$scratch/refused" ||
  fail "a commit outside the shell failed the hooks before convco: $(cat "$scratch/refused")"

outside "$kit/places/b" merge -q --no-ff --no-edit -m "chore(b): build on a as it stands" refs/heads/unit/a >"$scratch/take" 2>&1 ||
  fail "a take outside the shell failed the hooks: $(cat "$scratch/take")"
echo "check-hooks: a take outside the shell passes the hooks, and convco refuses what it refuses"

#!/bin/sh
# Every Rust tool the gate runs is the build .prototools pins, on every
# host (rule core.26): rust-toolchain.toml names the same version, and
# rustc, cargo, clippy and rustfmt each report it. OpenSpec is the release
# devenv.nix pins, and the Claude Code skills the tree tracks are the ones
# that release writes (ADR 0030). A tool found in another toolchain, a
# runner's rustup default say, fails here rather than holding the tree to
# another version's rules.
set -eu

fail() {
  echo "check-toolchain: $*" >&2
  exit 1
}

pin=$(sed -n 's/^rust *= *"\(.*\)"$/\1/p' .prototools)
[ -n "$pin" ] || fail ".prototools pins no rust"
channel=$(sed -n 's/^channel *= *"\(.*\)"$/\1/p' rust-toolchain.toml)
[ "$channel" = "$pin" ] || fail "rust-toolchain.toml names ${channel:-no channel}, .prototools pins $pin"

rustc=$(rustc -vV)
[ "$(printf '%s\n' "$rustc" | sed -n 's/^release: //p')" = "$pin" ] ||
  fail "rustc is not $pin: $(rustc --version)"
[ "$(cargo -vV | sed -n 's/^release: //p')" = "$pin" ] ||
  fail "cargo is not $pin: $(cargo --version)"

# clippy and rustfmt carry the commit of the rustc they ship with.
commit=$(printf '%s\n' "$rustc" | sed -n 's/^commit-hash: \(.\{9\}\).*/\1/p')
same() {
  case $2 in
  *"($commit"*) ;;
  *) fail "$1 is not $pin's: $2" ;;
  esac
}
same clippy "$(cargo clippy --version)"
same rustfmt "$(rustfmt --version)"
echo "rustc, cargo, clippy and rustfmt are $pin"

# OpenSpec, and the skills under .claude/skills it writes, each naming the
# release that wrote it: a skill another release wrote is written again
# with the pinned one (devenv.nix).
openspec=$(sed -n 's/^ *openspecVersion *= *"\(.*\)";$/\1/p' devenv.nix)
[ -n "$openspec" ] || fail "devenv.nix pins no openspec"
[ "$(openspec --version)" = "$openspec" ] ||
  fail "openspec is not $openspec: $(openspec --version)"
skills=0
for skill in .claude/skills/openspec-*/SKILL.md; do
  [ -f "$skill" ] || break
  skills=$((skills + 1))
  grep -qx "  generatedBy: \"$openspec\"" "$skill" ||
    fail "$skill is not openspec $openspec's: write the skills again with it"
done
[ "$skills" -gt 0 ] || fail "no OpenSpec skill under .claude/skills"
echo "openspec and its $skills Claude Code skills are $openspec"

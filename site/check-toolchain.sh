#!/bin/sh
# The bun the site's tasks run is the build the Pages workflow publishes
# the site with, on every host (rule core.26): devenv.nix carries bun's
# release build and .github/workflows/pages.yml restates its version, so
# a bun found outside the shell's, or a pin moved in one file and not the
# other, fails here rather than building the site under another bun.
set -eu

fail() {
  echo "check-toolchain: $*" >&2
  exit 1
}

pin=$(sed -n 's/^ *bun-version: *\(.*\)$/\1/p' ../.github/workflows/pages.yml)
[ -n "$pin" ] || fail ".github/workflows/pages.yml pins no bun"
bun=$(bun --version)
[ "$bun" = "$pin" ] || fail "bun is $bun, .github/workflows/pages.yml pins $pin"
echo "bun is $pin"

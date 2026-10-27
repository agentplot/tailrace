#!/bin/sh
# rustc compiles through the compile server the host's sluice settings
# declare, their `[[caches]]` entry whose variables set RUSTC_WRAPPER, and
# through nothing of tailrace's: sluice hands a run the server's variables
# (sluice 0.43.0), and a session's own shell inherits them from the host,
# which exports them into every shell (clause 133; devenv.nix). It checks
# the shell it runs in: the gate's runs check a run's, and a session's
# `moon run` checks a session's.
#
#   the host declares a compile server
#       rustc reaches it: every variable the server declares is set as
#       declared, its wrapper is found and its socket, where it has one, is
#       there; a run holding a check the policy marks cold (`[tests] cold`)
#       is given none of them, and rustc runs unwrapped
#   the host declares none
#       rustc runs unwrapped
set -eu

fail() {
  echo "check-compile-server: $*" >&2
  exit 1
}

if [ -n "${SLUICE_CHECKOUT:-}" ]; then
  shell="a run sluice started"
else
  shell="a session's shell"
fi

# Where sluice reads the host's settings.
settings="${XDG_CONFIG_HOME:-$HOME/.config}/sluice/config.toml"
server=null
if [ -e "$settings" ]; then
  host=$(taplo get -f "$settings" -o json) ||
    fail "the host's sluice settings, $settings, do not read as TOML"
  server=$(printf '%s\n' "$host" |
    jq -c '[.caches // [] | .[] | select(.env.RUSTC_WRAPPER != null)] | first')
fi

if [ "$server" = null ]; then
  [ -z "${RUSTC_WRAPPER:-}" ] ||
    fail "$shell wraps rustc in $RUSTC_WRAPPER, and the host declares no compile server"
  echo "check-compile-server: rustc runs unwrapped in $shell, the host declaring no compile server"
  exit 0
fi

given="" missing=""
for name in $(printf '%s\n' "$server" | jq -r '.env | keys[]'); do
  want=$(printf '%s\n' "$server" | jq -r --arg n "$name" '.env[$n]')
  if have=$(printenv "$name"); then
    [ "$have" = "$want" ] ||
      fail "$shell sets $name to $have, and the host's compile server declares $want"
    given="$given $name"
  else
    missing="$missing $name"
  fi
done

# A run holding a cold check is given none of the host's caches; sluice
# says nothing of which check made it cold, so it is the policy's naming one.
if [ -z "$given" ] && [ -n "${SLUICE_CHECKOUT:-}" ]; then
  cold=$(taplo get -f .config/sluice.toml -o json | jq '.tests.cold // [] | length')
  [ "$cold" -gt 0 ] ||
    fail "$shell is given none of the host's compile server (${missing# }), and the policy marks no check cold"
  echo "check-compile-server: rustc runs unwrapped in a run holding a check the policy marks cold"
  exit 0
fi
[ -z "$missing" ] ||
  fail "$shell lacks$missing, which the host's compile server declares"

command -v "$RUSTC_WRAPPER" >/dev/null ||
  fail "$shell wraps rustc in $RUSTC_WRAPPER, which is not found"
# sluice expands `~/` in a socket, as in a directory.
socket=$(printf '%s\n' "$server" | jq -r '.socket // empty')
case $socket in
\~/*) socket="$HOME/${socket#\~/}" ;;
esac
[ -z "$socket" ] || [ -S "$socket" ] ||
  fail "the host's compile server has no socket at $socket"
echo "check-compile-server: rustc reaches the host's compile server in $shell"

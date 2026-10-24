#!/bin/sh
# rustc runs under sccache in a session's own shell, so the agents'
# worktrees on a host reuse each other's compiled crates, and unwrapped in a
# run sluice starts, which SLUICE_CHECKOUT in its environment tells
# (devenv.nix). It checks the shell it runs in: the gate's runs check a
# run's, and a session's `moon run` checks a session's.
set -eu

fail() {
  echo "check-sccache: $*" >&2
  exit 1
}

if [ -n "${SLUICE_CHECKOUT:-}" ]; then
  [ -z "${RUSTC_WRAPPER:-}" ] ||
    fail "a run sluice started wraps rustc in $RUSTC_WRAPPER"
  echo "check-sccache: rustc runs unwrapped in a run sluice started"
else
  [ -n "${RUSTC_WRAPPER:-}" ] ||
    fail "a session's shell wraps rustc in nothing"
  case $("$RUSTC_WRAPPER" --version 2>&1) in
  "sccache "*) ;;
  *) fail "a session's shell wraps rustc in $RUSTC_WRAPPER, not sccache" ;;
  esac
  echo "check-sccache: rustc runs under sccache in a session's shell"
fi

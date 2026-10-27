# shellcheck shell=sh
# What every script of the plugin shares: where it is and where its state
# is. Sourced, never run.

: "${HERDR_PLUGIN_ROOT:=$(cd "$(dirname "$0")/.." && pwd -P)}"
: "${HERDR_PLUGIN_STATE_DIR:=${XDG_STATE_HOME:-$HOME/.local/state}/herdr-agentplot.tailrace}"
root=$HERDR_PLUGIN_ROOT
state=$HERDR_PLUGIN_STATE_DIR
lib=$root/lib
mkdir -p "$state"

# jq with the plugin's words loaded.
jqw() {
  jq -L "$lib" "$@"
}

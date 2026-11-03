#!/bin/sh
# Open one of the plugin's panes: status.
exec "${HERDR_BIN_PATH:-herdr}" plugin pane open --plugin agentplot.tailrace --entrypoint "$1"

#!/bin/sh
# shellcheck disable=SC1091 # common.sh, sourced from the plugin's lib.
# The pane: every repository the host's serving process knows, and one
# repository whole. --once prints one frame and exits.
set -u
. "$(dirname "$0")/../lib/common.sh"

jqw -n -r 'include "words"; empty_frame'

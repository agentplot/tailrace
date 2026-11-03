#!/bin/sh
# herdr-plugin.toml as herdr will read it: every line parses, the top-level
# keys herdr requires are there and the id is agentplot.tailrace, every
# command names a script that exists, and every action that opens a pane
# names one declared here. $1 is the manifest (default: the plugin's own);
# the plugin's directory is its.
set -u
manifest=${1:-$(cd "$(dirname "$0")/.." && pwd -P)/herdr-plugin.toml}
dir=$(cd "$(dirname "$manifest")" && pwd -P)
awk -v dir="$dir" '
function fail(why) { printf "%s:%d: %s\n", FILENAME, NR, why; bad = 1 }
function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
# The strings of a one-line array, into arr; their count.
function strings(v, arr,   n) {
  n = 0
  v = trim(v)
  if (v !~ /^\[.*\]$/) return -1
  v = substr(v, 2, length(v) - 2)
  while ((v = trim(v)) != "") {
    if (!match(v, /^"([^"\\]|\\.)*"/)) return -1
    arr[++n] = substr(v, 2, RLENGTH - 2)
    v = trim(substr(v, RLENGTH + 1))
    if (substr(v, 1, 1) == ",") v = substr(v, 2)
    else if (v != "") return -1
  }
  return n
}
function exists(path) {
  return system("test -f \"" dir "/" path "\"") == 0
}
function unquote(v) { return substr(v, 2, length(v) - 2) }
BEGIN { table = ""; no = 0 }
{
  line = $0
  sub(/[ \t]+#.*$/, "", line)
  line = trim(line)
  if (line == "" || substr(line, 1, 1) == "#") next
  if (line ~ /^\[\[[a-z_]+\]\]$/) {
    table = substr(line, 3, length(line) - 4)
    if (table !~ /^(startup|events|actions|panes|link_handlers)$/) fail("a table herdr does not read: " line)
    next
  }
  if (!match(line, /^[a-z_]+[ \t]*=[ \t]*/)) { fail("not key = value: " $0); next }
  key = line; sub(/[ \t]*=.*/, "", key)
  val = trim(substr(line, RLENGTH + 1))
  if (val !~ /^("([^"\\]|\\.)*"|[0-9]+|true|false|\[.*\])$/) { fail("a value herdr cannot read: " $0); next }
  if (table == "") top[key] = val
  if (key == "id" && table == "panes") panes[unquote(val)] = 1
  if (key == "command" && table != "") {
    delete c
    n = strings(val, c)
    if (n < 1) { fail("a command that is not an array of strings: " $0); next }
    script = (c[1] == "sh" && n >= 2) ? c[2] : (c[1] ~ /^bin\// ? c[1] : "")
    if (script != "" && !exists(script)) fail("runs " script ", which is not in the plugin")
    # An action that opens a pane names it after bin/open.sh.
    if (table == "actions" && script == "bin/open.sh") {
      if (n < 3) fail("opens no pane: " $0)
      else { opens[++no] = c[3]; opens_line[no] = NR }
    }
  }
}
END {
  split("id name version min_herdr_version", need, " ")
  for (i in need) if (!(need[i] in top)) { printf "%s: no %s\n", FILENAME, need[i]; bad = 1 }
  if (("id" in top) && top["id"] != "\"agentplot.tailrace\"") { printf "%s: the id is %s, not \"agentplot.tailrace\"\n", FILENAME, top["id"]; bad = 1 }
  for (i = 1; i <= no; i++)
    if (!(opens[i] in panes)) { printf "%s:%d: an action opens a pane this plugin does not declare: \"%s\"\n", FILENAME, opens_line[i], opens[i]; bad = 1 }
  exit bad
}' "$manifest"

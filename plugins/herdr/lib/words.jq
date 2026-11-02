# The pane's words: how each document the serving process answers, and
# each answer in the one form (57), is rendered, as lines {text, row},
# row naming the row of every repository a line belongs to. It formats;
# it never counts, takes a head from a chain, compares versions or turns
# one answer into another (56, core.31).

# A text as the frame shows it: control characters, which would move the
# pane's cursor, taken out, and a tab a space.
def clean: tostring | gsub("\t"; " ") | gsub("[[:cntrl:]]"; "");

# A moment, in seconds since the Unix epoch, in UTC as the core names one.
def utc: if type == "number" then todate else clean end;

# A digest shown short: its algorithm and the first 12 of its hex; the
# whole digest is in the JSON.
def short: if type == "string" and test("^[a-z0-9]+:[0-9a-f]{13,}$")
  then split(":") | "\(.[0]):\(.[1][0:12])" else clean end;

# A revision shown short: its first 12 hex.
def rev: if type == "string" and test("^[0-9a-f]{13,}$") then .[0:12] else clean end;

def pad($n): . + ((" " * ($n - length)) // "");

def name_of: "\(.owner | clean)/\(.name | clean)";

# The seam and the profile that answered, as the core names them.
def source_words: "the \(.seam | clean) (\(.profile | clean))";

# A value of a refusal's why: text as it is, a digest short, none for
# nothing, a list joined, and an object's fields each named, a moment
# in UTC.
def plain: if type == "string" then short
  elif . == null then "none"
  elif type == "array" then (if length == 0 then "none" else map(plain) | join(", ") end)
  elif type == "object" then [to_entries[]
    | "\(.key | gsub("_"; " ")) \(if (.key == "at" or .key == "fetched") and (.value | type) == "number" then .value | utc else .value | plain end)"]
    | join(", ")
  else tostring end;

# 59: why the source refused, as the core's answer names it: its kind,
# then each of its fields; a refusal that names nothing beyond the
# source's words says only that it refused.
def why_words: if . == "said" then "it refused"
  elif type == "string" then gsub("_"; " ")
  elif type == "object" and length == 1 then to_entries[0] | "\(.key | gsub("_"; " ")): \(.value | plain)"
  else plain end;

# 58, 59: an answer that is not a value, as lines: $lead, then its own
# leading word, absent, refused or unreadable, and what it was, then the
# source's own words verbatim, a line of the frame each, under $indent.
def not_lines($lead; $indent):
  .answer as $kind | (.[$kind] // {}) as $not
  | (if $kind == "absent" then "absent — \($not.source | source_words) holds no \($not.what | clean)"
     elif $kind == "refused" then "refused — \($not.source | source_words) refused: \($not.why | why_words)"
     elif $kind == "unreadable" then "unreadable — \($not.source | source_words) could not read \($not.what | clean)"
     else "an answer of no kind tailrace gives: \(tojson | clean)" end) as $first
  | [{text: $lead + $first}]
    + [($not.said // null) | strings | rtrimstr("\n") | split("\n") | .[] | {text: "\($indent)  \(clean)"}];

# 98: the environments' four counts under their four words.
def count_words:
  "\(.converged) running what they should · \(.behind) behind · \(.waiting) waiting on a person · \(.unreadable) cannot be read";

def environments_lines($lead; $indent):
  if .answer == "ok" then [{text: $lead + (.value | count_words)}] else not_lines($lead; $indent) end;

# A row's head inline: its component, its channel, and the version it is
# at, empty, or the leading word of its own answer.
def head_inline:
  "\(.component | clean) \(.channel | clean) "
  + (.at | if .answer != "ok" then .answer
     elif .value.state == "at" then .value.version | clean
     elif .value.state == "empty" then "empty"
     else .value | tojson | clean end);

# 132: one repository's row, numbered $n, the cursor's mark where it is
# on it: its channels' heads, each head that is not a value said beneath
# it, and its environments' counts or the answer in their place.
def row_lines($n; $cursor; $width):
  .value as $r
  | "\(if $cursor == $n then "▸" else " " end) [\($n)] \($r.repository | name_of | pad($width))  " as $lead
  | (if $r.heads.answer == "ok" then
      $r.heads.value as $heads
      | [{text: $lead + (if ($heads | length) == 0 then "no channel declared" else $heads | map(head_inline) | join(" · ") end)}]
        + [$heads[] | . as $h | select(.at.answer != "ok") | .at | not_lines("      \($h.component | clean) \($h.channel | clean): "; "      ")[]]
    else $r.heads | not_lines($lead; "      ") end)
  + ($r.environments | environments_lines("      environments: "; "      "))
  | map(. + {row: $n});

# 132: every repository the host knows, one row each in the process's
# order; a row that is not a value is said where its number would be,
# and has none, since it names no repository to open.
def every($cursor):
  if .answer != "ok" then [{text: "tailrace · every repository on this host"}, {text: ""}] + not_lines(""; "")
  else .value as $v
  | ($v.repositories | length) as $count
  | ([$v.repositories[] | select(.answer == "ok") | .value.repository | name_of | length] | max // 0) as $width
  | [{text: "tailrace · \(if $count == 0 then "no repository" elif $count == 1 then "1 repository" else "\($count) repositories" end) · as of \($v.asked_at | utc)"}, {text: ""}]
    + if $count == 0 then
        [{text: "This host knows no repository yet. A repository appears once a tailrace"},
         {text: "command run in a checkout of it names it, as tailrace status --page does."}]
      else
        reduce $v.repositories[] as $r ({n: 0, lines: []};
          if $r.answer == "ok" then (.n + 1) as $n | .n = $n | .lines += ($r | row_lines($n; $cursor; $width))
          else .lines += ($r | not_lines("  "; "    ")) end)
        | .lines
      end
  end;

# The repositories every repository's rows open, in their numbers' order.
def openable: if .answer == "ok" then .value.repositories[] | select(.answer == "ok") | .value.repository | name_of else empty end;

# 77: how the landing read the branch's head: nothing said where it read
# itself; where it read the host's copy, when that was fetched and the
# remote's words.
def landing_lines:
  if type == "object" and has("kept") then
    [{text: "landing: as last fetched \(.kept.at | utc)"}] + [.kept.said | strings | rtrimstr("\n") | split("\n") | .[] | {text: "  \(clean)"}]
  else [] end;

# How a channel moves, as the command line says it.
def moves_words: if .by == "bound" then ", bound to \(.to | clean)"
  elif .by == "each_publish" and .prereleases == true then ", prereleases too" else "" end;

# A chain read as the host last read it says when, the source's words
# beneath.
def read_words: if type == "object" and has("kept") then ", as last read \(.kept.at | utc)" else "" end;
def read_lines: if type == "object" and has("kept") then [.kept.said | strings | rtrimstr("\n") | split("\n") | .[] | {text: "    \(clean)"}] else [] end;

# 33: one promotion of a chain: the version it moved the channel to and
# from, who asked, when, and its claim.
def promoted_line:
  .[0] as $claim | .[1]
  | {text: "  ← \(.to.version | clean)\(if .from == null then ", the first" else " from \(.from.version | clean)" end), asked by \(.asked_by | clean) at \(.at | utc), claim \($claim | short)"};

# 134: the notes of every version a channel's chain moved it to, newest
# first as its chain is shown, each by its version and index: as
# rendered, or why they cannot be.
def logged_lines:
  [(. // []) | reverse | .[]
   | "  notes of \(.version | clean) \(.index | short)" as $lead
   | (.notes | if .answer == "ok" then [{text: "\($lead):"}] + [.value | strings | rtrimstr("\n") | split("\n") | .[] | {text: "    \(clean)"}]
      else not_lines("\($lead): "; "  ") end)
   | .[]];

# 31, 101, 134: one channel as the policy declares it and its chain puts
# it: its version and index, or empty, or its own answer; then its chain
# newest first, and its versions' notes.
def channel_lines:
  "\(.channel.name | clean)\(if .channel.component == null then "" else " (\(.channel.component | clean))" end)" as $name
  | (.moves | moves_words) as $moves
  | (.notes | logged_lines) as $logged
  | (.at
  | if .answer != "ok" then not_lines("\($name) "; "")
    elif .value.state == "empty" then [{text: "\($name) empty\($moves)\(.value.read | read_words)"}] + (.value.read | read_lines)
    elif .value.state == "at" then
      .value as $at
      | [{text: "\($name) \($at.head.to.version | clean) \($at.head.to.index | short)\($moves)\($at.read | read_words)"}]
        + ($at.read | read_lines)
        + [($at.chain // []) | reverse | .[] | promoted_line]
    else [{text: "\($name) \(.value | tojson | clean)"}] end)
  + $logged;

# 61, 134: one version this host published, and its notes or why they
# cannot be rendered.
def published_lines:
  if .answer != "ok" then not_lines("published on this host: "; "")
  elif (.value.components | length) == 0 then [{text: "published on this host: none"}]
  else [{text: "published on this host:"}]
    + [.value.components[] | .component as $c | .versions[]
       | [{text: "  \($c | clean) \(.version | clean) (\(.tag | clean)) from \(.revision | rev) on \(.branch | clean): index \(.index | short) in \(.store | source_words)"}]
         + (.notes | if type == "object" and .answer == "ok" then [.value | strings | rtrimstr("\n") | split("\n") | .[] | {text: "    \(clean)"}]
            elif type == "object" then not_lines("    notes: "; "    ")
            else [] end)
       | .[]]
  end;

# 57, 132: one repository whole, as the process answers it at its path,
# what `tailrace status --json` prints in its checkout: the branch and
# the landed revision its policy was read at, each channel, the
# environments, and what this host published.
def one($repo):
  if .answer != "ok" then [{text: $repo}, {text: ""}] + not_lines(""; "")
  else .value as $o
  | (if $o.channels.answer == "ok" then
      [{text: "\($repo) · \($o.branch | clean) at \($o.channels.value.revision | rev), as of \($o.asked_at | utc)"}]
      + ($o.channels.value.landing | landing_lines)
      + [{text: ""}]
      + (if ($o.channels.value.channels | length) == 0 then [{text: "The policy declares no channel."}]
         else [$o.channels.value.channels[] | channel_lines[]] end)
    else
      [{text: "\($repo) · \($o.branch | clean), as of \($o.asked_at | utc)"}, {text: ""}]
      + ($o.channels | not_lines("channels: "; ""))
    end)
  + [{text: ""}] + ($o.environments | environments_lines("environments: "; ""))
  + [{text: ""}] + ($o.published | published_lines)
  end;

# The lines of a frame that fit $height, its footer's two kept out of
# them, from $top, moved so the rows of $focus, the cursor's, are drawn;
# every line where $height is null. The top drawn beside them.
def fit($height; $top; $focus):
  . as $lines | length as $n
  | if $height == null then {top: 0, lines: $lines}
    else ([$height - 2, 1] | max) as $room
    | ([([$top, $n - $room] | min), 0] | max) as $t
    | (if $focus == null then $t
       else [$lines | to_entries[] | select(.value.row == $focus) | .key] as $at
       | if ($at | length) == 0 then $t
         elif $at[0] < $t then $at[0]
         elif ($at | last) >= $t + $room then ([($at | last) - $room + 1, $at[0]] | min)
         else $t end
       end) as $top
    | {top: $top, lines: $lines[$top:$top + $room]}
    end;

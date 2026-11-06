//! The marker the push profile leaves on a machine after a switch (119;
//! design D4): one JSON object, `{ "index", "version", "closure", "at" }`,
//! in the state directory of the user the fleet deploys as,
//! `${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/<owner>/<name>/<component>/<environment>.json`,
//! so a machine that is an environment of two repositories keeps one
//! marker for each. It is written to a temporary file renamed into place,
//! so a reader never reads part of one, and `at` is the machine's own
//! clock at the write. What the machine runs and the marker are read in
//! one run, `readlink /run/current-system` and the file.
//!
//! The scripts here are what `clan ssh` hands the machine's shell, `sh
//! -c <script>`: every part of the path the repository, the policy or the
//! version gives is single-quoted, and the script holds no backslash
//! pair, so a login shell parses the command the same whatever it is.
//! clan's ssh answers no exit of the command it ran, so each script ends
//! by printing a line of its own, and a run that does not print it did
//! not finish.

use std::fmt;

use serde::{Deserialize, Serialize};
use timber_seams::{Answer, Digest, Environment, Not, RepositoryName, Source, Version};

/// The deploy user's state directory, as the machine's shell expands it.
const STATE: &str = "${XDG_STATE_HOME:-$HOME/.local/state}";

/// The directory below it that holds timber's markers.
const MARKERS: &str = "timber/markers";

/// The link to the closure a NixOS or nix-darwin machine runs.
pub(super) const CURRENT: &str = "/run/current-system";

/// The lines the read script frames its answer with.
const RUNS: &str = "timber-runs ";
const HELD: &str = "timber-marker";
const NONE: &str = "timber-no-marker";
const END: &str = "timber-end";

/// The line the mark script ends with once the marker is in place.
const MARKED: &str = "timber-marked";

/// What a marker names: the index applied, its version, the closure's
/// root switched to, and the machine's time at the write.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Marker {
    pub index: Digest,
    pub version: Version,
    pub closure: String,
    pub at: String,
}

impl Marker {
    /// The marker's JSON up to its `at`, written by the timber that
    /// applies; the machine writes the rest, its time and the close.
    pub fn head(index: &Digest, version: &Version, closure: &str) -> String {
        let json = |value: &str| serde_json::Value::from(value).to_string();
        format!(
            "{{\"index\":{},\"version\":{},\"closure\":{},",
            json(index.as_str()),
            json(version.as_str()),
            json(closure)
        )
    }
}

/// Where an environment's marker is on its machine: the repository's
/// owner and name, the component and the environment, each one part of a
/// path below the deploy user's state directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MarkerPath {
    parts: [String; 4],
    environment: String,
}

impl MarkerPath {
    /// The marker of `environment` in `repository`, or unreadable naming
    /// the first part that is no one part of a path: empty, `.`, `..`, or
    /// holding a `/` or a control character.
    pub fn of(
        repository: &RepositoryName,
        environment: &Environment,
        source: &Source,
    ) -> Answer<MarkerPath> {
        let component = environment.channel.component_name();
        let parts = [
            ("the repository's owner", repository.owner.as_str()),
            ("the repository's name", repository.name.as_str()),
            ("the component", component.as_str()),
            ("the environment", environment.name.as_str()),
        ];
        for (what, part) in parts {
            let wanting = part.is_empty()
                || part == "."
                || part == ".."
                || part.chars().any(|c| c == '/' || c.is_control());
            if wanting {
                return Err(Not::unreadable(
                    source,
                    format!(
                        "the marker of environment {} on its machine",
                        environment.name
                    ),
                    Some(format!(
                        "{what}, {part:?}, is no one part of a path, which the marker's path \
                         names it as"
                    )),
                ));
            }
        }
        Ok(MarkerPath {
            parts: parts.map(|(_, part)| part.to_string()),
            environment: format!("{}.json", environment.name),
        })
    }

    /// The marker's directory as the machine's shell reads it.
    fn directory(&self) -> String {
        let mut at = format!("\"{STATE}\"/{MARKERS}");
        for part in &self.parts[..3] {
            at.push('/');
            at.push_str(&quoted(part));
        }
        at
    }

    /// What the machine runs and the marker, read in one run: the closure
    /// `/run/current-system` links to, then the marker's bytes or that
    /// none is held, then the end line.
    pub fn read_script(&self) -> String {
        let file = format!("{}/{}", self.directory(), quoted(&self.environment));
        format!(
            "m={file}\n\
             r=$(readlink {CURRENT}) || {{ ls -ld {CURRENT} >&2; exit 1; }}\n\
             printf '{RUNS}%s\\n' \"$r\"\n\
             if [ -e \"$m\" ]; then\n\
             printf '{HELD}\\n'\n\
             cat \"$m\" || exit 1\n\
             printf '\\n'\n\
             else\n\
             printf '{NONE}\\n'\n\
             fi\n\
             printf '{END}\\n'\n"
        )
    }

    /// The marker written: `head` and the machine's time at the write,
    /// into a temporary file beside the marker, renamed over it, then the
    /// mark's end line.
    pub fn mark_script(&self, head: &str) -> String {
        let (directory, file) = (self.directory(), quoted(&self.environment));
        format!(
            "set -e\n\
             d={directory}\n\
             mkdir -p \"$d\"\n\
             t=\"$d\"/.{file}.$$\n\
             printf '%s\"at\":\"%s\"}}\\n' {head} \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" > \"$t\"\n\
             mv -f \"$t\" \"$d\"/{file}\n\
             printf '{MARKED}\\n'\n",
            head = quoted(head),
        )
    }
}

impl fmt::Display for MarkerPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{STATE}/{MARKERS}")?;
        for part in &self.parts[..3] {
            write!(f, "/{part}")?;
        }
        write!(f, "/{}", self.environment)
    }
}

/// `text` single-quoted for sh, a `'` in it closed, escaped in double
/// quotes and reopened.
fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\"'\"'"))
}

/// What a machine answered the read script: the closure it runs, and the
/// marker's bytes where one is held.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Read {
    pub runs: String,
    pub marker: Option<Vec<u8>>,
}

impl Read {
    /// The read script's answer in `printed`, or `None` where it did not
    /// print one to its end, whatever else surrounds it.
    pub fn parse(printed: &[u8]) -> Option<Read> {
        let text = String::from_utf8_lossy(printed).replace("\r\n", "\n");
        let start = text.find(RUNS)?;
        let (runs, rest) = text[start + RUNS.len()..].split_once('\n')?;
        let body = rest
            .trim_end_matches('\n')
            .strip_suffix(END)?
            .strip_suffix('\n')?;
        let marker = if body == NONE {
            None
        } else {
            let held = body.strip_prefix(HELD)?.strip_prefix('\n')?;
            Some(held.trim_end_matches('\n').as_bytes().to_vec())
        };
        Some(Read {
            runs: runs.to_string(),
            marker,
        })
    }
}

/// Whether the mark script ran to its end, by what it printed.
pub(super) fn marked(printed: &[u8]) -> bool {
    String::from_utf8_lossy(printed)
        .lines()
        .any(|line| line.trim_end_matches('\r') == MARKED)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde_json::json;
    use tempfile::TempDir;
    use timber_seams::{ChannelId, ChannelName, ComponentName, EnvironmentName, Seam};

    use super::*;

    const ROOT: &str = "/nix/store/0c7yzjm6jy3dxfyb8zd6x0fv1r6l1sbm-nixos-system-srv-25.05";

    fn source() -> Source {
        Source::new(Seam::Target, "clan")
    }

    fn repository() -> RepositoryName {
        RepositoryName {
            owner: "afterthought".into(),
            name: "swancloud".into(),
        }
    }

    fn environment(name: &str) -> Environment {
        Environment {
            name: EnvironmentName::new(name),
            channel: ChannelId::new(
                Some(ComponentName::new("swancloud-srv")),
                ChannelName::new("stable"),
            ),
            waits: false,
            settings: json!({}),
        }
    }

    fn path() -> MarkerPath {
        MarkerPath::of(&repository(), &environment("srv"), &source()).unwrap()
    }

    /// `script` run by this host's sh with `XDG_STATE_HOME` at `state`.
    fn sh(script: &str, state: &std::path::Path) -> std::process::Output {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .env("XDG_STATE_HOME", state)
            .output()
            .unwrap()
    }

    #[test]
    fn the_path_names_the_repository_the_component_and_the_environment() {
        assert_eq!(
            path().to_string(),
            "${XDG_STATE_HOME:-$HOME/.local/state}/timber/markers/afterthought/swancloud/\
             swancloud-srv/srv.json"
        );
    }

    #[test]
    fn a_part_that_is_no_one_part_of_a_path_is_unreadable_naming_it() {
        for name in ["..", "a/b", "line\nbreak"] {
            match MarkerPath::of(&repository(), &environment(name), &source()) {
                Err(Not::Unreadable(u)) => {
                    let said = u.said.unwrap();
                    assert!(said.starts_with("the environment, "), "{said}");
                }
                other => panic!("{name:?} answered {other:?}"),
            }
        }
    }

    #[test]
    fn the_mark_script_writes_the_markers_fields_and_the_machines_time() {
        let state = TempDir::new().unwrap();
        let index = Digest::of(b"index");
        let head = Marker::head(&index, &Version::new("0.4.0-rc.1+it's"), ROOT);
        let ran = sh(&path().mark_script(&head), state.path());
        assert!(
            ran.status.success(),
            "{}",
            String::from_utf8_lossy(&ran.stderr)
        );
        assert!(marked(&ran.stdout));
        let dir = state
            .path()
            .join("timber/markers/afterthought/swancloud/swancloud-srv");
        let files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(files, ["srv.json"], "a temporary file was left");
        let marker: Marker =
            serde_json::from_slice(&std::fs::read(dir.join("srv.json")).unwrap()).unwrap();
        assert_eq!(marker.index, index);
        assert_eq!(marker.version, Version::new("0.4.0-rc.1+it's"));
        assert_eq!(marker.closure, ROOT);
        assert_eq!(
            marker.at.len(),
            "2026-11-06T10:00:00Z".len(),
            "{}",
            marker.at
        );
        assert!(marker.at.ends_with('Z'), "{}", marker.at);
    }

    #[test]
    fn a_mark_the_machine_refuses_prints_no_end_line_and_says_why() {
        let state = TempDir::new().unwrap();
        std::fs::write(state.path().join("timber"), "a file, not a directory").unwrap();
        let ran = sh(&path().mark_script("{"), state.path());
        assert!(!marked(&ran.stdout));
        assert!(!ran.stderr.is_empty());
    }

    #[test]
    fn a_read_is_parsed_with_or_without_a_marker_and_with_words_around_it() {
        let held = format!("{RUNS}{ROOT}\n{HELD}\n{{\"a\":1}}\n\n{END}\n");
        assert_eq!(
            Read::parse(held.as_bytes()),
            Some(Read {
                runs: ROOT.into(),
                marker: Some(b"{\"a\":1}".to_vec()),
            })
        );
        let none = format!("a banner\r\n{RUNS}{ROOT}\r\n{NONE}\r\n{END}\r\n");
        assert_eq!(
            Read::parse(none.as_bytes()),
            Some(Read {
                runs: ROOT.into(),
                marker: None,
            })
        );
        assert_eq!(
            Read::parse(format!("{RUNS}{ROOT}\n{HELD}\n{{").as_bytes()),
            None
        );
        assert_eq!(Read::parse(b""), None);
    }

    #[test]
    fn the_read_script_answers_what_the_mark_script_wrote() {
        let state = TempDir::new().unwrap();
        let head = Marker::head(&Digest::of(b"index"), &Version::new("0.4.0"), ROOT);
        sh(&path().mark_script(&head), state.path());
        // A host with no /run/current-system answers in its own words.
        let script = path()
            .read_script()
            .replace(CURRENT, &state.path().join("current").display().to_string());
        std::os::unix::fs::symlink(ROOT, state.path().join("current")).unwrap();
        let ran = sh(&script, state.path());
        let read = Read::parse(&ran.stdout).unwrap();
        assert_eq!(read.runs, ROOT);
        let marker: Marker = serde_json::from_slice(&read.marker.unwrap()).unwrap();
        assert_eq!(marker.closure, ROOT);
        std::fs::remove_file(state.path().join("current")).unwrap();
        let ran = sh(&script, state.path());
        assert_eq!(Read::parse(&ran.stdout), None);
        assert!(
            String::from_utf8_lossy(&ran.stderr).contains("current"),
            "{}",
            String::from_utf8_lossy(&ran.stderr)
        );
    }
}

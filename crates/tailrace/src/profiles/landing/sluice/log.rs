//! A branch's log as sluice keeps it, `log.jsonl`, one event per line,
//! and the landings it records done: a `landing` line written before
//! sluice moves the branch and a `landed` line with the same `ids` and
//! `to` after (77, C.5; sluice 23, 24; design D5). Every other event, and
//! every other field, is sluice's own.

use std::fs;
use std::io::ErrorKind;

use serde::Deserialize;
use serde_json::Value;
use tailrace_seams::{Answer, Branch, Not};

use super::{Opened, Sluice};

/// The log's file, in a branch's records.
const LOG_FILE: &str = "log.jsonl";

/// A landing the log records done: the revision it moved the branch
/// from, and the one it moved it to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Landed {
    pub ids: Vec<u64>,
    pub from: String,
    pub to: String,
}

/// A branch's log, and where it was read, in the landing's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Log {
    pub at: String,
    pub bytes: Vec<u8>,
}

#[derive(Deserialize)]
struct Landing {
    ids: Vec<u64>,
    from: String,
    to: String,
}

#[derive(Deserialize)]
struct Done {
    ids: Vec<u64>,
    to: String,
}

/// The landings `log` records done, in the order begun; or why a line is
/// not one of sluice's.
pub(super) fn landings(log: &[u8]) -> Result<Vec<Landed>, String> {
    let text = std::str::from_utf8(log).map_err(|e| format!("the log is not UTF-8: {e}"))?;
    let mut begun = Vec::new();
    let mut done = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let malformed = |e: serde_json::Error| format!("line {} of the log: {e}", n + 1);
        let event: Value = serde_json::from_str(line).map_err(malformed)?;
        match event["event"].as_str() {
            Some("landing") => {
                let l: Landing = serde_json::from_value(event).map_err(malformed)?;
                begun.push(Landed {
                    ids: l.ids,
                    from: l.from,
                    to: l.to,
                });
            }
            Some("landed") => {
                let d: Done = serde_json::from_value(event).map_err(malformed)?;
                done.push((d.ids, d.to));
            }
            _ => {}
        }
    }
    Ok(begun
        .into_iter()
        .filter(|l| done.iter().any(|(ids, to)| *ids == l.ids && *to == l.to))
        .collect())
}

impl Sluice {
    /// `branch`'s log: `log.jsonl` of the commit `refs/sluice/state/<branch>`
    /// names on the remote, as `opened` lists it, where it lists it;
    /// otherwise, where the remote answered, the host's own, under the
    /// common directory, empty where sluice keeps no log of the branch,
    /// which has landed nothing (sluice C.6). From the copy, a log it
    /// does not hold is the remote's failure, since the copy cannot say
    /// whether the remote held one (design D4).
    pub(super) fn log(&self, opened: &Opened, branch: &Branch) -> Answer<Log> {
        let state = format!("refs/sluice/state/{branch}");
        if let (Some(remote), Some(commit)) = (&self.remote, opened.listed.get(&state)) {
            let bytes = self.file_at(commit, LOG_FILE)?.unwrap_or_default();
            return Ok(Log {
                at: format!("{remote} {state}:{LOG_FILE}"),
                bytes,
            });
        }
        if let Some(failed) = opened.failed() {
            return Err(failed);
        }
        let path = self
            .common()?
            .join("sluice")
            .join(branch.as_str())
            .join(LOG_FILE);
        let at = path.display().to_string();
        match fs::read(&path) {
            Ok(bytes) => Ok(Log { at, bytes }),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Log { at, bytes: vec![] }),
            Err(e) => Err(Not::unreadable(self.source(), at, Some(e.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn landed(ids: &[u64], from: &str, to: &str) -> Landed {
        Landed {
            ids: ids.to_vec(),
            from: from.into(),
            to: to.into(),
        }
    }

    #[test]
    fn a_landing_done() {
        let log = br#"{"schema":"sluice/log@1","at":"2026-10-19T12:00:00Z","event":"landing","ids":[12],"from":"a0","to":"b1"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:01Z","event":"landed","ids":[12],"to":"b1"}
"#;
        assert_eq!(landings(log), Ok(vec![landed(&[12], "a0", "b1")]));
    }

    #[test]
    fn a_landing_begun_and_not_done() {
        let log = br#"{"schema":"sluice/log@1","at":"2026-10-19T12:00:00Z","event":"landing","ids":[12],"from":"a0","to":"b1"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:01Z","event":"refused","ids":[12],"expected":"a0","found":"c2"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:02Z","event":"landed","ids":[13],"to":"b1"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:03Z","event":"landed","ids":[12],"to":"c2"}
"#;
        assert_eq!(landings(log), Ok(vec![]));
    }

    #[test]
    fn two_landings() {
        let log = br#"{"schema":"sluice/log@1","at":"2026-10-19T12:00:00Z","event":"landing","ids":[12,13],"from":"a0","to":"b1"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:01Z","event":"landed","ids":[12,13],"to":"b1"}
{"schema":"sluice/log@1","at":"2026-10-19T12:01:00Z","event":"landing","ids":[14],"from":"b1","to":"d3"}

{"schema":"sluice/log@1","at":"2026-10-19T12:01:01Z","event":"landed","ids":[14],"to":"d3"}
"#;
        assert_eq!(
            landings(log),
            Ok(vec![
                landed(&[12, 13], "a0", "b1"),
                landed(&[14], "b1", "d3")
            ])
        );
    }

    #[test]
    fn lines_of_other_events_and_fields_are_ignored() {
        let log = br#"{"schema":"sluice/log@1","at":"2026-10-19T11:59:00Z","event":"submitted","id":12,"head":"f00"}
{"schema":"sluice/log@1","at":"2026-10-19T11:59:30Z","event":"ran","combination":"c0","task":"app:test","hash":"4863a9af","outcome":"pass","host":"box"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:00Z","event":"landing","ids":[12],"from":"a0","to":"b1","by":"a later field"}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:01Z","event":"landed","ids":[12],"to":"b1","took_ms":4}
{"schema":"sluice/log@1","at":"2026-10-19T12:00:02Z","event":"told","id":12,"notification":"landed"}
"#;
        assert_eq!(landings(log), Ok(vec![landed(&[12], "a0", "b1")]));
    }

    #[test]
    fn a_line_that_is_not_sluices_says_which() {
        let not_json = landings(b"{\"event\":\"landed\",\"ids\":[1],\"to\":\"b1\"}\nnot a line\n");
        assert!(
            not_json
                .as_ref()
                .is_err_and(|e| e.starts_with("line 2 of the log")),
            "{not_json:?}"
        );
        let no_to = landings(b"{\"event\":\"landing\",\"ids\":[1],\"from\":\"a0\"}\n");
        assert!(
            no_to
                .as_ref()
                .is_err_and(|e| e.starts_with("line 1 of the log")),
            "{no_to:?}"
        );
    }
}

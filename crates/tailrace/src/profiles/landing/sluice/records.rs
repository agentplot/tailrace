//! sluice's signed run records, as verdicts (78, 104, C.5; design D6): a
//! file in the tree of `refs/sluice/runs/<signer>` at
//! `<combination>/<system or any>/<16 hex>.json`, a DSSE envelope whose
//! payload is an in-toto statement of one run, a subject `<check>@<system>`
//! per result with the key as its sha256 digest, and a predicate holding
//! each result, the signer's claims of the run, and its combination.
//!
//! Each result of the combination that landed a revision is one verdict,
//! carrying the record's envelope whole. A result sluice marks replayed
//! is no one's observation: it is carried as every pass that ran on the
//! same key and system, under whichever signer's ref it was found, and
//! which of those counts is the policy's (9, 104; sluice 108).

use std::collections::BTreeMap;
use std::ffi::OsStr;

use serde::Deserialize;
use tailrace_seams::{
    Answer, BuildId, Check, Envelope, Not, Outcome, SignerId, Source, Verdict, IN_TOTO,
};

use super::{dsse, Sluice};
use crate::profiles::run::GIT_REPOSITORY_ENV;

/// The type every in-toto statement names itself by.
const STATEMENT: &str = "https://in-toto.io/Statement/v1";

/// The predicate type of a sluice run.
const RUN: &str = "https://agentplot.github.io/sluice/run/v1";

/// Where each signer's records are, on the remote.
pub(super) const RUNS: &str = "refs/sluice/runs/";

#[derive(Deserialize)]
struct Statement {
    #[serde(rename = "_type")]
    statement: String,
    subject: Vec<Subject>,
    #[serde(rename = "predicateType")]
    predicate_type: String,
    predicate: Run,
}

#[derive(Deserialize, PartialEq, Eq)]
struct Subject {
    name: String,
    digest: BTreeMap<String, String>,
}

/// What this profile reads of a run: its results, the system it ran on,
/// and its combination; every other field is sluice's own.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(super) struct Run {
    pub results: Vec<RunResult>,
    pub claims: Claims,
    pub combination: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(super) struct RunResult {
    pub check: String,
    pub key: String,
    pub outcome: Outcome,
    #[serde(default)]
    pub replayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(super) struct Claims {
    pub system: String,
}

/// A run record as read: its envelope whole, every signature found under
/// the signer whose ref holds it, and the run its statement holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Record {
    pub envelope: Envelope,
    pub run: Run,
}

/// A record and where it was found: the signer whose ref holds it and
/// its path in that ref's tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Held {
    pub signer: String,
    pub path: String,
    pub record: Record,
}

/// Where a record was found, in the landing's words.
pub(super) fn found(remote: &str, signer: &str, path: &str) -> String {
    format!("{remote} {RUNS}{signer}:{path}")
}

/// The record `bytes` hold, found under `signer`'s ref at `path` on
/// `remote`; unreadable naming the ref and the path, with the parser's
/// words, when they are not a DSSE envelope holding a run's statement.
/// sluice writes no such file, so one is a record changed by hand, which
/// is never answered as no verdict (58).
pub(super) fn record(
    source: &Source,
    remote: &str,
    signer: &str,
    path: &str,
    bytes: &[u8],
) -> Answer<Record> {
    let read = || -> Result<Record, String> {
        let envelope = dsse::read(bytes, &SignerId::new(signer))?;
        if envelope.payload_type != IN_TOTO {
            return Err(format!(
                "the envelope holds {}, not an in-toto statement",
                envelope.payload_type
            ));
        }
        let statement: Statement = serde_json::from_slice(&envelope.payload)
            .map_err(|e| format!("the envelope holds no run's statement: {e}"))?;
        if statement.statement != STATEMENT {
            return Err(format!(
                "the statement is {}, not {STATEMENT}",
                statement.statement
            ));
        }
        if statement.predicate_type != RUN {
            return Err(format!(
                "the statement's predicate is {}, not a run",
                statement.predicate_type
            ));
        }
        let run = statement.predicate;
        let subjects: Vec<Subject> = run
            .results
            .iter()
            .map(|r| Subject {
                name: format!("{}@{}", r.check, run.claims.system),
                digest: BTreeMap::from([("sha256".to_string(), r.key.clone())]),
            })
            .collect();
        if subjects != statement.subject {
            return Err("the statement's subjects are not its results".into());
        }
        Ok(Record { envelope, run })
    };
    read().map_err(|why| Not::unreadable(source, found(remote, signer, path), Some(why)))
}

/// The verdict `result` of the record `held` is.
fn verdict(remote: &str, held: &Held, result: &RunResult) -> Verdict {
    Verdict {
        check: Check::new(result.check.clone()),
        build: BuildId::new(result.key.clone()),
        outcome: result.outcome,
        signer: SignerId::new(held.signer.clone()),
        envelope: held.record.envelope.clone(),
        found: found(remote, &held.signer, &held.path),
    }
}

/// The verdicts of `combination`'s records among `under`, the records
/// under its path in every signer's ref: one per result of each whose
/// statement names it, a record whose statement names another left out.
/// A replayed result is carried as every pass recorded on its key and its
/// system by a result that was not replayed, among every record `all`
/// reads, which it reads only when a result was replayed; a replay with
/// no such pass gives no verdict, and its own envelope is never carried.
pub(super) fn verdicts(
    remote: &str,
    combination: &str,
    under: &[Held],
    all: impl FnOnce() -> Answer<Vec<Held>>,
) -> Answer<Vec<Verdict>> {
    let mut verdicts = Vec::new();
    let mut replayed = Vec::new();
    for held in under {
        let run = &held.record.run;
        if run.combination != combination {
            continue;
        }
        for result in &run.results {
            if result.replayed {
                replayed.push((result.key.as_str(), run.claims.system.as_str()));
            } else {
                verdicts.push(verdict(remote, held, result));
            }
        }
    }
    if replayed.is_empty() {
        return Ok(verdicts);
    }
    let all = all()?;
    for (key, system) in replayed {
        for held in &all {
            if held.record.run.claims.system != system {
                continue;
            }
            for result in &held.record.run.results {
                let ran_and_passed =
                    !result.replayed && result.key == key && result.outcome == Outcome::Pass;
                let carried = verdict(remote, held, result);
                if ran_and_passed && !verdicts.contains(&carried) {
                    verdicts.push(carried);
                }
            }
        }
    }
    Ok(verdicts)
}

// The landing's verdicts-of reads through it from the task that writes
// it.
#[allow(dead_code)]
impl Sluice {
    /// Every record in the tree of each signer's ref `runs` names, by the
    /// commit the remote's listing gave it, under `prefix` where one is
    /// given: each read once, by its blob, for the life of the handle
    /// (design D6).
    pub(super) fn held(
        &self,
        remote: &str,
        runs: &[(String, String)],
        prefix: Option<&str>,
    ) -> Answer<Vec<Held>> {
        let mut listed = Vec::new();
        for (signer, commit) in runs {
            let mut args = vec!["ls-tree", "-r", "-z", commit.as_str()];
            if let Some(prefix) = prefix {
                args.extend(["--", prefix]);
            }
            let ran = self.git(&args, &[0])?;
            for entry in String::from_utf8_lossy(&ran.stdout).split('\0') {
                let Some((meta, path)) = entry.split_once('\t') else {
                    continue;
                };
                let mut words = meta.split_whitespace();
                if let (Some(_), Some("blob"), Some(blob)) =
                    (words.next(), words.next(), words.next())
                {
                    listed.push((signer.clone(), path.to_string(), blob.to_string()));
                }
            }
        }
        let unread: Vec<&str> = {
            let read = self.records.borrow();
            listed
                .iter()
                .filter(|(signer, _, blob)| !read.contains_key(&(signer.clone(), blob.clone())))
                .map(|(_, _, blob)| blob.as_str())
                .collect()
        };
        let bytes = self.blobs(&unread)?;
        let mut held = Vec::new();
        for (signer, path, blob) in listed {
            let key = (signer.clone(), blob.clone());
            let cached = self.records.borrow().get(&key).cloned();
            let record = match cached {
                Some(record) => record,
                None => {
                    let file = bytes.get(&blob).map(Vec::as_slice).unwrap_or_default();
                    let record = record(self.source(), remote, &signer, &path, file)?;
                    self.records.borrow_mut().insert(key, record.clone());
                    record
                }
            };
            held.push(Held {
                signer,
                path,
                record,
            });
        }
        Ok(held)
    }

    /// The bytes of each of `blobs`, read in one `git cat-file --batch`.
    fn blobs(&self, blobs: &[&str]) -> Answer<BTreeMap<String, Vec<u8>>> {
        let mut read = BTreeMap::new();
        if blobs.is_empty() {
            return Ok(read);
        }
        let input: String = blobs.iter().map(|b| format!("{b}\n")).collect();
        let args = [
            OsStr::new("-C"),
            self.repository.as_os_str(),
            OsStr::new("cat-file"),
            OsStr::new("--batch"),
        ];
        let ran = self
            .git
            .fed(None, &args, GIT_REPOSITORY_ENV, input.as_bytes())?;
        if ran.code != Some(0) {
            return Err(self.git.failed(&args, &ran));
        }
        let mut rest = ran.stdout.as_slice();
        while let Some(end) = rest.iter().position(|b| *b == b'\n') {
            let header = String::from_utf8_lossy(&rest[..end]).to_string();
            rest = &rest[end + 1..];
            let words: Vec<&str> = header.split_whitespace().collect();
            let [oid, _, size] = words.as_slice() else {
                return Err(self
                    .git
                    .unreadable(&args, format!("git cat-file printed {header}")));
            };
            let size: usize = size.parse().map_err(|_| {
                self.git
                    .unreadable(&args, format!("git cat-file printed {header}"))
            })?;
            if rest.len() < size + 1 {
                return Err(self
                    .git
                    .unreadable(&args, format!("git cat-file ended within {oid}")));
            }
            read.insert(oid.to_string(), rest[..size].to_vec());
            rest = &rest[size + 1..];
        }
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use serde_json::{json, Value};
    use tailrace_seams::Seam;

    use super::*;

    const REMOTE: &str = "origin";

    fn source() -> Source {
        Source::new(Seam::Landing, "sluice")
    }

    /// A result as sluice records it.
    fn result(check: &str, key: &str, outcome: &str, replayed: bool) -> Value {
        let mut r = json!({ "check": check, "key": key, "outcome": outcome });
        if replayed {
            r["replayed"] = json!(true);
        }
        r
    }

    /// A run of `results` on `system` for `combination`, in sluice's
    /// statement, signed with a key named `keyid`.
    fn run(combination: &str, system: &str, results: &[Value], keyid: &str) -> Vec<u8> {
        let subject: Vec<Value> = results
            .iter()
            .map(|r| {
                json!({
                    "name": format!("{}@{system}", r["check"].as_str().unwrap()),
                    "digest": { "sha256": r["key"] },
                })
            })
            .collect();
        let statement = json!({
            "_type": STATEMENT,
            "subject": subject,
            "predicateType": RUN,
            "predicate": {
                "results": results,
                "claims": { "executor": "process", "confined": false, "system": system },
                "branch": "main",
                "combination": combination,
                "at": "2026-10-19T12:00:00Z",
            },
        });
        let payload = serde_json::to_vec(&statement).unwrap();
        serde_json::to_vec(&json!({
            "payload": STANDARD.encode(payload),
            "payloadType": IN_TOTO,
            "signatures": [{ "keyid": keyid, "sig": STANDARD.encode(format!("signed by {keyid}")) }],
        }))
        .unwrap()
    }

    fn held(signer: &str, path: &str, bytes: &[u8]) -> Held {
        Held {
            signer: signer.into(),
            path: path.into(),
            record: record(&source(), REMOTE, signer, path, bytes).unwrap(),
        }
    }

    fn none() -> Answer<Vec<Held>> {
        panic!("no result was replayed, and every record was read")
    }

    #[test]
    fn each_result_is_a_verdict_by_the_signer_whose_ref_holds_it() {
        let bytes = run(
            "c0ffee",
            "linux-x64",
            &[
                result("app:test", "4863a9af", "pass", false),
                result("app:lint", "6de6b709", "fail", false),
            ],
            "SHA256:b0x",
        );
        let box_ = held("box", "c0ffee/linux-x64/0123456789abcdef.json", &bytes);
        let read = verdicts(REMOTE, "c0ffee", std::slice::from_ref(&box_), none).unwrap();
        let named: Vec<_> = read
            .iter()
            .map(|v| {
                (
                    v.check.as_str(),
                    v.build.as_str(),
                    v.outcome,
                    v.signer.as_str(),
                )
            })
            .collect();
        assert_eq!(
            named,
            [
                ("app:test", "4863a9af", Outcome::Pass, "box"),
                ("app:lint", "6de6b709", Outcome::Fail, "box"),
            ]
        );
        for verdict in &read {
            assert_eq!(dsse::write(&verdict.envelope), bytes);
            assert_eq!(verdict.envelope.signatures[0].keyid, "SHA256:b0x");
            assert_eq!(
                verdict.found,
                "origin refs/sluice/runs/box:c0ffee/linux-x64/0123456789abcdef.json"
            );
        }
    }

    #[test]
    fn a_statement_naming_another_combination_is_left_out() {
        let other = run(
            "beef00",
            "linux-x64",
            &[result("app:test", "4863a9af", "pass", false)],
            "SHA256:b0x",
        );
        let moved = held("box", "c0ffee/linux-x64/0123456789abcdef.json", &other);
        assert_eq!(verdicts(REMOTE, "c0ffee", &[moved], none), Ok(vec![]));
    }

    #[test]
    fn a_replay_is_carried_as_the_passes_that_ran_on_its_key_and_system() {
        let replay = held(
            "box",
            "c0ffee/linux-x64/1111111111111111.json",
            &run(
                "c0ffee",
                "linux-x64",
                &[
                    result("app:test", "4863a9af", "pass", true),
                    result("app:lint", "6de6b709", "pass", false),
                ],
                "SHA256:b0x",
            ),
        );
        let ran = held(
            "box",
            "beef00/linux-x64/2222222222222222.json",
            &run(
                "beef00",
                "linux-x64",
                &[result("app:test", "4863a9af", "pass", false)],
                "SHA256:b0x",
            ),
        );
        let elsewhere = held(
            "forge",
            "beef00/macos-arm64/3333333333333333.json",
            &run(
                "beef00",
                "macos-arm64",
                &[result("app:test", "4863a9af", "pass", false)],
                "SHA256:f0rge",
            ),
        );
        let failed = held(
            "forge",
            "beef00/linux-x64/4444444444444444.json",
            &run(
                "beef00",
                "linux-x64",
                &[result("app:test", "4863a9af", "fail", false)],
                "SHA256:f0rge",
            ),
        );
        let every = vec![replay.clone(), ran.clone(), elsewhere, failed];
        let read = verdicts(REMOTE, "c0ffee", std::slice::from_ref(&replay), || {
            Ok(every)
        })
        .unwrap();
        assert_eq!(read.len(), 2, "{read:?}");
        assert_eq!(read[0].check, Check::new("app:lint"));
        let carried = &read[1];
        assert_eq!(
            (
                carried.check.as_str(),
                carried.build.as_str(),
                carried.signer.as_str()
            ),
            ("app:test", "4863a9af", "box")
        );
        assert_eq!(carried.envelope, ran.record.envelope);
        assert_eq!(
            carried.found,
            "origin refs/sluice/runs/box:beef00/linux-x64/2222222222222222.json"
        );
        assert!(
            read.iter()
                .all(|v| v.check.as_str() != "app:test" || v.envelope != replay.record.envelope),
            "the replay's own envelope was carried as a pass"
        );
    }

    #[test]
    fn a_pass_on_another_system_does_not_stand_in() {
        let replay = held(
            "box",
            "c0ffee/linux-x64/1111111111111111.json",
            &run(
                "c0ffee",
                "linux-x64",
                &[result("app:test", "4863a9af", "pass", true)],
                "SHA256:b0x",
            ),
        );
        let elsewhere = held(
            "box",
            "beef00/macos-arm64/3333333333333333.json",
            &run(
                "beef00",
                "macos-arm64",
                &[result("app:test", "4863a9af", "pass", false)],
                "SHA256:b0x",
            ),
        );
        let every = vec![replay.clone(), elsewhere];
        assert_eq!(
            verdicts(REMOTE, "c0ffee", &[replay], || Ok(every)),
            Ok(vec![])
        );
    }

    #[test]
    fn a_malformed_record_is_unreadable_naming_its_ref_and_path() {
        let path = "c0ffee/linux-x64/0123456789abcdef.json";
        let cases: [(&[u8], &str); 3] = [
            (b"changed by hand", "the record is not a DSSE envelope"),
            (
                br#"{"payload":"e30=","payloadType":"application/vnd.in-toto+json","signatures":[]}"#,
                "the envelope holds no run's statement",
            ),
            (
                br#"{"payload":"e30=","payloadType":"text/plain","signatures":[]}"#,
                "the envelope holds text/plain",
            ),
        ];
        for (bytes, why) in cases {
            match record(&source(), REMOTE, "box", path, bytes) {
                Err(Not::Unreadable(u)) => {
                    assert_eq!(u.source, source());
                    assert_eq!(u.what, format!("origin refs/sluice/runs/box:{path}"));
                    let said = u.said.unwrap();
                    assert!(said.starts_with(why), "{said}");
                }
                other => panic!("answered {other:?}"),
            }
        }
        let mut statement: Value = serde_json::from_slice(
            &dsse::read(
                &run(
                    "c0ffee",
                    "linux-x64",
                    &[result("app:test", "4863a9af", "pass", false)],
                    "k",
                ),
                &SignerId::new("box"),
            )
            .unwrap()
            .payload,
        )
        .unwrap();
        statement["subject"][0]["digest"]["sha256"] = json!("beef");
        let forged = serde_json::to_vec(&json!({
            "payload": STANDARD.encode(serde_json::to_vec(&statement).unwrap()),
            "payloadType": IN_TOTO,
            "signatures": [],
        }))
        .unwrap();
        assert!(matches!(
            record(&source(), REMOTE, "box", path, &forged),
            Err(Not::Unreadable(u)) if u.said.as_deref() == Some("the statement's subjects are not its results")
        ));
    }
}

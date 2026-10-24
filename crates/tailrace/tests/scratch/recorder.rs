//! A recorder of sluice's records as sluice v0.37.0 writes them, for a
//! landing a test must stage with a given build identity, which no real
//! sluice landing can be told (design D9): landing commits carrying
//! `Sluice-Tested`, the branch's `log.jsonl` and `version` in the commit
//! `refs/sluice/state/<branch>` names, or in the host's own files, and run
//! records signed by `ssh-keygen -Y sign -n sluice-run` over DSSE's
//! pre-authentication encoding, each signer's key its own and its key id
//! the key's `SHA256:` fingerprint, added to `refs/sluice/runs/<signer>`.

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;
use tailrace_seams::{Digest, IN_TOTO};

use super::{git, git_command, ssh_keygen, Repo, MAIN};

/// One result of a run, as the recorder records it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran {
    pub check: String,
    pub key: String,
    /// `pass` or `fail`, as sluice writes an outcome.
    pub outcome: String,
    pub replayed: bool,
}

impl Ran {
    pub fn new(check: &str, key: &str, outcome: &str) -> Ran {
        Ran {
            check: check.into(),
            key: key.into(),
            outcome: outcome.into(),
            replayed: false,
        }
    }

    pub fn replayed(check: &str, key: &str) -> Ran {
        Ran {
            replayed: true,
            ..Ran::new(check, key, "pass")
        }
    }
}

/// A landing the recorder made: its commits, oldest first, and the
/// combination whose verdict they name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub commits: Vec<String>,
    pub combination: String,
}

// sluice's statement and envelope, their fields in the order sluice
// writes them (sluice `signing/statement.rs`, `signing/dsse.rs`).

#[derive(Serialize)]
struct Statement<'a> {
    #[serde(rename = "_type")]
    statement: &'a str,
    subject: Vec<Subject>,
    #[serde(rename = "predicateType")]
    predicate_type: &'a str,
    predicate: Run<'a>,
}

#[derive(Serialize)]
struct Subject {
    name: String,
    digest: std::collections::BTreeMap<String, String>,
}

#[derive(Serialize)]
struct Run<'a> {
    results: Vec<RunResult<'a>>,
    claims: Claims<'a>,
    branch: &'a str,
    combination: &'a str,
    at: &'a str,
}

#[derive(Serialize)]
struct RunResult<'a> {
    check: &'a str,
    key: &'a str,
    outcome: &'a str,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    replayed: bool,
}

#[derive(Serialize)]
struct Claims<'a> {
    executor: &'a str,
    confined: bool,
    system: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    payload: String,
    payload_type: String,
    signatures: Vec<Signature>,
}

#[derive(Serialize)]
struct Signature {
    keyid: String,
    sig: String,
}

/// sluice's records of `main` in a repository, kept on its remote or in
/// the host's own files.
pub struct Recorder {
    repo: Repo,
    remote: bool,
    keys: PathBuf,
    log: RefCell<String>,
    version: Cell<u64>,
    next: Cell<u64>,
}

impl Recorder {
    /// sluice's records of `repo`'s `main`: on its remote `origin` when
    /// `remote`, and in the host's own files otherwise; each signer's key
    /// kept under `keys`.
    pub fn new(repo: &Repo, remote: bool, keys: &Path) -> Recorder {
        fs::create_dir_all(keys).unwrap();
        Recorder {
            repo: repo.clone(),
            remote,
            keys: keys.to_path_buf(),
            log: RefCell::default(),
            version: Cell::new(0),
            next: Cell::new(1),
        }
    }

    /// A landing of `entries` submissions together, as sluice lands them:
    /// a candidate combination tested, then one commit per entry on
    /// `main`, its first parent the one before it, each naming the
    /// combination; the log's `landing` line before the branch moves, and
    /// its `landed` line after where `done`.
    pub fn land(&self, entries: usize, done: bool) -> Landed {
        let from = self.repo.head();
        let combination = self.repo.git(&[
            "commit-tree",
            "HEAD^{tree}",
            "-p",
            "HEAD",
            "-m",
            "a candidate",
        ]);
        let ids: Vec<u64> = (0..entries)
            .map(|_| {
                let id = self.next.get();
                self.next.set(id + 1);
                id
            })
            .collect();
        self.append(&format!(
            r#"{{"schema":"sluice/log@1","at":"2026-10-19T12:00:00Z","event":"testing","combination":"{combination}","submissions":{ids:?},"place":"sluice/try/main/{}"}}"#,
            ids[0]
        ));
        let mut commits = Vec::new();
        for id in &ids {
            self.repo
                .write(&format!("entries/{id}.txt"), &id.to_string());
            self.repo.git(&["add", "-A"]);
            self.repo.git(&[
                "commit",
                "-q",
                "-m",
                &format!(
                    "chore(land): #{id} unit/{id}\n\n\
                     Sluice-Submission: {id}\n\
                     Sluice-From: unit/{id} (unit/{id})\n\
                     Sluice-Tested: {combination}\n\
                     Sluice-Result: passed (1 run, 0 replayed)"
                ),
            ]);
            commits.push(self.repo.head());
        }
        let to = commits.last().unwrap().clone();
        self.append(&format!(
            r#"{{"schema":"sluice/log@1","at":"2026-10-19T12:00:01Z","event":"landing","ids":{ids:?},"from":"{from}","to":"{to}"}}"#
        ));
        if self.remote {
            self.repo.push();
        }
        if done {
            self.append(&format!(
                r#"{{"schema":"sluice/log@1","at":"2026-10-19T12:00:02Z","event":"landed","ids":{ids:?},"to":"{to}"}}"#
            ));
        }
        self.write_state();
        Landed {
            commits,
            combination,
        }
    }

    /// A line added to the log, as sluice appends one.
    fn append(&self, line: &str) {
        let mut log = self.log.borrow_mut();
        log.push_str(line);
        log.push('\n');
    }

    /// The branch's records written where sluice keeps them: a new root
    /// commit of `log.jsonl` and `version` at `refs/sluice/state/main` on
    /// the remote, or the host's own `log.jsonl` under the common
    /// directory.
    fn write_state(&self) {
        let version = self.version.get() + 1;
        self.version.set(version);
        if !self.remote {
            let common =
                self.repo
                    .git(&["rev-parse", "--path-format=absolute", "--git-common-dir"]);
            let dir = Path::new(&common).join("sluice").join(MAIN);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("log.jsonl"), self.log.borrow().as_bytes()).unwrap();
            fs::write(dir.join("version"), format!("{version}\n")).unwrap();
            return;
        }
        let log = self.blob(self.log.borrow().as_bytes());
        let version = self.blob(format!("{version}\n").as_bytes());
        let tree = self.tree(None, &[("log.jsonl", &log), ("version", &version)]);
        let commit = self
            .repo
            .git(&["commit-tree", &tree, "-m", &format!("{MAIN} at {version}")]);
        self.repo.git(&[
            "push",
            "-q",
            "origin",
            &format!("+{commit}:refs/sluice/state/{MAIN}"),
        ]);
    }

    /// The key `signer` signs with, made the first time, and its key id.
    pub fn key(&self, signer: &str) -> (PathBuf, String) {
        let key = self.keys.join(signer);
        if !key.exists() {
            let out = Command::new(ssh_keygen())
                .args(["-q", "-t", "ed25519", "-N", "", "-C", signer, "-f"])
                .arg(&key)
                .output()
                .unwrap();
            assert!(out.status.success(), "{out:?}");
        }
        let out = Command::new(ssh_keygen())
            .args(["-l", "-E", "sha256", "-f"])
            .arg(key.with_extension("pub"))
            .output()
            .unwrap();
        let listed = String::from_utf8(out.stdout).unwrap();
        let keyid = listed.split_whitespace().nth(1).unwrap().to_string();
        (key, keyid)
    }

    /// The public key `signer` signs with, as an allowed-signers line
    /// names it.
    pub fn public_key(&self, signer: &str) -> String {
        let (key, _) = self.key(signer);
        let text = fs::read_to_string(key.with_extension("pub")).unwrap();
        text.split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A run of `results` on `system` for `combination`, signed by
    /// `signer` as sluice signs it, added to `refs/sluice/runs/<signer>`
    /// at `<combination>/<system or any>/<16 hex>.json`; answers the
    /// record's path and its bytes.
    pub fn record(
        &self,
        signer: &str,
        combination: &str,
        system: &str,
        results: &[Ran],
    ) -> (String, Vec<u8>) {
        let statement = Statement {
            statement: "https://in-toto.io/Statement/v1",
            subject: results
                .iter()
                .map(|r| Subject {
                    name: format!("{}@{system}", r.check),
                    digest: [("sha256".to_string(), r.key.clone())].into(),
                })
                .collect(),
            predicate_type: "https://agentplot.github.io/sluice/run/v1",
            predicate: Run {
                results: results
                    .iter()
                    .map(|r| RunResult {
                        check: &r.check,
                        key: &r.key,
                        outcome: &r.outcome,
                        replayed: r.replayed,
                    })
                    .collect(),
                claims: Claims {
                    executor: "process",
                    confined: false,
                    system,
                },
                branch: MAIN,
                combination,
                at: "2026-10-19T12:00:00Z",
            },
        };
        let payload = serde_json::to_vec(&statement).unwrap();
        let (key, keyid) = self.key(signer);
        let mut pae = format!("DSSEv1 {} {IN_TOTO} {} ", IN_TOTO.len(), payload.len()).into_bytes();
        pae.extend_from_slice(&payload);
        let signed = self.keys.join(format!("{signer}.pae"));
        fs::write(&signed, &pae).unwrap();
        let _ = fs::remove_file(signed.with_extension("pae.sig"));
        let out = Command::new(ssh_keygen())
            .args(["-q", "-Y", "sign", "-n", "sluice-run", "-f"])
            .arg(&key)
            .arg(&signed)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        let sig = fs::read(signed.with_extension("pae.sig")).unwrap();
        let bytes = serde_json::to_vec(&Envelope {
            payload: STANDARD.encode(&payload),
            payload_type: IN_TOTO.into(),
            signatures: vec![Signature {
                keyid,
                sig: STANDARD.encode(sig),
            }],
        })
        .unwrap();
        let path = format!(
            "{combination}/{}/{}.json",
            if system.is_empty() { "any" } else { system },
            &Digest::of(&bytes).hex()[..16]
        );
        self.add(signer, &path, &bytes);
        (path, bytes)
    }

    /// `bytes` added at `path` to `refs/sluice/runs/<signer>` on the
    /// remote, on top of what it holds, as sluice adds a record.
    pub fn add(&self, signer: &str, path: &str, bytes: &[u8]) {
        let name = format!("refs/sluice/runs/{signer}");
        let listed = self.repo.git(&["ls-remote", "origin", &name]);
        let tip = listed.split_whitespace().next().map(str::to_string);
        if let Some(tip) = &tip {
            self.repo.git(&["fetch", "-q", "origin", &name]);
            assert_eq!(self.repo.git(&["rev-parse", "FETCH_HEAD"]), *tip);
        }
        let blob = self.blob(bytes);
        let tree = self.tree(tip.as_deref(), &[(path, &blob)]);
        let mut args = vec!["commit-tree", tree.as_str(), "-m", "runs"];
        if let Some(tip) = &tip {
            args.extend(["-p", tip.as_str()]);
        }
        let commit = self.repo.git(&args);
        self.repo
            .git(&["push", "-q", "origin", &format!("{commit}:{name}")]);
    }

    fn blob(&self, bytes: &[u8]) -> String {
        let file = self.keys.join("blob");
        fs::write(&file, bytes).unwrap();
        self.repo
            .git(&["hash-object", "-w", file.to_str().unwrap()])
    }

    /// The tree of `base` with each of `files` put at its path.
    fn tree(&self, base: Option<&str>, files: &[(&str, &String)]) -> String {
        let index = self.keys.join("index");
        let _ = fs::remove_file(&index);
        let indexed = |args: &[&str]| {
            let out = git_command()
                .env("GIT_INDEX_FILE", &index)
                .arg("-C")
                .arg(&self.repo.path)
                .args(args)
                .output()
                .unwrap();
            assert!(out.status.success(), "git {args:?}: {out:?}");
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        };
        if let Some(base) = base {
            indexed(&["read-tree", base]);
        }
        for (path, blob) in files {
            indexed(&[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("100644,{blob},{path}"),
            ]);
        }
        indexed(&["write-tree"])
    }
}

/// The repository the recorder's remote holds, for a test that reads it
/// as sluice's own.
pub fn remote_file(remote: &Path, reference: &str, path: &str) -> Vec<u8> {
    let out = git_command()
        .arg("--git-dir")
        .arg(remote)
        .args(["show", &format!("{reference}:{path}")])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    out.stdout
}

/// Every file of the tree `reference` names in `remote`.
pub fn remote_files(remote: &Path, reference: &str) -> Vec<String> {
    git(remote, &["ls-tree", "-r", "--name-only", reference])
        .lines()
        .map(str::to_string)
        .collect()
}

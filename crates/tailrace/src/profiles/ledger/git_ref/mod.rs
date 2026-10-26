//! The ledger seam (B.8) held as git refs (C.13): one ref per channel,
//! `refs/tailrace/ledger/<channel>` where the repository declares no
//! component and `refs/tailrace/ledger/<component>/<channel>` where it
//! declares components (125), each commit holding one promotion claim
//! and naming as its one parent the commit of the claim before it, the
//! first a commit with no parent (105, 120 to 122; design D9).
//!
//! Where a remote is bound, every chain and append first reads the
//! remote's ref into the repository's own copy, which moves only forward:
//! a remote that no longer extends what this repository read is a fork,
//! or a loss of the claims it no longer holds, reported and never chosen
//! between (105, 122, rule core.34). Append is a push that only
//! fast-forwards, never forced, and a push refused is answered after a
//! read again, never retried (120, rule core.6). A remote that does not
//! answer leaves the chain answered from the repository's copy as last
//! fetched, saying when, with git's words, and nothing appended (60).
//! With no remote, the repository's own ref is the ledger, moved only
//! from the commit it was read at.
//!
//! The push reaches the remote with the host's own git credentials at
//! the moment of use, and the profile keeps none (64, rule core.21).

use std::ffi::OsStr;
use std::path::PathBuf;

use tailrace_seams::{
    canonical, Answer, Chain, ChannelId, Claim, Digest, Envelope, Keeper, Not, Promotion, Read,
    Seam, Source, Why,
};

use crate::profiles::run::{Ran, Tool, GIT_REPOSITORY_ENV};

/// The file of each commit's tree holding its claim.
const FILE: &str = "promotion.json";

/// Who writes every commit of the ledger, with the claim's own time.
const IDENTITY: &str = "tailrace <tailrace@localhost>";

/// The ledger seam held as refs of the repository at a path, pushed to
/// and fetched from a remote where one is bound.
#[derive(Clone, Debug)]
pub struct GitRef {
    repository: PathBuf,
    remote: Option<String>,
    git: Tool,
}

fn source() -> Source {
    Source::new(Seam::Keeper, "git_ref")
}

/// The ref holding `channel`'s chain, the channel as it is spelled: a
/// component's slash is a segment of the ref's path, which git allows
/// (C.13).
fn ledger(channel: &ChannelId) -> String {
    format!("refs/tailrace/ledger/{}", channel.spelled())
}

/// The ref the remote's chain of `channel` is fetched into, made anew at
/// every fetch, so its reflog says when the remote was last read.
fn fetched(channel: &ChannelId) -> String {
    format!("refs/tailrace/fetched/{}", channel.spelled())
}

/// A chain as a read found it: each claim by the commit holding it, from
/// the first, and how it was read.
struct Held {
    links: Vec<(String, Envelope)>,
    read: Read,
}

impl Held {
    fn head(&self) -> Option<&(String, Envelope)> {
        self.links.last()
    }

    fn head_digest(&self) -> Option<Digest> {
        self.head().map(|(_, e)| e.digest())
    }
}

fn digests(links: &[(String, Envelope)]) -> String {
    links
        .iter()
        .map(|(_, e)| e.digest().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl GitRef {
    /// The chains of the repository at `repository`, as `remote` holds
    /// them where one is bound and as the repository does otherwise, with
    /// `git` as found on `PATH`.
    pub fn new(repository: impl Into<PathBuf>, remote: Option<String>) -> GitRef {
        GitRef {
            repository: repository.into(),
            remote,
            git: Tool::new("git", source()),
        }
    }

    /// The ledger seam and this profile, as every answer names them.
    pub fn source(&self) -> &Source {
        self.git.source()
    }

    fn args<'a, S: AsRef<OsStr>>(&'a self, args: &'a [S]) -> Vec<&'a OsStr> {
        let mut all = vec![
            OsStr::new("-C"),
            self.repository.as_os_str(),
            OsStr::new("-c"),
            OsStr::new("core.logAllRefUpdates=always"),
        ];
        all.extend(args.iter().map(AsRef::as_ref));
        all
    }

    /// git run in the repository with `args`, answering when it exits
    /// with one of `allowed`.
    fn git<S: AsRef<OsStr>>(&self, args: &[S], allowed: &[i32]) -> Answer<Ran> {
        self.git
            .output(None, &self.args(args), GIT_REPOSITORY_ENV, allowed)
    }

    /// git run with `input` on its stdin, answering what it printed,
    /// trimmed, when it exits 0.
    fn fed<S: AsRef<OsStr>>(&self, args: &[S], input: &[u8]) -> Answer<String> {
        let all = self.args(args);
        let ran = self.git.fed(None, &all, GIT_REPOSITORY_ENV, input)?;
        if ran.code != Some(0) {
            return Err(self.git.failed(&all, &ran));
        }
        Ok(String::from_utf8_lossy(&ran.stdout).trim().to_string())
    }

    /// The commit `name` names, `None` when it names none.
    fn commit_at(&self, name: &str) -> Answer<Option<String>> {
        let ran = self.git(
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{name}^{{commit}}"),
            ],
            &[0, 1],
        )?;
        Ok((ran.code == Some(0)).then(|| String::from_utf8_lossy(&ran.stdout).trim().to_string()))
    }

    fn is_ancestor(&self, ancestor: &str, of: &str) -> Answer<bool> {
        let ran = self.git(&["merge-base", "--is-ancestor", ancestor, of], &[0, 1])?;
        Ok(ran.code == Some(0))
    }

    /// `name` moved to `to` from `from`, or made where `from` is `None`,
    /// its move logged.
    fn move_ref(&self, name: &str, to: &str, from: Option<&str>, why: &str) -> Answer<Ran> {
        self.git(
            &[
                "update-ref",
                "--create-reflog",
                "-m",
                why,
                name,
                to,
                from.unwrap_or(""),
            ],
            &[0, 128],
        )
    }

    /// The claim `commit` holds; unreadable naming the commit where its
    /// tree holds no claim.
    fn envelope_at(&self, channel: &ChannelId, commit: &str) -> Answer<Envelope> {
        let ran = self.git(
            &["cat-file", "blob", &format!("{commit}:{FILE}")],
            &[0, 128],
        )?;
        let envelope = (ran.code == Some(0))
            .then(|| serde_json::from_slice::<Envelope>(&ran.stdout).ok())
            .flatten();
        envelope.ok_or_else(|| {
            Not::unreadable(
                self.source(),
                format!("the chain of {channel} at {commit}"),
                Some(format!("{commit} holds no {FILE} holding a claim")),
            )
        })
    }

    /// 121: every claim from the first commit to `tip`, in order;
    /// unreadable naming the first commit with two parents, or holding no
    /// claim, never answered in part.
    fn walk(&self, channel: &ChannelId, tip: &str) -> Answer<Vec<(String, Envelope)>> {
        let ran = self.git(&["rev-list", "--reverse", "--parents", tip], &[0])?;
        let mut links = Vec::new();
        for line in String::from_utf8_lossy(&ran.stdout).lines() {
            let mut ids = line.split_whitespace();
            let Some(commit) = ids.next() else { continue };
            if ids.count() > 1 {
                return Err(Not::unreadable(
                    self.source(),
                    format!("the chain of {channel} at {commit}"),
                    Some(format!("{commit} has more than one parent")),
                ));
            }
            links.push((commit.to_string(), self.envelope_at(channel, commit)?));
        }
        Ok(links)
    }

    /// The newest time, in seconds since the Unix epoch, the repository
    /// wrote `channel`'s ref or fetched its remote's, from their reflogs;
    /// `None` where neither logged a write.
    fn when(&self, channel: &ChannelId) -> Answer<Option<u64>> {
        let mut newest = None;
        for name in [ledger(channel), fetched(channel)] {
            if self.commit_at(&name)?.is_none() {
                continue;
            }
            let ran = self.git(
                &[
                    "reflog",
                    "show",
                    "--date=unix",
                    "--format=%gd",
                    "-n",
                    "1",
                    &name,
                ],
                &[0],
            )?;
            let text = String::from_utf8_lossy(&ran.stdout);
            let at = text
                .rsplit_once("@{")
                .and_then(|(_, rest)| rest.trim().strip_suffix('}'))
                .and_then(|n| n.parse::<u64>().ok());
            newest = newest.max(at);
        }
        Ok(newest)
    }

    /// 60, C.13: the remote not answering, for `failed`, git's own words:
    /// the repository's copy of `channel`'s chain as last fetched or
    /// appended, saying when; `failed` itself where it holds none.
    fn kept(&self, channel: &ChannelId, failed: Not) -> Answer<Held> {
        let said = match &failed {
            Not::Unreadable(u) => u.said.clone().unwrap_or_default(),
            _ => return Err(failed),
        };
        let Some(local) = self.commit_at(&ledger(channel))? else {
            return Err(failed);
        };
        let links = self.walk(channel, &local)?;
        let Some(at) = self.when(channel)? else {
            return Err(failed);
        };
        Ok(Held {
            links,
            read: Read::Kept { at, said },
        })
    }

    /// The opening read of every operation. With a remote: its ref
    /// listed, and fetched, into the repository's copy where it extends
    /// what was read; a remote holding less than was read unreadable
    /// naming the claims it lost, and a remote and a copy that part a
    /// fork naming the first claim of each side, the copy kept (105,
    /// 122). With none, the repository's own ref.
    fn read(&self, channel: &ChannelId) -> Answer<Held> {
        let name = ledger(channel);
        let Some(remote) = &self.remote else {
            let links = match self.commit_at(&name)? {
                Some(local) => self.walk(channel, &local)?,
                None => Vec::new(),
            };
            return Ok(Held {
                links,
                read: Read::Source,
            });
        };
        let listed = match self.git(&["ls-remote", remote, &name], &[0]) {
            Ok(ran) => String::from_utf8_lossy(&ran.stdout)
                .lines()
                .filter_map(|line| line.split_once('\t'))
                .find(|(_, listed)| *listed == name)
                .map(|(object, _)| object.to_string()),
            Err(failed) => return self.kept(channel, failed),
        };
        let local = self.commit_at(&name)?;
        let Some(_) = listed else {
            return match local {
                None => Ok(Held {
                    links: Vec::new(),
                    read: Read::Source,
                }),
                Some(local) => Err(self.lost(channel, remote, &self.walk(channel, &local)?)),
            };
        };

        let scratch = fetched(channel);
        self.git(&["update-ref", "-d", &scratch], &[0])?;
        let fetch = self.git(
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "--no-write-fetch-head",
                remote,
                &format!("+{name}:{scratch}"),
            ],
            &[0],
        );
        if let Err(failed) = fetch {
            return self.kept(channel, failed);
        }
        let Some(theirs) = self.commit_at(&scratch)? else {
            return Err(Not::unreadable(
                self.source(),
                format!("the chain of {channel} fetched from {remote}"),
                Some(format!("{scratch} names no commit after the fetch")),
            ));
        };
        let source = |links| {
            Ok(Held {
                links,
                read: Read::Source,
            })
        };
        let Some(local) = local else {
            let links = self.walk(channel, &theirs)?;
            self.move_ref(&name, &theirs, None, "fetch")?;
            return source(links);
        };
        if local == theirs {
            return source(self.walk(channel, &local)?);
        }
        if self.is_ancestor(&local, &theirs)? {
            let links = self.walk(channel, &theirs)?;
            self.move_ref(&name, &theirs, Some(&local), "fetch")?;
            return source(links);
        }
        let ours = self.walk(channel, &local)?;
        let fetched_links = self.walk(channel, &theirs)?;
        let shared = ours
            .iter()
            .zip(&fetched_links)
            .take_while(|(a, b)| a.0 == b.0)
            .count();
        if shared == fetched_links.len() {
            return Err(self.lost(channel, remote, &ours[shared..]));
        }
        Err(Not::refused(
            self.source(),
            Why::Fork {
                predecessor: shared.checked_sub(1).map(|i| ours[i].1.digest()),
                claims: vec![ours[shared].1.digest(), fetched_links[shared].1.digest()],
            },
            None,
        ))
    }

    /// 122: `remote` no longer holds `claims`, which this repository read
    /// from it.
    fn lost(&self, channel: &ChannelId, remote: &str, claims: &[(String, Envelope)]) -> Not {
        Not::unreadable(
            self.source(),
            format!(
                "the chain of {channel} on {remote}, which no longer holds {}",
                digests(claims)
            ),
            Some(format!(
                "{remote}'s {} does not descend from what this repository read",
                ledger(channel)
            )),
        )
    }

    /// The commit holding `promotion` on `parent`: a tree of one blob,
    /// the envelope's canonical JSON, written by tailrace at the claim's
    /// time where it reads as a promotion, so no index or working tree is
    /// touched.
    fn commit_of(&self, promotion: &Envelope, parent: Option<&str>) -> Answer<String> {
        let blob = self.fed(&["hash-object", "-w", "--stdin"], &canonical(promotion))?;
        let tree = self.fed(
            &["mktree"],
            format!("100644 blob {blob}\t{FILE}\n").as_bytes(),
        )?;
        let at = Claim::from_envelope(promotion)
            .as_ref()
            .and_then(Promotion::of)
            .map_or(0, |p| p.at);
        let mut text = format!("tree {tree}\n");
        if let Some(parent) = parent {
            text.push_str(&format!("parent {parent}\n"));
        }
        text.push_str(&format!(
            "author {IDENTITY} {at} +0000\ncommitter {IDENTITY} {at} +0000\n\npromotion {}\n",
            promotion.digest()
        ));
        self.fed(
            &["hash-object", "-t", "commit", "-w", "--stdin"],
            text.as_bytes(),
        )
    }
}

impl Keeper for GitRef {
    /// 120: the claim's commit on the head's, held when `predecessor` is
    /// the head as the remote holds it; pushed so the remote takes only a
    /// fast-forward. A push refused is read again: refused naming the
    /// remote's head where it moved, and with git's words where it did
    /// not (59). Unreadable with git's words where the remote does not
    /// answer, never appended to the copy alone.
    fn append(
        &self,
        channel: &ChannelId,
        promotion: &Envelope,
        predecessor: Option<&Digest>,
    ) -> Answer<Digest> {
        let held = self.read(channel)?;
        if let Read::Kept { said, .. } = &held.read {
            return Err(Not::unreadable(
                self.source(),
                format!("the chain of {channel}"),
                Some(said.clone()),
            ));
        }
        let head = held.head_digest();
        if head.as_ref() != predecessor {
            return Err(Not::refused(self.source(), Why::NotHead { head }, None));
        }
        let parent = held.head().map(|(commit, _)| commit.as_str());
        let commit = self.commit_of(promotion, parent)?;
        let name = ledger(channel);
        let Some(remote) = &self.remote else {
            let moved = self.move_ref(&name, &commit, parent, "append")?;
            if moved.code == Some(0) {
                return Ok(promotion.digest());
            }
            let head = self.read(channel)?.head_digest();
            return Err(Not::refused(self.source(), Why::NotHead { head }, None));
        };
        let pushed = self.git(
            &["push", "--quiet", remote, &format!("{commit}:{name}")],
            &[0, 1],
        )?;
        if pushed.code == Some(0) {
            // The remote holds it; the copy follows, and a copy another
            // writer moved meanwhile is brought forward by the next read.
            self.move_ref(&name, &commit, parent, "append")?;
            return Ok(promotion.digest());
        }
        let words = String::from_utf8_lossy(&pushed.stderr).trim().to_string();
        let now = self.read(channel)?.head_digest();
        let why = if now == head {
            Why::Said
        } else {
            Why::NotHead { head: now }
        };
        Err(Not::refused(self.source(), why, Some(words)))
    }

    /// 121: every claim of `channel` from the first, read from the remote
    /// where one is bound and answers, and from the repository's copy as
    /// last fetched where it does not (60); absent where none is held.
    fn chain(&self, channel: &ChannelId) -> Answer<Chain> {
        let held = self.read(channel)?;
        if held.links.is_empty() {
            return Err(Not::absent(self.source(), format!("chain of {channel}")));
        }
        Ok(Chain {
            claims: held.links.into_iter().map(|(_, e)| e).collect(),
            read: held.read,
        })
    }
}

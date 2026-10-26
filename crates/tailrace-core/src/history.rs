//! The repository's history, which the core reads itself through git
//! (section 9): the files of a landed revision's tree, the commits that
//! touch a component's paths and the tags that carry its prefix, read
//! from the local history and the shared one (21 to 23, 65, 123; design
//! D3, D6).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use git2::{ErrorCode, Object, ObjectType, Oid, Repository};
use glob::Pattern;
use regex::Regex;
use semver::Version;
use tailrace_seams::{Answer, Branch, Not, Revision, Source};

use crate::Landed;

/// What the history's source names when the history read is the
/// repository's own, not a remote's.
pub const LOCAL: &str = "local";

/// A repository's history, opened at one path.
pub struct History {
    path: PathBuf,
    git: Repository,
    cliff: git_cliff_core::repo::Repository,
}

/// A commit: its revision and its message as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    pub revision: Revision,
    pub message: String,
}

/// A tag carrying a component's prefix: its name, the semantic version
/// after the prefix, and the revision it names, peeled to the commit.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    pub name: String,
    pub version: Version,
    pub revision: Revision,
}

/// What a push of a tag to the shared history came to.
pub(crate) enum Pushed {
    /// The shared history holds the tag where it was pushed from.
    Held,
    /// The shared history refused it, in git's words.
    Rejected(String),
}

fn local() -> Source {
    Source::history(LOCAL)
}

fn words(error: &git2::Error) -> Option<String> {
    Some(error.message().to_string())
}

fn said(output: &Output) -> Option<String> {
    Some(String::from_utf8_lossy(&output.stderr).into_owned())
}

impl History {
    /// The repository at `path`, a working tree or a bare repository,
    /// with no search of its parents; unreadable naming the path with
    /// git's words when it is none.
    pub fn open(path: impl AsRef<Path>) -> Answer<History> {
        let path = path.as_ref().to_path_buf();
        let what = format!("the repository at {}", path.display());
        let git = Repository::open(&path)
            .map_err(|e| Not::unreadable(&local(), what.clone(), words(&e)))?;
        let cliff = git_cliff_core::repo::Repository::init(path.clone())
            .map_err(|e| Not::unreadable(&local(), what, Some(e.to_string())))?;
        Ok(History { path, git, cliff })
    }

    /// The commit the landed revision names; unreadable when this
    /// repository does not hold it.
    fn commit(&self, revision: &Revision) -> Answer<git2::Commit<'_>> {
        self.git
            .find_commit_by_prefix(revision.as_str())
            .map_err(|e| Not::unreadable(&local(), format!("the revision {revision}"), words(&e)))
    }

    /// The landed revision by its full git id, as tags name revisions.
    pub fn revision(&self, landed: &Landed) -> Answer<Revision> {
        Ok(Revision::new(
            self.commit(landed.revision())?.id().to_string(),
        ))
    }

    /// The file at `path` in the landed revision's tree, never the working
    /// tree's (65); absent when the tree holds nothing there.
    pub fn blob(&self, landed: &Landed, path: &str) -> Answer<Vec<u8>> {
        let revision = landed.revision();
        let what = format!("{path} at {revision}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        let tree = self.commit(revision)?.tree().map_err(unreadable)?;
        let entry = match tree.get_path(Path::new(path)) {
            Ok(entry) => entry,
            Err(e) if e.code() == ErrorCode::NotFound => {
                return Err(Not::absent(&local(), what));
            }
            Err(e) => return Err(unreadable(e)),
        };
        let object = entry.to_object(&self.git).map_err(unreadable)?;
        match object.into_blob() {
            Ok(blob) => Ok(blob.content().to_vec()),
            Err(_) => Err(Not::unreadable(
                &local(),
                what.clone(),
                Some(format!("{path} is a tree, not a file")),
            )),
        }
    }

    /// The commits the landed revision reaches, after `since` where it is
    /// given, that change a file matching one of `paths`, matched by
    /// git-cliff's own path filter (123, ADR 0010).
    pub fn commits(
        &self,
        landed: &Landed,
        since: Option<&Revision>,
        paths: &[String],
    ) -> Answer<Vec<Commit>> {
        let revision = self.commit(landed.revision())?.id();
        let range = match since {
            Some(since) => format!("{}..{revision}", self.commit(since)?.id()),
            None => revision.to_string(),
        };
        let what = format!("the commits of {range}");
        let patterns = paths
            .iter()
            .map(|p| Pattern::new(p))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Not::unreadable(&local(), what.clone(), Some(e.to_string())))?;
        let commits = self
            .cliff
            .commits(Some(&range), Some(patterns), None, false)
            .map_err(|e| Not::unreadable(&local(), what, Some(e.to_string())))?;
        Ok(commits
            .iter()
            .map(|c| Commit {
                revision: Revision::new(c.id().to_string()),
                message: String::from_utf8_lossy(c.message_bytes()).into_owned(),
            })
            .collect())
    }

    /// Every tag carrying `prefix` then a semantic version, in the local
    /// history and the shared one, that names the landed revision or a
    /// revision it reaches (22, 123). A tag only the shared history holds
    /// is read with `git ls-remote`, which writes nothing; a shared
    /// history that cannot be reached makes the read unreadable, never
    /// one of the local history alone.
    pub fn tags(&self, landed: &Landed, prefix: &str) -> Answer<Vec<Tag>> {
        let pattern = Regex::new(&format!("^{}(.+)$", regex::escape(prefix))).map_err(|e| {
            Not::unreadable(
                &local(),
                format!("the prefix {prefix}"),
                Some(e.to_string()),
            )
        })?;
        let version = |name: &str| {
            pattern
                .captures(name)
                .and_then(|c| Version::parse(&c[1]).ok())
        };
        let what = format!("the tags carrying {prefix}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));

        let mut found = BTreeSet::new();
        let names = self.git.tag_names(None).map_err(unreadable)?;
        for name in &names {
            // A name that is not UTF-8 carries no prefix the policy declares.
            let Some(name) = name.map_err(unreadable)? else {
                continue;
            };
            let Some(v) = version(name) else { continue };
            let object = self
                .git
                .revparse_single(&format!("refs/tags/{name}"))
                .map_err(unreadable)?;
            if let Some(oid) = peeled(object).map_err(unreadable)? {
                found.insert((name.to_string(), v, oid));
            }
        }
        if let Some(remote) = self.remote(landed.branch())? {
            for (name, oid) in self.remote_tags(&remote, &format!("refs/tags/{prefix}*"))? {
                if let Some(v) = version(&name) {
                    found.insert((name, v, oid));
                }
            }
        }

        let landed = self.commit(landed.revision())?.id();
        let mut tags = Vec::new();
        for (name, version, oid) in found {
            if self.reaches(landed, oid).map_err(unreadable)? {
                tags.push(Tag {
                    name,
                    version,
                    revision: Revision::new(oid.to_string()),
                });
            }
        }
        Ok(tags)
    }

    /// Whether `from` is `to` or a descendant of it; a commit this
    /// repository does not hold is one no revision it holds reaches.
    fn reaches(&self, from: Oid, to: Oid) -> Result<bool, git2::Error> {
        if from == to {
            return Ok(true);
        }
        match self.git.find_commit(to) {
            Ok(_) => self.git.graph_descendant_of(from, to),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 22: the remote that holds the history every publisher of `branch`
    /// shares, as `branch.<name>.remote` names it; none when it names no
    /// remote, and the local history is then the whole of it.
    pub fn remote(&self, branch: &Branch) -> Answer<Option<String>> {
        let key = format!("branch.{branch}.remote");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), key.clone(), words(&e));
        match self.git.config().map_err(unreadable)?.get_string(&key) {
            Ok(remote) if remote == "." => Ok(None),
            Ok(remote) => Ok(Some(remote)),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// 114: the address `remote.<remote>.url` names in the repository's
    /// git configuration, as written, no `insteadOf` applied; none where
    /// it names no such remote.
    pub fn remote_url(&self, remote: &str) -> Answer<Option<String>> {
        let key = format!("remote.{remote}.url");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), key.clone(), words(&e));
        match self.git.config().map_err(unreadable)?.get_string(&key) {
            Ok(url) => Ok(Some(url)),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// The git command at this repository, with no prompt, so a remote
    /// asks the host's own credential helper and never a terminal (rule
    /// core.21).
    fn git(&self, at: &str, what: &str, args: &[&str]) -> Answer<Output> {
        Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| Not::unreadable(&Source::history(at), what, Some(e.to_string())))
    }

    /// The tags `remote` holds matching `pattern`, each peeled to the
    /// revision it names.
    fn remote_tags(&self, remote: &str, pattern: &str) -> Answer<Vec<(String, Oid)>> {
        let what = format!("the tags of {remote}");
        let out = self.git(remote, &what, &["ls-remote", "--tags", remote, pattern])?;
        if !out.status.success() {
            return Err(Not::unreadable(&Source::history(remote), what, said(&out)));
        }
        let mut tags: Vec<(String, Oid)> = Vec::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let Some((oid, name)) = line.split_once('\t') else {
                continue;
            };
            let Some(name) = name.strip_prefix("refs/tags/") else {
                continue;
            };
            let oid = Oid::from_str(oid)
                .map_err(|e| Not::unreadable(&Source::history(remote), what.clone(), words(&e)))?;
            match name.strip_suffix("^{}") {
                Some(name) => match tags.iter_mut().find(|(n, _)| n == name) {
                    Some(tag) => tag.1 = oid,
                    None => tags.push((name.to_string(), oid)),
                },
                None if !tags.iter().any(|(n, _)| n == name) => tags.push((name.to_string(), oid)),
                None => {}
            }
        }
        Ok(tags)
    }

    /// The revision the local tag `name` names, peeled, or none when the
    /// local history holds no such tag.
    pub(crate) fn tag_at(&self, name: &str) -> Answer<Option<Revision>> {
        let what = format!("the tag {name}");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        match self.git.revparse_single(&format!("refs/tags/{name}")) {
            Ok(object) => Ok(peeled(object)
                .map_err(unreadable)?
                .map(|oid| Revision::new(oid.to_string()))),
            Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// A lightweight tag `name` at the landed revision in the local
    /// history, never forced over one already there (rule core.6).
    pub(crate) fn make_tag(&self, name: &str, landed: &Landed) -> Answer<()> {
        let what = format!("the tag {name}");
        let commit = self.commit(landed.revision())?;
        self.git
            .tag_lightweight(name, commit.as_object(), false)
            .map(|_| ())
            .map_err(|e| Not::unreadable(&local(), what, words(&e)))
    }

    /// Removes the local tag `name`, made by a claim the shared history
    /// refused.
    pub(crate) fn drop_tag(&self, name: &str) -> Answer<()> {
        self.git
            .tag_delete(name)
            .map_err(|e| Not::unreadable(&local(), format!("the tag {name}"), words(&e)))
    }

    /// Pushes the local tag `name` to `remote` without force, by the git
    /// command, so the host's own credential helper answers (rule
    /// core.21).
    pub(crate) fn push_tag(&self, remote: &str, name: &str) -> Answer<Pushed> {
        let what = format!("{remote}, pushing {name}");
        let refspec = format!("refs/tags/{name}:refs/tags/{name}");
        let out = self.git(remote, &what, &["push", "--porcelain", remote, &refspec])?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        // Each ref's line is its flag, the refspec and git's summary.
        let flag = stdout.lines().find_map(|line| {
            let mut fields = line.split('\t');
            let flag = fields.next()?;
            (fields.next()? == refspec).then_some(flag)
        });
        match flag {
            Some("*" | "=") if out.status.success() => Ok(Pushed::Held),
            Some("!") => Ok(Pushed::Rejected(format!(
                "{stdout}{}",
                String::from_utf8_lossy(&out.stderr)
            ))),
            _ => Err(Not::unreadable(&Source::history(remote), what, said(&out))),
        }
    }

    /// The revision the tag `name` names in `remote`'s history, peeled,
    /// or none when it holds no such tag.
    pub(crate) fn remote_tag(&self, remote: &str, name: &str) -> Answer<Option<Revision>> {
        let refname = format!("refs/tags/{name}");
        Ok(self
            .remote_tags(remote, &refname)?
            .into_iter()
            .find(|(n, _)| n == name)
            .map(|(_, oid)| Revision::new(oid.to_string())))
    }
}

/// The commit a tag's object names, through any annotated tags; none
/// when it names a tree or a file.
fn peeled(mut object: Object<'_>) -> Result<Option<Oid>, git2::Error> {
    loop {
        match object.kind() {
            Some(ObjectType::Commit) => return Ok(Some(object.id())),
            Some(ObjectType::Tag) => {
                object = object
                    .into_tag()
                    .map_err(|_| git2::Error::from_str("a tag that is not a tag"))?
                    .target()?
            }
            _ => return Ok(None),
        }
    }
}

impl History {
    /// The commit the local branch `branch` names, by its full id; absent
    /// when the repository holds no such branch.
    pub fn head(&self, branch: &Branch) -> Answer<Revision> {
        let what = format!("branch {branch}");
        match self
            .git
            .find_branch(branch.as_str(), git2::BranchType::Local)
        {
            Ok(found) => found
                .get()
                .peel_to_commit()
                .map(|c| Revision::new(c.id().to_string()))
                .map_err(|e| Not::unreadable(&local(), what, words(&e))),
            Err(e) if e.code() == ErrorCode::NotFound => Err(Not::absent(&local(), what)),
            Err(e) => Err(Not::unreadable(&local(), what, words(&e))),
        }
    }

    /// The newest revision of the local branch `branch` that the
    /// checkout's head descends from, their common ancestor, by its full
    /// id (design D4); absent naming the branch when the repository holds
    /// no such branch or the two share no history.
    pub fn base(&self, branch: &Branch) -> Answer<Revision> {
        let tip = self.head(branch)?;
        let what = format!("a revision of branch {branch} that HEAD descends from");
        let unreadable = |e: git2::Error| Not::unreadable(&local(), what.clone(), words(&e));
        let head = self
            .git
            .head()
            .and_then(|h| h.peel_to_commit())
            .map_err(unreadable)?;
        let tip = Oid::from_str(tip.as_str()).map_err(unreadable)?;
        match self.git.merge_base(head.id(), tip) {
            Ok(base) => Ok(Revision::new(base.to_string())),
            Err(e) if e.code() == ErrorCode::NotFound => Err(Not::absent(&local(), what)),
            Err(e) => Err(unreadable(e)),
        }
    }

    /// The commit `text` names, a full or short id, a branch, a tag or
    /// any revision git reads, by its full id, so every act records the
    /// revision it acted on (rule core.4); absent with git's words when it
    /// names no commit.
    pub fn resolve(&self, text: &str) -> Answer<Revision> {
        let absent =
            |e: git2::Error| Not::absent(&local(), format!("commit named {text}: {}", e.message()));
        let object = match self.git.revparse_single(text) {
            Ok(object) => object,
            Err(e)
                if matches!(
                    e.code(),
                    ErrorCode::NotFound | ErrorCode::Ambiguous | ErrorCode::InvalidSpec
                ) =>
            {
                return Err(absent(e));
            }
            Err(e) => {
                return Err(Not::unreadable(
                    &local(),
                    format!("the revision {text}"),
                    words(&e),
                ))
            }
        };
        object
            .peel_to_commit()
            .map(|c| Revision::new(c.id().to_string()))
            .map_err(absent)
    }

    /// The root of the working tree this history was opened at, absolute;
    /// none for a bare repository.
    pub fn workdir(&self) -> Option<PathBuf> {
        let dir = self.git.workdir()?;
        Some(dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf()))
    }

    /// The repository's common git directory, which every worktree of it
    /// on the host shares and no push carries (62).
    pub fn common_dir(&self) -> PathBuf {
        let dir = self.git.commondir();
        dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf())
    }
}

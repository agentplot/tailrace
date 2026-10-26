//! The version a landed revision takes per component, derived from the
//! history alone by the authors' rule, and which components a landing
//! touched, and the claim of its name in the shared history (21 to 24,
//! 123, 124, rules core.6, core.13, core.37; design D5, D6). Nothing here
//! takes the build seam: the build never learns the version.

use git_cliff_core::changelog::Changelog;
use git_cliff_core::commit::Commit as CliffCommit;
use git_cliff_core::config::{Bump, ChangelogConfig, Config, GitConfig, RemoteConfig};
use git_cliff_core::release::Release;
use semver::{BuildMetadata, Prerelease, Version};
use tailrace_seams::{
    Answer, Component, ComponentName, Not, Revision, Source, Version as Named, Why,
};

use crate::history::Pushed;
use crate::{Commit, Declared, History, Landed, Rule, Tag, LOCAL};

/// Whether a version is a release or a prerelease of the next one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Release,
    Prerelease,
}

/// A component's version at a landed revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versioned {
    pub component: ComponentName,
    /// The version, bare of the component's prefix.
    pub version: Named,
    /// The version's name, the component's prefix then the version (22).
    pub name: String,
    pub kind: Kind,
    /// The component's last release tag before this version.
    pub last: Option<Tag>,
    /// The commits the version is derived from: those the landed
    /// revision reaches since `last` that touch the component's paths, in
    /// the order the history gives them, the same on a revision a tag
    /// already names as on its first publish (134; design D1).
    pub since: Vec<Commit>,
    /// Whether a tag of this name already names the landed revision.
    pub claimed: bool,
}

/// What the rule derives for a component at a landed revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Derived {
    Versioned(Versioned),
    /// No commit since the component's last release tag touched its
    /// paths, so it gets no new version (124).
    Unchanged {
        component: ComponentName,
        last: Option<Tag>,
    },
}

fn rule_failed(component: &Component, said: String) -> Not {
    Not::unreadable(
        &Source::history(LOCAL),
        format!("the version of {}", component.name),
        Some(said),
    )
}

/// The highest of `tags` by semantic version precedence, never by time,
/// spelling or the order git lists them (23).
fn highest<'a>(tags: impl Iterator<Item = &'a Tag>) -> Option<&'a Tag> {
    tags.max_by(|a, b| a.version.cmp_precedence(&b.version))
}

/// 21: the version of `component` at the landed revision under `rule`,
/// from the history alone. The commits since the highest release tag the
/// revision reaches below it that touch the component's paths are read
/// first, and the version keeps them (134). Then, in order: a tag
/// carrying the component's prefix at the revision itself gives its
/// version (18); otherwise those commits decide it, none giving no
/// version, a feature, a fix or a breaking change a release by git-cliff's
/// bump, and anything else a prerelease of the next patch numbered by how
/// many there are.
pub fn derive(
    history: &History,
    landed: &Landed,
    rule: &Rule,
    component: &Component,
) -> Answer<Derived> {
    let revision = history.revision(landed)?;
    let tags = history.tags(landed, &component.prefix)?;
    let release = |t: &&Tag| t.version.pre.is_empty();
    let (here, before): (Vec<&Tag>, Vec<&Tag>) = tags.iter().partition(|t| t.revision == revision);
    let last = highest(before.iter().copied().filter(release)).cloned();

    let since = history.commits(landed, last.as_ref().map(|t| &t.revision), &component.paths)?;

    let held = highest(here.iter().copied().filter(release)).or(highest(here.iter().copied()));
    if let Some(tag) = held {
        return Ok(Derived::Versioned(Versioned {
            component: component.name.clone(),
            version: Named::new(tag.version.to_string()),
            name: tag.name.clone(),
            kind: if tag.version.pre.is_empty() {
                Kind::Release
            } else {
                Kind::Prerelease
            },
            last,
            since,
            claimed: true,
        }));
    }

    if since.is_empty() {
        return Ok(Derived::Unchanged {
            component: component.name.clone(),
            last,
        });
    }

    let (version, kind) = if since.iter().any(releases) {
        let version = match &last {
            Some(last) => {
                bump(rule, &last.version, &since).map_err(|said| rule_failed(component, said))?
            }
            None => rule.initial.clone(),
        };
        (version, Kind::Release)
    } else {
        let mut version = match &last {
            Some(last) => Version::new(
                last.version.major,
                last.version.minor,
                last.version.patch + 1,
            ),
            None => rule.initial.clone(),
        };
        version.build = BuildMetadata::EMPTY;
        version.pre = Prerelease::new(&format!("{}.{}", rule.prerelease, since.len()))
            .map_err(|e| rule_failed(component, e.to_string()))?;
        (version, Kind::Prerelease)
    };
    Ok(Derived::Versioned(Versioned {
        component: component.name.clone(),
        name: format!("{}{version}", component.prefix),
        version: Named::new(version.to_string()),
        kind,
        last,
        since,
        claimed: false,
    }))
}

/// Whether git-cliff reads `commit` as a conventional feature, fix or
/// breaking change, the commits that make a release.
fn releases(commit: &Commit) -> bool {
    let Ok(read) = cliff_commit(commit).into_conventional() else {
        return false;
    };
    read.conv.is_some_and(|conv| {
        let kind = conv.type_();
        conv.breaking()
            || kind.as_str().eq_ignore_ascii_case("feat")
            || kind.as_str().eq_ignore_ascii_case("fix")
    })
}

pub(crate) fn cliff_commit(commit: &Commit) -> CliffCommit<'static> {
    CliffCommit::new(
        commit.revision.to_string(),
        commit.message.trim_end().to_string(),
    )
}

/// git-cliff's conventional bump of `last` over `since`, with the rule's
/// bump settings (section 9, ADR 0010).
fn bump(rule: &Rule, last: &Version, since: &[Commit]) -> Result<Version, String> {
    let config = Config {
        changelog: ChangelogConfig::default(),
        git: GitConfig {
            conventional_commits: true,
            ..GitConfig::default()
        },
        remote: RemoteConfig {
            offline: true,
            ..RemoteConfig::default()
        },
        bump: Bump {
            features_always_bump_minor: Some(rule.features_always_bump_minor),
            breaking_always_bump_major: Some(rule.breaking_always_bump_major),
            initial_tag: Some(rule.initial.to_string()),
            ..Bump::default()
        },
    };
    let previous = Release {
        version: Some(last.to_string()),
        ..Release::default()
    };
    let unreleased = Release {
        version: None,
        commits: since.iter().map(cliff_commit).collect(),
        previous: Some(Box::new(previous)),
        ..Release::default()
    };
    let mut changelog =
        Changelog::new(vec![unreleased], config, None).map_err(|e| e.to_string())?;
    let next = changelog
        .bump_version()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("git-cliff gave no version after {last}"))?;
    Version::parse(&next).map_err(|e| format!("{next}: {e}"))
}

/// 124: every component the landed revision touched since its last
/// release tag, each with its version; a commit under a path two
/// components name versions both, and a component no such commit touched
/// is not answered (S15).
pub fn touched(history: &History, landed: &Landed, declared: &Declared) -> Answer<Vec<Versioned>> {
    let mut touched = Vec::new();
    for component in &declared.components {
        if let Derived::Versioned(v) = derive(history, landed, &declared.rule, component)? {
            touched.push(v);
        }
    }
    Ok(touched)
}

/// A version's name claimed at a landed revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claimed {
    pub name: String,
    pub revision: Revision,
    /// The remote holding the shared history the name was claimed in;
    /// none when the branch shares no history beyond the local one.
    pub shared: Option<String>,
}

/// 22: claims `versioned`'s name as a tag at the landed revision, in the
/// local history and then the shared one, before anything is packed. A
/// lightweight tag is made without force and pushed without force, so a
/// name the same revision already holds is claimed again with nothing
/// changed, and a name another revision holds, in either history, is
/// refused [`Why::NameHeld`] naming both with git's words, the local tag
/// this claim made removed and no tag moved (rule core.6). A shared
/// history that cannot be reached is unreadable with git's words, and the
/// local tag stays for a later claim at the revision to push again.
pub fn claim(history: &History, landed: &Landed, versioned: &Versioned) -> Answer<Claimed> {
    let name = &versioned.name;
    let revision = history.revision(landed)?;
    let held = |at: &str, other: Revision, said: Option<String>| {
        Not::refused(
            &Source::history(at),
            Why::NameHeld {
                name: name.clone(),
                revision: revision.clone(),
                other,
            },
            said,
        )
    };

    let made = match history.tag_at(name)? {
        Some(other) if other != revision => return Err(held(LOCAL, other, None)),
        Some(_) => false,
        None => {
            history.make_tag(name, landed)?;
            true
        }
    };
    let claimed = |shared: Option<String>| Claimed {
        name: name.clone(),
        revision: revision.clone(),
        shared,
    };
    let Some(remote) = history.remote(landed.branch())? else {
        return Ok(claimed(None));
    };
    match history.push_tag(&remote, name)? {
        Pushed::Held => Ok(claimed(Some(remote))),
        Pushed::Rejected(words) => match history.remote_tag(&remote, name)? {
            Some(other) if other == revision => Ok(claimed(Some(remote))),
            other => {
                if made {
                    history.drop_tag(name)?;
                }
                Err(match other {
                    Some(other) => held(&remote, other, Some(words)),
                    None => Not::refused(&Source::history(&remote), Why::Said, Some(words)),
                })
            }
        },
    }
}

//! A version's notes: the commits the version is derived from, rendered
//! by the version rule's own tool, git-cliff, under the repository's own
//! template for it, the `cliff.toml` the version's revision holds, and
//! git-cliff's own default where it holds none; tailrace imposes no
//! template of its own (21, 134, section 9). The release is handed to the
//! template as git-cliff's own run hands a tagged one: the tag, the
//! tagged commit and its time, the last release tag, and each commit as
//! the history holds it. Nothing tailrace hands it reads the clock, the
//! host, its environment or the checkout's path, so the same revision
//! renders the same notes on every host (18).

use git_cliff_core::changelog::Changelog;
use git_cliff_core::config::Config;
use git_cliff_core::embed::EmbeddedConfig;
use git_cliff_core::release::Release;
use git_cliff_core::CONFIG_FILES;
use tailrace_seams::{Answer, ComponentName, Not, Revision, Source, Version};
use toml::{Table, Value};

// `derive` by its module's path: imported through the root's globs,
// which take in this module's own, the compiler cannot settle the
// attribute of the same name.
use crate::version::{derive, Derived};
use crate::{Declared, History, Landed, Noted, Tag, LOCAL, POLICY_FILE};

/// The ecosystem of a version's notes, reserved: no output is packed in
/// it (design D3, D9).
pub const NOTES: &str = "notes";

/// A version's notes as rendered: Markdown, ending in one newline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    pub text: String,
}

impl Rendered {
    /// The bytes the store holds the notes as: the text, UTF-8.
    pub fn bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }
}

/// 134: the notes of `component` at `version`, tagged `tag` at `at`,
/// rendered from what `noted` names, the last release tag as the history
/// spells it and the commits the version was derived from, under the
/// template `at` holds. A commit the history no longer holds, and
/// git-cliff's words on the template, make them unreadable naming the
/// component and the version (58, 59).
pub fn render(
    history: &History,
    component: &ComponentName,
    version: &Version,
    tag: &str,
    at: &Revision,
    noted: &Noted,
) -> Answer<Rendered> {
    let what = format!("the notes of {component} {version}");
    let failed = |said: String| Not::unreadable(&Source::history(LOCAL), &what, Some(said));
    let config = template(history, at)?;

    let mut commits = Vec::new();
    for revision in &noted.commits {
        commits.push(history.cliff_commit(revision).map_err(|not| match not {
            Not::Absent(_) => Not::unreadable(
                &Source::history(LOCAL),
                format!("{what}, rendered from {revision}"),
                Some(format!("the history holds no commit {revision}")),
            ),
            not => not,
        })?);
    }
    // The history gives the newest first; git-cliff hands a release's
    // commits oldest first unless the template sorts them newest first.
    if config.git.sort_commits != "newest" {
        commits.reverse();
    }
    let tagged = history.cliff_commit(at)?;

    // A previous release, with no version where there is no last tag,
    // keeps a release with no commits rendered where the template renders
    // one always.
    let previous = Release {
        version: noted.last.clone(),
        ..Release::default()
    };
    let release = Release {
        version: Some(tag.to_string()),
        commit_id: Some(tagged.id),
        timestamp: Some(tagged.committer.timestamp),
        commits,
        previous: Some(Box::new(previous)),
        ..Release::default()
    };
    let changelog =
        Changelog::new(vec![release], config, None).map_err(|e| failed(e.to_string()))?;
    let mut out = Vec::new();
    changelog
        .generate(&mut out)
        .map_err(|e| failed(e.to_string()))?;
    let text = String::from_utf8(out).map_err(|e| failed(e.to_string()))?;
    Ok(Rendered {
        text: format!("{}\n", text.trim()),
    })
}

/// 134, 21: the commits `tag`'s notes were rendered from, derived again
/// at its landed revision under the policy file there, as the publish of
/// it derived them: what a rebuild keeps, and what status renders a
/// channel's changelog from (design D1, D8).
pub(crate) fn noted(
    history: &History,
    at: &Landed,
    component: &ComponentName,
    tag: &Tag,
) -> Answer<Noted> {
    let declared = Declared::read(history, at)?;
    let found = declared
        .components
        .iter()
        .find(|c| c.name == *component)
        .ok_or_else(|| {
            Not::absent(
                &Source::history(LOCAL),
                format!("component {component} in {POLICY_FILE} at {}", tag.revision),
            )
        })?;
    match derive(history, at, &declared.rule, found)? {
        Derived::Versioned(versioned) => Ok(Noted::of(&versioned)),
        Derived::Unchanged { .. } => Err(Not::unreadable(
            &Source::history(LOCAL),
            format!("the commits the notes of {} were rendered from", tag.name),
            Some(format!(
                "no version of {component} is derived at {}",
                tag.revision
            )),
        )),
    }
}

/// 134: git-cliff's configuration as the repository keeps it at `at`:
/// the first file of git-cliff's own names its tree holds, laid over
/// git-cliff's default as git-cliff lays it, a table into a table and
/// every other value in place of the default's, and the default alone
/// where the tree holds none. Never the working tree's, the host's own
/// configuration or its environment (18, 65). A file that is not
/// well formed is unreadable naming it at `at`, with the parser's words.
fn template(history: &History, at: &Revision) -> Answer<Config> {
    let unreadable =
        |what: &str, said: String| Not::unreadable(&Source::history(LOCAL), what, Some(said));
    let mut what = "git-cliff's default configuration".to_string();
    let mut config = EmbeddedConfig::get_config()
        .map_err(|e| e.to_string())
        .and_then(|text| text.parse::<Table>().map_err(|e| e.to_string()))
        .map_err(|said| unreadable(&what, said))?;
    for file in CONFIG_FILES {
        match history.file(at, file) {
            Ok(bytes) => {
                what = format!("{file} at {at}");
                let own = String::from_utf8(bytes)
                    .map_err(|e| e.to_string())
                    .and_then(|text| text.parse::<Table>().map_err(|e| e.to_string()))
                    .map_err(|said| unreadable(&what, said))?;
                lay(&mut config, own);
                break;
            }
            Err(Not::Absent(_)) => {}
            Err(not) => return Err(not),
        }
    }
    Value::Table(config)
        .try_into()
        .map_err(|e: toml::de::Error| unreadable(&what, e.to_string()))
}

/// `over` laid on `under`: a table into the table under it, every other
/// value in place of what is under it.
fn lay(under: &mut Table, over: Table) {
    for (key, value) in over {
        match (under.get_mut(&key), value) {
            (Some(Value::Table(below)), Value::Table(above)) => lay(below, above),
            (_, value) => {
                under.insert(key, value);
            }
        }
    }
}

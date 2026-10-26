//! A version's notes: the commits the version is derived from, rendered
//! by the version rule's own tool, git-cliff, in one fixed form (21, 134,
//! section 9; design D2). Nothing in the form reads the clock, the host
//! or the checkout's path, so the bytes are a function of the version,
//! the last release tag and the commits alone, and the same revision
//! renders the same notes on every host (18).

use git_cliff_core::changelog::Changelog;
use git_cliff_core::config::{ChangelogConfig, CommitParser, Config, GitConfig, RemoteConfig};
use git_cliff_core::release::Release;
use regex::Regex;
use tailrace_seams::{Answer, ComponentName, Not, Source, Version};

use crate::{cliff_commit, Commit, LOCAL};

/// The ecosystem of a version's notes, reserved: no output is packed in
/// it (design D3, D9).
pub const NOTES: &str = "notes";

/// The notes' one form: a heading naming the version, the last release
/// tag they follow where there is one, then each group that has commits,
/// one line per commit with its subject, `(breaking)` where it is one,
/// and its seven-character revision, in the order the history gives
/// them. The template is trimmed line by line before it is read.
const BODY: &str = r#"
## {{ version }}
{%- if previous.version %}

Since {{ previous.version }}.
{%- endif %}
{%- for group in commits | commit_groups(groups=commit_parsers_groups) %}

### {{ group.group }}
{% for commit in group.commits %}
- {{ commit.message | split(pat="\n") | first | trim }}{% if commit.breaking %} (breaking){% endif %} ({{ commit.id | truncate(length=7, end="") }})
{%- endfor %}
{%- endfor %}
"#;

/// The groups, in the order the notes print them: a feature, a fix, and
/// every other commit, conventional or not, so none is dropped.
const GROUPS: [(&str, &str); 3] = [
    ("Features", r"(?i)^feat(\([^)]*\))?!?:"),
    ("Fixes", r"(?i)^fix(\([^)]*\))?!?:"),
    ("Other", r"(?s).*"),
];

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

/// 134: the notes of `component` at `version`, rendered from `commits`,
/// the commits the version was derived from, since `last`, the last
/// release tag as the history spells it. git-cliff's words make it
/// unreadable naming the component and the version (58, 59).
pub fn render(
    component: &ComponentName,
    version: &Version,
    last: Option<&str>,
    commits: &[Commit],
) -> Answer<Rendered> {
    let failed = |said: String| {
        Not::unreadable(
            &Source::history(LOCAL),
            format!("the notes of {component} {version}"),
            Some(said),
        )
    };
    let mut parsers = Vec::new();
    for (group, message) in GROUPS {
        parsers.push(CommitParser {
            message: Some(Regex::new(message).map_err(|e| failed(e.to_string()))?),
            group: Some(group.to_string()),
            ..CommitParser::default()
        });
    }
    let config = Config {
        changelog: ChangelogConfig {
            body: BODY.to_string(),
            trim: true,
            render_always: true,
            format: false,
            ..ChangelogConfig::default()
        },
        git: GitConfig {
            conventional_commits: true,
            filter_unconventional: false,
            filter_commits: false,
            commit_parsers: parsers,
            ..GitConfig::default()
        },
        remote: RemoteConfig {
            offline: true,
            ..RemoteConfig::default()
        },
        bump: Default::default(),
    };
    // A previous release, with no version where there is no last tag,
    // keeps a release with no commits rendered rather than dropped.
    let previous = Release {
        version: last.map(str::to_string),
        ..Release::default()
    };
    let release = Release {
        version: Some(version.to_string()),
        commits: commits.iter().map(cliff_commit).collect(),
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

//! The four answers every seam operation gives: what was asked, or
//! absent, refused or unreadable, each naming the source that answered
//! (58, 59, rule core.19; design D1).

use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    ArtifactMeta, Branch, BuildId, ComponentName, Digest, Outcome, OutputName, RequiredClaim,
    Revision, SignerId,
};

/// What every seam operation answers: what was asked, or [`Not`].
pub type Answer<T> = Result<T, Not>;

/// Why an operation did not answer what was asked: absent, refused or
/// unreadable, and never anything else (58).
///
/// It does not implement `std::error::Error`, so no `?` folds it into an
/// error of another kind and loses which of the three it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Not {
    Absent(Absent),
    Refused(Refusal),
    Unreadable(Unreadable),
}

impl Not {
    /// `what` is not there.
    pub fn absent(source: &Source, what: impl Into<String>) -> Self {
        Not::Absent(Absent {
            source: source.clone(),
            what: what.into(),
        })
    }

    /// Refused for `why`, with the source's own words where it gave any.
    pub fn refused(source: &Source, why: Why, said: Option<String>) -> Self {
        Not::Refused(Refusal {
            source: source.clone(),
            why,
            said,
        })
    }

    /// `what` could not be read, with the source's own words where it
    /// gave any.
    pub fn unreadable(source: &Source, what: impl Into<String>, said: Option<String>) -> Self {
        Not::Unreadable(Unreadable {
            source: source.clone(),
            what: what.into(),
            said,
        })
    }

    /// The seam and profile that answered.
    pub fn source(&self) -> &Source {
        match self {
            Not::Absent(a) => &a.source,
            Not::Refused(r) => &r.source,
            Not::Unreadable(u) => &u.source,
        }
    }
}

/// What was asked is not there: a digest not held, a name naming nothing,
/// no identity at hand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Absent {
    pub source: Source,
    pub what: String,
}

/// 59: a refusal Part B names, with the refusing source's own words
/// verbatim beside it where it gave any.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refusal {
    pub source: Source,
    pub why: Why,
    pub said: Option<String>,
}

/// 58, 59: what could not be read, with the failing source's own words
/// verbatim where it gave any.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unreadable {
    pub source: Source,
    pub what: String,
    pub said: Option<String>,
}

/// The seam and the profile bound to it that answered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub seam: Seam,
    pub profile: String,
}

impl Source {
    pub fn new(seam: Seam, profile: impl Into<String>) -> Self {
        Source {
            seam,
            profile: profile.into(),
        }
    }

    /// The repository's history, read through git: `at` the remote that
    /// holds the shared history, or `local` for the repository's own.
    pub fn history(at: impl fmt::Display) -> Self {
        Source::new(Seam::History, format!("git, {at}"))
    }

    /// The host itself, by the file or directory of its own it read: the
    /// record of the versions it published, or its bindings (61, 92).
    pub fn host(path: impl AsRef<Path>) -> Self {
        Source::new(Seam::Host, path.as_ref().display().to_string())
    }
}

/// The eight seams of Part B, and beside them the repository's history,
/// which the core reads itself through git (section 9), and the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Seam {
    Store,
    Build,
    Landing,
    Signer,
    Policy,
    Projection,
    Target,
    Keeper,
    /// No seam of Part B: the repository's history, local or shared,
    /// whose files, commits and tags the core reads at a landed revision
    /// and where it claims a version's name (21, 22).
    History,
    /// No seam of Part B: the host running tailrace, whose own record of
    /// the versions it published and whose bindings the core reads (61,
    /// 92).
    Host,
}

/// The refusals the seams answer, each naming what its clause says it
/// names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Why {
    /// 70: the name is not at the digest it was expected to be at; `at`
    /// is where it is, `None` when it names nothing.
    NameAt { at: Option<Digest> },
    /// 120: the predecessor given is not the head of the channel's chain;
    /// `head` is the claim at the head, `None` when the chain is empty.
    NotHead { head: Option<Digest> },
    /// 74: the build could not make these outputs.
    Unmade { outputs: Vec<OutputName> },
    /// 80: none of the signers checked made the attestation.
    NoNamedSigner { checked: Vec<SignerId> },
    /// 105, 122: the chain holds these claims all naming one predecessor,
    /// `None` when each is the first of its channel.
    Fork {
        predecessor: Option<Digest>,
        claims: Vec<Digest>,
    },
    /// 14, 65, 77: `branch` has not landed `revision`, so nothing of the
    /// repository is read at it.
    NotLanded { branch: Branch, revision: Revision },
    /// 123: the components `first` and `second` declare one name or one
    /// prefix, `what` naming which and its text.
    DeclaredTwice {
        first: ComponentName,
        second: ComponentName,
        what: String,
    },
    /// 22: the version's name `name` is held by `other`, not by
    /// `revision`, where it was claimed.
    NameHeld {
        name: String,
        revision: Revision,
        other: Revision,
    },
    /// 16, 111: `output` came from `build`, whose outcome, by the build
    /// or by the landing's verdicts naming it, is `outcome`.
    NotPassed {
        output: OutputName,
        build: BuildId,
        outcome: Outcome,
    },
    /// 15: `output` came from `build`, which none of the landing's
    /// verdicts names; `carried` is every build identity they name.
    Unnamed {
        output: OutputName,
        build: BuildId,
        carried: Vec<BuildId>,
    },
    /// C.10: `component` declares `output` and names no projection to
    /// pack it.
    NoProjection {
        component: ComponentName,
        output: OutputName,
    },
    /// 2: the store named bytes `answered` whose digest is `expected`.
    OtherBytes { expected: Digest, answered: Digest },
    /// 18, 87: `artifact` packed again is other bytes or other metadata
    /// than the index holds: `first` the bytes' digest the index names,
    /// `None` when it names no such artifact, and `now` the digest of the
    /// bytes packed now.
    Differed {
        artifact: ArtifactMeta,
        first: Option<Digest>,
        now: Digest,
    },
    /// 61: the record already holds an entry at `path` with other
    /// content.
    KeptOther { path: String },
    /// 123: the repository declares several components, `declared`, and
    /// the question named none of them.
    WhichComponent { declared: Vec<ComponentName> },
    /// 47, 99: `version`, whose index is `index`, lacks `wanting`, the
    /// first claim in the policy's order that no claim attached to the
    /// index or the artifact satisfies; `checked` the signers it was
    /// checked against.
    Wanting {
        version: ArtifactMeta,
        index: Digest,
        wanting: RequiredClaim,
        checked: Vec<SignerId>,
    },
    /// A refusal that names nothing beyond the source's own words.
    Said,
}

impl fmt::Display for Seam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Seam::Store => "store",
            Seam::Build => "build",
            Seam::Landing => "landing",
            Seam::Signer => "signer",
            Seam::Policy => "policy",
            Seam::Projection => "projection",
            Seam::Target => "target",
            Seam::Keeper => "keeper",
            Seam::History => "history",
            Seam::Host => "host",
        })
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the {} ({})", self.seam, self.profile)
    }
}

fn list(items: impl IntoIterator<Item = impl fmt::Display>) -> String {
    let items: Vec<String> = items.into_iter().map(|i| i.to_string()).collect();
    if items.is_empty() {
        "none".into()
    } else {
        items.join(", ")
    }
}

fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Pass => "pass",
        Outcome::Fail => "fail",
    }
}

fn or_nothing(digest: &Option<Digest>) -> String {
    digest
        .as_ref()
        .map_or_else(|| "nothing".into(), Digest::to_string)
}

impl fmt::Display for Why {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Why::NameAt { at } => write!(f, "the name is at {}", or_nothing(at)),
            Why::NotHead { head: None } => write!(f, "the chain is empty, so nothing is its head"),
            Why::NotHead { head: Some(head) } => {
                write!(f, "the predecessor is not the head, which is {head}")
            }
            Why::Unmade { outputs } => write!(f, "the build could not make {}", list(outputs)),
            Why::NoNamedSigner { checked } => {
                write!(
                    f,
                    "no signer it was checked against made it: {}",
                    list(checked)
                )
            }
            Why::Fork {
                predecessor,
                claims,
            } => write!(
                f,
                "the chain forks: {} each name {} as their predecessor",
                list(claims),
                or_nothing(predecessor)
            ),
            Why::NotLanded { branch, revision } => {
                write!(f, "{branch} has not landed {revision}")
            }
            Why::DeclaredTwice {
                first,
                second,
                what,
            } => write!(f, "the components {first} and {second} declare one {what}"),
            Why::NameHeld {
                name,
                revision,
                other,
            } => write!(f, "{name} is held by {other}, not by {revision}"),
            Why::NotPassed {
                output,
                build,
                outcome,
            } => write!(
                f,
                "{output} came from build {build}, whose outcome is {}",
                outcome_name(*outcome)
            ),
            Why::Unnamed {
                output,
                build,
                carried,
            } => write!(
                f,
                "{output} came from build {build}, which no verdict names; the verdicts name {}",
                list(carried)
            ),
            Why::NoProjection { component, output } => {
                write!(f, "{component} names no projection for {output}")
            }
            Why::OtherBytes { expected, answered } => write!(
                f,
                "the store named the bytes {answered}, and their digest is {expected}"
            ),
            Why::Differed {
                artifact,
                first,
                now,
            } => write!(
                f,
                "{} {} {} packs to {now} and the index names {}",
                artifact.ecosystem(),
                artifact.name(),
                artifact.version(),
                or_nothing(first)
            ),
            Why::KeptOther { path } => write!(f, "{path} already holds another entry"),
            Why::WhichComponent { declared } => write!(
                f,
                "several components are declared, {}, and none was named",
                list(declared)
            ),
            Why::Wanting {
                version,
                index,
                wanting,
                checked,
            } => {
                write!(
                    f,
                    "{} {} {} at {index} wants a claim of {}",
                    version.ecosystem(),
                    version.name(),
                    version.version(),
                    wanting.kind
                )?;
                if let Some(check) = &wanting.check {
                    write!(f, " for the check {check}")?;
                }
                write!(
                    f,
                    ", made by none of the signers checked: {}",
                    list(checked)
                )
            }
            Why::Said => write!(f, "it refused"),
        }
    }
}

fn said(f: &mut fmt::Formatter<'_>, said: &Option<String>) -> fmt::Result {
    match said {
        Some(words) => write!(f, "; it said: {words}"),
        None => Ok(()),
    }
}

impl fmt::Display for Not {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Not::Absent(a) => write!(f, "{} holds no {}", a.source, a.what),
            Not::Refused(r) => {
                write!(f, "{} refused: {}", r.source, r.why)?;
                said(f, &r.said)
            }
            Not::Unreadable(u) => {
                write!(f, "{} could not read {}", u.source, u.what)?;
                said(f, &u.said)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: &str = "denied: tag \"v1\" is immutable\n\t\u{fc} \u{1f6ab} \\0 ";

    fn store() -> Source {
        Source::new(Seam::Store, "fake")
    }

    fn round_trip(not: &Not) -> Not {
        serde_json::from_str(&serde_json::to_string(not).unwrap()).unwrap()
    }

    #[test]
    fn each_keeps_its_source_and_words_through_json() {
        let at = Digest::of(b"elsewhere");
        let nots = [
            Not::absent(&store(), at.to_string()),
            Not::refused(
                &store(),
                Why::NameAt {
                    at: Some(at.clone()),
                },
                Some(WORDS.into()),
            ),
            Not::unreadable(
                &Source::new(Seam::Target, "a target"),
                "the environment",
                Some(WORDS.into()),
            ),
        ];
        for not in &nots {
            let back = round_trip(not);
            assert_eq!(&back, not);
            assert_eq!(back.source(), not.source());
        }
        let Not::Refused(r) = round_trip(&nots[1]) else {
            panic!("refused read back as another answer")
        };
        assert_eq!(r.said.unwrap().as_bytes(), WORDS.as_bytes());
        let Not::Unreadable(u) = round_trip(&nots[2]) else {
            panic!("unreadable read back as another answer")
        };
        assert_eq!(u.said.unwrap().as_bytes(), WORDS.as_bytes());
        assert_eq!(u.source, Source::new(Seam::Target, "a target"));
    }

    #[test]
    fn the_three_are_told_apart_in_json() {
        let json = |not: Not| serde_json::to_value(not).unwrap();
        assert!(json(Not::absent(&store(), "x")).get("absent").is_some());
        assert!(json(Not::refused(&store(), Why::Said, None))
            .get("refused")
            .is_some());
        assert!(json(Not::unreadable(&store(), "x", None))
            .get("unreadable")
            .is_some());
    }

    #[test]
    fn a_refusal_carries_the_words_beside_the_reason() {
        let at = Digest::of(b"elsewhere");
        let not = Not::refused(
            &store(),
            Why::NameAt {
                at: Some(at.clone()),
            },
            Some(WORDS.into()),
        );
        let Not::Refused(r) = &not else {
            unreachable!()
        };
        assert_eq!(
            r.why,
            Why::NameAt {
                at: Some(at.clone())
            }
        );
        assert_eq!(r.said.as_deref(), Some(WORDS));
        let text = not.to_string();
        assert!(text.contains(&format!("the name is at {at}")), "{text}");
        assert!(text.ends_with(&format!("it said: {WORDS}")), "{text}");
        assert!(text.starts_with("the store (fake) refused"), "{text}");
    }

    #[test]
    fn each_refusal_names_what_its_clause_says() {
        let head = Digest::of(b"head");
        let cases = [
            (
                Why::NameAt { at: None },
                "the name is at nothing".to_string(),
            ),
            (
                Why::NotHead {
                    head: Some(head.clone()),
                },
                format!("which is {head}"),
            ),
            (Why::NotHead { head: None }, "the chain is empty".into()),
            (
                Why::Unmade {
                    outputs: vec![OutputName::new("site")],
                },
                "could not make site".into(),
            ),
            (
                Why::NoNamedSigner {
                    checked: vec![SignerId::new("gate"), SignerId::new("host")],
                },
                "gate, host".into(),
            ),
            (
                Why::NotLanded {
                    branch: Branch::new("main"),
                    revision: Revision::new("4f2a9c1"),
                },
                "main has not landed 4f2a9c1".into(),
            ),
            (
                Why::DeclaredTwice {
                    first: ComponentName::new("lib"),
                    second: ComponentName::new("web"),
                    what: "prefix \"v\"".into(),
                },
                "lib and web declare one prefix \"v\"".into(),
            ),
            (
                Why::NameHeld {
                    name: "lib-v1.3.0".into(),
                    revision: Revision::new("4f2a9c1"),
                    other: Revision::new("77b0e3d"),
                },
                "lib-v1.3.0 is held by 77b0e3d, not by 4f2a9c1".into(),
            ),
            (
                Why::Fork {
                    predecessor: Some(head.clone()),
                    claims: vec![Digest::of(b"a"), Digest::of(b"b")],
                },
                format!(
                    "{}, {} each name {head}",
                    Digest::of(b"a"),
                    Digest::of(b"b")
                ),
            ),
        ];
        for (why, names) in cases {
            let text = why.to_string();
            assert!(text.contains(&names), "{text} does not name {names}");
            let not = Not::refused(&store(), why.clone(), None);
            assert_eq!(round_trip(&not), not);
        }
    }

    #[test]
    fn the_history_answers_as_a_source_with_gits_words() {
        const GIT: &str = " ! [rejected]        lib-v1.3.0 -> lib-v1.3.0 (already exists)\n\
                           error: failed to push some refs to '/srv/shared.git'\n\
                           hint: Updates were rejected because the tag already exists in the remote.\n";
        let origin = Source::history("origin");
        assert_eq!(origin.seam, Seam::History);
        assert_eq!(origin.to_string(), "the history (git, origin)");
        assert_eq!(Source::history("local").profile, "git, local");
        let nots = [
            Not::refused(
                &Source::new(Seam::Landing, "fake"),
                Why::NotLanded {
                    branch: Branch::new("main"),
                    revision: Revision::new("4f2a9c1"),
                },
                None,
            ),
            Not::refused(
                &Source::history("local"),
                Why::DeclaredTwice {
                    first: ComponentName::new("lib"),
                    second: ComponentName::new("web"),
                    what: "prefix \"v\"".into(),
                },
                None,
            ),
            Not::refused(
                &origin,
                Why::NameHeld {
                    name: "lib-v1.3.0".into(),
                    revision: Revision::new("4f2a9c1"),
                    other: Revision::new("77b0e3d"),
                },
                Some(GIT.into()),
            ),
            Not::unreadable(&origin, "the tags of origin", Some(GIT.into())),
        ];
        for not in &nots {
            let back = round_trip(not);
            assert_eq!(&back, not);
            assert_eq!(back.source(), not.source());
        }
        let Not::Refused(r) = round_trip(&nots[2]) else {
            panic!("refused read back as another answer")
        };
        assert_eq!(r.source, Source::history("origin"));
        assert_eq!(r.said.unwrap().as_bytes(), GIT.as_bytes());
        let Not::Unreadable(u) = round_trip(&nots[3]) else {
            panic!("unreadable read back as another answer")
        };
        assert_eq!(u.said.unwrap().as_bytes(), GIT.as_bytes());
        let json = serde_json::to_value(&nots[2]).unwrap();
        assert_eq!(json["refused"]["source"]["seam"], "history", "{json}");
        assert_eq!(json["refused"]["why"]["name_held"]["other"], "77b0e3d");
    }

    #[test]
    fn the_host_answers_as_a_source_of_its_own() {
        let record = Source::host("/srv/repo.git/tailrace/published");
        assert_eq!(record.seam, Seam::Host);
        assert_eq!(record.profile, "/srv/repo.git/tailrace/published");
        assert_eq!(
            record.to_string(),
            "the host (/srv/repo.git/tailrace/published)"
        );
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["seam"], "host");
    }

    #[test]
    fn a_version_wanting_a_claim_names_it_and_the_signers_checked() {
        let version = ArtifactMeta::new(
            crate::Ecosystem::new("eco"),
            crate::EcosystemName::new("pkg"),
            crate::Version::new("1.3.0"),
        )
        .unwrap();
        let index = Digest::of(b"index");
        let named = |id: &str| crate::NamedSigner {
            id: SignerId::new(id),
            verified_by: format!("{id}'s key"),
            kinds: vec![crate::ClaimKind::Verdict],
            checks: vec![crate::Check::new("test")],
        };
        let wanting = RequiredClaim {
            kind: crate::ClaimKind::Verdict,
            check: Some(crate::Check::new("test")),
            signers: vec![named("landing"), named("gate")],
        };
        let why = Why::Wanting {
            version,
            index: index.clone(),
            wanting,
            checked: vec![SignerId::new("landing"), SignerId::new("gate")],
        };
        assert_eq!(
            why.to_string(),
            format!(
                "eco pkg 1.3.0 at {index} wants a claim of verdict for the check test, \
                 made by none of the signers checked: landing, gate"
            )
        );
        let not = Not::refused(&Source::new(Seam::Policy, "bound"), why.clone(), None);
        assert_eq!(round_trip(&not), not);
        let json = serde_json::to_value(&not).unwrap();
        let wanting = &json["refused"]["why"]["wanting"];
        assert_eq!(wanting["version"]["version"], "1.3.0", "{json}");
        assert_eq!(wanting["index"], index.to_string(), "{json}");
        assert_eq!(wanting["wanting"]["kind"], "verdict", "{json}");
        assert_eq!(wanting["checked"], serde_json::json!(["landing", "gate"]));

        let Why::Wanting { version, .. } = why else {
            unreachable!()
        };
        let none = Why::Wanting {
            version,
            index: index.clone(),
            wanting: RequiredClaim {
                kind: crate::ClaimKind::Acceptance,
                check: None,
                signers: vec![],
            },
            checked: vec![],
        };
        assert_eq!(
            none.to_string(),
            format!(
                "eco pkg 1.3.0 at {index} wants a claim of acceptance, \
                 made by none of the signers checked: none"
            )
        );
    }

    #[test]
    fn the_refusals_of_publish_verify_and_status_keep_their_words_through_json() {
        let meta = ArtifactMeta::new(
            crate::Ecosystem::new("eco"),
            crate::EcosystemName::new("pkg"),
            crate::Version::new("1.3.0"),
        )
        .unwrap();
        let first = Digest::of(b"first");
        let now = Digest::of(b"now");
        let cases = [
            (
                store(),
                Why::NotPassed {
                    output: OutputName::new("site"),
                    build: BuildId::new("b7e1"),
                    outcome: Outcome::Fail,
                },
                "site came from build b7e1, whose outcome is fail".to_string(),
            ),
            (
                Source::new(Seam::Landing, "fake"),
                Why::Unnamed {
                    output: OutputName::new("site"),
                    build: BuildId::new("b7e1"),
                    carried: vec![BuildId::new("c001"), BuildId::new("c002")],
                },
                "which no verdict names; the verdicts name c001, c002".into(),
            ),
            (
                Source::host("/srv/repo/.config/tailrace.toml"),
                Why::NoProjection {
                    component: ComponentName::new("lib"),
                    output: OutputName::new("lib-pack"),
                },
                "lib names no projection for lib-pack".into(),
            ),
            (
                store(),
                Why::OtherBytes {
                    expected: first.clone(),
                    answered: now.clone(),
                },
                format!("named the bytes {now}, and their digest is {first}"),
            ),
            (
                Source::new(Seam::Projection, "fake"),
                Why::Differed {
                    artifact: meta.clone(),
                    first: Some(first.clone()),
                    now: now.clone(),
                },
                format!("eco pkg 1.3.0 packs to {now} and the index names {first}"),
            ),
            (
                Source::new(Seam::Projection, "fake"),
                Why::Differed {
                    artifact: meta,
                    first: None,
                    now: now.clone(),
                },
                format!("packs to {now} and the index names nothing"),
            ),
            (
                Source::host("/srv/repo.git/tailrace/published"),
                Why::KeptOther {
                    path: "/srv/repo.git/tailrace/published/lib/lib-v1.3.0.json".into(),
                },
                "lib/lib-v1.3.0.json already holds another entry".into(),
            ),
            (
                Source::host("/srv/repo/.config/tailrace.toml"),
                Why::WhichComponent {
                    declared: vec![ComponentName::new("lib"), ComponentName::new("web")],
                },
                "several components are declared, lib, web, and none was named".into(),
            ),
        ];
        for (source, why, names) in cases {
            let text = why.to_string();
            assert!(text.contains(&names), "{text} does not name {names}");
            let not = Not::refused(&source, why, Some(WORDS.into()));
            let back = round_trip(&not);
            assert_eq!(back, not);
            assert_eq!(back.source(), &source);
            let Not::Refused(r) = back else {
                panic!("refused read back as another answer")
            };
            assert_eq!(r.said.unwrap().as_bytes(), WORDS.as_bytes());
        }
    }
}

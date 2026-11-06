//! `timber apply <environment>`: one environment converged on its
//! channel's head, the claims the channel's policy requires of the head's
//! index judged before its target is handed the version, and the report
//! of what the host observed it run after signed as the host and
//! attached beside the index (11, 40, 42, 43, 88 to 91, 136; design D5,
//! D6). It is bound as status is, with the host's signer to sign that
//! report, the store opened and never made and every target profile built
//! beside it, the repository's environments picking among them (92);
//! timber keeps no credential that reaches a target (45, rule core.21).
//!
//! After the core's apply has answered a report claim, the command tells
//! the host's gate of it as the publish command tells it of a version:
//! where the bindings file names this host's own steps as what tells it,
//! or names no `[gate]` (118), it asks the core to move each channel that
//! comes after the environment, in the same run, and prints each
//! channel's answer beside the apply's; where the file names a job, it
//! says so and asks the gate nothing (95, 130; design D9). The core's
//! apply reaches nothing of the gate: this step, the host's binding of
//! what tells it, calls one and then the other (rule core.52). Its exit
//! is the worst of the apply's, the report's and each channel's, a
//! channel held counting as refused (58).

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Args;
use serde_json::{json, Value};
use timber_core::after::{follow, After, FollowAsk, Follows};
use timber_core::{answer_json, apply, Applied, ApplyAsk, History};
use timber_seams::{Answer, Branch, ComponentName, EnvironmentName, Not};

use crate::bindings::Teller;
use crate::commands::promote;
use crate::{render, unsigned, Binding};

#[derive(Debug, Args)]
pub struct ApplyArgs {
    /// The environment to converge, as the policy file declares it.
    pub environment: String,
    /// The component the environment is of; needed only where more than
    /// one is declared.
    #[arg(long)]
    pub component: Option<String>,
    /// The branch whose landed head holds the policy that declares the
    /// environment.
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Print the answer as one JSON document.
    #[arg(long)]
    pub json: bool,
}

/// The core's apply answer and, where it answered a report claim, what
/// tells the gate of it; none where the report is not one, so no gate is
/// asked (design D9).
struct Converged {
    applied: Applied,
    after: Option<Gate>,
}

/// What became of telling the gate of an environment's report.
#[derive(Clone, Debug, PartialEq)]
enum Gate {
    /// This host's apply told its own gate, and this is its answer for
    /// each channel that comes after the environment.
    Told(Answer<Follows>),
    /// A job outside timber tells the gate, in the file's words; the
    /// gate was not asked, and no channel moved.
    Job(String),
}

impl Gate {
    /// 57: the apply's `after` member, in the core's one form.
    fn json(&self) -> Value {
        match self {
            Gate::Told(answer) => {
                let mut value = answer_json(answer);
                value["told_by"] = json!("apply");
                value
            }
            Gate::Job(words) => json!({ "told_by": "job", "job": words }),
        }
    }

    /// The exit code the gate's part adds: each channel's by its kind, a
    /// channel held counting as refused; a job adds none, the apply
    /// having done what it was asked.
    fn code(&self) -> i32 {
        match self {
            Gate::Told(Err(not)) => render::code(not),
            Gate::Told(Ok(follows)) => {
                render::worst(follows.channels.iter().map(|(_, after)| match after {
                    Ok(After::Held { .. }) => render::REFUSED,
                    other => render::exit(other),
                }))
            }
            Gate::Job(_) => render::OK,
        }
    }
}

pub fn run(args: &ApplyArgs, binding: &Answer<Binding>, at: &Path, out: &mut dyn Write) -> i32 {
    let answer = ask(args, binding, at);
    let _ = if args.json {
        json(out, &answer)
    } else {
        text(out, &answer)
    };
    match &answer {
        Ok(converged) => render::worst(
            [render::exit(&converged.applied.report)]
                .into_iter()
                .chain(converged.after.iter().map(Gate::code)),
        ),
        Err(not) => render::code(not),
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn ask(args: &ApplyArgs, binding: &Answer<Binding>, at: &Path) -> Answer<Converged> {
    let binding = binding.as_ref().map_err(Not::clone)?;
    let signer = binding.signer.clone().ok_or_else(unsigned)?;
    let history = History::open(at)?;
    let ask = ApplyAsk {
        branch: Branch::new(&args.branch),
        component: args.component.as_ref().map(ComponentName::new),
        environment: EnvironmentName::new(&args.environment),
        signer,
        at: now(),
    };
    let applied = apply(&binding.seams, &history, &ask)?;
    // Only once the apply has answered a report claim: the gate, told of
    // it, where this host's own steps are what tell it (130).
    let after = applied.report.is_ok().then(|| match &binding.gate {
        Teller::Publish => Gate::Told(follow(
            &binding.seams,
            &history,
            &FollowAsk {
                branch: ask.branch.clone(),
                component: Some(applied.component.clone()),
                environment: applied.environment.clone(),
                signer: ask.signer.clone(),
                at: now(),
            },
        )),
        Teller::Job(words) => Gate::Job(words.clone()),
    });
    Ok(Converged { applied, after })
}

/// The answer as one JSON document: the apply's, in the core's one form,
/// with its `after` member where the gate was told (57).
fn json(out: &mut dyn Write, answer: &Answer<Converged>) -> io::Result<()> {
    let value = match answer {
        Ok(converged) => {
            let mut value = answer_json::<&Applied>(&Ok(&converged.applied));
            if let Some(gate) = &converged.after {
                value["value"]["after"] = gate.json();
            }
            value
        }
        Err(not) => answer_json::<()>(&Err(not.clone())),
    };
    serde_json::to_writer_pretty(&mut *out, &value)?;
    writeln!(out)
}

/// One part of the answer under its label: its value, or its own answer
/// on the lines beneath with the source's words unchanged (59, 119).
fn part<T>(
    out: &mut dyn Write,
    label: &str,
    answer: &Answer<T>,
    shown: impl Fn(&T) -> String,
) -> io::Result<()> {
    match answer {
        Ok(value) => writeln!(out, "  {label}: {}", shown(value)),
        Err(not) => {
            writeln!(out, "  {label}:")?;
            render::not(out, "    ", not)
        }
    }
}

/// The environment, its component and channel, the version and index
/// its chain's head names and whether it was applied; then what the
/// environment ran before and what the target's report says it runs
/// after, each its digest or its unread answer (43, 91, 119); and the
/// report claim's digest, or the answer given in its place (design D6);
/// and where the gate was told, what it answered for each channel that
/// comes after the environment, or the job that tells it (design D9).
fn text(out: &mut dyn Write, answer: &Answer<Converged>) -> io::Result<()> {
    let (applied, after) = match answer {
        Ok(converged) => (&converged.applied, &converged.after),
        Err(not) => return render::not(out, "", not),
    };
    let done = if applied.applied {
        "applied"
    } else {
        "running it already, nothing applied"
    };
    writeln!(
        out,
        "{} of {} follows {}: {} ({}) {done}",
        applied.environment, applied.component, applied.channel, applied.version, applied.index
    )?;
    part(out, "before", &applied.current, ToString::to_string)?;
    part(out, "runs", &applied.running, |running| {
        format!("{}, converged {}", running.digest, running.converged_at)
    })?;
    part(out, "report", &applied.report, ToString::to_string)?;
    match after {
        Some(gate) => self::gate(out, &applied.environment, gate),
        None => Ok(()),
    }
}

/// The gate's part, below the report: each channel that comes after the
/// environment with its answer, a move as the promote command prints it,
/// or the job that tells the gate.
fn gate(out: &mut dyn Write, environment: &EnvironmentName, gate: &Gate) -> io::Result<()> {
    let follows = match gate {
        Gate::Job(words) => {
            return writeln!(
                out,
                "  after: the gate is told by \"{words}\", and no channel moved"
            )
        }
        Gate::Told(Err(not)) => {
            writeln!(out, "  after:")?;
            return render::not(out, "    ", not);
        }
        Gate::Told(Ok(follows)) => follows,
    };
    if follows.channels.is_empty() {
        return writeln!(out, "  after: no channel comes after {environment}");
    }
    writeln!(out, "  after:")?;
    for (channel, after) in &follows.channels {
        match after {
            Ok(After::Promoted(promoted)) => {
                let mut moved = Vec::new();
                promote::text(&mut moved, &Ok(promoted.clone()))?;
                for line in String::from_utf8_lossy(&moved).lines() {
                    writeln!(out, "    {line}")?;
                }
            }
            Ok(After::Held { reports }) => {
                writeln!(
                    out,
                    "    {channel} held, its environments not at one index:"
                )?;
                for (environment, newest) in reports {
                    match newest {
                        Ok(n) => writeln!(
                            out,
                            "      {environment} last reported {} ({})",
                            n.report.version, n.report.index
                        )?,
                        Err(not) => {
                            writeln!(out, "      {environment}:")?;
                            render::not(out, "        ", not)?;
                        }
                    }
                }
            }
            Err(not) => {
                writeln!(out, "    {channel}:")?;
                render::not(out, "      ", not)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    use clap::Parser;
    use serde_json::{json, Value};
    use tempfile::TempDir;
    use timber_core::answer_json;
    use timber_fakes::{FakeSigner, FakeTarget, Fakes, TargetOp, NO_MARKER};
    use timber_seams::{
        Artifact, ArtifactMeta, AskedOf, Audience, ChannelId, ChannelName, Claim, ClaimKind,
        DeclaredEnvironment, Digest, Ecosystem, EcosystemName, IndexMeta, Keeper, Manifest, Moved,
        NamedSigner, Promotion, Required, RequiredClaim, Revision, Running, Satisfied, Seam,
        Signer, SignerId, Source, Store, TargetProfile, TargetTable, Version,
    };

    use super::*;
    use crate::bindings::{self, For, Teller};
    use crate::{profiles, Cli};

    /// The repository's one component.
    const COMPONENT: &str = "swancloud-srv";

    /// The policy file declaring it, its one output packed as a file.
    const POLICY: &str = "[[component]]\nname = \"swancloud-srv\"\noutputs = \
                          [\"swancloud-srv\"]\npaths = [\"machines/**\"]\nprefix = \
                          \"swancloud-srv-v\"\nprojections = { swancloud-srv = \"file\" }\n";

    /// The gate, whose acceptance every channel here requires.
    const GATE: &str = "gate";

    /// The host, which signed the promotions.
    const HOST: &str = "host";

    /// When the fake target's apply converges.
    const CONVERGED: &str = "2026-11-05T09:00:00Z";

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "A. Uthor")
            .env("GIT_AUTHOR_EMAIL", "author@example.com")
            .env("GIT_COMMITTER_NAME", "A. Uthor")
            .env("GIT_COMMITTER_EMAIL", "author@example.com")
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout)
            .unwrap()
            .trim_end()
            .to_string()
    }

    fn of(channel: &str) -> ChannelId {
        ChannelId::new(
            Some(ComponentName::new(COMPONENT)),
            ChannelName::new(channel),
        )
    }

    fn named(id: &str, kinds: &[ClaimKind]) -> NamedSigner {
        NamedSigner {
            id: SignerId::new(id),
            verified_by: format!("{id}'s key"),
            kinds: kinds.to_vec(),
            checks: vec![],
        }
    }

    /// A channel's policy requiring a claim of each `(kind, signer)`.
    fn requiring(claims: &[(ClaimKind, &str)]) -> Required {
        Required::Channel {
            claims: claims
                .iter()
                .map(|(kind, signer)| RequiredClaim {
                    kind: *kind,
                    check: None,
                    environment: None,
                    signers: vec![named(signer, &[*kind])],
                    asked_of: AskedOf::Every,
                })
                .collect(),
            bound_to: None,
            promoted_by: vec![],
            approvers: vec![],
        }
    }

    /// Every file under `dir` with its bytes.
    fn files(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut found = BTreeMap::new();
        let mut walk = vec![dir.to_path_buf()];
        while let Some(at) = walk.pop() {
            for entry in fs::read_dir(&at).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk.push(path);
                } else {
                    found.insert(path.clone(), fs::read(&path).unwrap());
                }
            }
        }
        found
    }

    /// A repository on `main` declaring `swancloud-srv`, landed; the fakes
    /// holding its 0.3.0 and 0.4.0, each index accepted by the gate; `beta`
    /// and `stable` each promoted to 0.4.0 by the host; and the fake target
    /// bound under `fake`, `cache` running 0.3.0 on it.
    struct Scratch {
        dir: TempDir,
        repo: PathBuf,
        head: Revision,
        fakes: Fakes,
        target: FakeTarget,
        first: Digest,
        second: Digest,
    }

    impl Scratch {
        fn new() -> Scratch {
            let dir = TempDir::new().unwrap();
            let repo = dir.path().join("repo");
            git(dir.path(), &["init", "-q", "-b", "main", "repo"]);
            fs::create_dir_all(repo.join(".config")).unwrap();
            fs::write(repo.join(".config/timber.toml"), POLICY).unwrap();
            git(&repo, &["add", "--", ".config/timber.toml"]);
            git(&repo, &["commit", "-q", "-m", "feat: the machine"]);
            let head = Revision::new(git(&repo, &["rev-parse", "HEAD"]));
            let mut fakes = Fakes::new();
            fakes.landing.land(&Branch::new("main"), &head);
            fakes.signer.hold(&SignerId::new(HOST));
            let target = fakes.target(&TargetProfile::new("fake"));
            let mut scratch = Scratch {
                dir,
                repo,
                head,
                fakes,
                target,
                first: Digest::of(b"none yet"),
                second: Digest::of(b"none yet"),
            };
            scratch.first = scratch.published("0.3.0");
            scratch.second = scratch.published("0.4.0");
            for channel in ["beta", "stable"] {
                scratch.promoted(channel);
            }
            scratch.target.runs(
                &EnvironmentName::new("cache"),
                Running {
                    digest: scratch.first.clone(),
                    converged_at: "2026-01-01T00:00:00Z".into(),
                },
            );
            scratch.target.at(CONVERGED);
            scratch
        }

        /// `swancloud-srv` at `version` put in the fake store, its index
        /// carrying the gate's acceptance; the index's digest.
        fn published(&self, version: &str) -> Digest {
            let store = &self.fakes.store;
            let version = Version::new(version);
            let meta = ArtifactMeta::new(
                Ecosystem::new("eco"),
                EcosystemName::new(COMPONENT),
                version.clone(),
            )
            .unwrap();
            let layer = store
                .put_bytes(format!("{COMPONENT} {version}").as_bytes())
                .unwrap()
                .digest;
            let digest = store
                .put_artifact(&Manifest::Artifact {
                    meta: meta.clone(),
                    layers: vec![layer],
                })
                .unwrap()
                .digest;
            let index = IndexMeta::new(
                ComponentName::new(COMPONENT),
                version,
                vec![Artifact { digest, meta }],
            )
            .unwrap()
            .published_from(self.head.clone(), None);
            let index = store.put_artifact(&Manifest::Index(index)).unwrap().digest;
            let gate = SignerId::new(GATE);
            let signer = FakeSigner::new();
            signer.hold(&gate);
            let claim = Claim {
                subject: index.clone(),
                kind: ClaimKind::Acceptance,
                revision: self.head.clone(),
                build: None,
                body: json!(null),
            };
            store
                .attach(&index, &signer.sign(&claim, &gate).unwrap())
                .unwrap();
            index
        }

        /// `channel` promoted by the host to 0.4.0, its first promotion,
        /// its policy requiring the gate's acceptance.
        fn promoted(&self, channel: &str) {
            let promotion = Promotion {
                channel: ChannelName::new(channel),
                component: ComponentName::new(COMPONENT),
                to: Moved {
                    version: Version::new("0.4.0"),
                    index: self.second.clone(),
                },
                from: None,
                predecessor: None,
                asked_by: SignerId::new(HOST),
                policy: Satisfied {
                    revision: self.head.clone(),
                    claims: vec![],
                },
                at: 1_793_000_000,
            };
            let envelope = self
                .fakes
                .signer
                .sign(&promotion.claim(self.head.clone()), &SignerId::new(HOST))
                .unwrap();
            self.fakes
                .keeper
                .append(&of(channel), &envelope, None)
                .unwrap();
            self.fakes.policy.require(
                Audience::Channel(of(channel)),
                requiring(&[(ClaimKind::Acceptance, GATE)]),
            );
        }

        /// `environment` declared at the head, following `channel`, its
        /// target the profile `profile` with `settings`.
        fn declares(&self, environment: &str, channel: &str, profile: &str, settings: Value) {
            self.fakes.policy.declares_at(
                &self.head,
                DeclaredEnvironment {
                    name: EnvironmentName::new(environment),
                    channel: of(channel),
                    waits: false,
                    approvers: vec![],
                    target: Some(TargetTable {
                        profile: TargetProfile::new(profile),
                        settings,
                    }),
                },
            );
        }

        fn binding(&self) -> Binding {
            Binding {
                seams: self.fakes.seams(),
                store: Source::new(Seam::Store, "layout /srv/timber/layout"),
                signer: Some(SignerId::new(HOST)),
                gate: Teller::Job("the gate's job".into()),
                trust: vec![],
            }
        }

        /// `args` asked through `run` over `binding`: the exit code and
        /// what it printed.
        fn run_over(&self, binding: &Answer<Binding>, args: &[&str]) -> (i32, String) {
            let cli = Cli::try_parse_from(std::iter::once("timber").chain(args.iter().copied()))
                .expect("the arguments parse");
            let mut out = Vec::new();
            let code = crate::run(&cli, binding, &self.repo, &mut out);
            (code, String::from_utf8(out).unwrap())
        }

        /// `args` asked through `run` over the fakes.
        fn run(&self, args: &[&str]) -> (i32, String) {
            self.run_over(&Ok(self.binding()), args)
        }

        fn ops(&self) -> Vec<TargetOp> {
            self.target.calls().into_iter().map(|c| c.op).collect()
        }

        /// The digests of the report claims attached to `index`, in the
        /// order attached.
        fn reports_on(&self, index: &Digest) -> Vec<Digest> {
            self.fakes
                .store
                .claims_of(index)
                .unwrap()
                .iter()
                .filter(|e| Claim::from_envelope(e).is_some_and(|c| c.kind == ClaimKind::Report))
                .map(|e| e.digest())
                .collect()
        }
    }

    /// The source a JSON answer that is not `ok` names.
    fn source_of(printed: &Value) -> Value {
        let answer = printed["answer"].as_str().expect("an answer");
        assert_ne!(answer, "ok", "{printed}");
        printed[answer]["source"].clone()
    }

    // Requirement: Apply converges one environment

    #[test]
    fn an_environment_applied_prints_it_its_channel_the_version_its_index_and_the_report() {
        let s = Scratch::new();
        s.declares("cache", "beta", "fake", json!({}));
        let (code, printed) = s.run(&["apply", "cache", "--component", COMPONENT]);
        assert_eq!(code, render::OK, "{printed}");
        let lines: Vec<&str> = printed.lines().collect();
        assert_eq!(
            lines,
            [
                format!(
                    "cache of swancloud-srv follows beta: 0.4.0 ({}) applied",
                    s.second
                ),
                format!("  before: {}", s.first),
                format!("  runs: {}, converged {CONVERGED}", s.second),
                format!("  report: {}", s.reports_on(&s.second)[0]),
                "  after: the gate is told by \"the gate's job\", and no channel moved".into(),
            ],
            "{printed}"
        );
        assert_eq!(
            s.ops(),
            [
                TargetOp::Desired,
                TargetOp::Current,
                TargetOp::Apply,
                TargetOp::Report
            ]
        );
    }

    #[test]
    fn json_prints_the_cores_answer_value_for_value() {
        let s = Scratch::new();
        s.declares("cache", "beta", "fake", json!({}));
        let (code, printed) = s.run(&["apply", "cache", "--component", COMPONENT, "--json"]);
        assert_eq!(code, render::OK, "{printed}");
        let mut printed: Value = serde_json::from_str(&printed).unwrap();
        // The one thing beside the core's answer: the job that tells the
        // gate of the report.
        let after = printed["value"]
            .as_object_mut()
            .unwrap()
            .remove("after")
            .expect("the apply names what tells the gate");
        assert_eq!(after, json!({ "told_by": "job", "job": "the gate's job" }));
        let core: Answer<Applied> = Ok(Applied {
            environment: EnvironmentName::new("cache"),
            component: ComponentName::new(COMPONENT),
            channel: ChannelName::new("beta"),
            version: Version::new("0.4.0"),
            index: s.second.clone(),
            applied: true,
            current: Ok(s.first.clone()),
            running: Ok(Running {
                digest: s.second.clone(),
                converged_at: CONVERGED.into(),
            }),
            report: Ok(s.reports_on(&s.second)[0].clone()),
        });
        assert_eq!(printed, answer_json(&core));
    }

    #[test]
    fn a_target_that_keeps_no_marker_prints_its_unread_reason_asks_no_gate_and_exits_unreadable() {
        let s = Scratch::new();
        s.declares("cache", "beta", "fake", json!({}));
        s.target.holds_no_marker();
        let (code, printed) = s.run(&["apply", "cache"]);
        assert_eq!(code, render::UNREADABLE, "{printed}");
        let lines: Vec<&str> = printed.lines().collect();
        assert_eq!(
            lines,
            [
                format!(
                    "cache of swancloud-srv follows beta: 0.4.0 ({}) applied",
                    s.second
                ),
                "  before:".to_string(),
                "    the target (fake) could not read what cache runs".to_string(),
                NO_MARKER.to_string(),
                "  runs:".to_string(),
                "    the target (fake) could not read what cache runs".to_string(),
                NO_MARKER.to_string(),
                "  report:".to_string(),
                "    the target (fake) could not read what cache runs".to_string(),
                NO_MARKER.to_string(),
            ],
            "{printed}"
        );
    }

    #[test]
    fn an_environment_running_the_head_already_says_so_and_applies_nothing() {
        let s = Scratch::new();
        s.declares("cache", "beta", "fake", json!({}));
        assert_eq!(s.run(&["apply", "cache"]).0, render::OK);
        let (code, printed) = s.run(&["apply", "cache"]);
        assert_eq!(code, render::OK, "{printed}");
        assert_eq!(
            printed.lines().next(),
            Some(
                format!(
                    "cache of swancloud-srv follows beta: 0.4.0 ({}) running it already, \
                     nothing applied",
                    s.second
                )
                .as_str()
            ),
            "{printed}"
        );
        let applies = s.ops().iter().filter(|op| **op == TargetOp::Apply).count();
        assert_eq!(applies, 1, "applied again");
    }

    #[test]
    fn a_claim_found_wanting_prints_the_refusal_naming_it_exits_1_and_applies_nothing() {
        let s = Scratch::new();
        s.declares("prod", "stable", "fake", json!({}));
        s.fakes.policy.require(
            Audience::Channel(of("stable")),
            requiring(&[(ClaimKind::Acceptance, GATE), (ClaimKind::Rebuild, "box")]),
        );
        let (code, printed) = s.run(&["apply", "prod"]);
        assert_eq!(code, render::REFUSED, "{printed}");
        assert!(printed.contains("no rebuild"), "{printed}");
        assert!(printed.contains("box"), "{printed}");
        assert!(!s.ops().contains(&TargetOp::Apply), "applied");

        let (code, printed) = s.run(&["apply", "prod", "--json"]);
        assert_eq!(code, render::REFUSED, "{printed}");
        let printed: Value = serde_json::from_str(&printed).unwrap();
        assert_eq!(
            printed["refused"]["why"]["wanting"]["kind"], "rebuild",
            "{printed}"
        );
        assert!(!s.ops().contains(&TargetOp::Apply), "applied");
    }

    #[test]
    fn an_environment_the_policy_does_not_declare_is_absent_and_exits_4() {
        let s = Scratch::new();
        let (code, printed) = s.run(&["apply", "nowhere"]);
        assert_eq!(code, render::ABSENT, "{printed}");
        assert_eq!(
            printed,
            format!(
                "the policy (bound) holds no environment nowhere of swancloud-srv at {}\n",
                s.head
            )
        );
        assert!(s.target.calls().is_empty(), "the target was asked");
    }

    #[test]
    fn apply_makes_no_store() {
        let s = Scratch::new();
        let file = s.dir.path().join("config/timber/config.toml");
        let data = s.dir.path().join("data");
        let home = s.dir.path().join("home");
        let binding = bindings::bind(
            &file,
            Some(&home),
            Some(data.as_os_str()),
            &s.repo,
            For::Apply,
        );
        let (code, printed) = s.run_over(&binding, &["apply", "cache", "--json"]);
        assert_eq!(code, render::UNREADABLE, "{printed}");
        let printed: Value = serde_json::from_str(&printed).unwrap();
        assert_eq!(source_of(&printed)["seam"], "store", "{printed}");
        let layout = data.join("timber/layout");
        assert!(!layout.exists(), "apply made {}", layout.display());
        assert!(!data.exists(), "apply made {}", data.display());
        assert!(!file.exists(), "apply wrote a bindings file");
    }

    // Requirement: Apply binds the repository's targets

    #[test]
    fn an_environment_naming_the_nix_cache_reaches_it_and_timber_writes_nothing() {
        let s = Scratch::new();
        s.declares(
            "cache",
            "beta",
            "nix_cache",
            json!({ "attic": "fleet:swancloud", "closures": [COMPONENT] }),
        );
        let mut binding = s.binding();
        let store = || Ok(Box::new(s.fakes.store.clone()) as Box<dyn timber_seams::Store>);
        let repository = timber_seams::RepositoryName {
            owner: "afterthought".into(),
            name: "swancloud".into(),
        };
        binding.seams.targets = profiles::target::bound(&store, &s.repo, Ok(repository)).unwrap();
        let before = files(s.dir.path());
        let (_, printed) = s.run_over(&Ok(binding), &["apply", "cache", "--json"]);
        let printed: Value = serde_json::from_str(&printed).unwrap();
        assert_eq!(
            source_of(&printed),
            serde_json::to_value(Source::new(Seam::Target, "nix_cache")).unwrap(),
            "{printed}"
        );
        assert_eq!(files(s.dir.path()), before, "timber wrote a file");
    }
}

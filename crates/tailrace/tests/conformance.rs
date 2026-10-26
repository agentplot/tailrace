//! The surfaces' group of the conformance suite over the command line and
//! over the served page: publish, verify and status asked with `--json`,
//! and a repository's `status.json` asked of the process's own handling
//! of a request, answer what the one core answers (56, 57, 132; design
//! D6, and status-on-any-machine's D9).

use clap::Parser;
use serde_json::{json, Value};
use tailrace::bindings::Teller;
use tailrace::commands::serve::host::{name, Host};
use tailrace::commands::serve::respond_with;
use tailrace::{run, Binding, Cli};
use tailrace_conformance::core::surfaces::{self, Question, Surface};
use tailrace_conformance::not_applicable;
use tailrace_conformance::world::FakeWorld;
use tailrace_core::RepositoryName;
use tempfile::TempDir;

/// The world's fakes as the one binding a surface is handed.
fn binding(world: &FakeWorld) -> Binding {
    Binding {
        seams: world.seams(),
        store: world.store(),
        signer: Some(world.host.clone()),
        gate: Teller::Job(JOB.into()),
    }
}

/// The job the binding names as what tells the gate, so the command
/// line's publish asks it nothing.
const JOB: &str = "the gate's job in another place";

/// The command line over a world of fakes, asked as a program asks it.
struct CommandLine {
    world: FakeWorld,
}

impl Surface for CommandLine {
    fn name(&self) -> String {
        "the command line".into()
    }

    fn world(&self) -> &FakeWorld {
        &self.world
    }

    fn ask(&self, question: &Question) -> Value {
        let args: Vec<String> = match question {
            Question::Publish { revision } => vec!["publish".into(), revision.to_string()],
            Question::Verify { version } => vec!["verify".into(), version.to_string()],
            Question::Status => vec!["status".into()],
            Question::Channels { branch } => {
                vec!["status".into(), "--branch".into(), branch.to_string()]
            }
        };
        let cli = Cli::try_parse_from(
            ["tailrace".to_string()]
                .into_iter()
                .chain(args)
                .chain(["--json".to_string()]),
        )
        .expect("the arguments parse");
        let mut out = Vec::new();
        run(
            &cli,
            &Ok(binding(&self.world)),
            &self.world.repo().path,
            &mut out,
        );
        let mut printed: Value = serde_json::from_slice(&out).unwrap_or_else(|e| {
            panic!(
                "{question:?} printed no JSON document ({e}): {}",
                String::from_utf8_lossy(&out)
            )
        });
        match question {
            // The record's part of status, every version the host
            // published.
            Question::Status => printed["value"]["published"].clone(),
            // The one thing beside the core's answer: each offered
            // version's job that tells the gate.
            Question::Publish { .. } => {
                if let Some(components) = printed["value"]["components"].as_array_mut() {
                    for component in components {
                        if let Some(gate) = component.as_object_mut().unwrap().remove("gate") {
                            assert_eq!(gate, json!({ "told_by": "job", "job": JOB }));
                        }
                    }
                }
                printed
            }
            _ => printed,
        }
    }
}

#[test]
fn the_command_line_answers_from_the_one_core() {
    tailrace_conformance::run(surfaces::GROUP, &surfaces::scenarios(), &|| {
        Box::new(CommandLine {
            world: FakeWorld::new(),
        })
    })
    .assert();
}

/// The served page over a world of fakes: the world's repository named
/// `o/r` in a state directory of its own, each request answered by the
/// process's own handling of it, without a socket, over the world's
/// binding.
struct Page {
    world: FakeWorld,
    host: Host,
    _state: TempDir,
}

impl Page {
    fn new() -> Page {
        let world = FakeWorld::new();
        let state = TempDir::new().expect("a temporary directory");
        let host = Host {
            state: state.path().join("tailrace"),
            config: state.path().join("no bindings file"),
            home: None,
            data: None,
        };
        let repository = RepositoryName {
            owner: "o".into(),
            name: "r".into(),
        };
        name(&host, &repository, &world.repo().path).expect("the repository is named");
        Page {
            world,
            host,
            _state: state,
        }
    }
}

impl Surface for Page {
    fn name(&self) -> String {
        "the served page".into()
    }

    fn world(&self) -> &FakeWorld {
        &self.world
    }

    fn ask(&self, question: &Question) -> Value {
        match question {
            Question::Publish { .. } | Question::Verify { .. } => not_applicable(
                "the served page is a reader: it answers status, and publishes and verifies nothing",
            ),
            Question::Status | Question::Channels { .. } => {}
        }
        let response = respond_with(
            &self.host,
            &|_| Ok(binding(&self.world)),
            "GET",
            "/o/r/status.json",
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let served: Value = serde_json::from_str(&response.body).unwrap_or_else(|e| {
            panic!(
                "{question:?} served no JSON document ({e}): {}",
                response.body
            )
        });
        match question {
            Question::Status => served["value"]["published"].clone(),
            _ => served,
        }
    }
}

#[test]
fn the_served_page_answers_from_the_one_core() {
    let report = tailrace_conformance::run(surfaces::GROUP, &surfaces::scenarios(), &|| {
        Box::new(Page::new())
    });
    report.assert();
    let inapplicable: Vec<&str> = report.not_applicable.iter().map(|n| n.scenario).collect();
    assert_eq!(
        inapplicable,
        ["the command line answers what the core answers"],
        "{report}"
    );
}

//! The surfaces' group of the conformance suite over the command line:
//! publish, verify and status asked with `--json` answer what the one
//! core answers (56, 57; design D6).

use clap::Parser;
use serde_json::Value;
use tailrace::bindings::Teller;
use tailrace::{run, Binding, Cli};
use tailrace_conformance::core::surfaces::{self, Question, Surface};
use tailrace_conformance::world::FakeWorld;
use tailrace_seams::{Not, Seam, Source};

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
        };
        let cli = Cli::try_parse_from(
            ["tailrace".to_string()]
                .into_iter()
                .chain(args)
                .chain(["--json".to_string()]),
        )
        .expect("the arguments parse");
        let binding = Binding {
            seams: self.world.seams(),
            store: self.world.store(),
            signer: Some(self.world.host.clone()),
            gate: Teller::Nothing(Not::absent(
                &Source::new(Seam::Host, "bindings"),
                "binding of what tells the gate, [gate] in the bindings file",
            )),
        };
        let mut out = Vec::new();
        run(&cli, &Ok(binding), &self.world.repo().path, &mut out);
        let printed: Value = serde_json::from_slice(&out).unwrap_or_else(|e| {
            panic!(
                "{question:?} printed no JSON document ({e}): {}",
                String::from_utf8_lossy(&out)
            )
        });
        match question {
            // The record's part of status, every version the host
            // published.
            Question::Status => printed["value"]["published"].clone(),
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

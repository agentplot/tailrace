//! The surfaces' group: a surface answers what the one core answers, value
//! for value, as a program reads it (56, 57; design D6).

use serde_json::Value;
use tailrace_core::{answer_json, status, StatusAsk};
use tailrace_seams::{
    Audience, Check, ClaimKind, Digest, NamedSigner, Required, RequiredClaim, Revision, SignerId,
    Version,
};

use crate::core::publish::{passing, VERSION};
use crate::world::{FakeWorld, HOST, LANDING};
use crate::{Rig, Scenario};

pub const GROUP: &str = "surfaces";

/// A question every surface puts to the core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Question {
    /// Publish `revision`, every component the landing touched.
    Publish { revision: Revision },
    /// Verify `version` of the one component, under the policy for
    /// acceptance.
    Verify { version: Version },
    /// Status, of every version the host published.
    Status,
}

/// A surface over a world of fakes, made new for each scenario.
pub trait Surface {
    /// The surface, by name.
    fn name(&self) -> String;

    /// The world the surface asks over.
    fn world(&self) -> &FakeWorld;

    /// `question` asked through the surface, its answer as a program
    /// reads it.
    fn ask(&self, question: &Question) -> Value;
}

impl Rig for dyn Surface {
    fn profile(&self) -> String {
        self.name()
    }
}

pub fn scenarios() -> Vec<Scenario<dyn Surface>> {
    vec![
        Scenario {
            name: "the command line answers what the core answers",
            plays: None,
            cites: &[56, 57],
            run: the_command_line_answers_what_the_core_answers,
        },
        // Below this line a later unit adds its scenarios; nothing above it changes.
    ]
}

fn named(id: &str, kind: ClaimKind, checks: &[&str]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: vec![kind],
        checks: checks.iter().map(|c| Check::new(*c)).collect(),
    }
}

/// `question` asked through `surface` and of the core, its answer the
/// same value for value.
fn alike(surface: &dyn Surface, question: &Question, core: Value) -> Value {
    let through = surface.ask(question);
    assert_eq!(
        through,
        core,
        "{question:?} asked through {} answered otherwise than the core",
        surface.name()
    );
    through
}

fn the_command_line_answers_what_the_core_answers(surface: &dyn Surface) {
    let world = surface.world();
    passing(world, &world.head);
    world.fakes.policy.require(
        Audience::Acceptance,
        Required::Acceptance {
            claims: vec![
                RequiredClaim {
                    kind: ClaimKind::Provenance,
                    check: None,
                    signers: vec![named(HOST, ClaimKind::Provenance, &[])],
                },
                RequiredClaim {
                    kind: ClaimKind::Verdict,
                    check: Some(Check::new("test")),
                    signers: vec![named(LANDING, ClaimKind::Verdict, &["test"])],
                },
            ],
        },
    );

    // Publish through the surface first: the core's answer is then a
    // repeat, which ends where the first did.
    let question = Question::Publish {
        revision: world.head.clone(),
    };
    let through = surface.ask(&question);
    let core = answer_json(&world.publish(&world.ask(&world.head)));
    assert_eq!(
        through,
        core,
        "{question:?} asked through {} answered otherwise than the core",
        surface.name()
    );
    let published = &through["value"]["components"][0]["value"];
    assert_eq!(published["version"], VERSION, "{through}");
    let index = published["index"].as_str().expect("the index, by digest");
    assert!(Digest::parse(index).is_ok(), "{index} is no digest");

    let verified = alike(
        surface,
        &Question::Verify {
            version: Version::new(VERSION),
        },
        answer_json(&world.verify(&world.verify_ask(VERSION, Audience::Acceptance))),
    );
    let kinds: Vec<&str> = verified["value"]["claims"]
        .as_array()
        .expect("the claims told")
        .iter()
        .filter_map(|t| t["kind"].as_str())
        .collect();
    assert!(
        kinds.contains(&"provenance") && kinds.contains(&"verdict"),
        "verify's answer names no provenance or verdict as the statement does: {kinds:?}"
    );

    alike(
        surface,
        &Question::Status,
        answer_json(&status(
            &world.history(),
            &world.kept(),
            &StatusAsk::default(),
        )),
    );
}

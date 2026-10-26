//! Spec command-line: publish tells the host's gate where the host's
//! bindings file binds it to, and says what tells the gate where it binds
//! a job or nothing; over the fakes, through `tailrace::run` (95, 108,
//! 130; design D5, D7).

use clap::Parser;
use serde_json::Value;
use tailrace::bindings::Teller;
use tailrace::render::{ABSENT, OK, REFUSED};
use tailrace::{run, Binding, Cli};
use tailrace_conformance::world::{
    build_of, FakeWorld, Write, HOST, LANDING, POLICY, TWO_COMPONENTS,
};
use tailrace_core::{accept, answer_json, publish, AcceptAsk, Kept, Published, ToldBy};
use tailrace_fakes::KeeperOp;
use tailrace_seams::{
    Audience, Chain, ChannelName, Check, Claim, ClaimKind, Digest, Envelope, Keeper, NamedSigner,
    Not, Outcome, Promotion, Required, RequiredClaim, Revision, Seam, Source, Store, Version,
};

/// The channel that follows each publish.
const BETA: &str = "beta";

/// What the file binds where it names no `[gate]`.
const NO_GATE: &str =
    "binding of what tells the gate, [gate] in /home/a/.config/tailrace/config.toml";

/// The job a separated gate is told by.
const JOB: &str = "the release job a push to main wakes";

fn beta() -> ChannelName {
    ChannelName::new(BETA)
}

fn named(id: &str, kinds: &[ClaimKind]) -> NamedSigner {
    NamedSigner {
        id: tailrace_seams::SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    }
}

fn test_from_the_landing() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Verdict,
        check: Some(Check::new("test")),
        signers: vec![NamedSigner {
            checks: vec![Check::new("test")],
            ..named(LANDING, &[ClaimKind::Verdict])
        }],
    }
}

/// Acceptance requires the host's provenance, the host named for
/// `kinds`, and the landing's verdict of `test`.
fn acceptance(kinds: &[ClaimKind]) -> Required {
    Required::Acceptance {
        claims: vec![
            RequiredClaim {
                kind: ClaimKind::Provenance,
                check: None,
                signers: vec![named(HOST, kinds)],
            },
            test_from_the_landing(),
        ],
    }
}

/// The one host: its signer may claim both provenance and acceptance
/// (108).
fn one_host() -> Required {
    acceptance(&[ClaimKind::Provenance, ClaimKind::Acceptance])
}

/// `beta` requires the host's acceptance and each of `more`.
fn beta_requires(world: &FakeWorld, more: Vec<RequiredClaim>) {
    let mut claims = vec![RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![named(HOST, &[ClaimKind::Acceptance, ClaimKind::Promotion])],
    }];
    claims.extend(more);
    world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims,
            bound_to: None,
        },
    );
}

/// `policy` with `channel` landed, each of `outputs` built and given the
/// landing's passing `test` at that revision, acceptance requiring
/// `required` and `beta` the host's acceptance; the revision.
fn landed(
    world: &FakeWorld,
    policy: &str,
    channel: &str,
    outputs: &[&str],
    required: Required,
) -> Revision {
    let revision = world.land_policy(&format!("{policy}\n{channel}"));
    let verdicts: Vec<_> = outputs
        .iter()
        .map(|o| {
            world.verdict(
                &revision,
                "test",
                &build_of(o, &revision),
                Outcome::Pass,
                LANDING,
            )
        })
        .collect();
    world.land(&revision, &verdicts);
    let built: Vec<_> = outputs.iter().map(|o| (*o, Outcome::Pass)).collect();
    world.outputs(&revision, &built);
    world.fakes.policy.require(Audience::Acceptance, required);
    beta_requires(world, vec![]);
    revision
}

/// The one-component world with `beta` following each publish, landed
/// under `required`; the revision.
fn following(world: &FakeWorld, required: Required) -> Revision {
    landed(
        world,
        POLICY,
        "[channels.beta]\non_each_publish = true\n",
        &["site", "server"],
        required,
    )
}

/// The host's binding over the world's fakes, signing as the host, with
/// `gate` as what tells its gate.
fn binding(world: &FakeWorld, gate: Teller) -> Binding {
    Binding {
        seams: world.seams(),
        store: world.store(),
        signer: Some(world.host.clone()),
        gate,
    }
}

/// `publish <revision>` asked through `run` over `binding`, with
/// `flags` after it: the exit code and what it printed.
fn publish_through(
    world: &FakeWorld,
    binding: Binding,
    revision: &Revision,
    flags: &[&str],
) -> (i32, String) {
    let mut args = vec!["tailrace", "publish", revision.as_str()];
    args.extend(flags);
    let cli = Cli::try_parse_from(args).expect("the arguments parse");
    let mut out = Vec::new();
    let code = run(&cli, &Ok(binding), &world.repo().path, &mut out);
    (code, String::from_utf8(out).unwrap())
}

fn json(printed: &str) -> Value {
    serde_json::from_str(printed).unwrap_or_else(|e| panic!("no JSON ({e}): {printed}"))
}

/// The index the store's name for `tag` resolves to.
fn index_of(world: &FakeWorld, tag: &str) -> Digest {
    world
        .fakes
        .store
        .resolve(&tailrace_seams::Name::new(tag))
        .unwrap_or_else(|e| panic!("{tag} names nothing: {e}"))
}

/// Every claim of `kind` on `digest`.
fn of_kind(world: &FakeWorld, digest: &Digest, kind: ClaimKind) -> Vec<Claim> {
    world
        .fakes
        .store
        .claims_of(digest)
        .unwrap()
        .iter()
        .filter_map(Claim::from_envelope)
        .filter(|c| c.kind == kind)
        .collect()
}

/// The promotions `beta`'s chain holds, in its order.
fn beta_chain(world: &FakeWorld) -> Vec<Envelope> {
    world
        .fakes
        .keeper
        .chain(&beta())
        .map(|c| c.claims)
        .unwrap_or_default()
}

/// Nothing accepted, nothing promoted and the ledger never asked.
fn the_gate_untold(world: &FakeWorld, index: &Digest) {
    assert!(of_kind(world, index, ClaimKind::Acceptance).is_empty());
    assert!(of_kind(world, index, ClaimKind::Promotion).is_empty());
    assert!(
        world.keeper_calls().is_empty(),
        "the ledger was asked {:?}",
        world.keeper_calls()
    );
}

// Requirement: Publish tells the host's gate where the host binds it to

#[test]
fn one_host_accepts_and_moves_beta_in_the_same_run() {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let (code, printed) = publish_through(&world, binding(&world, Teller::Publish), &revision, &[]);
    assert_eq!(code, OK, "{printed}");
    let index = index_of(&world, "v0.2.0");

    let acceptances = of_kind(&world, &index, ClaimKind::Acceptance);
    assert_eq!(acceptances.len(), 1, "{acceptances:?}");
    assert_eq!(acceptances[0].body["offered"]["one_host"], true);
    let chain = beta_chain(&world);
    assert_eq!(chain.len(), 1);
    let promotion = Promotion::of(&Claim::from_envelope(&chain[0]).unwrap()).unwrap();
    assert_eq!(promotion.to.index, index);
    assert_eq!(promotion.predecessor, None);
    assert_eq!(
        world
            .fakes
            .store
            .resolve(&tailrace_seams::channel_name(&beta()))
            .unwrap(),
        index
    );

    assert!(printed.contains(&format!("0.2.0 (v0.2.0) from {revision}: index {index}")));
    assert!(
        printed.contains("  gate: accepted, acceptance ")
            && printed.contains(", one host offered and accepted"),
        "{printed}"
    );
    assert!(
        printed.contains(&format!(
            "    beta promoted from nothing to {index}: claim {}",
            chain[0].digest()
        )),
        "{printed}"
    );
}

#[test]
fn the_gate_refusing_what_the_host_offered_leaves_it_offered() {
    let world = FakeWorld::new();
    let revision = following(&world, acceptance(&[ClaimKind::Provenance]));
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Publish),
        &revision,
        &["--json"],
    );
    assert_eq!(code, REFUSED, "{printed}");
    let index = index_of(&world, "v0.2.0");

    let component = &json(&printed)["value"]["components"][0];
    assert_eq!(component["answer"], "ok", "the version is not offered");
    let gate = &component["gate"];
    assert_eq!(gate["told_by"], "publish");
    assert_eq!(gate["value"]["judged"], "refused");
    assert_eq!(gate["value"]["wanting"]["kind"], "provenance");
    assert_eq!(gate["value"]["why"]["offered_by_gate"]["signer"], HOST);
    let first = &component["value"]["artifacts"][0]["digest"];
    assert_eq!(&gate["value"]["artifact"], first);

    assert!(of_kind(&world, &index, ClaimKind::Acceptance).is_empty());
    assert!(beta_chain(&world).is_empty());
    assert!(!world.keeper_calls().contains(&KeeperOp::Append));
}

#[test]
fn a_channel_held_is_refused_while_the_version_is_accepted() {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let review = RequiredClaim {
        kind: ClaimKind::Review,
        check: None,
        signers: vec![named("reviewer", &[ClaimKind::Review])],
    };
    beta_requires(&world, vec![review]);
    let (code, printed) = publish_through(&world, binding(&world, Teller::Publish), &revision, &[]);
    assert_eq!(code, REFUSED, "{printed}");
    let index = index_of(&world, "v0.2.0");
    assert_eq!(of_kind(&world, &index, ClaimKind::Acceptance).len(), 1);
    assert!(beta_chain(&world).is_empty());
    assert!(
        printed.contains("  gate: accepted, acceptance ")
            && printed.contains("    beta held: ")
            && printed.contains(" lacks review: no claim attached counts for it"),
        "{printed}"
    );
}

/// The ledger of a file naming no `[ledger]`, as the bindings bind it.
struct Unbound;

impl Unbound {
    fn absent() -> Not {
        Not::absent(
            &Source::new(Seam::Keeper, "unbound"),
            "binding of the ledger, [ledger] in /home/a/.config/tailrace/config.toml",
        )
    }
}

impl Keeper for Unbound {
    fn append(
        &self,
        _: &ChannelName,
        _: &Envelope,
        _: Option<&Digest>,
    ) -> tailrace_seams::Answer<Digest> {
        Err(Unbound::absent())
    }

    fn chain(&self, _: &ChannelName) -> tailrace_seams::Answer<Chain> {
        Err(Unbound::absent())
    }
}

#[test]
fn accepted_with_no_ledger_bound_beta_is_absent_naming_the_ledger() {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let mut bound = binding(&world, Teller::Publish);
    bound.seams.keeper = Box::new(Unbound);
    let (code, printed) = publish_through(&world, bound, &revision, &["--json"]);
    assert_eq!(code, ABSENT, "{printed}");
    let index = index_of(&world, "v0.2.0");

    let gate = &json(&printed)["value"]["components"][0]["gate"];
    assert_eq!(gate["value"]["judged"], "accepted");
    assert_eq!(gate["value"]["channels"][0]["channel"], BETA);
    assert_eq!(gate["value"]["channels"][0], {
        let mut value = answer_json::<()>(&Err(Unbound::absent()));
        value["channel"] = BETA.into();
        value
    });
    assert_eq!(of_kind(&world, &index, ClaimKind::Acceptance).len(), 1);
    assert!(of_kind(&world, &index, ClaimKind::Promotion).is_empty());
    assert!(world
        .fakes
        .store
        .resolve(&tailrace_seams::channel_name(&beta()))
        .is_err());
}

#[test]
fn a_component_left_unchanged_tells_the_gate_nothing() {
    let world = FakeWorld::two_components();
    let revision = landed(
        &world,
        TWO_COMPONENTS,
        "[channels.beta]\non_each_publish = true\ncomponent = \"lib\"\n",
        &["library", "server"],
        one_host(),
    );
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Publish),
        &revision,
        &["--component", "app", "--json"],
    );
    assert_eq!(code, OK, "{printed}");
    let components = json(&printed)["value"]["components"].clone();
    let [app] = components.as_array().unwrap().as_slice() else {
        panic!("not one component: {components}")
    };
    assert_eq!(app["component"], "app");
    assert_eq!(app["value"]["published"], "unchanged");
    assert!(app.get("gate").is_none(), "{app}");
    assert!(world.keeper_calls().is_empty(), "the ledger was asked");
    let accepted = world
        .writes()
        .into_iter()
        .filter(|w| {
            matches!(w, Write::Attach { envelope, .. }
                if Claim::from_envelope(envelope).is_some_and(|c| c.kind == ClaimKind::Acceptance))
        })
        .count();
    assert_eq!(accepted, 0, "the gate was asked");
}

#[test]
fn the_json_gate_member_is_the_gates_own_answer_value_for_value() {
    let told = FakeWorld::new();
    let revision = following(&told, one_host());
    let (code, printed) = publish_through(
        &told,
        binding(&told, Teller::Publish),
        &revision,
        &["--json"],
    );
    assert_eq!(code, OK, "{printed}");
    let mut gate = json(&printed)["value"]["components"][0]["gate"].clone();
    assert_eq!(
        gate.as_object_mut().unwrap().remove("told_by"),
        Some("publish".into())
    );

    // The same landing in a world of its own, published by the core and
    // then judged by the core's gate at the moment the command's
    // promotion names.
    let asked = FakeWorld::new();
    let same = following(&asked, one_host());
    assert_eq!(same, revision);
    let history = asked.history();
    let seams = asked.seams();
    let published = publish(&seams, &history, &Kept::at(&history), &asked.ask(&same)).unwrap();
    let Ok(Published::Version { index, .. }) = &published.components[0].1 else {
        panic!("nothing offered: {published:?}")
    };
    let chain = beta_chain(&told);
    let at = Promotion::of(&Claim::from_envelope(&chain[0]).unwrap())
        .unwrap()
        .at;
    let core = accept(
        &seams,
        &history,
        &AcceptAsk {
            branch: asked.main.clone(),
            component: Some(published.components[0].0.clone()),
            version: Version::new("0.2.0"),
            told: ToldBy::Publish {
                index: index.clone(),
            },
            gate: asked.host.clone(),
            when: at,
        },
    );
    assert_eq!(gate, answer_json(&core));
}

// Requirement: Publish says when nothing on the host tells the gate

#[test]
fn with_no_gate_table_the_version_stays_offered_and_says_so() {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let nothing = Not::absent(
        &Source::host("/home/a/.config/tailrace/config.toml"),
        NO_GATE,
    );
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Nothing(nothing.clone())),
        &revision,
        &[],
    );
    assert_eq!(code, OK, "{printed}");
    let index = index_of(&world, "v0.2.0");
    the_gate_untold(&world, &index);
    assert!(
        printed.contains("  gate: 0.2.0 is offered, and nothing on this host tells the gate:\n")
            && printed.contains(&format!("    {nothing}\n")),
        "{printed}"
    );
    assert!(printed.contains("[gate] in /home/a/.config/tailrace/config.toml"));

    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Nothing(nothing.clone())),
        &revision,
        &["--json"],
    );
    assert_eq!(code, OK, "{printed}");
    assert_eq!(
        json(&printed)["value"]["components"][0]["gate"],
        answer_json::<()>(&Err(nothing))
    );
}

#[test]
fn a_job_tells_the_gate_in_its_words_and_the_gate_is_not_asked() {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Job(JOB.into())),
        &revision,
        &[],
    );
    assert_eq!(code, OK, "{printed}");
    let index = index_of(&world, "v0.2.0");
    the_gate_untold(&world, &index);
    assert!(
        printed.contains(&format!(
            "  gate: 0.2.0 is offered, and the gate is told by \"{JOB}\"\n"
        )),
        "{printed}"
    );

    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Job(JOB.into())),
        &revision,
        &["--json"],
    );
    assert_eq!(code, OK, "{printed}");
    assert_eq!(
        json(&printed)["value"]["components"][0]["gate"],
        serde_json::json!({ "told_by": "job", "job": JOB })
    );
}

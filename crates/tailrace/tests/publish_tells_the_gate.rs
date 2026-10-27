//! Spec command-line: publish tells the host's gate where the host's
//! bindings file binds it to, by `publish = {}` or by naming no `[gate]`
//! (118), and says what tells the gate where it binds a job; over the
//! fakes, through `tailrace::run` (95, 108, 130; design D5, D7).

use std::fs;

use clap::Parser;
use serde_json::Value;
use tailrace::bindings::{self, For, Teller};
use tailrace::render::{ABSENT, OK, REFUSED};
use tailrace::{run, Binding, Cli};
use tailrace_conformance::world::{
    build_of, FakeWorld, Write, HOST, LANDING, POLICY, TWO_COMPONENTS,
};
use tailrace_core::{accept, answer_json, publish, AcceptAsk, Kept, Published, ToldBy};
use tailrace_fakes::KeeperOp;
use tailrace_seams::{
    AskedOf, Audience, Chain, ChannelId, ChannelName, Check, Claim, ClaimKind, ComponentName,
    Digest, Envelope, Keeper, NamedSigner, Not, Outcome, Promotion, Required, RequiredClaim,
    Revision, Seam, Source, Store, Version,
};
use tempfile::TempDir;

/// The channel that follows each publish.
const BETA: &str = "beta";

/// The job a separated gate is told by.
const JOB: &str = "the release job a push to main wakes";

/// `beta` of the repository that declares no component.
fn beta() -> ChannelId {
    ChannelId::new(None, ChannelName::new(BETA))
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
        asked_of: AskedOf::Every,
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
                asked_of: AskedOf::Every,
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

/// `channel` requires the host's acceptance and each of `more`, and is
/// promoted by the host, the gate's signer.
fn requires(world: &FakeWorld, channel: &ChannelId, more: Vec<RequiredClaim>) {
    let host = named(HOST, &[ClaimKind::Acceptance, ClaimKind::Promotion]);
    let mut claims = vec![RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![host.clone()],
        asked_of: AskedOf::Every,
    }];
    claims.extend(more);
    world.fakes.policy.require(
        Audience::Channel(channel.clone()),
        Required::Channel {
            claims,
            bound_to: None,
            promoted_by: vec![host],
            approvers: vec![],
        },
    );
}

/// `beta` requires the host's acceptance and each of `more`.
fn beta_requires(world: &FakeWorld, more: Vec<RequiredClaim>) {
    requires(world, &beta(), more);
}

/// `beta` requires the acceptance of a gate other than the host, and is
/// promoted by it, so the policy names a gate (118).
fn beta_requires_a_gate_elsewhere(world: &FakeWorld) {
    let gate = named("gate", &[ClaimKind::Acceptance, ClaimKind::Promotion]);
    world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![gate.clone()],
                asked_of: AskedOf::Every,
            }],
            bound_to: None,
            promoted_by: vec![gate],
            approvers: vec![],
        },
    );
}

/// The policy as the policy file profile answers a file that names the
/// host for provenance and no signer that may claim acceptance: the host,
/// the given gate, with provenance and acceptance in acceptance's
/// required provenance, and the given `beta` requiring its acceptance
/// and promoted by it (118, 108, C.10).
fn the_host_the_given_gate(world: &FakeWorld) {
    let host = named(HOST, &[ClaimKind::Provenance, ClaimKind::Acceptance]);
    world.fakes.policy.require(
        Audience::Acceptance,
        acceptance(&[ClaimKind::Provenance, ClaimKind::Acceptance]),
    );
    world.fakes.policy.require(
        Audience::Channel(beta()),
        Required::Channel {
            claims: vec![RequiredClaim {
                kind: ClaimKind::Acceptance,
                check: None,
                signers: vec![host.clone()],
                asked_of: AskedOf::Every,
            }],
            bound_to: None,
            promoted_by: vec![host],
            approvers: vec![],
        },
    );
}

/// `policy` landed, each of `outputs` built and given the landing's
/// passing `test` at that revision, acceptance requiring `required`, and
/// `channel` following each publish as the policy seam lists it,
/// requiring the host's acceptance; the revision.
fn landed(
    world: &FakeWorld,
    policy: &str,
    channel: &ChannelId,
    outputs: &[&str],
    required: Required,
) -> Revision {
    let revision = world.land_policy(policy);
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
    requires(world, channel, vec![]);
    world.fakes.policy.follows(channel, false);
    revision
}

/// The one-component world with `beta` following each publish, landed
/// under `required`; the revision.
fn following(world: &FakeWorld, required: Required) -> Revision {
    landed(world, POLICY, &beta(), &["site", "server"], required)
}

/// The host's binding over the world's fakes, signing as the host, with
/// `gate` as what tells its gate.
fn binding(world: &FakeWorld, gate: Teller) -> Binding {
    Binding {
        seams: world.seams(),
        store: world.store(),
        signer: Some(world.host.clone()),
        gate,
        trust: Vec::new(),
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
    beta_requires_a_gate_elsewhere(&world);
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
        asked_of: AskedOf::Every,
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
        _: &ChannelId,
        _: &Envelope,
        _: Option<&Digest>,
    ) -> tailrace_seams::Answer<Digest> {
        Err(Unbound::absent())
    }

    fn chain(&self, _: &ChannelId) -> tailrace_seams::Answer<Chain> {
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
    let beta_json = serde_json::json!({ "component": null, "name": BETA });
    assert_eq!(gate["value"]["channels"][0]["channel"], beta_json);
    assert_eq!(gate["value"]["channels"][0], {
        let mut value = answer_json::<()>(&Err(Unbound::absent()));
        value["channel"] = beta_json.clone();
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
        &ChannelId::new(Some(ComponentName::new("lib")), ChannelName::new(BETA)),
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

/// The teller a host's bindings file holding no `[gate]` binds, read
/// through the bindings as the host reads its file, in the world's
/// repository.
fn teller_of_a_file_naming_none(world: &FakeWorld) -> Teller {
    let host = TempDir::new().unwrap();
    let file = host.path().join("config.toml");
    fs::write(
        &file,
        format!(
            "[store]\nlayout = \"{}\"\n\n[landing]\nbranch = {{}}\n\n\
             [signing]\nsigner = \"{HOST}\"\nkey = \"~/signing\"\n",
            host.path().join("layout").display()
        ),
    )
    .unwrap();
    match bindings::bind(
        &file,
        Some(host.path()),
        None,
        &world.repo().path,
        For::Publish,
    ) {
        Ok(binding) => binding.gate,
        Err(not) => panic!("a file naming no [gate] bound nothing: {not}"),
    }
}

/// The teller a host with no bindings file binds, read through the
/// bindings from no file, the given layout under a data directory of its
/// own, in the world's repository.
fn teller_of_no_file(world: &FakeWorld) -> Teller {
    let host = TempDir::new().unwrap();
    let file = bindings::path(Some(host.path().join("config").as_os_str()), None).unwrap();
    let data = host.path().join("data");
    let bound = bindings::bind(
        &file,
        Some(&host.path().join("home")),
        Some(data.as_os_str()),
        &world.repo().path,
        For::Publish,
    );
    assert!(!file.exists(), "binding wrote a bindings file");
    match bound {
        Ok(binding) => binding.gate,
        Err(not) => panic!("a host with no bindings file bound nothing: {not}"),
    }
}

/// What one publish on the one host with `beta` following each publish
/// came to, over a world of its own, where `teller` tells the gate: the
/// exit code; the JSON answer, each promotion's envelope taken out of
/// it; the acceptances on the index; and `beta`'s promotions, each
/// moment taken out of it. The moment is the clock's, read by the
/// command (design D5), so two runs may read two seconds.
fn one_host_run(
    teller: impl FnOnce(&FakeWorld) -> Teller,
) -> (i32, Value, Vec<Claim>, Vec<Promotion>) {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let told = teller(&world);
    let (code, printed) = publish_through(&world, binding(&world, told), &revision, &["--json"]);
    let index = index_of(&world, "v0.2.0");
    let chain = beta_chain(&world);
    let mut answer = json(&printed);
    for moved in answer["value"]["components"][0]["gate"]["value"]["channels"]
        .as_array_mut()
        .unwrap_or_else(|| panic!("no channel moved: {printed}"))
    {
        if let Some(promotion) = moved["value"].as_object_mut().unwrap().remove("promotion") {
            assert!(
                chain.iter().any(|c| promotion == c.digest().to_string()),
                "{promotion} is no promotion of beta's chain"
            );
        }
    }
    let promotions = chain
        .iter()
        .map(|envelope| Promotion {
            at: 0,
            ..Promotion::of(&Claim::from_envelope(envelope).unwrap()).unwrap()
        })
        .collect();
    (
        code,
        answer,
        of_kind(&world, &index, ClaimKind::Acceptance),
        promotions,
    )
}

#[test]
fn a_file_with_no_gate_table_tells_the_gate_as_publish_does() {
    let named = one_host_run(|_| Teller::Publish);
    let naming_none = one_host_run(teller_of_a_file_naming_none);
    assert_eq!(naming_none.0, OK, "{}", naming_none.1);
    assert_eq!(naming_none.0, named.0);
    assert_eq!(naming_none.1, named.1);
    assert_eq!(
        naming_none.1["value"]["components"][0]["gate"]["told_by"],
        "publish"
    );
    assert_eq!(naming_none.2.len(), 1, "{:?}", naming_none.2);
    assert_eq!(naming_none.2, named.2);
    assert_eq!(naming_none.3.len(), 1, "{:?}", naming_none.3);
    assert_eq!(naming_none.3, named.3);
}

#[test]
fn a_host_with_no_file_under_a_policy_naming_no_gate_accepts_as_one_host_and_moves_beta() {
    let world = FakeWorld::new();
    let revision = following(&world, acceptance(&[ClaimKind::Provenance]));
    the_host_the_given_gate(&world);
    let teller = teller_of_no_file(&world);
    assert_eq!(teller, Teller::Publish);
    let (code, printed) = publish_through(&world, binding(&world, teller), &revision, &["--json"]);
    assert_eq!(code, OK, "{printed}");
    let index = index_of(&world, "v0.2.0");

    let gate = &json(&printed)["value"]["components"][0]["gate"];
    assert_eq!(gate["told_by"], "publish");
    assert_eq!(gate["value"]["judged"], "accepted", "{gate}");
    assert_eq!(gate["value"]["one_host"], true, "{gate}");
    let acceptances = of_kind(&world, &index, ClaimKind::Acceptance);
    assert_eq!(acceptances.len(), 1, "{acceptances:?}");
    assert_eq!(
        acceptances[0].body["offered"],
        serde_json::json!({ "by": [HOST], "one_host": true })
    );
    let chain = beta_chain(&world);
    assert_eq!(chain.len(), 1, "beta did not move");
    let promotion = Promotion::of(&Claim::from_envelope(&chain[0]).unwrap()).unwrap();
    assert_eq!(promotion.to.index, index);
}

// Requirement: Publish says when a job tells the gate

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
}

#[test]
fn the_json_answer_says_a_job_tells_the_gate_in_its_words() {
    let world = FakeWorld::new();
    let revision = following(&world, one_host());
    let (code, printed) = publish_through(
        &world,
        binding(&world, Teller::Job(JOB.into())),
        &revision,
        &["--json"],
    );
    assert_eq!(code, OK, "{printed}");
    let component = &json(&printed)["value"]["components"][0];
    assert_eq!(component["answer"], "ok", "the version is not offered");
    assert_eq!(component["value"]["version"], "0.2.0");
    assert_eq!(
        component["gate"],
        serde_json::json!({ "told_by": "job", "job": JOB })
    );
    the_gate_untold(&world, &index_of(&world, "v0.2.0"));
}

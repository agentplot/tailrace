//! The layouts' group: a checkout's layouts rendered from the store with
//! the versions its lockfiles pin, each under a stacked policy, the
//! consumer's written layout line, else its publisher's, else the given,
//! each claim read through the signers the consumer's file names for it,
//! else through the signers the host trusts, holding only what the
//! policy admits, S4 among them; a refusal answered naming the claim
//! found wanting, a layout that cannot be refreshed answering what it
//! holds, and the host's record of each layout, rebuilt when deleted (47,
//! 50, 51, 61 to 63, 99, 113, 118; design D10).

use std::fs;
use std::path::Path;

use serde_json::{json, Value};
use tailrace_core::render::{Kept, Refreshed};
use tailrace_core::{layout_policy_body, CheckoutSynced, PinAnswer, PolicyOf, Refresh, Under};
use tailrace_fakes::{Fault, StoreOp, Times};
use tailrace_seams::{
    Artifact, Audience, ClaimKind, Digest, Ecosystem, NamedSigner, Not, Required, RequiredClaim,
    Revision, SignerId, Store, Why,
};

use crate::world::{FakeWorld, FILE, HOST};
use crate::{ok, Scenario};

pub const GROUP: &str = "layouts";

pub fn scenarios() -> Vec<Scenario<FakeWorld>> {
    vec![
        // Below this line a later unit adds its scenarios; nothing above it changes.
        Scenario {
            name: "a lockfile's versions and no other",
            plays: Some("S4"),
            cites: &[50],
            run: a_lockfiles_versions_and_no_other,
        },
        Scenario {
            name: "no lockfile, no layout",
            plays: None,
            cites: &[50],
            run: no_lockfile_no_layout,
        },
        Scenario {
            name: "a pin the store does not name",
            plays: None,
            cites: &[51],
            run: a_pin_the_store_does_not_name,
        },
        Scenario {
            name: "the consumer's written line at its landed revision",
            plays: None,
            cites: &[113],
            run: the_consumers_written_line_at_its_landed_revision,
        },
        Scenario {
            name: "the publisher's policy through the host's trust",
            plays: None,
            cites: &[113],
            run: the_publishers_policy_through_the_hosts_trust,
        },
        Scenario {
            name: "the publisher's above the consumer's given",
            plays: None,
            cites: &[113],
            run: the_publishers_above_the_consumers_given,
        },
        Scenario {
            name: "the given where neither writes one",
            plays: None,
            cites: &[47, 113],
            run: the_given_where_neither_writes_one,
        },
        Scenario {
            name: "a version refused naming its claim",
            plays: None,
            cites: &[47, 99],
            run: a_version_refused_naming_its_claim,
        },
        Scenario {
            name: "a version refused after it was held",
            plays: None,
            cites: &[47],
            run: a_version_refused_after_it_was_held,
        },
        Scenario {
            name: "a layout that cannot be refreshed",
            plays: None,
            cites: &[51],
            run: a_layout_that_cannot_be_refreshed,
        },
        Scenario {
            name: "the record deleted and rebuilt",
            plays: None,
            cites: &[61, 62, 63],
            run: the_record_deleted_and_rebuilt,
        },
        Scenario {
            name: "a consumer's file reads under its own signers alone",
            plays: None,
            cites: &[47, 113],
            run: a_consumers_file_reads_under_its_own_signers_alone,
        },
        Scenario {
            name: "a claim the file names no signer for read through the host's trust",
            plays: None,
            cites: &[47, 113, 118],
            run: a_claim_the_file_names_no_signer_for_read_through_the_hosts_trust,
        },
    ]
}

/// The gate's signer, whose acceptance the layouts' policies require.
const GATE: &str = "gate";

/// `id` named for `kinds`, verified by its own key.
fn named(id: &str, kinds: &[ClaimKind]) -> NamedSigner {
    NamedSigner {
        id: SignerId::new(id),
        verified_by: format!("{id}'s key"),
        kinds: kinds.to_vec(),
        checks: vec![],
    }
}

/// Acceptance by the gate's signer.
fn acceptance() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![named(GATE, &[ClaimKind::Acceptance])],
    }
}

/// Provenance by the host's signer.
fn provenance() -> RequiredClaim {
    RequiredClaim {
        kind: ClaimKind::Provenance,
        check: None,
        signers: vec![named(HOST, &[ClaimKind::Provenance])],
    }
}

/// The signers a host trusts, its own among them, each with no claim of
/// its own to make: the consumer allows each the claim it reads through
/// it.
fn trusts(ids: &[&str]) -> Vec<NamedSigner> {
    ids.iter().map(|id| named(id, &[])).collect()
}

/// The world's head landed with no verdict, and the consumer's file
/// writing at it a layout line requiring `claims`, where it has one.
fn landed(world: &FakeWorld, claims: Option<Vec<RequiredClaim>>) -> Revision {
    world.land(&world.head, &[]);
    if let Some(claims) = claims {
        world.fakes.policy.require_at(
            &world.head,
            Audience::Layout,
            Required::Layout {
                claims,
                given: false,
                signers: vec![],
            },
        );
    }
    world.head.clone()
}

/// The consumer's file at `revision` naming `signers` and writing no
/// layout line, so the policy answers its given, requiring `claims`
/// (C.10).
fn given_at(
    world: &FakeWorld,
    revision: &Revision,
    claims: Vec<RequiredClaim>,
    signers: Vec<NamedSigner>,
) {
    world.fakes.policy.require_at(
        revision,
        Audience::Layout,
        Required::Layout {
            claims,
            given: true,
            signers,
        },
    );
}

/// `version` of `site` held in the store under its `artifact/` name; and,
/// where `accepted`, the gate's acceptance attached to its index.
fn site(world: &FakeWorld, version: &str, accepted: bool) -> (Digest, Artifact) {
    let (index, artifact) = world.artifact(FILE, "site", version);
    if accepted {
        world.attach(&index, ClaimKind::Acceptance, Value::Null, GATE);
    }
    (index, artifact)
}

/// The publisher's layout policy requiring `claims`, by `signer`,
/// attached to `index`.
fn publishers(world: &FakeWorld, index: &Digest, claims: &[RequiredClaim], signer: &str) {
    world.attach(
        index,
        ClaimKind::LayoutPolicy,
        layout_policy_body(claims),
        signer,
    );
}

/// The clone synced for `revision` at `when` on a host trusting `trusted`.
fn synced(
    world: &FakeWorld,
    revision: &Revision,
    when: u64,
    trusted: &[NamedSigner],
) -> CheckoutSynced {
    ok(world.sync_checkout(revision, when, trusted), "the sync")
}

/// The one layout of `synced`, the file projection's, rendered now.
fn refreshed(synced: &CheckoutSynced) -> &Refreshed {
    match synced.layouts.as_slice() {
        [(ecosystem, Ok(Refresh::Refreshed(layout)))] if ecosystem.as_str() == FILE => layout,
        other => panic!("the sync answered the layouts {other:?}, not one rendered"),
    }
}

/// The one layout of `synced`, left as it was.
fn kept(synced: &CheckoutSynced) -> &Kept {
    match synced.layouts.as_slice() {
        [(_, Ok(Refresh::Kept(layout)))] => layout,
        other => panic!("the sync answered the layouts {other:?}, not one left as it was"),
    }
}

/// The pin of `site` at `version` in `layout`.
fn pin<'a>(layout: &'a Refreshed, version: &str) -> &'a PinAnswer {
    layout
        .pins
        .iter()
        .find(|p| p.pin.version.as_str() == version)
        .unwrap_or_else(|| panic!("the layout names no pin of {version}: {layout:?}"))
}

/// The claim `pin` was found wanting and the signers it was checked
/// against.
fn wanting(pin: &PinAnswer) -> (RequiredClaim, Vec<SignerId>) {
    match &pin.answer {
        Err(Not::Refused(r)) => match &r.why {
            Why::Wanting {
                wanting, checked, ..
            } => (wanting.clone(), checked.clone()),
            other => panic!("{:?} was refused for {other:?}", pin.pin),
        },
        other => panic!("{:?} answered {other:?}, not refused", pin.pin),
    }
}

/// The artifacts each render of the file projection was given, by digest,
/// one list per render, in order.
fn renders(world: &FakeWorld) -> Vec<Vec<Digest>> {
    world.fakes.projections[&Ecosystem::new(FILE)]
        .renders()
        .into_iter()
        .map(|(_, artifacts)| artifacts.into_iter().map(|a| a.digest).collect())
        .collect()
}

fn a_lockfiles_versions_and_no_other(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance()]));
    let one = site(world, "1.0.0", true);
    let two = site(world, "1.1.0", true);
    let three = site(world, "1.2.0", true);
    world.lockfile(FILE, &[("site", "1.0.0"), ("site", "1.1.0")]);
    let since = world.fakes.store.calls().len();
    let synced = synced(world, &revision, 1, &[]);

    let layout = refreshed(&synced);
    for (version, (index, artifact)) in [("1.0.0", &one), ("1.1.0", &two)] {
        let held = pin(layout, version)
            .answer
            .as_ref()
            .unwrap_or_else(|not| panic!("site {version} answered {not:?}, not held"));
        assert_eq!(
            (&held.index, &held.artifact),
            (index, &artifact.digest),
            "site {version} is held at other digests than its name resolves to"
        );
    }
    assert_eq!(
        renders(world).last(),
        Some(&vec![one.1.digest.clone(), two.1.digest.clone()]),
        "the layout holds other versions than the lockfile pins"
    );
    let asked: String = world.fakes.store.calls()[since..]
        .iter()
        .map(|c| c.given.to_string())
        .collect();
    for about in ["1.2.0", three.0.as_str(), three.1.digest.as_str()] {
        assert!(
            !asked.contains(about),
            "the store was asked about the version no lockfile pins: {about}"
        );
    }
    let relative = Path::new(&layout.at)
        .strip_prefix(&world.repo().path)
        .unwrap_or_else(|_| panic!("the layout at {} is outside the checkout", layout.at));
    assert_eq!(
        relative,
        Path::new(".tailrace/layouts/file"),
        "the layout's place names more than the checkout"
    );
    assert_eq!(
        world.repo().git(&["status", "--porcelain"]),
        "",
        "the layout shows in the checkout's status"
    );
}

fn no_lockfile_no_layout(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance()]));
    site(world, "1.0.0", true);
    let synced = synced(world, &revision, 1, &[]);
    assert_eq!(
        synced.layouts,
        [],
        "a projection with no lockfile has a layout"
    );
    assert_eq!(world.fakes.policy.calls(), [], "the policy was asked");
    assert_eq!(renders(world), Vec::<Vec<Digest>>::new());
    assert!(
        !world.layouts().path(&Ecosystem::new(FILE)).exists(),
        "a layout of no lockfile was recorded"
    );
}

fn a_pin_the_store_does_not_name(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance()]));
    site(world, "1.0.0", true);
    world.lockfile(FILE, &[("site", "1.0.0"), ("left-pad", "1.3.0")]);
    let synced = synced(world, &revision, 1, &[]);
    let layout = refreshed(&synced);
    assert!(
        pin(layout, "1.0.0").answer.is_ok(),
        "the family version answered {:?}",
        pin(layout, "1.0.0").answer
    );
    match &pin(layout, "1.3.0").answer {
        Err(Not::Absent(a)) => assert!(
            a.what.contains("artifact/file/left-pad@1.3.0"),
            "the third-party pin's absence names {:?}",
            a.what
        ),
        other => panic!("the third-party pin answered {other:?}, not absent"),
    }
    assert!(
        !layout
            .pins
            .iter()
            .any(|p| matches!(p.answer, Err(Not::Refused(_)))),
        "a pin was refused: {layout:?}"
    );
}

fn the_consumers_written_line_at_its_landed_revision(world: &FakeWorld) {
    world.fakes.policy.require(
        Audience::Layout,
        Required::Layout {
            claims: vec![],
            given: false,
            signers: vec![],
        },
    );
    let revision = landed(world, Some(vec![acceptance()]));
    let (index, _) = site(world, "1.0.0", false);
    publishers(world, &index, &[], GATE);
    world.lockfile(FILE, &[("site", "1.0.0")]);
    let synced = synced(world, &revision, 1, &trusts(&[GATE]));

    let layout = refreshed(&synced);
    assert_eq!(
        layout.policy,
        Under::Own {
            revision: revision.clone()
        }
    );
    let refused = pin(layout, "1.0.0");
    assert_eq!(
        refused.policy,
        Some(PolicyOf::Own {
            revision: revision.clone()
        }),
        "the version was judged under another policy than the consumer's line"
    );
    assert_eq!(wanting(refused).0, acceptance());
    let asked: Vec<_> = world
        .fakes
        .policy
        .calls()
        .into_iter()
        .map(|c| (c.given["revision"].clone(), c.given["audience"].clone()))
        .collect();
    assert_eq!(
        asked,
        [(json!(revision), json!("layout"))],
        "the policy was asked other than once for layouts at the landed revision"
    );
}

fn the_publishers_policy_through_the_hosts_trust(world: &FakeWorld) {
    let revision = landed(world, None);
    let gate_or_box = RequiredClaim {
        kind: ClaimKind::Acceptance,
        check: None,
        signers: vec![
            named(GATE, &[ClaimKind::Acceptance]),
            named("box", &[ClaimKind::Acceptance]),
        ],
    };
    let (by_gate, _) = site(world, "1.0.0", true);
    publishers(world, &by_gate, std::slice::from_ref(&gate_or_box), GATE);
    publishers(world, &by_gate, &[], "stranger");
    let (by_box, _) = site(world, "1.1.0", false);
    publishers(world, &by_box, std::slice::from_ref(&gate_or_box), GATE);
    world.attach(&by_box, ClaimKind::Acceptance, Value::Null, "box");
    world.lockfile(FILE, &[("site", "1.0.0"), ("site", "1.1.0")]);
    let synced = synced(world, &revision, 1, &trusts(&[GATE]));

    let layout = refreshed(&synced);
    let held = pin(layout, "1.0.0");
    assert!(
        held.answer.is_ok(),
        "the version under its trusted publisher's policy answered {:?}",
        held.answer
    );
    assert_eq!(
        held.policy,
        Some(PolicyOf::Publisher {
            revision: world.start.clone()
        }),
        "the version was judged under another policy than its trusted publisher's"
    );
    let (claim, checked) = wanting(pin(layout, "1.1.0"));
    assert_eq!(claim.kind, ClaimKind::Acceptance);
    assert_eq!(
        checked,
        [SignerId::new(GATE)],
        "a required claim counted through a signer the host does not trust"
    );
    let strangers = world
        .fakes
        .store
        .claims_of(&by_gate)
        .expect("the index's claims")
        .iter()
        .filter(|e| e.sole().is_some_and(|s| s.signer.as_str() == "stranger"))
        .count();
    assert_eq!(strangers, 1, "the untrusted signer's claim was not kept");
}

fn the_publishers_above_the_consumers_given(world: &FakeWorld) {
    let revision = landed(world, None);
    given_at(world, &revision, vec![acceptance()], acceptance().signers);
    let (published, artifact) = site(world, "1.0.0", false);
    publishers(world, &published, &[provenance()], GATE);
    world.attach(&artifact.digest, ClaimKind::Provenance, json!({}), HOST);
    site(world, "1.1.0", true);
    world.lockfile(FILE, &[("site", "1.0.0"), ("site", "1.1.0")]);
    let synced = synced(world, &revision, 1, &trusts(&[GATE, HOST]));

    let layout = refreshed(&synced);
    assert_eq!(
        layout.policy,
        Under::Publisher,
        "the consumer's given was read as a layout line it writes"
    );
    let publishers = pin(layout, "1.0.0");
    assert!(
        publishers.answer.is_ok(),
        "the version under its trusted publisher's policy answered {:?}",
        publishers.answer
    );
    assert_eq!(
        publishers.policy,
        Some(PolicyOf::Publisher {
            revision: world.start.clone()
        }),
        "the consumer's given was laid above its publisher's policy"
    );
    let givens = pin(layout, "1.1.0");
    assert!(
        givens.answer.is_ok(),
        "the version under the consumer's given answered {:?}",
        givens.answer
    );
    assert_eq!(
        givens.policy,
        Some(PolicyOf::Given {
            revision: Some(revision)
        }),
        "a version with no publisher's policy was judged under another than the consumer's given"
    );
}

fn the_given_where_neither_writes_one(world: &FakeWorld) {
    let revision = landed(world, None);
    let (index, _) = site(world, "1.0.0", true);
    world.lockfile(FILE, &[("site", "1.0.0")]);

    let trusting = synced(world, &revision, 1, &trusts(&[GATE]));
    let held = pin(refreshed(&trusting), "1.0.0");
    assert!(
        held.answer.is_ok(),
        "the version its host's trusted gate accepted answered {:?}",
        held.answer
    );
    assert_eq!(
        held.policy,
        Some(PolicyOf::Given { revision: None }),
        "the version was judged under another policy than the given"
    );
    let path = world.layouts().path(&Ecosystem::new(FILE));
    let text = fs::read_to_string(&path).expect("the record is a file a reader opens");
    let record: Value = serde_json::from_str(&text).expect("the record is JSON");
    assert_eq!(record["held"][0]["index"], json!(index));
    assert_eq!(
        record["held"][0]["policy"],
        json!({ "given": { "revision": null } }),
        "the record names another policy than the given with no revision"
    );

    let trusting_none = synced(world, &revision, 2, &[]);
    let (claim, checked) = wanting(pin(refreshed(&trusting_none), "1.0.0"));
    assert_eq!(
        (claim.kind, claim.check),
        (ClaimKind::Acceptance, None),
        "a host that trusts no signer refused naming another claim than acceptance"
    );
    assert_eq!(checked, [], "a host that trusts no signer checked one");
    assert_eq!(
        renders(world).last(),
        Some(&vec![]),
        "a version no signer the host trusts accepted was rendered"
    );
}

fn a_version_refused_naming_its_claim(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance(), provenance()]));
    let (index, artifact) = site(world, "1.0.0", false);
    world.attach(&artifact.digest, ClaimKind::Provenance, json!({}), HOST);
    world.lockfile(FILE, &[("site", "1.0.0")]);
    let synced = synced(world, &revision, 1, &[]);
    match &pin(refreshed(&synced), "1.0.0").answer {
        Err(Not::Refused(r)) => assert_eq!(
            r.why,
            Why::Wanting {
                version: artifact.meta.clone(),
                index,
                wanting: acceptance(),
                checked: vec![SignerId::new(GATE)],
            },
            "the refusal names another version, index, claim or signers"
        ),
        other => panic!("the version lacking acceptance answered {other:?}"),
    }
    assert_eq!(
        renders(world),
        [Vec::<Digest>::new()],
        "the render was given the refused version"
    );
}

fn a_version_refused_after_it_was_held(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance()]));
    let (_, artifact) = site(world, "1.0.0", true);
    world.lockfile(FILE, &[("site", "1.0.0")]);
    let first = synced(world, &revision, 1, &[]);
    assert!(pin(refreshed(&first), "1.0.0").answer.is_ok());

    let since = world
        .repo()
        .commit(&[("site/index.html", "<p>again</p>")], "fix: again");
    world.repo().push();
    world.land(&since, &[]);
    world.fakes.policy.require_at(
        &since,
        Audience::Layout,
        Required::Layout {
            claims: vec![acceptance(), provenance()],
            given: false,
            signers: vec![],
        },
    );
    let again = synced(world, &since, 2, &[]);
    assert_eq!(wanting(pin(refreshed(&again), "1.0.0")).0, provenance());
    assert_eq!(
        renders(world),
        [vec![artifact.digest], vec![]],
        "the version refused now stayed in the layout"
    );
}

fn a_layout_that_cannot_be_refreshed(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance()]));
    let (index, _) = site(world, "1.0.0", true);
    world.lockfile(FILE, &[("site", "1.0.0")]);
    synced(world, &revision, 1_767_225_600, &[]);
    world.fakes.store.fault(
        StoreOp::Resolve,
        Fault::Unreadable("connection refused".into()),
        Times::UntilHealed,
    );
    let synced = synced(world, &revision, 1_767_312_000, &[]);

    let layout = kept(&synced);
    let holds: Vec<(&str, &Digest)> = layout
        .holds
        .iter()
        .map(|h| (h.version.as_str(), &h.index))
        .collect();
    assert_eq!(
        holds,
        [("1.0.0", &index)],
        "the answer names other versions"
    );
    assert_eq!(layout.rendered_at, Some(1_767_225_600));
    assert_eq!(layout.policy, Some(Under::Own { revision }));
    assert!(
        matches!(&layout.because, Not::Unreadable(u) if u.said.as_deref() == Some("connection refused")),
        "the answer carries other words than the store's: {:?}",
        layout.because
    );
    assert_eq!(
        renders(world).len(),
        1,
        "the projection was asked to render"
    );
    let answer = serde_json::to_string(&synced).expect("the answer is JSON");
    assert!(
        !answer.contains("\"refused\""),
        "an unreachable store's answer names a refusal: {answer}"
    );
}

fn the_record_deleted_and_rebuilt(world: &FakeWorld) {
    let revision = landed(world, Some(vec![acceptance()]));
    let (held, _) = site(world, "1.0.0", true);
    let (refused, _) = site(world, "1.1.0", false);
    world.lockfile(FILE, &[("site", "1.0.0"), ("site", "1.1.0")]);
    synced(world, &revision, 1, &[]);

    let layouts = world.layouts();
    let path = layouts.path(&Ecosystem::new(FILE));
    assert!(
        path.starts_with(world.history().git_dir()),
        "the record at {} is outside the checkout's git directory",
        path.display()
    );
    let read = || -> Value {
        let text = fs::read_to_string(&path).expect("the record is a file a reader opens");
        serde_json::from_str(&text).expect("the record is JSON")
    };
    let mut first = read();
    assert_eq!(first["held"][0]["index"], json!(held));
    assert_eq!(first["refused"][0]["index"], json!(refused));
    assert_eq!(first["refused"][0]["wanting"], json!(acceptance()));
    assert_eq!(first["policy"], json!({ "own": { "revision": revision } }));
    assert_eq!(first["rendered_at"], 1);

    fs::remove_dir_all(layouts.dir()).expect("the record is deleted");
    synced(world, &revision, 2, &[]);
    let mut again = read();
    assert_eq!(
        (first["rendered_at"].take(), again["rendered_at"].take()),
        (json!(1), json!(2))
    );
    assert_eq!(again, first, "the record was rebuilt to another state");
    let renders = renders(world);
    assert_eq!(
        renders[0], renders[1],
        "the layout was rebuilt to other files"
    );
}

/// C.10: a consumer whose file names signers reads under its file alone,
/// the host's trusted signers read for no claim its file names one for.
fn a_consumers_file_reads_under_its_own_signers_alone(world: &FakeWorld) {
    let revision = landed(world, None);
    let gate = named(GATE, &[ClaimKind::Acceptance]);
    let curator = named("curator", &[ClaimKind::LayoutPolicy]);
    given_at(
        world,
        &revision,
        vec![acceptance()],
        vec![gate.clone(), curator.clone()],
    );
    let (by_box, _) = site(world, "1.0.0", false);
    world.attach(&by_box, ClaimKind::Acceptance, Value::Null, "box");
    let (by_gate, _) = site(world, "1.1.0", true);
    publishers(world, &by_gate, &[provenance()], "box");
    let (curated, artifact) = site(world, "1.2.0", false);
    let host_or_box = RequiredClaim {
        kind: ClaimKind::Provenance,
        check: None,
        signers: vec![
            named(HOST, &[ClaimKind::Provenance]),
            named("box", &[ClaimKind::Provenance]),
        ],
    };
    publishers(world, &curated, &[host_or_box], "curator");
    world.attach(&artifact.digest, ClaimKind::Provenance, json!({}), HOST);
    world.lockfile(
        FILE,
        &[("site", "1.0.0"), ("site", "1.1.0"), ("site", "1.2.0")],
    );
    let synced = synced(world, &revision, 1, &trusts(&["box"]));
    let layout = refreshed(&synced);

    let (claim, checked) = wanting(pin(layout, "1.0.0"));
    assert_eq!(claim.kind, ClaimKind::Acceptance);
    assert_eq!(
        checked,
        [SignerId::new(GATE)],
        "acceptance was read through the host's trust beside the gate the file names"
    );
    let held = pin(layout, "1.1.0");
    assert!(
        held.answer.is_ok(),
        "the version the file's gate accepted answered {:?}",
        held.answer
    );
    assert_eq!(
        held.policy,
        Some(PolicyOf::Given {
            revision: Some(revision.clone())
        }),
        "a layout-policy claim by a signer the file does not name for one counted"
    );
    let curated = pin(layout, "1.2.0");
    assert_eq!(
        curated.policy,
        Some(PolicyOf::Publisher {
            revision: world.start.clone()
        }),
        "the layout-policy claim by the signer the file names for one did not count"
    );
    let (claim, checked) = wanting(curated);
    assert_eq!(claim.kind, ClaimKind::Provenance);
    assert_eq!(
        checked,
        [SignerId::new("box")],
        "provenance, which the file names no signer for, was read through another than the host's trust"
    );
}

/// C.10, 118: a claim the consumer's file names no signer for is read
/// through the host's trusted signers, its own among them, so a host
/// consumes what it published itself; the one host 108 admits as the
/// gate is for publishing, not reading.
fn a_claim_the_file_names_no_signer_for_read_through_the_hosts_trust(world: &FakeWorld) {
    let revision = landed(world, None);
    let laptop = named("laptop", &[ClaimKind::Provenance]);
    given_at(
        world,
        &revision,
        vec![RequiredClaim {
            kind: ClaimKind::Acceptance,
            check: None,
            signers: vec![],
        }],
        vec![laptop],
    );
    let (by_laptop, _) = site(world, "1.0.0", false);
    world.attach(&by_laptop, ClaimKind::Acceptance, Value::Null, "laptop");
    let (by_host, _) = site(world, "1.1.0", false);
    world.attach(&by_host, ClaimKind::Acceptance, Value::Null, HOST);
    world.lockfile(FILE, &[("site", "1.0.0"), ("site", "1.1.0")]);
    let synced = synced(world, &revision, 1, &trusts(&[HOST, GATE]));
    let layout = refreshed(&synced);

    let (claim, checked) = wanting(pin(layout, "1.0.0"));
    assert_eq!(
        (claim.kind, checked),
        (
            ClaimKind::Acceptance,
            vec![SignerId::new(HOST), SignerId::new(GATE)]
        ),
        "acceptance was read through the file's one publisher as the gate, not the host's trust"
    );
    let held = pin(layout, "1.1.0");
    assert!(
        held.answer.is_ok(),
        "the version the host accepted itself answered {:?}",
        held.answer
    );
    assert_eq!(
        held.policy,
        Some(PolicyOf::Given {
            revision: Some(revision)
        })
    );
}

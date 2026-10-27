//! `tailrace publish`: one landed revision turned into a version of each
//! component it publishes, in the statement's order: landed, declared,
//! ensured, judged, claimed, packed, put, attested, indexed, named,
//! recorded, answered (14 to 22, rules core.10 to core.13; design D1).
//! Where the repository's file writes a policy for layouts at the landed
//! revision, the index carries it as the publisher's layout-policy claim,
//! attached before the version is named (113, 8); the given is never
//! carried, and a consumer lays it beneath itself (C.10).
//! A publish offers and does nothing more: it never accepts, promotes,
//! moves a channel or reaches a target (20, rule core.10).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, Audience, Branch, BuildId, Claim, ClaimKind, Component,
    ComponentName, Digest, Ecosystem, EcosystemName, Held, IndexMeta, Manifest, Name, Not, Outcome,
    Output, OutputName, Projection, Report, Required, Revision, Seam, Seams, SignerId, Source,
    Verdict, Version, Why,
};

use crate::notes::{self, Rendered};
use crate::{
    claim, derive, layout_policy_body, policy_at, touched, Carried, Declared, Derived, Entry,
    History, Kept, Landed, Noted, Provenance, Versioned, BOUND, LOCAL, NOTES, POLICY_FILE,
};

/// What a publish is asked: the revision, the branch that landed it, the
/// one component to publish or every one the landing touched (124), the
/// identity of the party running it, which signs (11, 79), and the store
/// as the host's bindings name it, which the record keeps (61, 118).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ask {
    pub branch: Branch,
    pub revision: Revision,
    pub component: Option<ComponentName>,
    pub signer: SignerId,
    pub store: Source,
}

/// A publish's answer: per component, in the order the policy file
/// declares them, what it published or why it did not; one component's
/// answer does not stop the next (124).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Publish {
    pub components: Vec<(ComponentName, Answer<Published>)>,
}

/// 19, 57: what a component's publish answers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "published", rename_all = "snake_case")]
pub enum Published {
    /// The version its index is named by, the index and every artifact
    /// with its claims, and the store it was offered to.
    Version {
        component: ComponentName,
        version: Version,
        tag: String,
        revision: Revision,
        branch: Branch,
        index: Digest,
        artifacts: Vec<PublishedArtifact>,
        /// The notes the index names; none for an index offered before
        /// notes were carried (134).
        #[serde(skip_serializing_if = "Option::is_none")]
        notes: Option<PublishedNotes>,
        store: Source,
    },
    /// No commit since the component's last release tag touched it, and
    /// no tag names the revision: nothing was ensured or offered.
    Unchanged {
        component: ComponentName,
        last: Option<String>,
    },
}

/// An artifact a publish offered: its digest, its bytes' digest, its
/// ecosystem and name there, the output and the build identity it came
/// from, whether its projection promises the same bytes again (87), and
/// every claim this publish attached to it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PublishedArtifact {
    pub digest: Digest,
    pub bytes: Digest,
    pub ecosystem: Ecosystem,
    pub name: EcosystemName,
    pub output: OutputName,
    pub build: BuildId,
    pub same_again: bool,
    pub claims: Vec<Attached>,
}

/// 134: a version's notes as a publish answers them: their artifact's
/// digest, their bytes' digest, their text, and every claim this host
/// attached to them (design D5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PublishedNotes {
    pub digest: Digest,
    pub bytes: Digest,
    pub text: String,
    pub claims: Vec<Attached>,
}

/// A claim attached: its kind, its signer, its envelope's digest and the
/// digest it is attached to (8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Attached {
    pub kind: ClaimKind,
    pub signer: SignerId,
    pub envelope: Digest,
    pub on: Digest,
}

/// 57: an answer as a program reads it: `answer` names which of the four
/// it is, with the value or the reason beside it. Every surface renders
/// this one form.
pub fn answer_json<T: Serialize>(answer: &Answer<T>) -> Value {
    match answer {
        Ok(value) => json!({ "answer": "ok", "value": value }),
        Err(Not::Absent(a)) => json!({ "answer": "absent", "absent": a }),
        Err(Not::Refused(r)) => json!({ "answer": "refused", "refused": r }),
        Err(Not::Unreadable(u)) => json!({ "answer": "unreadable", "unreadable": u }),
    }
}

impl Serialize for Publish {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let components: Vec<Value> = self
            .components
            .iter()
            .map(|(component, answer)| {
                let mut value = answer_json(answer);
                value["component"] = json!(component);
                value
            })
            .collect();
        json!({ "components": components }).serialize(serializer)
    }
}

fn bound(seam: Seam) -> Source {
    Source::new(seam, BOUND)
}

/// 14 to 22: publishes `ask.revision`. The landing is asked first and
/// its no ends the publish before anything else is read (rule core.11);
/// then each component the ask names, or the landing touched, is
/// published in the order the policy file declares them (D1).
pub fn publish(seams: &Seams, history: &History, kept: &Kept, ask: &Ask) -> Answer<Publish> {
    let landed = Landed::check(seams.landing.as_ref(), &ask.branch, &ask.revision)?;
    let declared = Declared::read(history, &landed)?;
    let versions: Vec<Derived> = match &ask.component {
        Some(name) => {
            let component = declared
                .components
                .iter()
                .find(|c| c.name == *name)
                .ok_or_else(|| {
                    Not::absent(
                        &Source::history(LOCAL),
                        format!("component {name} in {POLICY_FILE} at {}", ask.revision),
                    )
                })?;
            vec![derive(history, &landed, &declared.rule, component)?]
        }
        None => touched(history, &landed, &declared)?
            .into_iter()
            .map(Derived::Versioned)
            .collect(),
    };

    let mut run = Run {
        seams,
        history,
        kept,
        ask,
        landed: &landed,
        declared: &declared,
        verdicts: None,
        layout: None,
    };
    let components = versions
        .into_iter()
        .map(|derived| match derived {
            Derived::Unchanged { component, last } => (
                component.clone(),
                Ok(Published::Unchanged {
                    component,
                    last: last.map(|t| t.name),
                }),
            ),
            Derived::Versioned(versioned) => {
                let answer = run.component(&versioned);
                (versioned.component, answer)
            }
        })
        .collect();
    Ok(Publish { components })
}

/// One publish's state across its components: the landing's verdicts,
/// read once for the revision (78), and the policy for layouts, read once
/// at it where an index is put (113).
struct Run<'a> {
    seams: &'a Seams,
    history: &'a History,
    kept: &'a Kept,
    ask: &'a Ask,
    landed: &'a Landed,
    declared: &'a Declared,
    verdicts: Option<Answer<Vec<Verdict>>>,
    layout: Option<Answer<Required>>,
}

/// An output with what the build reported of it, the ecosystem and
/// projection that pack it, and the verdicts naming its build.
struct Judged<'a> {
    name: OutputName,
    report: Report,
    ecosystem: Ecosystem,
    projection: &'a dyn Projection,
    verdicts: Vec<Verdict>,
}

/// 84: an output packed as an artifact: the bytes, the metadata the
/// projection gave them, and whether it promised the same bytes again.
struct PackedArtifact {
    bytes: Vec<u8>,
    meta: ArtifactMeta,
    same_again: bool,
}

impl<'a> Run<'a> {
    fn verdicts(&mut self) -> Answer<Vec<Verdict>> {
        let seams = self.seams;
        let revision = self.landed.revision();
        self.verdicts
            .get_or_insert_with(|| seams.landing.verdicts_of(revision).map(|v| v.verdicts))
            .clone()
    }

    /// 113, 8: the layout line the repository's file writes at the landed
    /// revision, signed as the party running the publish about `index`,
    /// naming the revision and no build identity, and attached to it
    /// (rules core.8, core.9; design D8). The given, which a file with no
    /// layout line answers, and a revision with no policy file attach
    /// nothing: a consumer lays the given beneath itself (C.10). A policy
    /// that cannot be read stops the publish before the name moves. An
    /// index already carrying a layout-policy claim of the same body has
    /// nothing signed (18).
    fn layout_policy(&mut self, index: &Digest) -> Answer<()> {
        let (seams, landed) = (self.seams, self.landed);
        let required = self
            .layout
            .get_or_insert_with(|| policy_at(seams.policy.as_ref(), landed, &Audience::Layout))
            .clone();
        let required = match required {
            Ok(Required::Layout { given: true, .. }) | Err(Not::Absent(_)) => return Ok(()),
            Ok(required) => required,
            Err(not) => return Err(not),
        };
        let body = layout_policy_body(required.claims());
        let carried = seams
            .store
            .claims_of(index)?
            .iter()
            .filter_map(Claim::from_envelope)
            .any(|c| c.kind == ClaimKind::LayoutPolicy && c.subject == *index && c.body == body);
        if carried {
            return Ok(());
        }
        let signer = seams
            .signers
            .first()
            .ok_or_else(|| Not::absent(&bound(Seam::Signer), "signer bound"))?;
        let claim = Claim {
            subject: index.clone(),
            kind: ClaimKind::LayoutPolicy,
            revision: landed.revision().clone(),
            build: None,
            body,
        };
        let envelope = signer.sign(&claim, &self.ask.signer)?;
        seams.store.attach(index, &envelope)?;
        Ok(())
    }

    fn component(&mut self, versioned: &Versioned) -> Answer<Published> {
        let component = self
            .declared
            .components
            .iter()
            .find(|c| c.name == versioned.component)
            .expect("a derived component is a declared one");
        let seams = self.seams;
        let revision = self.landed.revision();

        // 3: every output's projection, before the build is asked.
        let mut projections = Vec::new();
        for output in &component.outputs {
            let ecosystem = self
                .declared
                .projection_of(&component.name, output)
                .ok_or_else(|| {
                    Not::refused(
                        &Source::history(LOCAL),
                        Why::NoProjection {
                            component: component.name.clone(),
                            output: output.clone(),
                        },
                        None,
                    )
                })?;
            let projection = seams.projection(ecosystem).ok_or_else(|| {
                Not::absent(&bound(Seam::Host), format!("projection for {ecosystem}"))
            })?;
            projections.push((ecosystem.clone(), projection));
        }

        // 4, 5: the landing's verdicts, then every output in one asking
        // and each one's report (74, 75, 78).
        let verdicts = self.verdicts()?;
        self.seams.build.ensure(revision, &component.outputs)?;
        let mut outputs = Vec::new();
        for (output, (ecosystem, projection)) in component.outputs.iter().zip(projections) {
            let report = self.seams.build.report(revision, output)?;
            outputs.push(Judged {
                name: output.clone(),
                report,
                ecosystem,
                projection,
                verdicts: Vec::new(),
            });
        }

        // 6: judged in declared order, the first that fails refusing the
        // version whole (15, 16, 111).
        for output in &mut outputs {
            output.verdicts = judge(&output.name, &output.report, &verdicts)?;
        }

        // 7: the name claimed after judging and before any pack (22, rule
        // core.13), then the version's notes rendered from the commits it
        // is derived from, so nothing of them exists before the claim
        // (134; design D5).
        let claimed = claim(self.history, self.landed, versioned)?;
        let name = Name::new(&versioned.name);
        let rendered = notes::render(
            self.history,
            &component.name,
            &versioned.version,
            &versioned.name,
            &claimed.revision,
            &Noted::of(versioned),
        )?;

        // 8: a name already held makes this a repeat (18).
        let held = match self.seams.store.resolve(&name) {
            Ok(index) => Some(self.held(&index)?),
            Err(Not::Absent(_)) => None,
            Err(not) => return Err(not),
        };

        // 9: each output packed at its build identity with the version,
        // and a pack that names no metadata refused before anything is
        // offered (84, rule core.12).
        let mut packed = Vec::new();
        for output in &outputs {
            let files = Output {
                name: output.name.clone(),
                files: output.report.files.clone(),
            };
            let answer =
                output
                    .projection
                    .pack(&files, &output.report.build, Some(&versioned.version))?;
            let Some(meta) = answer.meta else {
                return Err(Not::refused(
                    &Source::new(Seam::Projection, output.ecosystem.as_str()),
                    Why::Said,
                    Some(format!(
                        "{} was packed at {} and the answer names no metadata",
                        output.name, versioned.version
                    )),
                ));
            };
            packed.push(PackedArtifact {
                bytes: answer.bytes,
                meta,
                same_again: answer.same_again,
            });
        }

        let (artifacts, (notes, answered)) = match held {
            Some((index, artifacts)) => {
                let (artifacts, notes) =
                    self.again(versioned, &outputs, &packed, &rendered, &artifacts)?;
                return self.record(
                    versioned,
                    component,
                    &claimed.revision,
                    index,
                    artifacts,
                    notes,
                );
            }
            None => self.offer(versioned, &outputs, &packed, &rendered, &claimed.revision)?,
        };

        // 11: the index last, naming the notes after the outputs'
        // artifacts, then the version's name moved from nothing to it (6,
        // 17, 70, 134, rule core.6).
        let meta = IndexMeta::new(
            component.name.clone(),
            versioned.version.clone(),
            artifacts
                .iter()
                .map(|(artifact, _)| artifact.clone())
                .chain(std::iter::once(notes))
                .collect(),
        )
        .map_err(|bad| Not::refused(&bound(Seam::Projection), Why::Said, Some(bad.to_string())))?;
        let index = self
            .seams
            .store
            .put_artifact(&Manifest::Index(meta))?
            .digest;
        self.layout_policy(&index)?;
        self.seams.store.name(&name, &index, None)?;
        let artifacts = artifacts.into_iter().map(|(_, a)| a).collect();
        self.record(
            versioned,
            component,
            &claimed.revision,
            index,
            artifacts,
            Some(answered),
        )
    }

    /// The index a held name names and each artifact it names, with the
    /// digests of its bytes, as the store holds them (72).
    fn held(&self, index: &Digest) -> Answer<(Digest, Vec<(Artifact, Vec<Digest>)>)> {
        let not_a = |what: &str, digest: &Digest| {
            Not::unreadable(
                &self.ask.store,
                format!("{digest} as {what}"),
                Some(format!("{digest} holds no {what}")),
            )
        };
        let Held::Manifest(Manifest::Index(meta)) = self.seams.store.read(index)? else {
            return Err(not_a("an index", index));
        };
        let mut artifacts = Vec::new();
        for artifact in meta.artifacts() {
            let Held::Manifest(Manifest::Artifact { layers, .. }) =
                self.seams.store.read(&artifact.digest)?
            else {
                return Err(not_a("an artifact", &artifact.digest));
            };
            artifacts.push((artifact.clone(), layers));
        }
        Ok((index.clone(), artifacts))
    }

    /// 18, 87: a repeat publish's packs, and the notes rendered again,
    /// compared with what the index holds, before anything is put; all
    /// the same, the artifacts as the index names them with the claims
    /// this host attached to them, read and not signed again (design D5,
    /// D6). An index naming no notes was offered before notes were
    /// carried, and none are added (134).
    fn again(
        &self,
        versioned: &Versioned,
        outputs: &[Judged],
        packed: &[PackedArtifact],
        rendered: &Rendered,
        held: &[(Artifact, Vec<Digest>)],
    ) -> Answer<(Vec<PublishedArtifact>, Option<PublishedNotes>)> {
        let mut same = Vec::new();
        for (output, packed) in outputs.iter().zip(packed) {
            let now = Digest::of(&packed.bytes);
            let found = held.iter().find(|(a, _)| {
                a.meta.ecosystem() == packed.meta.ecosystem() && a.meta.name() == packed.meta.name()
            });
            let differed = |first: Option<Digest>| {
                Not::refused(
                    &bound(Seam::Projection),
                    Why::Differed {
                        artifact: packed.meta.clone(),
                        first,
                        now: now.clone(),
                    },
                    None,
                )
            };
            let Some((artifact, layers)) = found else {
                return Err(differed(None));
            };
            if *layers != [now.clone()] || artifact.meta != packed.meta {
                return Err(differed(layers.first().cloned()));
            }
            same.push((output, packed, artifact.digest.clone(), now));
        }

        let meta = notes_meta(versioned)?;
        let now = Digest::of(rendered.bytes());
        let found = held.iter().find(|(a, _)| {
            a.meta.ecosystem().as_str() == NOTES && a.meta.name().as_str() == meta.name().as_str()
        });
        let notes = match found {
            None => None,
            Some((artifact, layers)) if *layers == [now.clone()] && artifact.meta == meta => {
                Some(artifact.digest.clone())
            }
            Some((_, layers)) => {
                return Err(Not::refused(
                    &Source::history(LOCAL),
                    Why::Differed {
                        artifact: meta,
                        first: layers.first().cloned(),
                        now,
                    },
                    None,
                ))
            }
        };

        let mut artifacts = Vec::new();
        for (output, packed, digest, bytes) in same {
            let claims = self.ours(&digest)?;
            artifacts.push(answered(output, packed, digest, bytes, claims));
        }
        let notes = match notes {
            Some(digest) => Some(PublishedNotes {
                claims: self.ours(&digest)?,
                digest,
                bytes: now,
                text: rendered.text.clone(),
            }),
            None => None,
        };
        Ok((artifacts, notes))
    }

    /// The provenance claims this host attached to `digest`, as the store
    /// holds them.
    fn ours(&self, digest: &Digest) -> Answer<Vec<Attached>> {
        let mut claims = Vec::new();
        for envelope in self.seams.store.claims_of(digest)? {
            let ours = envelope.sole().is_some_and(|s| s.signer == self.ask.signer)
                && Claim::from_envelope(&envelope)
                    .is_some_and(|c| c.kind == ClaimKind::Provenance && c.subject == *digest);
            if ours {
                claims.push(Attached {
                    kind: ClaimKind::Provenance,
                    signer: self.ask.signer.clone(),
                    envelope: envelope.digest(),
                    on: digest.clone(),
                });
            }
        }
        Ok(claims)
    }

    /// `bytes` put and checked against their own digest, then an artifact
    /// of `meta` naming them as its one layer (2, 66, 67, rule core.5);
    /// the artifact's digest and the bytes'.
    fn put(&self, bytes: &[u8], meta: &ArtifactMeta) -> Answer<(Digest, Digest)> {
        let store = &self.seams.store;
        let expected = Digest::of(bytes);
        let answered = store.put_bytes(bytes)?.digest;
        if answered != expected {
            return Err(Not::refused(
                &self.ask.store,
                Why::OtherBytes { expected, answered },
                None,
            ));
        }
        let digest = store
            .put_artifact(&Manifest::Artifact {
                meta: meta.clone(),
                layers: vec![expected.clone()],
            })?
            .digest;
        Ok((digest, expected))
    }

    /// A first publish's offer: each output's bytes and artifact put,
    /// then the notes', each checked against their own digest (2, 66,
    /// 67), then each output's provenance signed and attached beside its
    /// artifact (8, 11 to 13), and the notes' claim last (134; design D3
    /// to D5).
    fn offer(
        &self,
        versioned: &Versioned,
        outputs: &[Judged],
        packed: &[PackedArtifact],
        rendered: &Rendered,
        revision: &Revision,
    ) -> Answer<Offered> {
        let store = &self.seams.store;
        let mut put = Vec::new();
        for packed in packed {
            put.push(self.put(&packed.bytes, &packed.meta)?);
        }
        let meta = notes_meta(versioned)?;
        let (notes, notes_bytes) = self.put(rendered.bytes(), &meta)?;

        let signer = self
            .seams
            .signers
            .first()
            .ok_or_else(|| Not::absent(&bound(Seam::Signer), "signer bound"))?;
        let mut artifacts = Vec::new();
        for ((output, packed), (digest, bytes)) in outputs.iter().zip(packed).zip(put) {
            let provenance = Provenance {
                output: output.name.clone(),
                ecosystem: packed.meta.ecosystem().clone(),
                name: packed.meta.name().clone(),
                version: versioned.version.clone(),
                outcome: output.report.outcome,
                carried: output
                    .verdicts
                    .iter()
                    .map(|v| Carried::from_landing(v, self.landed.revision()))
                    .collect(),
            };
            let claim = provenance.claim(
                digest.clone(),
                revision.clone(),
                output.report.build.clone(),
            );
            let envelope = signer.sign(&claim, &self.ask.signer)?;
            let attached = store.attach(&digest, &envelope)?;
            let claims = vec![Attached {
                kind: ClaimKind::Provenance,
                signer: self.ask.signer.clone(),
                envelope: attached.digest,
                on: digest.clone(),
            }];
            artifacts.push((
                Artifact {
                    digest: digest.clone(),
                    meta: packed.meta.clone(),
                },
                answered(output, packed, digest, bytes, claims),
            ));
        }

        // 134: the notes' claim, the publisher's own, naming the revision
        // and the commits they were rendered from and no build, since no
        // build made them (rules core.8, core.9; design D4).
        let claim = Claim {
            subject: notes.clone(),
            kind: ClaimKind::Provenance,
            revision: revision.clone(),
            build: None,
            body: notes_body(versioned),
        };
        let envelope = signer.sign(&claim, &self.ask.signer)?;
        let attached = store.attach(&notes, &envelope)?;
        let claims = vec![Attached {
            kind: ClaimKind::Provenance,
            signer: self.ask.signer.clone(),
            envelope: attached.digest,
            on: notes.clone(),
        }];
        Ok((
            artifacts,
            (
                Artifact {
                    digest: notes.clone(),
                    meta,
                },
                PublishedNotes {
                    digest: notes,
                    bytes: notes_bytes,
                    text: rendered.text.clone(),
                    claims,
                },
            ),
        ))
    }

    /// 12: the host's record of the version, its last write, then the
    /// answer, each naming the store as the host's bindings name it (61,
    /// 118, design D9); the record names the notes' commits exactly when
    /// the index names notes (134; design D6).
    fn record(
        &self,
        versioned: &Versioned,
        component: &Component,
        revision: &Revision,
        index: Digest,
        artifacts: Vec<PublishedArtifact>,
        notes: Option<PublishedNotes>,
    ) -> Answer<Published> {
        let store = self.ask.store.clone();
        self.kept.write(&Entry {
            component: component.name.clone(),
            version: versioned.version.clone(),
            tag: versioned.name.clone(),
            revision: revision.clone(),
            branch: self.ask.branch.clone(),
            index: index.clone(),
            store: store.clone(),
            notes: notes.as_ref().map(|_| Noted::of(versioned)),
        })?;
        Ok(Published::Version {
            component: component.name.clone(),
            version: versioned.version.clone(),
            tag: versioned.name.clone(),
            revision: revision.clone(),
            branch: self.ask.branch.clone(),
            index,
            artifacts,
            notes,
            store,
        })
    }
}

/// What a first publish offered: each output's artifact with its
/// answer, and the notes' artifact with theirs.
type Offered = (
    Vec<(Artifact, PublishedArtifact)>,
    (Artifact, PublishedNotes),
);

/// The notes' metadata: the ecosystem reserved for them, named by the
/// component, at the version (design D3).
fn notes_meta(versioned: &Versioned) -> Answer<ArtifactMeta> {
    ArtifactMeta::new(
        Ecosystem::new(NOTES),
        EcosystemName::new(versioned.component.as_str()),
        versioned.version.clone(),
    )
    .map_err(|bad| Not::refused(&bound(Seam::Projection), Why::Said, Some(bad.to_string())))
}

/// The body of the notes' claim: the component, the version, the last
/// release tag and every commit the notes were rendered from, by its full
/// revision (design D4).
fn notes_body(versioned: &Versioned) -> Value {
    json!({
        "notes": {
            "component": versioned.component,
            "version": versioned.version,
            "last": versioned.last.as_ref().map(|t| &t.name),
            "commits": versioned.since.iter().map(|c| &c.revision).collect::<Vec<_>>(),
        }
    })
}

fn answered(
    output: &Judged,
    packed: &PackedArtifact,
    digest: Digest,
    bytes: Digest,
    claims: Vec<Attached>,
) -> PublishedArtifact {
    let meta: &ArtifactMeta = &packed.meta;
    PublishedArtifact {
        digest,
        bytes,
        ecosystem: meta.ecosystem().clone(),
        name: meta.name().clone(),
        output: output.name.clone(),
        build: output.report.build.clone(),
        same_again: packed.same_again,
        claims,
    }
}

/// 15, 16, 111 (design D2): an output passes only by the build's pass
/// and, where the landing carries verdicts, a passing verdict naming its
/// build identity. Answers every verdict naming that build, pass or not,
/// to be carried in its provenance (12).
fn judge(output: &OutputName, report: &Report, verdicts: &[Verdict]) -> Answer<Vec<Verdict>> {
    if report.outcome != Outcome::Pass {
        return Err(Not::refused(
            &bound(Seam::Build),
            Why::NotPassed {
                output: output.clone(),
                build: report.build.clone(),
                outcome: report.outcome,
            },
            None,
        ));
    }
    if verdicts.is_empty() {
        return Ok(Vec::new());
    }
    let naming: Vec<Verdict> = verdicts
        .iter()
        .filter(|v| v.build == report.build)
        .cloned()
        .collect();
    if naming.is_empty() {
        let mut carried: Vec<BuildId> = Vec::new();
        for verdict in verdicts {
            if !carried.contains(&verdict.build) {
                carried.push(verdict.build.clone());
            }
        }
        return Err(Not::refused(
            &bound(Seam::Landing),
            Why::Unnamed {
                output: output.clone(),
                build: report.build.clone(),
                carried,
            },
            None,
        ));
    }
    if !naming.iter().any(|v| v.outcome == Outcome::Pass) {
        return Err(Not::refused(
            &bound(Seam::Landing),
            Why::NotPassed {
                output: output.clone(),
                build: report.build.clone(),
                outcome: Outcome::Fail,
            },
            None,
        ));
    }
    Ok(naming)
}

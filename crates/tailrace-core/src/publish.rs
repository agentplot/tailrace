//! `tailrace publish`: one landed revision turned into a version of each
//! component it publishes, in the statement's order: landed, declared,
//! ensured, judged, claimed, packed, put, attested, indexed, named,
//! recorded, answered (14 to 22, rules core.10 to core.13; design D1).
//! A publish offers and does nothing more: it never accepts, promotes,
//! moves a channel or reaches a target (20, rule core.10).

use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use tailrace_seams::{
    Answer, Artifact, ArtifactMeta, Branch, BuildId, ClaimKind, Component, ComponentName, Digest,
    Ecosystem, EcosystemName, Held, IndexMeta, Manifest, Name, Not, Outcome, Output, OutputName,
    Packed, Projection, Report, Revision, Seam, Seams, SignerId, Source, Verdict, Version, Why,
};

use crate::{
    claim, derive, touched, Carried, Declared, Derived, Entry, History, Kept, Landed, Provenance,
    Versioned, BOUND, LOCAL, POLICY_FILE,
};

/// What a publish is asked: the revision, the branch that landed it, the
/// one component to publish or every one the landing touched (124), and
/// the identity of the party running it, which signs (11, 79).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ask {
    pub branch: Branch,
    pub revision: Revision,
    pub component: Option<ComponentName>,
    pub signer: SignerId,
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
/// read once for the revision (78).
struct Run<'a> {
    seams: &'a Seams,
    history: &'a History,
    kept: &'a Kept,
    ask: &'a Ask,
    landed: &'a Landed,
    declared: &'a Declared,
    verdicts: Option<Answer<Vec<Verdict>>>,
}

/// An output with what the build reported of it, the projection that
/// packs it, and the verdicts naming its build.
struct Judged<'a> {
    name: OutputName,
    report: Report,
    projection: &'a dyn Projection,
    verdicts: Vec<Verdict>,
}

impl<'a> Run<'a> {
    fn verdicts(&mut self) -> Answer<Vec<Verdict>> {
        let seams = self.seams;
        let revision = self.landed.revision();
        self.verdicts
            .get_or_insert_with(|| seams.landing.verdicts_of(revision))
            .clone()
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
            projections.push(projection);
        }

        // 4, 5: the landing's verdicts, then every output in one asking
        // and each one's report (74, 75, 78).
        let verdicts = self.verdicts()?;
        self.seams.build.ensure(revision, &component.outputs)?;
        let mut outputs = Vec::new();
        for (output, projection) in component.outputs.iter().zip(projections) {
            let report = self.seams.build.report(revision, output)?;
            outputs.push(Judged {
                name: output.clone(),
                report,
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
        // core.13).
        let claimed = claim(self.history, self.landed, versioned)?;
        let name = Name::new(&versioned.name);

        // 8: a name already held makes this a repeat (18).
        let held = match self.seams.store.resolve(&name) {
            Ok(index) => Some(self.held(&index)?),
            Err(Not::Absent(_)) => None,
            Err(not) => return Err(not),
        };

        // 9: each output packed with the version (84).
        let mut packed = Vec::new();
        for output in &outputs {
            let files = Output {
                name: output.name.clone(),
                files: output.report.files.clone(),
            };
            packed.push(output.projection.pack(&files, &versioned.version)?);
        }

        let artifacts = match held {
            Some((index, artifacts)) => {
                let artifacts = self.again(&outputs, &packed, &artifacts)?;
                return self.record(versioned, component, &claimed.revision, index, artifacts);
            }
            None => self.offer(versioned, &outputs, &packed, &claimed.revision)?,
        };

        // 11: the index last, then the version's name moved from nothing
        // to it (6, 17, 70, rule core.6).
        let meta = IndexMeta::new(
            component.name.clone(),
            versioned.version.clone(),
            artifacts
                .iter()
                .map(|(artifact, _)| artifact.clone())
                .collect(),
        )
        .map_err(|bad| Not::refused(&bound(Seam::Projection), Why::Said, Some(bad.to_string())))?;
        let index = self
            .seams
            .store
            .put_artifact(&Manifest::Index(meta))?
            .digest;
        self.seams.store.name(&name, &index, None)?;
        let artifacts = artifacts.into_iter().map(|(_, a)| a).collect();
        self.record(versioned, component, &claimed.revision, index, artifacts)
    }

    /// The index a held name names and each artifact it names, with the
    /// digests of its bytes, as the store holds them (72).
    fn held(&self, index: &Digest) -> Answer<(Digest, Vec<(Artifact, Vec<Digest>)>)> {
        let not_a = |what: &str, digest: &Digest| {
            Not::unreadable(
                &bound(Seam::Store),
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

    /// 18, 87: a repeat publish's packs compared with what the index
    /// holds, before anything is put; all the same, the artifacts as the
    /// index names them with the claims this host attached to them, read
    /// and not signed again (design D6).
    fn again(
        &self,
        outputs: &[Judged],
        packed: &[Packed],
        held: &[(Artifact, Vec<Digest>)],
    ) -> Answer<Vec<PublishedArtifact>> {
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
        let mut artifacts = Vec::new();
        for (output, packed, digest, bytes) in same {
            let mut claims = Vec::new();
            for envelope in self.seams.store.claims_of(&digest)? {
                let ours = envelope.signature.signer == self.ask.signer
                    && tailrace_seams::Claim::from_envelope(&envelope)
                        .is_some_and(|c| c.kind == ClaimKind::Provenance && c.subject == digest);
                if ours {
                    claims.push(Attached {
                        kind: ClaimKind::Provenance,
                        signer: envelope.signature.signer.clone(),
                        envelope: envelope.digest(),
                        on: digest.clone(),
                    });
                }
            }
            artifacts.push(answered(output, packed, digest, bytes, claims));
        }
        Ok(artifacts)
    }

    /// A first publish's offer: each output's bytes and artifact put,
    /// checked against their own digest (2, 66, 67), then each artifact's
    /// provenance signed and attached beside it (8, 11 to 13).
    fn offer(
        &self,
        versioned: &Versioned,
        outputs: &[Judged],
        packed: &[Packed],
        revision: &Revision,
    ) -> Answer<Vec<(Artifact, PublishedArtifact)>> {
        let store = &self.seams.store;
        let mut put = Vec::new();
        for packed in packed {
            let bytes = Digest::of(&packed.bytes);
            let answered = store.put_bytes(&packed.bytes)?.digest;
            if answered != bytes {
                return Err(Not::refused(
                    &bound(Seam::Store),
                    Why::OtherBytes {
                        expected: bytes,
                        answered,
                    },
                    None,
                ));
            }
            let digest = store
                .put_artifact(&Manifest::Artifact {
                    meta: packed.meta.clone(),
                    layers: vec![bytes.clone()],
                })?
                .digest;
            put.push((digest, bytes));
        }

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
                signer: envelope.signature.signer.clone(),
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
        Ok(artifacts)
    }

    /// 12: the host's record of the version, its last write, then the
    /// answer (61, design D9).
    fn record(
        &self,
        versioned: &Versioned,
        component: &Component,
        revision: &Revision,
        index: Digest,
        artifacts: Vec<PublishedArtifact>,
    ) -> Answer<Published> {
        let store = bound(Seam::Store);
        self.kept.write(&Entry {
            component: component.name.clone(),
            version: versioned.version.clone(),
            tag: versioned.name.clone(),
            revision: revision.clone(),
            branch: self.ask.branch.clone(),
            index: index.clone(),
            store: store.clone(),
        })?;
        Ok(Published::Version {
            component: component.name.clone(),
            version: versioned.version.clone(),
            tag: versioned.name.clone(),
            revision: revision.clone(),
            branch: self.ask.branch.clone(),
            index,
            artifacts,
            store,
        })
    }
}

fn answered(
    output: &Judged,
    packed: &Packed,
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

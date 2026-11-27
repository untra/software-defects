use crate::model::FindingAssessment;
use crate::heap::{Event, Observation, Sharing, WriteFact, ANY_ELEMENT};
use crate::model::{DocumentReport, Effect, Evidence, Finding, Span};
use crate::state::{
    Certainty, Condition, EvidenceLevel, InvalidationStatus, Location, OperationId, Predicate,
    RequirementRef, StateGraph, StateTrace, CONTRACT_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFECT: &str = "aliasing-mutable-defaults";
pub const FAMILY: &str = "mutated-state";
pub const RESOLUTION: &str = "object-identity";
const SYNTAX_RESOLUTION: &str = "candidate";

const MECHANISM_PRIORITY: [Sharing; 6] = [
    Sharing::RepeatedElement,
    Sharing::Reslice,
    Sharing::ShallowCopy,
    Sharing::SharedDefault,
    Sharing::ReturnedInternal,
    Sharing::ReturnedAlias,
];

pub fn explanation(mechanism: Sharing) -> &'static str {
    match mechanism {
        Sharing::RepeatedElement => "Write through one array element changes an object shared by other elements: the elements hold the same reference, not equal copies.",
        Sharing::ShallowCopy => "Write through a shallow copy changes a nested object still shared with the source; only the outer object was copied.",
        Sharing::SharedDefault => "Write to a default value that is shared across invocations; another invocation or owner observes the change.",
        Sharing::ReturnedAlias => "Write through a returned reference changes an object owned outside the callee; the callee returned an alias, not a copy.",
        Sharing::ReturnedInternal => "Write through a returned internal collection changes the owner's state; the accessor exposed its internal reference.",
        Sharing::Reslice => "Append within the shared backing array's capacity overwrites elements visible through the original slice.",
    }
}

fn semantics(mechanism: Sharing) -> &'static str {
    match mechanism {
        Sharing::RepeatedElement => {
            "Repeating or filling with an object stores the same identity in every element."
        }
        Sharing::ShallowCopy => {
            "Shallow copies allocate only the outer container; nested objects keep their identity."
        }
        Sharing::SharedDefault => {
            "The default value's identity is reused instead of being allocated per invocation."
        }
        Sharing::ReturnedAlias | Sharing::ReturnedInternal => {
            "Returning a reference does not copy the referenced object."
        }
        Sharing::Reslice => {
            "A slice that has spare capacity shares its backing array; append writes in place."
        }
    }
}

const ISOLATION_ASSUMPTION: &str = "Isolation is required only where a declared requirement says so; without one, this is a state relationship, not a confirmed defect.";
const DECLARED_ASSUMPTION: &str =
    "Declared immutability of an owner is treated as covering state reachable from it.";

fn matches(
    write: &WriteFact,
    observation: &Observation,
    mechanism: Sharing,
) -> Option<(BTreeSet<OperationId>, Certainty)> {
    let ew = &write.events;
    let er = &observation.events;
    let mut origins = BTreeSet::new();
    let mut certainty = Certainty::Definite;
    for e in ew.iter().filter(|e| e.kind == mechanism) {
        let hit = match mechanism {
            Sharing::RepeatedElement => er
                .iter()
                .find(|f| f.kind == mechanism && f.op == e.op && f.qualifier != e.qualifier)
                .map(|f| {
                    if f.qualifier == ANY_ELEMENT || e.qualifier == ANY_ELEMENT {
                        certainty = Certainty::Possible;
                    }
                }),
            Sharing::Reslice => (write.append && !er.iter().any(|f| f.op == e.op)).then_some(()),
            Sharing::ShallowCopy => (!er.iter().any(|f| f.op == e.op)).then_some(()),
            Sharing::SharedDefault => (!er.iter().any(|f| same(f, e))).then_some(()),
            Sharing::ReturnedAlias | Sharing::ReturnedInternal => {
                (!er.iter().any(|f| same(f, e))).then_some(())
            }
        };
        if hit.is_some() {
            origins.insert(e.op);
        }
    }
    (!origins.is_empty()).then_some((origins, certainty))
}

fn same(a: &Event, b: &Event) -> bool {
    a.op == b.op && a.qualifier == b.qualifier
}

struct Group {
    mechanism: Sharing,
    writes: BTreeSet<OperationId>,
    objects: BTreeSet<OperationId>,
    origins: BTreeSet<OperationId>,
    reads: BTreeSet<OperationId>,
    pairs: Vec<(OperationId, OperationId)>,
    certainty: Certainty,
    boundaries: BTreeSet<usize>,
}

pub fn detect(graph: &mut StateGraph, report: &mut DocumentReport) {
    let heap = graph.heap.clone();
    if heap.language.is_none() {
        return;
    }
    let mut groups: BTreeMap<(Span, Sharing), Group> = BTreeMap::new();
    for observation in &heap.observations {
        let write = &heap.writes[observation.write];
        if write.op == observation.read {
            continue;
        }
        let Some((mechanism, origins, certainty)) = MECHANISM_PRIORITY
            .iter()
            .find_map(|m| matches(write, observation, *m).map(|(o, c)| (*m, o, c)))
        else {
            continue;
        };
        if mechanism == Sharing::SharedDefault && heap.guarded.contains(&write.object) {
            continue;
        }
        let range = graph.operations[write.op].location.range;
        let group = groups.entry((range, mechanism)).or_insert_with(|| Group {
            mechanism,
            writes: BTreeSet::new(),
            objects: BTreeSet::new(),
            origins: BTreeSet::new(),
            reads: BTreeSet::new(),
            pairs: vec![],
            certainty: Certainty::Definite,
            boundaries: BTreeSet::new(),
        });
        group.writes.insert(write.op);
        group.objects.insert(write.object);
        group.origins.extend(origins);
        group.reads.insert(observation.read);
        group.pairs.push((write.op, observation.read));
        let possible =
            certainty == Certainty::Possible || observation.certainty == Certainty::Possible;
        if possible {
            group.certainty = Certainty::Possible;
        }
        if let Some(b) = observation.boundary {
            group.boundaries.insert(b);
        }
    }
    let prefix = format!("{}:{}", graph.snapshot_id, report.uri);
    let mut traces = vec![];
    for ((trigger, _), group) in &groups {
        let mut condition_ids = vec![];
        for (write, read) in &group.pairs {
            let (Some(left), Some(right)) = (
                graph.operations[*write].access_path.clone(),
                graph.operations[*read].access_path.clone(),
            ) else {
                continue;
            };
            let id = format!("{prefix}:condition:independence:{write}:{read}");
            if !graph.conditions.iter().any(|c| c.id == id) {
                let mut evidence = graph.operations[*write].evidence.clone();
                evidence.extend(graph.operations[*read].evidence.clone());
                graph.conditions.push(Condition {
                    id: id.clone(),
                    predicate: Predicate::Independence { left, right },
                    uncertainty: group
                        .boundaries
                        .iter()
                        .map(|b| heap.boundaries[*b].reason.to_string())
                        .collect(),
                    evidence,
                });
            }
            if !graph.operations[*read].required_conditions.contains(&id) {
                graph.operations[*read].required_conditions.push(id.clone());
            }
            condition_ids.push(id);
        }
        traces.push((*trigger, condition_ids));
    }
    for (((trigger, _), group), (_, condition_ids)) in groups.iter().zip(traces) {
        let allocations: BTreeSet<OperationId> = group.objects.iter().copied().collect();
        let mut origin_ops: BTreeSet<OperationId> = allocations.clone();
        origin_ops.extend(&group.origins);
        let requirements: Vec<_> = group
            .objects
            .iter()
            .flat_map(|o| heap.requirements.get(o).cloned().unwrap_or_default())
            .collect();
        let requirement =
            (!requirements.is_empty() && !condition_ids.is_empty()).then(|| RequirementRef {
                condition: condition_ids[0].clone(),
                kind: requirements[0].kind.label().into(),
                declarations: dedup_locations(
                    requirements
                        .iter()
                        .map(|r| Location {
                            uri: report.uri.clone(),
                            range: r.declaration,
                        })
                        .collect(),
                ),
            });
        let feasible = group.certainty == Certainty::Definite && group.boundaries.is_empty();
        let ops: BTreeSet<OperationId> = origin_ops
            .iter()
            .chain(&group.writes)
            .chain(&group.reads)
            .copied()
            .collect();
        let location = |id: &OperationId| graph.operations[*id].location.clone();
        let mut assumptions = vec![
            semantics(group.mechanism).to_string(),
            ISOLATION_ASSUMPTION.to_string(),
        ];
        if requirement.is_some() {
            assumptions.push(DECLARED_ASSUMPTION.into());
        }
        assumptions.extend(
            group
                .boundaries
                .iter()
                .map(|b| heap.boundaries[*b].reason.to_string()),
        );
        assumptions.extend(graph.limitations.iter().cloned());
        assumptions.dedup();
        let trace = StateTrace {
            contract_version: CONTRACT_VERSION,
            snapshot_id: graph.snapshot_id.clone(),
            origin_operations: origin_ops.iter().map(|id| graph.operations[*id].identity.clone()).collect(),
            invalidation_status: if requirement.is_some() {
                InvalidationStatus::Invalidated
            } else {
                InvalidationStatus::PotentiallyInvalidated
            },
            invalidating_operations: group.writes.iter().map(|id| graph.operations[*id].identity.clone()).collect(),
            affected_operations: group.reads.iter().map(|id| graph.operations[*id].identity.clone()).collect(),
            supporting_evidence: ops.iter().flat_map(|id| graph.operations[*id].evidence.clone()).collect(),
            required_conditions: condition_ids.clone(),
            evidence_level: if feasible {
                EvidenceLevel::FeasibleUnderModel
            } else {
                EvidenceLevel::CandidateRelationship
            },
            source_hash: graph.source_hash.clone(),
            origins: dedup_locations(origin_ops.iter().map(location).collect()),
            invalidations: dedup_locations(group.writes.iter().map(location).collect()),
            affected_uses: dedup_locations(group.reads.iter().map(location).collect()),
            required_condition: Some(format!(
                "The object written here must stay independent of the state observed at the affected uses ({}).",
                group.mechanism.mechanism()
            )),
            operations: ops.iter().map(|id| graph.operations[*id].clone()).collect(),
            edges: graph
                .edges
                .iter()
                .filter(|e| ops.contains(&e.from) && ops.contains(&e.to))
                .cloned()
                .collect(),
            assumptions,
            requirement,
        };
        let mut blast: BTreeSet<Span> = group
            .origins
            .iter()
            .map(|id| graph.operations[*id].location.range)
            .collect();
        blast.insert(*trigger);
        blast.extend(
            group
                .reads
                .iter()
                .map(|id| graph.operations[*id].location.range),
        );
        let blast = disjoint(blast);
        let function = report
            .functions
            .iter()
            .filter(|f| f.range.contains(*trigger))
            .min_by_key(|f| f.range.len())
            .map(|f| f.range);
        let escapes = group.objects.iter().any(|o| {
            let allocated = heap.allocation_ranges.get(o).copied();
            match (function, allocated) {
                (Some(f), Some(a)) => !f.contains(a),
                _ => false,
            }
        });
        let rule_id = format!("{}.{FAMILY}.{DEFECT}", report.language);
        report.findings.push(Finding {
        assessment: FindingAssessment::Candidate,
        shared_state: None,
            state_trace: Some(trace),
            division_risk: None,
            rule_id: rule_id.clone(),
            family: FAMILY.into(),
            compound: None,
            defect: DEFECT.into(),
            mechanism: Some(group.mechanism.mechanism().into()),
            blast_radius: blast,
            trigger: *trigger,
            function,
            effect: if escapes {
                Effect::GlobalMutation
            } else {
                Effect::LocalMutation
            },
            origin: "direct".into(),
            resolution: RESOLUTION.into(),
            explanation: explanation(group.mechanism).into(),
            evidence: vec![Evidence {
                uri: report.uri.clone(),
                range: *trigger,
                rule_id: rule_id.clone(),
            }],
            call_path: vec![],
            related_cwes: crate::analysis::related_cwes(&rule_id),
        });
    }
    suppress_modeled_signals(graph, report);
}

fn dedup_locations(mut locations: Vec<Location>) -> Vec<Location> {
    locations.sort_by_key(|l| (l.uri.clone(), l.range));
    locations.dedup_by(|a, b| a.uri == b.uri && a.range == b.range);
    locations
}

fn disjoint(spans: BTreeSet<Span>) -> Vec<Span> {
    let mut out: Vec<Span> = vec![];
    for span in spans {
        if out.iter().any(|s| s.contains(span)) {
            continue;
        }
        out.retain(|s| !span.contains(*s));
        out.push(span);
    }
    out.sort();
    out
}

fn suppress_modeled_signals(graph: &StateGraph, report: &mut DocumentReport) {
    let boundary_spans: Vec<Span> = graph
        .heap
        .boundaries
        .iter()
        .map(|b| graph.operations[b.op].location.range)
        .collect();
    let semantic: Vec<Span> = report
        .findings
        .iter()
        .filter(|f| f.resolution == RESOLUTION)
        .flat_map(|f| f.blast_radius.clone())
        .collect();
    let modeled = |trigger: Span| {
        let explained = semantic
            .iter()
            .any(|s| s.start < trigger.end && trigger.start < s.end);
        !explained
            && graph.heap.modeled.iter().any(|m| m.contains(trigger))
            && !boundary_spans
                .iter()
                .any(|b| trigger.contains(*b) || b.contains(trigger))
    };
    let removed: Vec<(Span, String)> = report
        .findings
        .iter()
        .filter(|f| f.defect == DEFECT && f.resolution == SYNTAX_RESOLUTION && modeled(f.trigger))
        .map(|f| (f.trigger, f.explanation.clone()))
        .collect();
    report.findings.retain(|f| {
        !(f.defect == DEFECT && f.resolution == SYNTAX_RESOLUTION && modeled(f.trigger))
    });
    report
        .unresolved
        .retain(|u| !removed.iter().any(|(t, e)| *t == u.range && *e == u.reason));
}

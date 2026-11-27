use crate::model::*;
use crate::state::{EvidenceLevel, Location, StateTrace};
use crate::state_contract::{InvalidationStatus, RequirementRef};
use tree_sitter::Node;

pub(crate) const RESOLUTION: &str = "unstructured-input-output";
const FAMILY: &str = "unstructured-state";
const RETURNS: &str = "missing-undefined-returns";
const BOUNDARY_OUTPUT: &str = "boundary-output";
const LIMIT: &str = "Bounded syntax and local binding relationships; application contracts, dynamic dispatch and arbitrary control flow are not proven.";

fn children(n: Node<'_>) -> Vec<Node<'_>> {
    n.named_children(&mut n.walk()).collect()
}
fn text<'a>(n: Node<'_>, source: &'a str) -> &'a str {
    &source[n.byte_range()]
}
fn owner(report: &DocumentReport, at: Span) -> Option<Span> {
    report
        .functions
        .iter()
        .filter(|f| f.range.contains(at))
        .min_by_key(|f| f.range.len())
        .map(|f| f.range)
}
fn callable(n: Node<'_>) -> bool {
    matches!(
        n.kind(),
        "function_declaration"
            | "function_expression"
            | "arrow_function"
            | "function_definition"
            | "function_item"
            | "closure_expression"
            | "func_literal"
            | "method_definition"
            | "method_declaration"
    )
}
fn local_nodes(n: Node<'_>) -> Vec<Node<'_>> {
    let mut result = vec![];
    for child in children(n) {
        if callable(child) {
            continue;
        }
        result.push(child);
        result.extend(local_nodes(child));
    }
    result
}
fn trace(
    report: &DocumentReport,
    origins: &[Span],
    uses: &[Span],
    invalidations: &[Span],
    condition: &str,
) -> StateTrace {
    let locations = |spans: &[Span]| {
        spans
            .iter()
            .map(|&range| Location {
                uri: report.uri.clone(),
                range,
            })
            .collect()
    };
    StateTrace {
        contract_version: crate::state::CONTRACT_VERSION,
        snapshot_id: report.source_hash.clone(),
        origin_operations: vec![],
        invalidation_status: if invalidations.is_empty() {
            InvalidationStatus::Unknown
        } else {
            InvalidationStatus::PotentiallyInvalidated
        },
        invalidating_operations: vec![],
        affected_operations: vec![],
        supporting_evidence: vec![],
        required_conditions: vec![],
        evidence_level: EvidenceLevel::CandidateRelationship,
        source_hash: report.source_hash.clone(),
        origins: locations(origins),
        invalidations: locations(invalidations),
        affected_uses: locations(uses),
        required_condition: Some(condition.into()),
        operations: vec![],
        edges: vec![],
        assumptions: vec![LIMIT.into()],
        requirement: None,
    }
}
pub(crate) fn add(
    report: &mut DocumentReport,
    at: Span,
    defect: &str,
    mechanism: &str,
    explanation: &str,
    origins: &[Span],
) {
    if report
        .findings
        .iter()
        .any(|f| f.resolution == RESOLUTION && f.defect == defect && f.trigger == at)
    {
        return;
    }
    let rule_id = format!("{}.{FAMILY}.{defect}", report.language);
    let mut evidence_trace = trace(report, origins, &[at], &[], explanation);
    evidence_trace.required_condition = None;
    report.findings.push(Finding {
        assessment: if matches!(mechanism, "boundary-input" | "boundary-output") { FindingAssessment::Usage } else { FindingAssessment::Candidate },
        shared_state: None,
        state_trace: Some(evidence_trace),
        division_risk: None,
        rule_id: rule_id.clone(),
        family: FAMILY.into(),
        compound: None,
        defect: defect.into(),
        mechanism: Some(mechanism.into()),
        blast_radius: vec![at],
        trigger: at,
        function: owner(report, at),
        effect: Effect::StructuralSignal,
        origin: "direct".into(),
        resolution: RESOLUTION.into(),
        explanation: explanation.into(),
        evidence: origins
            .iter()
            .copied()
            .chain(std::iter::once(at))
            .map(|range| Evidence {
                uri: report.uri.clone(),
                range,
                rule_id: rule_id.clone(),
            })
            .collect(),
        call_path: vec![],
        related_cwes: if matches!(defect, "insecure-deserialization" | "index-out-of-bounds" | "type-errors") { crate::analysis::related_cwes(&rule_id) } else { vec![] },
    });
}

pub(crate) fn usage(report: &mut DocumentReport, at: Span, defect: &str, mechanism: &str, explanation: &str) {
    add(report, at, defect, mechanism, explanation, &[]);
    if let Some(finding) = report.findings.iter_mut().find(|f| f.defect == defect && f.trigger == at && f.mechanism.as_deref() == Some(mechanism)) {
        finding.assessment = FindingAssessment::Usage;
    }
}

#[derive(Default, Clone, Copy)]
struct Exits {
    open: bool,
    value: bool,
    empty: bool,
    unknown: bool,
}
fn flow(n: Node<'_>, source: &str, resolve: &impl Fn(&str, Span) -> Option<Span>) -> Exits {
    match n.kind() {
        "return_statement" => {
            let absent = n.named_child_count() == 0
                || n.named_child(0).is_some_and(|value| {
                    matches!(value.kind(), "none" | "null")
                        || matches!(value.kind(), "identifier" | "undefined")
                            && text(value, source) == "undefined"
                            && resolve("undefined", Span::of(value)).is_none()
                });
            Exits {
                value: !absent,
                empty: absent,
                ..Exits::default()
            }
        }
        "throw_statement" | "raise_statement" => Exits::default(),
        "statement_block" | "block" => {
            let mut result = Exits {
                open: true,
                ..Exits::default()
            };
            for child in children(n) {
                if !result.open {
                    break;
                }
                let next = flow(child, source, resolve);
                result.open = next.open;
                result.value |= next.value;
                result.empty |= next.empty;
                result.unknown |= next.unknown;
            }
            result
        }
        "if_statement" | "elif_clause" => {
            let yes = n
                .child_by_field_name("consequence")
                .map(|n| flow(n, source, resolve))
                .unwrap_or(Exits {
                    open: true,
                    unknown: true,
                    ..Exits::default()
                });
            let alternatives: Vec<_> = children(n)
                .into_iter()
                .filter(|c| matches!(c.kind(), "else_clause" | "elif_clause"))
                .collect();
            let no = n
                .child_by_field_name("alternative")
                .or_else(|| alternatives.first().copied())
                .map(|n| flow(n, source, resolve))
                .unwrap_or(Exits {
                    open: true,
                    ..Exits::default()
                });
            Exits {
                open: yes.open || no.open,
                value: yes.value || no.value,
                empty: yes.empty || no.empty,
                unknown: yes.unknown || no.unknown || alternatives.len() > 1,
            }
        }
        "else_clause" => children(n)
            .last()
            .copied()
            .map(|n| flow(n, source, resolve))
            .unwrap_or(Exits {
                open: true,
                ..Exits::default()
            }),
        "for_statement" | "for_in_statement" | "while_statement" | "do_statement"
        | "labeled_statement" | "try_statement" | "switch_statement" | "match_statement"
        | "with_statement" => Exits {
            open: true,
            unknown: true,
            ..Exits::default()
        },
        _ => Exits {
            open: true,
            ..Exits::default()
        },
    }
}
fn producer(
    n: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
) {
    let Some(body) = n.child_by_field_name("body") else {
        return;
    };
    if report.language == "rust" && n.child_by_field_name("return_type").is_some() {
        if let Some(tail) = children(body).last().copied().filter(|tail| {
            tail.kind() != "expression_statement" && tail.kind() != "let_declaration"
        }) {
            let output = std::iter::once(tail).chain(local_nodes(tail)).any(|part| {
                part.kind() == "identifier" && text(part, source) == "None"
                    || part.kind() == "call_expression"
                        && part.child_by_field_name("function").is_some_and(|callee| {
                            text(callee, source) == "Ok"
                                || callee.child_by_field_name("field").is_some_and(|field| {
                                    matches!(text(field, source), "unwrap_or" | "unwrap_or_default")
                                })
                        })
            });
            if output {
                add(report, Span::of(tail), RETURNS, "producer-output", "Producer tail expression supplies optional or default output. Verify its application contract; explicit absence and fallback policies can be correct.", &[Span::of(tail)]);
            }
        }
        return;
    }
    if !matches!(
        report.language.as_str(),
        "javascript" | "typescript" | "python"
    ) {
        return;
    }
    if !matches!(body.kind(), "statement_block" | "block") {
        return;
    }
    let nodes = local_nodes(body);
    if nodes
        .iter()
        .any(|n| matches!(n.kind(), "yield" | "yield_expression"))
    {
        return;
    }
    let result = flow(body, source, resolve);
    let declared = n
        .child_by_field_name("return_type")
        .map(|t| text(t, source).trim().trim_start_matches(':').trim());
    let required = declared.is_some_and(|t| {
        [
            "number",
            "string",
            "boolean",
            "Promise<number>",
            "Promise<string>",
            "Promise<boolean>",
            "int",
            "str",
            "float",
            "bool",
        ]
        .contains(&t)
    });
    let optional = declared.is_some_and(|t| {
        ["void", "None", "Any", "unknown", "Promise<void>"].contains(&t)
            || t.contains("None")
            || t.contains("undefined")
            || t.contains("Optional")
            || t.contains("null")
    });
    let callback = n
        .parent()
        .filter(|p| p.kind() == "arguments")
        .and_then(|p| p.parent())
        .and_then(|p| p.child_by_field_name("function"))
        .is_some_and(|f| text(f, source).ends_with(".map"));
    if optional || !(result.open || result.empty) || !(result.value || required || callback) {
        return;
    }
    let signature = Span {
        start: n.start_byte(),
        end: body.start_byte(),
    };
    add(
        report,
        signature,
        RETURNS,
        "producer-output",
        if result.unknown {
            "Producer has a value-returning contract and paths requiring review. Return completeness is unresolved for this control flow."
        } else {
            "Producer has a value-returning contract or branch, but a reachable path supplies no value. Intentional optional output remains valid; verify the producer contract."
        },
        &[signature],
    );
    if let Some(finding) = report.findings.last_mut() {
        finding.function = Some(Span::of(n));
        if required && !result.unknown {
            let trace = finding.state_trace.as_mut().unwrap();
            trace.evidence_level = EvidenceLevel::FeasibleUnderModel;
            trace.required_condition =
                Some("Every normal exit produces the declared nonoptional result.".into());
            trace.requirement = Some(RequirementRef {
                condition: "Every normal exit produces the declared nonoptional result.".into(),
                kind: "producer-result".into(),
                declarations: vec![Location {
                    uri: report.uri.clone(),
                    range: signature,
                }],
            });
        }
    }
}

pub(crate) fn analyze<F: Fn(&str, Span) -> Option<Span>>(
    captures: &[(&str, &str, Node<'_>)],
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    imports: &[(String, String, Option<String>)],
    resolve: F,
) {
    for &(path, capture, node) in captures {
        if !capture.starts_with("unstructured_") || node.has_error() {
            continue;
        }
        let defect = path.split('/').nth(2).unwrap_or_default();
        let at = Span::of(node);
        match capture {
            "unstructured_function" => producer(node, source, report, &resolve),
            "unstructured_parameter" if defect == "lack-of-input-validation" => {
                add(report, at, defect, "boundary-input", "Function input boundary: static types do not establish every runtime shape or domain requirement. Validation may be intentional or supplied by the caller.", &[at]);
            }
            "unstructured_boundary" => {
                let callee = node.child_by_field_name("function");
                if callee.is_some_and(|c| c.kind() == "identifier" && (resolve(text(c, source), Span::of(c)).is_some() || imports.iter().any(|(local, _, _)| local == text(c, source)))) { continue; }
                if report.language == "python" && callee.is_some_and(|c| crate::annotations::nodes(root).iter().any(|n| matches!(n.kind(), "import_statement" | "import_from_statement") && crate::annotations::nodes(*n).iter().any(|part| part.kind() == "identifier" && text(*part, source) == text(c, source)))) { continue; }
                add(report, at, defect, "boundary-input", "Parsing or input boundary candidate. Successful decoding does not establish the application's domain contract; API identity and error handling require review.", &[at]);
            }
            "unstructured_assertion" => {
                if text(node, source).ends_with("as const") { continue; }
                add(report, at, defect, "unchecked-contract", "Type or capability assumption candidate. Establish the required runtime contract before relying on this value; a cast or annotation alone is not validation.", &[at]);
            }
            "unstructured_resource" => add(report, at, defect, "resource-input", "Input-dependent resource use candidate. Establish an applicable size or expansion budget before consumption; method names alone do not prove unbounded work.", &[at]),
            "unstructured_return" if matches!(report.language.as_str(), "go" | "rust") => {
                let mut parent = node.parent();
                while parent.is_some_and(|n| !callable(n)) { parent = parent.and_then(|n| n.parent()); }
                let output_type = parent.and_then(|n| n.child_by_field_name("result").or_else(|| n.child_by_field_name("return_type")));
                if output_type.is_none_or(|n| text(n, source) == "()") { continue; }
                let empty = node.named_child_count() == 0;
                let candidate = local_nodes(node).iter().any(|n| n.kind() == "nil" || n.kind() == "identifier" && text(*n, source) == "None" || n.kind() == "call_expression" && n.child_by_field_name("function").is_some_and(|callee| text(callee, source) == "Ok"));
                if empty || candidate {
                    add(report, at, RETURNS, "producer-output", "Producer output candidate: verify that empty or default success preserves the declared application contract. This language prevents ordinary missing non-unit returns.", &[at]);
                }
            }
            _ => {}
        }
    }
    crate::deserialization::analyze(captures, root, source, report, &resolve);
    crate::indexing::analyze(captures, root, source, report, &resolve);
    crate::type_errors::analyze(captures, root, source, report, &resolve);
    relationships(root, source, report, resolve);
}

fn relationships<F: Fn(&str, Span) -> Option<Span>>(
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: F,
) {
    let all = crate::annotations::nodes(root);
    let sources = report
        .findings
        .iter()
        .filter(|f| f.resolution == RESOLUTION && f.defect != RETURNS)
        .cloned()
        .collect::<Vec<_>>();
    for finding in sources {
        let Some(node) = all
            .iter()
            .find(|n| Span::of(**n) == finding.trigger)
            .copied()
        else {
            continue;
        };
        let name = input_name(node);
        let Some(name) = name else {
            continue;
        };
        let Some(binding) = resolve(text(name, source), Span::of(name)) else {
            continue;
        };
        let mut identities = vec![(binding, name.end_byte(), usize::MAX)];
        let mut uncertain = vec![];
        for &n in &all {
            if n.start_byte() < name.end_byte() || owner(report, Span::of(n)) != finding.function {
                continue;
            }
            if let Some(left) = assigned_target(n) {
                if left.kind() != "identifier" {
                    continue;
                }
                let Some(target) = resolve(text(left, source), Span::of(left)) else {
                    continue;
                };
                if conditional(n) {
                    uncertain.push((target, n.start_byte()));
                    continue;
                }
                let value = n
                    .child_by_field_name("value")
                    .or_else(|| n.child_by_field_name("right"));
                let from = value
                    .filter(|v| v.kind() == "identifier")
                    .and_then(|v| resolve(text(v, source), Span::of(v)));
                let alias = from.is_some_and(|from| {
                    identities.iter().any(|(id, start, end)| {
                        *id == from && *start <= n.start_byte() && n.start_byte() < *end
                    })
                });
                for (id, _, end) in &mut identities {
                    if *id == target {
                        *end = (*end).min(n.start_byte());
                    }
                }
                if alias {
                    identities.push((target, n.end_byte(), usize::MAX));
                }
            }
        }
        let relevant = |n: Node<'_>| {
            resolve(text(n, source), Span::of(n)).is_some_and(|id| {
                identities.iter().any(|(b, start, end)| {
                    *b == id && *start <= n.start_byte() && n.start_byte() < *end
                })
            })
        };
        let uses: Vec<_> = all
            .iter()
            .filter(|n| {
                n.kind() == "identifier"
                    && owner(report, Span::of(**n)) == finding.function
                    && relevant(**n)
            })
            .map(|n| Span::of(*n))
            .collect();
        if let Some(target) = report
            .findings
            .iter_mut()
            .find(|f| f.rule_id == finding.rule_id && f.trigger == finding.trigger)
        {
            target.blast_radius.extend(uses.iter().copied());
            if let Some(trace) = &mut target.state_trace {
                trace
                    .affected_uses
                    .extend(uses.iter().map(|&range| Location {
                        uri: report.uri.clone(),
                        range,
                    }));
                if !uncertain.is_empty() {
                    trace.assumptions.push("Conditional rebinding is unresolved; provenance after that boundary remains a possibility, not an established relationship.".into());
                }
            }
        }
        let relevant = |n: Node<'_>| {
            relevant(n)
                && !resolve(text(n, source), Span::of(n)).is_some_and(|id| {
                    uncertain
                        .iter()
                        .any(|(binding, start)| *binding == id && *start <= n.start_byte())
                })
        };
        if finding.mechanism.as_deref() == Some("boundary-input") {
            let returns: Vec<_> = all
                .iter()
                .filter(|n| {
                    matches!(n.kind(), "return_statement" | "return_expression")
                        && owner(report, Span::of(**n)) == finding.function
                })
                .copied()
                .collect();
            for returned in returns {
                let Some(mut value) = returned.named_child(0) else {
                    continue;
                };
                if value.kind() == "expression_list" && value.named_child_count() == 1 {
                    value = value.named_child(0).unwrap();
                }
                if value.kind() != "identifier"
                    || !relevant(value)
                    || resolve(text(value, source), Span::of(value)) != Some(binding)
                {
                    continue;
                }
                add(report, Span::of(returned), "lack-of-input-validation", BOUNDARY_OUTPUT, "Producer returns an input value directly. Its input contract remains relevant to callers; validation may already be intentional and correct.", &[finding.trigger]);
            }
        }
        let guards: Vec<_> = all
            .iter()
            .filter(|n| {
                n.kind() == "if_statement" && owner(report, Span::of(**n)) == finding.function
            })
            .copied()
            .collect();
        for guard in guards {
            let Some(condition) = guard.child_by_field_name("condition") else {
                continue;
            };
            let Some(mut path) = access_in(condition, source, &relevant) else {
                continue;
            };
            let resource_bound = path.ends_with(".length")
                || path.ends_with(".len")
                || text(condition, source).contains("len(");
            if resource_bound {
                path = path
                    .trim_end_matches(".length")
                    .trim_end_matches(".len")
                    .to_string();
            }
            let Some(consequence) = guard.child_by_field_name("consequence") else {
                continue;
            };
            let rejection = flow(consequence, source, &resolve);
            let after = if !rejection.open && !rejection.unknown {
                guard.end_byte()
            } else {
                consequence.start_byte()
            };
            let end = if !rejection.open && !rejection.unknown {
                finding.function.map_or(root.end_byte(), |f| f.end)
            } else {
                consequence.end_byte()
            };
            for &write in &all {
                if write.start_byte() < after || write.end_byte() > end {
                    continue;
                }
                let shared = report
                    .findings
                    .iter()
                    .find(|f| {
                        f.shared_state.as_ref().is_some_and(|s| {
                            s.events.iter().any(|e| {
                                e.kind == "shared-write" && e.range.contains(Span::of(write))
                            })
                        })
                    })
                    .cloned();
                if owner(report, Span::of(write)) != finding.function && shared.is_none() {
                    continue;
                }
                let Some(target) = mutation_target(write) else {
                    continue;
                };
                let Some(changed) = access_in(target, source, &relevant) else {
                    continue;
                };
                if !same_path(&path, &changed) {
                    continue;
                }
                let Some(mutation) = report
                    .findings
                    .iter()
                    .find(|f| {
                        f.family == "mutated-state"
                            && f.trigger.contains(Span::of(write))
                            && f.effect != Effect::StructuralSignal
                    })
                    .cloned()
                else {
                    continue;
                };
                let affected = all.iter().find(|n| {
                    n.start_byte() >= write.end_byte()
                        && n.end_byte() <= end
                        && owner(report, Span::of(**n)) == finding.function
                        && !in_condition(**n)
                        && matches!(
                            n.kind(),
                            "member_expression"
                                | "attribute"
                                | "selector_expression"
                                | "field_expression"
                                | "subscript_expression"
                                | "index_expression"
                        )
                        && access_in(**n, source, &relevant)
                            .is_some_and(|use_path| same_path(&path, &use_path))
                });
                let Some(affected) = affected else {
                    continue;
                };
                let at = Span::of(*affected);
                let rechecked = all.iter().any(|n| {
                    if n.kind() != "if_statement"
                        || n.parent() != guard.parent()
                        || n.start_byte() < write.end_byte()
                        || n.end_byte() > at.start
                        || owner(report, Span::of(*n)) != finding.function
                    {
                        return false;
                    }
                    let Some(check) = n.child_by_field_name("condition") else {
                        return false;
                    };
                    if text(check, source) != text(condition, source) {
                        return false;
                    }
                    n.child_by_field_name("consequence").is_some_and(|body| {
                        let exits = flow(body, source, &resolve);
                        !exits.open && !exits.unknown
                    })
                });
                if rechecked {
                    continue;
                }
                if report
                    .findings
                    .iter()
                    .any(|f| f.mechanism.as_deref() == Some("guard-invalidated") && f.trigger == at)
                {
                    continue;
                }
                let mut compound = finding.clone();
                compound.family = "compound-defects".into();
                compound.defect = "shape-drift".into();
                compound.rule_id =
                    format!("{}.compound-defects.{}", report.language, compound.defect);
                compound.mechanism = Some("guard-invalidated".into());
                compound.trigger = at;
                compound.blast_radius = vec![Span::of(condition), Span::of(write), at];
                compound.explanation = if shared.is_some() { "A launched scope can mutate a checked input path before a later use. The earlier condition may no longer hold; runtime scheduling and synchronization are unresolved." } else { "A check and a later use refer to the same input path, with a connected mutation between them. The earlier condition may no longer hold; this relationship does not prove a contract violation." }.into();
                compound.state_trace = Some(trace(
                    report,
                    &[finding.trigger, Span::of(condition)],
                    &[at],
                    &[Span::of(write)],
                    "Re-establish the required condition after the connected mutation.",
                ));
                let ingredient = shared.as_ref().unwrap_or(&mutation);
                compound.compound = Some(Compound {
                    class: "runtime-errors".into(),
                    category: compound.defect.clone(),
                    ingredients: [
                        Evidence {
                            uri: report.uri.clone(),
                            range: finding.trigger,
                            rule_id: finding.rule_id.clone(),
                        },
                        Evidence {
                            uri: report.uri.clone(),
                            range: ingredient.trigger,
                            rule_id: ingredient.rule_id.clone(),
                        },
                    ],
                });
                report.findings.push(compound);
            }
        }
    }
}

fn input_name(n: Node<'_>) -> Option<Node<'_>> {
    if n.kind() == "identifier" {
        return Some(n);
    }
    if let Some(name) = n
        .child_by_field_name("name")
        .or_else(|| n.child_by_field_name("pattern"))
    {
        if name.kind() == "identifier" {
            return Some(name);
        }
    }
    if matches!(
        n.kind(),
        "typed_parameter" | "default_parameter" | "typed_default_parameter"
    ) {
        return n.named_child(0).filter(|n| n.kind() == "identifier");
    }
    if matches!(
        n.kind(),
        "call" | "call_expression" | "as_expression" | "type_assertion"
    ) {
        let mut current = n;
        while let Some(parent) = current.parent() {
            if let Some(target) = assigned_target(parent) {
                return (target.kind() == "identifier").then_some(target);
            }
            if !matches!(
                parent.kind(),
                "await_expression"
                    | "await"
                    | "parenthesized_expression"
                    | "expression_list"
                    | "as_expression"
            ) {
                break;
            }
            current = parent;
        }
    }
    None
}
fn assigned_target(n: Node<'_>) -> Option<Node<'_>> {
    match n.kind() {
        "variable_declarator" => n.child_by_field_name("name"),
        "let_declaration" => n.child_by_field_name("pattern"),
        "assignment"
        | "assignment_expression"
        | "assignment_statement"
        | "short_var_declaration" => n.child_by_field_name("left"),
        _ => None,
    }
}
fn mutation_target(n: Node<'_>) -> Option<Node<'_>> {
    if matches!(
        n.kind(),
        "assignment"
            | "assignment_expression"
            | "assignment_statement"
            | "augmented_assignment"
            | "augmented_assignment_expression"
            | "compound_assignment_expr"
    ) {
        return n.child_by_field_name("left");
    }
    if matches!(n.kind(), "call" | "call_expression") {
        let callee = n.child_by_field_name("function")?;
        let method = callee
            .child_by_field_name("property")
            .or_else(|| callee.child_by_field_name("attribute"))
            .or_else(|| callee.child_by_field_name("field"))?;
        let _ = method;
        return callee
            .child_by_field_name("object")
            .or_else(|| callee.child_by_field_name("value"))
            .or_else(|| callee.child_by_field_name("operand"));
    }
    None
}
fn access_in(n: Node<'_>, source: &str, relevant: &impl Fn(Node<'_>) -> bool) -> Option<String> {
    if n.kind() == "identifier" && relevant(n) {
        return Some(String::new());
    }
    for child in children(n) {
        if let Some(mut path) = access_in(child, source, relevant) {
            if matches!(
                n.kind(),
                "member_expression" | "attribute" | "selector_expression" | "field_expression"
            ) {
                let field = n
                    .child_by_field_name("property")
                    .or_else(|| n.child_by_field_name("attribute"))
                    .or_else(|| n.child_by_field_name("field"))?;
                path.push('.');
                path.push_str(text(field, source));
            } else if matches!(
                n.kind(),
                "subscript_expression" | "index_expression" | "subscript"
            ) {
                return None;
            }
            return Some(path);
        }
    }
    None
}
fn same_path(a: &str, b: &str) -> bool {
    a == b
        || a.is_empty()
        || b.is_empty()
        || a.strip_prefix(b).is_some_and(|s| s.starts_with('.'))
        || b.strip_prefix(a).is_some_and(|s| s.starts_with('.'))
}

fn in_condition(mut n: Node<'_>) -> bool {
    let at = Span::of(n);
    while let Some(parent) = n.parent() {
        if callable(parent) {
            return false;
        }
        if parent
            .child_by_field_name("condition")
            .is_some_and(|condition| Span::of(condition).contains(at))
        {
            return true;
        }
        n = parent;
    }
    false
}

fn conditional(mut n: Node<'_>) -> bool {
    while let Some(parent) = n.parent() {
        if callable(parent) {
            return false;
        }
        if matches!(
            parent.kind(),
            "if_statement"
                | "if_expression"
                | "else_clause"
                | "elif_clause"
                | "for_statement"
                | "for_in_statement"
                | "for_expression"
                | "while_statement"
                | "while_expression"
                | "try_statement"
                | "match_expression"
                | "switch_statement"
        ) {
            return true;
        }
        n = parent;
    }
    false
}

pub(crate) fn propagates(finding: &Finding) -> bool {
    matches!(finding.resolution.as_str(), RESOLUTION | "resolved-call")
        && finding.family == FAMILY
        && (finding.defect == RETURNS || finding.mechanism.as_deref() == Some(BOUNDARY_OUTPUT)
            || matches!(finding.defect.as_str(), "insecure-deserialization" | "index-out-of-bounds" | "type-errors") && finding.assessment == FindingAssessment::Candidate)
        && finding.state_trace.is_some()
}

pub(crate) fn returned_bindings(
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: impl Fn(&str, Span) -> Option<Span>,
) {
    let all = crate::annotations::nodes(root);
    for index in 0..report.findings.len() {
        let finding = &report.findings[index];
        if finding.origin != "propagated" || !propagates(finding) {
            continue;
        }
        let Some(call) = all
            .iter()
            .find(|n| {
                matches!(n.kind(), "call" | "call_expression") && Span::of(**n) == finding.trigger
            })
            .copied()
        else {
            continue;
        };
        let Some(name) = input_name(call) else {
            continue;
        };
        let Some(binding) = resolve(text(name, source), Span::of(name)) else {
            continue;
        };
        let end = all
            .iter()
            .filter(|n| n.start_byte() >= call.end_byte())
            .filter_map(|n| assigned_target(*n).map(|target| (*n, target)))
            .find(|(_, target)| {
                target.kind() == "identifier"
                    && resolve(text(*target, source), Span::of(*target)) == Some(binding)
            })
            .map_or(root.end_byte(), |(n, _)| n.start_byte());
        let uses: Vec<_> = all
            .iter()
            .filter(|n| {
                n.kind() == "identifier"
                    && n.start_byte() >= call.end_byte()
                    && n.end_byte() <= end
                    && owner(report, Span::of(**n)) == finding.function
                    && resolve(text(**n, source), Span::of(**n)) == Some(binding)
            })
            .map(|n| Span::of(*n))
            .collect();
        let finding = &mut report.findings[index];
        finding.blast_radius.extend(uses.iter().copied());
        if let Some(trace) = &mut finding.state_trace {
            if trace.source_hash == report.source_hash {
                trace
                    .affected_uses
                    .extend(uses.into_iter().map(|range| Location {
                        uri: report.uri.clone(),
                        range,
                    }));
            }
        }
    }
}

pub(crate) fn qualify(report: &mut DocumentReport, graph: &mut crate::state::StateGraph) {
    use crate::state_contract::{Condition, Predicate, SourceEvidence};
    for finding in &mut report.findings {
        if finding.resolution != RESOLUTION && !propagates(finding) {
            continue;
        }
        let Some(trace) = &mut finding.state_trace else {
            continue;
        };
        trace.snapshot_id = graph.snapshot_id.clone();
        for location in trace
            .origins
            .iter()
            .chain(&trace.invalidations)
            .chain(&trace.affected_uses)
            .chain(trace.requirement.iter().flat_map(|r| &r.declarations))
        {
            let Some(document) = graph.documents.iter().find(|d| d.uri == location.uri) else {
                continue;
            };
            let id = format!(
                "{}:{}:unstructured-evidence:{}:{}",
                graph.snapshot_id, location.uri, location.range.start, location.range.end
            );
            if !graph.evidence.iter().any(|e| e.id == id) {
                graph.evidence.push(SourceEvidence {
                    id: id.clone(),
                    document: document.id.clone(),
                    source_hash: document.source_hash.clone(),
                    range: location.range,
                });
            }
            if !trace.supporting_evidence.contains(&id) {
                trace.supporting_evidence.push(id);
            }
        }
        if let Some(requirement) = &mut trace.requirement {
            let declaration = &requirement.declarations[0];
            let id = format!(
                "{}:{}:producer-result:{}",
                graph.snapshot_id, declaration.uri, declaration.range.start
            );
            if !graph.conditions.iter().any(|c| c.id == id) {
                graph.conditions.push(Condition {
                    id: id.clone(),
                    predicate: Predicate::Unknown {
                        reason:
                            "Declared nonoptional producer output must be supplied on normal exits."
                                .into(),
                    },
                    uncertainty: vec![],
                    evidence: trace.supporting_evidence.clone(),
                });
            }
            requirement.condition = id.clone();
            trace.required_conditions = vec![id];
        }
    }
}

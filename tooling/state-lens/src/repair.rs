use crate::analysis::Engine;
use crate::heap::ir::Container;
use crate::heap::{Segment, Sharing};
use crate::model::{hash, DocumentReport, Finding, Span};
use crate::state::{EvidenceLevel, StateGraph};
use serde::Serialize;
use tree_sitter::{Node, Parser};

const REPAIR_URI: &str = "file:///state-lens-repair-snapshot";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Applicability {
    Auto,
    Advisory,
    Ambiguous,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CandidateStatus {
    Verified,
    Rejected,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub strategy: &'static str,
    pub edits: Vec<Edit>,
    pub status: CandidateStatus,
    pub reason: String,
    pub remaining_relationships: Vec<String>,
    pub remaining_shallow_sharing: usize,
    pub result_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairResult {
    pub source_hash: String,
    pub mechanism: Option<String>,
    pub postcondition: String,
    pub applicability: Applicability,
    pub reason: String,
    pub candidates: Vec<Candidate>,
}

fn postcondition(mechanism: &str) -> &'static str {
    match mechanism {
        "repeated-element" => "Each element written and observed has a distinct object identity.",
        "shallow-copy-nested" => {
            "The mutated nested object is independent of the object observed through the source."
        }
        "shared-default" => "Each invocation obtains independent default state.",
        "argument-returned-alias" | "internal-state-escape" => {
            "The caller receives an object independent of the owner's state."
        }
        "append-within-capacity" => {
            "Append allocates storage that does not overlap the original slice's backing array."
        }
        _ => "Unsupported mechanism: no postcondition is defined.",
    }
}

fn rejected(source: &str, mechanism: Option<String>, reason: &str) -> RepairResult {
    RepairResult {
        source_hash: hash(source),
        postcondition: postcondition(mechanism.as_deref().unwrap_or_default()).into(),
        mechanism,
        applicability: Applicability::Rejected,
        reason: reason.into(),
        candidates: vec![],
    }
}

fn analyze(source: &str, language: &str) -> Option<DocumentReport> {
    let mut engine = Engine::new().ok()?;
    engine
        .set(REPAIR_URI.into(), language, source.into(), Some(1))
        .ok()?;
    engine.analyze().documents.into_iter().next()
}

pub fn check(
    source: &str,
    language: &str,
    _path: &str,
    finding: usize,
    expected_hash: Option<&str>,
) -> RepairResult {
    if expected_hash.is_some_and(|h| h != hash(source)) {
        return rejected(
            source,
            None,
            "Source changed since the finding was produced; reanalyze before repairing.",
        );
    }
    let Some(document) = analyze(source, language) else {
        return rejected(source, None, "Language is not supported by this server.");
    };
    let Some(target) = document.findings.get(finding).cloned() else {
        return rejected(source, None, "Finding index is out of range.");
    };
    if target.resolution != crate::aliasing::RESOLUTION {
        return rejected(
            source,
            target.mechanism,
            "Only object-identity findings have repair postconditions.",
        );
    }
    let mechanism = target.mechanism.clone().unwrap_or_default();
    let graph = document.state_graph.as_ref().expect("graph");
    let trace = target.state_trace.as_ref().expect("trace");
    let strategies = strategies(source, language, &target, graph);
    if strategies.is_empty() {
        return rejected(
            source,
            Some(mechanism),
            "No supported transformation matches this sharing introduction.",
        );
    }
    let before: Vec<(String, String)> = relationships(source, &document);
    let mut candidates = vec![];
    for (strategy, edits) in strategies {
        candidates.push(verify(source, language, &target, &before, strategy, edits));
    }
    let verified: Vec<&Candidate> = candidates
        .iter()
        .filter(|c| c.status == CandidateStatus::Verified)
        .collect();
    let declared =
        trace.requirement.is_some() && trace.evidence_level == EvidenceLevel::FeasibleUnderModel;
    let (applicability, reason) = match verified.len() {
        0 => (Applicability::Rejected, "No candidate transformation established the postcondition.".to_string()),
        n if n > 1 && verified.windows(2).any(|w| w[0].remaining_relationships != w[1].remaining_relationships) => (
            Applicability::Ambiguous,
            "Verified transformations leave different remaining relationships; choose explicitly.".into(),
        ),
        _ if !declared => (
            Applicability::Advisory,
            "Postcondition verified, but no declared requirement shows the sharing is unintended.".into(),
        ),
        _ => (Applicability::Auto, "Postcondition verified against a declared requirement.".into()),
    };
    RepairResult {
        source_hash: hash(source),
        postcondition: postcondition(&mechanism).into(),
        mechanism: Some(mechanism),
        applicability,
        reason,
        candidates,
    }
}

pub fn first_finding(source: &str, language: &str) -> Option<usize> {
    analyze(source, language)?
        .findings
        .iter()
        .position(|f| f.resolution == crate::aliasing::RESOLUTION)
}

fn relationships(source: &str, document: &DocumentReport) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = document
        .findings
        .iter()
        .filter(|f| f.resolution == crate::aliasing::RESOLUTION)
        .map(|f| {
            (
                f.mechanism.clone().unwrap_or_default(),
                source[f.trigger.start..f.trigger.end].to_string(),
            )
        })
        .collect();
    out.sort();
    out
}

fn apply(source: &str, edits: &[Edit]) -> Option<String> {
    let mut sorted = edits.to_vec();
    sorted.sort_by_key(|e| e.start);
    if sorted.windows(2).any(|w| w[0].end > w[1].start) {
        return None;
    }
    let mut out = source.to_string();
    for edit in sorted.iter().rev() {
        out.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Some(out)
}

fn verify(
    source: &str,
    language: &str,
    target: &Finding,
    before: &[(String, String)],
    strategy: &'static str,
    edits: Vec<Edit>,
) -> Candidate {
    let reject = |reason: &str, edits: Vec<Edit>| Candidate {
        strategy,
        edits,
        status: CandidateStatus::Rejected,
        reason: reason.into(),
        remaining_relationships: vec![],
        remaining_shallow_sharing: 0,
        result_hash: String::new(),
    };
    let Some(candidate) = apply(source, &edits) else {
        return reject("Edits overlap.", edits);
    };
    let Some(after) = analyze(&candidate, language) else {
        return reject("Candidate could not be analyzed.", edits);
    };
    if after.parse_errors {
        return reject("Candidate does not parse.", edits);
    }
    let shift = |position: usize| -> usize {
        let mut delta: isize = 0;
        for e in &edits {
            if e.end <= position {
                delta += e.replacement.len() as isize - (e.end - e.start) as isize;
            }
        }
        (position as isize + delta) as usize
    };
    let moved = Span {
        start: shift(target.trigger.start),
        end: shift(target.trigger.end),
    };
    let remaining = relationships(&candidate, &after);
    let persists = after.findings.iter().any(|f| {
        f.resolution == crate::aliasing::RESOLUTION
            && f.trigger == moved
            && f.mechanism == target.mechanism
    });
    if persists {
        return reject(
            "The targeted write still reaches the observed state.",
            edits,
        );
    }
    let introduced: Vec<_> = remaining.iter().filter(|r| !before.contains(r)).collect();
    if !introduced.is_empty() {
        return reject(
            "Candidate introduces new object-identity relationships.",
            edits,
        );
    }
    let graph = after.state_graph.as_ref().expect("graph");
    Candidate {
        strategy,
        edits,
        status: CandidateStatus::Verified,
        reason: "Reparsed and reanalyzed: the targeted heap dependency is absent.".into(),
        remaining_relationships: remaining.iter().map(|(m, t)| format!("{m}: {t}")).collect(),
        remaining_shallow_sharing: graph
            .sharing
            .iter()
            .filter(|s| s.mechanism == Sharing::ShallowCopy.mechanism())
            .count(),
        result_hash: hash(&candidate),
    }
}

fn parse(source: &str, language: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&crate::analysis::grammar(language).ok()?)
        .ok()?;
    parser.parse(source, None)
}

fn node_at<'t>(root: Node<'t>, span: Span) -> Option<Node<'t>> {
    root.descendant_for_byte_range(span.start, span.end)
        .filter(|n| n.start_byte() == span.start && n.end_byte() == span.end)
}

fn ancestor<'t>(mut n: Node<'t>, kind: &str) -> Option<Node<'t>> {
    loop {
        if n.kind() == kind {
            return Some(n);
        }
        n = n.parent()?;
    }
}

fn text<'s>(source: &'s str, n: Node<'_>) -> &'s str {
    &source[n.byte_range()]
}

struct Context<'a> {
    source: &'a str,
    language: &'a str,
    graph: &'a StateGraph,
    target: &'a Finding,
}

impl Context<'_> {
    fn sharing(&self) -> Vec<(Span, Option<Container>)> {
        let heap = &self.graph.heap;
        let trace = self.target.state_trace.as_ref().unwrap();
        heap.sharing
            .iter()
            .filter(|s| Some(s.kind.mechanism()) == self.target.mechanism.as_deref())
            .filter(|s| {
                trace
                    .origins
                    .iter()
                    .any(|o| o.range == self.graph.operations[s.op].location.range)
            })
            .fold(Vec::<(Span, Option<Container>)>::new(), |mut out, s| {
                let span = self.graph.operations[s.op].location.range;
                let container = heap.containers.get(&s.allocation).copied();
                match out.iter_mut().find(|(existing, _)| *existing == span) {
                    Some(entry) => entry.1 = entry.1.or(container),
                    None => out.push((span, container)),
                }
                out
            })
    }

    fn written_path(&self) -> Option<crate::heap::RawPath> {
        let trace = self.target.state_trace.as_ref()?;
        let identity = trace.invalidating_operations.first()?;
        let op = self
            .graph
            .operations
            .iter()
            .find(|o| &o.identity == identity)?;
        self.graph.heap.paths.get(&op.id).cloned()
    }

    fn written_container(&self) -> Option<Container> {
        self.graph
            .heap
            .writes
            .iter()
            .find(|w| self.graph.operations[w.op].location.range == self.target.trigger)
            .and_then(|w| self.graph.heap.containers.get(&w.object).copied())
    }

    fn copy_expression(&self, expr: &str, container: Option<Container>) -> String {
        match (self.language, container) {
            ("javascript" | "typescript", Some(Container::Array)) => format!("[...{expr}]"),
            ("javascript" | "typescript", _) => format!("{{ ...{expr} }}"),
            ("python", Some(Container::Array)) => format!("list({expr})"),
            ("python", Some(Container::Set)) => format!("set({expr})"),
            ("python", _) => format!("dict({expr})"),
            ("go", Some(Container::Map)) => format!("maps.Clone({expr})"),
            ("go", _) => format!("slices.Clone({expr})"),
            _ => expr.into(),
        }
    }

    fn go_import(&self, root: Node<'_>, package: &str) -> Option<Edit> {
        if self.source.contains(&format!("\"{package}\"")) {
            return None;
        }
        let mut cursor = root.walk();
        let import = root
            .named_children(&mut cursor)
            .find(|n| n.kind() == "import_declaration")?;
        let spec = import.named_child(0)?;
        if spec.kind() == "import_spec_list" {
            let at = spec.start_byte() + 1;
            return Some(Edit {
                start: at,
                end: at,
                replacement: format!("\n\t\"{package}\""),
            });
        }
        Some(Edit {
            start: spec.start_byte(),
            end: spec.end_byte(),
            replacement: format!("(\n\t{}\n\t\"{package}\"\n)", text(self.source, spec)),
        })
    }

    fn python_import(&self, module: &str) -> Option<Edit> {
        let line = format!("import {module}");
        if self.source.lines().any(|l| l.trim() == line) {
            return None;
        }
        Some(Edit {
            start: 0,
            end: 0,
            replacement: format!("{line}\n"),
        })
    }
}

fn strategies(
    source: &str,
    language: &str,
    target: &Finding,
    graph: &StateGraph,
) -> Vec<(&'static str, Vec<Edit>)> {
    let Some(tree) = parse(source, language) else {
        return vec![];
    };
    let root = tree.root_node();
    let context = Context {
        source,
        language,
        graph,
        target,
    };
    let mut out = vec![];
    let mechanism = target.mechanism.as_deref().unwrap_or_default();
    for (span, container) in context.sharing() {
        let Some(node) = node_at(root, span) else {
            continue;
        };
        match (mechanism, language) {
            ("repeated-element", "javascript" | "typescript") => {
                let Some(call) = ancestor(node, "call_expression") else {
                    continue;
                };
                let receiver = call
                    .child_by_field_name("function")
                    .and_then(|f| f.child_by_field_name("object"));
                let argument = call
                    .child_by_field_name("arguments")
                    .and_then(|a| a.named_child(0));
                let length = receiver
                    .filter(|r| r.kind() == "new_expression")
                    .and_then(|r| r.child_by_field_name("arguments"))
                    .and_then(|a| {
                        (a.named_child_count() == 1)
                            .then(|| a.named_child(0))
                            .flatten()
                    });
                if let (Some(length), Some(argument)) = (length, argument) {
                    if argument.kind() == "identifier" || argument.kind() == "member_expression" {
                        out.push((
                            "copy-per-element",
                            vec![Edit {
                                start: call.start_byte(),
                                end: call.end_byte(),
                                replacement: format!(
                                    "Array.from({{ length: {} }}, () => ({}))",
                                    text(source, length),
                                    context.copy_expression(text(source, argument), container)
                                ),
                            }],
                        ));
                    }
                    if argument.kind() == "object" || argument.kind() == "array" {
                        out.push((
                            "allocate-per-element",
                            vec![Edit {
                                start: call.start_byte(),
                                end: call.end_byte(),
                                replacement: format!(
                                    "Array.from({{ length: {} }}, () => ({}))",
                                    text(source, length),
                                    text(source, argument)
                                ),
                            }],
                        ));
                    }
                }
            }
            ("repeated-element", "python") => {
                let Some(operator) = ancestor(node, "binary_operator") else {
                    continue;
                };
                let (Some(left), Some(right)) = (
                    operator.child_by_field_name("left"),
                    operator.child_by_field_name("right"),
                ) else {
                    continue;
                };
                if left.kind() == "list" && left.named_child_count() == 1 {
                    let element = left.named_child(0).unwrap();
                    out.push((
                        "allocate-per-element",
                        vec![Edit {
                            start: operator.start_byte(),
                            end: operator.end_byte(),
                            replacement: format!(
                                "[{} for _ in range({})]",
                                text(source, element),
                                text(source, right)
                            ),
                        }],
                    ));
                }
            }
            ("shallow-copy-nested", _) => {
                let Some(path) = context.written_path() else {
                    continue;
                };
                let Some(Segment::Field(field)) = path.segments.first().cloned() else {
                    continue;
                };
                match language {
                    "javascript" | "typescript" => {
                        let Some(spread) = ancestor(node, "spread_element") else {
                            continue;
                        };
                        let Some(object) = spread.parent() else {
                            continue;
                        };
                        let source_expr = text(source, node);
                        let last = object
                            .named_child(object.named_child_count().saturating_sub(1))
                            .unwrap_or(spread);
                        out.push((
                            "copy-mutated-segment",
                            vec![Edit {
                                start: last.end_byte(),
                                end: last.end_byte(),
                                replacement: format!(
                                    ", {field}: {}",
                                    context.copy_expression(
                                        &format!("{source_expr}.{field}"),
                                        context.written_container()
                                    )
                                ),
                            }],
                        ));
                    }
                    "python" => {
                        let Some(call) = ancestor(node, "call") else {
                            continue;
                        };
                        let receiver = call
                            .child_by_field_name("function")
                            .and_then(|f| f.child_by_field_name("object"))
                            .map(|o| text(source, o).to_string())
                            .or_else(|| {
                                call.child_by_field_name("arguments")
                                    .and_then(|a| a.named_child(0))
                                    .map(|a| text(source, a).to_string())
                            });
                        let Some(receiver) = receiver else { continue };
                        let mut edits = vec![Edit {
                            start: call.start_byte(),
                            end: call.end_byte(),
                            replacement: format!("copy.deepcopy({receiver})"),
                        }];
                        edits.extend(context.python_import("copy"));
                        out.push(("deep-copy", edits));
                    }
                    "go" => {
                        let Some(statement) = ancestor(node, "short_var_declaration")
                            .or_else(|| ancestor(node, "assignment_statement"))
                        else {
                            continue;
                        };
                        let Some(crate::heap::Root::Binding(binding)) = Some(path.root.clone())
                        else {
                            continue;
                        };
                        let copy = &source[binding.start..binding.end];
                        let package = if context.written_container() == Some(Container::Map) {
                            "maps"
                        } else {
                            "slices"
                        };
                        let indent: String = source[..statement.start_byte()]
                            .rsplit('\n')
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .collect();
                        let mut edits = vec![Edit {
                            start: statement.end_byte(),
                            end: statement.end_byte(),
                            replacement: format!(
                                "\n{indent}{copy}.{field} = {package}.Clone({copy}.{field})"
                            ),
                        }];
                        edits.extend(context.go_import(root, package));
                        out.push(("copy-mutated-segment", edits));
                    }
                    _ => {}
                }
            }
            ("shared-default", "javascript" | "typescript") => {
                out.push((
                    "copy-default-per-call",
                    vec![Edit {
                        start: span.start,
                        end: span.end,
                        replacement: context.copy_expression(text(source, node), container),
                    }],
                ));
            }
            ("shared-default", "python") => {
                let Some(parameter) = node.parent().filter(|p| {
                    matches!(p.kind(), "default_parameter" | "typed_default_parameter")
                }) else {
                    continue;
                };
                let Some(name) = parameter.child_by_field_name("name") else {
                    continue;
                };
                let Some(function) = ancestor(parameter, "function_definition") else {
                    continue;
                };
                let Some(body) = function.child_by_field_name("body") else {
                    continue;
                };
                let Some(first) = body.named_child(0) else {
                    continue;
                };
                let indent: String = source[..first.start_byte()]
                    .rsplit('\n')
                    .next()
                    .unwrap_or("")
                    .to_string();
                let name = text(source, name);
                out.push((
                    "none-sentinel",
                    vec![
                        Edit {
                            start: span.start,
                            end: span.end,
                            replacement: "None".into(),
                        },
                        Edit {
                            start: first.start_byte(),
                            end: first.start_byte(),
                            replacement: format!(
                                "if {name} is None:\n{indent}    {name} = {}\n{indent}",
                                text(source, node)
                            ),
                        },
                    ],
                ));
            }
            ("argument-returned-alias" | "internal-state-escape", _) => {
                let Some(statement) = ancestor(node, "return_statement") else {
                    continue;
                };
                let Some(value) = statement.named_child(0) else {
                    continue;
                };
                let value = if value.kind() == "expression_list" {
                    value.named_child(0).unwrap_or(value)
                } else {
                    value
                };
                let mut edits = vec![Edit {
                    start: value.start_byte(),
                    end: value.end_byte(),
                    replacement: context.copy_expression(text(source, value), container),
                }];
                if language == "go" {
                    let package = if container == Some(Container::Map) {
                        "maps"
                    } else {
                        "slices"
                    };
                    edits.extend(context.go_import(root, package));
                }
                out.push(("copy-on-return", edits));
            }
            ("append-within-capacity", "go") => {
                let Some(slice) = ancestor(node, "slice_expression") else {
                    continue;
                };
                if slice.child_by_field_name("capacity").is_some() {
                    continue;
                }
                let Some(end) = slice.child_by_field_name("end") else {
                    continue;
                };
                out.push((
                    "cap-subslice",
                    vec![Edit {
                        start: end.end_byte(),
                        end: end.end_byte(),
                        replacement: format!(":{}", text(source, end)),
                    }],
                ));
            }
            _ => {}
        }
    }
    out.sort_by_key(|(s, e)| (*s, e.first().map(|e| e.start)));
    out.dedup_by(|a, b| a.0 == b.0 && a.1.first().map(|e| e.start) == b.1.first().map(|e| e.start));
    out
}

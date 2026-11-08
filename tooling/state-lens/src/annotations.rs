use crate::model::*;
use tree_sitter::Node;

pub(crate) fn nodes(node: Node<'_>) -> Vec<Node<'_>> {
    let mut result = vec![node];
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        result.extend(nodes(child));
    }
    result
}
fn owner(report: &DocumentReport, span: Span) -> Option<Span> {
    report
        .functions
        .iter()
        .filter(|f| f.range.contains(span))
        .min_by_key(|f| f.range.len())
        .map(|f| f.range)
}
fn add(
    report: &mut DocumentReport,
    range: Span,
    family: &str,
    defect: &str,
    explanation: &str,
    related: &[Span],
) {
    let mechanism = defect;
    let defect = match mechanism {
        "async-function" | "async-execution" => "async-io-misuse",
        "external-input" | "external-input-binding" => "lack-of-input-validation",
        "unsafe-input-execution" => "insecure-deserialization",
        "runtime-type-dispatch" => "duck-typing-without-contracts",
        "resource-lifecycle" | "resource-binding" => "resource-leaks",
        _ => defect,
    };
    let rule_id = format!("{}.{family}.{defect}", report.language);
    report.findings.push(Finding {
        state_trace: None,
        division_risk: None,
        rule_id: rule_id.clone(),
        family: family.into(),
        compound: None,
        blast_radius: vec![],
        defect: defect.into(),
        trigger: range,
        function: owner(report, range),
        effect: Effect::StructuralSignal,
        origin: "direct".into(),
        resolution: mechanism.into(),
        explanation: explanation.into(),
        evidence: std::iter::once(range)
            .chain(related.iter().copied())
            .map(|range| Evidence {
                uri: report.uri.clone(),
                range,
                rule_id: rule_id.clone(),
            })
            .collect(),
        call_path: vec![],
        related_cwes: vec![],
    });
}

fn assigned_name(node: Node<'_>) -> Option<Node<'_>> {
    let mut current = node;
    loop {
        if matches!(
            current.kind(),
            "variable_declarator" | "let_declaration" | "assignment" | "short_var_declaration"
        ) {
            let target = current
                .child_by_field_name("name")
                .or_else(|| current.child_by_field_name("pattern"))
                .or_else(|| current.child_by_field_name("left"))?;
            return nodes(target).into_iter().find(|n| n.kind() == "identifier");
        }
        if current != node
            && !matches!(
                current.kind(),
                "await_expression"
                    | "try_expression"
                    | "parenthesized_expression"
                    | "expression_list"
            )
        {
            return None;
        }
        current = current.parent()?;
    }
}

fn binding_scope(node: Node<'_>, language: &str) -> Span {
    let mut current = node;
    while let Some(parent) = current.parent() {
        current = parent;
        if matches!(
            current.kind(),
            "function_definition" | "module" | "program" | "source_file"
        ) || language != "python" && matches!(current.kind(), "block" | "statement_block")
        {
            break;
        }
    }
    Span::of(current)
}

fn rebound(all: &[Node<'_>], source: &str, name: &str, after: usize, before: usize) -> bool {
    all.iter().any(|n| {
        n.start_byte() > after
            && n.end_byte() < before
            && matches!(
                n.kind(),
                "assignment"
                    | "augmented_assignment"
                    | "assignment_expression"
                    | "augmented_assignment_expression"
                    | "assignment_statement"
                    | "short_var_declaration"
                    | "let_declaration"
                    | "variable_declarator"
            )
            && n.child_by_field_name("left")
                .or_else(|| n.child_by_field_name("pattern"))
                .or_else(|| n.child_by_field_name("name"))
                .is_some_and(|target| {
                    nodes(target)
                        .iter()
                        .any(|n| n.kind() == "identifier" && &source[n.byte_range()] == name)
                })
    })
}

pub fn analyze(
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    imports: &[(String, String, Option<String>)],
    shadowed: impl Fn(&str, Span) -> bool,
) {
    let shared: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            matches!(f.effect, Effect::GlobalMutation | Effect::ExternalRead)
                && f.resolution == "binding-analysis"
        })
        .cloned()
        .map(|mut f| {
            f.family = "shared-state".into();
            f.defect = "cross-boundary-state-exposure".into();
            f.rule_id = format!(
                "{}.shared-state.cross-boundary-state-exposure",
                report.language
            );
            f
        })
        .collect();
    report.findings.extend(shared);
    let all = nodes(root);
    let text = |n: Node<'_>| &source[n.byte_range()];
    let language = report.language.clone();
    let shadowed = |name: &str, range: Span| {
        shadowed(name, range)
            || all.iter().any(|node| {
                if !binding_scope(*node, &language).contains(range) {
                    return false;
                }
                match (language.as_str(), node.kind()) {
                    ("python", "import_from_statement") => {
                        let mut cursor = node.walk();
                        let found =
                            node.children_by_field_name("name", &mut cursor)
                                .any(|import| {
                                    import
                                        .child_by_field_name("alias")
                                        .map(text)
                                        .unwrap_or_else(|| text(import))
                                        == name
                                });
                        found
                    }
                    ("python", "import_statement") => {
                        nodes(*node).iter().any(|n| {
                            n.kind() == "aliased_import"
                                && n.child_by_field_name("alias")
                                    .is_some_and(|alias| text(alias) == name)
                        }) || matches!(name, "input" | "open" | "eval" | "exec" | "isinstance")
                            && text(*node) == format!("import {name}")
                    }
                    ("rust", "mod_item" | "use_declaration") => {
                        node.child_by_field_name("name")
                            .is_some_and(|n| text(n) == name)
                            || imports.iter().any(|(local, _, _)| local == name)
                    }
                    (
                        _,
                        "function_declaration"
                        | "function_definition"
                        | "function_item"
                        | "class_declaration"
                        | "class_definition",
                    ) => node
                        .child_by_field_name("name")
                        .is_some_and(|n| text(n) == name),
                    _ => false,
                }
            })
    };
    let mut resources = vec![];
    let mut inputs: Vec<(Node<'_>, Node<'_>)> = vec![];
    for node in &all {
        let range = Span::of(*node);
        if matches!(
            node.kind(),
            "function_definition"
                | "function_declaration"
                | "function_item"
                | "arrow_function"
                | "method_definition"
                | "function_expression"
        ) {
            let mut cursor = node.walk();
            let keyword = node.children(&mut cursor).find_map(|child| {
                if child.kind() == "async" {
                    Some(child)
                } else if child.kind() == "function_modifiers" {
                    let mut cursor = child.walk();
                    let found = child.children(&mut cursor).find(|n| n.kind() == "async");
                    found
                } else {
                    None
                }
            });
            if let Some(keyword) = keyword {
                add(report, Span::of(keyword), "shared-state", "async-function", "Explicit asynchronous function. Suspension and scheduling are susceptibility mechanisms, not proof of a race.", &[]);
            }
        }
        if node.kind() == "go_statement" {
            add(report, range, "shared-state", "async-execution", "Explicit goroutine launch; asynchronous execution applies to this launch, not every function it can call.", &[]);
            if let Some(keyword) = node
                .named_child(0)
                .and_then(|call| call.child_by_field_name("function"))
                .filter(|callee| callee.kind() == "func_literal")
                .and_then(|callee| callee.child(0))
            {
                add(
                    report,
                    Span::of(keyword),
                    "shared-state",
                    "async-function",
                    "Function literal executed directly as a goroutine.",
                    &[],
                );
            }
        }
        if !matches!(node.kind(), "call_expression" | "call") {
            continue;
        }
        let Some(callee) = node.child_by_field_name("function") else {
            continue;
        };
        let name = text(callee);
        let base = name.split(['.', ':']).next().unwrap_or(name);
        let response_body = matches!(report.language.as_str(), "javascript" | "typescript")
            && matches!(name.rsplit('.').next(), Some("text" | "json"))
            && inputs.iter().any(|(input, target)| {
                text(*target) == base
                    && input.end_byte() < node.start_byte()
                    && owner(report, Span::of(*input)) == owner(report, range)
                    && binding_scope(*target, &report.language).contains(range)
                    && input
                        .child_by_field_name("function")
                        .is_some_and(|f| text(f) == "fetch")
                    && !rebound(&all, source, base, input.end_byte(), node.start_byte())
            });
        if shadowed(base, range) && !response_body {
            continue;
        }
        let resolved =
            imports
                .iter()
                .find(|(local, _, _)| local == base)
                .map(|(local, module, symbol)| {
                    format!(
                        "{}{}{}",
                        module,
                        symbol.as_ref().map_or(String::new(), |s| format!(".{s}")),
                        &name[local.len()..]
                    )
                });
        let python_import = report.language == "python"
            && all.iter().any(|n| {
                n.kind() == "import_statement"
                    && text(*n) == format!("import {base}")
                    && binding_scope(*n, &report.language).contains(range)
                    && n.start_byte() < node.start_byte()
            });
        let external = response_body
            || match report.language.as_str() {
                "javascript" | "typescript" => {
                    name == "fetch" && resolved.is_none()
                        || matches!(
                            resolved.as_deref(),
                            Some("node:fs.readFileSync" | "fs.readFileSync")
                        )
                }
                "go" => matches!(
                    resolved.as_deref(),
                    Some("net/http.Get" | "net/http.Post" | "io.ReadAll")
                ),
                "python" => {
                    name == "input"
                        || python_import
                            && matches!(name, "requests.get" | "requests.post" | "json.loads")
                }
                "rust" => matches!(
                    name,
                    "reqwest::get" | "std::env::args" | "std::fs::read_to_string"
                ),
                _ => false,
            };
        let acquisition = match report.language.as_str() {
            "javascript" | "typescript" => matches!(
                resolved.as_deref(),
                Some("node:fs.openSync" | "fs.openSync")
            ),
            "go" => matches!(resolved.as_deref(), Some("os.Open" | "os.Create")),
            "python" => name == "open",
            "rust" => name == "std::fs::File::open",
            _ => false,
        };
        if acquisition {
            if let Some(target) = assigned_name(*node) {
                resources.push((*node, target));
            }
            if report.language == "python" {
                let mut parent = node.parent();
                while let Some(n) = parent {
                    if n.kind() == "with_statement" {
                        add(report, Span::of(n), "shared-state", "resource-lifecycle", "Context-managed acquisition and release. This labels a resource lifetime, not a proven leak.", &[]);
                        break;
                    }
                    if matches!(n.kind(), "block" | "function_definition") {
                        break;
                    }
                    parent = n.parent();
                }
            }
        }
        if external {
            if let Some(target) = assigned_name(*node) {
                inputs.push((*node, target));
            }
            add(report, range, "unstructured-state", "external-input", "External or deserialized input at its source boundary. Its application-specific shape has not been established here; this is not a claim of insecure handling.", &[]);
        }
    }
    for (acquisition, target) in resources {
        let name = text(target);
        let function = owner(report, Span::of(acquisition));
        for release in all
            .iter()
            .filter(|n| {
                n.start_byte() > acquisition.end_byte()
                    && matches!(n.kind(), "call" | "call_expression")
                    && owner(report, Span::of(**n)) == function
                    && binding_scope(target, &report.language).contains(Span::of(**n))
            })
            .copied()
            .collect::<Vec<_>>()
        {
            let Some(callee) = release.child_by_field_name("function") else {
                continue;
            };
            let args = release
                .child_by_field_name("arguments")
                .map(text)
                .unwrap_or("");
            let release_name = text(callee);
            let matched = match report.language.as_str() {
                "python" => release_name == format!("{name}.close"),
                "go" => release_name == format!("{name}.Close"),
                "rust" => {
                    release_name == "drop"
                        && !shadowed("drop", Span::of(release))
                        && args == format!("({name})")
                }
                "javascript" | "typescript" => {
                    imports.iter().any(|(local, module, _)| {
                        matches!(module.as_str(), "fs" | "node:fs")
                            && release_name == format!("{local}.closeSync")
                            && !shadowed(local, Span::of(release))
                    }) && args == format!("({name})")
                }
                _ => false,
            };
            if matched
                && !rebound(
                    &all,
                    source,
                    name,
                    acquisition.end_byte(),
                    release.start_byte(),
                )
            {
                add(report, Span::of(target), "shared-state", "resource-binding", "This binding owns a resource with an explicitly matched acquisition and release.", &[Span::of(acquisition), Span::of(release)]);
                add(report, Span { start: acquisition.start_byte(), end: release.end_byte() }, "shared-state", "resource-lifecycle", "Matched acquisition and release of the same lexical resource. Control-flow completeness and exception safety are not proven.", &[Span::of(acquisition), Span::of(release)]);
                break;
            }
        }
    }
    for &(input, target) in &inputs {
        add(report, Span::of(target), "unstructured-state", "external-input-binding", "Binding receives external input; no application-specific validation is established at this assignment.", &[Span::of(input)]);
        let name = text(target);
        let function = owner(report, Span::of(input));
        let input_callee = input
            .child_by_field_name("function")
            .map(text)
            .unwrap_or("");
        let executable_input = match report.language.as_str() {
            "python" => matches!(input_callee, "input" | "json.loads"),
            "javascript" | "typescript" => {
                input_callee.ends_with(".text")
                    || input_callee.ends_with(".readFileSync")
                        && input.child_by_field_name("arguments").is_some_and(|args| {
                            args.named_child(1).is_some_and(|encoding| {
                                encoding.kind() == "string"
                                    && text(encoding).trim_matches(['\'', '"']) == "utf8"
                            })
                        })
            }
            _ => false,
        };
        for call in all
            .iter()
            .filter(|n| {
                n.start_byte() > input.end_byte()
                    && matches!(n.kind(), "call" | "call_expression")
                    && owner(report, Span::of(**n)) == function
                    && binding_scope(target, &report.language).contains(Span::of(**n))
            })
            .copied()
            .collect::<Vec<_>>()
        {
            let Some(callee) = call.child_by_field_name("function") else {
                continue;
            };
            let callable = text(callee);
            let sink = matches!(
                report.language.as_str(),
                "javascript" | "typescript" | "python"
            ) && (callable == "eval"
                || report.language == "python" && callable == "exec")
                && !shadowed(callable, Span::of(call));
            let direct_argument = call
                .child_by_field_name("arguments")
                .and_then(|args| args.named_child(0))
                .is_some_and(|arg| arg.kind() == "identifier" && text(arg) == name);
            let reassigned = rebound(&all, source, name, input.end_byte(), call.start_byte());
            if executable_input && sink && direct_argument && !reassigned {
                add(report, Span::of(call), "unstructured-state", "unsafe-input-execution", "External input flows directly into dynamic code execution without an intervening transformation. Shape validation alone would not make this operation safe.", &[Span::of(input)]);
            }
        }
    }
    let polymorphic: Vec<_> = all
        .iter()
        .filter_map(|node| {
            if node.kind() != "if_statement" || node.child_by_field_name("alternative").is_none() {
                return None;
            }
            let condition = node.child_by_field_name("condition")?;
            let dispatches_on_input = nodes(condition).iter().find_map(|n| {
                let tested = match report.language.as_str() {
                    "typescript" | "javascript"
                        if n.kind() == "unary_expression"
                            && n.child_by_field_name("operator")
                                .is_some_and(|op| op.kind() == "typeof") =>
                    {
                        n.child_by_field_name("argument")
                    }
                    "python"
                        if n.kind() == "call"
                            && n.child_by_field_name("function").is_some_and(|callee| {
                                text(callee) == "isinstance"
                                    && !shadowed("isinstance", Span::of(*n))
                            }) =>
                    {
                        n.child_by_field_name("arguments")
                            .and_then(|args| args.named_child(0))
                    }
                    _ => None,
                };
                tested
                    .filter(|tested| tested.kind() == "identifier")
                    .and_then(|tested| {
                        inputs
                            .iter()
                            .find(|(input, target)| {
                                text(*target) == text(tested)
                                    && input.end_byte() < tested.start_byte()
                                    && owner(report, Span::of(*input))
                                        == owner(report, Span::of(tested))
                                    && binding_scope(*target, &report.language)
                                        .contains(Span::of(tested))
                                    && !rebound(
                                        &all,
                                        source,
                                        text(*target),
                                        input.end_byte(),
                                        tested.start_byte(),
                                    )
                            })
                            .map(|(input, _)| Span::of(*input))
                    })
            });
            dispatches_on_input.map(|input| (Span::of(condition), input))
        })
        .collect();
    for (range, input) in polymorphic {
        add(report, range, "unstructured-state", "runtime-type-dispatch", "Function branches explicitly on the runtime type of an external-input binding. This identifies shape-dependent behavior, not a proven contract violation.", &[input]);
    }
}

pub(crate) fn build_annotations(all: &[Node<'_>], source: &str, report: &mut DocumentReport) {
    use AnnotationLabel as L;
    use AnnotationTarget as T;
    use StateCategory as C;
    let mut annotations: Vec<StateAnnotation> = vec![];
    for (index, finding) in report.findings.iter().enumerate() {
        if let Some(compound) = &finding.compound {
            let label = match compound.class.as_str() {
                "nondeterminism" => L::Unstable,
                "data-corruption" => L::Corruptible,
                _ => L::Fragile,
            };
            for &range in &finding.blast_radius {
                annotations.push(StateAnnotation {
                    depth: all.iter().filter(|n| Span::of(**n).contains(range)).count(),
                    category: C::CompoundDefects,
                    target: T::Segment,
                    range,
                    position: range.end,
                    label,
                    function: finding.function,
                    findings: vec![index],
                });
            }
            continue;
        }
        let category = match finding.family.as_str() {
            "mutated-state" => C::MutatedState,
            "unstructured-state" => C::UnstructuredState,
            "shared-state" => C::SharedState,
            _ => continue,
        };
        let segment_label = match category {
            C::CompoundDefects => unreachable!(),
            C::MutatedState => L::Mutacious,
            C::UnstructuredState => L::Unstructured,
            C::SharedState => L::Shared,
        };
        let mut targets = vec![(
            T::Segment,
            finding.trigger,
            finding.trigger.start,
            segment_label,
        )];
        if let Some(function) = report
            .functions
            .iter()
            .find(|f| Some(f.range) == finding.function)
        {
            let label = match category {
                C::CompoundDefects => None,
                C::MutatedState if finding.effect != Effect::StructuralSignal => Some(L::Impure),
                C::MutatedState => None,
                C::UnstructuredState => {
                    (finding.resolution == "runtime-type-dispatch").then_some(L::Polymorphic)
                }
                C::SharedState => Some(if finding.resolution == "async-function" {
                    L::Async
                } else {
                    L::Shared
                }),
            };
            if let Some(label) = label {
                targets.push((
                    T::Function,
                    Span {
                        start: function.range.start,
                        end: function.body.start,
                    },
                    function.body.start,
                    label,
                ));
            }
        }
        if let Some(node) = all.iter().rev().find(|n| Span::of(**n) == finding.trigger) {
            let target = match node.kind() {
                "let_declaration" => node.child_by_field_name("pattern"),
                "variable_declarator" => node.child_by_field_name("name"),
                "lexical_declaration" | "variable_declaration" => nodes(*node)
                    .into_iter()
                    .find(|n| n.kind() == "variable_declarator")
                    .and_then(|n| n.child_by_field_name("name")),
                "assignment_expression"
                | "augmented_assignment"
                | "assignment"
                | "compound_assignment_expr"
                | "augmented_assignment_expression"
                | "assignment_statement"
                | "short_var_declaration" => node.child_by_field_name("left"),
                "call_expression" | "call"
                    if matches!(
                        finding.effect,
                        Effect::LocalMutation
                            | Effect::ReceiverMutation
                            | Effect::ArgumentMutation(_)
                    ) =>
                {
                    node.child_by_field_name("function").and_then(|callee| {
                        callee
                            .child_by_field_name("object")
                            .or_else(|| callee.child_by_field_name("value"))
                            .or_else(|| callee.child_by_field_name("operand"))
                    })
                }
                "update_expression" => node.child_by_field_name("argument"),
                "inc_statement" | "dec_statement" => node.named_child(0),
                "identifier"
                | "attribute"
                | "member_expression"
                | "field_expression"
                | "selector_expression" => Some(*node),
                _ => None,
            };
            if let Some(target) = target {
                let label = match category {
                    C::CompoundDefects => unreachable!(),
                    C::MutatedState => L::Volatile,
                    C::UnstructuredState => L::Untyped,
                    C::SharedState => L::Shared,
                };
                targets.push((T::Property, Span::of(target), target.end_byte(), label));
            }
        }
        for (target, range, position, label) in targets {
            if range.is_empty() || !source.is_char_boundary(position) {
                continue;
            }
            if let Some(existing) = annotations.iter_mut().find(|a| {
                a.category == category && a.target == target && a.range == range && a.label == label
            }) {
                existing.findings.push(index);
            } else {
                annotations.push(StateAnnotation {
                    depth: all.iter().filter(|n| Span::of(**n).contains(range)).count(),
                    category,
                    target,
                    range,
                    position,
                    label,
                    function: finding.function,
                    findings: vec![index],
                });
            }
        }
    }
    let specialized: Vec<_> = annotations
        .iter()
        .filter(|a| a.target == T::Function && matches!(a.label, L::Polymorphic | L::Async))
        .map(|a| (a.category, a.range))
        .collect();
    let generic: Vec<_> = annotations
        .iter()
        .filter(|a| a.target == T::Function && matches!(a.label, L::Unstructured | L::Shared))
        .cloned()
        .collect();
    for annotation in &mut annotations {
        if annotation.target == T::Function && matches!(annotation.label, L::Polymorphic | L::Async)
        {
            for generic in generic
                .iter()
                .filter(|a| a.category == annotation.category && a.range == annotation.range)
            {
                annotation.findings.extend(&generic.findings);
            }
            annotation.findings.sort_unstable();
            annotation.findings.dedup();
        }
    }
    annotations.retain(|a| {
        !(a.target == T::Function
            && matches!(a.label, L::Unstructured | L::Shared)
            && specialized.contains(&(a.category, a.range)))
    });
    annotations.sort_by_key(|a| (a.category, a.position, a.target, a.label));
    report.annotations = annotations;
}

pub(crate) fn coalesce(report: &DocumentReport, source: &str) -> Vec<StateAnnotation> {
    let mut groups = std::collections::BTreeMap::<_, Vec<StateAnnotation>>::new();
    for annotation in &report.annotations {
        let line = source[..annotation.position]
            .bytes()
            .filter(|b| *b == b'\n')
            .count();
        groups
            .entry((
                annotation.category,
                line,
                if annotation.category == StateCategory::CompoundDefects {
                    Some(annotation.label)
                } else {
                    None
                },
            ))
            .or_default()
            .push(annotation.clone());
    }
    groups
        .into_values()
        .map(|mut group| {
            group.sort_by_key(|a| {
                (
                    std::cmp::Reverse(a.depth),
                    a.range.len(),
                    std::cmp::Reverse(a.position),
                    a.label,
                )
            });
            let mut selected = group[0].clone();
            if group.len() > 1 {
                selected.target = AnnotationTarget::Segment;
                selected.label = match selected.category {
                    StateCategory::CompoundDefects => selected.label,
                    StateCategory::MutatedState => AnnotationLabel::Mutacious,
                    StateCategory::UnstructuredState => AnnotationLabel::Unstructured,
                    StateCategory::SharedState => AnnotationLabel::Shared,
                };
            }
            selected.findings = group
                .iter()
                .flat_map(|a| a.findings.iter().copied())
                .collect();
            selected.findings.sort_unstable();
            selected.findings.dedup();
            selected
        })
        .collect()
}

pub(crate) fn resolve_regions(root: Node<'_>, report: &mut DocumentReport) {
    let all = nodes(root);
    let divisions: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.division_risk
                .as_ref()
                .is_some_and(|r| r.mutated && r.confidence != "candidate")
        })
        .cloned()
        .collect();
    for division in divisions {
        let Some(expression) = all
            .iter()
            .filter(|n| {
                Span::of(**n).contains(division.trigger)
                    && matches!(n.kind(), "binary_expression" | "binary_operator")
            })
            .min_by_key(|n| n.byte_range().len())
        else {
            continue;
        };
        let radius = Span::of(*expression);
        if let Some(finding) = report
            .findings
            .iter_mut()
            .find(|f| f.rule_id == division.rule_id && f.trigger == division.trigger)
        {
            finding.blast_radius.push(radius);
        }
        let mutation = report.findings.iter_mut().find(|f| {
            f.family == "mutated-state"
                && f.function == division.function
                && f.origin == "direct"
                && division
                    .evidence
                    .iter()
                    .skip(1)
                    .any(|e| e.range == f.trigger)
                && matches!(
                    f.effect,
                    Effect::LocalMutation
                        | Effect::ArgumentMutation(_)
                        | Effect::GlobalMutation
                        | Effect::ReceiverMutation
                )
        });
        let Some(mutation) = mutation else {
            continue;
        };
        mutation.blast_radius.push(radius);
        let ingredient = Evidence {
            uri: report.uri.clone(),
            range: mutation.trigger,
            rule_id: mutation.rule_id.clone(),
        };
        let mut compound = division.clone();
        compound.division_risk = None;
        compound.rule_id = format!("{}.compound-defects.mutable-denominator", report.language);
        compound.family = "compound-defects".into();
        compound.defect = "mutable-denominator".into();
        compound.trigger = division.trigger;
        compound.blast_radius = vec![radius];
        compound.compound = Some(Compound {
            class: "runtime-errors".into(),
            category: "computational-exhaustion".into(),
            ingredients: [
                ingredient,
                Evidence {
                    uri: report.uri.clone(),
                    range: division.trigger,
                    rule_id: division.rule_id.clone(),
                },
            ],
        });
        compound.explanation = "Fragile: a detected mutation reaches an unguarded division denominator. The division may fail or produce a non-finite result; this is susceptibility, not proof of failure.".into();
        report.findings.push(compound);
    }
}

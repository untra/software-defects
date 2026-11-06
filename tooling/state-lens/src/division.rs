use crate::model::{DivisionRisk, DocumentReport, Effect, Evidence, Finding, Span};
use tree_sitter::Node;
const MAX_VALUE_DEPTH: usize = 16;

#[derive(Clone, Copy, PartialEq)]
enum Value {
    Zero,
    Nonzero,
    Input,
    Unknown,
}
fn children(n: Node<'_>) -> Vec<Node<'_>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}
fn text<'a>(n: Node<'_>, s: &'a str) -> &'a str {
    &s[n.start_byte()..n.end_byte()]
}
fn scope(mut n: Node<'_>) -> Node<'_> {
    while let Some(p) = n.parent() {
        if matches!(
            n.kind(),
            "function_definition"
                | "function_declaration"
                | "function_item"
                | "method_definition"
                | "arrow_function"
        ) {
            return n;
        }
        n = p;
    }
    n
}
fn descendants<'a>(n: Node<'a>, out: &mut Vec<Node<'a>>) {
    out.push(n);
    for child in children(n) {
        descendants(child, out);
    }
}
fn assignment(n: Node<'_>) -> Option<(Node<'_>, Node<'_>)> {
    if !matches!(
        n.kind(),
        "variable_declarator"
            | "let_declaration"
            | "assignment"
            | "assignment_expression"
            | "short_var_declaration"
            | "assignment_statement"
    ) {
        return None;
    }
    let left = n
        .child_by_field_name("name")
        .or_else(|| n.child_by_field_name("pattern"))
        .or_else(|| n.child_by_field_name("left"))?;
    let right = n
        .child_by_field_name("value")
        .or_else(|| n.child_by_field_name("right"))?;
    Some((left, right))
}
fn binding<'a>(name: &str, at: Node<'a>, s: &str) -> Option<Node<'a>> {
    let mut all = vec![];
    descendants(scope(at), &mut all);
    all.into_iter()
        .filter(|n| n.end_byte() <= at.start_byte() && scope(*n) == scope(at))
        .filter(|n| assignment(*n).is_some_and(|(l, _)| text(l, s) == name))
        .filter(|n| {
            let mut p = n.parent();
            while let Some(parent) = p {
                if matches!(parent.kind(), "block" | "statement_block")
                    && !(parent.start_byte() <= at.start_byte()
                        && at.end_byte() <= parent.end_byte())
                {
                    return false;
                }
                p = parent.parent();
            }
            true
        })
        .max_by_key(|n| n.start_byte())
}
fn value(n: Node<'_>, s: &str, depth: usize) -> Value {
    if depth > MAX_VALUE_DEPTH {
        return Value::Unknown;
    }
    let raw = text(n, s).trim();
    if let Ok(v) = raw.trim_end_matches('n').replace('_', "").parse::<f64>() {
        return if v == 0.0 {
            Value::Zero
        } else {
            Value::Nonzero
        };
    }
    if matches!(n.kind(), "parenthesized_expression" | "expression_list") && children(n).len() == 1
    {
        return value(children(n)[0], s, depth + 1);
    }
    if n.kind() == "identifier" {
        if let Some(b) = binding(raw, n, s) {
            if changed_between(b.end_byte(), n.start_byte(), scope(n), raw, s) {
                return Value::Input;
            }
            return value(assignment(b).unwrap().1, s, depth + 1);
        }
        let sc = scope(n);
        if sc.child_by_field_name("parameters").is_some_and(|p| {
            let mut ns = vec![];
            descendants(p, &mut ns);
            ns.iter()
                .any(|i| i.kind() == "identifier" && text(*i, s) == raw)
        }) {
            return Value::Input;
        }
    }
    if matches!(n.kind(), "call_expression" | "call") {
        let fun = n.child_by_field_name("function");
        let args = n
            .child_by_field_name("arguments")
            .map(children)
            .unwrap_or_default();
        if let Some(f) = fun {
            let name = text(f, s);
            if name == "std::hint::black_box" && args.len() == 1 {
                return value(args[0], s, depth + 1);
            }
            if name == "len" && args.len() == 1 && binding("len", n, s).is_none() {
                return length(args[0], s, depth + 1);
            }
        }
    }
    if n.kind() == "member_expression"
        && n.child_by_field_name("property")
            .is_some_and(|p| text(p, s) == "length")
    {
        if let Some(obj) = n.child_by_field_name("object") {
            return length(obj, s, depth + 1);
        }
    }
    Value::Unknown
}
fn length(n: Node<'_>, s: &str, depth: usize) -> Value {
    if depth > MAX_VALUE_DEPTH {
        return Value::Unknown;
    }
    if matches!(text(n, s), "[]" | "nil" | "\"\"") {
        return Value::Zero;
    }
    if let Some(b) = binding(text(n, s), n, s) {
        if changed_between(b.end_byte(), n.start_byte(), scope(n), text(n, s), s) {
            return Value::Input;
        }
        return length(assignment(b).unwrap().1, s, depth + 1);
    }
    Value::Input
}
fn condition(n: Node<'_>, den: &str, s: &str) -> Option<bool> {
    if n.kind() == "parenthesized_expression" {
        return children(n).first().and_then(|n| condition(*n, den, s));
    }
    let cs = children(n);
    let l = n
        .child_by_field_name("left")
        .or_else(|| cs.first().copied())?;
    let r = n
        .child_by_field_name("right")
        .or_else(|| cs.get(1).copied())?;
    let op = n
        .child_by_field_name("operator")
        .map(|n| text(n, s))
        .unwrap_or_else(|| s[l.end_byte()..r.start_byte()].trim());
    if (text(l, s) == den && value(r, s, 0) == Value::Zero)
        || (text(r, s) == den && value(l, s, 0) == Value::Zero)
    {
        return match op {
            "!=" | "!==" => Some(true),
            "==" | "===" => Some(false),
            _ => None,
        };
    }
    None
}
fn guarded(n: Node<'_>, den: &str, s: &str) -> bool {
    let mut child = n;
    while let Some(p) = child.parent() {
        if matches!(
            p.kind(),
            "conditional_expression" | "ternary_expression" | "if_statement" | "if_expression"
        ) {
            if let Some(c) = p.child_by_field_name("condition").or_else(|| {
                if p.kind() == "conditional_expression" {
                    children(p).get(1).copied()
                } else {
                    None
                }
            }) {
                if let Some(nonzero) = condition(c, den, s) {
                    let branch = p
                        .child_by_field_name(if nonzero {
                            "consequence"
                        } else {
                            "alternative"
                        })
                        .or_else(|| {
                            if p.kind() == "conditional_expression" {
                                children(p).get(if nonzero { 0 } else { 2 }).copied()
                            } else {
                                None
                            }
                        });
                    if branch.is_some_and(|b| {
                        b.start_byte() <= n.start_byte() && n.end_byte() <= b.end_byte()
                    }) {
                        return !changed_between(c.end_byte(), n.start_byte(), scope(n), den, s);
                    }
                }
            }
        }
        if matches!(p.kind(), "block" | "statement_block" | "statement_list") {
            for prev in children(p)
                .into_iter()
                .filter(|v| v.end_byte() <= child.start_byte())
            {
                let prev = if prev.kind() == "expression_statement" {
                    children(prev)[0]
                } else {
                    prev
                };
                if let Some(c) = prev.child_by_field_name("condition") {
                    if condition(c, den, s) == Some(false) {
                        if let Some(body) = prev.child_by_field_name("consequence") {
                            let direct = (if body.kind() == "return_statement" {
                                vec![body]
                            } else {
                                children(body)
                            })
                            .into_iter()
                            .flat_map(|v| {
                                if matches!(v.kind(), "statement_list" | "expression_statement") {
                                    children(v)
                                } else {
                                    vec![v]
                                }
                            })
                            .collect::<Vec<_>>();
                            if direct.iter().any(|v| {
                                matches!(v.kind(), "return_statement" | "return_expression")
                            }) && !changed_between(prev.end_byte(), n.start_byte(), p, den, s)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        if p == scope(n) {
            break;
        }
        child = p;
    }
    false
}
fn pure_reduce(n: Node<'_>, s: &str) -> bool {
    let Some(f) = n.child_by_field_name("function") else {
        return false;
    };
    if f.child_by_field_name("property")
        .is_none_or(|p| text(p, s) != "reduce")
    {
        return false;
    }
    let Some(args) = n.child_by_field_name("arguments") else {
        return false;
    };
    let cs = children(args);
    cs.first()
        .and_then(|cb| cb.child_by_field_name("body"))
        .is_some_and(|body| {
            body.kind() == "binary_expression"
                && children(body).iter().all(|v| v.kind() == "identifier")
        })
}
fn changed_between(start: usize, end: usize, root: Node<'_>, den: &str, s: &str) -> bool {
    let mut all = vec![];
    descendants(root, &mut all);
    all.into_iter().any(|n| {
        n.start_byte() >= start
            && n.end_byte() <= end
            && ((matches!(n.kind(), "call" | "call_expression") && !pure_reduce(n, s))
                || assignment(n).is_some_and(|(l, _)| text(l, s) == den)
                || matches!(
                    n.kind(),
                    "augmented_assignment"
                        | "augmented_assignment_expression"
                        | "update_expression"
                        | "compound_assignment_expr"
                ))
    })
}
pub fn analyze(operations: &[Node<'_>], s: &str, report: &mut DocumentReport) {
    for &n in operations {
        if !matches!(
            n.kind(),
            "binary_expression"
                | "binary_operator"
                | "assignment_statement"
                | "compound_assignment_expr"
                | "augmented_assignment_expression"
                | "augmented_assignment"
        ) {
            continue;
        }
        let Some(op) = n.child_by_field_name("operator") else {
            continue;
        };
        if !matches!(text(op, s), "/" | "%" | "//" | "/=" | "%=" | "//=") {
            continue;
        }
        let Some(den) = n.child_by_field_name("right") else {
            continue;
        };
        if report.language == "python"
            && text(op, s) == "%"
            && n.child_by_field_name("left")
                .is_some_and(|l| l.kind() == "string")
        {
            continue;
        }
        let v = value(den, s, 0);
        if v == Value::Nonzero || guarded(n, text(den, s), s) {
            continue;
        }
        let rule_id = format!("{}.unstructured-state.divide-by-zero", report.language);
        let mut evidence = vec![Evidence {
            uri: report.uri.clone(),
            range: Span::of(den),
            rule_id: rule_id.clone(),
        }];
        let mut mutated = false;
        if let Some(b) = binding(text(den, s), den, s) {
            evidence.push(Evidence {
                uri: report.uri.clone(),
                range: Span::of(b),
                rule_id: rule_id.clone(),
            });
            mutated = matches!(
                b.kind(),
                "assignment" | "assignment_expression" | "assignment_statement"
            ) && binding(text(den, s), b, s).is_some();
        }
        let mut prior = vec![];
        descendants(scope(n), &mut prior);
        for write in prior
            .into_iter()
            .filter(|w| w.end_byte() <= n.start_byte() && scope(*w) == scope(n))
        {
            if matches!(
                write.kind(),
                "augmented_assignment"
                    | "augmented_assignment_expression"
                    | "compound_assignment_expr"
                    | "update_expression"
                    | "inc_statement"
                    | "dec_statement"
            ) {
                let target = write
                    .child_by_field_name("left")
                    .or_else(|| write.child_by_field_name("argument"))
                    .or_else(|| children(write).first().copied());
                if target.is_some_and(|t| text(t, s) == text(den, s)) {
                    mutated = true;
                    evidence.push(Evidence {
                        uri: report.uri.clone(),
                        range: Span::of(write),
                        rule_id: rule_id.clone(),
                    });
                }
            }
        }
        let confidence = if v == Value::Zero {
            "known-zero"
        } else if v == Value::Input || mutated {
            "possible-zero"
        } else {
            "candidate"
        };
        let outcome = if report.language == "python" {
            "May raise ZeroDivisionError."
        } else if matches!(report.language.as_str(), "typescript" | "javascript") {
            "May produce Infinity/NaN for numbers or throw for BigInt."
        } else {
            "May panic for integers or produce a non-finite floating-point result."
        };
        report.findings.push(Finding {
            division_risk: Some(DivisionRisk {
                confidence: confidence.into(),
                outcome: outcome.into(),
                mutated,
            }),
            rule_id,
            family: "unstructured-state".into(),
            compound: None,
            defect: "divide-by-zero".into(),
            trigger: Span::of(den),
            function: report
                .functions
                .iter()
                .filter(|f| f.range.contains(Span::of(n)))
                .min_by_key(|f| f.range.len())
                .map(|f| f.range),
            effect: Effect::StructuralSignal,
            origin: "direct".into(),
            resolution: confidence.into(),
            explanation: format!(
                "{} denominator without an established nonzero guard. {outcome}",
                if v == Value::Zero {
                    "Zero"
                } else {
                    "Possibly zero"
                }
            ),
            evidence,
            call_path: vec![],
            related_cwes: vec!["CWE-369".into(), "CWE-682".into()],
        });
    }
}

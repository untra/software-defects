use crate::{annotations::nodes, defect_support::*, model::*, unstructured::add};
use tree_sitter::Node;

pub(crate) const DEFECT: &str = "type-errors";
const COERCION: &str = "implicit-coercion";
const REPETITION: &str = "sequence-repetition";
const FALSY: &str = "falsy-default";
const BOOL_INT: &str = "bool-is-int";
const MIXED_SORT: &str = "mixed-sort";
const CONVERSION: &str = "failed-conversion";
const ASSERTION: &str = "runtime-type-assertion";
const DOWNCAST: &str = "runtime-downcast";
const NUMERIC_COERCION: &str = "lossy-numeric-coercion";
const MEMBER: &str = "undeclared-member-access";
const FORMAT: &str = "format-type-mismatch";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Unknown,
}
pub(crate) fn kind(node: Node<'_>) -> Kind {
    match node.kind() {
        "string"
        | "string_literal"
        | "interpreted_string_literal"
        | "raw_string_literal"
        | "template_string" => Kind::String,
        "number" | "integer" | "float" | "integer_literal" | "float_literal" | "int_literal" => {
            Kind::Number
        }
        "true" | "false" | "boolean_literal" => Kind::Boolean,
        "array" | "list" | "array_expression" => Kind::Array,
        "object" | "dictionary" => Kind::Object,
        _ => Kind::Unknown,
    }
}
pub(crate) fn analyze(
    captures: &[(&str, &str, Node<'_>)],
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: impl Fn(&str, Span) -> Option<Span>,
) {
    let all = nodes(root);
    for &(path, capture, node) in captures {
        if !path.contains(DEFECT) || capture != "unstructured_type" || node.has_error() {
            continue;
        }
        let inferred = |n| {
            value(n, &all, source, &resolve)
                .map(kind)
                .unwrap_or(Kind::Unknown)
        };
        let mut mechanism = None;
        if let (Some(left), Some(right)) = (field(node, &["left"]), field(node, &["right"])) {
            let operator = field(node, &["operator"])
                .map(|n| text(n, source))
                .unwrap_or_else(|| source[left.end_byte()..right.start_byte()].trim());
            let kinds = (inferred(left), inferred(right));
            if matches!(report.language.as_str(), "javascript" | "typescript")
                && matches!(operator, "|" | ">>>")
                && text(right, source) == "0"
            {
                mechanism = Some(NUMERIC_COERCION);
            }
            if matches!(operator, "||" | "or") && kinds.0 != Kind::Boolean {
                mechanism = Some(FALSY);
            }
            if matches!(report.language.as_str(), "javascript" | "typescript")
                && (matches!(operator, "==" | "!=")
                    && (kinds.0 != kinds.1 || kinds.0 == Kind::Unknown)
                    || operator == "+"
                        && matches!(
                            kinds,
                            (Kind::String, Kind::Number) | (Kind::Number, Kind::String)
                        ))
            {
                mechanism = Some(COERCION);
            }
            if report.language == "python"
                && operator == "*"
                && matches!(
                    kinds,
                    (Kind::String, Kind::Number) | (Kind::Number, Kind::String)
                )
            {
                mechanism = Some(REPETITION);
            }
        }
        if matches!(node.kind(), "if_statement")
            && matches!(report.language.as_str(), "javascript" | "typescript")
        {
            if field(node, &["condition"]).is_some_and(|n| {
                let t = text(n, source).trim_matches(['(', ')', ' ']);
                t.starts_with('!') && !t.starts_with("!!") && !t.contains(['=', '.', '('])
            }) {
                mechanism = Some(FALSY);
            }
        }
        if calls(node) {
            let name = callee(node).map(|n| text(n, source)).unwrap_or("");
            let arguments = args(node);
            if method(node, source) == "sort"
                && arguments.is_empty()
                && receiver(node)
                    .and_then(|r| value(r, &all, source, &resolve))
                    .is_some_and(|v| {
                        kind(v) == Kind::Array
                            && v.named_children(&mut v.walk()).any(|n| {
                                kind(n) == Kind::Number
                                    || kind(n) == Kind::String
                                        && text(n, source)
                                            .trim_matches(['\'', '"'])
                                            .parse::<f64>()
                                            .is_ok()
                            })
                    })
            {
                mechanism = Some(COERCION);
            }
            if name == "parseInt"
                && (arguments.len() < 2
                    || arguments.first().is_some_and(|n| {
                        value(*n, &all, source, &resolve).is_some_and(|v| {
                            kind(v) == Kind::String
                                && text(v, source)
                                    .trim_matches(['\'', '"'])
                                    .parse::<i64>()
                                    .is_err()
                        })
                    }))
            {
                mechanism = Some(COERCION);
            }
            if name == "sorted"
                && arguments.first().is_some_and(|n| {
                    let kinds: Vec<_> = n.named_children(&mut n.walk()).map(kind).collect();
                    kinds.contains(&Kind::String) && kinds.contains(&Kind::Number)
                })
            {
                mechanism = Some(MIXED_SORT);
            }
            if name == "int"
                && ancestor(node, &["conditional_expression", "if_statement"]).is_none()
                && arguments.first().is_some_and(|n| {
                    value(*n, &all, source, &resolve).is_some_and(|v| {
                        kind(v) == Kind::String
                            && text(v, source)
                                .trim_matches(['\'', '"'])
                                .parse::<i64>()
                                .is_err()
                    })
                })
            {
                mechanism = Some(CONVERSION);
            }
            if report.language == "rust"
                && callee(node)
                    .map(|n| field(n, &["function"]).unwrap_or(n))
                    .and_then(|n| field(n, &["field"]))
                    .is_some_and(|n| text(n, source) == "downcast_ref")
            {
                let raw_function = callee(node);
                let requested = raw_function
                    .and_then(|n| field(n, &["type_arguments"]))
                    .map(|n| text(n, source).trim_matches(['<', '>']));
                let stored = raw_function
                    .and_then(|n| field(n, &["function"]))
                    .and_then(|n| field(n, &["value"]))
                    .and_then(|n| value(n, &all, source, &resolve));
                let concrete = stored
                    .and_then(|v| nodes(v).into_iter().find(|n| n.kind() == "integer_literal"))
                    .and_then(|n| text(n, source).rsplit_once('_').map(|(_, suffix)| suffix));
                if requested.is_none() || requested != concrete {
                    mechanism = Some(DOWNCAST);
                }
            }
            if report.language == "go"
                && name == "fmt.Printf"
                && arguments
                    .first()
                    .is_some_and(|n| text(*n, source).contains("%d"))
                && arguments
                    .iter()
                    .skip(1)
                    .any(|n| inferred(*n) == Kind::String)
            {
                mechanism = Some(FORMAT);
            }
        }
        if report.language == "python"
            && node.kind() == "subscript"
            && field(node, &["subscript"])
                .is_some_and(|n| matches!(text(n, source), "True" | "False"))
        {
            mechanism = Some(BOOL_INT);
        }
        if node.kind() == "type_assertion_expression"
            && report.language == "go"
            && !ancestor(node, &["short_var_declaration", "assignment_statement"])
                .and_then(left)
                .is_some_and(|n| n.named_child_count() > 1)
        {
            mechanism = Some(ASSERTION);
        }
        if matches!(report.language.as_str(), "javascript" | "python")
            && matches!(node.kind(), "member_expression" | "attribute")
        {
            if let (Some(object), Some(member)) = (
                field(node, &["object"]),
                field(node, &["property", "attribute"]),
            ) {
                if let Some(initial) =
                    value(object, &all, source, &resolve).filter(|n| kind(*n) == Kind::Object)
                {
                    let name = text(member, source);
                    let declared = initial.named_children(&mut initial.walk()).any(|entry| {
                        field(entry, &["key"])
                            .is_some_and(|key| text(key, source).trim_matches(['\'', '"']) == name)
                    });
                    let builtin = matches!(
                        name,
                        "toString"
                            | "hasOwnProperty"
                            | "constructor"
                            | "__proto__"
                            | "keys"
                            | "values"
                            | "items"
                            | "get"
                            | "pop"
                            | "clear"
                            | "copy"
                            | "update"
                            | "setdefault"
                    );
                    let guarded = ancestor(node, &["ternary_expression", "conditional_expression"])
                        .is_some()
                        || node
                            .parent()
                            .is_some_and(|p| text(p, source).contains("?."));
                    if !declared && !builtin && !guarded {
                        mechanism = Some(MEMBER);
                    }
                }
            }
        }
        if let Some(mechanism) = mechanism {
            add(report, Span::of(node), DEFECT, mechanism, "The locally observed operand or dynamic type contract may change the operation's meaning or fail at runtime. Caller intent and external type invariants are not established.", &[]);
        }
    }
}

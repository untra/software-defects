use crate::{
    annotations::nodes,
    defect_support::*,
    model::*,
    type_errors::{kind, Kind},
    unstructured::add,
};
use tree_sitter::Node;
pub(crate) const DEFECT: &str = "index-out-of-bounds";
const INDEX: &str = "unguarded-index";
const STRING_SLICE: &str = "byte-vs-char-slice";
const OPTIONAL: &str = "unchecked-optional-access";

fn guarded(node: Node<'_>, source: &str) -> bool {
    if node
        .parent()
        .is_some_and(|p| text(p, source).contains("?."))
    {
        return true;
    }
    let mut parent = node.parent();
    while let Some(p) = parent {
        if matches!(p.kind(), "ternary_expression" | "conditional_expression") {
            if field(p, &["condition"])
                .or_else(|| {
                    if p.kind() == "conditional_expression" {
                        p.named_child(1)
                    } else {
                        None
                    }
                })
                .is_some_and(|c| {
                    text(c, source).contains(text(node, source))
                        || node.named_child(0).is_some_and(|collection| {
                            text(c, source).trim_matches(['(', ')', ' '])
                                == text(collection, source)
                                && node.named_child(1).is_some_and(|i| text(i, source) == "0")
                        })
                })
            {
                return true;
            }
        }
        if matches!(
            p.kind(),
            "function_declaration" | "function_definition" | "function_item"
        ) {
            break;
        }
        parent = p.parent();
    }
    false
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
        if !path.contains(DEFECT)
            || !matches!(capture, "unstructured_index" | "unstructured_optional")
            || node.has_error()
            || guarded(node, source)
        {
            continue;
        }
        if capture == "unstructured_optional" {
            if method(node, source) == "unwrap"
                && receiver(node).is_some_and(|n| {
                    calls(n) && matches!(method(n, source), "last" | "first" | "get")
                })
            {
                add(report, Span::of(node), DEFECT, OPTIONAL, "An optional collection lookup is unwrapped without an established membership contract.", &[]);
            }
            continue;
        }
        let Some(collection) =
            field(node, &["object", "value", "operand"]).or_else(|| node.named_child(0))
        else {
            continue;
        };
        let Some(index) = field(node, &["index", "subscript", "field", "property"])
            .or_else(|| node.named_child(1))
        else {
            continue;
        };
        let initial = value(collection, &all, source, &resolve).unwrap_or(collection);
        let checked_length = all
            .iter()
            .filter(|n| {
                n.kind() == "if_statement"
                    && n.end_byte() < node.start_byte()
                    && owner(report, **n) == owner(report, node)
            })
            .any(|guard| {
                let Some(condition) = field(*guard, &["condition"]) else {
                    return false;
                };
                let Some(body) = field(*guard, &["consequence"]) else {
                    return false;
                };
                let exits = nodes(body).iter().any(|n| {
                    matches!(n.kind(), "return_statement" | "raise_statement")
                        && ancestor(*n, &["if_statement"]) == Some(*guard)
                });
                let same_block = guard
                    .parent()
                    .is_some_and(|block| Span::of(block).contains(Span::of(node)));
                let invalidated = all.iter().any(|n| {
                    n.start_byte() > guard.end_byte()
                        && n.end_byte() < node.start_byte()
                        && (assignment(*n)
                            && left(*n)
                                .is_some_and(|l| base(l, source) == base(collection, source))
                            || calls(*n)
                                && receiver(*n)
                                    .is_some_and(|r| base(r, source) == base(collection, source))
                            || matches!(n.kind(), "await" | "await_expression"))
                });
                if !exits || !same_block || invalidated {
                    return false;
                }
                nodes(condition).iter().any(|comparison| {
                    let (Some(a), Some(b)) = (
                        field(*comparison, &["left"]),
                        field(*comparison, &["right"]),
                    ) else {
                        return false;
                    };
                    if !calls(a)
                        || method(a, source) != "len"
                        || !args(a)
                            .first()
                            .is_some_and(|a| text(*a, source) == text(collection, source))
                    {
                        return false;
                    }
                    let Ok(bound) = text(b, source).parse::<i64>() else {
                        return false;
                    };
                    let operator = source[a.end_byte()..b.start_byte()].trim();
                    let index_text = text(index, source).replace(' ', "");
                    let offset = index_text.parse::<i64>().ok();
                    match operator {
                        "==" if bound == 0 => offset == Some(0) || index_text.ends_with("-1"),
                        "!=" => offset.is_some_and(|i| i >= 0 && i < bound),
                        "<=" => offset.is_some_and(|i| i >= 0 && i <= bound),
                        "<" => offset.is_some_and(|i| i >= 0 && i < bound),
                        _ => false,
                    }
                })
            });
        if checked_length {
            continue;
        }
        let initial_kind = kind(initial);
        let mut risk = false;
        let mut mechanism = INDEX;
        if initial_kind == Kind::Array {
            let length = initial.named_child_count() as i64;
            if let Ok(offset) = text(index, source).parse::<i64>() {
                risk = offset >= length
                    || offset < 0
                        && (matches!(report.language.as_str(), "javascript" | "typescript")
                            || -offset > length);
            }
        }
        if callee(initial).is_some_and(|n| text(n, source) == "Vec::new") {
            risk = true;
        }
        if initial.kind() == "macro_invocation" && text(initial, source).starts_with("vec![") {
            let values = text(initial, source)
                .trim_start_matches("vec![")
                .trim_end_matches(']');
            if values.is_empty() {
                risk = true;
            }
        }
        if report.language == "go"
            && initial.kind() == "composite_literal"
            && field(initial, &["type"]).is_some_and(|t| t.kind() == "map_type")
            && index.kind() == "interpreted_string_literal"
        {
            let checked = ancestor(node, &["short_var_declaration", "assignment_statement"])
                .and_then(left)
                .is_some_and(|l| l.named_child_count() > 1);
            risk = !checked
                && !nodes(initial).iter().any(|n| {
                    n.kind() == "interpreted_string_literal"
                        && text(*n, source) == text(index, source)
                });
        }
        if initial_kind == Kind::Object {
            let key = text(index, source).trim_matches(['\'', '"']);
            let found = initial.named_children(&mut initial.walk()).any(|entry| {
                field(entry, &["key"])
                    .is_some_and(|k| text(k, source).trim_matches(['\'', '"']) == key)
            });
            risk = !found
                && matches!(
                    index.kind(),
                    "string" | "string_literal" | "property_identifier"
                );
        }
        if initial_kind == Kind::String
            && report.language != "python"
            && !text(initial, source).is_ascii()
        {
            risk = true;
            mechanism = STRING_SLICE;
        }
        if matches!(node.kind(), "index_expression" | "subscript")
            && matches!(index.kind(), "binary_expression" | "binary_operator")
            && text(index, source).contains("-1")
        {
            risk = true;
        }
        if calls(collection)
            && matches!(method(collection, source), "split" | "Split" | "SplitN")
            && text(index, source) == "1"
        {
            risk = true;
        }
        if initial_kind == Kind::Unknown && collection.kind() == "identifier" {
            let fn_parameters = report
                .functions
                .iter()
                .find(|f| Some(f.range) == owner(report, node))
                .map(|f| &f.parameters);
            if fn_parameters.is_some_and(|ps| ps.iter().any(|p| p == text(collection, source)))
                && index.kind() == "identifier"
            {
                risk = true;
            }
        }
        if risk {
            add(report, Span::of(node), DEFECT, mechanism, "The local collection shape or string encoding does not establish a valid access at this index. Bounds, membership, or character-boundary handling is required.", &[Span::of(initial)]);
        }
    }
}

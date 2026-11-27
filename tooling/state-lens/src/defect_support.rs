use crate::model::{DocumentReport, Span};
use tree_sitter::Node;

pub(crate) fn text<'s>(node: Node<'_>, source: &'s str) -> &'s str {
    &source[node.byte_range()]
}
pub(crate) fn field<'a>(node: Node<'a>, fields: &[&str]) -> Option<Node<'a>> {
    fields
        .iter()
        .find_map(|name| node.child_by_field_name(name))
}
pub(crate) fn owner(report: &DocumentReport, node: Node<'_>) -> Option<Span> {
    report
        .functions
        .iter()
        .filter(|f| f.range.contains(Span::of(node)))
        .min_by_key(|f| f.range.len())
        .map(|f| f.range)
}
pub(crate) fn callee(node: Node<'_>) -> Option<Node<'_>> {
    field(node, &["function", "constructor"])
}
pub(crate) fn receiver(node: Node<'_>) -> Option<Node<'_>> {
    field(callee(node)?, &["object", "value", "operand"])
}
pub(crate) fn method<'s>(node: Node<'_>, source: &'s str) -> &'s str {
    callee(node)
        .map(|n| text(n, source).rsplit(['.', ':']).next().unwrap_or(""))
        .unwrap_or("")
}
pub(crate) fn args(node: Node<'_>) -> Vec<Node<'_>> {
    field(node, &["arguments"])
        .map(|n| n.named_children(&mut n.walk()).collect())
        .unwrap_or_default()
}
pub(crate) fn base<'s>(node: Node<'_>, source: &'s str) -> &'s str {
    text(node, source)
        .trim_start_matches(['&', '*'])
        .split(['.', '[', '('])
        .next()
        .unwrap_or("")
        .trim()
}
pub(crate) fn ancestor<'a>(mut node: Node<'a>, kinds: &[&str]) -> Option<Node<'a>> {
    while let Some(parent) = node.parent() {
        if kinds.contains(&parent.kind()) {
            return Some(parent);
        }
        node = parent;
    }
    None
}
pub(crate) fn assignment(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "assignment"
            | "assignment_expression"
            | "assignment_statement"
            | "variable_declarator"
            | "let_declaration"
            | "short_var_declaration"
            | "var_spec"
    )
}
pub(crate) fn left(node: Node<'_>) -> Option<Node<'_>> {
    field(node, &["left", "name", "pattern"])
}
pub(crate) fn right(node: Node<'_>) -> Option<Node<'_>> {
    field(node, &["right", "value"]).map(|n| {
        if n.kind() == "expression_list" && n.named_child_count() == 1 {
            n.named_child(0).unwrap()
        } else {
            n
        }
    })
}
pub(crate) fn calls(node: Node<'_>) -> bool {
    matches!(node.kind(), "call" | "call_expression" | "new_expression")
}

pub(crate) fn value<'a>(
    node: Node<'a>,
    all: &[Node<'a>],
    source: &str,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
) -> Option<Node<'a>> {
    if node.kind() != "identifier" {
        return Some(node);
    }
    let binding = resolve(text(node, source), Span::of(node))?;
    all.iter()
        .filter(|n| assignment(**n) && n.start_byte() <= node.start_byte())
        .filter(|n| {
            left(**n).is_some_and(|l| {
                text(l, source) == text(node, source)
                    && resolve(text(l, source), Span::of(l)) == Some(binding)
            })
        })
        .max_by_key(|n| n.start_byte())
        .and_then(|n| right(*n))
}

pub(crate) fn identifiers(node: Node<'_>) -> Vec<Node<'_>> {
    crate::annotations::nodes(node)
        .into_iter()
        .filter(|n| n.kind() == "identifier")
        .collect()
}

pub(crate) fn imported_call(
    node: Node<'_>,
    all: &[Node<'_>],
    source: &str,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
) -> Option<String> {
    let called = callee(node)?;
    let raw = text(called, source);
    let head = raw.split(['.', ':']).next()?;
    if resolve(head, Span::of(called)).is_some() {
        return None;
    }
    for import in all.iter().filter(|n| {
        matches!(n.kind(), "import_statement" | "import_from_statement")
            && n.start_byte() < called.start_byte()
    }) {
        if ancestor(*import, &["function_definition"])
            .is_some_and(|scope| !Span::of(scope).contains(Span::of(node)))
        {
            continue;
        }
        let module = field(*import, &["module_name"]).map(|n| text(n, source));
        for item in import.children_by_field_name("name", &mut import.walk()) {
            let original = field(item, &["name"]).unwrap_or(item);
            let local = field(item, &["alias"]).unwrap_or(original);
            if text(local, source) == head {
                return Some(format!(
                    "{}{}{}",
                    module.map(|m| format!("{m}.")).unwrap_or_default(),
                    text(original, source),
                    &raw[head.len()..]
                ));
            }
        }
    }
    Some(raw.to_owned())
}

use super::ir::{Container, Entry, Expr, Func, Index, Key, Param, Stmt};
use crate::model::Span;
use std::rc::Rc;
use tree_sitter::Node;

const SCALAR_IDENTIFIERS: [&str; 3] = ["true", "false", "iota"];
const TYPE_KINDS: [&str; 7] = [
    "slice_type",
    "array_type",
    "map_type",
    "pointer_type",
    "qualified_type",
    "channel_type",
    "function_type",
];

fn children(n: Node<'_>) -> Vec<Node<'_>> {
    n.named_children(&mut n.walk())
        .filter(|c| c.kind() != "comment")
        .collect()
}

pub fn lower(
    root: Node<'_>,
    source: &str,
    resolve: &dyn Fn(&str, Span) -> Option<Span>,
) -> Vec<Stmt> {
    let lower = Lower { source, resolve };
    let nodes = children(root);
    let mut out = vec![];
    for n in &nodes {
        if matches!(n.kind(), "function_declaration" | "method_declaration") {
            out.extend(lower.stmt(*n));
        }
    }
    for n in &nodes {
        if !matches!(n.kind(), "function_declaration" | "method_declaration") {
            out.extend(lower.stmt(*n));
        }
    }
    out
}

struct Lower<'a> {
    source: &'a str,
    resolve: &'a dyn Fn(&str, Span) -> Option<Span>,
}

impl Lower<'_> {
    fn text(&self, n: Node<'_>) -> &str {
        &self.source[n.byte_range()]
    }

    fn resolved(&self, n: Node<'_>) -> Option<Span> {
        (self.resolve)(self.text(n), Span::of(n))
    }

    fn key(&self, n: Node<'_>) -> Key {
        match self.resolved(n) {
            Some(decl) => Key::Binding(decl),
            None => Key::Builtin,
        }
    }

    fn block(&self, n: Option<Node<'_>>) -> Vec<Stmt> {
        let Some(n) = n else { return vec![] };
        let mut out = vec![];
        for c in children(n) {
            if c.kind() == "statement_list" {
                for s in children(c) {
                    out.extend(self.stmt(s));
                }
            } else {
                out.extend(self.stmt(c));
            }
        }
        out
    }

    fn type_name(&self, n: Node<'_>) -> String {
        let text = self.text(n).trim_start_matches('*');
        text.split('[').next().unwrap_or(text).trim().to_string()
    }

    fn param(&self, p: Node<'_>) -> Vec<Param> {
        let mut c = p.walk();
        p.children_by_field_name("name", &mut c)
            .map(|name| Param {
                name: self.text(name).into(),
                span: self.resolved(name).unwrap_or(Span::of(name)),
                default: None,
                requirement: None,
            })
            .collect()
    }

    fn function(&self, n: Node<'_>, receiver: Option<Node<'_>>) -> Rc<Func> {
        let params = n
            .child_by_field_name("parameters")
            .map(|ps| {
                children(ps)
                    .into_iter()
                    .flat_map(|p| self.param(p))
                    .collect()
            })
            .unwrap_or_default();
        let receiver_declaration = receiver.and_then(|r| children(r).into_iter().next());
        let receiver_by_value = receiver_declaration
            .and_then(|r| r.child_by_field_name("type"))
            .is_some_and(|t| t.kind() != "pointer_type");
        Rc::new(Func {
            span: Span::of(n),
            params,
            receiver: receiver_declaration.and_then(|r| self.param(r).into_iter().next()),
            body: self.block(n.child_by_field_name("body")),
            expression_body: None,
            defaults_at_definition: false,
            receiver_by_value,
            opaque: None,
        })
    }

    fn list<'t>(&self, n: Option<Node<'t>>) -> Vec<Node<'t>> {
        match n {
            Some(n) if n.kind() == "expression_list" => children(n),
            Some(n) => vec![n],
            None => vec![],
        }
    }

    fn stmt(&self, n: Node<'_>) -> Vec<Stmt> {
        let span = Span::of(n);
        match n.kind() {
            "function_declaration" => {
                let Some(name) = n.child_by_field_name("name") else { return vec![] };
                vec![Stmt::Bind {
                    key: Key::Binding(self.resolved(name).unwrap_or(Span::of(name))),
                    name: self.text(name).into(),
                    span: Span::of(name),
                    statement: span,
                    value: Some(Expr::Function(self.function(n, None))),
                    requirement: None,
                }]
            }
            "method_declaration" => {
                let (Some(name), Some(receiver)) = (n.child_by_field_name("name"), n.child_by_field_name("receiver")) else {
                    return vec![];
                };
                let type_name = children(receiver)
                    .into_iter()
                    .next()
                    .and_then(|r| r.child_by_field_name("type"))
                    .map(|t| self.type_name(t))
                    .unwrap_or_default();
                vec![Stmt::Method {
                    type_name,
                    name: self.text(name).into(),
                    func: self.function(n, Some(receiver)),
                }]
            }
            "var_declaration" | "const_declaration" => children(n)
                .into_iter()
                .filter(|s| matches!(s.kind(), "var_spec" | "const_spec"))
                .flat_map(|s| {
                    let mut c = s.walk();
                    let names: Vec<_> = s.children_by_field_name("name", &mut c).collect();
                    let values = self.list(s.child_by_field_name("value"));
                    self.declare(&names, &values, Span::of(s))
                })
                .collect(),
            "short_var_declaration" => {
                let names = self.list(n.child_by_field_name("left"));
                let values = self.list(n.child_by_field_name("right"));
                self.declare(&names, &values, span)
            }
            "assignment_statement" => {
                let targets = self.list(n.child_by_field_name("left"));
                let values = self.list(n.child_by_field_name("right"));
                let operator = n.child_by_field_name("operator").map(|o| self.text(o)).unwrap_or("=");
                if operator != "=" {
                    return targets
                        .iter()
                        .zip(values.iter())
                        .map(|(t, v)| Stmt::Assign {
                            target: self.expr(*t),
                            value: Expr::Scalar(vec![self.expr(*t), self.expr(*v)], span),
                            span,
                        })
                        .collect();
                }
                self.pairs(&targets, &values, span, false)
            }
            "inc_statement" | "dec_statement" => children(n)
                .into_iter()
                .map(|t| Stmt::Assign {
                    target: self.expr(t),
                    value: Expr::Scalar(vec![self.expr(t)], span),
                    span,
                })
                .collect(),
            "expression_statement" => children(n).into_iter().map(|e| Stmt::Eval(self.expr(e))).collect(),
            "block" => self.block(Some(n)),
            "if_statement" => {
                let mut out = n.child_by_field_name("initializer").map(|i| self.stmt(i)).unwrap_or_default();
                let condition = n.child_by_field_name("condition").map(|c| self.expr(c)).unwrap_or(Expr::Scalar(vec![], span));
                let then = self.block(n.child_by_field_name("consequence"));
                let otherwise = match n.child_by_field_name("alternative") {
                    Some(a) if a.kind() == "block" => self.block(Some(a)),
                    Some(a) => self.stmt(a),
                    None => vec![],
                };
                out.push(Stmt::If {
                    condition,
                    then,
                    otherwise,
                });
                out
            }
            "for_statement" => {
                let mut header = vec![];
                let mut body = vec![];
                for c in children(n) {
                    match c.kind() {
                        "for_clause" => {
                            if let Some(i) = c.child_by_field_name("initializer") {
                                header.extend(self.stmt(i));
                            }
                            if let Some(cond) = c.child_by_field_name("condition") {
                                body.push(Stmt::Eval(Expr::Scalar(vec![self.expr(cond)], Span::of(cond))));
                            }
                            if let Some(u) = c.child_by_field_name("update") {
                                body.extend(self.stmt(u));
                            }
                        }
                        "range_clause" => {
                            let right = c.child_by_field_name("right").map(|r| self.expr(r));
                            let left = self.list(c.child_by_field_name("left"));
                            if let Some(right) = right {
                                match left.get(1) {
                                    Some(value) if value.kind() == "identifier" && self.text(*value) != "_" => {
                                        body.push(Stmt::Bind {
                                            key: self.key(*value),
                                            name: self.text(*value).into(),
                                            span: Span::of(*value),
                                            statement: Span::of(c),
                                            value: Some(Expr::Index {
                                                base: Box::new(right),
                                                index: Index::Dynamic(Box::new(Expr::Scalar(vec![], Span::of(c)))),
                                                span: Span::of(*value),
                                            }),
                                            requirement: None,
                                        });
                                    }
                                    _ => body.push(Stmt::Eval(Expr::Scalar(vec![right], Span::of(c)))),
                                }
                            }
                        }
                        "block" => body.extend(self.block(Some(c))),
                        _ => body.push(Stmt::Eval(Expr::Scalar(vec![self.expr(c)], Span::of(c)))),
                    }
                }
                vec![Stmt::Loop { header, body, span }]
            }
            "expression_switch_statement" | "type_switch_statement" | "select_statement" => {
                let mut out = n.child_by_field_name("initializer").map(|i| self.stmt(i)).unwrap_or_default();
                if let Some(v) = n.child_by_field_name("value") {
                    out.push(Stmt::Eval(Expr::Scalar(vec![self.expr(v)], span)));
                }
                let mut cases = vec![];
                for case in children(n).into_iter().filter(|c| c.kind().ends_with("_case") || c.kind() == "default_case") {
                    let statements = children(case)
                        .into_iter()
                        .filter(|c| c.kind() == "statement_list")
                        .flat_map(|l| children(l).into_iter().flat_map(|s| self.stmt(s)).collect::<Vec<_>>())
                        .collect();
                    cases.push(statements);
                }
                let mut chain = vec![];
                for case in cases.into_iter().rev() {
                    chain = vec![Stmt::If {
                        condition: Expr::Scalar(vec![], span),
                        then: case,
                        otherwise: chain,
                    }];
                }
                out.extend(chain);
                out
            }
            "return_statement" => vec![Stmt::Return {
                value: self.list(children(n).first().copied()).first().map(|v| self.expr(*v)),
                span,
            }],
            "go_statement" | "defer_statement" => vec![Stmt::Unsupported {
                reason: "Deferred and concurrent calls execute at another time; their effects are not ordered here.",
                span,
                binds: vec![],
            }],
            "labeled_statement" => children(n).into_iter().skip(1).flat_map(|s| self.stmt(s)).collect(),
            "package_clause" | "import_declaration" | "type_declaration" | "break_statement" | "continue_statement" | "fallthrough_statement" | "goto_statement" | "empty_statement" => vec![],
            "send_statement" => vec![Stmt::Unsupported {
                reason: "Channel sends transfer values to other goroutines.",
                span,
                binds: vec![],
            }],
            _ => vec![Stmt::Eval(self.expr(n))],
        }
    }

    fn declare(&self, names: &[Node<'_>], values: &[Node<'_>], span: Span) -> Vec<Stmt> {
        self.pairs(names, values, span, true)
    }

    fn pairs(
        &self,
        targets: &[Node<'_>],
        values: &[Node<'_>],
        span: Span,
        declaring: bool,
    ) -> Vec<Stmt> {
        let mut out = vec![];
        let single = values.len() == 1 && targets.len() > 1;
        for (i, target) in targets.iter().enumerate() {
            if target.kind() == "identifier" && self.text(*target) == "_" {
                if let Some(v) = values.get(i) {
                    out.push(Stmt::Eval(Expr::Scalar(vec![self.expr(*v)], span)));
                }
                continue;
            }
            let value = match (single, values.get(i)) {
                (true, _) if i == 0 => self.expr(values[0]),
                (true, _) => Expr::Scalar(vec![], span),
                (false, Some(v)) => self.expr(*v),
                (false, None) => Expr::Scalar(vec![], span),
            };
            let declares = declaring
                && target.kind() == "identifier"
                && self
                    .resolved(*target)
                    .is_none_or(|d| d == Span::of(*target));
            if declares {
                out.push(Stmt::Bind {
                    key: Key::Binding(Span::of(*target)),
                    name: self.text(*target).into(),
                    span: Span::of(*target),
                    statement: span,
                    value: Some(value),
                    requirement: None,
                });
            } else {
                out.push(Stmt::Assign {
                    target: self.expr(*target),
                    value,
                    span,
                });
            }
        }
        out
    }

    fn unknown(&self, n: Node<'_>, reason: &'static str) -> Expr {
        Expr::Unknown {
            reason,
            inputs: children(n).into_iter().map(|c| self.expr(c)).collect(),
            span: Span::of(n),
        }
    }

    fn literal(&self, n: Node<'_>, ty: Option<Node<'_>>) -> Expr {
        let span = Span::of(n);
        let body = n
            .child_by_field_name("body")
            .or_else(|| (n.kind() == "literal_value").then_some(n));
        let kind = ty.map(|t| t.kind()).unwrap_or("");
        let (container, value_semantics) = match kind {
            "slice_type" => (Container::Array, false),
            "array_type" | "implicit_length_array_type" => (Container::Array, true),
            "map_type" => (Container::Map, false),
            _ => (Container::Struct, true),
        };
        let mut entries = vec![];
        for element in body.map(children).unwrap_or_default() {
            match element.kind() {
                "keyed_element" => {
                    let key = element
                        .child_by_field_name("key")
                        .and_then(|k| children(k).into_iter().next().or(Some(k)));
                    let value = element
                        .child_by_field_name("value")
                        .and_then(|v| children(v).into_iter().next().or(Some(v)));
                    let name = key.and_then(|k| match k.kind() {
                        "identifier" | "field_identifier" if container == Container::Struct => {
                            Some(self.text(k).to_string())
                        }
                        "interpreted_string_literal" | "raw_string_literal" => Some(
                            self.text(k)
                                .trim_matches(|c| c == '"' || c == '`')
                                .to_string(),
                        ),
                        "int_literal" => Some(self.text(k).to_string()),
                        _ => None,
                    });
                    match (name, value) {
                        (Some(name), Some(value)) => {
                            entries.push(Entry::Field(name, self.element(value)))
                        }
                        _ => entries.push(Entry::Opaque(
                            "Composite literal keys are not resolved.",
                            Span::of(element),
                        )),
                    }
                }
                "literal_element" => {
                    let value = children(element).into_iter().next().unwrap_or(element);
                    if container == Container::Struct {
                        entries.push(Entry::Opaque(
                            "Positional struct fields are not resolved.",
                            Span::of(element),
                        ));
                    } else {
                        entries.push(Entry::Element(self.element(value)));
                    }
                }
                _ => {}
            }
        }
        Expr::Literal {
            container,
            type_name: ty.map(|t| self.type_name(t)),
            entries,
            elements: vec![],
            span,
            value_semantics,
        }
    }

    fn element(&self, n: Node<'_>) -> Expr {
        if n.kind() == "literal_value" {
            self.literal(n, None)
        } else {
            self.expr(n)
        }
    }

    fn index(&self, n: Node<'_>) -> Index {
        match n.kind() {
            "int_literal" => match self.text(n).parse() {
                Ok(i) => Index::Constant(i),
                Err(_) => Index::Dynamic(Box::new(self.expr(n))),
            },
            "interpreted_string_literal" | "raw_string_literal" => {
                Index::Name(self.text(n).trim_matches(|c| c == '"' || c == '`').into())
            }
            _ => Index::Dynamic(Box::new(self.expr(n))),
        }
    }

    fn expr(&self, n: Node<'_>) -> Expr {
        let span = Span::of(n);
        match n.kind() {
            "identifier" => {
                let name = self.text(n);
                let key = self.key(n);
                if key == Key::Builtin && SCALAR_IDENTIFIERS.contains(&name) {
                    return Expr::Scalar(vec![], span);
                }
                if key == Key::Builtin && name == "nil" {
                    return Expr::Null(span);
                }
                Expr::Var {
                    name: name.into(),
                    span,
                    key,
                }
            }
            "int_literal" => match self.text(n).parse() {
                Ok(v) => Expr::Integer(v, span),
                Err(_) => Expr::Scalar(vec![], span),
            },
            "interpreted_string_literal"
            | "raw_string_literal"
            | "float_literal"
            | "rune_literal"
            | "imaginary_literal"
            | "true"
            | "false"
            | "iota" => Expr::Scalar(vec![], span),
            "nil" => Expr::Null(span),
            "selector_expression" => {
                let (Some(operand), Some(field)) = (
                    n.child_by_field_name("operand"),
                    n.child_by_field_name("field"),
                ) else {
                    return self.unknown(n, "Selector shape is not modeled.");
                };
                Expr::Field {
                    base: Box::new(self.expr(operand)),
                    name: self.text(field).into(),
                    span,
                }
            }
            "index_expression" => {
                let (Some(operand), Some(index)) = (
                    n.child_by_field_name("operand"),
                    n.child_by_field_name("index"),
                ) else {
                    return self.unknown(n, "Index shape is not modeled.");
                };
                Expr::Index {
                    base: Box::new(self.expr(operand)),
                    index: self.index(index),
                    span,
                }
            }
            "slice_expression" => {
                let Some(operand) = n.child_by_field_name("operand") else {
                    return self.unknown(n, "Slice shape is not modeled.");
                };
                let part = |f: &str| n.child_by_field_name(f).map(|p| Box::new(self.expr(p)));
                Expr::Slice {
                    base: Box::new(self.expr(operand)),
                    low: part("start"),
                    high: part("end"),
                    max: part("capacity"),
                    span,
                }
            }
            "composite_literal" => self.literal(n, n.child_by_field_name("type")),
            "unary_expression" => {
                let operator = n
                    .child_by_field_name("operator")
                    .map(|o| self.text(o))
                    .unwrap_or("");
                let Some(operand) = n.child_by_field_name("operand") else {
                    return self.unknown(n, "Unary shape is not modeled.");
                };
                match operator {
                    "&" => Expr::AddressOf {
                        operand: Box::new(self.expr(operand)),
                        span,
                    },
                    "*" => self.expr(operand),
                    "<-" => self.unknown(n, "Channel receives are not modeled."),
                    _ => Expr::Scalar(vec![self.expr(operand)], span),
                }
            }
            "parenthesized_expression" => children(n)
                .first()
                .map(|c| self.expr(*c))
                .unwrap_or(Expr::Scalar(vec![], span)),
            "type_assertion_expression" => n
                .child_by_field_name("operand")
                .map(|o| self.expr(o))
                .unwrap_or(Expr::Scalar(vec![], span)),
            "binary_expression" => Expr::Scalar(
                ["left", "right"]
                    .iter()
                    .filter_map(|f| n.child_by_field_name(f))
                    .map(|c| self.expr(c))
                    .collect(),
                span,
            ),
            "func_literal" => Expr::Function(self.function(n, None)),
            "call_expression" => self.call(n),
            _ => self.unknown(
                n,
                "Expression form is not modeled by object-identity analysis.",
            ),
        }
    }

    fn call(&self, n: Node<'_>) -> Expr {
        let span = Span::of(n);
        let Some(function) = n.child_by_field_name("function") else {
            return self.unknown(n, "Call shape is not modeled.");
        };
        let arguments: Vec<Node<'_>> = n
            .child_by_field_name("arguments")
            .map(children)
            .unwrap_or_default();
        if TYPE_KINDS.contains(&function.kind()) {
            return Expr::Sequence(
                vec![
                    Expr::Scalar(arguments.iter().map(|a| self.expr(*a)).collect(), span),
                    Expr::Literal {
                        container: Container::Array,
                        type_name: None,
                        entries: vec![],
                        elements: vec![],
                        span,
                        value_semantics: false,
                    },
                ],
                span,
            );
        }
        let builtin = function.kind() == "identifier" && self.key(function) == Key::Builtin;
        if builtin && matches!(self.text(function), "make" | "new") {
            let ty = arguments.first();
            if ty.is_some_and(|t| t.kind() == "map_type") {
                return Expr::Literal {
                    container: Container::Map,
                    type_name: None,
                    entries: vec![],
                    elements: vec![],
                    span,
                    value_semantics: false,
                };
            }
            if self.text(function) == "new" {
                let literal = Expr::Literal {
                    container: Container::Struct,
                    type_name: ty.map(|t| self.type_name(*t)),
                    entries: vec![],
                    elements: vec![],
                    span,
                    value_semantics: true,
                };
                return Expr::AddressOf {
                    operand: Box::new(literal),
                    span,
                };
            }
            return Expr::Call {
                callee: Box::new(self.expr(function)),
                args: arguments.iter().skip(1).map(|a| self.expr(*a)).collect(),
                span,
            };
        }
        let args = arguments
            .iter()
            .map(|a| match a.kind() {
                "variadic_argument" => Expr::Variadic(
                    Box::new(
                        children(*a)
                            .first()
                            .map(|c| self.expr(*c))
                            .unwrap_or(Expr::Scalar(vec![], Span::of(*a))),
                    ),
                    Span::of(*a),
                ),
                _ => self.expr(*a),
            })
            .collect();
        Expr::Call {
            callee: Box::new(self.expr(function)),
            args,
            span,
        }
    }
}

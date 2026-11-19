use super::ir::{Container, Entry, Expr, Func, Index, Key, Param, RequirementKind, Stmt};
use crate::model::Span;
use std::rc::Rc;
use tree_sitter::Node;

const READONLY_TYPE_PREFIXES: [&str; 3] = ["Readonly<", "ReadonlyArray<", "readonly "];
const SCALAR_GLOBALS: [&str; 3] = ["undefined", "NaN", "Infinity"];
const COLLECTION_CONSTRUCTORS: [(&str, Container); 3] = [
    ("Map", Container::Map),
    ("Set", Container::Set),
    ("Object", Container::Object),
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
    Lower { source, resolve }.block(&children(root))
}

struct Lower<'a> {
    source: &'a str,
    resolve: &'a dyn Fn(&str, Span) -> Option<Span>,
}

impl Lower<'_> {
    fn text(&self, n: Node<'_>) -> &str {
        &self.source[n.byte_range()]
    }

    fn key(&self, n: Node<'_>) -> Key {
        match (self.resolve)(self.text(n), Span::of(n)) {
            Some(decl) => Key::Binding(decl),
            None => Key::Builtin,
        }
    }

    fn declared(&self, n: Node<'_>) -> Key {
        Key::Binding((self.resolve)(self.text(n), Span::of(n)).unwrap_or(Span::of(n)))
    }

    fn block(&self, nodes: &[Node<'_>]) -> Vec<Stmt> {
        let mut out = vec![];
        for n in nodes {
            let n = if n.kind() == "export_statement" {
                match n.child_by_field_name("declaration") {
                    Some(d) => d,
                    None => continue,
                }
            } else {
                *n
            };
            if n.kind() == "function_declaration" {
                out.extend(self.stmt(n));
            }
        }
        for n in nodes {
            let n = if n.kind() == "export_statement" {
                match n.child_by_field_name("declaration") {
                    Some(d) => d,
                    None => continue,
                }
            } else {
                *n
            };
            if n.kind() != "function_declaration" {
                out.extend(self.stmt(n));
            }
        }
        out
    }

    fn body(&self, n: Option<Node<'_>>) -> Vec<Stmt> {
        match n {
            Some(n) if n.kind() == "statement_block" => self.block(&children(n)),
            Some(n) => self.stmt(n),
            None => vec![],
        }
    }

    fn readonly_type(&self, annotation: Option<Node<'_>>) -> Option<Span> {
        let annotation = annotation?;
        let text = self.text(annotation).trim_start_matches(':').trim();
        READONLY_TYPE_PREFIXES
            .iter()
            .any(|p| text.starts_with(p))
            .then(|| Span::of(annotation))
    }

    fn stmt(&self, n: Node<'_>) -> Vec<Stmt> {
        match n.kind() {
            "expression_statement" => children(n)
                .into_iter()
                .map(|e| self.expression_statement(e))
                .collect(),
            "lexical_declaration" | "variable_declaration" => children(n)
                .into_iter()
                .filter(|d| d.kind() == "variable_declarator")
                .map(|d| self.declarator(d))
                .collect(),
            "if_statement" => {
                let condition = n
                    .child_by_field_name("condition")
                    .map(|c| self.expr(c))
                    .unwrap_or(Expr::Scalar(vec![], Span::of(n)));
                let then = self.body(n.child_by_field_name("consequence"));
                let otherwise = n
                    .child_by_field_name("alternative")
                    .map(|a| {
                        children(a)
                            .into_iter()
                            .flat_map(|c| self.body(Some(c)))
                            .collect()
                    })
                    .unwrap_or_default();
                vec![Stmt::If {
                    condition,
                    then,
                    otherwise,
                }]
            }
            "statement_block" => self.block(&children(n)),
            "return_statement" => vec![Stmt::Return {
                value: children(n).first().map(|c| self.expr(*c)),
                span: Span::of(n),
            }],
            "throw_statement" => vec![Stmt::Throw {
                value: children(n).first().map(|c| self.expr(*c)),
                span: Span::of(n),
            }],
            "for_statement" => {
                let header = n
                    .child_by_field_name("initializer")
                    .map(|i| self.stmt(i))
                    .unwrap_or_default();
                let mut body = vec![];
                for field in ["condition", "increment"] {
                    if let Some(c) = n.child_by_field_name(field) {
                        body.extend(self.stmt_or_expr(c));
                    }
                }
                body.extend(self.body(n.child_by_field_name("body")));
                vec![Stmt::Loop {
                    header,
                    body,
                    span: Span::of(n),
                }]
            }
            "for_in_statement" => {
                let right = n.child_by_field_name("right").map(|r| self.expr(r));
                let of = n
                    .child_by_field_name("operator")
                    .is_some_and(|o| self.text(o) == "of");
                let mut body = vec![];
                if let (Some(left), Some(right)) = (n.child_by_field_name("left"), right) {
                    let name = if left.kind() == "identifier" {
                        Some(left)
                    } else {
                        children(left)
                            .into_iter()
                            .find(|c| c.kind() == "identifier")
                    };
                    if let Some(name) = name {
                        let span = Span::of(n);
                        let value = if of {
                            Expr::Index {
                                base: Box::new(right),
                                index: Index::Dynamic(Box::new(Expr::Scalar(vec![], span))),
                                span: Span::of(name),
                            }
                        } else {
                            Expr::Scalar(vec![right], span)
                        };
                        body.push(Stmt::Bind {
                            key: self.declared(name),
                            name: self.text(name).into(),
                            span: Span::of(name),
                            statement: Span::of(left),
                            value: Some(value),
                            requirement: None,
                        });
                    }
                }
                body.extend(self.body(n.child_by_field_name("body")));
                vec![Stmt::Loop {
                    header: vec![],
                    body,
                    span: Span::of(n),
                }]
            }
            "while_statement" | "do_statement" => {
                let mut body = vec![];
                if let Some(c) = n.child_by_field_name("condition") {
                    body.push(Stmt::Eval(Expr::Scalar(vec![self.expr(c)], Span::of(c))));
                }
                body.extend(self.body(n.child_by_field_name("body")));
                vec![Stmt::Loop {
                    header: vec![],
                    body,
                    span: Span::of(n),
                }]
            }
            "try_statement" => {
                let body = self.body(n.child_by_field_name("body"));
                let handler = n
                    .child_by_field_name("handler")
                    .map(|h| self.body(h.child_by_field_name("body")))
                    .unwrap_or_default();
                let finally = n
                    .child_by_field_name("finalizer")
                    .map(|f| self.body(f.child_by_field_name("body")))
                    .unwrap_or_default();
                vec![Stmt::Try {
                    body,
                    handler,
                    finally,
                    span: Span::of(n),
                }]
            }
            "switch_statement" => {
                let value = n.child_by_field_name("value").map(|v| self.expr(v));
                let mut cases: Vec<Vec<Stmt>> = vec![];
                if let Some(body) = n.child_by_field_name("body") {
                    for case in children(body) {
                        let statements: Vec<Node<'_>> = {
                            let mut c = case.walk();
                            case.children_by_field_name("body", &mut c).collect()
                        };
                        cases.push(self.block(&statements));
                    }
                }
                let mut out = vec![];
                if let Some(v) = value {
                    out.push(Stmt::Eval(Expr::Scalar(vec![v], Span::of(n))));
                }
                let mut chain = vec![];
                for case in cases.into_iter().rev() {
                    chain = vec![Stmt::If {
                        condition: Expr::Scalar(vec![], Span::of(n)),
                        then: case,
                        otherwise: chain,
                    }];
                }
                out.extend(chain);
                out
            }
            "function_declaration" => {
                let Some(name) = n.child_by_field_name("name") else {
                    return vec![];
                };
                vec![Stmt::Bind {
                    key: self.declared(name),
                    name: self.text(name).into(),
                    span: Span::of(name),
                    statement: Span::of(n),
                    value: Some(Expr::Function(self.function(n))),
                    requirement: None,
                }]
            }
            "generator_function_declaration"
            | "class_declaration"
            | "abstract_class_declaration"
            | "enum_declaration" => {
                let binds = n
                    .child_by_field_name("name")
                    .map(|name| {
                        vec![(
                            self.declared(name),
                            self.text(name).to_string(),
                            Span::of(name),
                        )]
                    })
                    .unwrap_or_default();
                vec![Stmt::Unsupported {
                    reason:
                        "Classes, generators and enums are not modeled by object-identity analysis.",
                    span: Span::of(n),
                    binds,
                }]
            }
            "labeled_statement" => self.body(n.child_by_field_name("body")),
            "import_statement"
            | "export_statement"
            | "break_statement"
            | "continue_statement"
            | "empty_statement"
            | "debugger_statement"
            | "type_alias_declaration"
            | "interface_declaration"
            | "ambient_declaration"
            | "hash_bang_line" => vec![],
            _ => self.stmt_or_expr(n),
        }
    }

    fn stmt_or_expr(&self, n: Node<'_>) -> Vec<Stmt> {
        if n.kind().ends_with("_declaration") || n.kind().ends_with("_statement") {
            return vec![Stmt::Unsupported {
                reason: "Statement form is not modeled by object-identity analysis.",
                span: Span::of(n),
                binds: vec![],
            }];
        }
        vec![self.expression_statement(n)]
    }

    fn expression_statement(&self, n: Node<'_>) -> Stmt {
        match n.kind() {
            "assignment_expression" => self.assignment(n),
            "augmented_assignment_expression" | "update_expression" => self.update(n),
            _ => Stmt::Eval(self.expr(n)),
        }
    }

    fn assignment(&self, n: Node<'_>) -> Stmt {
        let (Some(left), Some(right)) = (
            n.child_by_field_name("left"),
            n.child_by_field_name("right"),
        ) else {
            return Stmt::Eval(Expr::Scalar(vec![], Span::of(n)));
        };
        if !matches!(
            left.kind(),
            "identifier" | "member_expression" | "subscript_expression"
        ) {
            return Stmt::Unsupported {
                reason: "Destructuring assignment is not modeled.",
                span: Span::of(n),
                binds: vec![],
            };
        }
        Stmt::Assign {
            target: self.expr(left),
            value: self.expr(right),
            span: Span::of(n),
        }
    }

    fn update(&self, n: Node<'_>) -> Stmt {
        let target = n
            .child_by_field_name("left")
            .or_else(|| n.child_by_field_name("argument"));
        let Some(target) = target else {
            return Stmt::Eval(Expr::Scalar(vec![], Span::of(n)));
        };
        let mut inputs = vec![self.expr(target)];
        if let Some(right) = n.child_by_field_name("right") {
            inputs.push(self.expr(right));
        }
        Stmt::Assign {
            target: self.expr(target),
            value: Expr::Scalar(inputs, Span::of(n)),
            span: Span::of(n),
        }
    }

    fn declarator(&self, d: Node<'_>) -> Stmt {
        let Some(name) = d.child_by_field_name("name") else {
            return Stmt::Unsupported {
                reason: "Declaration without a name is not modeled.",
                span: Span::of(d),
                binds: vec![],
            };
        };
        let value = d.child_by_field_name("value");
        if name.kind() != "identifier" {
            return Stmt::Unsupported {
                reason: "Destructuring declarations are not modeled.",
                span: Span::of(d),
                binds: vec![],
            };
        }
        let mut requirement = self
            .readonly_type(d.child_by_field_name("type"))
            .map(|s| (RequirementKind::DeclaredReadonly, s));
        if let Some(v) = value {
            if v.kind() == "as_expression" && self.text(v).trim_end().ends_with("as const") {
                requirement = Some((RequirementKind::DeclaredReadonly, Span::of(v)));
            }
        }
        Stmt::Bind {
            key: self.declared(name),
            name: self.text(name).into(),
            span: Span::of(name),
            statement: Span::of(d),
            value: value.map(|v| self.expr(v)),
            requirement,
        }
    }

    fn function(&self, n: Node<'_>) -> Rc<Func> {
        let mut params = vec![];
        let parameters = n
            .child_by_field_name("parameters")
            .or_else(|| n.child_by_field_name("parameter"));
        if let Some(parameters) = parameters {
            let list = if parameters.kind() == "identifier" {
                vec![parameters]
            } else {
                children(parameters)
            };
            for p in list {
                params.push(self.param(p));
            }
        }
        let opaque = (n.kind() == "generator_function"
            || self.text(n).trim_start().starts_with("async"))
        .then_some("Asynchronous and generator functions suspend; their bodies are not inlined.");
        let body = n.child_by_field_name("body");
        let (body, expression_body) = match body {
            Some(b) if b.kind() == "statement_block" => (self.block(&children(b)), None),
            Some(b) => (vec![], Some(self.expr(b))),
            None => (vec![], None),
        };
        Rc::new(Func {
            span: Span::of(n),
            params,
            receiver: None,
            body,
            expression_body,
            defaults_at_definition: false,
            receiver_by_value: false,
            opaque,
        })
    }

    fn param(&self, p: Node<'_>) -> Param {
        let (name, default, annotation) = match p.kind() {
            "identifier" => (p, None, None),
            "assignment_pattern" => (
                p.child_by_field_name("left").unwrap_or(p),
                p.child_by_field_name("right"),
                None,
            ),
            "required_parameter" | "optional_parameter" => (
                p.child_by_field_name("pattern").unwrap_or(p),
                p.child_by_field_name("value"),
                p.child_by_field_name("type"),
            ),
            _ => (p, None, None),
        };
        let span = match self.declared(name) {
            Key::Binding(s) => s,
            Key::Builtin => Span::of(name),
        };
        Param {
            name: self.text(name).into(),
            span,
            default: default.map(|d| self.expr(d)),
            requirement: self
                .readonly_type(annotation)
                .map(|s| (RequirementKind::ReadOnlyParameter, s)),
        }
    }

    fn unknown(&self, n: Node<'_>, reason: &'static str) -> Expr {
        Expr::Unknown {
            reason,
            inputs: children(n).into_iter().map(|c| self.expr(c)).collect(),
            span: Span::of(n),
        }
    }

    fn property_name(&self, key: Node<'_>) -> Option<String> {
        match key.kind() {
            "property_identifier" | "private_property_identifier" => Some(self.text(key).into()),
            "string" => Some(
                self.text(key)
                    .trim_matches(|c| c == '"' || c == '\'')
                    .into(),
            ),
            "number" => Some(self.text(key).into()),
            _ => None,
        }
    }

    fn expr(&self, n: Node<'_>) -> Expr {
        let span = Span::of(n);
        match n.kind() {
            "identifier" => {
                let name = self.text(n);
                let key = self.key(n);
                if key == Key::Builtin && SCALAR_GLOBALS.contains(&name) {
                    return Expr::Scalar(vec![], span);
                }
                Expr::Var {
                    name: name.into(),
                    span,
                    key,
                }
            }
            "this" | "super" => Expr::Unknown {
                reason: "Receiver objects are not modeled.",
                inputs: vec![],
                span,
            },
            "member_expression" => {
                let (Some(object), Some(property)) = (
                    n.child_by_field_name("object"),
                    n.child_by_field_name("property"),
                ) else {
                    return self.unknown(n, "Member expression shape is not modeled.");
                };
                Expr::Field {
                    base: Box::new(self.expr(object)),
                    name: self.text(property).into(),
                    span,
                }
            }
            "subscript_expression" => {
                let (Some(object), Some(index)) = (
                    n.child_by_field_name("object"),
                    n.child_by_field_name("index"),
                ) else {
                    return self.unknown(n, "Subscript shape is not modeled.");
                };
                let index = match index.kind() {
                    "number" => match self.text(index).parse::<u64>() {
                        Ok(i) => Index::Constant(i),
                        Err(_) => Index::Dynamic(Box::new(self.expr(index))),
                    },
                    "string" => Index::Name(
                        self.text(index)
                            .trim_matches(|c| c == '"' || c == '\'')
                            .into(),
                    ),
                    _ => Index::Dynamic(Box::new(self.expr(index))),
                };
                Expr::Index {
                    base: Box::new(self.expr(object)),
                    index,
                    span,
                }
            }
            "call_expression" => {
                let Some(function) = n.child_by_field_name("function") else {
                    return self.unknown(n, "Call shape is not modeled.");
                };
                if matches!(function.kind(), "import" | "super") {
                    return self.unknown(n, "Dynamic import and super calls are not modeled.");
                }
                Expr::Call {
                    callee: Box::new(self.expr(function)),
                    args: self.arguments(n.child_by_field_name("arguments")),
                    span,
                }
            }
            "new_expression" => {
                let constructor = n.child_by_field_name("constructor");
                let args = self.arguments(n.child_by_field_name("arguments"));
                if let Some(c) =
                    constructor.filter(|c| c.kind() == "identifier" && self.key(*c) == Key::Builtin)
                {
                    let name = self.text(c);
                    if name == "Array" {
                        return match args.len() {
                            1 => Expr::NewArray {
                                length: args.into_iter().next().map(Box::new),
                                span,
                            },
                            _ => Expr::Literal {
                                container: Container::Array,
                                type_name: None,
                                entries: args.into_iter().map(Entry::Element).collect(),
                                elements: vec![],
                                span,
                                value_semantics: false,
                            },
                        };
                    }
                    if let Some((_, container)) =
                        COLLECTION_CONSTRUCTORS.iter().find(|(n, _)| *n == name)
                    {
                        return Expr::Sequence(
                            vec![
                                Expr::Scalar(args, span),
                                Expr::Literal {
                                    container: *container,
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
                }
                Expr::Unknown {
                    reason: "Constructor behavior is not modeled.",
                    inputs: args,
                    span,
                }
            }
            "object" => {
                let mut entries = vec![];
                for c in children(n) {
                    match c.kind() {
                        "pair" => {
                            let key = c.child_by_field_name("key");
                            let value = c.child_by_field_name("value");
                            match (key.and_then(|k| self.property_name(k)), value) {
                                (Some(name), Some(value)) => {
                                    entries.push(Entry::Field(name, self.expr(value)))
                                }
                                _ => entries.push(Entry::Opaque(
                                    "Computed property keys are not resolved.",
                                    Span::of(c),
                                )),
                            }
                        }
                        "shorthand_property_identifier" => entries.push(Entry::Field(
                            self.text(c).into(),
                            Expr::Var {
                                name: self.text(c).into(),
                                span: Span::of(c),
                                key: self.key(c),
                            },
                        )),
                        "spread_element" => {
                            if let Some(inner) = children(c).first() {
                                entries.push(Entry::Spread(self.expr(*inner)));
                            }
                        }
                        _ => entries.push(Entry::Opaque(
                            "Object methods, getters and setters are not modeled.",
                            Span::of(c),
                        )),
                    }
                }
                Expr::Literal {
                    container: Container::Object,
                    type_name: None,
                    entries,
                    elements: vec![],
                    span,
                    value_semantics: false,
                }
            }
            "array" => Expr::Literal {
                container: Container::Array,
                type_name: None,
                entries: children(n)
                    .into_iter()
                    .map(|c| match c.kind() {
                        "spread_element" => Entry::Spread(
                            children(c)
                                .first()
                                .map(|i| self.expr(*i))
                                .unwrap_or(Expr::Scalar(vec![], Span::of(c))),
                        ),
                        _ => Entry::Element(self.expr(c)),
                    })
                    .collect(),
                elements: vec![],
                span,
                value_semantics: false,
            },
            "arrow_function" | "function_expression" | "function" => {
                Expr::Function(self.function(n))
            }
            "parenthesized_expression"
            | "as_expression"
            | "satisfies_expression"
            | "non_null_expression"
            | "type_assertion" => match children(n).first() {
                Some(inner) => self.expr(*inner),
                None => Expr::Scalar(vec![], span),
            },
            "sequence_expression" => Expr::Sequence(
                children(n).into_iter().map(|c| self.expr(c)).collect(),
                span,
            ),
            "assignment_expression" => Expr::Assign(Box::new(self.assignment(n)), span),
            "augmented_assignment_expression" | "update_expression" => {
                Expr::Assign(Box::new(self.update(n)), span)
            }
            "ternary_expression" => {
                let condition = n.child_by_field_name("condition").map(|c| self.expr(c));
                let options = ["consequence", "alternative"]
                    .iter()
                    .filter_map(|f| n.child_by_field_name(f))
                    .map(|c| self.expr(c))
                    .collect();
                Expr::Sequence(
                    vec![
                        Expr::Scalar(condition.into_iter().collect(), span),
                        Expr::Choice(options, span),
                    ],
                    span,
                )
            }
            "binary_expression" => {
                let operator = n
                    .child_by_field_name("operator")
                    .map(|o| self.text(o))
                    .unwrap_or("");
                let operands = ["left", "right"]
                    .iter()
                    .filter_map(|f| n.child_by_field_name(f))
                    .map(|c| self.expr(c))
                    .collect();
                if matches!(operator, "&&" | "||" | "??") {
                    Expr::Choice(operands, span)
                } else {
                    Expr::Scalar(operands, span)
                }
            }
            "unary_expression" => Expr::Scalar(
                children(n).into_iter().map(|c| self.expr(c)).collect(),
                span,
            ),
            "number" => match self.text(n).parse::<i64>() {
                Ok(v) => Expr::Integer(v, span),
                Err(_) => Expr::Scalar(vec![], span),
            },
            "string" | "true" | "false" | "null" | "undefined" | "regex" => {
                Expr::Scalar(vec![], span)
            }
            "template_string" => Expr::Scalar(
                children(n)
                    .into_iter()
                    .filter(|c| c.kind() == "template_substitution")
                    .flat_map(children)
                    .map(|c| self.expr(c))
                    .collect(),
                span,
            ),
            "await_expression" => self.unknown(
                n,
                "Suspension at await is not modeled; awaited values are unknown.",
            ),
            _ => self.unknown(
                n,
                "Expression form is not modeled by object-identity analysis.",
            ),
        }
    }

    fn arguments(&self, n: Option<Node<'_>>) -> Vec<Expr> {
        let Some(n) = n else { return vec![] };
        children(n)
            .into_iter()
            .map(|c| match c.kind() {
                "spread_element" => self.unknown(c, "Spread arguments are not modeled."),
                _ => self.expr(c),
            })
            .collect()
    }
}

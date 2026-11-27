use super::ir::{Container, Entry, Expr, Func, Index, Key, Param, RequirementKind, Stmt};
use crate::model::Span;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use tree_sitter::Node;

const READONLY_ANNOTATIONS: [&str; 6] = [
    "Sequence[",
    "Mapping[",
    "AbstractSet[",
    "tuple[",
    "Tuple[",
    "frozenset[",
];
const MODELED_MODULES: [&str; 2] = ["copy", "json"];
const OPAQUE_DECORATOR: &str =
    "Decorated functions may be replaced or wrapped; their bodies are not inlined.";

fn children(n: Node<'_>) -> Vec<Node<'_>> {
    n.named_children(&mut n.walk())
        .filter(|c| c.kind() != "comment")
        .collect()
}

pub fn lower(root: Node<'_>, source: &str) -> Vec<Stmt> {
    let lower = Lower {
        source,
        scopes: RefCell::new(vec![]),
        modules: RefCell::new(BTreeMap::new()),
    };
    lower.enter(&children(root), &[]);
    let out = lower.block(&children(root));
    lower.scopes.borrow_mut().pop();
    out
}

struct Lower<'a> {
    source: &'a str,
    scopes: RefCell<Vec<BTreeMap<String, Span>>>,
    modules: RefCell<BTreeMap<String, String>>,
}

impl Lower<'_> {
    fn text(&self, n: Node<'_>) -> &str {
        &self.source[n.byte_range()]
    }

    fn enter(&self, body: &[Node<'_>], params: &[Node<'_>]) {
        let mut names = BTreeMap::new();
        let mut excluded = BTreeSet::new();
        for p in params {
            names
                .entry(self.text(*p).to_string())
                .or_insert(Span::of(*p));
        }
        for n in body {
            self.targets(*n, &mut names, &mut excluded);
        }
        for name in excluded {
            names.remove(&name);
        }
        self.scopes.borrow_mut().push(names);
    }

    fn targets(
        &self,
        n: Node<'_>,
        names: &mut BTreeMap<String, Span>,
        excluded: &mut BTreeSet<String>,
    ) {
        match n.kind() {
            "function_definition" | "class_definition" => {
                if let Some(name) = n.child_by_field_name("name") {
                    names
                        .entry(self.text(name).into())
                        .or_insert(Span::of(name));
                }
                return;
            }
            "lambda"
            | "list_comprehension"
            | "dictionary_comprehension"
            | "set_comprehension"
            | "generator_expression" => return,
            "global_statement" | "nonlocal_statement" => {
                for c in children(n) {
                    excluded.insert(self.text(c).into());
                }
                return;
            }
            "import_statement" | "import_from_statement" => {
                for c in children(n) {
                    let alias = match c.kind() {
                        "aliased_import" => c.child_by_field_name("alias"),
                        "dotted_name" => children(c).first().copied(),
                        _ => None,
                    };
                    if let Some(alias) = alias {
                        let module = match c.kind() {
                            "aliased_import" => c
                                .child_by_field_name("name")
                                .map(|m| self.text(m).to_string()),
                            _ => Some(self.text(c).to_string()),
                        };
                        if n.kind() == "import_statement" {
                            if let Some(module) =
                                module.filter(|m| MODELED_MODULES.contains(&m.as_str()))
                            {
                                self.modules
                                    .borrow_mut()
                                    .insert(self.text(alias).into(), module);
                                continue;
                            }
                        }
                        names
                            .entry(self.text(alias).into())
                            .or_insert(Span::of(alias));
                    }
                }
                return;
            }
            "assignment" | "augmented_assignment" => {
                if let Some(left) = n.child_by_field_name("left") {
                    self.pattern_names(left, names);
                }
            }
            "for_statement" => {
                if let Some(left) = n.child_by_field_name("left") {
                    self.pattern_names(left, names);
                }
            }
            "as_pattern_target" => self.pattern_names(n, names),
            _ => {}
        }
        for c in children(n) {
            self.targets(c, names, excluded);
        }
    }

    fn pattern_names(&self, n: Node<'_>, names: &mut BTreeMap<String, Span>) {
        match n.kind() {
            "identifier" => {
                names.entry(self.text(n).into()).or_insert(Span::of(n));
            }
            "pattern_list" | "tuple_pattern" | "list_pattern" | "as_pattern_target" => {
                for c in children(n) {
                    self.pattern_names(c, names);
                }
            }
            _ => {}
        }
    }

    fn key(&self, name: &str) -> Key {
        for scope in self.scopes.borrow().iter().rev() {
            if let Some(span) = scope.get(name) {
                return Key::Binding(*span);
            }
        }
        Key::Builtin
    }

    fn var(&self, n: Node<'_>) -> Expr {
        let name = self.text(n).to_string();
        if let Some(module) = self.modules.borrow().get(&name) {
            return Expr::Var {
                name: module.clone(),
                span: Span::of(n),
                key: Key::Builtin,
            };
        }
        Expr::Var {
            key: self.key(&name),
            name,
            span: Span::of(n),
        }
    }

    fn readonly(&self, annotation: Option<Node<'_>>) -> Option<Span> {
        let annotation = annotation?;
        let text = self.text(annotation).trim();
        READONLY_ANNOTATIONS
            .iter()
            .any(|p| text.starts_with(p) || text.starts_with(&format!("typing.{p}")))
            .then(|| Span::of(annotation))
    }

    fn block(&self, nodes: &[Node<'_>]) -> Vec<Stmt> {
        nodes.iter().flat_map(|n| self.stmt(*n)).collect()
    }

    fn body(&self, n: Option<Node<'_>>) -> Vec<Stmt> {
        match n {
            Some(n) if n.kind() == "block" => self.block(&children(n)),
            Some(n) => self.stmt(n),
            None => vec![],
        }
    }

    fn stmt(&self, n: Node<'_>) -> Vec<Stmt> {
        let span = Span::of(n);
        match n.kind() {
            "expression_statement" => children(n)
                .into_iter()
                .flat_map(|c| self.expression_statement(c))
                .collect(),
            "if_statement" => {
                let condition = n
                    .child_by_field_name("condition")
                    .map(|c| self.expr(c))
                    .unwrap_or(Expr::Scalar(vec![], span));
                let then = self.body(n.child_by_field_name("consequence"));
                let mut alternatives: Vec<Node<'_>> = {
                    let mut c = n.walk();
                    n.children_by_field_name("alternative", &mut c).collect()
                };
                alternatives.reverse();
                let mut otherwise = vec![];
                for alt in alternatives {
                    otherwise = match alt.kind() {
                        "elif_clause" => vec![Stmt::If {
                            condition: alt
                                .child_by_field_name("condition")
                                .map(|c| self.expr(c))
                                .unwrap_or(Expr::Scalar(vec![], span)),
                            then: self.body(alt.child_by_field_name("consequence")),
                            otherwise,
                        }],
                        _ => self.body(alt.child_by_field_name("body")),
                    };
                }
                vec![Stmt::If {
                    condition,
                    then,
                    otherwise,
                }]
            }
            "for_statement" => {
                let mut body = vec![];
                if let (Some(left), Some(right)) = (
                    n.child_by_field_name("left"),
                    n.child_by_field_name("right"),
                ) {
                    let right = self.expr(right);
                    if left.kind() == "identifier" {
                        body.push(Stmt::Assign {
                            target: self.var(left),
                            value: Expr::Index {
                                base: Box::new(right),
                                index: Index::Dynamic(Box::new(Expr::Scalar(vec![], span))),
                                span: Span::of(left),
                            },
                            span: Span::of(left),
                        });
                    } else {
                        body.push(Stmt::Eval(Expr::Scalar(vec![right], span)));
                    }
                }
                body.extend(self.body(n.child_by_field_name("body")));
                vec![Stmt::Loop {
                    header: vec![],
                    body,
                    span,
                }]
            }
            "while_statement" => {
                let mut body = vec![];
                if let Some(c) = n.child_by_field_name("condition") {
                    body.push(Stmt::Eval(Expr::Scalar(vec![self.expr(c)], span)));
                }
                body.extend(self.body(n.child_by_field_name("body")));
                vec![Stmt::Loop {
                    header: vec![],
                    body,
                    span,
                }]
            }
            "try_statement" => {
                let body = self.body(n.child_by_field_name("body"));
                let mut handler = vec![];
                let mut finally = vec![];
                for c in children(n) {
                    match c.kind() {
                        "except_clause" | "except_group_clause" | "else_clause" => {
                            for b in children(c).into_iter().filter(|b| b.kind() == "block") {
                                handler.extend(self.body(Some(b)));
                            }
                        }
                        "finally_clause" => {
                            for b in children(c).into_iter().filter(|b| b.kind() == "block") {
                                finally.extend(self.body(Some(b)));
                            }
                        }
                        _ => {}
                    }
                }
                vec![Stmt::Try {
                    body,
                    handler,
                    finally,
                    span,
                }]
            }
            "with_statement" => {
                let mut out = vec![Stmt::Unsupported {
                    reason: "Context managers may run arbitrary enter and exit code.",
                    span,
                    binds: vec![],
                }];
                out.extend(self.body(n.child_by_field_name("body")));
                out
            }
            "return_statement" => vec![Stmt::Return {
                value: children(n).first().map(|c| self.expr(*c)),
                span,
            }],
            "raise_statement" => vec![Stmt::Throw {
                value: children(n).first().map(|c| self.expr(*c)),
                span,
            }],
            "function_definition" => self.definition(n, None),
            "decorated_definition" => {
                let definition = n.child_by_field_name("definition");
                match definition {
                    Some(d) if d.kind() == "function_definition" => {
                        self.definition(d, Some(OPAQUE_DECORATOR))
                    }
                    Some(d) => self.stmt(d),
                    None => vec![],
                }
            }
            "class_definition" => {
                let binds = n
                    .child_by_field_name("name")
                    .map(|name| {
                        vec![(
                            self.key(self.text(name)),
                            self.text(name).to_string(),
                            Span::of(name),
                        )]
                    })
                    .unwrap_or_default();
                vec![Stmt::Unsupported {
                    reason: "Class bodies and instance attributes are not modeled by object-identity analysis.",
                    span,
                    binds,
                }]
            }
            "import_statement"
            | "import_from_statement"
            | "future_import_statement"
            | "global_statement"
            | "nonlocal_statement"
            | "pass_statement"
            | "break_statement"
            | "continue_statement" => vec![],
            "delete_statement" => vec![Stmt::Unsupported {
                reason: "del statements are not modeled.",
                span,
                binds: vec![],
            }],
            _ => vec![Stmt::Eval(self.expr(n))],
        }
    }

    fn definition(&self, n: Node<'_>, opaque: Option<&'static str>) -> Vec<Stmt> {
        let Some(name) = n.child_by_field_name("name") else {
            return vec![];
        };
        let func = self.function(n, opaque);
        vec![Stmt::Bind {
            key: self.key(self.text(name)),
            name: self.text(name).into(),
            span: Span::of(name),
            statement: Span::of(n),
            value: Some(Expr::Function(func)),
            requirement: None,
        }]
    }

    fn function(&self, n: Node<'_>, opaque: Option<&'static str>) -> Rc<Func> {
        let mut params = vec![];
        let mut names = vec![];
        if let Some(parameters) = n.child_by_field_name("parameters") {
            for p in children(parameters) {
                let (name, default, annotation) = match p.kind() {
                    "identifier" => (Some(p), None, None),
                    "default_parameter" => (
                        p.child_by_field_name("name"),
                        p.child_by_field_name("value"),
                        None,
                    ),
                    "typed_parameter" => (
                        children(p).into_iter().find(|c| c.kind() == "identifier"),
                        None,
                        p.child_by_field_name("type"),
                    ),
                    "typed_default_parameter" => (
                        p.child_by_field_name("name"),
                        p.child_by_field_name("value"),
                        p.child_by_field_name("type"),
                    ),
                    "list_splat_pattern" | "dictionary_splat_pattern" => (
                        children(p).into_iter().find(|c| c.kind() == "identifier"),
                        None,
                        None,
                    ),
                    _ => (None, None, None),
                };
                let Some(name) = name else { continue };
                names.push(name);
                params.push((name, default.map(|d| self.expr(d)), annotation));
            }
        }
        let body_nodes = n
            .child_by_field_name("body")
            .map(children)
            .unwrap_or_default();
        let lambda_body = (n.kind() == "lambda")
            .then(|| n.child_by_field_name("body"))
            .flatten();
        self.enter(&body_nodes, &names);
        let params = params
            .into_iter()
            .map(|(name, default, annotation)| Param {
                name: self.text(name).into(),
                span: match self.key(self.text(name)) {
                    Key::Binding(s) => s,
                    Key::Builtin => Span::of(name),
                },
                default,
                requirement: self
                    .readonly(annotation)
                    .map(|s| (RequirementKind::ReadOnlyParameter, s)),
            })
            .collect();
        let body = if lambda_body.is_some() {
            vec![]
        } else {
            self.block(&body_nodes)
        };
        let expression_body = lambda_body.map(|b| self.expr(b));
        self.scopes.borrow_mut().pop();
        Rc::new(Func {
            span: Span::of(n),
            params,
            receiver: None,
            body,
            expression_body,
            defaults_at_definition: true,
            receiver_by_value: false,
            opaque,
        })
    }

    fn expression_statement(&self, n: Node<'_>) -> Vec<Stmt> {
        let span = Span::of(n);
        match n.kind() {
            "assignment" => {
                let (Some(left), right) = (
                    n.child_by_field_name("left"),
                    n.child_by_field_name("right"),
                ) else {
                    return vec![];
                };
                let value = right
                    .map(|r| self.expr(r))
                    .unwrap_or(Expr::Scalar(vec![], span));
                let requirement = self
                    .readonly(n.child_by_field_name("type"))
                    .map(|s| (RequirementKind::DeclaredReadonly, s));
                match left.kind() {
                    "identifier" => {
                        let key = self.key(self.text(left));
                        let declares = key == Key::Binding(Span::of(left));
                        if declares || requirement.is_some() {
                            vec![Stmt::Bind {
                                key,
                                name: self.text(left).into(),
                                span: Span::of(left),
                                statement: span,
                                value: Some(value),
                                requirement,
                            }]
                        } else {
                            vec![Stmt::Assign {
                                target: self.var(left),
                                value,
                                span,
                            }]
                        }
                    }
                    "attribute" | "subscript" => vec![Stmt::Assign {
                        target: self.expr(left),
                        value,
                        span,
                    }],
                    _ => {
                        let mut names = BTreeMap::new();
                        self.pattern_names(left, &mut names);
                        vec![
                            Stmt::Eval(Expr::Scalar(vec![value], span)),
                            Stmt::Unsupported {
                                reason: "Unpacking assignment is not modeled.",
                                span,
                                binds: names
                                    .into_iter()
                                    .map(|(name, s)| (self.key(&name), name, s))
                                    .collect(),
                            },
                        ]
                    }
                }
            }
            "augmented_assignment" => {
                let Some(left) = n.child_by_field_name("left") else {
                    return vec![];
                };
                let mut inputs = vec![self.expr(left)];
                if let Some(right) = n.child_by_field_name("right") {
                    inputs.push(self.expr(right));
                }
                vec![Stmt::Assign {
                    target: self.expr(left),
                    value: Expr::Scalar(inputs, span),
                    span,
                }]
            }
            _ => vec![Stmt::Eval(self.expr(n))],
        }
    }

    fn unknown(&self, n: Node<'_>, reason: &'static str) -> Expr {
        Expr::Unknown {
            reason,
            inputs: children(n).into_iter().map(|c| self.expr(c)).collect(),
            span: Span::of(n),
        }
    }

    fn string_key(&self, n: Node<'_>) -> Option<String> {
        if n.kind() != "string" || children(n).iter().any(|c| c.kind() == "interpolation") {
            return None;
        }
        children(n)
            .into_iter()
            .find(|c| c.kind() == "string_content")
            .map(|c| self.text(c).to_string())
            .or(Some(String::new()))
    }

    fn integer(&self, n: Node<'_>) -> Option<i64> {
        (n.kind() == "integer")
            .then(|| self.text(n).parse().ok())
            .flatten()
    }

    fn range_count(&self, n: Node<'_>) -> Option<u64> {
        let clause = children(n)
            .into_iter()
            .find(|c| c.kind() == "for_in_clause")?;
        let right = clause.child_by_field_name("right")?;
        if right.kind() != "call"
            || right.child_by_field_name("function").map(|f| self.text(f)) != Some("range")
        {
            return None;
        }
        let args = children(right.child_by_field_name("arguments")?);
        if args.len() != 1 {
            return None;
        }
        self.integer(args[0]).and_then(|v| u64::try_from(v).ok())
    }

    fn expr(&self, n: Node<'_>) -> Expr {
        let span = Span::of(n);
        match n.kind() {
            "identifier" => self.var(n),
            "attribute" => {
                let (Some(object), Some(attribute)) = (
                    n.child_by_field_name("object"),
                    n.child_by_field_name("attribute"),
                ) else {
                    return self.unknown(n, "Attribute shape is not modeled.");
                };
                Expr::Field {
                    base: Box::new(self.expr(object)),
                    name: self.text(attribute).into(),
                    span,
                }
            }
            "subscript" => {
                let (Some(value), Some(subscript)) = (
                    n.child_by_field_name("value"),
                    n.child_by_field_name("subscript"),
                ) else {
                    return self.unknown(n, "Subscript shape is not modeled.");
                };
                if subscript.kind() == "slice" {
                    return Expr::Sequence(
                        vec![
                            Expr::Scalar(
                                children(subscript)
                                    .into_iter()
                                    .map(|c| self.expr(c))
                                    .collect(),
                                Span::of(subscript),
                            ),
                            Expr::Call {
                                callee: Box::new(Expr::Field {
                                    base: Box::new(self.expr(value)),
                                    name: "copy".into(),
                                    span,
                                }),
                                args: vec![],
                                span,
                            },
                        ],
                        span,
                    );
                }
                let index =
                    if let Some(i) = self.integer(subscript).and_then(|v| u64::try_from(v).ok()) {
                        Index::Constant(i)
                    } else if let Some(k) = self.string_key(subscript) {
                        Index::Name(k)
                    } else {
                        Index::Dynamic(Box::new(self.expr(subscript)))
                    };
                Expr::Index {
                    base: Box::new(self.expr(value)),
                    index,
                    span,
                }
            }
            "call" => {
                let Some(function) = n.child_by_field_name("function") else {
                    return self.unknown(n, "Call shape is not modeled.");
                };
                let mut args = vec![];
                if let Some(arguments) = n.child_by_field_name("arguments") {
                    for a in children(arguments) {
                        args.push(match a.kind() {
                            "keyword_argument" => match (
                                a.child_by_field_name("name"),
                                a.child_by_field_name("value"),
                            ) {
                                (Some(name), Some(value)) => Expr::Named(
                                    self.text(name).into(),
                                    Box::new(self.expr(value)),
                                    Span::of(a),
                                ),
                                _ => self.unknown(a, "Keyword argument shape is not modeled."),
                            },
                            "list_splat" | "dictionary_splat" => {
                                self.unknown(a, "Argument unpacking is not modeled.")
                            }
                            _ => self.expr(a),
                        });
                    }
                }
                Expr::Call {
                    callee: Box::new(self.expr(function)),
                    args,
                    span,
                }
            }
            "list" | "tuple" | "set" => Expr::Literal {
                container: if n.kind() == "set" {
                    Container::Set
                } else {
                    Container::Array
                },
                type_name: None,
                entries: children(n)
                    .into_iter()
                    .map(|c| match c.kind() {
                        "list_splat" => Entry::Spread(
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
            "dictionary" => Expr::Literal {
                container: Container::Map,
                type_name: None,
                entries: children(n)
                    .into_iter()
                    .map(|c| match c.kind() {
                        "pair" => {
                            let key = c.child_by_field_name("key").and_then(|k| {
                                self.string_key(k)
                                    .or_else(|| self.integer(k).map(|v| v.to_string()))
                            });
                            match (key, c.child_by_field_name("value")) {
                                (Some(k), Some(v)) => Entry::Field(k, self.expr(v)),
                                _ => Entry::Opaque(
                                    "Non-literal dictionary keys are not resolved.",
                                    Span::of(c),
                                ),
                            }
                        }
                        "dictionary_splat" => Entry::Spread(
                            children(c)
                                .first()
                                .map(|i| self.expr(*i))
                                .unwrap_or(Expr::Scalar(vec![], Span::of(c))),
                        ),
                        _ => Entry::Opaque("Dictionary entry form is not modeled.", Span::of(c)),
                    })
                    .collect(),
                elements: vec![],
                span,
                value_semantics: false,
            },
            "list_comprehension"
            | "set_comprehension"
            | "dictionary_comprehension"
            | "generator_expression" => {
                let body = n.child_by_field_name("body");
                let element = match body {
                    Some(b) if b.kind() != "pair" => self.expr(b),
                    _ => Expr::Scalar(vec![], span),
                };
                Expr::Comprehension {
                    element: Box::new(element),
                    count: self.range_count(n),
                    span,
                }
            }
            "binary_operator" => {
                let operator = n
                    .child_by_field_name("operator")
                    .map(|o| self.text(o))
                    .unwrap_or("");
                let (Some(left), Some(right)) = (
                    n.child_by_field_name("left"),
                    n.child_by_field_name("right"),
                ) else {
                    return self.unknown(n, "Operator shape is not modeled.");
                };
                if operator == "*" {
                    if left.kind() == "list" {
                        return Expr::Repeat {
                            element: Box::new(self.expr(left)),
                            count: Box::new(self.expr(right)),
                            span,
                        };
                    }
                    if right.kind() == "list" {
                        return Expr::Repeat {
                            element: Box::new(self.expr(right)),
                            count: Box::new(self.expr(left)),
                            span,
                        };
                    }
                }
                Expr::Scalar(vec![self.expr(left), self.expr(right)], span)
            }
            "boolean_operator" => Expr::Choice(
                ["left", "right"]
                    .iter()
                    .filter_map(|f| n.child_by_field_name(f))
                    .map(|c| self.expr(c))
                    .collect(),
                span,
            ),
            "not_operator" | "comparison_operator" | "unary_operator" => Expr::Scalar(
                children(n).into_iter().map(|c| self.expr(c)).collect(),
                span,
            ),
            "conditional_expression" => {
                let parts = children(n);
                if parts.len() != 3 {
                    return self.unknown(n, "Conditional expression shape is not modeled.");
                }
                Expr::Sequence(
                    vec![
                        Expr::Scalar(vec![self.expr(parts[1])], span),
                        Expr::Choice(vec![self.expr(parts[0]), self.expr(parts[2])], span),
                    ],
                    span,
                )
            }
            "lambda" => Expr::Function(self.function(n, None)),
            "parenthesized_expression" => children(n)
                .first()
                .map(|c| self.expr(*c))
                .unwrap_or(Expr::Scalar(vec![], span)),
            "integer" => match self.integer(n) {
                Some(v) => Expr::Integer(v, span),
                None => Expr::Scalar(vec![], span),
            },
            "none" => Expr::Null(span),
            "string" => Expr::Scalar(
                children(n)
                    .into_iter()
                    .filter(|c| c.kind() == "interpolation")
                    .flat_map(children)
                    .filter(|c| !matches!(c.kind(), "format_specifier" | "type_conversion"))
                    .map(|c| self.expr(c))
                    .collect(),
                span,
            ),
            "float" | "true" | "false" | "concatenated_string" | "ellipsis" => {
                Expr::Scalar(vec![], span)
            }
            "await" => self.unknown(
                n,
                "Suspension at await is not modeled; awaited values are unknown.",
            ),
            _ => self.unknown(
                n,
                "Expression form is not modeled by object-identity analysis.",
            ),
        }
    }
}

use crate::model::{hash, Span};
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use tree_sitter::Node;

fn children(n: Node<'_>) -> Vec<Node<'_>> {
    n.named_children(&mut n.walk()).collect()
}
fn text<'a>(n: Node<'_>, source: &'a str) -> &'a str {
    &source[n.byte_range()]
}

#[derive(Clone)]
enum Data {
    Object(usize),
    Scalar,
}

#[derive(Default)]
struct Facts {
    scopes: Vec<BTreeMap<String, Span>>,
    objects: BTreeMap<usize, BTreeMap<String, Data>>,
    values: BTreeMap<String, Data>,
    events: Vec<Value>,
    unsupported: Vec<String>,
    function: String,
}
impl Facts {
    fn lookup(&self, name: &str) -> Option<Span> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
    fn go(&mut self, n: Node<'_>, source: &str) {
        let scoped = matches!(
            n.kind(),
            "source_file" | "block" | "function_declaration" | "if_statement" | "for_statement"
        );
        if scoped {
            self.scopes.push(BTreeMap::new());
        }
        let previous = self.function.clone();
        if n.kind() == "function_declaration" {
            self.function = n
                .child_by_field_name("name")
                .map(|n| text(n, source))
                .unwrap_or_default()
                .into();
        }
        if matches!(
            n.kind(),
            "short_var_declaration" | "assignment_statement" | "var_spec" | "parameter_declaration"
        ) {
            let left = n
                .child_by_field_name("left")
                .or_else(|| n.child_by_field_name("name"));
            if let Some(left) = left {
                let targets = if left.kind() == "expression_list" {
                    children(left)
                } else {
                    vec![left]
                };
                for target in targets {
                    if target.kind() != "identifier" || text(target, source) == "_" {
                        continue;
                    }
                    let name = text(target, source);
                    let outer = self.lookup(name);
                    let declares = n.kind() != "assignment_statement";
                    let current = self
                        .scopes
                        .last()
                        .and_then(|scope| scope.get(name))
                        .copied();
                    let binding = if declares && current.is_none() {
                        let binding = Span::of(target);
                        self.scopes.last_mut().unwrap().insert(name.into(), binding);
                        binding
                    } else {
                        outer.unwrap_or(Span::of(target))
                    };
                    let right = n.child_by_field_name("right");
                    let call = right.and_then(|r| {
                        if r.kind() == "call_expression" {
                            Some(r)
                        } else {
                            children(r)
                                .into_iter()
                                .find(|c| c.kind() == "call_expression")
                        }
                    });
                    let producer = call
                        .and_then(|c| c.child_by_field_name("function"))
                        .map(|c| text(c, source));
                    self.events.push(json!({"producer":producer,"kind":"assignment", "function":self.function,"name":name,"range":Span::of(target),"statement":Span::of(n),"binding":binding,"previous_binding":outer,"declares":declares && current.is_none()}));
                }
            }
        }
        if n.kind() == "return_statement" {
            let mut pending = children(n);
            while let Some(value) = pending.pop() {
                if value.kind() == "identifier" {
                    self.events.push(json!({"kind":"return_read","function":self.function,"name":text(value,source),"range":Span::of(value),"binding":self.lookup(text(value,source))}));
                }
                pending.extend(children(value));
            }
        }
        for child in children(n) {
            self.go(child, source);
        }
        self.function = previous;
        if scoped {
            self.scopes.pop();
        }
    }
    fn value(&mut self, n: Node<'_>, source: &str) -> Data {
        match n.kind() {
            "identifier" => self
                .values
                .get(text(n, source))
                .cloned()
                .unwrap_or(Data::Scalar),
            "member_expression" => {
                let Some(object) = n.child_by_field_name("object") else {
                    return Data::Scalar;
                };
                let Some(property) = n.child_by_field_name("property") else {
                    return Data::Scalar;
                };
                if let Data::Object(id) = self.value(object, source) {
                    self.objects
                        .get(&id)
                        .and_then(|fields| fields.get(text(property, source)))
                        .cloned()
                        .unwrap_or(Data::Scalar)
                } else {
                    Data::Scalar
                }
            }
            "object" => {
                let id = n.start_byte();
                let mut fields = BTreeMap::new();
                for child in children(n) {
                    match child.kind() {
                        "pair" => {
                            let key = child.child_by_field_name("key").unwrap();
                            let value = child.child_by_field_name("value").unwrap();
                            if key.kind() != "property_identifier" {
                                self.unsupported.push("Non-identifier object key".into());
                            } else {
                                fields.insert(text(key, source).into(), self.value(value, source));
                            }
                        }
                        "spread_element" => {
                            if let Some(value) = children(child).first() {
                                if let Data::Object(object) = self.value(*value, source) {
                                    fields.extend(self.objects[&object].clone());
                                } else {
                                    self.unsupported.push("Unknown spread source".into());
                                }
                            }
                        }
                        _ => self.unsupported.push(
                            "Object methods, getters and shorthand properties are unsupported"
                                .into(),
                        ),
                    }
                }
                self.objects.insert(id, fields);
                self.events
                    .push(json!({"kind":"allocation","object":id,"range":Span::of(n)}));
                Data::Object(id)
            }
            _ => Data::Scalar,
        }
    }
    fn access(&mut self, n: Node<'_>, source: &str, kind: &str) {
        if n.kind() == "member_expression" {
            if let (Some(base), Some(property)) = (
                n.child_by_field_name("object"),
                n.child_by_field_name("property"),
            ) {
                if let Data::Object(id) = self.value(base, source) {
                    self.events.push(json!({"kind":kind,"range":Span::of(n),"path":text(n,source),"object":id,"property":text(property,source)}));
                }
            }
        }
    }
    fn js(&mut self, n: Node<'_>, source: &str) {
        match n.kind() {
            "variable_declarator" => {
                if let (Some(name), Some(value)) = (
                    n.child_by_field_name("name"),
                    n.child_by_field_name("value"),
                ) {
                    if name.kind() != "identifier" || self.values.contains_key(text(name, source)) {
                        self.unsupported
                            .push("Destructuring or shadowed bindings".into());
                        return;
                    }
                    let range = Span::of(value);
                    let value = self.value(value, source);
                    if let Data::Object(id) = &value {
                        self.events.push(json!({"kind":"object_binding","name":text(name,source),"object":id,"range":range}));
                    }
                    self.values.insert(text(name, source).into(), value);
                }
                return;
            }
            "assignment_expression" => {
                if let (Some(left), Some(right)) = (
                    n.child_by_field_name("left"),
                    n.child_by_field_name("right"),
                ) {
                    self.access(left, source, "write");
                    self.js(right, source);
                    let value = self.value(right, source);
                    if left.kind() == "identifier" {
                        self.values.insert(text(left, source).into(), value);
                    } else if let (Some(base), Some(property)) = (
                        left.child_by_field_name("object"),
                        left.child_by_field_name("property"),
                    ) {
                        if let Data::Object(id) = self.value(base, source) {
                            self.objects
                                .get_mut(&id)
                                .unwrap()
                                .insert(text(property, source).into(), value);
                        }
                    } else {
                        self.unsupported.push("Dynamic assignment target".into());
                    }
                }
                return;
            }
            "function_declaration"
            | "arrow_function"
            | "function_expression"
            | "if_statement"
            | "for_statement"
            | "while_statement" => {
                self.unsupported
                    .push("Only straight-line JavaScript object flows are supported".into());
                return;
            }
            "subscript_expression" | "new_expression" | "await_expression" => self
                .unsupported
                .push("Dynamic objects, indexes and suspension are unsupported".into()),
            "call_expression"
                if n.child_by_field_name("function")
                    .is_none_or(|f| text(f, source) != "console.log")
                    || self.values.contains_key("console") =>
            {
                self.unsupported.push("Unknown call effects".into());
            }
            "member_expression" => self.access(n, source, "read"),
            _ => {}
        }
        for child in children(n) {
            self.js(child, source);
        }
    }
}

pub fn analyze(language: &str, source: &str) -> Result<Value> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&crate::analysis::grammar(language)?)?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("parse failed"))?;
    let mut facts = Facts::default();
    if tree.root_node().has_error() {
        facts.unsupported.push("Syntax errors".into());
    }
    match language {
        "go" => facts.go(tree.root_node(), source),
        "javascript" => facts.js(tree.root_node(), source),
        _ => bail!("semantic pilots support Go and JavaScript"),
    }
    facts.unsupported.sort();
    facts.unsupported.dedup();
    Ok(
        json!({"schema_version":1,"source_hash":hash(source),"language":language,"events":facts.events,"unsupported":facts.unsupported,"status":"candidate"}),
    )
}

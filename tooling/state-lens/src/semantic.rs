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

#[derive(Default)]
struct Facts {
    scopes: Vec<BTreeMap<String, Span>>,
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
        "javascript" => return projection(source),
        _ => bail!("semantic pilots support Go and JavaScript"),
    }
    facts.unsupported.sort();
    facts.unsupported.dedup();
    Ok(
        json!({"schema_version":1,"source_hash":hash(source),"language":language,"events":facts.events,"unsupported":facts.unsupported,"status":"candidate"}),
    )
}

const PROJECTION_URI: &str = "file:///semantic-projection.js";

fn projection(source: &str) -> Result<Value> {
    let mut engine = crate::analysis::Engine::new()?;
    engine.set(PROJECTION_URI.into(), "javascript", source.into(), None)?;
    let report = engine.analyze();
    let document = report
        .documents
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("projection unavailable"))?;
    let graph = document
        .state_graph
        .ok_or_else(|| anyhow::anyhow!("graph unavailable"))?;
    let heap = &graph.heap;
    let text = |range: Span| source[range.start..range.end].to_string();
    let mut events = vec![];
    for (op, range) in &heap.allocation_ranges {
        if heap.context_of(*op) == 0 {
            events.push((
                range.start,
                json!({"kind":"allocation","object":op,"range":range}),
            ));
        }
    }
    for binding in &heap.bindings {
        events.push((binding.range.start, json!({"kind":"object_binding","name":binding.name,"object":binding.object,"range":binding.range})));
    }
    for write in heap.writes.iter().filter(|w| w.context == 0) {
        let range = graph.operations[write.op].location.range;
        let property = match &write.slot {
            crate::heap::Slot::Field(name) => Some(name.clone()),
            _ => None,
        };
        events.push((range.start, json!({"kind":"write","range":range,"path":text(range),"object":write.object,"property":property})));
    }
    for read in heap.reads.iter().filter(|r| r.context == 0) {
        events.push((read.range.start, json!({"kind":"read","range":read.range,"path":text(read.range),"object":read.object,"property":read.property})));
    }
    events.sort_by_key(|(start, _)| *start);
    let mut unsupported: Vec<String> = heap
        .boundaries
        .iter()
        .map(|b| b.reason.to_string())
        .collect();
    if document.parse_errors {
        unsupported.push("Syntax errors".into());
    }
    unsupported.sort();
    unsupported.dedup();
    Ok(
        json!({"schema_version":2,"source_hash":hash(source),"language":"javascript","projection":"state_graph","events":events.into_iter().map(|(_, e)| e).collect::<Vec<_>>(),"unsupported":unsupported,"status":"candidate"}),
    )
}

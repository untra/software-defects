use crate::{model::*, shared::{Event, SharedState, SiteCounts}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tree_sitter::Node;

pub const DEFECT: &str = "threading-mutexes";
const LIMIT: &str = "Bounded source-level lock analysis. Dynamic dispatch, escaping aliases, external implementations and runtime scheduling are not proven.";
const MAX_ALIAS_DEPTH: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Assessment { Usage, Candidate, Misuse }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synchronization {
    pub api: String,
    pub lock: String,
    pub assessment: Assessment,
    pub acquisitions: Vec<Span>,
    pub releases: Vec<Span>,
    pub protected_regions: Vec<Span>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Api { Go, GoRw, Rust, Tokio, Python, Reentrant, Asyncio, Wrapper, Web }
impl Api {
    fn name(self) -> &'static str { match self {
        Self::Go => "sync.Mutex", Self::GoRw => "sync.RWMutex", Self::Rust => "std::sync::Mutex/RwLock",
        Self::Tokio => "tokio::sync::Mutex/RwLock", Self::Python => "threading.Lock", Self::Reentrant => "threading.RLock",
        Self::Asyncio => "asyncio.Lock", Self::Wrapper => "reviewed promise-tail lock", Self::Web => "Web Locks",
    }}
}
#[derive(Clone)]
struct Resource { key: String, api: Api, declaration: Span }
#[derive(Clone)]
struct Operation<'a> { node: Node<'a>, resource: Option<Resource>, owner: Option<Span>, release: bool, conditional: bool, method: String }
fn text<'a>(n: Node<'_>, s: &'a str) -> &'a str { &s[n.byte_range()] }
fn children(n: Node<'_>) -> Vec<Node<'_>> { let mut c = n.walk(); n.named_children(&mut c).collect() }
fn ancestor<'a>(mut n: Node<'a>, kinds: &[&str]) -> Option<Node<'a>> {
    while let Some(p) = n.parent() { if kinds.contains(&p.kind()) { return Some(p); } n = p; } None
}
fn owner(report: &DocumentReport, at: Span) -> Option<Span> {
    report.functions.iter().filter(|f| f.range.contains(at)).min_by_key(|f| f.range.len()).map(|f| f.range)
}
fn callee(n: Node<'_>) -> Option<Node<'_>> { n.child_by_field_name("function") }
fn receiver(n: Node<'_>) -> Option<Node<'_>> {
    let f = callee(n)?;
    f.child_by_field_name("object").or_else(|| f.child_by_field_name("value")).or_else(|| f.child_by_field_name("operand"))
}
fn method<'a>(n: Node<'_>, s: &'a str) -> &'a str {
    let Some(f) = callee(n) else { return ""; };
    text(f.child_by_field_name("field").or_else(|| f.child_by_field_name("property")).or_else(|| f.child_by_field_name("attribute")).unwrap_or(f), s)
}
fn declaration(n: Node<'_>) -> Option<Node<'_>> {
    ancestor(n, &["variable_declarator", "let_declaration", "assignment", "short_var_declaration", "var_spec"])
}
fn declared_name(n: Node<'_>) -> Option<Node<'_>> {
    n.child_by_field_name("name").or_else(|| n.child_by_field_name("pattern")).or_else(|| n.child_by_field_name("left"))
}
fn block(n: Node<'_>) -> Node<'_> { ancestor(n, &["block", "statement_block", "module", "source_file", "program"]).unwrap_or(n) }
fn same_path(acquire: Node<'_>, later: Node<'_>) -> bool { Span::of(block(acquire)).contains(Span::of(later)) }
fn direct_in(n: Node<'_>, scope: Node<'_>) -> bool { block(n) == scope }

struct Resolver<'a, F> {
    source: &'a str,
    all: &'a [Node<'a>],
    imports: &'a [(String, String, Option<String>)],
    binding: F,
}
impl<F: Fn(&str, Span) -> Option<Span>> Resolver<'_, F> {
    fn api(&self, value: &str, at: Span) -> Option<Api> {
        let mut canonical = value.trim().trim_start_matches('&').trim_start_matches("mut ").trim_start_matches('*').to_owned();
        for (local, module, symbol) in self.imports {
            if canonical == *local || canonical.starts_with(&format!("{local}.")) || canonical.starts_with(&format!("{local}::")) || canonical.starts_with(&format!("{local}<")) {
                if (self.binding)(local, at).is_some() { return None; }
                canonical = format!("{}{}{}", module, symbol.as_ref().map(|s| format!("::{s}")).unwrap_or_default(), &canonical[local.len()..]);
                break;
            }
        }
        let head = canonical.split(['.', ':', '<', '(']).next().unwrap_or("");
        if (self.binding)(head, at).is_some() { return None; }
        let c = canonical.replace("::", ".");
        if c.starts_with("sync.RWMutex") { Some(Api::GoRw) }
        else if c.starts_with("sync.Mutex") { Some(Api::Go) }
        else if c.starts_with("std.sync.Mutex") || c.starts_with("std.sync.RwLock") { Some(Api::Rust) }
        else if c.starts_with("tokio.sync.Mutex") || c.starts_with("tokio.sync.RwLock") { Some(Api::Tokio) }
        else if c.starts_with("threading.RLock") { Some(Api::Reentrant) }
        else if c.starts_with("threading.Lock") { Some(Api::Python) }
        else if c.starts_with("asyncio.Lock") { Some(Api::Asyncio) }
        else { None }
    }
    fn resource(&self, expression: &str, at: Span, depth: usize) -> Option<Resource> {
        if depth >= MAX_ALIAS_DEPTH { return None; }
        let value = expression.trim().trim_start_matches('&').trim_start_matches("mut ").trim_start_matches('*');
        let head = value.split(['.', '[', '(', ':']).next()?;
        if head == "self" && value.contains('.') {
            if let Some(implementation) = self.all.iter().find(|n| n.kind() == "impl_item" && Span::of(**n).contains(at)) {
                if let Some(ty) = implementation.child_by_field_name("type") {
                    let field = value.split('.').nth(1)?;
                    for structure in self.all.iter().filter(|n| n.kind() == "struct_item" && n.child_by_field_name("name").is_some_and(|name| text(name, self.source) == text(ty, self.source))) {
                        for f in crate::annotations::nodes(*structure) {
                            if f.kind() == "field_declaration" && f.child_by_field_name("name").is_some_and(|n| text(n, self.source) == field) {
                                if let Some(api) = f.child_by_field_name("type").and_then(|n| self.api(text(n, self.source), at)) {
                                    return Some(Resource { key: format!("self:{}.{field}", text(ty, self.source)), api, declaration: Span::of(f) });
                                }
                            }
                        }
                    }
                }
            }
        }
        let span = (self.binding)(head, at)?;
        let node = self.all.iter().find(|n| Span::of(**n) == span).copied()?;
        let decl = if matches!(node.kind(), "parameter_declaration" | "parameter" | "typed_parameter") { node } else {
            ancestor(node, &["variable_declarator", "let_declaration", "assignment", "short_var_declaration", "var_spec", "parameter_declaration", "parameter", "typed_parameter", "required_parameter"])?
        };
        let key = format!("{}:{}{}", span.start, span.end, &value[head.len()..]);
        if let Some(ty) = decl.child_by_field_name("type") {
            if let Some(api) = self.api(text(ty, self.source), at) { return Some(Resource { key, api, declaration: span }); }
        }
        let initial = decl.child_by_field_name("value").or_else(|| decl.child_by_field_name("right"));
        if let Some(initial) = initial {
            let init_nodes = crate::annotations::nodes(initial);
            for n in &init_nodes {
                if let Some(f) = callee(*n).or_else(|| n.child_by_field_name("constructor")) {
                    if let Some(api) = self.api(text(f, self.source), at) { return Some(Resource { key, api, declaration: span }); }
                    if n.kind() == "new_expression" && self.wrapper(text(f, self.source), at) { return Some(Resource { key, api: Api::Wrapper, declaration: span }); }
                }
            }
            let raw = text(initial, self.source);
            if raw != value {
                if let Some(mut resource) = self.resource(raw, Span::of(initial), depth + 1) {
                    resource.key.push_str(&value[head.len()..]);
                    return Some(resource);
                }
            }
            for n in &init_nodes {
                if matches!(method(*n, self.source), "Arc::clone" | "clone") {
                    let alias = receiver(*n).or_else(|| n.child_by_field_name("arguments").and_then(|a| a.named_child(0)));
                    if let Some(alias) = alias {
                        if let Some(r) = self.resource(text(alias, self.source), Span::of(alias), depth + 1) { return Some(r); }
                    }
                }
            }
        }
        if value.contains('.') {
            let field = value.split('.').nth(1)?;
            let ty = decl.child_by_field_name("type").map(|n| text(n, self.source).trim_start_matches('&').trim_start_matches('*').trim_start_matches("mut ").to_owned())
                .or_else(|| initial.and_then(|n| crate::annotations::nodes(n).iter().find_map(|n| n.child_by_field_name("type").map(|t| text(t, self.source).to_owned()))));
            if let Some(ty) = ty {
                for n in self.all {
                    if !matches!(n.kind(), "type_spec" | "struct_item") || n.child_by_field_name("name").is_none_or(|name| text(name, self.source) != ty) { continue; }
                    for f in crate::annotations::nodes(*n) {
                        if !matches!(f.kind(), "field_declaration" | "field_declaration_list") { continue; }
                        if f.child_by_field_name("name").is_some_and(|n| text(n, self.source) == field) {
                            if let Some(api) = f.child_by_field_name("type").and_then(|n| self.api(text(n, self.source), at)) { return Some(Resource { key, api, declaration: span }); }
                        }
                    }
                }
            }
        }
        None
    }
    fn wrapper(&self, name: &str, at: Span) -> bool {
        let Some(span) = (self.binding)(name, at) else { return false; };
        let Some(node) = self.all.iter().find(|n| Span::of(**n) == span) else { return false; };
        let Some(class) = ancestor(*node, &["class_declaration", "class"]) else { return false; };
        let nodes = crate::annotations::nodes(class);
        let Some(acquire) = nodes.iter().find(|n| n.kind() == "method_definition" && n.child_by_field_name("name").is_some_and(|n| text(n, self.source) == "acquire")) else { return false; };
        let body = text(*acquire, self.source).split_whitespace().collect::<String>();
        // This narrow model recognizes the corpus protocol, not arbitrary acquire methods.
        body.contains("constprevious=this.tail;") && (body.contains("this.tail=newPromise((resolve)=>(release=resolve))") || body.contains("this.tail=newPromise<void>((resolve)=>(release=resolve))")) && body.contains("awaitprevious;") && body.contains("returnrelease;")
    }
    fn web(&self, n: Node<'_>) -> Option<Resource> {
        if method(n, self.source) != "request" { return None; }
        let object = receiver(n)?;
        let name = text(object, self.source);
        let known = name == "navigator.locks" && (self.binding)("navigator", Span::of(n)).is_none()
            || self.imports.iter().any(|(local, module, symbol)| name == local && module == "node:worker_threads" && symbol.as_deref() == Some("locks") && (self.binding)(name, Span::of(n)).is_none());
        if !known { return None; }
        let mut arg = n.child_by_field_name("arguments")?.named_child(0)?;
        if arg.kind() == "identifier" {
            let span = (self.binding)(text(arg, self.source), Span::of(n))?;
            let node = self.all.iter().find(|n| Span::of(**n) == span)?;
            let decl = ancestor(*node, &["variable_declarator"])?;
            if !decl.parent().is_some_and(|p| text(p, self.source).starts_with("const ")) { return None; }
            arg = decl.child_by_field_name("value")?;
        }
        if !matches!(arg.kind(), "string" | "string_literal") { return None; }
        Some(Resource { key: format!("web:{}", text(arg, self.source).trim_matches(['\'', '"'])), api: Api::Web, declaration: Span::of(arg) })
    }
}

fn emit(report: &mut DocumentReport, op: &Operation<'_>, mechanism: &str, assessment: Assessment, events: Vec<(&str, Span)>, regions: Vec<Span>, releases: Vec<Span>, explanation: &str) {
    let resource = op.resource.as_ref();
    let rule_id = format!("{}.shared-state.{DEFECT}", report.language);
    let events: Vec<Event> = events.into_iter().map(|(kind, range)| Event { kind: kind.into(), range, resource: resource.map(|r| r.key.clone()), owner: owner(report, range) }).collect();
    let trigger = events.last().map(|e| e.range).unwrap_or(Span::of(op.node));
    let sync = resource.map(|r| Synchronization { api: r.api.name().into(), lock: r.key.clone(), assessment, acquisitions: vec![Span::of(op.node)], releases, protected_regions: if assessment == Assessment::Usage { regions.clone() } else { vec![] } });
    report.findings.push(Finding {
        shared_state: Some(SharedState {
            source_uri: report.uri.clone(), source_hash: report.source_hash.clone(), evidence_level: if assessment == Assessment::Usage { "syntax-candidate" } else { "relationship-candidate" }.into(),
            relationship: mechanism.into(), resource: resource.map(|r| r.key.clone()), events: events.clone(), sites: SiteCounts::default(), contenders: 0,
            assumptions: vec![LIMIT.into()], synchronization: sync,
        }),
        state_trace: None, division_risk: None, rule_id: rule_id.clone(), family: "shared-state".into(), compound: None,
        mechanism: Some(mechanism.into()), defect: DEFECT.into(), trigger, function: op.owner, blast_radius: regions,
        effect: if assessment == Assessment::Usage && resource.is_some() { Effect::Synchronization } else { Effect::StructuralSignal }, origin: "direct".into(), resolution: mechanism.into(), explanation: explanation.into(),
        evidence: events.iter().map(|e| Evidence { uri: report.uri.clone(), range: e.range, rule_id: rule_id.clone() }).collect(), call_path: vec![], related_cwes: crate::analysis::related_cwes(&rule_id),
    });
}
fn regions(report: &DocumentReport, span: Span, allowed: Option<Span>) -> Vec<Span> {
    let mut spans = vec![span];
    for function in report.functions.iter().filter(|f| span.contains(f.range) && Some(f.range) != allowed) {
        spans = spans.into_iter().flat_map(|s| if s.contains(function.range) { vec![Span { start: s.start, end: function.range.start }, Span { start: function.range.end, end: s.end }] } else { vec![s] }).filter(|s| !s.is_empty()).collect();
    }
    spans
}

pub(crate) fn analyze<'a>(captures: &[(&str, &str, Node<'a>)], root: Node<'a>, source: &'a str, report: &mut DocumentReport, imports: &[(String, String, Option<String>)], binding: impl Fn(&str, Span) -> Option<Span>) {
    let all = crate::annotations::nodes(root);
    let mut imports = imports.to_vec();
    fn rust_use(node: Node<'_>, prefix: &str, source: &str, imports: &mut Vec<(String, String, Option<String>)>) {
        if node.kind() == "scoped_use_list" {
            let path = node.child_by_field_name("path").map(|n| text(n, source)).unwrap_or("");
            let prefix = if prefix.is_empty() { path.to_owned() } else { format!("{prefix}::{path}") };
            if let Some(list) = node.child_by_field_name("list") { for child in children(list) { rust_use(child, &prefix, source, imports); } }
        } else if node.kind() == "use_list" {
            for child in children(node) { rust_use(child, prefix, source, imports); }
        } else if !text(node, source).contains('*') {
            let raw = text(node, source);
            let mut alias = raw.split(" as ");
            let path = alias.next().unwrap();
            let full = if prefix.is_empty() { path.to_owned() } else { format!("{prefix}::{path}") };
            if let Some((module, symbol)) = full.rsplit_once("::") { imports.push((alias.next().unwrap_or(symbol).into(), module.into(), Some(symbol.into()))); }
        }
    }
    if report.language == "rust" {
        for node in all.iter().filter(|n| n.kind() == "use_declaration") {
            if let Some(argument) = node.child_by_field_name("argument") { rust_use(argument, "", source, &mut imports); }
        }
    }
    let resolver = Resolver { source, all: &all, imports: &imports, binding };
    let mut operations = Vec::new();
    for (path, capture, n) in captures {
        if !path.contains(DEFECT) || *capture != "mutex_call" { continue; }
        let method = method(*n, source);
        if matches!(method, "Lock" | "RLock") && report.language == "python" { continue; }
        let resource = resolver.web(*n).or_else(|| receiver(*n).and_then(|r| resolver.resource(text(r, source), Span::of(*n), 0)));
        operations.push(Operation { node: *n, resource, owner: owner(report, Span::of(*n)), release: matches!(method, "Unlock" | "RUnlock" | "release"), conditional: method.starts_with("try_") || method.starts_with("Try") || report.language == "python" && text(*n, source).contains("timeout="), method: method.into() });
    }
    for n in &all {
        if n.kind() == "with_statement" {
            for item in crate::annotations::nodes(*n).into_iter().filter(|i| i.kind() == "with_item") {
                if let Some(value) = item.child_by_field_name("value").or_else(|| item.named_child(0)) {
                    if let Some(resource) = resolver.resource(text(value, source), Span::of(value), 0) {
                        operations.push(Operation { node: value, resource: Some(resource), owner: owner(report, Span::of(value)), release: false, conditional: false, method: "context".into() });
                    }
                }
            }
        }
    }
    operations.sort_by_key(|o| o.node.start_byte());
    operations.dedup_by_key(|o| o.node.id());
    let mut holds: Vec<(Operation<'a>, Span)> = Vec::new();
    for op in operations.iter().filter(|o| !o.release) {
        let at = Span::of(op.node);
        let Some(resource) = &op.resource else {
            if !matches!(op.method.as_str(), "read" | "write" | "request") {
                emit(report, op, "lock-operation", Assessment::Candidate, vec![("unresolved-lock-operation", at)], vec![at], vec![], "Lock-shaped operation; receiver type and synchronization contract are unresolved.");
            }
            continue;
        };
        let scope = block(op.node);
        let context = if op.method == "context" { ancestor(op.node, &["with_statement"]) } else { None };
        let web_body = if resource.api == Api::Web { op.node.child_by_field_name("arguments").and_then(|args| children(args).into_iter().find(|n| matches!(n.kind(), "arrow_function" | "function_expression"))) } else { None };
        let raii = matches!(resource.api, Api::Rust | Api::Tokio);
        let assigned = declaration(op.node).and_then(declared_name);
        let release_callback = if resource.api == Api::Wrapper { assigned.map(|n| text(n, source).to_owned()) } else { None };
        let mut releases: Vec<Node<'a>> = operations.iter().filter(|r| r.release && r.owner == op.owner && r.node.start_byte() > at.start && r.resource.as_ref().is_some_and(|r| r.key == resource.key)).map(|r| r.node).collect();
        for n in &all {
            if n.start_byte() <= at.end || owner(report, Span::of(*n)) != op.owner { continue; }
            if release_callback.as_ref().is_some_and(|name| callee(*n).is_some_and(|f| text(f, source) == name) && (resolver.binding)(name, Span::of(*n)) == assigned.map(Span::of)) { releases.push(*n); }
            if raii && method(*n, source) == "drop" && assigned.is_some_and(|a| n.child_by_field_name("arguments").is_some_and(|args| children(args).iter().any(|arg| text(*arg, source) == text(a, source)))) { releases.push(*n); }
        }
        releases.sort_by_key(Node::start_byte);
        let cleanup = releases.iter().copied().find(|r| {
            ancestor(*r, &["defer_statement"]).is_some_and(|d| direct_in(d, scope)) || ancestor(*r, &["finally_clause"]).is_some_and(|f| f.parent().is_some_and(|t| t.start_byte() >= at.end && direct_in(t, scope) || Span::of(t).contains(at)))
        });
        let explicit = releases.iter().copied().find(|r| direct_in(*r, scope) && ancestor(*r, &["defer_statement"]).is_none());
        let temporary_end = if raii {
            let declaration = declaration(op.node);
            let guard = declaration.and_then(|d| d.child_by_field_name("value")).is_some_and(|value| {
                let v = text(value, source);
                !v.starts_with('*') && (v.ends_with(".unwrap()") || v.ends_with(".await") || v.ends_with('?'))
            });
            if guard { None } else {
                ancestor(op.node, &["match_expression"]).map(|n| n.end_byte()).or_else(|| ancestor(op.node, &["expression_statement", "let_declaration"]).map(|n| n.end_byte()))
            }
        } else { None };
        let end = context.map(|n| n.end_byte()).or_else(|| web_body.map(|n| n.end_byte())).or_else(|| explicit.map(|n| n.end_byte())).or(temporary_end).unwrap_or(scope.end_byte());
        let span = Span { start: at.start, end };
        let protected = if op.conditional { vec![at] } else { regions(report, span, web_body.map(Span::of)) };
        let mut events = vec![("acquire", at)];
        events.extend(releases.iter().filter(|r| r.start_byte() <= end).map(|r| ("release", Span::of(*r))));
        if context.is_some() || raii || web_body.is_some() { events.push(("scope-release", Span { start: end, end })); }
        emit(report, op, "critical-section", Assessment::Usage, events, protected, releases.iter().map(|r| Span::of(*r)).collect(), if op.conditional { "Conditional lock acquisition; protection applies only on the successful branch." } else { "Recognized critical section. Synchronization remains visible even when used correctly." });
        if op.conditional {
            if declaration(op.node).is_none() && ancestor(op.node, &["if_statement", "if_expression"]).is_none() {
                emit(report, op, "unchecked-acquisition", Assessment::Candidate, vec![("unchecked-acquire", at)], vec![at], vec![], "Conditional acquisition result is not checked; subsequent accesses may run without the lock.");
            }
        } else { holds.push((op.clone(), span)); }
        if !raii && context.is_none() && web_body.is_none() && cleanup.is_none() && !op.conditional {
            let exit = all.iter().find(|n| matches!(n.kind(), "return_statement" | "throw_statement" | "raise_statement") && n.start_byte() > at.end && n.end_byte() < end && owner(report, Span::of(**n)) == op.owner && same_path(op.node, **n));
            if let Some(exit) = exit {
                let certain = direct_in(*exit, scope) && explicit.is_some();
                emit(report, op, "missing-release", if certain { Assessment::Misuse } else { Assessment::Candidate }, vec![("acquire", at), ("exit-before-release", Span::of(*exit))], vec![span], releases.iter().map(|r| Span::of(*r)).collect(), "An exit can bypass the lock release. Use deferred, context-managed or finally cleanup.");
            } else if releases.is_empty() {
                emit(report, op, "missing-release", Assessment::Candidate, vec![("acquire", at)], vec![span], vec![], "No release is resolved in this scope; the release may be missing or ownership may escape.");
            }
        }
        if resource.api == Api::Asyncio && context.is_none() && cleanup.is_none() {
            if let Some(suspend) = all.iter().find(|n| matches!(n.kind(), "await" | "await_expression") && n.start_byte() > at.end && n.end_byte() < end && owner(report, Span::of(**n)) == op.owner) {
                emit(report, op, "missing-release", Assessment::Candidate, vec![("acquire", at), ("cancellation-exit", Span::of(*suspend))], vec![span], vec![], "Cancellation at this suspension can skip release; use async with or finally cleanup.");
            }
        }
        if report.language == "go" {
            if let Some(method) = ancestor(op.node, &["method_declaration"]) {
                if let Some(receiver) = method.child_by_field_name("receiver").and_then(|r| r.named_child(0)) {
                    if receiver.child_by_field_name("type").is_some_and(|t| t.kind() != "pointer_type") && receiver.child_by_field_name("name").is_some_and(|n| resource.key.starts_with(&format!("{}:{}.", n.start_byte(), n.end_byte()))) {
                        emit(report, op, "copied-lock", Assessment::Candidate, vec![("value-receiver", Span::of(receiver)), ("acquire", at)], vec![Span::of(receiver), at], vec![], "A value receiver copies a lock-bearing struct. After first use copying is forbidden, and reference-valued fields may remain shared behind independent locks.");
                    }
                }
            }
        }
        for suspend in all.iter().filter(|n| matches!(n.kind(), "await_expression" | "await") && n.start_byte() > at.end && n.end_byte() < end && owner(report, Span::of(**n)) == op.owner).copied().collect::<Vec<_>>() {
            if matches!(resource.api, Api::Rust | Api::Python | Api::Reentrant) {
                emit(report, op, "blocking-across-suspension", Assessment::Candidate, vec![("acquire", at), ("suspension", Span::of(suspend))], vec![span], vec![], "A thread-owned or blocking lock spans suspension. Review executor blocking and task ownership; asyncio/Tokio locks have different semantics.");
            }
        }
        if let Some(callback) = web_body {
            for call in crate::annotations::nodes(callback).into_iter().filter(|n| n.kind() == "call_expression" && n.parent().is_some_and(|p| p.kind() == "expression_statement") && owner(report, Span::of(*n)) == Some(Span::of(callback))).collect::<Vec<_>>() {
                emit(report, op, "callback-lifetime", Assessment::Candidate, vec![("acquire", at), ("unobserved-callback-work", Span::of(call))], vec![Span::of(call)], vec![], "If this call starts asynchronous work, the lock callback does not await or return its completion; the lock may be released before the work finishes.");
            }
        }
        let local = op.owner.is_some_and(|o| o.contains(resource.declaration)) && all.iter().find(|n| Span::of(**n) == resource.declaration).is_some_and(|n| ancestor(*n, &["parameter", "parameter_declaration", "required_parameter", "typed_parameter"]).is_none());
        if local {
            let writes: Vec<Span> = report.findings.iter().filter(|f| f.function == op.owner && matches!(f.effect, Effect::GlobalMutation | Effect::ArgumentMutation(_) | Effect::ReceiverMutation) && span.contains(f.trigger)).map(|f| f.trigger).collect();
            for write in writes {
                emit(report, op, "per-call-lock", Assessment::Candidate, vec![("local-lock", resource.declaration), ("acquire", at), ("external-write", write)], vec![span], vec![], "A locally created lock surrounds externally reachable mutation. Independent invocations may use different locks for the same resource.");
            }
        }
    }
    for (first, span) in &holds {
        let resource = first.resource.as_ref().unwrap();
        for second in operations.iter().filter(|o| !o.release && o.node.start_byte() > first.node.end_byte() && o.node.end_byte() <= span.end && (o.owner == first.owner || resource.api == Api::Web && span.contains(Span::of(o.node))) && same_path(first.node, o.node)) {
            if second.resource.as_ref().is_some_and(|r| r.key == resource.key) && resource.api != Api::Reentrant && !second.conditional {
                if first.method == "RLock" && second.method == "RLock" { continue; }
                let certain = direct_in(first.node, block(second.node)) && !second.conditional && !matches!(first.method.as_str(), "read" | "RLock") && !operations.iter().any(|o| o.release && o.node.start_byte() > first.node.end_byte() && o.node.end_byte() < second.node.start_byte() && o.resource.as_ref().is_some_and(|r| r.key == resource.key));
                emit(report, first, "non-reentrant-acquisition", if certain { Assessment::Misuse } else { Assessment::Candidate }, vec![("acquire", Span::of(first.node)), ("reacquire", Span::of(second.node))], vec![*span], vec![], "The same non-reentrant lock is acquired again before release. This may deadlock, fail, or panic according to the API.");
            }
        }
    }
    for (held, span) in &holds {
        let resource = held.resource.as_ref().unwrap();
        if resource.api == Api::Reentrant { continue; }
        for call in all.iter().filter(|n| matches!(n.kind(), "call" | "call_expression") && span.contains(Span::of(**n)) && n.start_byte() > held.node.end_byte() && owner(report, Span::of(**n)) == held.owner) {
            let name = method(*call, source);
            let targets: Vec<_> = report.functions.iter().filter(|f| f.name == name).map(|f| f.range).collect();
            if targets.len() != 1 { continue; }
            let target = targets[0];
            for acquisition in operations.iter().filter(|o| !o.release && o.owner == Some(target)) {
                let Some(acquired) = &acquisition.resource else { continue; };
                let mapped = if let Some(object) = receiver(*call) {
                    if let Some((_, suffix)) = acquired.key.split_once('.') {
                        resolver.resource(&format!("{}.{}", text(object, source), suffix), Span::of(*call), 0).map(|r| r.key)
                    } else { None }
                } else { Some(acquired.key.clone()) };
                if mapped.as_deref() == Some(resource.key.as_str()) {
                    emit(report, held, "non-reentrant-acquisition", Assessment::Candidate, vec![("acquire", Span::of(held.node)), ("nested-call", Span::of(*call)), ("callee-acquire", Span::of(acquisition.node))], vec![Span::of(held.node), Span::of(*call)], vec![], "A resolved local call acquires the lock already held by its caller. Recursive reader locking can also block behind a pending writer.");
                }
            }
        }
    }
    let mut order: BTreeMap<(String, String), (&Operation<'_>, &Operation<'_>)> = BTreeMap::new();
    for (first, span) in &holds {
        for second in operations.iter().filter(|o| !o.release && o.owner == first.owner && o.node.start_byte() > first.node.end_byte() && span.contains(Span::of(o.node))) {
            let (Some(a), Some(b)) = (&first.resource, &second.resource) else { continue; };
            if a.key == b.key { continue; }
            if let Some((other_first, other_second)) = order.get(&(b.key.clone(), a.key.clone())) {
                emit(report, second, "lock-order-cycle", Assessment::Candidate, vec![("first-order-acquire", Span::of(other_first.node)), ("first-order-nested", Span::of(other_second.node)), ("opposing-acquire", Span::of(first.node)), ("opposing-nested", Span::of(second.node))], vec![Span::of(other_first.node), Span::of(other_second.node), Span::of(first.node), Span::of(second.node)], vec![], "Opposing lock acquisition orders can deadlock if these paths execute concurrently without a common outer lock.");
            }
            order.insert((a.key.clone(), b.key.clone()), (first, second));
        }
    }
}

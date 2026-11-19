use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

const ASYNC: &str = "async-io-misuse";
const RACE: &str = "race-conditions";
const IO: &str = "file-network-access";
const LIMIT: &str = "Bounded lexical relationships; dynamic dispatch, escaping aliases, branch completeness and runtime scheduling are not proven.";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SiteCounts {
    pub declarations: usize,
    pub launches: usize,
    pub suspensions: usize,
    pub dynamic_fan_out: bool,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub kind: String,
    pub range: Span,
    pub resource: Option<String>,
    pub owner: Option<Span>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SharedState {
    pub source_uri: String,
    pub source_hash: String,
    pub evidence_level: String,
    pub relationship: String,
    pub resource: Option<String>,
    pub events: Vec<Event>,
    pub sites: SiteCounts,
    pub contenders: usize,
    pub assumptions: Vec<String>,
}
fn text<'a>(n: Node<'_>, s: &'a str) -> &'a str {
    &s[n.byte_range()]
}
fn owner(report: &DocumentReport, at: Span) -> Option<Span> {
    report
        .functions
        .iter()
        .filter(|f| f.range.contains(at))
        .min_by_key(|f| f.range.len())
        .map(|f| f.range)
}
fn base(s: &str) -> &str {
    s.trim()
        .trim_start_matches('&')
        .trim_start_matches("mut ")
        .trim_start_matches('*')
        .split(['.', '[', '(', ':'])
        .next()
        .unwrap_or("")
        .trim()
}
fn assigned(n: Node<'_>) -> Option<Node<'_>> {
    let mut current = n;
    while let Some(p) = current.parent() {
        if matches!(
            p.kind(),
            "variable_declarator"
                | "let_declaration"
                | "assignment"
                | "short_var_declaration"
                | "with_item"
        ) {
            return p
                .child_by_field_name("name")
                .or_else(|| p.child_by_field_name("pattern"))
                .or_else(|| p.child_by_field_name("left"))
                .or_else(|| p.child_by_field_name("alias"));
        }
        if matches!(
            p.kind(),
            "expression_statement" | "block" | "statement_block" | "return_statement"
        ) {
            break;
        }
        if !matches!(
            p.kind(),
            "await_expression"
                | "await"
                | "try_expression"
                | "parenthesized_expression"
                | "expression_list"
        ) {
            break;
        }
        current = p;
    }
    None
}
fn in_loop(mut n: Node<'_>) -> bool {
    while let Some(p) = n.parent() {
        if matches!(
            p.kind(),
            "for_statement"
                | "for_expression"
                | "while_statement"
                | "while_expression"
                | "for_in_statement"
                | "list_comprehension"
        ) {
            return true;
        }
        if matches!(
            p.kind(),
            "function_definition"
                | "function_declaration"
                | "function_item"
                | "func_literal"
                | "arrow_function"
        ) {
            break;
        }
        n = p;
    }
    false
}
fn emit(
    report: &mut DocumentReport,
    defect: &str,
    relationship: &str,
    events: Vec<Event>,
    candidate: bool,
    sites: SiteCounts,
    explanation: &str,
) {
    if events.is_empty() {
        return;
    }
    let trigger = events.last().unwrap().range;
    let resource = events.iter().find_map(|e| e.resource.clone());
    let contenders = events
        .iter()
        .filter_map(|e| e.owner)
        .collect::<BTreeSet<_>>()
        .len();
    let rule_id = format!("{}.shared-state.{defect}", report.language);
    let start = events.iter().map(|e| e.range.start).min().unwrap();
    let end = events.iter().map(|e| e.range.end).max().unwrap();
    let mut regions = vec![Span { start, end }];
    for function in report
        .functions
        .iter()
        .filter(|f| f.range.start > start && f.range.end < end)
    {
        regions = regions
            .into_iter()
            .flat_map(|r| {
                if r.contains(function.range) {
                    vec![
                        Span {
                            start: r.start,
                            end: function.range.start,
                        },
                        Span {
                            start: function.range.end,
                            end: r.end,
                        },
                    ]
                } else {
                    vec![r]
                }
            })
            .filter(|r| !r.is_empty())
            .collect();
    }
    report.findings.push(Finding {
        shared_state: Some(SharedState {
            source_uri: report.uri.clone(),
            source_hash: report.source_hash.clone(),
            evidence_level: if candidate {
                "relationship-candidate"
            } else {
                "syntax-candidate"
            }
            .into(),
            relationship: relationship.into(),
            resource,
            events: events.clone(),
            sites,
            contenders,
            assumptions: vec![LIMIT.into()],
        }),
        state_trace: None,
        division_risk: None,
        rule_id: rule_id.clone(),
        family: "shared-state".into(),
        compound: None,
        mechanism: Some(relationship.into()),
        defect: defect.into(),
        trigger,
        function: owner(report, trigger),
        blast_radius: regions,
        effect: if relationship == "async-execution" {
            Effect::AsyncExecution
        } else if relationship == "resource-operation" {
            Effect::ResourceAccess
        } else {
            Effect::StructuralSignal
        },
        origin: "direct".into(),
        resolution: relationship.into(),
        explanation: explanation.into(),
        evidence: events
            .iter()
            .map(|e| Evidence {
                uri: report.uri.clone(),
                range: e.range,
                rule_id: rule_id.clone(),
            })
            .collect(),
        call_path: vec![],
        related_cwes: crate::analysis::related_cwes(&rule_id),
    });
}

pub(crate) fn analyze<'a>(
    captures: &[(&str, &str, Node<'a>)],
    root: Node<'a>,
    source: &str,
    report: &mut DocumentReport,
    imports: &[(String, String, Option<String>)],
    binding: impl Fn(&str, Span) -> Option<Span>,
) {
    let all = crate::annotations::nodes(root);
    let identity = |expression: &str, at: Span| {
        let expression = expression
            .trim()
            .trim_start_matches('&')
            .trim_start_matches("mut ")
            .trim_start_matches('*');
        let name = base(expression);
        binding(name, at).map(|b| format!("{}:{}{}", b.start, b.end, &expression[name.len()..]))
    };
    let api = |callee: Node<'_>| {
        let name = text(callee, source);
        let head = base(name);
        if binding(head, Span::of(callee)).is_some() {
            return None;
        }
        if all.iter().any(|n| {
            matches!(
                n.kind(),
                "function_declaration"
                    | "function_definition"
                    | "function_item"
                    | "mod_item"
                    | "class_declaration"
                    | "class_definition"
            ) && n
                .child_by_field_name("name")
                .is_some_and(|v| text(v, source) == head)
                && owner(report, Span::of(n.parent().unwrap_or(*n)))
                    .is_none_or(|scope| scope.contains(Span::of(callee)))
        }) {
            return None;
        }
        if let Some((local, module, symbol)) = imports.iter().find(|(local, _, _)| local == head) {
            return Some(
                format!(
                    "{module}{}{}",
                    symbol.as_ref().map(|s| format!(".{s}")).unwrap_or_default(),
                    &name[local.len()..]
                )
                .replace("::", "."),
            );
        }
        if report.language == "python" {
            for import in all
                .iter()
                .filter(|n| matches!(n.kind(), "import_statement" | "import_from_statement"))
            {
                if import.start_byte() > callee.start_byte()
                    || owner(report, Span::of(*import))
                        .is_some_and(|scope| !scope.contains(Span::of(callee)))
                {
                    continue;
                }
                let module = import
                    .child_by_field_name("module_name")
                    .map(|m| text(m, source));
                let mut cursor = import.walk();
                for item in import.children_by_field_name("name", &mut cursor) {
                    let imported = item.child_by_field_name("name").unwrap_or(item);
                    let local = item.child_by_field_name("alias").unwrap_or(imported);
                    if text(local, source) == head {
                        return Some(format!(
                            "{}{}{}",
                            module.map(|m| format!("{m}.")).unwrap_or_default(),
                            text(imported, source),
                            &name[head.len()..]
                        ));
                    }
                }
            }
        }
        let builtin = match report.language.as_str() {
            "javascript" | "typescript" => matches!(head, "Promise" | "fetch" | "AbortSignal"),
            "python" => head == "open",
            "rust" => matches!(head, "std" | "tokio"),
            _ => false,
        };
        builtin.then(|| name.replace("::", "."))
    };
    let event = |kind: &str, n: Node<'_>, resource: Option<String>| Event {
        kind: kind.into(),
        range: Span::of(n),
        resource,
        owner: owner(report, Span::of(n)),
    };
    let mut events = vec![];
    let mut resources = BTreeSet::new();
    let mut tasks = BTreeSet::new();
    let mut async_resources = BTreeSet::new();
    let mut pending_io = BTreeMap::new();
    let mut aliases: BTreeMap<String, String> = BTreeMap::new();
    let mut counts: BTreeMap<Option<Span>, SiteCounts> = BTreeMap::new();
    let mut calls = vec![];
    let mut actors: BTreeMap<Span, Vec<Event>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for (_, capture, n) in captures {
        if !seen.insert((capture.to_string(), Span::of(*n))) {
            continue;
        }
        match *capture {
            "async_keyword" | "launch_keyword" | "suspension" => {
                let kind = match *capture {
                    "async_keyword" => "declaration",
                    "launch_keyword" => "launch",
                    _ => "suspend",
                };
                let e = event(kind, *n, None);
                let c = counts.entry(e.owner).or_default();
                match kind {
                    "declaration" => c.declarations += 1,
                    "launch" => {
                        c.launches += 1;
                        c.dynamic_fan_out |= in_loop(*n);
                    }
                    _ => c.suspensions += 1,
                }
                if kind == "launch" {
                    if let Some(statement) = n.parent() {
                        for function in &report.functions {
                            if Span::of(statement).contains(function.range) {
                                actors.entry(function.range).or_default().push(e.clone());
                            }
                        }
                    }
                }
                events.push(e);
            }
            "task_call" => calls.push(*n),
            _ => {}
        }
    }
    calls.sort_by_key(|n| n.start_byte());
    let async_names: BTreeMap<_, _> = all
        .iter()
        .filter_map(|n| {
            let name = n.child_by_field_name("name")?;
            let body = n.child_by_field_name("body")?;
            text(*n, source)
                .get(..body.start_byte() - n.start_byte())?
                .contains("async")
                .then(|| (text(name, source), Span::of(*n)))
        })
        .collect();
    for n in &calls {
        for declaration in all.iter().filter(|d| d.end_byte() < n.start_byte()) {
            let Some(value) = declaration
                .child_by_field_name("value")
                .or_else(|| declaration.child_by_field_name("right"))
            else {
                continue;
            };
            let Some(target) = declaration
                .child_by_field_name("name")
                .or_else(|| declaration.child_by_field_name("pattern"))
                .or_else(|| declaration.child_by_field_name("left"))
            else {
                continue;
            };
            if value.kind() != "identifier" || target.kind() != "identifier" {
                continue;
            }
            let (Some(from), Some(to)) = (
                identity(text(value, source), Span::of(value)),
                identity(text(target, source), Span::of(target)),
            ) else {
                continue;
            };
            let rebound = all.iter().any(|write| {
                write.start_byte() > declaration.end_byte()
                    && matches!(
                        write.kind(),
                        "assignment_expression" | "assignment_statement" | "assignment"
                    )
                    && write.child_by_field_name("left").is_some_and(|left| {
                        identity(text(left, source), Span::of(left)).as_ref() == Some(&to)
                    })
            });
            if rebound {
                continue;
            }
            let canonical = aliases.get(&from).unwrap_or(&from).clone();
            if resources.contains(&canonical) || tasks.contains(&canonical) {
                aliases.insert(to, canonical);
            }
        }
        let Some(callee) = n.child_by_field_name("function") else {
            continue;
        };
        let resolved = api(callee);
        let name = text(callee, source);
        let leaf = name.rsplit(['.', ':']).next().unwrap_or(name);
        let args = n.child_by_field_name("arguments");
        let argument = args.and_then(|a| a.named_child(0));
        let argument_id = argument
            .and_then(|a| identity(text(a, source), Span::of(a)))
            .or_else(|| {
                argument
                    .filter(|a| matches!(a.kind(), "string" | "string_literal"))
                    .map(|a| format!("literal:{}", text(a, source)))
            });
        let target_id = assigned(*n).and_then(|a| identity(base(text(a, source)), Span::of(a)));
        let receiver_id = identity(
            name.rsplit_once('.').map(|(r, _)| r).unwrap_or(name),
            Span::of(callee),
        );
        let receiver_id = receiver_id.map(|id| aliases.get(&id).unwrap_or(&id).clone());
        let known = resolved.as_deref().unwrap_or("");
        let launch = matches!(
            known,
            "asyncio.create_task"
                | "asyncio.ensure_future"
                | "asyncio.gather"
                | "asyncio.TaskGroup.create_task"
                | "std.thread.spawn"
                | "tokio.spawn"
                | "tokio.task.spawn"
                | "Promise.all"
                | "Promise.allSettled"
                | "Promise.race"
                | "Promise.any"
        );
        let check = matches!(
            known,
            "os.Stat"
                | "os.Lstat"
                | "os.path.exists"
                | "os.access"
                | "fs.existsSync"
                | "node:fs.existsSync"
                | "fs.access"
                | "node:fs.access"
                | "std.fs.metadata"
        );
        let acquire = matches!(
            known,
            "open"
                | "os.Open"
                | "os.OpenFile"
                | "os.Create"
                | "fs.open"
                | "node:fs.open"
                | "fs.openSync"
                | "node:fs.openSync"
                | "node:fs/promises.open"
                | "std.fs.File.open"
                | "std.fs.File.create"
                | "std.net.TcpStream.connect"
                | "tokio.fs.File.open"
                | "tokio.fs.File.create"
        );
        let io = check
            || acquire
            || matches!(
                known,
                "fs.writeFile"
                    | "node:fs.writeFile"
                    | "node:fs/promises.writeFile"
                    | "fs.writeFileSync"
                    | "node:fs.writeFileSync"
                    | "os.WriteFile"
                    | "std.fs.write"
                    | "tokio.fs.write"
                    | "fetch"
                    | "net/http.Get"
                    | "net/http.Post"
                    | "requests.get"
                    | "requests.post"
            );
        if resolved.is_none()
            && receiver_id.is_some()
            && matches!(
                leaf,
                "lock"
                    | "Lock"
                    | "Unlock"
                    | "load"
                    | "store"
                    | "fetch_add"
                    | "fetch_sub"
                    | "acquire"
                    | "release"
                    | "exists"
                    | "write_text"
                    | "open"
                    | "create_new"
            )
        {
            let kind = match leaf {
                "load" => "atomic-read",
                "store" => "atomic-write",
                "lock" | "Lock" | "acquire" => "lock",
                "Unlock" | "release" => "unlock",
                "exists" => "check",
                "write_text" | "open" => "io",
                _ => "synchronization",
            };
            events.push(event(kind, *n, receiver_id.clone()));
        }
        if launch {
            let e = event("launch", *n, target_id.clone());
            let c = counts.entry(e.owner).or_default();
            c.launches += 1;
            c.dynamic_fan_out |=
                in_loop(*n) || matches!(leaf, "all" | "gather" | "race" | "allSettled" | "any");
            if let Some(id) = &target_id {
                tasks.insert(id.clone());
            }
            for function in &report.functions {
                if Span::of(*n).contains(function.range) {
                    actors.entry(function.range).or_default().push(e.clone());
                }
            }
            events.push(e);
        }
        if io {
            if acquire {
                if let Some(id) = &target_id {
                    resources.insert(id.clone());
                    if known.contains("fs/promises") || known.starts_with("tokio.") {
                        async_resources.insert(id.clone());
                    }
                }
            }
            events.push(event(
                if check {
                    "check"
                } else if acquire {
                    "acquire"
                } else {
                    "io"
                },
                *n,
                if acquire {
                    target_id.clone().or(argument_id.clone())
                } else {
                    argument_id.clone()
                },
            ));
            if acquire && argument_id.is_some() {
                events.push(event("path-use", *n, argument_id));
            }
        }
        if let Some(id) = &receiver_id {
            let kind = if resources.contains(id) {
                match leaf {
                    "close" | "Close" => Some("release"),
                    "read" | "write" | "read_all" | "write_all" | "Read" | "Write" | "readFile"
                    | "writeFile" => Some("use"),
                    _ => None,
                }
            } else if tasks.contains(id) {
                match leaf {
                    "abort" | "cancel" => Some("cancel"),
                    "join" | "result" => Some("join"),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(kind) = kind {
                let deferred = n.parent().is_some_and(|p| p.kind() == "defer_statement");
                if !deferred {
                    let pending = kind == "use"
                        && async_resources.contains(id)
                        && !n
                            .parent()
                            .is_some_and(|p| matches!(p.kind(), "await_expression" | "await"));
                    events.push(event(
                        if pending { "pending-use" } else { kind },
                        *n,
                        Some(id.clone()),
                    ));
                    if pending {
                        if let Some(target) = &target_id {
                            pending_io.insert(target.clone(), id.clone());
                        }
                    }
                }
            }
        }
        if let Some(id) = argument
            .as_ref()
            .and_then(|a| identity(text(*a, source), Span::of(*a)))
        {
            if resources.contains(&id)
                && matches!(known, "fs.closeSync" | "node:fs.closeSync" | "std.mem.drop")
            {
                events.push(event("release", *n, Some(id)));
            }
        }
        if async_names
            .get(name)
            .is_some_and(|declaration| declaration.start != n.start_byte())
            && binding(name, Span::of(callee)).is_none_or(|bound| {
                async_names
                    .get(name)
                    .is_some_and(|decl| decl.contains(bound))
            })
        {
            let discarded = n.parent().is_some_and(|p| {
                p.kind() == "expression_statement"
                    || p.kind() == "let_declaration"
                        && text(p, source).trim_start().starts_with("let _ =")
            });
            if discarded {
                events.push(event("unobserved", *n, None));
            }
        }
    }
    // Awaiting a task identifier observes completion; awaiting unrelated work does not.
    for (_, capture, n) in captures.iter().filter(|(_, c, _)| *c == "suspension") {
        let _ = capture;
        if let Some(value) = n.named_child(0) {
            if let Some(id) = identity(text(value, source), Span::of(value)) {
                let id = aliases.get(&id).unwrap_or(&id).clone();
                if tasks.contains(&id) {
                    events.push(event("join", *n, Some(id.clone())));
                }
                if let Some(resource) = pending_io.get(&id) {
                    events.push(event("completion", *n, Some(resource.clone())));
                }
            }
        }
    }
    let mut findings = vec![];
    for (_, capture, write) in captures.iter().filter(|(_, c, _)| *c == "shared_write") {
        let _ = capture;
        let Some(actor) = owner(report, Span::of(*write)) else {
            continue;
        };
        let Some(launches) = actors.get(&actor) else {
            continue;
        };
        let Some(left) = write
            .child_by_field_name("left")
            .or_else(|| write.named_child(0))
        else {
            continue;
        };
        let name = base(text(left, source));
        let Some(bound) = binding(name, Span::of(left)) else {
            continue;
        };
        if actor.contains(bound) {
            continue;
        }
        let Some(id) = identity(text(left, source), Span::of(left)) else {
            continue;
        };
        let mut evidence = launches.clone();
        evidence.push(event("shared-write", *write, Some(id)));
        findings.push((RACE, "concurrent-shared-write", evidence, true, counts.get(&launches[0].owner).cloned().unwrap_or_default(), "A launched execution scope writes an outer binding. Conflicting callers and synchronization remain unresolved; this marks a contention candidate."));
    }
    for e in &events {
        let sites = counts.get(&e.owner).cloned().unwrap_or_default();
        let (defect, mechanism, explanation) = match e.kind.as_str() {
            "declaration" => (ASYNC, "async-function", "Explicit async declaration; does not establish concurrent execution."),
            "launch" => (ASYNC, "async-execution", "Asynchronous launch or task composition; counts describe static sites, not simultaneous tasks."),
            "suspend" => (ASYNC, "suspension", "Potential suspension point; another task may change shared state before continuation."),
            "lock" | "unlock" | "atomic-read" | "atomic-write" | "synchronization" => (RACE, "synchronization-operation", "Synchronization-shaped method call; receiver type and protection scope are unresolved."),
            "unobserved" => (ASYNC, "unobserved-completion", "Locally declared async work is discarded without observing completion or failure."),
            "acquire" | "use" | "pending-use" | "release" | "io" | "path-use" | "check" => (IO, "resource-operation", "Recognized I/O operation; resource ordering and ownership require review."),
            _ => continue,
        };
        if e.kind != "path-use" {
            findings.push((
                defect,
                mechanism,
                vec![e.clone()],
                e.kind == "unobserved",
                sites,
                explanation,
            ));
        }
    }
    for e in &events {
        let Some(id) = &e.resource else {
            continue;
        };
        let related: Vec<_> = events
            .iter()
            .filter(|other| {
                other.resource.as_ref() == Some(id)
                    && other.owner == e.owner
                    && other.range.start > e.range.start
            })
            .collect();
        if e.kind == "launch" {
            let mut lifecycle = vec![e.clone()];
            lifecycle.extend(
                related
                    .iter()
                    .filter(|other| matches!(other.kind.as_str(), "cancel" | "join"))
                    .map(|e| (*e).clone()),
            );
            findings.push((ASYNC, "task-lifecycle", lifecycle, false, counts.get(&e.owner).cloned().unwrap_or_default(), "Task launch and completion/cancellation observations on the same binding. Cancellation request alone does not establish completion."));
        }
        if e.kind == "cancel" && !related.iter().any(|e| e.kind == "join") {
            findings.push((ASYNC, "cancellation-unobserved", vec![e.clone()], true, SiteCounts::default(), "Cancellation is requested without a subsequent local completion observation. Ownership may escape; completion is not proven."));
        }
        if e.kind == "pending-use" {
            if let Some(release) = related.iter().find(|e| e.kind == "release") {
                if !related
                    .iter()
                    .any(|e| e.kind == "completion" && e.range.start < release.range.start)
                {
                    findings.push((ASYNC, "release-before-completion", vec![e.clone(), (*release).clone()], true, SiteCounts::default(), "Asynchronous I/O is started and the resource is released without a local completion observation."));
                }
            }
        }
        if e.kind == "acquire" {
            let mut lifetime = vec![e.clone()];
            lifetime.extend(
                related
                    .iter()
                    .filter(|other| {
                        matches!(
                            other.kind.as_str(),
                            "use" | "pending-use" | "completion" | "release"
                        )
                    })
                    .map(|e| (*e).clone()),
            );
            let released = lifetime.iter().any(|e| e.kind == "release");
            let rebound = lifetime.last().is_some_and(|last| {
                all.iter().any(|n| {
                    n.start_byte() > e.range.end
                        && n.end_byte() < last.range.start
                        && matches!(
                            n.kind(),
                            "assignment_expression" | "assignment" | "assignment_statement"
                        )
                        && n.child_by_field_name("left").is_some_and(|left| {
                            identity(text(left, source), Span::of(left)).as_ref() == Some(id)
                        })
                })
            });
            if released && !rebound {
                findings.push((IO, "resource-lifecycle", lifetime, false, counts.get(&e.owner).cloned().unwrap_or_default(), "Acquisition and explicit release of the same binding. Control-flow completeness, asynchronous completion and exception safety remain unresolved."));
            }
        }
        if e.kind == "release" {
            for use_event in related.iter().filter(|e| e.kind == "use") {
                findings.push((ASYNC, "use-after-release", vec![e.clone(), (*use_event).clone()], true, SiteCounts::default(), "The same lexical resource is used after an explicit release; branch reachability remains unresolved."));
            }
        }
        if matches!(e.kind.as_str(), "atomic-read" | "lock") {
            let ending = if e.kind == "atomic-read" {
                "atomic-write"
            } else {
                "lock"
            };
            if let Some(later) = related.iter().find(|e| e.kind == ending) {
                findings.push((RACE, if e.kind == "atomic-read" { "split-atomic-update" } else { "split-critical-section" }, vec![e.clone(), (*later).clone()], true, SiteCounts::default(), "Separate synchronization operations on the same receiver may leave a check/update gap. Receiver type, invariant and lock scope are unresolved."));
            }
        }
        if e.kind == "check" {
            for use_event in related
                .iter()
                .filter(|e| matches!(e.kind.as_str(), "io" | "path-use"))
            {
                let operation = &source[use_event.range.start..use_event.range.end];
                if operation.contains("O_EXCL")
                    || operation.contains("\"x\"")
                    || operation.contains("'x'")
                    || operation.contains("\"wx\"")
                    || operation.contains("'wx'")
                {
                    continue;
                }
                findings.push((IO, "check-then-use", vec![e.clone(), (*use_event).clone()], true, SiteCounts::default(), "The same path is checked and later used. Another actor can invalidate the checked state; no external actor is proven."));
                break;
            }
        }
    }
    // A saved property value feeds a later write after suspension in the same function.
    for n in &all {
        let Some(value) = n
            .child_by_field_name("value")
            .or_else(|| n.child_by_field_name("right"))
        else {
            continue;
        };
        if !matches!(
            value.kind(),
            "member_expression"
                | "attribute"
                | "selector_expression"
                | "field_expression"
                | "identifier"
        ) {
            continue;
        }
        let Some(target) = n
            .child_by_field_name("name")
            .or_else(|| n.child_by_field_name("pattern"))
            .or_else(|| n.child_by_field_name("left"))
        else {
            continue;
        };
        if target.kind() != "identifier" {
            continue;
        }
        let Some(id) = identity(text(value, source), Span::of(value)) else {
            continue;
        };
        let saved = identity(text(target, source), Span::of(target));
        for (_, capture, write) in captures.iter().filter(|(_, c, _)| *c == "shared_write") {
            let _ = capture;
            let Some(left) = write.child_by_field_name("left") else {
                continue;
            };
            let Some(right) = write.child_by_field_name("right") else {
                continue;
            };
            if write.start_byte() <= n.end_byte()
                || owner(report, Span::of(*write)) != owner(report, Span::of(*n))
                || identity(text(left, source), Span::of(left)).as_ref() != Some(&id)
            {
                continue;
            }
            if !crate::annotations::nodes(right).iter().any(|v| {
                v.kind() == "identifier" && identity(text(*v, source), Span::of(*v)) == saved
            }) {
                continue;
            }
            if let Some(suspend) = events.iter().find(|e| {
                e.kind == "suspend"
                    && e.owner == owner(report, Span::of(*n))
                    && e.range.start >= n.end_byte()
                    && e.range.end <= write.start_byte()
            }) {
                findings.push((RACE, "read-suspend-write", vec![event("read", value, Some(id.clone())), suspend.clone(), event("write", *write, Some(id.clone()))], true, counts.get(&suspend.owner).cloned().unwrap_or_default(), "A saved resource value feeds a write after suspension. Concurrent invocation can overwrite a newer value; exclusivity is not proven."));
            }
        }
    }
    for (defect, mechanism, events, candidate, sites, explanation) in findings {
        emit(
            report,
            defect,
            mechanism,
            events,
            candidate,
            sites,
            explanation,
        );
    }
    let compounds: Vec<_> = report.findings.iter().filter(|f| f.mechanism.as_deref() == Some("read-suspend-write")).filter_map(|race| {
        let mutation = report.findings.iter().find(|f| f.family == "mutated-state" && f.function == race.function && race.trigger.contains(f.trigger) && matches!(f.effect, Effect::LocalMutation | Effect::GlobalMutation | Effect::ArgumentMutation(_) | Effect::ReceiverMutation))?;
        let mut compound = race.clone();
        compound.rule_id = format!("{}.compound-defects.read-suspend-write", report.language);
        compound.family = "compound-defects".into();
        compound.compound = Some(Compound { class: "data-corruption".into(), category: "shared-mutation".into(), ingredients: [Evidence { uri: report.uri.clone(), range: race.trigger, rule_id: race.rule_id.clone() }, Evidence { uri: report.uri.clone(), range: mutation.trigger, rule_id: mutation.rule_id.clone() }] });
        compound.explanation = "Shared-state and mutation evidence meet at a saved-value write after suspension; concurrent invocation may lose an update.".into();
        Some(compound)
    }).collect();
    report.findings.extend(compounds);
}

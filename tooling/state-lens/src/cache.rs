use crate::{
    annotations::nodes,
    defect_support::*,
    model::*,
    shared::{emit, Event, SiteCounts},
};
use std::collections::BTreeSet;
use tree_sitter::Node;

pub(crate) const DEFECT: &str = "cache-invalidation";
const MAX_KEY_DEPTH: usize = 8;
const ACCESS: &str = "cache-access";
const HERD: &str = "thundering-herd";
const REJECTED: &str = "cached-rejected-promise";
const DEPENDENCY: &str = "key-missing-dependency";
const STALE: &str = "stale-after-write";
const WRONG_KEY: &str = "invalidate-wrong-key";
const IDENTITY: &str = "mutable-identity-key";
const NEGATIVE: &str = "negative-result-without-expiry";
const MUTABLE: &str = "cached-mutable-value";
const ONCE: &str = "one-time-cache-stale";

pub(crate) fn analyze(
    captures: &[(&str, &str, Node<'_>)],
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: impl Fn(&str, Span) -> Option<Span>,
) {
    let all = nodes(root);
    analyze_indexed(&all, source, report, &resolve);
    indexed_storage(&all, source, report, &resolve);
    for call in all
        .iter()
        .filter(|n| calls(**n) && method(**n, source) == "get_or_init")
    {
        if nodes(*call)
            .iter()
            .any(|n| calls(*n) && method(*n, source) == "lock")
        {
            emit(report, DEFECT, ONCE, vec![Event { kind: "once-initialized-mutable-source".into(), range: Span::of(*call), resource: receiver(*call).map(|r| text(r, source).into()), owner: owner(report, *call) }], true, SiteCounts::default(), "One-time initialization snapshots a mutable locked source. Subsequent source changes do not refresh this cache.");
        }
    }
    let calls = captures
        .iter()
        .filter(|(path, capture, _)| path.contains(DEFECT) && *capture == "cache_operation")
        .map(|(_, _, n)| *n)
        .collect::<Vec<_>>();
    for store in calls
        .iter()
        .filter(|n| matches!(method(**n, source), "set" | "insert"))
    {
        let (Some(map), arguments) = (receiver(*store), args(*store)) else {
            continue;
        };
        if arguments.len() < 2 {
            continue;
        }
        let Some(binding) =
            field_identity(map, &all, source).or_else(|| resolve(base(map, source), Span::of(map)))
        else {
            continue;
        };
        let own = owner(report, *store);
        if own.is_none_or(|o| o.contains(binding)) {
            continue;
        }
        let same_map = |n: Node<'_>| {
            receiver(n).is_some_and(|r| {
                field_identity(r, &all, source).or_else(|| resolve(base(r, source), Span::of(r)))
                    == Some(binding)
            })
        };
        let reads = calls
            .iter()
            .filter(|n| {
                same_map(**n)
                    && matches!(method(**n, source), "get" | "has" | "contains_key")
                    && owner(report, **n) == own
            })
            .copied()
            .collect::<Vec<_>>();
        if reads.is_empty() {
            continue;
        }
        let key = value(arguments[0], &all, source, &resolve).unwrap_or(arguments[0]);
        let computed = value(arguments[1], &all, source, &resolve).unwrap_or(arguments[1]);
        let mut related = nodes(computed);
        if let Some(callee) = callee(computed) {
            let local = report
                .functions
                .iter()
                .filter(|f| f.name == text(callee, source))
                .collect::<Vec<_>>();
            if local.len() == 1 {
                related.extend(
                    all.iter()
                        .filter(|n| {
                            local[0].body.contains(Span::of(**n))
                                && owner(report, **n) == Some(local[0].range)
                        })
                        .copied(),
                );
            }
        }
        let mut risks: Vec<(&str, Node<'_>)> = vec![];
        let key_bindings = identifiers(key)
            .iter()
            .filter_map(|n| resolve(text(*n, source), Span::of(*n)))
            .collect::<BTreeSet<_>>();
        let mutable_dependency = related
            .iter()
            .filter(|n| n.kind() == "identifier")
            .any(|n| {
                let Some(bound) = resolve(text(*n, source), Span::of(*n)) else {
                    return false;
                };
                if key_bindings.contains(&bound)
                    || owner(report, *n).is_none_or(|o| o.contains(bound))
                {
                    return false;
                }
                all.iter().any(|write| {
                    assignment(*write)
                        && left(*write).is_some_and(|l| {
                            l.kind() == "identifier"
                                && resolve(text(l, source), Span::of(l)) == Some(bound)
                        })
                        && write.start_byte() > bound.end
                })
            });
        if mutable_dependency {
            risks.push((DEPENDENCY, *store));
        }
        if related
            .iter()
            .any(|n| matches!(n.kind(), "await" | "await_expression"))
            && reads.iter().any(|r| r.end_byte() < computed.start_byte())
        {
            risks.push((HERD, *store));
        }
        let shared_future = report.language == "rust"
            && related
                .iter()
                .any(|n| crate::defect_support::calls(*n) && method(*n, source) == "shared")
            && all
                .iter()
                .any(|n| n.kind() == "use_declaration" && text(*n, source).contains("FutureExt"));
        let async_call = shared_future
            || related.iter().any(|n| {
                callee(*n).is_some_and(|c| {
                    report.functions.iter().any(|f| {
                        f.name == text(c, source)
                            && source[f.range.start..f.range.end]
                                .trim_start()
                                .starts_with("async")
                    })
                })
            });
        let evicts = related.iter().any(|n| {
            same_map(*n)
                && matches!(method(*n, source), "delete" | "remove")
                && args(*n)
                    .first()
                    .is_some_and(|k| text(*k, source) == text(arguments[0], source))
        });
        if async_call
            && !evicts
            && !related
                .iter()
                .any(|n| matches!(n.kind(), "await" | "await_expression"))
        {
            risks.push((REJECTED, *store));
        }
        let parameter_names = report
            .functions
            .iter()
            .find(|f| Some(f.range) == own)
            .map(|f| f.parameters.iter().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        let key_names = identifiers(key)
            .iter()
            .map(|n| text(*n, source))
            .collect::<BTreeSet<_>>();
        if identifiers(computed).iter().any(|n| {
            parameter_names.contains(text(*n, source)) && !key_names.contains(text(*n, source))
        }) {
            risks.push((DEPENDENCY, *store));
        }
        if let Some(initial) = value(map, &all, source, &resolve) {
            if callee(initial).is_some_and(|n| text(n, source) == "WeakMap") {
                risks.push((IDENTITY, *store));
            }
        }
        let upstream = related
            .iter()
            .filter_map(|n| {
                receiver(*n).or_else(|| {
                    if n.kind() == "index_expression" {
                        field(*n, &["value", "operand"])
                    } else {
                        None
                    }
                })
            })
            .filter_map(|r| {
                field_identity(r, &all, source).or_else(|| resolve(base(r, source), Span::of(r)))
            })
            .filter(|b| *b != binding)
            .collect::<BTreeSet<_>>();
        for update in calls.iter().filter(|n| {
            matches!(method(**n, source), "set" | "insert")
                && receiver(**n)
                    .and_then(|r| resolve(base(r, source), Span::of(r)))
                    .is_some_and(|b| upstream.contains(&b))
                && owner(report, **n) != own
        }) {
            let invalidations = calls
                .iter()
                .filter(|n| {
                    same_map(**n)
                        && matches!(method(**n, source), "delete" | "remove" | "clear")
                        && owner(report, **n) == owner(report, *update)
                })
                .collect::<Vec<_>>();
            if invalidations.is_empty() {
                risks.push((STALE, *update));
            }
            for invalidation in invalidations {
                if let Some(deleted) = args(*invalidation).first() {
                    let normalized = value(*deleted, &all, source, &resolve).unwrap_or(*deleted);
                    if key_shape(key, &all, source, report, &resolve, MAX_KEY_DEPTH)
                        != key_shape(normalized, &all, source, report, &resolve, MAX_KEY_DEPTH)
                    {
                        risks.push((WRONG_KEY, *invalidation));
                    }
                }
            }
        }
        if risks.is_empty() {
            emit(report, DEFECT, ACCESS, vec![Event { kind: "cache-read".into(), range: Span::of(reads[0]), resource: Some(format!("{}:{}", binding.start, binding.end)), owner: own }, Event { kind: "cache-store".into(), range: Span::of(*store), resource: Some(format!("{}:{}", binding.start, binding.end)), owner: own }], false, SiteCounts::default(), "A long-lived lookup stores computed values for reuse. This marks cache sharing without asserting an invalidation failure.");
        }
        for (mechanism, trigger) in risks {
            if report.findings.iter().any(|f| {
                f.defect == DEFECT
                    && f.trigger == Span::of(trigger)
                    && f.mechanism.as_deref() == Some(mechanism)
            }) {
                continue;
            }
            let resource = Some(format!("{}:{}", binding.start, binding.end));
            let mut events = reads
                .iter()
                .map(|r| Event {
                    kind: "cache-read".into(),
                    range: Span::of(*r),
                    resource: resource.clone(),
                    owner: own,
                })
                .collect::<Vec<_>>();
            events.push(Event {
                kind: "computed-value".into(),
                range: Span::of(computed),
                resource: resource.clone(),
                owner: own,
            });
            events.push(Event {
                kind: "cache-write".into(),
                range: Span::of(*store),
                resource: resource.clone(),
                owner: own,
            });
            events.push(Event {
                kind: mechanism.into(),
                range: Span::of(trigger),
                resource,
                owner: owner(report, trigger),
            });
            emit(report, DEFECT, mechanism, events, true, SiteCounts::default(), "A long-lived lookup and stored computation have a potential freshness, dependency, or completion mismatch. Application cache policy and external invalidation are not proven.");
        }
    }
}
fn key_shape(
    node: Node<'_>,
    all: &[Node<'_>],
    source: &str,
    report: &DocumentReport,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
    depth: usize,
) -> String {
    if depth == 0 {
        return text(node, source).into();
    }
    if node.kind() == "parenthesized_expression" {
        return node
            .named_child(0)
            .map(|n| key_shape(n, all, source, report, resolve, depth - 1))
            .unwrap_or_default();
    }
    if node.kind() == "identifier" {
        if let Some(initial) = value(node, all, source, resolve).filter(|n| n.id() != node.id()) {
            return key_shape(initial, all, source, report, resolve, depth - 1);
        }
        if let Some(position) = report
            .functions
            .iter()
            .find(|f| Some(f.range) == owner(report, node))
            .and_then(|f| f.parameters.iter().position(|p| p == text(node, source)))
        {
            return format!("parameter:{position}");
        }
    }
    if node.child_count() == 0 {
        return format!("{}:{}", node.kind(), text(node, source));
    }
    format!(
        "{}({})",
        node.kind(),
        node.children(&mut node.walk())
            .map(|n| key_shape(n, all, source, report, resolve, depth - 1))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn analyze_indexed(
    all: &[Node<'_>],
    source: &str,
    report: &mut DocumentReport,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
) {
    if report.language != "python" {
        return;
    }
    for store in all.iter().filter(|n| assignment(**n)) {
        let (Some(target), Some(computed)) = (left(*store), right(*store)) else {
            continue;
        };
        if target.kind() != "subscript" {
            continue;
        }
        let (Some(map), Some(key)) = (field(target, &["value"]), field(target, &["subscript"]))
        else {
            continue;
        };
        let Some(binding) = resolve(base(map, source), Span::of(map)) else {
            continue;
        };
        let own = owner(report, *store);
        if own.is_none_or(|o| o.contains(binding)) {
            continue;
        }
        let initial = value(map, all, source, resolve);
        if initial.is_none_or(|n| n.kind() != "dictionary" || n.named_child_count() != 0) {
            continue;
        }
        let same_map = |n: Node<'_>| {
            field(n, &["value"])
                .is_some_and(|r| resolve(base(r, source), Span::of(r)) == Some(binding))
        };
        let reads = all
            .iter()
            .filter(|n| {
                n.kind() == "subscript"
                    && same_map(**n)
                    && owner(report, **n) == own
                    && Span::of(**n) != Span::of(target)
            })
            .copied()
            .collect::<Vec<_>>();
        if reads.is_empty() {
            continue;
        }
        let mut risks = vec![];
        let task_value = value(computed, all, source, resolve).unwrap_or(computed);
        let cached_task = imported_call(task_value, all, source, resolve).is_some_and(|name| {
            matches!(
                name.as_str(),
                "asyncio.create_task" | "asyncio.ensure_future"
            )
        });
        let rejection_eviction = all.iter().any(|n| {
            calls(*n)
                && method(*n, source) == "pop"
                && receiver(*n)
                    .is_some_and(|r| resolve(base(r, source), Span::of(r)) == Some(binding))
                && args(*n)
                    .first()
                    .is_some_and(|k| text(*k, source) == text(key, source))
                && ancestor(*n, &["except_clause"]).is_some()
        });
        if cached_task && !rejection_eviction {
            risks.push((REJECTED, *store));
        }
        if computed.kind() == "none" {
            risks.push((NEGATIVE, *store));
        }
        let key_value = value(key, all, source, resolve).unwrap_or(key);
        let key_names = identifiers(key_value)
            .iter()
            .map(|n| text(*n, source))
            .collect::<BTreeSet<_>>();
        let params = report
            .functions
            .iter()
            .find(|f| Some(f.range) == own)
            .map(|f| f.parameters.clone())
            .unwrap_or_default();
        if identifiers(computed).iter().any(|n| {
            params.iter().any(|p| p == text(*n, source)) && !key_names.contains(text(*n, source))
        }) {
            risks.push((DEPENDENCY, *store));
        }
        if computed.kind() == "dictionary" {
            for read in &reads {
                if read
                    .parent()
                    .is_some_and(|p| p.kind() == "return_statement")
                {
                    let function_name = report
                        .functions
                        .iter()
                        .find(|f| Some(f.range) == own)
                        .map(|f| f.name.as_str())
                        .unwrap_or("");
                    let mutated_return = all.iter().filter(|n| assignment(**n)).any(|n| {
                        let (Some(alias), Some(call)) = (left(*n), right(*n)) else {
                            return false;
                        };
                        if !calls(call) || method(call, source) != function_name {
                            return false;
                        }
                        all.iter()
                            .filter(|other| {
                                assignment(**other) && other.start_byte() > n.end_byte()
                            })
                            .any(|other| {
                                left(*other).is_some_and(|l| {
                                    l.kind() == "subscript"
                                        && field(l, &["value"]).is_some_and(|r| {
                                            resolve(text(r, source), Span::of(r))
                                                == resolve(text(alias, source), Span::of(alias))
                                        })
                                })
                            })
                    });
                    if mutated_return {
                        risks.push((MUTABLE, *read));
                    }
                }
            }
        }
        if computed.kind() == "subscript" {
            if let Some(upstream) =
                field(computed, &["value"]).and_then(|r| resolve(base(r, source), Span::of(r)))
            {
                for update in all
                    .iter()
                    .filter(|n| assignment(**n) && owner(report, **n) != own)
                {
                    let Some(target) = left(*update).filter(|n| n.kind() == "subscript") else {
                        continue;
                    };
                    if field(target, &["value"]).and_then(|r| resolve(base(r, source), Span::of(r)))
                        != Some(upstream)
                    {
                        continue;
                    }
                    let evicted = all.iter().any(|n| {
                        calls(*n)
                            && owner(report, *n) == owner(report, *update)
                            && matches!(method(*n, source), "pop" | "clear")
                            && receiver(*n).and_then(|r| resolve(base(r, source), Span::of(r)))
                                == Some(binding)
                    });
                    if !evicted {
                        risks.push((STALE, *update));
                    }
                }
            }
        }
        for (mechanism, trigger) in risks {
            let mut events = vec![Event {
                kind: "cache-store".into(),
                range: Span::of(*store),
                resource: Some(format!("{}:{}", binding.start, binding.end)),
                owner: own,
            }];
            if let Some(condition) =
                ancestor(*store, &["if_statement"]).and_then(|n| field(n, &["condition"]))
            {
                events.push(Event {
                    kind: "cache-lookup".into(),
                    range: Span::of(condition),
                    resource: None,
                    owner: own,
                });
            }
            events.push(Event {
                kind: mechanism.into(),
                range: Span::of(trigger),
                resource: None,
                owner: owner(report, trigger),
            });
            emit(report, DEFECT, mechanism, events, true, SiteCounts::default(), "A long-lived indexed cache has a locally connected freshness, key dependency, or escaping mutable-value risk; external cache policy remains unresolved.");
        }
    }
    for delete in all
        .iter()
        .filter(|n| calls(**n) && method(**n, source) == "pop")
    {
        let Some(map) = receiver(*delete) else {
            continue;
        };
        let Some(initial) =
            value(map, all, source, resolve).filter(|n| calls(*n) && method(*n, source) == "dict")
        else {
            continue;
        };
        let Some(upstream) = args(initial)
            .first()
            .and_then(|n| resolve(text(*n, source), Span::of(*n)))
        else {
            continue;
        };
        let Some(key) = args(*delete)
            .first()
            .copied()
            .filter(|n| n.kind() == "string")
        else {
            continue;
        };
        if let Some(update) = all.iter().find(|n| {
            assignment(**n)
                && owner(report, **n) == owner(report, *delete)
                && left(**n).is_some_and(|l| {
                    field(l, &["value"]).and_then(|r| resolve(text(r, source), Span::of(r)))
                        == Some(upstream)
                        && field(l, &["subscript"]).is_some_and(|k| k.kind() == "identifier")
                })
        }) {
            emit(report, DEFECT, WRONG_KEY, vec![Event {kind: "source-write".into(),range:Span::of(*update),resource:None,owner:owner(report,*update)}, Event {kind:"cache-delete".into(),range:Span::of(*delete),resource:Some(text(key,source).into()),owner:owner(report,*delete)}], true, SiteCounts::default(), "A copied source map is updated by a variable key while invalidation removes a fixed key. The two keys need not identify the same cache entry.");
        }
    }
}

fn field_identity(node: Node<'_>, all: &[Node<'_>], source: &str) -> Option<Span> {
    let member = field(node, &["field", "property", "attribute"])?;
    let declarations = all
        .iter()
        .filter(|n| {
            n.kind() == "field_declaration"
                && field(**n, &["name"])
                    .is_some_and(|name| text(name, source) == text(member, source))
        })
        .collect::<Vec<_>>();
    (declarations.len() == 1).then(|| Span::of(*declarations[0]))
}

fn indexed_storage(
    all: &[Node<'_>],
    source: &str,
    report: &mut DocumentReport,
    resolve: &impl Fn(&str, Span) -> Option<Span>,
) {
    if report.language != "go" {
        return;
    }
    for store in all.iter().filter(|n| assignment(**n)) {
        let Some(targets) = left(*store) else {
            continue;
        };
        let target = if targets.kind() == "expression_list" {
            targets.named_child(0).unwrap_or(targets)
        } else {
            targets
        };
        if target.kind() != "index_expression" {
            continue;
        }
        let Some(map) = field(target, &["operand"]) else {
            continue;
        };
        let Some(cache_id) = field_identity(map, all, source) else {
            continue;
        };
        let own = owner(report, *store);
        let reads = all
            .iter()
            .filter(|n| {
                n.kind() == "index_expression"
                    && n.end_byte() < store.start_byte()
                    && owner(report, **n) == own
                    && field(**n, &["operand"]).and_then(|m| field_identity(m, all, source))
                        == Some(cache_id)
            })
            .copied()
            .collect::<Vec<_>>();
        if reads.is_empty() {
            continue;
        }
        let invalidates = |function: Option<Span>| {
            all.iter().any(|n| {
                owner(report, *n) == function
                    && calls(*n)
                    && method(*n, source) == "delete"
                    && args(*n)
                        .first()
                        .and_then(|m| field_identity(*m, all, source))
                        == Some(cache_id)
            })
        };
        let mut risks = Vec::new();
        if right(*store).is_some_and(|v| v.kind() == "nil")
            && !all.iter().any(|n| {
                calls(*n)
                    && method(*n, source) == "delete"
                    && args(*n)
                        .first()
                        .and_then(|m| field_identity(*m, all, source))
                        == Some(cache_id)
            })
        {
            risks.push((NEGATIVE, *store));
        }
        let computation = right(*store).and_then(|v| value(v, all, source, resolve));
        let source_reads = computation
            .map(nodes)
            .unwrap_or_default()
            .into_iter()
            .filter(|n| n.kind() == "index_expression")
            .filter_map(|n| field(n, &["operand"]).and_then(|m| field_identity(m, all, source)))
            .filter(|id| *id != cache_id)
            .collect::<BTreeSet<_>>();
        for update in all
            .iter()
            .filter(|n| assignment(**n) && owner(report, **n) != own)
        {
            let Some(targets) = left(*update) else {
                continue;
            };
            let target = if targets.kind() == "expression_list" {
                targets.named_child(0).unwrap_or(targets)
            } else {
                targets
            };
            if target.kind() == "index_expression"
                && field(target, &["operand"])
                    .and_then(|m| field_identity(m, all, source))
                    .is_some_and(|id| source_reads.contains(&id))
                && !invalidates(owner(report, *update))
            {
                risks.push((STALE, *update));
            }
        }
        if let Some(slow) = all.iter().find(|n| {
            owner(report, **n) == own
                && n.start_byte() > reads[0].end_byte()
                && n.end_byte() < store.start_byte()
                && calls(**n)
                && matches!(
                    callee(**n).map(|c| text(c, source)),
                    Some("time.Sleep" | "http.Get")
                )
        }) {
            let protected = report
                .findings
                .iter()
                .filter_map(|f| f.shared_state.as_ref()?.synchronization.as_ref())
                .any(|s| {
                    s.protected_regions
                        .iter()
                        .any(|r| r.contains(Span::of(reads[0])) && r.contains(Span::of(*store)))
                });
            if !protected {
                risks.push((HERD, *slow));
            }
        }
        for (mechanism, trigger) in risks {
            let resource = Some(format!("{}:{}", cache_id.start, cache_id.end));
            let mut events = reads
                .iter()
                .map(|r| Event {
                    kind: "cache-read".into(),
                    range: Span::of(*r),
                    resource: resource.clone(),
                    owner: own,
                })
                .collect::<Vec<_>>();
            events.push(Event {
                kind: "cache-store".into(),
                range: Span::of(*store),
                resource: resource.clone(),
                owner: own,
            });
            events.push(Event {
                kind: mechanism.into(),
                range: Span::of(trigger),
                resource,
                owner: owner(report, trigger),
            });
            emit(report, DEFECT, mechanism, events, true, SiteCounts::default(), "A field-backed cache lookup and store connect to a stale source update or an unprotected miss interval. Receiver aliases and external invalidation remain unresolved.");
        }
    }
}

use crate::{
    annotations::nodes,
    defect_support::*,
    model::*,
    shared::{emit, Event, SiteCounts},
};
use tree_sitter::Node;

pub(crate) const DEFECT: &str = "resource-contention";
const USAGE: &str = "resource-usage";
const QUEUE: &str = "unbounded-queue";
const FAN_OUT: &str = "unbounded-fan-out";
const LOCK_BLOCKING: &str = "lock-held-across-blocking";
const RELEASE: &str = "acquire-without-release-on-error";
const POOL: &str = "per-request-pool";
const FAN_IN: &str = "blocking-fan-in";
const DEADLINE: &str = "acquisition-without-deadline";
const CAPACITY: &str = "permit-accounting";
const EVENT_LOOP: &str = "event-loop-blocking";
const LOOPS: &[&str] = &[
    "for_statement",
    "for_in_statement",
    "while_statement",
    "for_expression",
    "list_comprehension",
];

pub(crate) fn analyze(
    captures: &[(&str, &str, Node<'_>)],
    root: Node<'_>,
    source: &str,
    report: &mut DocumentReport,
    resolve: impl Fn(&str, Span) -> Option<Span>,
) {
    let all = nodes(root);
    for &(path, capture, node) in captures {
        if !path.contains(DEFECT) || capture != "contention_operation" || node.has_error() {
            continue;
        }
        let name = imported_call(node, &all, source, &resolve)
            .unwrap_or_default()
            .split("::<")
            .next()
            .unwrap_or("")
            .replace("::", ".");
        if callee(node).is_some_and(|c| {
            let head = base(c, source).split("::").next().unwrap_or("");
            matches!(
                head,
                "Promise"
                    | "queue"
                    | "asyncio"
                    | "threading"
                    | "thread"
                    | "mpsc"
                    | "std"
                    | "tokio"
                    | "http"
                    | "https"
                    | "time"
                    | "requests"
                    | "fetch"
            ) && resolve(head, Span::of(c)).is_some()
        }) {
            continue;
        }
        if node.kind() == "if_statement" {
            if let Some(condition) = field(node, &["condition"]) {
                let owner = owner(report, node);
                let counters = nodes(condition)
                    .iter()
                    .filter(|n| n.kind() == "identifier")
                    .filter_map(|n| resolve(text(*n, source), Span::of(*n)))
                    .collect::<Vec<_>>();
                let updates = all
                    .iter()
                    .filter(|n| {
                        n.kind() == "update_expression"
                            && crate::defect_support::owner(report, **n) == owner
                            && identifiers(**n).iter().any(|id| {
                                resolve(text(*id, source), Span::of(*id))
                                    .is_some_and(|b| counters.contains(&b))
                            })
                    })
                    .copied()
                    .collect::<Vec<_>>();
                let increments = updates.iter().any(|n| text(*n, source).contains("++"));
                let releases = updates.iter().any(|n| {
                    text(*n, source).contains("--") && ancestor(*n, &["finally_clause"]).is_some()
                });
                if increments && releases {
                    let mut events = updates
                        .iter()
                        .map(|n| Event {
                            kind: "permit-count".into(),
                            range: Span::of(*n),
                            resource: None,
                            owner,
                        })
                        .collect::<Vec<_>>();
                    events.extend(
                        all.iter()
                            .filter(|n| {
                                matches!(n.kind(), "await_expression" | "await")
                                    && crate::defect_support::owner(report, **n) == owner
                            })
                            .map(|n| Event {
                                kind: "permit-suspension".into(),
                                range: Span::of(*n),
                                resource: None,
                                owner,
                            }),
                    );
                    events.push(Event {
                        kind: "capacity-check".into(),
                        range: Span::of(condition),
                        resource: None,
                        owner,
                    });
                    emit(report, DEFECT, CAPACITY, events, false, SiteCounts::default(), "An invocation checks shared capacity and releases its counted permit in finally. This marks resource sharing and its lifetime, not a capacity violation.");
                }
            }
        }
        let arguments = args(node);
        let own = owner(report, node);
        let mut mechanisms = Vec::new();
        let capacity = arguments
            .iter()
            .find(|n| field(**n, &["name"]).is_some_and(|name| text(name, source) == "maxsize"))
            .and_then(|n| field(*n, &["value"]))
            .or_else(|| {
                arguments
                    .first()
                    .copied()
                    .filter(|n| n.kind() != "keyword_argument")
            });
        let unbounded_capacity = capacity.is_none()
            || capacity
                .and_then(|n| value(n, &all, source, &resolve))
                .and_then(|n| text(n, source).replace('_', "").parse::<i64>().ok())
                .is_some_and(|n| n <= 0);
        if matches!(name.as_str(), "queue.Queue" | "asyncio.Queue") && unbounded_capacity
            || matches!(
                name.as_str(),
                "mpsc.channel"
                    | "mpsc.unbounded_channel"
                    | "tokio.sync.mpsc.unbounded_channel"
                    | "std.sync.mpsc.channel"
            )
        {
            mechanisms.push(QUEUE);
        }
        let dynamic = ancestor(node, LOOPS).is_some()
            || ancestor(node, &["closure_expression"])
                .and_then(|n| ancestor(n, &["call_expression"]))
                .is_some_and(|n| method(n, source) == "map")
            || nodes(node).iter().any(|n| {
                matches!(n.kind(), "list_comprehension" | "list_splat")
                    || calls(*n) && method(*n, source) == "map"
            });
        let launch = matches!(
            name.as_str(),
            "Promise.all"
                | "Promise.allSettled"
                | "asyncio.gather"
                | "threading.Thread"
                | "thread.spawn"
                | "std.thread.spawn"
                | "tokio.spawn"
        ) || node.kind() == "go_statement";
        let limited = report.language == "go"
            && own.is_some_and(|o| {
                all.iter()
                    .filter(|n| {
                        o.contains(Span::of(**n))
                            && n.kind() == "send_statement"
                            && n.end_byte() < node.start_byte()
                    })
                    .any(|send| {
                        let Some(channel) =
                            field(*send, &["channel"]).or_else(|| send.named_child(0))
                        else {
                            return false;
                        };
                        let identity = resolve(text(channel, source), Span::of(channel));
                        let buffered = value(channel, &all, source, &resolve).is_some_and(|v| {
                            calls(v) && method(v, source) == "make" && args(v).len() > 1
                        });
                        let released = nodes(node).iter().any(|n| {
                            n.kind() == "unary_expression"
                                && text(*n, source).starts_with("<-")
                                && identifiers(*n).iter().any(|i| {
                                    resolve(text(*i, source), Span::of(*i)) == identity
                                        && identity.is_some()
                                })
                        });
                        buffered && released && ancestor(*send, LOOPS) == ancestor(node, LOOPS)
                    })
            });
        let bounded_batch = nodes(node)
            .iter()
            .any(|n| calls(*n) && method(*n, source) == "slice" && args(*n).len() == 2);
        let worker_pool = report.language == "rust"
            && nodes(node).iter().any(|n| {
                n.kind() == "while_expression"
                    && field(*n, &["condition"]).is_some_and(|condition| {
                        nodes(condition).iter().any(|call| {
                            calls(*call) && matches!(method(*call, source), "fetch_add" | "recv")
                        })
                    })
            });
        if launch && dynamic && !limited && !bounded_batch && !worker_pool {
            mechanisms.push(FAN_OUT);
        }
        if own.is_some()
            && (matches!(
                name.as_str(),
                "http.Transport"
                    | "http.Client"
                    | "http.Agent"
                    | "https.Agent"
                    | "reqwest.Client.new"
                    | "aiohttp.ClientSession"
                    | "requests.Session"
            ) || node.kind() == "composite_literal"
                && field(node, &["type"])
                    .is_some_and(|t| matches!(text(t, source), "http.Transport" | "http.Client")))
        {
            mechanisms.push(POOL);
        }
        let blocking = matches!(
            name.as_str(),
            "time.sleep"
                | "time.Sleep"
                | "thread.sleep"
                | "std.thread.sleep"
                | "asyncio.sleep"
                | "fetch"
                | "http.Get"
                | "requests.get"
        ) || method(node, source) == "wait"
            || node.kind() == "await_expression"
            || node.kind() == "await";
        if blocking
            && report
                .findings
                .iter()
                .filter_map(|f| f.shared_state.as_ref()?.synchronization.as_ref())
                .any(|s| {
                    s.protected_regions
                        .iter()
                        .any(|r| r.contains(Span::of(node)))
                })
        {
            mechanisms.push(LOCK_BLOCKING);
        }
        if matches!(method(node, source), "acquire" | "connect") && own.is_some() {
            let receiver_name = receiver(node).map(|n| text(n, source));
            let guarded = ancestor(node, &["with_statement", "defer_statement"]).is_some()
                || all.iter().any(|n| {
                    owner(report, *n) == own
                        && calls(*n)
                        && method(*n, source) == "release"
                        && receiver(*n).map(|n| text(n, source)) == receiver_name
                        && ancestor(*n, &["finally_clause", "defer_statement"]).is_some()
                        && ancestor(*n, &["try_statement"])
                            .and_then(|t| field(t, &["body"]))
                            .is_some_and(|body| {
                                all.iter()
                                    .filter(|event| {
                                        owner(report, **event) == own
                                            && event.start_byte() > node.end_byte()
                                            && event.end_byte() < n.start_byte()
                                            && matches!(
                                                event.kind(),
                                                "throw_statement"
                                                    | "raise_statement"
                                                    | "await_expression"
                                                    | "await"
                                            )
                                    })
                                    .all(|event| Span::of(body).contains(Span::of(*event)))
                            })
                });
            if !guarded
                && report.language != "rust"
                && all.iter().any(|n| {
                    owner(report, *n) == own
                        && n.start_byte() > node.end_byte()
                        && matches!(
                            n.kind(),
                            "throw_statement" | "raise_statement" | "await_expression" | "await"
                        )
                })
            {
                mechanisms.push(RELEASE);
            }
        }
        if node.kind() == "send_statement" && ancestor(node, &["select_statement"]).is_none() {
            let channel = field(node, &["channel"]).or_else(|| node.named_child(0));
            let buffered = channel.is_some_and(|c| {
                all.iter().filter(|n| assignment(**n)).any(|n| {
                    left(*n).is_some_and(|l| text(l, source) == text(c, source))
                        && right(*n).is_some_and(|r| {
                            calls(r) && method(r, source) == "make" && args(r).len() > 1
                        })
                })
            });
            let semaphore =
                field(node, &["value"]).is_some_and(|n| text(n, source).starts_with("struct{}"));
            if semaphore && ancestor(node, LOOPS).is_none() {
                mechanisms.push(DEADLINE);
            } else if !buffered {
                mechanisms.push(FAN_IN);
            }
        }
        if matches!(report.language.as_str(), "javascript" | "typescript")
            && method(node, source) == "push"
        {
            if let Some(queue) = receiver(node) {
                let drained = all.iter().any(|n| {
                    calls(*n)
                        && receiver(*n).is_some_and(|r| text(r, source) == text(queue, source))
                        && matches!(method(*n, source), "shift" | "splice")
                });
                let bounded = ancestor(node, &["if_statement"]).is_some();
                let has_size_check = own.is_some_and(|o| {
                    all.iter().any(|n| {
                        o.contains(Span::of(*n))
                            && n.kind() == "if_statement"
                            && field(*n, &["condition"]).is_some_and(|c| {
                                text(c, source).contains(&format!("{}.length", text(queue, source)))
                            })
                    })
                });
                if drained && !bounded && !has_size_check {
                    mechanisms.push(QUEUE);
                }
            }
        }
        if matches!(report.language.as_str(), "javascript" | "typescript")
            && matches!(
                method(node, source),
                "createHash" | "pbkdf2Sync" | "scryptSync"
            )
            && ancestor(node, LOOPS).is_some()
        {
            let yields = ancestor(node, LOOPS).is_some_and(|loop_node| {
                nodes(loop_node)
                    .iter()
                    .any(|n| n.kind() == "await_expression")
            });
            if !yields {
                mechanisms.push(EVENT_LOOP);
            }
        }
        if mechanisms.is_empty()
            && (launch
                || matches!(
                    name.as_str(),
                    "queue.Queue"
                        | "asyncio.Queue"
                        | "mpsc.sync_channel"
                        | "tokio.sync.mpsc.channel"
                ))
        {
            emit(report, DEFECT, USAGE, vec![Event { kind: "resource-usage".into(), range: Span::of(node), resource: None, owner: own }], false, SiteCounts { dynamic_fan_out: dynamic, ..Default::default() }, "A scheduler or queue operation is present without an established local contention risk. This usage marker preserves the resource-sharing area.");
        }
        for mechanism in mechanisms {
            if report.findings.iter().any(|f| {
                f.defect == DEFECT
                    && f.trigger == Span::of(node)
                    && f.mechanism.as_deref() == Some(mechanism)
            }) {
                continue;
            }
            let mut evidence = vec![];
            if mechanism == LOCK_BLOCKING {
                for sync in report
                    .findings
                    .iter()
                    .filter_map(|f| f.shared_state.as_ref()?.synchronization.as_ref())
                    .filter(|s| {
                        s.protected_regions
                            .iter()
                            .any(|r| r.contains(Span::of(node)))
                    })
                {
                    evidence.extend(sync.acquisitions.iter().map(|range| Event {
                        kind: "lock-acquired".into(),
                        range: *range,
                        resource: Some(sync.lock.clone()),
                        owner: own,
                    }));
                }
            }
            if mechanism == RELEASE {
                evidence.extend(
                    all.iter()
                        .filter(|n| {
                            owner(report, **n) == own
                                && n.start_byte() > node.end_byte()
                                && matches!(
                                    n.kind(),
                                    "throw_statement"
                                        | "raise_statement"
                                        | "await_expression"
                                        | "await"
                                )
                        })
                        .map(|n| Event {
                            kind: "exception-before-release".into(),
                            range: Span::of(*n),
                            resource: None,
                            owner: own,
                        }),
                );
            }
            evidence.push(Event {
                kind: mechanism.into(),
                range: Span::of(node),
                resource: receiver(node).map(|n| text(n, source).into()),
                owner: own,
            });
            emit(report, DEFECT, mechanism, evidence, true, SiteCounts { dynamic_fan_out: dynamic && launch, ..Default::default() }, "Resource demand or blocking is not locally bounded by an established capacity, release, or isolation contract. Workload and external controls remain unresolved.");
        }
    }
}

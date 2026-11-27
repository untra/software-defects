use state_lens::state::{extract_for, Branch, OperationKind, StateGraph};
use std::collections::BTreeSet;
use tree_sitter::Parser;

fn graph(source: &str, language: &str) -> StateGraph {
    let mut parser = Parser::new();
    let grammar = if language == "javascript" {
        tree_sitter_javascript::LANGUAGE.into()
    } else {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    };
    parser.set_language(&grammar).unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error());
    let graph = extract_for(
        language,
        tree.root_node(),
        "file:///cfg.ts",
        source,
        |_, _| None,
    );
    graph.validate().unwrap();
    let decoded: StateGraph =
        serde_json::from_str(&serde_json::to_string(&graph).unwrap()).unwrap();
    decoded.validate().unwrap();
    for op in &graph.operations {
        assert_eq!(op.block_id.is_none(), graph.heap.ops.contains(&op.id));
    }
    graph
}
fn edges(graph: &StateGraph) -> Vec<(usize, usize, Branch)> {
    let index = |id: &str| {
        graph
            .blocks
            .iter()
            .position(|block| block.id == id)
            .unwrap()
    };
    graph
        .control_flow
        .iter()
        .map(|edge| {
            assert_eq!(
                edge.condition.is_some(),
                edge.branch != Branch::Unconditional
            );
            (index(&edge.from), index(&edge.to), edge.branch)
        })
        .collect()
}
fn calls(graph: &StateGraph, source: &str, entry: usize) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut work = vec![graph.blocks[entry].id.clone()];
    while let Some(id) = work.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        work.extend(
            graph
                .control_flow
                .iter()
                .filter(|edge| edge.from == id)
                .map(|edge| edge.to.clone()),
        );
    }
    graph
        .operations
        .iter()
        .filter(|op| {
            op.kind == OperationKind::Call
                && op.block_id.as_ref().is_some_and(|id| seen.contains(id))
        })
        .map(|op| source[op.location.range.start..op.location.range.end].into())
        .collect()
}
#[test]
fn exact_branch_successors() {
    use Branch::{False as F, True as T, Unconditional as U};
    for language in ["javascript", "typescript"] {
        for (source, expected, reachable) in [
            (
                "function f(x) { if(x) return a(); else return b(); dead(); }",
                vec![
                    (2, 4, T),
                    (2, 5, F),
                    (4, 3, U),
                    (5, 3, U),
                    (6, 3, U),
                    (0, 1, U),
                ],
                vec!["a()", "b()"],
            ),
            (
                "function f(x) { if(x) return a(); b(); }",
                vec![
                    (2, 4, T),
                    (2, 5, F),
                    (4, 3, U),
                    (5, 6, U),
                    (6, 3, U),
                    (0, 1, U),
                ],
                vec!["a()", "b()"],
            ),
        ] {
            let g = graph(source, language);
            assert_eq!(edges(&g), expected);
            assert!(calls(&g, source, 0).is_empty());
            assert_eq!(
                calls(&g, source, 2),
                reachable.into_iter().map(String::from).collect()
            );
        }
    }
}
#[test]
fn exact_short_circuit_and_ternary_successors() {
    use Branch::{False as F, True as T, Unconditional as U};
    for language in ["javascript", "typescript"] {
        for op in ["&&", "||", "??"] {
            let source = format!("left() {op} right(); after();");
            let g = graph(&source, language);
            let (evaluate, skip) = if op == "||" { (3, 2) } else { (2, 3) };
            assert_eq!(
                edges(&g),
                vec![
                    (0, 2, T),
                    (0, 3, F),
                    (evaluate, 4, U),
                    (skip, 4, U),
                    (4, 1, U)
                ]
            );
            let right = g
                .operations
                .iter()
                .find(|operation| {
                    operation.kind == OperationKind::Call
                        && &source[operation.location.range.start..operation.location.range.end]
                            == "right()"
                        && operation.block_id.is_some()
                })
                .unwrap();
            assert_eq!(right.block_id.as_ref(), Some(&g.blocks[evaluate].id));
            assert!(!calls(&g, &source, skip).contains("right()"));
        }
        let g = graph("flag ? yes() : no(); after();", language);
        assert_eq!(
            edges(&g),
            vec![(0, 2, T), (0, 3, F), (2, 4, U), (3, 4, U), (4, 1, U)]
        );
    }
}
#[test]
fn exact_switch_fallthrough_and_break() {
    use Branch::{False as F, True as T, Unconditional as U};
    for language in ["javascript", "typescript"] {
        for (body, end) in [("a();", 4), ("a(); break;", 2)] {
            let source =
                format!("switch(x) {{ case 1: {body} default: b(); case test(): c(); }} after();");
            let g = graph(&source, language);
            assert_eq!(
                edges(&g),
                vec![
                    (0, 3, T),
                    (0, 6, F),
                    (6, 5, T),
                    (6, 7, F),
                    (7, 4, U),
                    (3, end, U),
                    (4, 5, U),
                    (5, 2, U),
                    (2, 1, U)
                ]
            );
            assert_eq!(calls(&g, &source, 3).contains("b()"), end == 4);
        }
        let g = graph(
            "switch(x) { case 1: case 2: yes(); break; } after();",
            language,
        );
        assert_eq!(
            edges(&g),
            vec![
                (0, 3, T),
                (0, 5, F),
                (5, 4, T),
                (5, 6, F),
                (6, 2, U),
                (3, 4, U),
                (4, 2, U),
                (2, 1, U)
            ]
        );
    }
}
#[test]
fn nested_scopes_and_unreachable_functions() {
    for language in ["javascript", "typescript"] {
        let source = "function outer() { before(); return; dead(); function inner() { nested(); } } const arrow = () => arrowBody(); const expression = function() { expressionBody(); }; const object = { method() { methodBody(); } }; top();";
        let g = graph(source, language);
        assert_eq!(calls(&g, source, 0), BTreeSet::from(["top()".into()]));
        assert_eq!(calls(&g, source, 2), BTreeSet::from(["before()".into()]));
        for name in [
            "nested()",
            "arrowBody()",
            "expressionBody()",
            "methodBody()",
        ] {
            assert!(g.operations.iter().any(|op| op.kind == OperationKind::Call
                && &source[op.location.range.start..op.location.range.end] == name
                && op.block_id.is_some()));
        }
    }
}

#[test]
fn sequential_and_nested_control_flow() {
    use Branch::Unconditional as U;
    for language in ["javascript", "typescript"] {
        let g = graph("first(); second();", language);
        assert_eq!(edges(&g), vec![(0, 1, U)]);
        let ordered: Vec<_> = g
            .operations
            .iter()
            .filter(|op| op.kind == OperationKind::Call && op.block_id.is_some())
            .map(|op| op.location.range.start)
            .collect();
        assert_eq!(ordered, vec![0, 9]);
        let source = "switch(x) { case 1: switch(y) { case 2: inner(); break; default: fallback(); } outer(); break; default: other(); } after();";
        let g = graph(source, language);
        let inner = g
            .operations
            .iter()
            .find(|op| {
                op.kind == OperationKind::Call
                    && &source[op.location.range.start..op.location.range.end] == "inner()"
                    && op.block_id.is_some()
            })
            .unwrap();
        let index = g
            .blocks
            .iter()
            .position(|block| Some(&block.id) == inner.block_id.as_ref())
            .unwrap();
        assert_eq!(
            calls(&g, source, index),
            ["inner()", "outer()", "after()"]
                .into_iter()
                .map(String::from)
                .collect()
        );
        let source = "function f() { if(a) { if(b) return yes(); else return no(); } else return other(); dead(); } flag && (left() || right());";
        let g = graph(source, language);
        assert!(!calls(&g, source, 2).contains("dead()"));
        assert!(!calls(&g, source, 0).contains("yes()"));
    }
}

#[test]
fn workspace_qualification_preserves_cfg_and_heap_evidence() {
    let mut engine = state_lens::analysis::Engine::new().unwrap();
    engine.set("file:///cfg.ts".into(), "typescript", "const original = { db: { host: 'a' } }; const copy = { ...original }; if(flag) copy.db.host = 'b'; else observe(original.db.host);".into(), Some(1)).unwrap();
    let mut previous = None;
    for version in 1..=2 {
        engine
            .set(
                "file:///other.ts".into(),
                "typescript",
                format!("const value = {version};"),
                Some(version),
            )
            .unwrap();
        let report = engine.analyze();
        let g = report
            .documents
            .iter()
            .find(|doc| doc.uri == "file:///cfg.ts")
            .unwrap()
            .state_graph
            .as_ref()
            .unwrap();
        g.validate().unwrap();
        assert!(!g.conditions.is_empty());
        assert!(!g.heap.ops.is_empty());
        if let Some(snapshot) = previous {
            assert_ne!(g.snapshot_id, snapshot);
        }
        previous = Some(g.snapshot_id.clone());
        for edge in &g.control_flow {
            if let Some(condition) = &edge.condition {
                assert!(condition.starts_with(&g.snapshot_id));
                assert!(!edge.evidence.is_empty());
            }
        }
    }
}

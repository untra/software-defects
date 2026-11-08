use state_lens::{
    analysis::Engine,
    model::Span,
    state::{extract, OperationKind, Relationship},
};
use tree_sitter::Parser;

fn graph(source: &str) -> state_lens::state::StateGraph {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    extract(
        tree.root_node(),
        "file:///case.ts",
        source,
        |name, _| match name {
            "d" => Some(Span { start: 4, end: 5 }),
            "other" => Some(Span { start: 0, end: 1 }),
            _ => None,
        },
    )
}

#[test]
fn branch_join_retains_both_assignments_and_overwrite_kills_them() {
    let source = "let d = 1; if (flag) { d = 0; } else { d = 2; } use(d); d = 3; use(d);";
    let graph = graph(source);
    let reads: Vec<_> = graph
        .operations
        .iter()
        .filter(|op| {
            op.kind == OperationKind::Read
                && &source[op.location.range.start..op.location.range.end] == "d"
        })
        .collect();
    let incoming = |id| {
        graph
            .edges
            .iter()
            .filter(|edge| edge.to == id && edge.relationship == Relationship::ReachingAssignment)
            .count()
    };
    assert_eq!(incoming(reads[0].id), 2);
    assert_eq!(incoming(reads[1].id), 1);
}

#[test]
fn candidate_slice_excludes_unrelated_operations() {
    let graph = graph("let d = 0; let other = 4; use(d);");
    let origin = graph
        .operations
        .iter()
        .find(|op| op.kind == OperationKind::Write && op.binding == Some(Span { start: 4, end: 5 }))
        .unwrap();
    let read = graph
        .operations
        .iter()
        .find(|op| op.kind == OperationKind::Read && op.binding == origin.binding)
        .unwrap();
    let region = graph.candidate_region(origin.id, read.id);
    assert_eq!(region.len(), 2);
    assert!(region.contains(&origin.id));
    assert!(region.contains(&read.id));
}

#[test]
fn traces_are_snapshot_scoped_and_do_not_invent_missing_guard_spans() {
    let mut engine = Engine::new().unwrap();
    let uri = "file:///case.ts";
    engine
        .set(
            uri.into(),
            "typescript",
            "let d = 0; let result = 1 / d;".into(),
            Some(1),
        )
        .unwrap();
    let report = engine.analyze();
    let finding = report.documents[0]
        .findings
        .iter()
        .find(|f| f.division_risk.is_some())
        .unwrap();
    let trace = finding.state_trace.as_ref().unwrap();
    assert!(trace.required_condition.is_some());
    assert!(trace.invalidations.is_empty());
    assert_eq!(trace.origins.len(), 1);
    let previous = trace.source_hash.clone();
    engine
        .set(
            uri.into(),
            "typescript",
            "let d = 0; let result = 2 / d;".into(),
            Some(2),
        )
        .unwrap();
    let next = engine.analyze();
    let trace = next.documents[0]
        .findings
        .iter()
        .find_map(|f| f.state_trace.as_ref())
        .unwrap();
    assert_ne!(previous, trace.source_hash);
}

#[test]
fn shadows_do_not_connect_to_outer_assignments() {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///case.ts".into(),
            "typescript",
            "let d = 9; function f(d: number) { return 1 / d; }".into(),
            None,
        )
        .unwrap();
    let report = engine.analyze();
    let trace = report.documents[0]
        .findings
        .iter()
        .find(|f| f.division_risk.is_some())
        .unwrap()
        .state_trace
        .as_ref()
        .unwrap();
    assert!(trace.origins.is_empty());
}

#[test]
fn updates_reach_reads_and_keep_the_previous_value_dependency() {
    let source = "let d = 1; d++; use(d);";
    let graph = graph(source);
    let read = graph
        .operations
        .iter()
        .rev()
        .find(|op| op.kind == OperationKind::Read && op.binding == Some(Span { start: 4, end: 5 }))
        .unwrap();
    let write_id = graph
        .edges
        .iter()
        .find(|edge| edge.to == read.id && edge.relationship == Relationship::ReachingAssignment)
        .unwrap()
        .from;
    let write = &graph.operations[write_id];
    assert_eq!(
        &source[write.location.range.start..write.location.range.end],
        "d"
    );
    assert!(write.location.range.start > 4);
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.to == write_id && edge.relationship == Relationship::ValueDependency));
}

#[test]
fn returned_branch_does_not_reach_later_use() {
    let source = "let d = 1; if (flag) { d = 0; return; } use(d);";
    let graph = graph(source);
    let read = graph
        .operations
        .iter()
        .rev()
        .find(|op| op.kind == OperationKind::Read && op.binding == Some(Span { start: 4, end: 5 }))
        .unwrap();
    let definitions: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.to == read.id && edge.relationship == Relationship::ReachingAssignment)
        .collect();
    assert_eq!(definitions.len(), 1);
    assert_eq!(
        graph.operations[definitions[0].from].location.range.start,
        4
    );
}

#[test]
fn canonical_contract_roundtrips_and_rejects_invalid_references() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/state-contract.json")).unwrap();
    for value in fixture["graphs"].as_array().unwrap() {
        let graph: state_lens::state::StateGraph = serde_json::from_value(value.clone()).unwrap();
        graph.validate().unwrap();
        assert_eq!(serde_json::to_value(&graph).unwrap(), *value);
        let mut invalid = graph.clone();
        invalid.operations[0].evidence.push("missing".into());
        assert!(invalid.validate().is_err());
        let mut invalid = graph.clone();
        invalid.operations[0].location.range.end = usize::MAX;
        assert!(invalid.validate().is_err());
        let mut invalid = graph.clone();
        invalid.operations.push(invalid.operations[0].clone());
        assert!(invalid.validate().is_err());
        let mut invalid = graph.clone();
        invalid.generation += 1;
        assert!(invalid.validate().is_err());
        let mut invalid = graph.clone();
        invalid.contract_version += 1;
        assert!(invalid.validate().is_err());
    }
    for value in fixture["traces"].as_array().unwrap() {
        let trace: state_lens::state::StateTrace = serde_json::from_value(value.clone()).unwrap();
        let graph = serde_json::from_value(fixture["graphs"][0].clone()).unwrap();
        trace.validate(&graph).unwrap();
        assert_eq!(serde_json::to_value(trace).unwrap(), *value);
    }
}

#[test]
fn dependency_edits_invalidate_snapshot_qualified_evidence() {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///a.ts".into(),
            "typescript",
            "let d = 0; let result = 1 / d;".into(),
            Some(1),
        )
        .unwrap();
    engine
        .set(
            "file:///b.ts".into(),
            "typescript",
            "let value = 1;".into(),
            Some(1),
        )
        .unwrap();
    let first = engine.analyze();
    let graph = first
        .documents
        .iter()
        .find(|d| d.uri == "file:///a.ts")
        .unwrap()
        .state_graph
        .as_ref()
        .unwrap();
    graph.validate().unwrap();
    let identity = graph.operations[0].identity.clone();
    let snapshot = graph.snapshot_id.clone();
    assert!(first
        .documents
        .iter()
        .all(|d| d.state_graph.as_ref().unwrap().snapshot_id == snapshot));
    let trace = first
        .documents
        .iter()
        .find(|d| d.uri == "file:///a.ts")
        .unwrap()
        .findings
        .iter()
        .find_map(|f| f.state_trace.clone())
        .unwrap();
    trace.validate(graph).unwrap();
    engine
        .set(
            "file:///b.ts".into(),
            "typescript",
            "let value = 2;".into(),
            Some(2),
        )
        .unwrap();
    let second = engine.analyze();
    let next = second
        .documents
        .iter()
        .find(|d| d.uri == "file:///a.ts")
        .unwrap()
        .state_graph
        .as_ref()
        .unwrap();
    assert_eq!(graph.source_hash, next.source_hash);
    assert_ne!(identity, next.operations[0].identity);
    assert!(trace.validate(next).is_err());
    assert_eq!(
        serde_json::to_value(&second).unwrap(),
        serde_json::to_value(engine.analyze()).unwrap()
    );
}

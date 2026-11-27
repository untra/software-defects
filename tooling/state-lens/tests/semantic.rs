use serde_json::Value;
use state_lens::semantic::analyze;
fn events(language: &str, source: &str) -> Vec<Value> {
    analyze(language, source).unwrap()["events"]
        .as_array()
        .unwrap()
        .clone()
}
#[test]
fn go_shadowing_and_assignment_have_distinct_binding_relationships() {
    for (operator, equal) in [(":=", false), ("=", true)] {
        let source = format!("package p; func f() int {{ var port int; if true {{ port {operator} 2; _ = port }}; return port }}");
        let facts = events("go", &source);
        let assignment = facts
            .iter()
            .find(|e| e["name"] == "port" && !e["previous_binding"].is_null())
            .unwrap();
        let consumer = facts.iter().find(|e| e["kind"] == "return_read").unwrap();
        assert_eq!(assignment["binding"] == consumer["binding"], equal);
    }
}
#[test]
fn mixed_go_declaration_reuses_current_scope_binding() {
    let facts = events(
        "go",
        "package p; func f() int { x := 1; x, y := 2, 3; _ = y; return x }",
    );
    let x: Vec<_> = facts
        .iter()
        .filter(|e| e["kind"] == "assignment" && e["name"] == "x")
        .collect();
    assert_eq!(x[0]["binding"], x[1]["binding"]);
    assert_eq!(x[1]["declares"], false);
}
#[test]
fn nested_copy_separates_only_the_copied_object() {
    let source = "const prod = { db: { host: 'a', nested: { value: 1 } } }; const test = { ...prod, db: { ...prod.db } }; test.db.host = 'b'; console.log(prod.db.host); test.db.nested.value = 2; console.log(prod.db.nested.value);";
    let facts = events("javascript", source);
    let access = |path: &str, kind: &str| {
        facts
            .iter()
            .find(|e| e["path"] == path && e["kind"] == kind)
            .unwrap()
    };
    assert_ne!(
        access("test.db.host", "write")["object"],
        access("prod.db.host", "read")["object"]
    );
    assert_eq!(
        access("test.db.nested.value", "write")["object"],
        access("prod.db.nested.value", "read")["object"]
    );
}
#[test]
fn spread_order_can_restore_aliasing() {
    let facts = events("javascript", "const prod = { db: { host: 'a' } }; const test = { db: { ...prod.db }, ...prod }; test.db.host = 'b'; console.log(prod.db.host);");
    let write = facts.iter().find(|e| e["kind"] == "write").unwrap();
    let read = facts
        .iter()
        .find(|e| e["path"] == "prod.db.host" && e["kind"] == "read")
        .unwrap();
    assert_eq!(write["object"], read["object"]);
}
#[test]
fn getters_and_unknown_calls_block_semantic_validation() {
    for source in [
        "const x = { get host() { return 'a'; } };",
        "const x = {}; mutate(x);",
    ] {
        assert!(!analyze("javascript", source).unwrap()["unsupported"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}

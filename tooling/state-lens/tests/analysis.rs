use state_lens::{
    analysis::Engine,
    model::{byte_offset, position, Effect, Position},
};
fn engine(language: &str, source: &str) -> Engine {
    let mut e = Engine::new().unwrap();
    e.set(
        format!(
            "file:///project/main.{}",
            match language {
                "typescript" => "ts",
                "rust" => "rs",
                _ => "go",
            }
        ),
        language,
        source.into(),
        Some(1),
    )
    .unwrap();
    e
}
#[test]
fn all_queries_compile() {
    Engine::new().unwrap();
}
#[test]
fn pure_functions_remain_uncolored() {
    for (language, source) in [
        (
            "typescript",
            "function add(a: number, b: number) { return a + b; }",
        ),
        (
            "go",
            "package main\nfunc add(a int,b int) int { return a+b }",
        ),
        ("rust", "fn add(a: i32,b:i32)->i32 { a+b }"),
    ] {
        let report = engine(language, source).analyze();
        assert!(
            report.documents[0].findings.is_empty(),
            "{language}: {:?}",
            report.documents[0].findings
        );
    }
}
#[test]
fn clock_effect_propagates_and_is_removed() {
    let mut e = engine(
        "typescript",
        "function clock() { return Date.now(); } function caller() { return clock(); }",
    );
    assert!(e.analyze().documents[0]
        .findings
        .iter()
        .any(|f| f.origin == "propagated" && f.effect == Effect::Nondeterminism));
    e.set(
        "file:///project/main.ts".into(),
        "typescript",
        "function clock() { return 1; } function caller() { return clock(); }".into(),
        Some(2),
    )
    .unwrap();
    assert!(e.analyze().documents[0].findings.is_empty());
}
#[test]
fn local_mutation_does_not_propagate() {
    let mut e = engine(
        "typescript",
        "function count() { let n=0; n++; return n; } function caller() { return count(); }",
    );
    let report = e.analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::LocalMutation));
    assert!(report.documents[0]
        .findings
        .iter()
        .all(|f| f.origin == "direct"));
}
#[test]
fn shadowed_clock_is_not_modeled() {
    let report = engine(
        "typescript",
        "function read(Date: { now: () => number }) { return Date.now(); }",
    )
    .analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .all(|f| f.effect != Effect::Nondeterminism));
    assert!(!report.documents[0].unresolved.is_empty());
}
#[test]
fn shadowed_function_is_not_resolved() {
    let report=engine("typescript","function read() { return Date.now(); } function call(read: () => number) { return read(); }").analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .all(|f| f.origin == "direct"));
}
#[test]
fn imported_alias_and_unsaved_edit_invalidate_callers() {
    let mut e = engine(
        "typescript",
        "import { clock as tick } from './clock'; function now() { return tick(); }",
    );
    e.set(
        "file:///project/clock.ts".into(),
        "typescript",
        "export function clock() { return Date.now(); }".into(),
        Some(1),
    )
    .unwrap();
    assert!(e
        .analyze()
        .documents
        .iter()
        .find(|d| d.uri.ends_with("main.ts"))
        .unwrap()
        .findings
        .iter()
        .any(|f| f.origin == "propagated"));
    e.set(
        "file:///project/clock.ts".into(),
        "typescript",
        "export function clock() { return 0; }".into(),
        Some(2),
    )
    .unwrap();
    assert!(e.analyze().documents.iter().all(|d| d.findings.is_empty()));
}
#[test]
fn global_reads_and_argument_writes_are_distinct() {
    let report=engine("typescript","let rate=2; function read() { return rate; } function update(order: {n:number}) { order.n=3; }").analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::ExternalRead));
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::ArgumentMutation(0)));
}
#[test]
fn argument_effect_is_substituted_at_call_site() {
    let report=engine("typescript","function update(order: {n:number}) { order.n=3; } function caller(input: {n:number}) { update(input); }").analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.origin == "propagated" && f.effect == Effect::ArgumentMutation(0)));
}
#[test]
fn fresh_local_argument_contains_mutation() {
    let report=engine("typescript","function update(order: {n:number}) { order.n=3; } function caller() { const local={n:0}; update(local); } function outer(){caller();}").analyze();
    let functions = &report.documents[0].functions;
    let outer = functions.iter().find(|f| f.name == "outer").unwrap();
    assert!(outer.effects.is_empty());
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.origin == "propagated" && f.effect == Effect::LocalMutation));
}
#[test]
fn recursion_terminates_with_evidence() {
    let report = engine(
        "typescript",
        "function a() { return b(); } function b() { a(); return Date.now(); }",
    )
    .analyze();
    assert!(report.documents[0].findings.len() < 12);
    assert!(report.documents[0]
        .functions
        .iter()
        .all(|f| f.effects.contains(&Effect::Nondeterminism)));
}
#[test]
fn closure_creation_does_not_execute_closure() {
    let report = engine(
        "typescript",
        "function factory() { return () => Date.now(); } function caller(){return factory();}",
    )
    .analyze();
    assert!(report.documents[0]
        .functions
        .iter()
        .find(|f| f.name == "factory")
        .unwrap()
        .effects
        .is_empty());
    assert!(report.documents[0]
        .functions
        .iter()
        .find(|f| f.name == "caller")
        .unwrap()
        .effects
        .is_empty());
}
#[test]
fn unicode_crlf_positions_roundtrip() {
    let source = "// 😀\r\nconst café = '🚦';\r\n";
    for (byte, _) in source.char_indices().filter(|(_, ch)| *ch != '\n') {
        let p = position(source, byte);
        assert_eq!(byte_offset(source, p), Some(byte));
    }
    assert_eq!(
        byte_offset(
            source,
            Position {
                line: 0,
                character: 4
            }
        ),
        None
    );
}
#[test]
fn incomplete_syntax_is_explicit() {
    assert!(
        engine("typescript", "function broken( {")
            .analyze()
            .documents[0]
            .parse_errors
    );
}
#[test]
fn go_import_alias_clock() {
    let report=engine("go","package main\nimport clock \"time\"\nfunc read() { clock.Now() }\nfunc caller() { read() }").analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.origin == "propagated" && f.effect == Effect::Nondeterminism));
}
#[test]
fn rust_clock_import() {
    let report = engine(
        "rust",
        "use std::time::SystemTime; fn read(){SystemTime::now();} fn caller(){read();}",
    )
    .analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.origin == "propagated" && f.effect == Effect::Nondeterminism));
}
#[test]
fn typed_mutating_standard_methods() {
    for (language, source) in [
        (
            "typescript",
            "function median(values: number[]){ values.sort(); }",
        ),
        ("rust", "fn median(values: &mut [i32]){ values.sort(); }"),
    ] {
        assert!(
            engine(language, source).analyze().documents[0]
                .findings
                .iter()
                .any(|f| f.effect == Effect::ArgumentMutation(0)),
            "{language}"
        );
    }
}
#[test]
fn unknown_method_not_assumed_mutating_or_pure() {
    let report = engine(
        "typescript",
        "function call(service: Service){service.sort();}",
    )
    .analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .all(|f| !f.effect.observable()));
    assert!(!report.documents[0].unresolved.is_empty());
}

#[test]
fn unknown_downstream_behavior_reaches_callers() {
    let report=engine("typescript","function inner(service: Service){service.run();} function outer(service: Service){inner(service);}").analyze();
    let doc = &report.documents[0];
    let outer = doc.functions.iter().find(|f| f.name == "outer").unwrap();
    assert!(doc.unresolved.iter().any(|u| outer.range.contains(u.range)));
}
#[test]
fn methods_and_block_scoped_functions_are_not_guessed_by_name() {
    for source in [
        "class Reader { read(){return Date.now();} } function caller(){read();}",
        "function outer(){ { function inner(){return Date.now();} } inner(); }",
    ] {
        let report = engine("typescript", source).analyze();
        assert!(report.documents[0]
            .findings
            .iter()
            .all(|f| f.origin == "direct"));
    }
}
#[test]
fn callable_local_closure_propagates_only_when_called() {
    let report = engine(
        "typescript",
        "function outer(){const clock = () => Date.now(); return clock();}",
    )
    .analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.origin == "propagated" && f.effect == Effect::Nondeterminism));
}
#[test]
fn function_declarations_shadow_standard_globals() {
    let report = engine(
        "typescript",
        "function Date(){} function use(){return Date.now();}",
    )
    .analyze();
    assert!(!report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::Nondeterminism));
}

#[test]
fn owned_rust_parameter_mutation_is_local() {
    let report = engine("rust", "fn order(mut values: Vec<i32>){values.sort();}").analyze();
    assert!(report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::LocalMutation));
    assert!(!report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::ArgumentMutation(0)));
}
#[test]
fn lexical_shadowing_applies_before_initialization() {
    let report = engine(
        "typescript",
        "function use(){Date.now(); const Date={now:()=>0};}",
    )
    .analyze();
    assert!(!report.documents[0]
        .findings
        .iter()
        .any(|f| f.effect == Effect::Nondeterminism));
}
#[test]
fn multi_file_go_and_rust_resolution() {
    let root = std::env::temp_dir().join(format!("state-lens-resolution-{}", std::process::id()));
    std::fs::create_dir_all(root.join("clock")).unwrap();
    std::fs::write(root.join("go.mod"), "module example.local/app\n\ngo 1.22\n").unwrap();
    std::fs::write(
        root.join("main.go"),
        "package main\nimport timer \"example.local/app/clock\"\nfunc main(){timer.Read()}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("clock/clock.go"),
        "package clock\nimport \"time\"\nfunc Read(){time.Now()}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("main.rs"),
        "mod clock; use crate::clock::read; fn main(){read();}",
    )
    .unwrap();
    std::fs::write(
        root.join("clock.rs"),
        "pub fn read(){std::time::SystemTime::now();}",
    )
    .unwrap();
    let mut e = Engine::new().unwrap();
    e.load(&root).unwrap();
    let report = e.analyze();
    for suffix in ["main.go", "main.rs"] {
        assert!(
            report
                .documents
                .iter()
                .find(|d| d.uri.ends_with(suffix))
                .unwrap()
                .findings
                .iter()
                .any(|f| f.origin == "propagated" && f.effect == Effect::Nondeterminism),
            "{suffix}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn sidecar_expectations_are_exact_and_hash_guarded() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let path = root.join("expectations.json");
    let scores = state_lens::corpus::score(&path).unwrap();
    assert!(scores
        .iter()
        .all(|score| score.false_positive == 0 && score.false_negative == 0));
    let mut stale: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for case in stale["cases"].as_array_mut().unwrap() {
        case["file"] = root
            .join(case["file"].as_str().unwrap())
            .to_str()
            .unwrap()
            .into();
    }
    stale["cases"][0]["source_hash"] = "outdated".into();
    let temporary =
        std::env::temp_dir().join(format!("state-lens-stale-{}.json", std::process::id()));
    std::fs::write(&temporary, serde_json::to_string(&stale).unwrap()).unwrap();
    assert!(state_lens::corpus::score(&temporary)
        .unwrap_err()
        .to_string()
        .contains("stale expectations"));
    std::fs::remove_file(temporary).unwrap();
}

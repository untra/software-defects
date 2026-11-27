use state_lens::{
    analysis::Engine,
    model::{DocumentReport, Finding},
};

const RESOLUTION: &str = "unstructured-input-output";
const RETURNS: &str = "missing-undefined-returns";

#[test]
fn disabling_an_input_rule_removes_its_dependent_compounds() {
    let mut engine = Engine::new().unwrap();
    engine.set("file:///sample.js".into(), "javascript", "function title(record) { if (typeof record.name !== 'string') throw new Error('name'); record.name = null; return record.name.toUpperCase(); }".into(), Some(1)).unwrap();
    assert!(engine.analyze().documents[0]
        .findings
        .iter()
        .any(|f| f.mechanism.as_deref() == Some("guard-invalidated")));
    engine.set_disabled_checks(
        ["lack-of-input-validation".to_owned()]
            .into_iter()
            .collect(),
    );
    assert!(!engine.analyze().documents[0]
        .findings
        .iter()
        .any(|f| f.mechanism.as_deref() == Some("guard-invalidated")));
}

#[test]
fn direct_input_passthrough_propagates_without_propagating_every_boundary() {
    let source = "function identity(value: unknown) { return value; } function read(value: unknown) { console.log(value); } function use() { const value = identity(1); read(2); return value; }";
    let document = report("typescript", source);
    let passthrough = document
        .findings
        .iter()
        .find(|f| f.origin == "propagated" && f.mechanism.as_deref() == Some("boundary-output"))
        .unwrap();
    assert_eq!(
        &source[passthrough.trigger.start..passthrough.trigger.end],
        "identity(1)"
    );
    assert!(passthrough
        .blast_radius
        .iter()
        .any(|range| range.start == source.rfind("value;").unwrap()));
    assert!(!document.findings.iter().any(|f| f.origin == "propagated"
        && &source[f.trigger.start..f.trigger.end] == "read(2)"
        && f.family == "unstructured-state"));
    let trace = passthrough.state_trace.as_ref().unwrap();
    trace
        .validate(document.state_graph.as_ref().unwrap())
        .unwrap();
}

fn report(language: &str, source: &str) -> DocumentReport {
    let mut engine = Engine::new().unwrap();
    engine
        .set("file:///sample".into(), language, source.into(), Some(1))
        .unwrap();
    let document = engine.analyze().documents.remove(0);
    assert!(!document.parse_errors, "{language}: {source}");
    document
}

fn producers(document: &DocumentReport) -> Vec<&Finding> {
    document
        .findings
        .iter()
        .filter(|finding| finding.resolution == RESOLUTION && finding.defect == RETURNS)
        .collect()
}

#[test]
fn boundaries_are_candidates_in_all_languages_with_snapshot_evidence() {
    for (language, source) in [
        (
            "javascript",
            "function consume(payload) { return payload; }",
        ),
        (
            "typescript",
            "function consume(payload: unknown) { return payload; }",
        ),
        ("python", "def consume(payload):\n    return payload\n"),
        (
            "go",
            "package sample\nfunc consume(payload []byte) []byte { return payload }",
        ),
        ("rust", "fn consume(payload: &[u8]) -> &[u8] { payload }"),
    ] {
        let document = report(language, source);
        let finding = document
            .findings
            .iter()
            .find(|f| {
                f.resolution == RESOLUTION && f.mechanism.as_deref() == Some("boundary-input")
            })
            .expect(language);
        assert_eq!(finding.family, "unstructured-state");
        assert!(source[finding.trigger.start..finding.trigger.end].contains("payload"));
        assert!(
            finding.related_cwes.is_empty(),
            "a boundary is not an established CWE"
        );
        let trace = finding.state_trace.as_ref().unwrap();
        assert_eq!(trace.source_hash, document.source_hash);
        assert!(!trace.assumptions.is_empty());
        assert!(!trace.origins.is_empty());
    }
}

#[test]
fn branch_fallthrough_is_owned_by_the_producer_and_disappears_when_completed() {
    for (language, broken, fixed) in [
        (
            "javascript",
            "function price(member) { if (member) { return 5; } }",
            "function price(member) { if (member) { return 5; } return 10; }",
        ),
        (
            "typescript",
            "function price(member: boolean): number { if (member) { return 5; } }",
            "function price(member: boolean): number { if (member) { return 5; } return 10; }",
        ),
        (
            "python",
            "def price(member):\n    if member:\n        return 5\n",
            "def price(member):\n    if member:\n        return 5\n    return 10\n",
        ),
    ] {
        let document = report(language, broken);
        let findings = producers(&document);
        assert_eq!(findings.len(), 1, "{language}");
        let producer = findings[0];
        assert_eq!(producer.mechanism.as_deref(), Some("producer-output"));
        assert!(broken[producer.trigger.start..producer.trigger.end].contains("price"));
        assert!(producer.function.unwrap().contains(producer.trigger));
        assert!(producers(&report(language, fixed)).is_empty(), "{language}");
    }
}

#[test]
fn valid_optional_outputs_and_commands_do_not_become_missing_returns() {
    for (language, source) in [
        (
            "typescript",
            "function find(found: boolean): number | undefined { if (found) return 1; }",
        ),
        (
            "typescript",
            "function save(): void { console.log('saved'); }",
        ),
        (
            "typescript",
            "async function save(): Promise<void> { console.log('saved'); }",
        ),
        (
            "python",
            "def find(found) -> int | None:\n    if found:\n        return 1\n",
        ),
        ("python", "def save() -> None:\n    print('saved')\n"),
        ("javascript", "function save() { console.log('saved'); }"),
        ("javascript", "function* values() { yield 1; return 2; }"),
        ("python", "def values():\n    yield 1\n    return 2\n"),
        ("go", "package sample\nfunc save() {}"),
        ("go", "package sample\nfunc save() { return }"),
        ("rust", "fn save() {}"),
        ("rust", "fn save() { return; }"),
        ("rust", "fn save() -> () { return; }"),
    ] {
        assert!(
            producers(&report(language, source)).is_empty(),
            "{language}: {source}"
        );
    }
}

#[test]
fn optional_result_consumers_are_outside_the_producer_rule() {
    for (language, source) in [
        (
            "javascript",
            "const user = users.find(isAdmin); console.log(user.name);",
        ),
        (
            "typescript",
            "const user = users.find(isAdmin); console.log(user!.name);",
        ),
        ("python", "stock = {}\nprint(stock.get('pen') + 1)\n"),
        (
            "go",
            "package sample\nfunc use() { value, _ := lookup(); _ = value.Name }",
        ),
        (
            "rust",
            "fn use_value() { let value = lookup().unwrap(); println!(\"{:?}\", value); }",
        ),
    ] {
        assert!(
            producers(&report(language, source)).is_empty(),
            "{language}"
        );
    }
}

#[test]
fn nested_function_returns_do_not_complete_the_outer_contract() {
    for (language, source) in [
        (
            "typescript",
            "function outer(): number { function inner() { return 1; } inner(); }",
        ),
        (
            "python",
            "def outer() -> int:\n    def inner():\n        return 1\n    inner()\n",
        ),
    ] {
        let document = report(language, source);
        let findings = producers(&document);
        assert_eq!(findings.len(), 1, "{language}");
        assert!(source[findings[0].trigger.start..findings[0].trigger.end].contains("outer"));
    }
}

#[test]
fn returning_or_raising_satisfies_the_output_contract() {
    for (language, source) in [
        ("typescript", "function requireValue(ok: boolean): number { if (ok) return 1; throw new Error('missing'); }"),
        ("javascript", "function requireValue(ok) { if (ok) return 1; throw new Error('missing'); }"),
        ("python", "def require_value(ok) -> int:\n    if ok:\n        return 1\n    raise ValueError('missing')\n"),
    ] {
        assert!(producers(&report(language, source)).is_empty(), "{language}");
    }
}

#[test]
fn assertions_are_precise_candidates_and_const_assertions_are_excluded() {
    for (language, source, expected) in [
        (
            "typescript",
            "// 🧭\nconst value = payload as Invoice;",
            "payload as Invoice",
        ),
        (
            "go",
            "package sample\nfunc f(value any) { _ = value.(Invoice) }",
            "value.(Invoice)",
        ),
        (
            "python",
            "from typing import cast\nvalue = cast(Invoice, payload)\n",
            "cast(Invoice, payload)",
        ),
    ] {
        let document = report(language, source);
        let finding = document
            .findings
            .iter()
            .find(|f| {
                f.resolution == RESOLUTION && f.mechanism.as_deref() == Some("unchecked-contract")
            })
            .expect(language);
        assert_eq!(
            &source[finding.trigger.start..finding.trigger.end],
            expected
        );
        assert!(finding
            .blast_radius
            .iter()
            .all(|span| source.is_char_boundary(span.start) && source.is_char_boundary(span.end)));
    }
    let document = report(
        "typescript",
        "const directions = ['north', 'south'] as const;",
    );
    assert!(!document.findings.iter().any(
        |f| f.resolution == RESOLUTION && f.mechanism.as_deref() == Some("unchecked-contract")
    ));
}

#[test]
fn shadowed_bindings_do_not_inherit_the_outer_input_trace() {
    let source =
        "function outer(payload) { { const payload = 1; console.log(payload); } return payload; }";
    let document = report("javascript", source);
    let boundary = document
        .findings
        .iter()
        .find(|f| f.resolution == RESOLUTION && f.mechanism.as_deref() == Some("boundary-input"))
        .unwrap();
    let shadow_use = source.find("console.log(payload)").unwrap() + "console.log(".len();
    let outer_use = source.rfind("payload").unwrap();
    assert!(!boundary.blast_radius.iter().any(|r| r.start == shadow_use));
    assert!(boundary.blast_radius.iter().any(|r| r.start == outer_use));
}

#[test]
fn resource_operations_are_candidates_even_when_a_limit_is_present() {
    for (language, source) in [
        ("javascript", "function batch(rows) { if (rows.length > 10) throw new Error('large'); return rows.map(row => row); }"),
        ("typescript", "function batch(rows: string[]) { if (rows.length > 10) throw new Error('large'); return rows.map(row => row); }"),
        ("python", "def batch(rows):\n    if len(rows) > 10:\n        raise ValueError('large')\n    return list(rows)\n"),
        ("go", "package sample\nfunc allocate(count int) []byte { if count > 10 { return nil }; return make([]byte, count) }"),
        ("rust", "fn allocate(count: usize) -> Vec<u8> { Vec::with_capacity(count) }"),
    ] {
        let document = report(language, source);
        let finding = document.findings.iter().find(|f| f.resolution == RESOLUTION && f.mechanism.as_deref() == Some("resource-input")).expect(language);
        assert!(finding.explanation.contains("candidate"));
        assert!(finding.related_cwes.is_empty());
    }
}

#[test]
fn async_producer_fallthrough_has_the_same_result_obligation() {
    for (language, source) in [
        (
            "javascript",
            "async function quote(member) { if (member) return 5; }",
        ),
        (
            "typescript",
            "async function quote(member: boolean): Promise<number> { if (member) return 5; }",
        ),
        (
            "python",
            "async def quote(member):\n    if member:\n        return 5\n",
        ),
    ] {
        assert_eq!(producers(&report(language, source)).len(), 1, "{language}");
    }
}

#[test]
fn direct_call_propagates_the_producer_origin() {
    let source = "function quote(member) { if (member) return 5; } function total() { return quote(false) * 2; }";
    let document = report("javascript", source);
    let direct = producers(&document);
    assert_eq!(direct.len(), 1);
    let propagated = document
        .findings
        .iter()
        .find(|f| f.defect == RETURNS && f.origin == "propagated")
        .expect("resolved call retains the producer risk");
    assert_eq!(propagated.resolution, "resolved-call");
    assert!(!propagated.call_path.is_empty());
    assert!(propagated.state_trace.is_some());
    assert!(source[propagated.trigger.start..propagated.trigger.end].contains("quote(false)"));
}

#[test]
fn mutation_after_a_shape_guard_connects_to_the_affected_use() {
    let source = "function title(record) { if (typeof record.name !== 'string') throw new Error('name'); record.name = null; return record.name.toUpperCase(); }";
    let document = report("javascript", source);
    let compound = document
        .findings
        .iter()
        .find(|f| f.mechanism.as_deref() == Some("guard-invalidated"))
        .expect("guard, write and use should share one input identity");
    assert_eq!(compound.family, "compound-defects");
    let trace = compound.state_trace.as_ref().unwrap();
    assert!(!trace.invalidations.is_empty());
    assert!(!trace.affected_uses.is_empty());
    assert!(source[compound.trigger.start..compound.trigger.end].contains("record.name"));
    assert!(compound.compound.is_some());
}

#[test]
fn unrelated_mutation_does_not_invalidate_the_checked_path() {
    let source = "function title(record) { if (typeof record.name !== 'string') throw new Error('name'); record.count = 0; return record.name.toUpperCase(); }";
    let document = report("javascript", source);
    assert!(!document
        .findings
        .iter()
        .any(|f| f.mechanism.as_deref() == Some("guard-invalidated")));
}

#[test]
fn await_without_a_connected_write_does_not_invalidate_a_guard() {
    let source = "async function title(record) { if (typeof record.name !== 'string') throw new Error('name'); await checkpoint(); return record.name.toUpperCase(); }";
    let document = report("javascript", source);
    assert!(!document
        .findings
        .iter()
        .any(|f| f.mechanism.as_deref() == Some("guard-invalidated")));
}

#[test]
fn revalidation_after_mutation_breaks_the_old_guard_relationship() {
    let source = "function title(record) { if (typeof record.name !== 'string') throw new Error('name'); record.name = null; if (typeof record.name !== 'string') throw new Error('name'); return record.name.toUpperCase(); }";
    let document = report("javascript", source);
    assert!(!document
        .findings
        .iter()
        .any(|f| f.mechanism.as_deref() == Some("guard-invalidated")));
}

#[test]
fn explicit_absence_violates_a_declared_required_output() {
    for (language, source) in [
        (
            "typescript",
            "function quote(): number { return undefined; }",
        ),
        (
            "typescript",
            "async function quote(): Promise<number> { return undefined; }",
        ),
        ("python", "def quote() -> int:\n    return None\n"),
    ] {
        assert_eq!(producers(&report(language, source)).len(), 1, "{language}");
    }
}

#[test]
fn static_languages_expose_empty_output_candidates_without_claiming_fallthrough() {
    for (language, source) in [
        (
            "go",
            "package sample\nfunc required() (*int, error) { return nil, nil }",
        ),
        ("rust", "fn lookup() -> Option<i32> { return None; }"),
    ] {
        let document = report(language, source);
        let findings = producers(&document);
        assert_eq!(findings.len(), 1, "{language}");
        assert!(findings[0].explanation.contains("candidate"));
        assert!(findings[0].related_cwes.is_empty());
    }
}

#[test]
fn return_literal_contents_are_not_missing_output_syntax() {
    for (language, source) in [
        (
            "go",
            "package sample\nfunc flavor() string { return \"vanilla\" }",
        ),
        (
            "rust",
            "fn label() -> &'static str { return \"None Ok(\"; }",
        ),
    ] {
        assert!(
            producers(&report(language, source)).is_empty(),
            "{language}"
        );
    }
}

#[test]
fn editing_a_producer_removes_stale_direct_and_propagated_findings() {
    let mut engine = Engine::new().unwrap();
    let uri = "file:///pricing.ts".to_string();
    let original = "function quote(member: boolean): number { if (member) return 5; } function total() { return quote(false) * 2; }";
    engine
        .set(uri.clone(), "typescript", original.into(), Some(1))
        .unwrap();
    let first = engine.analyze().documents.remove(0);
    assert!(first
        .findings
        .iter()
        .any(|f| f.defect == RETURNS && f.origin == "propagated"));
    let fixed = "function quote(member: boolean): number { if (member) return 5; return 10; } function total() { return quote(false) * 2; }";
    engine
        .set(uri, "typescript", fixed.into(), Some(2))
        .unwrap();
    let second = engine.analyze().documents.remove(0);
    assert_ne!(first.source_hash, second.source_hash);
    assert!(!second.findings.iter().any(|f| f.defect == RETURNS));
    assert!(second.annotations.iter().all(|a| a
        .findings
        .iter()
        .all(|&index| index < second.findings.len())));
}

#[test]
fn shadowed_undefined_is_a_value_not_an_absent_result() {
    let source = "function quote(undefined: number): number { return undefined; }";
    assert!(producers(&report("typescript", source)).is_empty());
}

#[test]
fn unsupported_control_flow_never_establishes_a_missing_return_warning() {
    for source in [
        "function quote(): number { do { return 5; } while (false); }",
        "function quote(): number { price: { return 5; } }",
    ] {
        let document = report("typescript", source);
        for finding in producers(&document) {
            let trace = finding.state_trace.as_ref().unwrap();
            assert!(
                trace.requirement.is_none(),
                "unmodeled control flow is only a candidate"
            );
            assert_ne!(
                trace.evidence_level,
                state_lens::state::EvidenceLevel::FeasibleUnderModel
            );
        }
    }
}

#[test]
fn mutually_exclusive_alias_assignment_does_not_connect_a_write() {
    let source = "function title(record, other, flag) { if (typeof record.name !== 'string') throw new Error('name'); let alias = other; if (flag) { alias = record; } else { alias.name = null; } return record.name.toUpperCase(); }";
    let document = report("javascript", source);
    assert!(!document
        .findings
        .iter()
        .any(|f| f.mechanism.as_deref() == Some("guard-invalidated")));
}

#[test]
fn invalidating_a_size_guard_does_not_prove_quadratic_complexity() {
    let source = "function batch(rows: string[]) { if (rows.length > 10) throw new Error('large'); rows.push('extra'); return rows.map(row => row); }";
    let document = report("typescript", source);
    let compounds: Vec<_> = document
        .findings
        .iter()
        .filter(|f| f.mechanism.as_deref() == Some("guard-invalidated"))
        .collect();
    assert!(
        !compounds.is_empty(),
        "the size guard and following mutation are connected"
    );
    assert!(compounds.iter().all(|f| f.defect != "quadratic-blowup"));
}

#[test]
fn candidate_required_and_compound_traces_validate_against_their_snapshot() {
    for (language, source, mechanism) in [
        ("typescript", "function consume(payload: unknown) { return payload; }", "boundary-input"),
        ("typescript", "function quote(member: boolean): number { if (member) return 5; }", "producer-output"),
        ("javascript", "function title(record) { if (typeof record.name !== 'string') throw new Error('name'); record.name = null; return record.name.toUpperCase(); }", "guard-invalidated"),
    ] {
        let document = report(language, source);
        let graph = document.state_graph.as_ref().unwrap();
        let findings: Vec<_> = document.findings.iter().filter(|f| f.resolution == RESOLUTION && f.origin == "direct").collect();
        assert!(findings.iter().any(|f| f.mechanism.as_deref() == Some(mechanism)));
        for finding in findings {
            finding.state_trace.as_ref().unwrap().validate(graph).unwrap();
        }
        let old_trace = document.findings.iter().find(|f| f.mechanism.as_deref() == Some(mechanism)).unwrap().state_trace.as_ref().unwrap();
        let edited = report(language, &(source.to_owned() + "\n"));
        assert!(old_trace.validate(edited.state_graph.as_ref().unwrap()).is_err());
    }
}

#[test]
fn propagated_cross_document_trace_validates_against_the_origin_graph() {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///project/quote.ts".into(),
            "typescript",
            "export function quote(member: boolean): number { if (member) return 5; }".into(),
            Some(1),
        )
        .unwrap();
    engine
        .set(
            "file:///project/total.ts".into(),
            "typescript",
            "import { quote } from './quote'; function total() { return quote(false) * 2; }".into(),
            Some(1),
        )
        .unwrap();
    let snapshot = engine.analyze();
    let origin = snapshot
        .documents
        .iter()
        .find(|d| d.uri.ends_with("/quote.ts"))
        .unwrap();
    let caller = snapshot
        .documents
        .iter()
        .find(|d| d.uri.ends_with("/total.ts"))
        .unwrap();
    let propagated = caller
        .findings
        .iter()
        .find(|f| f.defect == RETURNS && f.origin == "propagated")
        .unwrap();
    let trace = propagated.state_trace.as_ref().unwrap();
    assert_eq!(trace.source_hash, origin.source_hash);
    trace
        .validate(origin.state_graph.as_ref().unwrap())
        .unwrap();
    assert!(trace
        .validate(caller.state_graph.as_ref().unwrap())
        .is_err());
}

#[test]
fn deserialization_sink_and_allowlist() {
    for language in ["javascript", "typescript"] {
        let bad = report(language, "const body = JSON.parse(raw); const user = {}; Object.assign(user, body);");
        assert!(bad.findings.iter().any(|f| f.defect == "insecure-deserialization" && f.mechanism.as_deref() == Some("decoded-to-sink")));
        let fixed = report(language, "const body = JSON.parse(raw); const user = {}; user.name = body.name;");
        assert!(!fixed.findings.iter().any(|f| f.defect == "insecure-deserialization" && f.assessment != state_lens::model::FindingAssessment::Usage));
    }
    let bad = report("python", "import pickle\ndef read(raw):\n    return pickle.loads(raw)\n");
    assert!(bad.findings.iter().any(|f| f.mechanism.as_deref() == Some("unsafe-loader")));
}

#[test]
fn runtime_type_candidates_preserve_explicit_operations() {
    for (language, bad, fixed, mechanism) in [
        ("javascript", "const x = '2'; console.log(x + 1);", "const x = '2'; console.log(Number(x) + 1);", "implicit-coercion"),
        ("python", "labels = {1: 'one'}\nlabels[True] = 'yes'\n", "labels = {1: 'one'}\nlabels['true'] = 'yes'\n", "bool-is-int"),
        ("python", "def qty(raw):\n    return raw or 50\n", "def qty(raw):\n    return 50 if raw is None else raw\n", "falsy-default"),
    ] {
        assert!(report(language, bad).findings.iter().any(|f| f.mechanism.as_deref() == Some(mechanism)), "{language}: {mechanism}");
        assert!(!report(language, fixed).findings.iter().any(|f| f.mechanism.as_deref() == Some(mechanism)), "{language}: fixed {mechanism}");
    }
}
#[test]
fn literal_index_bounds_and_safe_optional_access() {
    let bad = report("javascript", "const rows = []; console.log(rows[0].id);");
    assert!(bad.findings.iter().any(|f| f.mechanism.as_deref() == Some("unguarded-index")));
    let fixed = report("javascript", "const rows = []; console.log(rows[0]?.id ?? 'none');");
    assert!(!fixed.findings.iter().any(|f| f.defect == "index-out-of-bounds" && f.assessment != state_lens::model::FindingAssessment::Usage));
}

#[test]
fn independent_guards_and_stale_bounds_do_not_suppress_indexing() {
    for source in [
        "const rows = []; const other = [1]; if (other.length) console.log(rows[0].id);",
        "const rows = []; console.log(rows.length ? rows[3] : rows[0].id);",
    ] {
        assert!(report("javascript", source).findings.iter().any(|f| f.defect == "index-out-of-bounds" && f.assessment == state_lens::model::FindingAssessment::Candidate));
    }
    let fixed = report("python", "rows = []\nprint(rows[0] if rows else 'none')\n");
    assert!(!fixed.findings.iter().any(|f| f.defect == "index-out-of-bounds"));
}

#[test]
fn validated_result_is_distinct_from_an_ignored_validator_result() {
    let prefix = "import { z } from 'zod'; const schema = z.object({name: z.string()}); const body = JSON.parse(raw); const user = {}; ";
    let fixed = report("javascript", &(prefix.to_owned() + "const checked = schema.parse(body); Object.assign(user, checked);"));
    assert!(!fixed.findings.iter().any(|f| f.defect == "insecure-deserialization" && f.assessment != state_lens::model::FindingAssessment::Usage));
    let bad = report("javascript", &(prefix.to_owned() + "schema.parse(body); Object.assign(user, body);"));
    assert!(bad.findings.iter().any(|f| f.defect == "insecure-deserialization"));
}

#[test]
fn allowlisted_keys_cannot_be_rebound_before_a_merge() {
    let bad = report("javascript", "const incoming = JSON.parse(raw); const record = {}; for (let key of Object.keys(incoming)) { if (key === 'name') { key = '__proto__'; record[key] = incoming[key]; } }");
    assert!(bad.findings.iter().any(|f| f.mechanism.as_deref() == Some("prototype-key")));
    let unrelated = report("go", "package p\nimport \"encoding/json\"\ntype Privilege struct { Enabled bool }\ntype Label struct { Name string }\nfunc read(raw []byte) { var label Label; if err := json.Unmarshal(raw, &label); err != nil { return } }");
    assert!(!unrelated.findings.iter().any(|f| f.mechanism.as_deref() == Some("decoded-field-assignment")));
}

#[test]
fn matching_rust_downcast_does_not_flag_the_following_method_chain() {
    let fixed = report("rust", "use std::any::Any; fn main() { let stored: Box<dyn Any> = Box::new(12_i64); let qty = stored.downcast_ref::<i64>().copied().expect(\"type\"); }");
    assert!(!fixed.findings.iter().any(|f| f.mechanism.as_deref() == Some("runtime-downcast")));
}

#[test]
fn early_empty_guard_dominates_last_index_until_collection_is_reassigned() {
    let fixed = report("go", "package p\nfunc last(events []int) int { if len(events) == 0 { return 0 }; return events[len(events)-1] }");
    assert!(!fixed.findings.iter().any(|f| f.defect == "index-out-of-bounds"));
    let bad = report("go", "package p\nfunc last(events []int) int { if len(events) == 0 { return 0 }; events = nil; return events[len(events)-1] }");
    assert!(bad.findings.iter().any(|f| f.defect == "index-out-of-bounds"));
}

#[test]
fn inverted_or_optional_key_guards_do_not_establish_an_allowlist() {
    for condition in ["key !== 'name'", "key === 'name' || allowAll", "key === '__proto__'"] {
        let source = format!("const incoming = JSON.parse(raw); const record = {{}}; for (const key of Object.keys(incoming)) {{ if ({condition}) record[key] = incoming[key]; }}");
        assert!(report("javascript", &source).findings.iter().any(|f| f.mechanism.as_deref() == Some("prototype-key")), "{condition}");
    }
}

#[test]
fn pydantic_dump_supplies_the_validated_value_not_the_original_input() {
    let prefix = "import json\nfrom pydantic import BaseModel\nclass Body(BaseModel):\n    name: str\nraw = json.loads(payload)\nrecord = {}\n";
    let safe = report("python", &(prefix.to_owned() + "checked = Body.model_validate(raw).model_dump()\nrecord.update(checked)\n"));
    assert!(!safe.findings.iter().any(|f| f.defect == "insecure-deserialization" && f.assessment != state_lens::model::FindingAssessment::Usage));
    let bad = report("python", &(prefix.to_owned() + "Body.model_validate(raw)\nrecord.update(raw)\n"));
    assert!(bad.findings.iter().any(|f| f.mechanism.as_deref() == Some("decoded-to-sink")));
}

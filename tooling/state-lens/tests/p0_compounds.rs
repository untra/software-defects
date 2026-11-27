use state_lens::{analysis::Engine, model::FindingAssessment};

#[test]
fn mutex_relationship_triggers_belong_to_their_reported_function() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/shared-state/threading-mutexes");
    for file in [
        "go/01_lock_order_deadlock.go",
        "python/01_opposite_lock_order.py",
        "rust/01_lock_order_deadlock.rs",
    ] {
        let mut engine = Engine::new().unwrap();
        engine.load(&root.join(file)).unwrap();
        for finding in &engine.analyze().documents[0].findings {
            if let Some(function) = finding.function {
                assert!(
                    function.start <= finding.trigger.start && finding.trigger.end <= function.end,
                    "{file}: {}",
                    finding.rule_id
                );
            }
        }
    }
}

#[test]
fn overlapping_families_are_candidates_and_edits_remove_them() {
    let mut engine = Engine::new().unwrap();
    let uri = "file:///sample.js".to_string();
    engine.set(uri.clone(), "javascript", "const account = {}; function update(raw) { const body = JSON.parse(raw); Object.assign(account, body); }".into(), Some(1)).unwrap();
    let report = engine.analyze();
    let document = &report.documents[0];
    assert!(document.findings.iter().any(|f| f
        .compound
        .as_ref()
        .is_some_and(|c| c.class == "data-corruption")
        && f.assessment == FindingAssessment::Candidate));
    for finding in document
        .findings
        .iter()
        .filter(|f| f.resolution == "potential-overlap")
    {
        assert!(finding.evidence.iter().all(|e| document
            .findings
            .iter()
            .any(|base| base.rule_id == e.rule_id && base.trigger == e.range)));
    }
    engine
        .set(uri, "javascript", "const account = {};".into(), Some(2))
        .unwrap();
    assert!(!engine.analyze().documents[0]
        .findings
        .iter()
        .any(|f| f.resolution == "potential-overlap"));
}

#[test]
fn authored_compound_examples_gain_expected_family_intersections() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/compound-defects");
    for (class, defect, ingredient, file) in [
        (
            "nondeterminism",
            "stale-cache-heisenbug",
            "cache-invalidation",
            "01_shipping_quote.js",
        ),
        (
            "data-corruption",
            "untrusted-deserialization",
            "insecure-deserialization",
            "01_account_profile.js",
        ),
        ("runtime-errors", "off-by-one-crash", "index-out-of-bounds", "01_stock_labels.js"),
        (
            "nondeterminism",
            "pool-exhaustion-heisenbug",
            "resource-contention",
            "01_report_slot.js",
        ),
        (
            "data-corruption",
            "signedness-poisons-aggregate",
            "type-errors",
            "01_refund_balance.js",
        ),
    ] {
        let path = root.join(class).join(defect).join("javascript").join(file);
        let mut engine = Engine::new().unwrap();
        engine.load(&path).unwrap();
        assert!(
            engine.analyze().documents[0]
                .findings
                .iter()
                .any(|f| f.resolution == "potential-overlap" && f.compound.as_ref().is_some_and(|c| {
                    c.class == class && c.ingredients.iter().any(|e| e.rule_id.ends_with(&format!(".{ingredient}")))
                })),
            "{defect}"
        );
    }
}

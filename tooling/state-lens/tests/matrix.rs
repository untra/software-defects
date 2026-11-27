use serde_json::Value;
use state_lens::{analysis::Engine, catalog, model::Span};
use std::path::{Path, PathBuf};

const STATUSES: [&str; 5] = [
    "supported",
    "candidate-only",
    "syntax-candidate",
    "planned",
    "not-applicable",
];
const ROLES: [&str; 5] = ["failing", "fixed", "intentional", "independent", "breaker"];
const OBJECT_IDENTITY: &str = "object-identity";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn extension(language: &str) -> &'static str {
    match language {
        "javascript" => "js",
        "typescript" => "ts",
        "python" => "py",
        "go" => "go",
        "rust" => "rs",
        other => panic!("unknown language {other}"),
    }
}

fn directory(defect: &str, language: &str, mechanism: &str) -> PathBuf {
    root()
        .join("fixtures/rules")
        .join(defect)
        .join(language)
        .join(mechanism)
}

fn analyze(path: &Path, language: &str) -> (String, state_lens::model::DocumentReport) {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut engine = Engine::new().unwrap();
    let uri = url::Url::from_file_path(path).unwrap().to_string();
    engine.set(uri, language, source.clone(), Some(1)).unwrap();
    let report = engine.analyze();
    (source, report.documents.into_iter().next().unwrap())
}

fn anchor(source: &str, value: &Value, file: &Path) -> Span {
    let (text, occurrence) = match value {
        Value::String(text) => (text.as_str(), 1),
        Value::Object(o) => (
            o["text"].as_str().unwrap(),
            o.get("occurrence").and_then(Value::as_u64).unwrap_or(1) as usize,
        ),
        other => panic!("invalid anchor {other}"),
    };
    let positions: Vec<usize> = source.match_indices(text).map(|(i, _)| i).collect();
    if value.is_string() {
        assert_eq!(
            positions.len(),
            1,
            "{}: anchor {text:?} must be unique (use occurrence)",
            file.display()
        );
    }
    let start = *positions.get(occurrence - 1).unwrap_or_else(|| {
        panic!(
            "{}: anchor {text:?} occurrence {occurrence} missing",
            file.display()
        )
    });
    Span {
        start,
        end: start + text.len(),
    }
}

fn anchors(source: &str, value: &Value, file: &Path) -> Vec<Span> {
    let mut spans: Vec<Span> = value
        .as_array()
        .map(|a| a.iter().map(|v| anchor(source, v, file)).collect())
        .unwrap_or_default();
    spans.sort();
    spans
}

fn ranges(locations: &[state_lens::state::Location]) -> Vec<Span> {
    let mut spans: Vec<Span> = locations.iter().map(|l| l.range).collect();
    spans.sort();
    spans
}

fn check_expectation(file: &Path, language: &str, mechanism: &str, expected: &Value) {
    let (source, document) = analyze(file, language);
    let actual: Vec<_> = document
        .findings
        .iter()
        .filter(|f| f.resolution == OBJECT_IDENTITY)
        .collect();
    let wanted = expected["findings"].as_array().cloned().unwrap_or_default();
    let describe = |f: &state_lens::model::Finding| {
        let trace = f.state_trace.as_ref().unwrap();
        format!(
            "{} trigger={:?} origins={:?} uses={:?} level={:?} requirement={:?}",
            f.mechanism.clone().unwrap_or_default(),
            &source[f.trigger.start..f.trigger.end],
            trace
                .origins
                .iter()
                .map(|l| &source[l.range.start..l.range.end])
                .collect::<Vec<_>>(),
            trace
                .affected_uses
                .iter()
                .map(|l| &source[l.range.start..l.range.end])
                .collect::<Vec<_>>(),
            trace.evidence_level,
            trace.requirement.as_ref().map(|r| r.kind.clone()),
        )
    };
    assert_eq!(
        actual.len(),
        wanted.len(),
        "{}: expected {} object-identity findings, got [{}]",
        file.display(),
        wanted.len(),
        actual
            .iter()
            .map(|f| describe(f))
            .collect::<Vec<_>>()
            .join("; ")
    );
    for want in &wanted {
        let trigger = anchor(&source, &want["trigger"], file);
        let finding = actual
            .iter()
            .find(|f| f.trigger == trigger)
            .unwrap_or_else(|| {
                panic!(
                    "{}: no finding at {:?}; got [{}]",
                    file.display(),
                    want["trigger"],
                    actual
                        .iter()
                        .map(|f| describe(f))
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            });
        let trace = finding.state_trace.as_ref().unwrap();
        let context = describe(finding);
        assert_eq!(
            finding.mechanism.as_deref(),
            Some(
                want.get("mechanism")
                    .and_then(Value::as_str)
                    .unwrap_or(mechanism)
            ),
            "{}: mechanism {context}",
            file.display()
        );
        assert_eq!(
            ranges(&trace.origins),
            anchors(&source, &want["origins"], file),
            "{}: origins {context}",
            file.display()
        );
        assert_eq!(
            ranges(&trace.invalidations),
            vec![trigger],
            "{}: invalidations {context}",
            file.display()
        );
        assert_eq!(
            ranges(&trace.affected_uses),
            anchors(&source, &want["affected_uses"], file),
            "{}: affected uses {context}",
            file.display()
        );
        let level = serde_json::to_value(trace.evidence_level).unwrap();
        assert_eq!(
            level,
            want["evidence_level"],
            "{}: evidence level {context}",
            file.display()
        );
        let requirement = trace
            .requirement
            .as_ref()
            .map(|r| Value::String(r.kind.clone()))
            .unwrap_or(Value::Null);
        assert_eq!(
            requirement,
            want.get("requirement").cloned().unwrap_or(Value::Null),
            "{}: requirement {context}",
            file.display()
        );
        let mut blast: Vec<Span> = finding.blast_radius.clone();
        blast.sort();
        for span in &blast {
            assert!(span.end <= source.len());
        }
        let graph = document.state_graph.as_ref().unwrap();
        trace
            .validate(graph)
            .unwrap_or_else(|e| panic!("{}: invalid trace: {e}", file.display()));
    }
    if let Some(kinds) = expected.get("boundaries").and_then(Value::as_array) {
        let graph = document.state_graph.as_ref().unwrap();
        for kind in kinds {
            assert!(
                graph
                    .boundaries
                    .iter()
                    .any(|b| Value::String(b.kind.clone()) == *kind),
                "{}: expected boundary {kind}; got {:?}",
                file.display(),
                graph.boundaries.iter().map(|b| &b.kind).collect::<Vec<_>>()
            );
        }
    }
    if let Some(repair) = expected.get("repair") {
        let result = state_lens::repair::check(
            &source,
            language,
            &file.display().to_string(),
            finding_index(&document, &source, &wanted),
            None,
        );
        assert_eq!(
            serde_json::to_value(result.applicability).unwrap(),
            repair["applicability"],
            "{}: repair {}",
            file.display(),
            serde_json::to_string_pretty(&result).unwrap()
        );
    }
}

fn finding_index(
    document: &state_lens::model::DocumentReport,
    source: &str,
    wanted: &[Value],
) -> usize {
    let file = Path::new("expectation");
    let trigger = anchor(source, &wanted[0]["trigger"], file);
    document
        .findings
        .iter()
        .position(|f| f.resolution == OBJECT_IDENTITY && f.trigger == trigger)
        .expect("repair target finding")
}

fn expectations(defect: &str, language: &str, mechanism: &str) -> Value {
    let path = directory(defect, language, mechanism).join("expect.json");
    serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .unwrap()
}

fn role(defect: &str, language: &str, mechanism: &str, role: &str) {
    let expected = expectations(defect, language, mechanism);
    let file =
        directory(defect, language, mechanism).join(format!("{role}.{}", extension(language)));
    let spec = expected["roles"]
        .get(role)
        .unwrap_or_else(|| panic!("{}: missing role {role} in expect.json", file.display()));
    if catalog::cell(defect, mechanism, language).is_some_and(|cell| cell["role_fixtures"] == true) {
        let (_, report) = analyze(&file, language);
        assert!(!report.parse_errors, "{}", file.display());
        let present = report.findings.iter().any(|f| f.defect == defect && f.origin == "direct" && f.mechanism.as_deref() == Some(mechanism) && f.assessment != state_lens::model::FindingAssessment::Usage);
        assert_eq!(present, spec["present"].as_bool().unwrap(), "{}: {mechanism}", file.display());
        return;
    }
    if defect == "threading-mutexes" {
        let (_, report) = analyze(&file, language);
        let present = report.findings.iter().any(|f| f.defect == defect && f.origin == "direct" && f.mechanism.as_deref() == Some(mechanism));
        assert_eq!(present, spec["present"].as_bool().unwrap(), "{}: {mechanism}", file.display());
        return;
    }
    check_expectation(&file, language, mechanism, spec);
    if role == ROLES[0] {
        for (variant, spec) in expected["variants"].as_object().into_iter().flatten() {
            let file = directory(defect, language, mechanism)
                .join(format!("{variant}.{}", extension(language)));
            if defect == "threading-mutexes" {
        let (_, report) = analyze(&file, language);
        let present = report.findings.iter().any(|f| f.defect == defect && f.origin == "direct" && f.mechanism.as_deref() == Some(mechanism));
        assert_eq!(present, spec["present"].as_bool().unwrap(), "{}: {mechanism}", file.display());
        return;
    }
    check_expectation(&file, language, mechanism, spec);
        }
    }
}

fn status(defect: &str, language: &str, mechanism: &str) {
    let cell = catalog::cell(defect, mechanism, language).expect("catalog cell");
    let status = cell["status"].as_str().expect("status");
    assert!(
        STATUSES.contains(&status),
        "{defect}/{language}/{mechanism}: unknown status {status}"
    );
    if status != "supported" {
        assert!(
            cell["reason"].as_str().is_some_and(|r| !r.is_empty()),
            "{defect}/{language}/{mechanism}: reason required"
        );
    }
    match status {
        "supported" => {
            let dir = directory(defect, language, mechanism);
            for role in ROLES {
                let file = dir.join(format!("{role}.{}", extension(language)));
                assert!(
                    file.is_file(),
                    "{}: supported cells need every role fixture",
                    file.display()
                );
            }
        }
        "not-applicable" => {
            assert!(
                cell["guarantee"].as_str().is_some_and(|g| !g.is_empty()),
                "{defect}/{language}/{mechanism}: not-applicable needs the language guarantee"
            );
        }
        "syntax-candidate" => {
            let rules: Value = serde_json::from_str(
                &std::fs::read_to_string(root().join("../../queries/rules.json")).unwrap(),
            )
            .unwrap();
            assert!(
                rules["rules"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["language"] == language && r["defect"] == defect),
                "{defect}/{language}: syntax-candidate cells need a production query pack"
            );
        }
        "candidate-only" => {
            for example in cell["examples"].as_array().into_iter().flatten() {
                let path = root().join("../..").join(example.as_str().unwrap());
                let (_, document) = analyze(&path, language);
                assert!(
                    !document
                        .findings
                        .iter()
                        .any(|f| f.resolution == OBJECT_IDENTITY
                            && f.mechanism.as_deref() == Some(mechanism)),
                    "{}: candidate-only cell produced a semantic {mechanism} finding",
                    path.display()
                );
            }
        }
        _ => {}
    }
    if let Some(hatch) = cell.get("escape_hatch").and_then(Value::as_str) {
        let languages = catalog::catalog()["languages"].as_array().unwrap();
        let exists = languages
            .iter()
            .any(|l| catalog::cell(defect, hatch, l.as_str().unwrap()).is_some());
        assert!(exists, "{defect}/{language}/{mechanism}: escape hatch {hatch} must be a mechanism of the same rule");
    }
}

#[test]
fn every_rule_mechanism_defines_every_enabled_language() {
    let catalog = catalog::catalog();
    let languages = catalog["languages"].as_array().unwrap();
    for rule in catalog["rules"].as_array().unwrap() {
        for mechanism in rule["mechanisms"].as_array().unwrap() {
            for language in languages {
                assert!(
                    mechanism["cells"].get(language.as_str().unwrap()).is_some(),
                    "{}/{}: missing cell for {language}",
                    rule["defect"],
                    mechanism["id"]
                );
            }
        }
    }
}

#[allow(non_snake_case)]
mod cells {
    use super::{role, status};
    include!(concat!(env!("OUT_DIR"), "/matrix_tests.rs"));
}

#[test]
fn repair_preconditions_reject_stale_sources_and_unrelated_findings() {
    let path = directory(
        "aliasing-mutable-defaults",
        "typescript",
        "shallow-copy-nested",
    )
    .join("failing.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let (_, document) = analyze(&path, "typescript");
    let index = document
        .findings
        .iter()
        .position(|f| f.resolution == OBJECT_IDENTITY)
        .unwrap();
    let stale = state_lens::repair::check(
        &source,
        "typescript",
        "failing.ts",
        index,
        Some(&"0".repeat(64)),
    );
    assert_eq!(
        stale.applicability,
        state_lens::repair::Applicability::Rejected
    );
    assert!(stale.candidates.is_empty());
    let fresh = state_lens::repair::check(
        &source,
        "typescript",
        "failing.ts",
        index,
        Some(&state_lens::model::hash(&source)),
    );
    assert_eq!(fresh.applicability, state_lens::repair::Applicability::Auto);
    let verified = fresh
        .candidates
        .iter()
        .find(|c| c.status == state_lens::repair::CandidateStatus::Verified)
        .unwrap();
    assert!(verified.remaining_relationships.is_empty());
    let unrelated = document
        .findings
        .iter()
        .position(|f| f.resolution != OBJECT_IDENTITY)
        .unwrap();
    let rejected = state_lens::repair::check(&source, "typescript", "failing.ts", unrelated, None);
    assert_eq!(
        rejected.applicability,
        state_lens::repair::Applicability::Rejected
    );
}

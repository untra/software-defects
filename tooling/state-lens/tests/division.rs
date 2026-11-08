use state_lens::analysis::Engine;
use std::{fs, path::Path};
fn risks(lang: &str, source: &str) -> Vec<state_lens::model::Finding> {
    let mut e = Engine::new().unwrap();
    e.set("file:///test".into(), lang, source.into(), Some(1))
        .unwrap();
    e.analyze()
        .documents
        .remove(0)
        .findings
        .into_iter()
        .filter(|f| f.division_risk.is_some())
        .collect()
}
#[test]
fn corpus_pairs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for lang in ["go", "rust", "typescript", "python"] {
        for entry in fs::read_dir(
            root.join("examples/unstructured-state/divide-by-zero")
                .join(lang),
        )
        .unwrap()
        {
            let path = entry.unwrap().path();
            let source = fs::read_to_string(&path).unwrap();
            let found = risks(lang, &source);
            assert!(
                found
                    .iter()
                    .any(|f| f.division_risk.as_ref().unwrap().confidence != "candidate"),
                "{}: {found:?}",
                path.display()
            );
            let fixed = root
                .join("examples-fixed/unstructured-state/divide-by-zero")
                .join(lang)
                .join(path.file_name().unwrap());
            assert!(
                risks(lang, &fs::read_to_string(&fixed).unwrap()).is_empty(),
                "{}",
                fixed.display()
            );
        }
    }
}
#[test]
fn mutation_and_guards() {
    let findings = risks("typescript", "let d = 2; d = 0; const result = 1 / d;");
    assert!(findings[0].division_risk.as_ref().unwrap().mutated);
    assert!(risks("typescript", "const d = 2; const result = 1 / d;").is_empty());
    assert!(risks(
        "python",
        "def f(d):\n    if d == 0:\n        return None\n    return 1 / d\n"
    )
    .is_empty());
    assert!(!risks(
        "python",
        "def f(d):\n    if d == 0:\n        return None\n    d = 0\n    return 1 / d\n"
    )
    .is_empty());
    assert!(risks("python", "text = '%s' % 0").is_empty());
}
#[test]
fn compound_operations_and_stale_values() {
    for (lang, source) in [
        ("python", "d = 0\nx = 1\nx //= d\n"),
        ("typescript", "const d = 0; let x = 1; x /= d;"),
        ("rust", "fn main() { let d = 0; let mut x = 1; x %= d; }"),
        ("go", "package main\nfunc main() { d := 0; x := 1; x /= d }"),
    ] {
        assert_eq!(risks(lang, source).len(), 1, "{lang}");
    }
    assert!(!risks("typescript", "let d = 1; d--; const x = 1 / d;").is_empty());
    assert!(!risks(
        "python",
        "def f(d):\n    if d == 0:\n        return None\n    change()\n    return 1 / d\n"
    )
    .is_empty());
}

#[test]
fn smoke_mutation_fixtures_have_yellow_evidence() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/division-mutations.json")).unwrap();
    for (language, case) in cases.as_object().unwrap() {
        let found = risks(language, case["source"].as_str().unwrap());
        assert_eq!(found.len(), 1, "{language}");
        assert!(
            found[0].division_risk.as_ref().unwrap().mutated,
            "{language}"
        );
        assert_ne!(
            found[0].division_risk.as_ref().unwrap().confidence,
            "candidate"
        );
    }
}

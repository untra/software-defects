use state_lens::analysis::{language_name, Engine};
use std::path::Path;

#[test]
fn javascript_extensions_and_queries() {
    for extension in ["js", "mjs", "cjs"] {
        assert_eq!(
            language_name(Path::new(&format!("sample.{extension}"))),
            Some("javascript")
        );
    }
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///clock.js".into(),
            "javascript",
            "function clock(){return Date.now();}".into(),
            Some(1),
        )
        .unwrap();
    let report = engine.analyze();
    assert!(!report.documents[0].findings.is_empty());
    assert!(report.documents[0]
        .findings
        .iter()
        .all(|f| f.rule_id.starts_with("javascript.")));
}

#[test]
fn javascript_corpus_parses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fn visit(path: &Path, engine: &mut Engine) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, engine);
            } else if path.extension().is_some_and(|ext| ext == "js")
                && path.parent().unwrap().ends_with("javascript")
            {
                engine
                    .set(
                        url::Url::from_file_path(&path).unwrap().to_string(),
                        "javascript",
                        std::fs::read_to_string(&path).unwrap(),
                        None,
                    )
                    .unwrap();
            }
        }
    }
    let mut engine = Engine::new().unwrap();
    visit(&root.join("examples"), &mut engine);
    visit(&root.join("examples-fixed"), &mut engine);
    for document in &engine.analyze().documents {
        assert!(
            !document.parse_errors,
            "{}: {:?}",
            document.uri, document.parse_errors
        );
    }
}

#[test]
fn javascript_import_effects_and_edits() {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///library.mjs".into(),
            "javascript",
            "export function clock(){ return Date.now(); }".into(),
            Some(1),
        )
        .unwrap();
    engine
        .set(
            "file:///caller.js".into(),
            "javascript",
            "import {clock} from './library.mjs'; function caller(){ return clock(); }".into(),
            Some(1),
        )
        .unwrap();
    let report = engine.analyze();
    assert!(report
        .documents
        .iter()
        .find(|d| d.uri.ends_with("caller.js"))
        .unwrap()
        .findings
        .iter()
        .any(|finding| finding.origin == "propagated"));
    engine
        .set(
            "file:///library.mjs".into(),
            "javascript",
            "export function clock(){ return 1; }".into(),
            Some(2),
        )
        .unwrap();
    assert!(!engine
        .analyze()
        .documents
        .iter()
        .find(|d| d.uri.ends_with("caller.js"))
        .unwrap()
        .findings
        .iter()
        .any(|finding| finding.origin == "propagated"));
}

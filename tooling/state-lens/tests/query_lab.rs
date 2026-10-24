use std::{fs, path::Path, process::Command};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

#[test]
#[ignore = "requires Query Lab setup and Python 3.12+; run explicitly in query-lab CI"]
fn external_cli_matches_production_queries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for (name, extension, grammar) in [
        ("go", "go", Language::from(tree_sitter_go::LANGUAGE)),
        ("rust", "rs", Language::from(tree_sitter_rust::LANGUAGE)),
        (
            "typescript",
            "ts",
            Language::from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
        ),
    ] {
        let source_path = root.join(format!(
            "tooling/state-lens/fixtures/{name}/clock.{extension}"
        ));
        let source = fs::read(&source_path).unwrap();
        let mut parser = Parser::new();
        parser.set_language(&grammar).unwrap();
        let tree = parser.parse(&source, None).unwrap();
        for &(path, query_source) in state_lens::analysis::QUERIES {
            if !path.starts_with(&format!("{name}/")) {
                continue;
            }
            let query = Query::new(&grammar, query_source).unwrap();
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_slice());
            let mut expected = Vec::new();
            while let Some(found) = matches.next() {
                for capture in found.captures {
                    expected.push((
                        found.pattern_index,
                        query.capture_names()[capture.index as usize].to_owned(),
                        capture.node.start_byte(),
                        capture.node.end_byte(),
                    ));
                }
            }
            expected.sort();
            let output = Command::new("python3")
                .arg(root.join("tooling/query-lab/query_lab.py"))
                .args(["run", "--query"])
                .arg(root.join("queries").join(path))
                .arg("--source")
                .arg(&source_path)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{path}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            let mut actual = Vec::new();
            for found in report["matches"].as_array().unwrap() {
                for capture in found["captures"].as_array().unwrap() {
                    actual.push((
                        found["pattern"].as_u64().unwrap() as usize,
                        capture["name"].as_str().unwrap().to_owned(),
                        capture["range"][0].as_u64().unwrap() as usize,
                        capture["range"][1].as_u64().unwrap() as usize,
                    ));
                }
            }
            actual.sort();
            assert_eq!(actual, expected, "capture parity for {path}");
        }
    }
}

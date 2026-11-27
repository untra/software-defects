use serde_json::{json, Value};
use std::sync::OnceLock;

pub fn catalog() -> &'static Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalog.json")))
            .expect("rule catalog")
    })
}

pub fn cell(defect: &str, mechanism: &str, language: &str) -> Option<&'static Value> {
    catalog()["rules"]
        .as_array()?
        .iter()
        .find(|r| r["defect"] == defect)?["mechanisms"]
        .as_array()?
        .iter()
        .find(|m| m["id"] == mechanism)?["cells"]
        .get(language)
}

pub fn cells(languages: &[&str]) -> Value {
    let mut rows = vec![];
    for rule in catalog()["rules"].as_array().into_iter().flatten() {
        for mechanism in rule["mechanisms"].as_array().into_iter().flatten() {
            for language in languages {
                if let Some(cell) = mechanism["cells"].get(*language) {
                    rows.push(json!({
                        "family": rule["family"],
                        "defect": rule["defect"],
                        "mechanism": mechanism["id"],
                        "summary": mechanism["summary"],
                        "language": language,
                        "status": cell["status"],
                        "reason": cell.get("reason"),
                        "guarantee": cell.get("guarantee"),
                        "escapeHatch": cell.get("escape_hatch"),
                    }));
                }
            }
        }
    }
    json!({"schemaVersion": catalog()["schema_version"], "languages": languages, "rules": rows})
}

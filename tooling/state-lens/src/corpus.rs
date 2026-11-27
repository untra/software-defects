use crate::{analysis::Engine, model::*};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Expectations {
    pub schema_version: u32,
    pub cases: Vec<Case>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Case {
    pub file: String,
    pub source_hash: String,
    pub expected: Vec<Expected>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Expected {
    pub defect: String,
    pub trigger: Span,
    pub function: Option<Span>,
    pub effect: Effect,
    pub origin: String,
}
#[derive(Debug, Serialize)]
pub struct Score {
    pub file: String,
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
    pub precision: f64,
    pub recall: f64,
    pub language: String,
    pub dimensions: BTreeMap<String, Metrics>,
    pub by_defect: BTreeMap<String, Metrics>,
}
#[derive(Debug, Serialize)]
pub struct Metrics {
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
}
fn metrics(
    expected: &[Expected],
    actual: &[Expected],
    key: impl Fn(&Expected) -> String,
) -> Metrics {
    let expected: BTreeSet<_> = expected.iter().map(&key).collect();
    let actual: BTreeSet<_> = actual.iter().map(&key).collect();
    Metrics {
        true_positive: expected.intersection(&actual).count(),
        false_positive: actual.difference(&expected).count(),
        false_negative: expected.difference(&actual).count(),
    }
}

pub fn score(path: &Path) -> Result<Vec<Score>> {
    let expectations: Expectations = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    if expectations.schema_version != SCHEMA_VERSION {
        bail!("unsupported expectation schema");
    }
    let base = path.parent().context("expectations directory")?;
    let mut engine = Engine::new()?;
    for case in &expectations.cases {
        let file = base.join(&case.file).canonicalize()?;
        let source = std::fs::read_to_string(&file)?;
        if hash(&source) != case.source_hash {
            bail!("stale expectations: {}", case.file);
        }
        engine.load_file(&file)?;
    }
    let report = engine.analyze();
    let mut scores = vec![];
    for case in &expectations.cases {
        let uri = url::Url::from_file_path(base.join(&case.file).canonicalize()?)
            .unwrap()
            .to_string();
        let document = report
            .documents
            .iter()
            .find(|d| d.uri == uri)
            .context("missing report")?;
        let mut actual: Vec<_> = document
            .findings
            .iter()
            .map(|f| Expected {
                defect: f.defect.clone(),
                trigger: f.trigger,
                function: f.function,
                effect: f.effect.clone(),
                origin: f.origin.clone(),
            })
            .collect();
        let mut dimensions = BTreeMap::new();
        dimensions.insert(
            "trigger_ranges".into(),
            metrics(&case.expected, &actual, |f| {
                format!("{}:{:?}", f.defect, f.trigger)
            }),
        );
        dimensions.insert(
            "function_ownership".into(),
            metrics(&case.expected, &actual, |f| {
                format!("{}:{:?}:{:?}", f.defect, f.trigger, f.function)
            }),
        );
        dimensions.insert(
            "effects_and_propagation".into(),
            metrics(&case.expected, &actual, |f| {
                format!("{}:{:?}:{:?}:{}", f.defect, f.trigger, f.effect, f.origin)
            }),
        );
        let mut by_defect = BTreeMap::new();
        let defects: BTreeSet<_> = case
            .expected
            .iter()
            .chain(&actual)
            .map(|f| f.defect.clone())
            .collect();
        for defect in defects {
            let expected: Vec<_> = case
                .expected
                .iter()
                .filter(|f| f.defect == defect)
                .cloned()
                .collect();
            let observed: Vec<_> = actual
                .iter()
                .filter(|f| f.defect == defect)
                .cloned()
                .collect();
            by_defect.insert(defect, metrics(&expected, &observed, |f| format!("{f:?}")));
        }
        let mut tp = 0;
        for expected in &case.expected {
            if let Some(i) = actual.iter().position(|a| a == expected) {
                actual.remove(i);
                tp += 1;
            }
        }
        let fp = actual.len();
        let fn_ = case.expected.len() - tp;
        scores.push(Score {
            file: case.file.clone(),
            language: document.language.clone(),
            dimensions,
            by_defect,
            true_positive: tp,
            false_positive: fp,
            false_negative: fn_,
            precision: if tp + fp == 0 {
                1.0
            } else {
                tp as f64 / (tp + fp) as f64
            },
            recall: if tp + fn_ == 0 {
                1.0
            } else {
                tp as f64 / (tp + fn_) as f64
            },
        });
    }
    Ok(scores)
}

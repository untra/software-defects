use crate::model::*;
use std::collections::BTreeSet;

const RESOLUTION: &str = "potential-overlap";
const FAMILY: &str = "compound-defects";
const P0: &[&str] = &[
    "race-conditions",
    "threading-mutexes",
    "resource-contention",
    "cache-invalidation",
    "insecure-deserialization",
    "index-out-of-bounds",
    "type-errors",
];

pub(crate) fn analyze(report: &mut DocumentReport) {
    let mut overlaps = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, first) in report
        .findings
        .iter()
        .enumerate()
        .filter(|(_, f)| f.compound.is_none())
    {
        for second in report
            .findings
            .iter()
            .skip(index + 1)
            .filter(|f| f.compound.is_none())
        {
            if !P0.contains(&first.defect.as_str()) && !P0.contains(&second.defect.as_str()) {
                continue;
            }
            let class = match (first.family.as_str(), second.family.as_str()) {
                ("mutated-state", "shared-state") | ("shared-state", "mutated-state") => {
                    "nondeterminism"
                }
                ("shared-state", "unstructured-state") | ("unstructured-state", "shared-state") => {
                    "data-corruption"
                }
                ("mutated-state", "unstructured-state")
                | ("unstructured-state", "mutated-state") => "runtime-errors",
                _ => continue,
            };
            let first_regions = if first.blast_radius.is_empty() {
                std::slice::from_ref(&first.trigger)
            } else {
                &first.blast_radius
            };
            let second_regions = if second.blast_radius.is_empty() {
                std::slice::from_ref(&second.trigger)
            } else {
                &second.blast_radius
            };
            for a in first_regions {
                for b in second_regions {
                    let range = Span {
                        start: a.start.max(b.start),
                        end: a.end.min(b.end),
                    };
                    if range.start >= range.end
                        || !seen.insert((
                            class,
                            range,
                            first.rule_id.clone(),
                            second.rule_id.clone(),
                        ))
                    {
                        continue;
                    }
                    let ingredients = [first, second].map(|finding| Evidence {
                        uri: report.uri.clone(),
                        range: finding.trigger,
                        rule_id: finding.rule_id.clone(),
                    });
                    overlaps.push(Finding {
                        assessment: FindingAssessment::Candidate,
                        shared_state: None, state_trace: None, division_risk: None,
                        rule_id: format!("{}.{}.{}.{}", report.language, FAMILY, class, RESOLUTION),
                        family: FAMILY.into(), defect: class.into(), mechanism: Some(RESOLUTION.into()),
                        compound: Some(Compound { class: class.into(), category: RESOLUTION.into(), ingredients: ingredients.clone() }),
                        blast_radius: vec![range], trigger: range,
                        function: first.function.filter(|f| Some(*f) == second.function),
                        effect: Effect::StructuralSignal,
                        origin: if first.origin == "direct" && second.origin == "direct" { "direct" } else { "propagated" }.into(),
                        resolution: RESOLUTION.into(),
                        explanation: "Effects from two state families overlap in this region. This is a potential compound-defect area, not evidence that the combined failure occurs.".into(),
                        evidence: ingredients.to_vec(), call_path: vec![], related_cwes: vec![],
                    });
                }
            }
        }
    }
    report.findings.extend(overlaps);
}

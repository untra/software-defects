use state_lens::{analysis::Engine, model::FindingAssessment};
fn mechanisms(language: &str, source: &str) -> Vec<String> {
    let mut engine = Engine::new().unwrap();
    engine
        .set("file:///sample".into(), language, source.into(), Some(1))
        .unwrap();
    engine.analyze().documents[0]
        .findings
        .iter()
        .filter(|f| f.defect == "resource-contention" && f.assessment != FindingAssessment::Usage)
        .filter_map(|f| f.mechanism.clone())
        .collect()
}
#[test]
fn bounded_and_unbounded_queue() {
    assert!(mechanisms("python", "import queue\njobs = queue.Queue()\n")
        .contains(&"unbounded-queue".into()));
    assert!(mechanisms("python", "import queue\njobs = queue.Queue(maxsize=8)\n").is_empty());
}
#[test]
fn fan_out_needs_dynamic_work() {
    assert!(mechanisms(
        "javascript",
        "async function run(items) { await Promise.all(items.map(fetch)); }"
    )
    .contains(&"unbounded-fan-out".into()));
    assert!(mechanisms(
        "javascript",
        "async function run(items) { for (const item of items) await fetch(item); }"
    )
    .is_empty());
}

#[test]
fn shadowed_namespace_is_not_a_known_scheduler() {
    assert!(mechanisms(
        "javascript",
        "async function run(Promise, items) { await Promise.all(items.map(fetch)); }"
    )
    .is_empty());
}

#[test]
fn failed_await_requires_finally_for_a_permit() {
    let bad =
        "async def run(sem):\n    await sem.acquire()\n    await operation()\n    sem.release()\n";
    assert!(mechanisms("python", bad).contains(&"acquire-without-release-on-error".into()));
    let fixed = "async def run(sem):\n    await sem.acquire()\n    try:\n        await operation()\n    finally:\n        sem.release()\n";
    assert!(!mechanisms("python", fixed).contains(&"acquire-without-release-on-error".into()));
}

#[test]
fn finally_does_not_protect_work_before_its_try() {
    let source = "async def run(sem):\n    await sem.acquire()\n    await operation()\n    try:\n        await another()\n    finally:\n        sem.release()\n";
    assert!(mechanisms("python", source).contains(&"acquire-without-release-on-error".into()));
}

use state_lens::analysis::Engine;
fn has(source: &str, mechanism: &str) -> bool {
    let mut engine = Engine::new().unwrap();
    engine
        .set(
            "file:///cache.js".into(),
            "javascript",
            source.into(),
            Some(1),
        )
        .unwrap();
    engine.analyze().documents[0]
        .findings
        .iter()
        .any(|f| f.defect == "cache-invalidation" && f.mechanism.as_deref() == Some(mechanism))
}
#[test]
fn rejected_promise_requires_local_eviction() {
    assert!(has("const cache = new Map(); async function fetcher(k) { return fetch(k); } function get(k) { if (!cache.has(k)) cache.set(k, fetcher(k)); return cache.get(k); }", "cached-rejected-promise"));
    assert!(!has("const cache = new Map(); async function fetcher(k) { return fetch(k); } function get(k) { if (!cache.has(k)) cache.set(k, fetcher(k).catch(e => { cache.delete(k); throw e; })); return cache.get(k); }", "cached-rejected-promise"));
}
#[test]
fn miss_await_set_is_herd_candidate() {
    assert!(has("const cache = new Map(); async function get(k) { if (cache.has(k)) return cache.get(k); const v = await fetch(k); cache.set(k, v); return v; }", "thundering-herd"));
}

#[test]
fn python_task_rejection_needs_cache_eviction() {
    for (source, expected) in [
        ("import asyncio\ncache = {}\ndef get(k):\n    if k not in cache:\n        cache[k] = asyncio.create_task(fetch(k))\n    return cache[k]\n", true),
        ("import asyncio\ncache = {}\nasync def get(k):\n    if k not in cache:\n        cache[k] = asyncio.create_task(fetch(k))\n    try:\n        return await cache[k]\n    except Exception:\n        cache.pop(k, None)\n        raise\n", false),
    ] {
        let mut engine = Engine::new().unwrap();
        engine.set("file:///cache.py".into(), "python", source.into(), Some(1)).unwrap();
        assert_eq!(engine.analyze().documents[0].findings.iter().any(|f| f.mechanism.as_deref() == Some("cached-rejected-promise")), expected);
    }
}

#[test]
fn invalidation_compares_key_structure_not_just_call_presence() {
    let source = "const db = new Map(); const cache = new Map(); function get(k) { const key = lower(k); if (cache.has(key)) return cache.get(key); const v = db.get(key); cache.set(key, v); return v; } function set(k, v) { db.set(lower(k), v); cache.delete(upper(k)); }";
    assert!(has(source, "invalidate-wrong-key"));
    assert!(!has(
        &source.replace("cache.delete(upper(k))", "cache.delete(lower(k))"),
        "invalidate-wrong-key"
    ));
}

use state_lens::{analysis::Engine, model::DocumentReport, mutex::Assessment};
fn analyze(language: &str, source: &str) -> DocumentReport {
    let mut engine = Engine::new().unwrap();
    engine
        .set("file:///mutex".into(), language, source.into(), Some(1))
        .unwrap();
    engine.analyze().documents.remove(0)
}
fn has(report: &DocumentReport, mechanism: &str) -> bool {
    report.findings.iter().any(|f| {
        f.defect == "threading-mutexes"
            && f.origin == "direct"
            && f.mechanism.as_deref() == Some(mechanism)
    })
}
fn corpus(language: &str, name: &str, mechanism: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (folder, expected) in [("examples", true), ("examples-fixed", false)] {
        let source = std::fs::read_to_string(
            root.join(folder)
                .join("shared-state/threading-mutexes")
                .join(language)
                .join(name),
        )
        .unwrap();
        let report = analyze(language, &source);
        assert_eq!(
            has(&report, mechanism),
            expected,
            "{folder}/{language}/{name}: {mechanism}"
        );
        assert!(
            has(&report, "critical-section"),
            "{folder}/{language}/{name}"
        );
        for finding in report
            .findings
            .iter()
            .filter(|f| f.defect == "threading-mutexes")
        {
            let shared = finding.shared_state.as_ref().unwrap();
            assert_eq!(shared.source_hash, state_lens::model::hash(&source));
            for event in &shared.events {
                assert!(source.get(event.range.start..event.range.end).is_some());
            }
        }
    }
}
#[test]
fn release_on_exception() {
    for lang in ["javascript", "typescript"] {
        corpus(
            lang,
            &format!(
                "01_async_lock_not_released_on_throw.{}",
                if lang == "javascript" { "js" } else { "ts" }
            ),
            "missing-release",
        );
    }
}
#[test]
fn go_early_return() {
    corpus(
        "go",
        "03_unlock_skipped_on_early_return.go",
        "missing-release",
    );
}
#[test]
fn go_receiver_copy() {
    corpus("go", "04_mutex_copied_by_value.go", "copied-lock");
}
#[test]
fn rust_temporary_scope() {
    corpus(
        "rust",
        "03_match_scrutinee_holds_guard.rs",
        "non-reentrant-acquisition",
    );
}
#[test]
fn blocking_guard() {
    corpus(
        "rust",
        "04_blocking_guard_across_await.rs",
        "blocking-across-suspension",
    );
}
#[test]
fn python_cancellation() {
    corpus("python", "05_cancelled_owner.py", "missing-release");
}
#[test]
fn python_thread_owner() {
    corpus(
        "python",
        "06_thread_lock_in_tasks.py",
        "blocking-across-suspension",
    );
}
#[test]
fn local_lock() {
    corpus("go", "07_per_call_lock.go", "per-call-lock");
    corpus("python", "07_per_call_lock.py", "per-call-lock");
    for lang in ["javascript", "typescript"] {
        corpus(
            lang,
            &format!(
                "03_per_call_lock.{}",
                if lang == "javascript" { "js" } else { "ts" }
            ),
            "per-call-lock",
        );
    }
}
#[test]
fn web_lifetimes() {
    for lang in ["javascript", "typescript"] {
        let ext = if lang == "javascript" { "js" } else { "ts" };
        corpus(
            lang,
            &format!("04_callback_outlives_lock.{ext}"),
            "callback-lifetime",
        );
        corpus(
            lang,
            &format!("05_nested_web_lock.{ext}"),
            "non-reentrant-acquisition",
        );
    }
}
#[test]
fn local_reentrant_calls() {
    corpus("go", "02_reentrant_lock.go", "non-reentrant-acquisition");
    corpus(
        "rust",
        "02_relock_same_mutex.rs",
        "non-reentrant-acquisition",
    );
    corpus(
        "python",
        "04_needed_reentrant.py",
        "non-reentrant-acquisition",
    );
}
#[test]
fn reader_reentry() {
    corpus(
        "go",
        "05_recursive_rlock_with_writer.go",
        "non-reentrant-acquisition",
    );
}
#[test]
fn conditional_acquire_is_not_reentry() {
    corpus("go", "06_unchecked_try_lock.go", "unchecked-acquisition");
}
#[test]
fn opposite_orders() {
    corpus(
        "javascript",
        "02_opposite_order_async_locks.js",
        "lock-order-cycle",
    );
    corpus(
        "typescript",
        "02_opposite_order_async_locks.ts",
        "lock-order-cycle",
    );
}
#[test]
fn unrelated_methods_and_shadowing() {
    for (lang, source) in [
        ("javascript", "const lock = { acquire() {} }; lock.acquire();"),
        ("python", "import threading\ndef f(threading):\n    lock = threading.Lock()\n    lock.acquire()\n"),
        ("go", "package main\nimport \"sync\"\nfunc f(sync Fake) { var m sync.Mutex; m.Lock() }"),
        ("rust", "struct Mutex; fn f() { let m = Mutex::new(); m.lock(); }"),
    ] { assert!(!has(&analyze(lang, source), "critical-section"), "{lang}"); }
}
#[test]
fn explicit_release_and_independent_locks_break_reentry() {
    for source in ["import threading\nm = threading.Lock()\ndef f():\n    m.acquire()\n    m.release()\n    m.acquire()\n    m.release()\n", "import threading\na = threading.Lock()\nb = threading.Lock()\ndef f():\n    with a:\n        with b:\n            pass\n"] { assert!(!has(&analyze("python", source), "non-reentrant-acquisition")); }
}
#[test]
fn async_lock_can_cross_suspension() {
    let r=analyze("rust", "use tokio::sync::Mutex; async fn f(m: &Mutex<i32>) { let guard = m.lock().await; work().await; drop(guard); }");
    assert!(has(&r, "critical-section"));
    assert!(!has(&r, "blocking-across-suspension"));
}
#[test]
fn direct_reentry_is_a_witness() {
    let r = analyze(
        "python",
        "import threading\nm = threading.Lock()\ndef f():\n    m.acquire()\n    m.acquire()\n",
    );
    assert!(r.findings.iter().any(|f| f
        .shared_state
        .as_ref()
        .and_then(|s| s.synchronization.as_ref())
        .is_some_and(|s| s.assessment == Assessment::Misuse)));
}
#[test]
fn nested_function_is_excluded_and_propagation_is_local() {
    let source="import threading\nm = threading.Lock()\ndef f():\n    with m:\n        def pure():\n            return 1\n        pass\ndef caller():\n    f()\n";
    let r = analyze("python", source);
    let inner = r.functions.iter().find(|f| f.name == "pure").unwrap();
    for f in r
        .findings
        .iter()
        .filter(|f| f.defect == "threading-mutexes")
    {
        assert!(f.blast_radius.iter().all(|s| !s.contains(inner.range)));
        if f.origin == "propagated" {
            assert_eq!(f.blast_radius, vec![f.trigger]);
        }
    }
}
#[test]
fn parameter_order_substitution() {
    corpus("rust", "01_lock_order_deadlock.rs", "lock-order-cycle");
    corpus("go", "01_lock_order_deadlock.go", "lock-order-cycle");
    corpus("python", "01_opposite_lock_order.py", "lock-order-cycle");
}
#[test]
fn lock_aliases_and_rebinding() {
    let alias = analyze("python", "from threading import Lock as Gate\nm = Gate()\ndef f():\n    alias = m\n    with alias:\n        m.acquire()\n");
    assert!(has(&alias, "non-reentrant-acquisition"));
    let rebound = analyze(
        "javascript",
        "let m = navigator.locks; m = other; m.request('ledger', () => {});",
    );
    assert!(!has(&rebound, "critical-section"));
}
#[test]
fn compatible_web_readers_and_nonblocking_requests() {
    for source in [
        "navigator.locks.request('ledger', {mode: 'shared'}, async () => { await navigator.locks.request('ledger', {mode: 'shared'}, () => {}); });",
        "navigator.locks.request('ledger', async () => { await navigator.locks.request('ledger', {ifAvailable: true}, () => {}); });",
    ] { assert!(!has(&analyze("javascript", source), "non-reentrant-acquisition")); }
}
#[test]
fn intervening_unknown_cleanup_prevents_a_misuse_warning() {
    let r = analyze("python", "import threading\nm = threading.Lock()\ndef f():\n    m.acquire()\n    cleanup(m)\n    m.acquire()\n");
    assert!(r.findings.iter().all(|f| f
        .shared_state
        .as_ref()
        .and_then(|s| s.synchronization.as_ref())
        .is_none_or(|s| s.assessment != Assessment::Misuse)));
}

#[test]
fn promise_wrapper_protocol_survives_local_renaming() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/shared-state/threading-mutexes/javascript/01_async_lock_not_released_on_throw.js");
    let source = std::fs::read_to_string(path)
        .unwrap()
        .replace("AsyncLock", "QueueGate")
        .replace("tail", "pending")
        .replace("previous", "prior")
        .replace("release", "finish")
        .replace("(resolve)", "(resume)")
        .replace("= resolve", "= resume");
    let report = analyze("javascript", &source);
    assert!(has(&report, "critical-section"));
    assert!(has(&report, "missing-release"));
}

#[test]
fn split_updates_keep_the_race_detector_as_the_owner() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (lang, file) in [
        ("go", "08_split_update.go"),
        ("rust", "05_split_reservation.rs"),
    ] {
        for (folder, expected) in [("examples", true), ("examples-fixed", false)] {
            let source = std::fs::read_to_string(
                root.join(folder)
                    .join("shared-state/threading-mutexes")
                    .join(lang)
                    .join(file),
            )
            .unwrap();
            let report = analyze(lang, &source);
            assert_eq!(
                report.findings.iter().any(|f| f.defect == "race-conditions"
                    && f.origin == "direct"
                    && f.mechanism.as_deref() == Some("split-critical-section")),
                expected,
                "{folder}/{file}"
            );
            assert!(!has(&report, "non-reentrant-acquisition"));
        }
    }
}

#[test]
fn unicode_crlf_regions_and_edit_invalidation() {
    let mut engine = Engine::new().unwrap();
    let uri = "file:///unicode.py";
    let bad = "import threading\r\nlabel = '🌱'\r\nm = threading.Lock()\r\ndef f():\r\n    m.acquire()\r\n    m.acquire()\r\n";
    engine
        .set(uri.into(), "python", bad.into(), Some(1))
        .unwrap();
    let report = engine.analyze().documents.remove(0);
    let finding = report
        .findings
        .iter()
        .find(|f| f.mechanism.as_deref() == Some("non-reentrant-acquisition"))
        .unwrap();
    assert_eq!(
        &bad[finding.trigger.start..finding.trigger.end],
        "m.acquire()"
    );
    let safe = bad.replace("threading.Lock()", "threading.RLock()");
    engine
        .set(uri.into(), "python", safe.clone(), Some(2))
        .unwrap();
    let report = engine.analyze().documents.remove(0);
    assert!(!has(&report, "non-reentrant-acquisition"));
    for finding in report
        .findings
        .iter()
        .filter(|f| f.defect == "threading-mutexes")
    {
        assert_eq!(
            finding.shared_state.as_ref().unwrap().source_hash,
            state_lens::model::hash(&safe)
        );
    }
}

#[test]
fn unreachable_reacquisition_is_not_a_violation() {
    let source = "import threading\nm = threading.Lock()\ndef f():\n    m.acquire()\n    m.release()\n    return\n    m.acquire()\n    m.acquire()\n";
    let report = analyze("python", source);
    assert!(report
        .findings
        .iter()
        .filter(|f| f.mechanism.as_deref() == Some("non-reentrant-acquisition"))
        .all(|f| f
            .shared_state
            .as_ref()
            .unwrap()
            .synchronization
            .as_ref()
            .unwrap()
            .assessment
            != Assessment::Misuse));
}

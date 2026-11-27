use state_lens::{analysis::Engine, model::DocumentReport};
fn analyze(language: &str, source: &str) -> DocumentReport {
    let mut engine = Engine::new().unwrap();
    engine
        .set("file:///sample".into(), language, source.into(), Some(1))
        .unwrap();
    engine.analyze().documents.remove(0)
}
fn has(report: &DocumentReport, mechanism: &str) -> bool {
    report
        .findings
        .iter()
        .any(|f| f.shared_state.is_some() && f.mechanism.as_deref() == Some(mechanism))
}
#[test]
fn read_suspend_write_and_fixed_keep_async() {
    for lang in ["javascript", "typescript"] {
        let bad=analyze(lang,"const stock = { count: 2 }; async function buy() { const saved = stock.count; await persist(); stock.count = saved - 1; }");
        assert!(has(&bad, "read-suspend-write"));
        let fixed=analyze(lang,"const stock = { count: 2 }; async function buy() { stock.count -= 1; await persist(); }");
        assert!(!has(&fixed, "read-suspend-write"));
        assert!(has(&fixed, "async-function"));
    }
}
#[test]
fn renamed_import_and_shadowed_api() {
    let bad=analyze("typescript","import { existsSync as exists, writeFileSync as write } from 'node:fs'; function claim(path) { if (!exists(path)) write(path, 'leader'); }");
    assert!(has(&bad, "check-then-use"));
    let independent=analyze("typescript","import { existsSync, writeFileSync } from 'node:fs'; function claim(first, second) { if (!existsSync(first)) writeFileSync(second, 'leader'); }");
    assert!(!has(&independent, "check-then-use"));
    let shadowed = analyze(
        "typescript",
        "function claim(Promise) { return Promise.all([]); }",
    );
    assert!(!has(&shadowed, "async-execution"));
}
#[test]
fn nested_functions_do_not_inflate_counts() {
    let report = analyze(
        "typescript",
        "async function outer() { async function inner() { await work(); } await next(); }",
    );
    for f in report
        .findings
        .iter()
        .filter(|f| f.resolution == "async-function")
    {
        assert_eq!(f.shared_state.as_ref().unwrap().sites.suspensions, 1);
    }
}
#[test]
fn declaration_and_unobserved_work_in_python_and_rust() {
    for (lang, source) in [
        (
            "python",
            "async def save():\n    return 1\ndef main():\n    save()\n",
        ),
        ("rust", "async fn save() {} fn main() { let _ = save(); }"),
    ] {
        let report = analyze(lang, source);
        assert!(has(&report, "async-function"));
        assert!(has(&report, "unobserved-completion"));
    }
}
#[test]
fn go_launch_is_not_a_callee_async_declaration() {
    let report = analyze(
        "go",
        "package main\nfunc worker() {}\nfunc main() { go worker() }\n",
    );
    assert!(has(&report, "async-execution"));
    assert!(!has(&report, "async-function"));
}
#[test]
fn binding_identity_distinguishes_shadowed_state() {
    let report=analyze("typescript","const stock = { count: 2 }; async function buy() { const saved = stock.count; await persist(); { const stock = { count: 9 }; stock.count = saved - 1; } }");
    assert!(!has(&report, "read-suspend-write"));
}
#[test]
fn launched_scope_writing_outer_binding_is_a_contention_candidate() {
    let report = analyze(
        "go",
        "package main\nfunc main() { total := 0; go func() { total++ }(); _ = total }",
    );
    assert!(has(&report, "concurrent-shared-write"));
    let local = analyze(
        "go",
        "package main\nfunc main() { go func() { total := 0; total++; _ = total }() }",
    );
    assert!(!has(&local, "concurrent-shared-write"));
}
#[test]
fn shared_effects_propagate_with_caller_ranges() {
    let report = analyze(
        "typescript",
        "function launch() { return Promise.all([]); } function caller() { launch(); }",
    );
    let f = report
        .findings
        .iter()
        .find(|f| f.origin == "propagated" && f.shared_state.is_some())
        .unwrap();
    assert_eq!(f.blast_radius, vec![f.trigger]);
    assert!(!f.call_path.is_empty());
}
#[test]
fn synchronization_candidates_preserve_safe_atomic_operation() {
    let bad = analyze("rust", "fn record(counter: &AtomicUsize) { let n = counter.load(Ordering::SeqCst); counter.store(n + 1, Ordering::SeqCst); }");
    assert!(has(&bad, "split-atomic-update"));
    let fixed = analyze(
        "rust",
        "fn record(counter: &AtomicUsize) { counter.fetch_add(1, Ordering::SeqCst); }",
    );
    assert!(!has(&fixed, "split-atomic-update"));
    assert!(has(&fixed, "synchronization-operation"));
}
#[test]
fn python_import_alias_tracks_task_launch() {
    let report = analyze("python", "import asyncio as aio\nasync def work():\n    pass\nasync def main():\n    task = aio.create_task(work())\n    await task\n");
    assert!(has(&report, "async-execution"));
}
#[test]
fn release_waits_for_pending_io_completion() {
    let bad=analyze("typescript", "import { open } from 'node:fs/promises'; async function f(path) { const file = await open(path); const pending = file.write('x'); await file.close(); }");
    assert!(has(&bad, "release-before-completion"));
    let fixed=analyze("typescript", "import { open } from 'node:fs/promises'; async function f(path) { const file = await open(path); const pending = file.write('x'); await pending; await file.close(); }");
    assert!(!has(&fixed, "release-before-completion"));
    assert!(has(&fixed, "resource-lifecycle"));
}
#[test]
fn cancellation_is_not_completion() {
    let bad = analyze(
        "rust",
        "async fn f() { let task = tokio::spawn(async {}); task.abort(); }",
    );
    assert!(has(&bad, "cancellation-unobserved"));
    let fixed = analyze(
        "rust",
        "async fn f() { let task = tokio::spawn(async {}); task.abort(); task.await; }",
    );
    assert!(!has(&fixed, "cancellation-unobserved"));
    assert!(has(&fixed, "task-lifecycle"));
}
#[test]
fn unicode_ranges_and_compound_evidence_are_source_bound() {
    let source="const label = '🌱'; const stock = { count: 2 }; async function buy() { const saved = stock.count; await persist(); stock.count = saved - 1; }";
    let report = analyze("typescript", source);
    let compound = report
        .findings
        .iter()
        .find(|f| f.compound.is_some() && f.shared_state.is_some())
        .unwrap();
    assert_eq!(
        &source[compound.trigger.start..compound.trigger.end],
        "stock.count = saved - 1"
    );
    for f in &report.findings {
        if let Some(shared) = &f.shared_state {
            assert_eq!(shared.source_hash, state_lens::model::hash(source));
        }
    }
}
#[test]
fn local_builtin_lookalikes_are_not_io_sources() {
    let report = analyze(
        "typescript",
        "function fetch(path) { return path; } function f() { fetch('local'); }",
    );
    assert!(!has(&report, "resource-operation"));
}
#[test]
fn local_task_alias_preserves_completion_observation() {
    let report=analyze("rust", "async fn f() { let task = tokio::spawn(async {}); let alias = task; alias.abort(); alias.await; }");
    assert!(has(&report, "task-lifecycle"));
    assert!(!has(&report, "cancellation-unobserved"));
}

#[test]
fn named_launch_targets_and_transitive_workers() {
    for (language, source) in [
        ("go", "package p\nvar n int\nfunc increment() { n++ }\nfunc worker() { increment() }\nfunc main() { go worker() }"),
        ("python", "import threading\nn = 0\ndef worker():\n    global n\n    n += 1\nthreading.Thread(target=worker).start()\n"),
    ] {
        assert!(has(&analyze(language, source), "concurrent-shared-write"), "{language}");
    }
}

#[test]
fn membership_suspend_insert_and_outer_await_result() {
    let bad = analyze("javascript", "const users = new Map(); async function add(key) { if (users.has(key)) return; await fetch(key); users.set(key, true); }");
    assert!(has(&bad, "read-suspend-write"));
    let fixed = analyze("javascript", "const users = new Map(); async function add(key) { await fetch(key); if (users.has(key)) return; users.set(key, true); }");
    assert!(!has(&fixed, "read-suspend-write"));
    let bad = analyze("javascript", "let visible; async function update(key) { visible = await fetch(key); }");
    assert!(has(&bad, "out-of-order-response"));
}

#[test]
fn sequence_guard_and_cooperative_rollback_are_not_shared_write_risks() {
    let guarded = analyze("javascript", "let generation = 0; let visible; async function update(key) { const ticket = ++generation; const result = await fetch(key); if (ticket === generation) visible = result; } async function main() { await Promise.all([update('a'), update('b')]); }");
    assert!(!has(&guarded, "concurrent-shared-write"));
    let rollback = analyze("javascript", "let stock = 1; async function buy() { stock -= 1; try { await fetch('/buy'); } catch { stock += 1; } } async function main() { await Promise.all([buy(), buy()]); }");
    assert!(!has(&rollback, "concurrent-shared-write"));
}

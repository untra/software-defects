#[derive(Default, Debug)]
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u64,
}

fn run_with_retry(policy: &RetryPolicy, mut op: impl FnMut() -> bool) -> bool {
    for _ in 0..policy.max_attempts {
        if op() {
            return true;
        }
    }
    false
}

fn main() {
    let policy = RetryPolicy::default();
    let mut calls = 0;
    let ok = run_with_retry(&policy, || {
        calls += 1;
        true
    });
    println!("{:?} succeeded: {} after {} calls", policy, ok, calls);
}

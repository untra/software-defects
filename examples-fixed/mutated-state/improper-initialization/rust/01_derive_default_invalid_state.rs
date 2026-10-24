const DEFAULT_ATTEMPTS: u32 = 3;
const DEFAULT_BACKOFF_MS: u64 = 100;

#[derive(Debug)]
struct RetryPolicy {
    max_attempts: u32,
    backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_ATTEMPTS,
            backoff_ms: DEFAULT_BACKOFF_MS,
        }
    }
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

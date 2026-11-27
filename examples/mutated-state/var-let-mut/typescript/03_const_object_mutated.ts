const RETRY_POLICY = { maxAttempts: 3, backoffMs: 200 };
const AGGRESSIVE_ATTEMPTS = 10;

function aggressiveRetry(policy: typeof RETRY_POLICY) {
  policy.maxAttempts = AGGRESSIVE_ATTEMPTS;
  return policy;
}

const aggressive = aggressiveRetry(RETRY_POLICY);
console.log("aggressive:", aggressive.maxAttempts);
console.log("default:", RETRY_POLICY.maxAttempts);

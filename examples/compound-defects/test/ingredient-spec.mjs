export const FAMILIES = {
  nondeterminism: ['mutated-state', 'shared-state'],
  'runtime-errors': ['mutated-state', 'unstructured-state'],
  'data-corruption': ['shared-state', 'unstructured-state'],
};
export const WORK = {
  nondeterminism: {
    'toctou-race': ['shared-state/race-conditions', 'mutated-state/improper-initialization'],
    'lost-update': ['shared-state/threading-mutexes', 'mutated-state/increment-decrement-operators'],
    'out-of-order-async': ['mutated-state/var-let-mut', 'shared-state/async-io-misuse'],
    'concurrent-modification': ['mutated-state/for-loop-control-flow', 'shared-state/threading-mutexes'],
    'pool-exhaustion-heisenbug': ['shared-state/resource-contention', 'mutated-state/time-money-entropy'],
    'stale-cache-heisenbug': ['shared-state/cache-invalidation', 'mutated-state/impure-functions'],
    'use-after-free': ['mutated-state/aliasing-mutable-defaults', 'shared-state/pointer-mismanagement'],
    'pooled-resource-heisenbug': ['mutated-state/impure-functions', 'shared-state/resource-leaks'],
    'stale-replica-read': ['mutated-state/var-let-mut', 'shared-state/async-io-misuse'],
    'predictable-id-collision': ['mutated-state/time-money-entropy', 'shared-state/race-conditions'],
    'temp-file-race': ['shared-state/file-network-access', 'mutated-state/time-money-entropy'],
    'concurrent-singleton': ['mutated-state/this-mutation', 'shared-state/cross-boundary-state-exposure'],
    'dispatch-order-divergence': ['mutated-state/macro-mixin-misuse', 'shared-state/async-io-misuse'],
  },
  'runtime-errors': {
    'null-dereference': ['mutated-state/improper-initialization', 'unstructured-state/missing-undefined-returns'],
    'type-confusion': ['mutated-state/this-mutation', 'unstructured-state/type-errors'],
    'shape-drift': ['mutated-state/improper-initialization', 'unstructured-state/duck-typing-without-contracts'],
    'aliased-default-type-confusion': ['mutated-state/aliasing-mutable-defaults', 'unstructured-state/type-errors'],
    'unexpected-type-element': ['mutated-state/this-mutation', 'unstructured-state/duck-typing-without-contracts'],
    'off-by-one-crash': ['mutated-state/increment-decrement-operators', 'unstructured-state/index-out-of-bounds'],
    'divide-by-zero-empty': ['mutated-state/var-let-mut', 'unstructured-state/divide-by-zero'],
    'quadratic-blowup': ['mutated-state/for-loop-control-flow', 'unstructured-state/unconstrained-inputs'],
    'nan-poisons-consumer': ['mutated-state/time-money-entropy', 'unstructured-state/lack-of-input-validation'],
    'integration-crash': ['mutated-state/impure-functions', 'unstructured-state/lack-of-input-validation'],
    'stale-api-upgrade': ['mutated-state/macro-mixin-misuse', 'unstructured-state/version-library-mismanagement'],
    'deserialized-type-confusion': ['unstructured-state/insecure-deserialization', 'mutated-state/this-mutation'],
    'config-drift-crash': ['mutated-state/impure-functions', 'unstructured-state/version-library-mismanagement'],
    'schema-drift-on-load': ['mutated-state/macro-mixin-misuse', 'unstructured-state/insecure-deserialization'],
  },
  'data-corruption': {
    'nan-shared-aggregate': ['unstructured-state/divide-by-zero', 'shared-state/threading-mutexes'],
    mojibake: ['shared-state/file-network-access', 'unstructured-state/version-library-mismanagement'],
    'unframed-stream': ['shared-state/async-io-misuse', 'unstructured-state/unconstrained-inputs'],
    'lossy-coercion-ledger': ['unstructured-state/type-errors', 'shared-state/threading-mutexes'],
    'signedness-poisons-aggregate': ['unstructured-state/type-errors', 'shared-state/resource-contention'],
    'cross-session-contamination': ['shared-state/cross-boundary-state-exposure', 'unstructured-state/lack-of-input-validation'],
    'schema-drift': ['shared-state/file-network-access', 'unstructured-state/unconstrained-inputs'],
    'stored-injection': ['shared-state/file-network-access', 'unstructured-state/lack-of-input-validation'],
    'untrusted-deserialization': ['shared-state/file-network-access', 'unstructured-state/insecure-deserialization'],
    'web-cache-poisoning': ['shared-state/cache-invalidation', 'unstructured-state/unconstrained-inputs'],
  },
};

Object.assign(WORK.nondeterminism, {
  'lazy-init-visibility-race': ['mutated-state/improper-initialization', 'shared-state/threading-mutexes'],
  'hash-order-divergence': ['mutated-state/impure-functions', 'shared-state/cache-invalidation'],
});
WORK['runtime-errors']['integer-overflow-buffer'] = ['mutated-state/increment-decrement-operators', 'unstructured-state/index-out-of-bounds'];
Object.assign(WORK['data-corruption'], {
  'buffer-overflow': ['shared-state/pointer-mismanagement', 'unstructured-state/index-out-of-bounds'],
  'stack-smash': ['shared-state/stack-overflow', 'unstructured-state/unconstrained-inputs'],
  'type-confusion-corruption': ['unstructured-state/type-errors', 'shared-state/pointer-mismanagement'],
  'torn-write': ['shared-state/race-conditions', 'unstructured-state/lack-of-input-validation'],
  'unchecked-sentinel-corruption': ['unstructured-state/missing-undefined-returns', 'shared-state/file-network-access'],
});

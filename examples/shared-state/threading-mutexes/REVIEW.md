# Threading & Mutexes audit

The five-language corpus has 30 original/fixed pairs: Go 8, Rust 5, JavaScript 5,
TypeScript 5, and Python 7. Query Lab sidecars hold independent source-token
capture expectations and behavioral oracles. Python capture expectations were
checked against Python AST spans; other languages were reviewed against source
calls, excluding declarations and Rust macro token trees.

The [base-defect inventory](audit.json) audits all 29 mutated, unstructured, and
shared-state definitions. Its counts distinguish examples, missing fixes and
catalog cells. It is an inventory and definition review, not a claim that every
existing detector has been semantically validated. Unrelated missing Python
repairs remain outside this change.

## Effects, propagation and combinations

Mutation, unknown inputs and concurrency are legitimate capabilities. Their
syntax marks potential review areas. Observable effects can propagate through
resolved calls; aliases can propagate exposure to the same object. Local mutation
does not automatically contaminate a caller. Guarded inputs are not permanently
unstructured, and a correct critical section remains Shared after a repair.

A compound requires connected evidence: for example, a saved value is written
back to the same resource after suspension. Co-located keywords do not establish
a compound. The existing race detector retains its ownership of synchronization
needed regions; the mutex analysis supplies recognized protection context rather
than emitting duplicate race diagnostics. Protection of one path does not prove
that every other access follows the same lock protocol.

Several original taxonomy summaries overstated what a token implies. The audit
clarifies that impurity is testable, mutable declarations are not inherently
incorrect, overflow behavior varies by language, inexact arithmetic does not
necessarily produce a different answer on every call, and stack exhaustion is
distinct from heap growth.

## Detection contract

Production and lab `boundaries.scm` packs capture complete operations and method
tokens. Legacy name-only lab packs remain historical artifacts. Queries receive
source only, never manifests, bug lines, filenames or expected outcomes.

State Lens recognizes Go sync types, Rust std/Tokio mutexes and reader/writer
locks, Python threading/asyncio locks, Web Locks, and a deliberately bounded
promise-tail wrapper protocol. Imports, local aliases, parameter substitution
and receiver fields supply resource identity. Unknown receiver methods remain
candidates. Dynamic dispatch, arbitrary third-party wrappers, escaped release
callbacks, lock poisoning recovery, unsafe code and whole-program scheduling are
not proved.

| Mechanism | Evidence and limitations |
|---|---|
| Critical section | Acquisition, explicit/deferred/context/guard release, and bounded source scope; conditional acquisition only marks the acquisition unless success is established. |
| Non-reentrant acquisition | Same lock within its hold lifetime or a resolved local call; direct witnesses can warn, uncertain paths remain candidates. RLock and nonblocking acquisition differ. |
| Missing release | An exit or cancellation can bypass explicit cleanup; ownership escape remains uncertain. |
| Lock order | Opposing concrete orders or direct call-site parameter substitutions; concurrent scheduling remains an assumption. |
| Copied lock | Go value receivers copy lock-bearing structs; whether the lock was already used and which fields remain shared require review. |
| Blocking across suspension | std/threading locks across suspension require review; holding an async mutex across await is not itself a defect. |
| Per-call lock | A newly created local lock surrounds externally reachable mutation; parameter locks are not allocations. |
| Callback lifetime | A recognized async call is neither returned nor awaited by a Web Locks callback. |
| Synchronization needed | Existing race evidence identifies conflicting access/interleaving; lock scope is attached where recognized. |

The catalog distinguishes five-role validated mechanisms from candidate-only
mechanisms. The five roles are failing, fixed, intentional, independent, and
breaker. These validate bounded models, not universal race/deadlock freedom.

## Editor behavior

The existing Shared label and palette remain unchanged. Hover and Evidence show
API, lock identity, assessment (usage/candidate/misuse), acquisition and release
locations. Only direct violation witnesses become warnings. Propagated usage
means the callee uses synchronization, not that the caller owns its lock.
Protected ranges exclude unrelated nested functions, and evidence retains its
originating URI/source hash across calls. Version/epoch rejection and direct-only
filtering still apply.

## Runtime verification

Run `query_lab.py test --defect threading-mutexes` and
`query_lab.py verify-fixes --defect threading-mutexes` from the repository root
with the script under `tooling/query-lab/`. Fixed Go/Rust programs run separately; original Go deadlock oracles use the
compiled binary exit status 2 (the manifest records the `go run` wrapper status 1);
Rust async examples use the existing pinned Tokio runtime. Original race examples
are not treated as deterministic proofs. Watchdogs bound deliberately blocked
programs; their timing is not a performance guarantee.

Four Web Locks examples are browser modules. Run them in a secure browser context
with Web Locks support, one example per page. Transpile the TypeScript modules
with the existing TypeScript toolchain before loading them in the browser. Query Lab explicitly reports their
runtime checks unsupported under the pinned Node 22 runtime; parsing, capture,
and analyzer tests still run. This change does not raise the Node requirement or
substitute a fake browser lock implementation.

## References

- [Go sync contracts](https://pkg.go.dev/sync): copying, acquisition and reader-lock constraints.
- [Rust Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html): same-thread relocking may deadlock or panic.
- [Rust temporary scopes](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes): guards in match scrutinees.
- [Tokio Mutex](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Mutex.html): async acquisition and guards held across await.
- [Python threading](https://docs.python.org/3/library/threading.html) and [asyncio synchronization](https://docs.python.org/3/library/asyncio-sync.html): thread ownership, reentrancy, cancellation and task exclusion.
- [Web Locks](https://www.w3.org/TR/web-locks/) and [Node workers](https://nodejs.org/api/worker_threads.html): runtime coordination available to JavaScript and TypeScript.

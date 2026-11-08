# Bugs of Shared State — language susceptibility

Score **1–5**: how likely the defect is to show up in everyday code in that language, and how many distinct, demonstrable forms it takes. Each cell has exactly that many example files. Pointer mismanagement is C-centric and is included as a tenth defect, using the closest memory-safe analogues.

| Defect | Go | Rust | TypeScript | Python |
|---|:-:|:-:|:-:|:-:|
| [race-conditions](race-conditions/) | 5 | 2 | 4 | 4 |
| [cross-boundary-state-exposure](cross-boundary-state-exposure/) | 4 | 2 | 5 | 5 |
| [resource-contention](resource-contention/) | 4 | 3 | 4 | 4 |
| [stack-overflow](stack-overflow/) | 2 | 3 | 4 | 3 |
| [async-io-misuse](async-io-misuse/) | 3 | 2 | 5 | 4 |
| [pointer-mismanagement](pointer-mismanagement/) | 3 | 2 | 2 | 2 |
| [cache-invalidation](cache-invalidation/) | 4 | 3 | 5 | 5 |
| [threading-mutexes](threading-mutexes/) | 5 | 3 | 2 | 4 |
| [file-network-access](file-network-access/) | 4 | 3 | 4 | 4 |
| [resource-leaks](resource-leaks/) | 4 | 2 | 4 | 4 |
| **Total** | **38** | **25** | **39** | **39** |

## Rationale

**race-conditions**
- Go 5: goroutines share memory freely. Lost `++`, check-then-act across two critical sections, concurrent map writes, lazy init without `sync.Once`, and racy `append`.
- Rust 2: data races are a compile error. Logical races remain: dropping the guard between check and act, and an atomic load followed by a store.
- TS 4: one thread, but every `await` is an interleaving point. Read-await-write, check-then-insert, whole-document saves, and out-of-order responses.
- Python 4: the GIL does not make check-then-act safe. A lost `+=` across threads, a balance check then subtract, an asyncio read then `await` then write, and lazy init without a lock.

**cross-boundary-state-exposure**
- Go 4: package globals, `sync.Pool` without `Reset`, fields on a shared handler, and the global logger prefix.
- Rust 2: `thread_local!` on a pooled worker and a `static Mutex` singleton. Both need deliberate global state.
- TS 5: module state is shared by every request in a server. Module globals, object pools, static caches, shared default objects, and module trace ids.
- Python 5: a module `current_user`, a shared context dict, a class cache keyed only by sku, a module trace id read at log time, and `@lru_cache` on a lookup that reads the module user.

**resource-contention**
- Go 4: unbounded fan-out, a semaphore with no deadline, a `Transport` per request, and an unbuffered channel to a slow consumer.
- Rust 3: a thread per task, an unbounded `mpsc`, and a guard held across slow work.
- TS 4: `Promise.all` over everything, CPU work blocking the event loop, an unbounded queue, and a pool slot leaked on the error path.
- Python 4: a thread per item, a `Queue` with no max size, a lock held across a wait, and `asyncio.gather` of every item. The counts stay small.

**stack-overflow**
- Go 2: goroutine stacks grow to 1 GB, so only true unbounded recursion fails: cyclic dependencies and recursive retry.
- Rust 3: an 8 MB main stack. Recursion on cyclic `Rc` graphs, the recursive `Drop` of a `Box` list, and large arrays on the stack.
- TS 4: recursion on cyclic objects, spreading huge arrays, synchronous event feedback loops, and runaway retry growth.
- Python 3: `sys.setrecursionlimit` is harness. A cyclic graph, mutual recursion, and a recursive retry each raise `RecursionError` immediately.

**async-io-misuse**
- Go 3: a goroutine's error dropped, `main` exiting before workers, and senders leaked on an unbuffered result channel.
- Rust 2: lazy futures that are never awaited, and a dropped `JoinHandle` that hides a panic.
- TS 5: floating promises, unhandled rejections, `await` in a loop, `.then` without `return`, and `Promise.race` that doesn't cancel the loser.
- Python 4: a coroutine that is never awaited, a task exception that is dropped, `await` inside the loop, and a race whose loser still writes. No network.

**pointer-mismanagement**
- Go 3: an element pointer kept across an `append` reallocation, nil from a map lookup, and a pointer into a recycled ring slot.
- Rust 2: only reachable in `unsafe`: a raw pointer after `Vec` reallocation (use-after-free), and `Box::from_raw` twice (double free).
- TS 2: no pointers. A detached `ArrayBuffer` after transfer, and a stale reference to a replaced store object.
- Python 2: no raw pointers. A `memoryview` kept across a `bytearray` resize, and a `weakref` to a cache row that has been dropped. No `ctypes`.

**cache-invalidation**
- Go 4: a write without invalidation, negative results cached forever, a thundering herd, and a cached pointer that callers mutate.
- Rust 3: a missing invalidation, a `OnceLock` that never refreshes, and a key that omits a dependency.
- TS 5: a stale read after update, an identity-keyed memo, a herd on miss, a cached rejected promise, and a mismatched invalidation key.
- Python 5: a write that does not delete the cache entry, a negative `None` cached forever, a key that omits the tenant, a cached dict the caller mutates, and a delete of the wrong key.

**threading-mutexes**
- Go 5: lock-order deadlock, a non-reentrant `Mutex`, unlock skipped on early return, a copied mutex, and recursive `RLock` with a pending writer.
- Rust 3: lock-order deadlock, relocking on the same thread, and a guard kept alive by a `match` scrutinee.
- TS 2: no threads. Hand-rolled async locks still leak on throw and deadlock on order inversion.
- Python 4: `Lock` is not reentrant. Opposite lock order, a nested `acquire`, a `release` skipped after an exception, and a nested call that needed `RLock`. A blocked acquire uses a timeout and prints `blocked`.

**file-network-access**
- Go 4: TOCTOU stat-then-write, truncate-then-encode, an ignored `Close` error, and retrying a non-idempotent POST.
- Rust 3: TOCTOU `exists()`, `write` instead of `write_all`, and a duplicate order after a read timeout.
- TS 4: TOCTOU `existsSync`, truncate-then-serialize, `fetch` with no timeout, and a retry without an idempotency key.
- Python 4: exists-then-write, truncate-then-serialize, a close failure ignored on a small wrapper, and a retry that applies a charge twice. Temp files only. No sockets.

**resource-leaks**
- Go 4: `defer` in a loop, `Close` skipped on the error path, a `time.Tick` goroutine per request, and an unclosed `resp.Body`.
- Rust 2: RAII covers most cases. `Rc` cycles and `mem::forget` on a guard remain.
- TS 4: listeners never removed, intervals never cleared, a file handle left open on throw, and an unbounded request-keyed cache.
- Python 4: a file left open on the error path (counted on a wrapper, not by exhausting descriptors), a callback never removed, a `Timer` never cancelled, and a module list of requests that only grows.

## Harness conventions

Shared-state bugs depend on timing and on running out of real resources, so the `main` functions contain a few support lines. None of them are counted in `bug_lines`:
- Deadlocks end through Go's runtime deadlock detector, or through a watchdog (`recv_timeout`, `setTimeout` + `process.exit`) that prints that the work is still blocked.
- Exhaustion is bounded. Go stack depth is capped with `debug.SetMaxStack`, and the file-descriptor limit is lowered with `Setrlimit`. Growth is printed instead of being allowed to run until it crashes the host.
- Races are made reproducible with short sleeps or many iterations. Their manifests say `deterministic: false`, and each was verified to show the bug in at least 9 of 10 runs.
- All network traffic stays on loopback (`httptest`, `node:http` on port 0, `TcpListener` on `127.0.0.1:0`). Files go under the OS temp directory.
- Python caps recursion with `sys.setrecursionlimit`. That call is harness, not a bug line. A blocked `Lock` uses `acquire(timeout=...)` and prints `blocked`. The lost-update counter yields with `time.sleep(0)` between the read and the write and is marked `deterministic: false`. The other Python races use a `Barrier` or an `Event` and are deterministic. Python examples open temp files only, never sockets, and count a wrapper's opens instead of exhausting file descriptors.

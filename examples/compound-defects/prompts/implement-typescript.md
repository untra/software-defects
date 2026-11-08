# Task

Implement the remaining **TypeScript** compound-defect examples for this repo, and add tests that each example **contains or suggests both stateful defects** that compose it.

Read these before writing anything:

- `examples/compound-defects/ASSESSMENT.md` — the 0–2 score for TypeScript. Obey it.
- `examples/compound-defects/README.md` — layout and house rules.
- `examples/README.md` — manifest fields and the blind-corpus rules.
- `examples/compound-defects/nondeterminism/out-of-order-async/` — the existing TypeScript flagship. Match its size and style. Do not replace `typescript/01_last_write_wins.ts`. Add it to the test suite.

Work only under `examples/compound-defects/`. Do not edit `src/`, `site/`, `examples-fixed/`, or `tooling/query-lab/`. Do not flatten this tree to two levels. Do not create git commits.

## What to build

One TypeScript program per compound defect whose **TS** column in `ASSESSMENT.md` is **1 or 2**. Skip every TS **0**. That is eight skips: `lazy-init-visibility-race`, `hash-order-divergence`, `integer-overflow-buffer`, `buffer-overflow`, `stack-smash`, `type-confusion-corruption`, `torn-write`, `unchecked-sentinel-corruption`. A lookalike for a 0 would be a different CWE. Do not write those files.

36 new programs, plus tests that also cover the existing `out-of-order-async` program. 37 tested programs in all.

Put each new file at:

```
examples/compound-defects/<class>/<slug>/typescript/01_<domain>.ts
```

`<class>` is `nondeterminism`, `runtime-errors`, or `data-corruption`. `<slug>` is the defect slug below. `01_` is the only file for TypeScript in that folder.

If `manifest.json` already exists, append one object to `examples`. Leave every other language entry, and leave `scores`, as they are. If it does not exist, create it with `family` `compound-defects`, the `class`, the `category` from the work list, the compound `cwes`, and the `scores` object copied from the assessment row (all four languages, not only TypeScript).

Each TypeScript manifest entry needs the usual fields (`file`, `language`, `bug_lines`, `summary`, `mechanism`, `observed_output`, `exit_code`, `deterministic`, `expected_behavior`) and this extra array, naming the two ingredients in the work list:

```json
"ingredients": [
  {
    "family": "mutated-state",
    "defect": "var-let-mut",
    "suggests": "one sentence a reader can check against the source",
    "evidence": ["exact source substring", "another substring"]
  },
  {
    "family": "shared-state",
    "defect": "async-io-misuse",
    "suggests": "one sentence",
    "evidence": ["substring"]
  }
]
```

`evidence` strings must be copied from the program you wrote. They are how the test checks that the ingredient is present. Do not put the words bug, CWE, race, or the defect slug in the program. Names are domain words. Constants are named. No comments that explain the defect.

`mechanism` for a score **1** must name the escape hatch. For TypeScript the only allowed hatches are `SharedArrayBuffer`, a detached `ArrayBuffer`, and `>>> 0`. WebAssembly and native addons are out of bounds.

## Language rules

- Erasable TypeScript only. No `enum`, no `namespace`, no parameter properties. `node` 22 runs the file directly (`node path.ts`).
- One entry point. Running the file prints a wrong result or crashes. No test framework inside the example.
- No network. Temp files only if the defect needs a path, under `os.tmpdir()`.
- Do not hang. Cap loops, pools, and recursion. A quadratic example prints a comparison count for a small input.
- Stored injection prints the query or the markup. It does not open a database. Deserialization sets a flag from JSON. It does not use a gadget chain.
- Prefer `deterministic: true`. Use an explicit gate (see `01_last_write_wins.ts`) instead of `setTimeout` when order is the bug. If the outcome really depends on timing, set `deterministic: false` and make the bug show in at least 9 of 10 runs.
- `bug_lines` are 1-based lines a detector should flag. Do not flag harness lines (a gate, a temp directory, a window that only makes the bug visible).
- `observed_output` is what a real run prints. On a crash, include the stdout line if there is one, then the single error line (`TypeError: ...`), not the stack. `exit_code` is the real exit code.

## Tests

Add `examples/compound-defects/test/typescript-ingredients.test.mjs` and run it with `node --test`. No new dependencies.

The test loads every `manifest.json` under `examples/compound-defects/*/*/` that has a `typescript/` example. For each of those examples it asserts all of the following:

1. **Both ingredients are present.** `ingredients` has length 2. Their `family` values are exactly the pair for that class:
   - `nondeterminism`: `mutated-state` and `shared-state`
   - `runtime-errors`: `mutated-state` and `unstructured-state`
   - `data-corruption`: `shared-state` and `unstructured-state`
   The order in the array may follow the work list. The set of families must match. The `defect` slugs must be the two in the work list for that compound, not two other defects from the same families.
2. **The source suggests each ingredient.** Every `evidence` string occurs in the `.ts` file. `suggests` is non-empty. The test failure message includes both defect slugs and both `suggests` sentences, so a miss reads as "this file does not suggest `var-let-mut` and `async-io-misuse`".
3. **Evidence is not a label.** Fail if an evidence string is only the defect slug, the family name, or the word "bug".
4. **Score rules.** Read the TypeScript score from the manifest `scores.typescript`. If it is 0, fail because the file should not exist. If it is 1, the source must contain the escape hatch named in `mechanism` (`SharedArrayBuffer`, `detach`, or `>>> 0`).
5. **Behavior.** Spawn `node` on the file with a 10 second timeout. Deterministic examples must exit with `exit_code`, and the combined stdout and stderr must contain `observed_output`. Non-deterministic examples must do that on at least 9 of 10 runs.
6. **Coverage.** The set of tested slugs must equal the work list below, including `out-of-order-async`, and must not include the eight score-0 slugs.

Do not import the example into the test. The example runs on load. Spawn it.

## Work list

Each row is one program. The two defects in **Composed of** are the ingredients the test must find. Read a short example of each ingredient under `examples/<family>/<defect>/typescript/` when that folder exists, so the compound uses the same mechanism, then write a new scenario. Do not copy an ingredient file in unchanged.

### Nondeterminism (mutated × shared)

Skip `lazy-init-visibility-race` (TS 0) and `hash-order-divergence` (TS 0). `out-of-order-async` already has a program; only add `ingredients` and the test.

| Slug | Category | TS | CWE | Composed of | TypeScript shape |
|---|---|:-:|---|---|---|
| `toctou-race` | temporal-interleaving | 2 | CWE-367 | `shared-state/race-conditions` × `mutated-state/improper-initialization` | `existsSync` then `mkdirSync` on one path. The check and the create are separate statements. |
| `lost-update` | temporal-interleaving | 1 | CWE-567 | `shared-state/threading-mutexes` × `mutated-state/increment-decrement-operators` | `worker_threads` plus a `SharedArrayBuffer`. Increment with separate load and store, not `Atomics.add`. Say `SharedArrayBuffer` in `mechanism`. |
| `out-of-order-async` | temporal-interleaving | 2 | CWE-362, CWE-696 | `mutated-state/var-let-mut` × `shared-state/async-io-misuse` | Already `typescript/01_last_write_wins.ts`. Do not rewrite it. |
| `concurrent-modification` | temporal-interleaving | 1 | CWE-362, CWE-567 | `mutated-state/for-loop-control-flow` × `shared-state/threading-mutexes` | One worker walks a `SharedArrayBuffer` while another writes slots, without `Atomics`. Not a single-threaded `splice` during `forEach` (that bug is deterministic and is a different defect). |
| `pool-exhaustion-heisenbug` | temporal-interleaving | 2 | CWE-362 | `shared-state/resource-contention` × `mutated-state/time-money-entropy` | A pool of size 1. A holder keeps the slot across an `await`. The second caller reports the pool is busy. Bound the wait. |
| `stale-cache-heisenbug` | observational-staleness | 2 | CWE-471 | `shared-state/cache-invalidation` × `mutated-state/impure-functions` | A `Map` caches the result of a function that reads a mutable rate. The rate changes. The next call returns the cached number. |
| `use-after-free` | observational-staleness | 1 | CWE-825 | `mutated-state/aliasing-mutable-defaults` × `shared-state/pointer-mismanagement` | Two names for one `ArrayBuffer`. `structuredClone` transfers it. The other name is then read and throws because it is detached. Say so in `mechanism`. |
| `pooled-resource-heisenbug` | observational-staleness | 2 | CWE-459 | `mutated-state/impure-functions` × `shared-state/resource-leaks` | A connection object is returned to a pool without clearing `tenant`. The next borrower sees the previous tenant. |
| `stale-replica-read` | observational-staleness | 2 | CWE-821 | `mutated-state/var-let-mut` × `shared-state/async-io-misuse` | Write the primary, then read a copy that is applied only after a gate you open later. The read sees the old value. No sockets. |
| `predictable-id-collision` | entropic-divergence | 2 | CWE-340, CWE-362 | `mutated-state/time-money-entropy` × `shared-state/race-conditions` | An id built from one shared millisecond counter. Two callers in the same tick get the same id. Do not depend on the wall clock; freeze the tick. |
| `temp-file-race` | entropic-divergence | 2 | CWE-363, CWE-59 | `shared-state/file-network-access` × `mutated-state/time-money-entropy` | A path from a fixed prefix plus a predictable stamp. `existsSync`, then `writeFileSync`. Between them, plant a different file at that path in-process. Do not need a symlink exploit. |
| `concurrent-singleton` | entropic-divergence | 2 | CWE-488 | `mutated-state/this-mutation` × `shared-state/cross-boundary-state-exposure` | One module-level object. Two calls each set `user` on it. The second call's read sees the first call's user, or the reverse. |
| `dispatch-order-divergence` | entropic-divergence | 2 | CWE-758 | `mutated-state/macro-mixin-misuse` × `shared-state/async-io-misuse` | Two handlers register on a shared route table during `await` import-style initialization. Which one is stored depends on which registration finishes last. Use gates, not the filesystem. |

### Runtime errors (mutated × unstructured)

Skip `integer-overflow-buffer` (TS 0). Empty-array division is `NaN` here, not a throw. Show that `NaN` when the row says so.

| Slug | Category | TS | CWE | Composed of | TypeScript shape |
|---|---|:-:|---|---|---|
| `null-dereference` | structural-invalidation | 2 | CWE-476, CWE-690 | `mutated-state/improper-initialization` × `unstructured-state/missing-undefined-returns` | A lookup falls off the end and returns `undefined`. The caller reads `.cents` and throws. |
| `type-confusion` | structural-invalidation | 2 | CWE-843 | `mutated-state/this-mutation` × `unstructured-state/type-errors` | A field is stored as a number, then overwritten with a string from a form. A later `toFixed` throws. |
| `shape-drift` | structural-invalidation | 2 | CWE-754 | `mutated-state/improper-initialization` × `unstructured-state/duck-typing-without-contracts` | `seat` is set only when `kind` is `"reserved"`. A rush ticket then calls `seat.toUpperCase()` and throws. |
| `aliased-default-type-confusion` | structural-invalidation | 2 | CWE-843 | `mutated-state/aliasing-mutable-defaults` × `unstructured-state/type-errors` | A default `[]` parameter is shared. One call pushes a string. A later call reads `.sku` and throws. Same shape as the Python flagship, in TypeScript. |
| `unexpected-type-element` | structural-invalidation | 2 | CWE-241 | `mutated-state/this-mutation` × `unstructured-state/duck-typing-without-contracts` | A list of line objects. One line's `cents` is replaced with a string. The consumer calls `toFixed` on each element and throws. `+` is the wrong demonstration: it concatenates. |
| `off-by-one-crash` | computational-exhaustion | 2 | CWE-125, CWE-787 | `mutated-state/increment-decrement-operators` × `unstructured-state/index-out-of-bounds` | A counter advances while `i <= items.length`, then reads `.sku` of the missing slot and throws. This is a trapped crash, not a memory write. |
| `divide-by-zero-empty` | computational-exhaustion | 2 | CWE-369, CWE-754 | `mutated-state/var-let-mut` × `unstructured-state/divide-by-zero` | A running total divided by `samples.length` when `samples` is empty. Print the `NaN`. Do not throw unless a later consumer rejects the `NaN`; if you do, the `NaN` must be the cause. |
| `quadratic-blowup` | computational-exhaustion | 2 | CWE-400, CWE-407 | `mutated-state/for-loop-control-flow` × `unstructured-state/unconstrained-inputs` | For each name, scan the whole list accumulated so far. Print the comparison count for four names (6). No cap is applied to the input. |
| `nan-poisons-consumer` | computational-exhaustion | 2 | CWE-754 | `mutated-state/time-money-entropy` × `unstructured-state/lack-of-input-validation` | A rate becomes `NaN` from an unvalidated blank input. A table lookup uses `Math.round(rate)` and then calls a method on `undefined`, which throws. |
| `integration-crash` | architectural-drift | 2 | CWE-248, CWE-754 | `mutated-state/impure-functions` × `unstructured-state/lack-of-input-validation` | A payload is written onto a cart without checking `qty`. The next line calls `toFixed` on the missing quantity and throws. |
| `stale-api-upgrade` | architectural-drift | 2 | CWE-758, CWE-754 | `mutated-state/macro-mixin-misuse` × `unstructured-state/version-library-mismanagement` | A helper still calls `amount` on an object whose method was renamed to `totalCents`. The call throws. |
| `deserialized-type-confusion` | architectural-drift | 2 | CWE-704, CWE-843 | `unstructured-state/insecure-deserialization` × `mutated-state/this-mutation` | `JSON.parse` yields `reading: "10"`. The object then increments `reading` or calls `toFixed`, and throws. |
| `config-drift-crash` | architectural-drift | 2 | CWE-248, CWE-754 | `mutated-state/impure-functions` × `unstructured-state/version-library-mismanagement` | Set `process.env.DB_URL` inside the program. Read `process.env.DATABASE_URL` and call `.slice` on it. That throws. Restore the previous env value before exiting if you set anything. |
| `schema-drift-on-load` | architectural-drift | 2 | CWE-915 | `mutated-state/macro-mixin-misuse` × `unstructured-state/insecure-deserialization` | A loader copies every key from a parsed record with a dynamic assignment. The payload has `colour`. The next line reads `color.toUpperCase()` and throws. |

### Data corruption (shared × unstructured)

Skip all five memory-corruption slugs (`buffer-overflow`, `stack-smash`, `type-confusion-corruption`, `torn-write`, `unchecked-sentinel-corruption`). TypeScript cannot write past a buffer. `signedness-poisons-aggregate` is score 1: use `>>> 0`, not a C-style overflow.

| Slug | Category | TS | CWE | Composed of | TypeScript shape |
|---|---|:-:|---|---|---|
| `nan-shared-aggregate` | semantic-corruption | 2 | CWE-682 | `unstructured-state/divide-by-zero` × `shared-state/threading-mutexes` | A `NaN` quote is added into a module-level total while a lock flag is held. The lock does not stop the poison. Print the `NaN` total. |
| `mojibake` | semantic-corruption | 2 | CWE-172 | `shared-state/file-network-access` × `unstructured-state/version-library-mismanagement` | Write bytes as latin-1, read them as utf-8, store the string. Print the corrupted text. Use a temp file. |
| `unframed-stream` | semantic-corruption | 2 | CWE-240 | `shared-state/async-io-misuse` × `unstructured-state/unconstrained-inputs` | A byte list with no length prefix is read in fixed chunks of 4. Two records of different sizes split and merge. Print the mis-framed records. |
| `lossy-coercion-ledger` | semantic-corruption | 2 | CWE-682 | `unstructured-state/type-errors` × `shared-state/threading-mutexes` | A float is narrowed with a bitwise truncation, then added to a shared total under a lock. Print the total that lost the fraction. |
| `signedness-poisons-aggregate` | semantic-corruption | 1 | CWE-682, CWE-195 | `unstructured-state/type-errors` × `shared-state/resource-contention` | A refund of `-1` is converted with `>>> 0` and added to a shared balance. Print the huge balance. Name `>>> 0` in `mechanism`. |
| `cross-session-contamination` | representational-corruption | 2 | CWE-488 | `shared-state/cross-boundary-state-exposure` × `unstructured-state/lack-of-input-validation` | Per-request `locale` is written onto one module object with no check. The next request reads the previous locale. |
| `schema-drift` | representational-corruption | 2 | CWE-345 | `shared-state/file-network-access` × `unstructured-state/unconstrained-inputs` | Two writers append JSON lines to one temp file. One line has no version. The reader accepts it and prints the wrong field. |
| `stored-injection` | representational-corruption | 2 | CWE-89, CWE-79 | `shared-state/file-network-access` × `unstructured-state/lack-of-input-validation` | Save a name, then build a query by concatenation on the way back out. Print the query. The name contains a quote. Do not execute it. |
| `untrusted-deserialization` | representational-corruption | 2 | CWE-502 | `shared-state/file-network-access` × `unstructured-state/insecure-deserialization` | Read a JSON body that crossed a boundary (a string is enough) and copy `admin: true` onto the account. Print `admin true`. No gadget chain. |
| `web-cache-poisoning` | representational-corruption | 2 | CWE-349 | `shared-state/cache-invalidation` × `unstructured-state/unconstrained-inputs` | The cache key is only the path. A query parameter changes the stored body. The next request for the path gets the poisoned body. |

## Done when

- `node --test examples/compound-defects/test/typescript-ingredients.test.mjs` passes from the repo root.
- Each new program has been run once with `node`, and the manifest `observed_output` and `exit_code` match that run.
- `python3 -m unittest test_build.py` from `src/` still passes.
- `python3 -m unittest test_query_lab.CorpusTests` from `tooling/query-lab/` still passes. These manifests are three levels down so those tests keep the old case count. If a test count changes, the new file is in the wrong place.

## Do not

- Do not add a TypeScript file for a score of 0.
- Do not add `examples-fixed/` copies or Query Lab sidecars.
- Do not put defect hints in the example source. The ingredients live in the manifest and the test.
- Do not use `any` as a way to skip a shape the work list already shows with a crash. `any` is acceptable only where the erased type is the defect itself (`type-confusion`, `unexpected-type-element`, `deserialized-type-confusion`).

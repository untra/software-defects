# var-let-mut repair review

All 18 examples have corresponding files under `examples-fixed/mutated-state/var-let-mut/`, with matching relative paths. Query Lab sidecars supply source hashes, independently specified capture expectations, behavioral oracles, repair rationale, and analysis limits. The original manifests retain bug locations, not expected query captures.

## Detection and transformation limits

Production queries identify mutable declarations and writes (Python assignments are both bindings and writes). They do not resolve scope, aliases, control flow, or intent. Experimental Go queries in this category capture `Atoi`, Rust queries capture every `let`, and TypeScript queries capture `push` and `min`. Those are syntax candidates, not validated bug diagnostics: the new Go pointer and TypeScript async cases have explicit empty experimental captures. Python pairs use the production query directly; the runner supports Python parsing and execution despite the older Query Lab README description.

A correct repair need not eliminate captures. Removing all `mut`, `var`, or `let` occurrences is neither necessary nor sufficient. An automatic repair must first establish the intended state lifetime, whether writes are required, whether references escape, and what callers expect. These pairs demonstrate transformations; they do not authorize blind rewrites.

## Existing examples

| Language / example | Minimal repair | Evidence required before automation |
|---|---|---|
| Go 01 shadowed error | Assign the outer results with `=` | Resolve both scopes and error propagation. |
| Go 02 closure | Capture a parameter in a handler factory | Identify when handlers execute; values containing references need further analysis. |
| Go 03 defer | Restore balance only in the recovery branch | Preserve successful return and panic recovery semantics. |
| Go 04 currency | Pass currency explicitly through a helper | Update callers; retaining a global write and restoring it is unsafe under overlap. |
| Rust 01 shadowed total | Update the accumulator | The intended running total requires mutation; removing `mut` would not repair it. |
| Rust 02 reset | Remove the per-order reset | Establish that totals span all orders. |
| TypeScript 01 hoisting | Initialize the rate before the branch | Establish the no-discount default. |
| TypeScript 02 tenant | Pass tenant as an argument | Snapshot before the async boundary; a mutable object would require more analysis. |
| TypeScript 03 policy | Return a copied object with the override | Binding immutability does not prevent field mutation; shallow copy suffices for these scalar fields. |
| TypeScript 04 page size | Initialize inside each iteration | Overrides apply to one query only. |
| Python 01 closure | Capture the string as a default argument | Capture at construction, not invocation; this is not deep copying. |
| Python 02 search | Return on match; explicit missing result | Define missing and empty input behavior. |
| Python 03 subtotal | Return a computed value without rebinding | Keep the shared base unchanged across calls; actual output is `44`. |
| Python 04 rate | Explicit per-call rate | Requires changing the setter API and its caller; removing `global` alone would hide intent. |

## Added examples

- Rust 03: fixed-threshold filtering of `[20, 15, 30]` must yield `[40, 30, 60]`. The supplied `scan` preserves threshold drift and returns `None` on rejection, ending collection early. The repair removes the threshold write and `mut`, retaining the local output buffer. Running-maximum behavior would be a different contract.
- Go 05: initialize a task inside each iteration before storing its address. Check both values and pointer identity. `:=` creates a binding, not an immutable value; the important change is its lifetime, including under Go 1.22 semantics.
- TypeScript 05: assignments before the first `await` run synchronously. API arguments are already evaluated correctly; post-await logging is wrong. Use the per-invocation parameter for both. A callback parameter is not a `const` declaration, but no reassignment occurs here.
- Python 05: adapt the collection example into a real cross-call accumulator leak. Two calls with `[30, 25]` should each yield `[50]`. Moving initialization into the function is sufficient. The original Kotlin snippet has no demonstrated behavioral defect; a functional rewrite alone would be a benign alternative, not a bug repair or a general thread-safety guarantee.

## Verification

Run `python3 tooling/query-lab/query_lab.py test --defect var-let-mut` for captures and `python3 tooling/query-lab/query_lab.py verify-fixes --defect var-let-mut` for fixed outputs. Run `python3 -m unittest discover -s tooling/query-lab -p 'test_var_let_mut.py'` for edge cases and pair metadata integrity. Fixtures execute individually because each file is a standalone program.

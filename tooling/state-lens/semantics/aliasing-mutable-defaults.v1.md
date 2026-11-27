# aliasing-mutable-defaults — semantic specification v1

Status per language and mechanism is recorded in [`queries/catalog.json`](../../../queries/catalog.json). Every cell is checked by a generated test, `tests/matrix.rs`. This document defines what a supported cell means.

## Rule

A finding requires all of A–D, established from shared-graph facts (`StateGraph.heap`, `sharing`, `edges` with `heap_dependency`):

| | Condition | Graph fact |
|---|---|---|
| A | An allocation or shared identity exists | `Allocation` operation / `allocations[]` |
| B | Two or more access paths reach that identity | `SharingIntroduction` (`sharing[]`) on one path but not the other |
| C | A write through one path changes a field or element | `Write` operation with an `access_path` |
| D | A later operation observes the changed state through a connected path | `heap_dependency` edge write → read |
| E | A required isolation, value or ownership property may fail | `StateTrace.requirement` from an in-source declaration |

A–D produce `invalidation_status: potentially_invalidated` and Information severity. They describe a state relationship, not a bug.

E promotes a finding to `invalidated`. If the heap dependency is also definite and no uncertainty boundary is touched, the evidence level is `feasible_under_model` and the LSP severity is Warning.

Analysis inputs never include filenames, defect labels, manifests or expected bug lines. Intent is never inferred from variable names, aliases, or the fixed program.

## Mechanism predicates

Each write W and observation R carries the set of sharing events on the reference path that reached the object.

| Mechanism | Sharing introduction | Predicate |
|---|---|---|
| `repeated-element` | `fill(v)` on a tracked array; `[x] * n` | W's element index ≠ R's element index for the same introduction. An unknown index gives `possible`. |
| `shallow-copy-nested` | object/array spread, `Object.assign({}, …)`, `dict()`/`list()`/`.copy()`/`copy.copy`, Go struct value copy | W's path crosses the copy, R's path does not. |
| `shared-default` | default value whose identity predates the call; Python defaults are evaluated at definition | W and R hold the default in different invocation contexts, or R reaches it without the default. |
| `argument-returned-alias` | callee returns an object it did not allocate | W's path went through the return, R's did not. |
| `internal-state-escape` | callee returns a field of its receiver or parameter | same as above. |
| `append-within-capacity` | Go `s[a:b]` with spare constant capacity, then `append` | the append writes in place (`len + n <= cap`) through the resliced path; R does not cross the reslice. |

### Intentional sharing

A shared default consulted by a guard inside the callee before the write, such as `key in store` or `n in memo`, is treated as a cache by design and is not reported.

Other intentional sharing still produces a candidate relationship, because no contract says otherwise. Repairs for those findings are at most `advisory`.

## Language semantics

| | Allocation | Copy | Default | Notes |
|---|---|---|---|---|
| JS/TS | object/array literals, `new Array(n)`, `Array.from({length}, f)` (per element) | spread and `Object.assign({}, …)` are shallow and run in evaluation order; `structuredClone` is deep | evaluated per call: an outer binding is shared, a literal is fresh | `fill` is modeled only on a tracked array receiver. Shadowed `fill`, accessors, `Proxy`, computed keys, classes and `this` are boundaries. |
| Python | list/dict/set literals and comprehensions | `.copy()`, `dict()`, `list()`, `copy.copy` are shallow; `copy.deepcopy` is deep | evaluated once at `def` | Rebinding (`x = x + [y]`) is not mutation. Classes and decorators are boundaries. |
| Go | composite literals, `make`, `&T{}` | struct assignment and value parameters copy fields shallowly; maps and slices are references | none | Slices carry backing array, offset, length and constant capacity. `s[a:b:c]` caps the slice. A method resolves only on a statically known concrete receiver type. |
| Rust | — | — | — | Not modeled. The borrow checker excludes shared mutation; `Rc<RefCell<_>>` is a candidate-only escape hatch. |

### Requirement sources (condition E)

- TS: `Readonly<…>`, `ReadonlyArray<…>`, `readonly` type annotations, `as const`.
- JS/TS: `Object.freeze(x)`.
- Python: `Sequence[…]`, `Mapping[…]`, `AbstractSet[…]`, `tuple[…]`, `frozenset[…]` annotations on bindings or parameters.
- Go has no in-source declaration form, so Go findings stay candidates.
- A declaration on an owner is treated as covering state reachable from it. This assumption is recorded in the trace.

## Uncertainty boundaries

Each of these is recorded in `StateGraph.boundaries` with a kind:
- `unknown-call`: unresolved functions, methods, imports and cross-file callees.
- `opaque-member` and `opaque-object`: accessors, methods and computed keys in object literals.
- `dynamic-key`.
- `call-bound`: recursion, or depth beyond `MAX_CALL_DEPTH`.
- `unsupported-statement`: classes, `with`, `defer`/`go`, unpacking.
- `unmodeled-expression`: await, constructors, and other expressions outside the model.

Objects reachable from a boundary are marked escaped. Findings that touch them stay `candidate_relationship`.

## Analysis limits

All limits are named constants in `src/heap/interp.rs`:
- `MAX_STEPS`
- `MAX_CALL_DEPTH`
- `MAX_UNROLL`
- `MAX_OBSERVE_DEPTH`
- `MAX_REFS`

Loop bodies run once; facts inside them are `possible`. Exceptional completion is approximated. Exhausting the budget records a limitation.

## Repair

`state-lens repair-check FILE [--finding N] [--hash SHA256]` works in three steps:

1. **Postcondition.** It is chosen from the mechanism:
   - distinct element identities;
   - an independent nested object;
   - per-invocation default state;
   - an independent returned object;
   - non-overlapping append storage.
2. **Candidates.** It generates candidate edits, such as per-element allocation, a copy of the mutated segment, a copied or `None` default, a copy on return, or a capped subslice.
3. **Verification.** Each candidate is verified on an in-memory snapshot. The snapshot must reparse without errors and be reanalyzed, the targeted heap dependency must be gone, and no new object-identity relationship may appear. Deeper shared state left behind is reported.

| Applicability | When |
|---|---|
| `auto` | Verified, and the finding has a declared requirement |
| `advisory` | Verified, but no requirement |
| `ambiguous` | Verified strategies leave different post-states |
| `rejected` | Nothing verified, the source is stale, or no strategy applies. For example, a shallow copy on return still shares nested state. |

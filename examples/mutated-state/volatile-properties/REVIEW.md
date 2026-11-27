# Volatile properties: collection mutation

Contract: transform a working collection without changing the source. Each original fails its source-preservation assertion; each mirrored repair checks the result, relevant return values and the unchanged source. All fixtures use scalar elements. Copying is deliberately shallow and is not a general repair for nested objects or side-effecting callbacks.

"Volatile" is a taxonomy term here, not a hardware or memory-model qualifier. A mutating operation is a review signal, not inherently a bug. Rust allows mutation through an exclusive reference even without a mutable binding; the borrow checker does not establish the application's preservation contract.

Tree-sitter captures `callee` and `receiver`, or `operation` and `receiver` for indexed/slice assignment. Names cannot prove standard-library identity. Custom methods and shadowed Go built-ins can match. Assignment captures also include non-collection receivers. Copy-before-mutate repairs intentionally retain captures. No automatic AST fix is claimed: ownership, aliases, allocation cost, evaluation order and return values must be established first.

Go append reuses available capacity in the fixture and overwrites an element visible through the source. Allocation on exhausted capacity does not itself overwrite that backing array; do not diagnose every append as a defect. Copy writes its destination. Rust swap_remove changes order; dedup removes adjacent duplicates, not all duplicates. Python remove removes the first equal item and can raise; pop on an empty list raises. Repairs retain these semantics.

## Operations and paired repairs

| Operation | Original | Repair | Preconditions |
|---|---|---|---|
| typescript `push` | [typescript/01_push.ts](typescript/01_push.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/01_push.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `pop` | [typescript/02_pop.ts](typescript/02_pop.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/02_pop.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `shift` | [typescript/03_shift.ts](typescript/03_shift.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/03_shift.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `unshift` | [typescript/04_unshift.ts](typescript/04_unshift.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/04_unshift.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `splice` | [typescript/05_splice.ts](typescript/05_splice.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/05_splice.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `sort` | [typescript/06_sort.ts](typescript/06_sort.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/06_sort.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `reverse` | [typescript/07_reverse.ts](typescript/07_reverse.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/07_reverse.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `fill` | [typescript/08_fill.ts](typescript/08_fill.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/08_fill.ts) | Scalar copy; preserve operation and return semantics. |
| typescript `copyWithin` | [typescript/09_copyWithin.ts](typescript/09_copyWithin.ts) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/typescript/09_copyWithin.ts) | Scalar copy; preserve operation and return semantics. |
| python `append` | [python/01_append.py](python/01_append.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/01_append.py) | Scalar copy; preserve operation and return semantics. |
| python `extend` | [python/02_extend.py](python/02_extend.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/02_extend.py) | Scalar copy; preserve operation and return semantics. |
| python `insert` | [python/03_insert.py](python/03_insert.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/03_insert.py) | Scalar copy; preserve operation and return semantics. |
| python `remove` | [python/04_remove.py](python/04_remove.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/04_remove.py) | Scalar copy; preserve operation and return semantics. |
| python `pop` | [python/05_pop.py](python/05_pop.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/05_pop.py) | Scalar copy; preserve operation and return semantics. |
| python `clear` | [python/06_clear.py](python/06_clear.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/06_clear.py) | Scalar copy; preserve operation and return semantics. |
| python `sort` | [python/07_sort.py](python/07_sort.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/07_sort.py) | Scalar copy; preserve operation and return semantics. |
| python `reverse` | [python/08_reverse.py](python/08_reverse.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/08_reverse.py) | Scalar copy; preserve operation and return semantics. |
| python `slice_assignment` | [python/09_slice_assignment.py](python/09_slice_assignment.py) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/python/09_slice_assignment.py) | Scalar copy; preserve operation and return semantics. |
| rust `push` | [rust/01_push.rs](rust/01_push.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/01_push.rs) | Scalar copy; preserve operation and return semantics. |
| rust `pop` | [rust/02_pop.rs](rust/02_pop.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/02_pop.rs) | Scalar copy; preserve operation and return semantics. |
| rust `insert` | [rust/03_insert.rs](rust/03_insert.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/03_insert.rs) | Scalar copy; preserve operation and return semantics. |
| rust `remove` | [rust/04_remove.rs](rust/04_remove.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/04_remove.rs) | Scalar copy; preserve operation and return semantics. |
| rust `swap_remove` | [rust/05_swap_remove.rs](rust/05_swap_remove.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/05_swap_remove.rs) | Scalar copy; preserve operation and return semantics. |
| rust `retain` | [rust/06_retain.rs](rust/06_retain.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/06_retain.rs) | Scalar copy; preserve operation and return semantics. |
| rust `sort` | [rust/07_sort.rs](rust/07_sort.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/07_sort.rs) | Scalar copy; preserve operation and return semantics. |
| rust `sort_by` | [rust/08_sort_by.rs](rust/08_sort_by.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/08_sort_by.rs) | Scalar copy; preserve operation and return semantics. |
| rust `reverse` | [rust/09_reverse.rs](rust/09_reverse.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/09_reverse.rs) | Scalar copy; preserve operation and return semantics. |
| rust `truncate` | [rust/10_truncate.rs](rust/10_truncate.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/10_truncate.rs) | Scalar copy; preserve operation and return semantics. |
| rust `clear` | [rust/11_clear.rs](rust/11_clear.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/11_clear.rs) | Scalar copy; preserve operation and return semantics. |
| rust `dedup` | [rust/12_dedup.rs](rust/12_dedup.rs) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/rust/12_dedup.rs) | Scalar copy; preserve operation and return semantics. |
| go `index_assignment` | [go/01_index_assignment.go](go/01_index_assignment.go) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/go/01_index_assignment.go) | Scalar copy; preserve operation and return semantics. |
| go `append` | [go/02_append.go](go/02_append.go) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/go/02_append.go) | Scalar copy; preserve operation and return semantics. |
| go `copy` | [go/03_copy.go](go/03_copy.go) | [copy before mutation](../../../examples-fixed/mutated-state/volatile-properties/go/03_copy.go) | Scalar copy; preserve operation and return semantics. |

## Alternatives and limits

TypeScript sort/reverse/splice have toSorted/toReversed/toSpliced alternatives on supporting runtimes; concat or spread can replace append/prepend when the numeric return value is not needed. Python sorted and slicing can replace sort/reverse; concatenation can replace append/extend. Rust iterator collect and Go slices.Clone followed by a local transformation provide independent storage. General removal/fill/copy operations are safest illustrated by copy-before-mutate because a short expression can change return, index, or exception semantics.

Maps, dictionaries, sets, deques, string buffers, cursor position and vector capacity are covered below. Stack and queue roles reuse the existing array/list/vector primitives.

## References

- [JavaScript Array](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
- [Rust Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)
- [Python lists](https://docs.python.org/3/tutorial/datastructures.html)
- [Go append and copy](https://go.dev/ref/spec#Appending_and_copying_slices)

Run Query Lab `test --defect volatile-properties` and `verify-fixes --defect volatile-properties`. The generated site exposes this class at `/en/mutated-state/volatile-properties/`.

## Audit of adjacent classes

Reviewed all 18 var-let-mut, 13 this-mutation and 11 increment-decrement examples and their available repairs. The existing Go, Rust and TypeScript repairs address their stated fixture contracts; they need not eliminate mutation. var-let-mut's existing review records its domain assumptions.

Six Python repair files were missing and are now supplied: per-instance ledger storage, initialization before an overridable seed hook, explicit binding of an instance-stored function, a builder return, integer arithmetic for exact stepping, and exclusion of a boolean flag from a quantity. These are runnable repair fixtures, but the older categories remain outside Python Query Lab coverage. The initialization repair establishes a zero bonus before dispatch; it does not make arbitrary constructor overrides safe. Excluding the flag assumes flags are not quantities, as the manifest states.

Arithmetic repairs are domain-specific: widening Go's accumulator handles this bounded fixture, not arbitrary overflow; BigInt changes the TypeScript ID representation; Rust checked operations change return types. The TypeScript invoice repair fixes coupon-before-tax policy and idempotent application, which cannot be inferred from syntax. The TypeScript field/method override example also triggers a static type error under type checking, although direct Node execution demonstrates the runtime hazard. None of these belong in collection mutation merely because they use mutable bindings.

## Initial array/list validation results

All 33 originals fail specifically at the source-preservation assertion. All 33 repairs pass Query Lab runtime checks, and both variants pass independently authored capture expectations. Dedicated controls cover benign calls, custom/shadowed names, Python empty/removal behavior, repeated runs, Go append capacity and the six missing Python repairs. All 31 supported existing repair pairs across the three audited categories pass runtime checks; the five existing Python var-let-mut repairs retain their dedicated tests.

The full 320-case capture run has one unrelated stale hash in `shared-state/async-io-misuse/go/03_goroutine_leak_unbuffered_result`, already modified before this change. Its source and expectations were left intact. Site build tests include direct original and fixed source links.

## Expanded operation matrix

The preservation contract includes contents, stream position and explicitly requested capacity stability. "Immutable repair" describes source preservation, not the absence of all local mutation. No semantic type resolver or automatic rewrite engine is introduced.

| Operation/type | Original | Independent repair |
|---|---|---|
| `10_set_add` (typescript) | [source](typescript/10_set_add.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/10_set_add.ts) |
| `11_set_delete` (typescript) | [source](typescript/11_set_delete.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/11_set_delete.ts) |
| `12_set_clear` (typescript) | [source](typescript/12_set_clear.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/12_set_clear.ts) |
| `13_map_set` (typescript) | [source](typescript/13_map_set.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/13_map_set.ts) |
| `14_map_delete` (typescript) | [source](typescript/14_map_delete.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/14_map_delete.ts) |
| `15_map_clear` (typescript) | [source](typescript/15_map_clear.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/15_map_clear.ts) |
| `16_record_assign` (typescript) | [source](typescript/16_record_assign.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/16_record_assign.ts) |
| `17_record_delete` (typescript) | [source](typescript/17_record_delete.ts) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/typescript/17_record_delete.ts) |
| `10_set_add` (python) | [source](python/10_set_add.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/10_set_add.py) |
| `11_set_remove` (python) | [source](python/11_set_remove.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/11_set_remove.py) |
| `12_set_discard` (python) | [source](python/12_set_discard.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/12_set_discard.py) |
| `13_set_pop` (python) | [source](python/13_set_pop.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/13_set_pop.py) |
| `14_set_update` (python) | [source](python/14_set_update.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/14_set_update.py) |
| `15_set_intersection_update` (python) | [source](python/15_set_intersection_update.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/15_set_intersection_update.py) |
| `16_set_clear` (python) | [source](python/16_set_clear.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/16_set_clear.py) |
| `17_dict_update` (python) | [source](python/17_dict_update.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/17_dict_update.py) |
| `18_dict_pop` (python) | [source](python/18_dict_pop.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/18_dict_pop.py) |
| `19_dict_popitem` (python) | [source](python/19_dict_popitem.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/19_dict_popitem.py) |
| `20_dict_setdefault` (python) | [source](python/20_dict_setdefault.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/20_dict_setdefault.py) |
| `21_dict_clear` (python) | [source](python/21_dict_clear.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/21_dict_clear.py) |
| `22_deque_append` (python) | [source](python/22_deque_append.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/22_deque_append.py) |
| `23_deque_appendleft` (python) | [source](python/23_deque_appendleft.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/23_deque_appendleft.py) |
| `24_deque_pop` (python) | [source](python/24_deque_pop.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/24_deque_pop.py) |
| `25_deque_popleft` (python) | [source](python/25_deque_popleft.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/25_deque_popleft.py) |
| `26_deque_extend` (python) | [source](python/26_deque_extend.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/26_deque_extend.py) |
| `27_deque_rotate` (python) | [source](python/27_deque_rotate.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/27_deque_rotate.py) |
| `28_stringio_write` (python) | [source](python/28_stringio_write.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/28_stringio_write.py) |
| `29_stringio_seek` (python) | [source](python/29_stringio_seek.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/29_stringio_seek.py) |
| `30_stringio_truncate` (python) | [source](python/30_stringio_truncate.py) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/python/30_stringio_truncate.py) |
| `13_hashset_insert` (rust) | [source](rust/13_hashset_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/13_hashset_insert.rs) |
| `14_hashset_remove` (rust) | [source](rust/14_hashset_remove.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/14_hashset_remove.rs) |
| `15_hashset_clear` (rust) | [source](rust/15_hashset_clear.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/15_hashset_clear.rs) |
| `16_hashset_retain` (rust) | [source](rust/16_hashset_retain.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/16_hashset_retain.rs) |
| `17_btreeset_insert` (rust) | [source](rust/17_btreeset_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/17_btreeset_insert.rs) |
| `18_btreeset_remove` (rust) | [source](rust/18_btreeset_remove.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/18_btreeset_remove.rs) |
| `19_btreeset_clear` (rust) | [source](rust/19_btreeset_clear.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/19_btreeset_clear.rs) |
| `20_btreeset_retain` (rust) | [source](rust/20_btreeset_retain.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/20_btreeset_retain.rs) |
| `21_hashmap_insert` (rust) | [source](rust/21_hashmap_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/21_hashmap_insert.rs) |
| `22_hashmap_remove` (rust) | [source](rust/22_hashmap_remove.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/22_hashmap_remove.rs) |
| `23_hashmap_clear` (rust) | [source](rust/23_hashmap_clear.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/23_hashmap_clear.rs) |
| `24_hashmap_entry_insert` (rust) | [source](rust/24_hashmap_entry_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/24_hashmap_entry_insert.rs) |
| `25_hashmap_entry_modify` (rust) | [source](rust/25_hashmap_entry_modify.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/25_hashmap_entry_modify.rs) |
| `26_btreemap_insert` (rust) | [source](rust/26_btreemap_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/26_btreemap_insert.rs) |
| `27_btreemap_remove` (rust) | [source](rust/27_btreemap_remove.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/27_btreemap_remove.rs) |
| `28_btreemap_clear` (rust) | [source](rust/28_btreemap_clear.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/28_btreemap_clear.rs) |
| `29_btreemap_entry_insert` (rust) | [source](rust/29_btreemap_entry_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/29_btreemap_entry_insert.rs) |
| `30_btreemap_entry_modify` (rust) | [source](rust/30_btreemap_entry_modify.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/30_btreemap_entry_modify.rs) |
| `31_string_push` (rust) | [source](rust/31_string_push.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/31_string_push.rs) |
| `32_string_push_str` (rust) | [source](rust/32_string_push_str.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/32_string_push_str.rs) |
| `33_string_insert` (rust) | [source](rust/33_string_insert.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/33_string_insert.rs) |
| `34_string_remove` (rust) | [source](rust/34_string_remove.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/34_string_remove.rs) |
| `35_string_truncate` (rust) | [source](rust/35_string_truncate.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/35_string_truncate.rs) |
| `36_string_clear` (rust) | [source](rust/36_string_clear.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/36_string_clear.rs) |
| `37_vecdeque_push_back` (rust) | [source](rust/37_vecdeque_push_back.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/37_vecdeque_push_back.rs) |
| `38_vecdeque_push_front` (rust) | [source](rust/38_vecdeque_push_front.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/38_vecdeque_push_front.rs) |
| `39_vecdeque_pop_back` (rust) | [source](rust/39_vecdeque_pop_back.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/39_vecdeque_pop_back.rs) |
| `40_vecdeque_pop_front` (rust) | [source](rust/40_vecdeque_pop_front.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/40_vecdeque_pop_front.rs) |
| `41_vec_drain` (rust) | [source](rust/41_vec_drain.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/41_vec_drain.rs) |
| `42_vec_reserve` (rust) | [source](rust/42_vec_reserve.rs) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/rust/42_vec_reserve.rs) |
| `04_map_assign` (go) | [source](go/04_map_assign.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/04_map_assign.go) |
| `05_map_delete` (go) | [source](go/05_map_delete.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/05_map_delete.go) |
| `06_set_assign` (go) | [source](go/06_set_assign.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/06_set_assign.go) |
| `07_set_delete` (go) | [source](go/07_set_delete.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/07_set_delete.go) |
| `08_builder_Write` (go) | [source](go/08_builder_Write.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/08_builder_Write.go) |
| `09_builder_WriteString` (go) | [source](go/09_builder_WriteString.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/09_builder_WriteString.go) |
| `10_builder_WriteByte` (go) | [source](go/10_builder_WriteByte.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/10_builder_WriteByte.go) |
| `11_builder_Reset` (go) | [source](go/11_builder_Reset.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/11_builder_Reset.go) |
| `12_slice_Sort` (go) | [source](go/12_slice_Sort.go) | [fixed](../../../examples-fixed/mutated-state/volatile-properties/go/12_slice_Sort.go) |

### Interpretation and alternatives

- JavaScript Map and Set copies preserve insertion order and scalar contents; object spread is appropriate for these plain scalar records. It is not a general prototype/accessor-preserving clone. Array push/pop examples also cover stacks; push/shift cover queues. Array-based string accumulation uses those same mutations; join itself is a non-mutating control.
- Rust HashSet/BTreeSet and HashMap/BTreeMap clones preserve these scalar collections. Iterator collect can construct an independent result where return values permit. Bare entry creates an access handle; or_insert and and_modify are conditional update candidates. The entry fixture tests both vacant insertion and occupied modification. Rust mutability is a property of access: an &mut reference permits mutation without requiring the reference binding itself to be mut.
- Rust String indexes are UTF-8 byte offsets. Fixtures use valid boundaries; invalid boundaries must retain their panic behavior. Vec drain is collected and dropped before checking preservation. reserve changes capacity, not elements; its fixture explicitly requires source capacity to remain unchanged and asserts a lower bound, never exact allocation growth.
- Python set operations may be replaced by union/intersection/difference expressions or frozenset when return types permit. Copy-before-mutate preserves remove exceptions and pop results. Unordered removal is tested by membership, not a particular chosen item. Dict popitem uses its defined last-in order; setdefault only inserts when absent.
- A deque copy retains maxlen; extending a bounded deque can evict old items. List append/pop cover stack roles. StringIO reconstruction copies contents and restores position before the requested operation; seek changes position without changing text. The fixtures use default newline handling, not arbitrary stream configurations. sorted, string joining and reads are not mutation candidates.
- Go map/set repairs use maps.Clone. Builder repairs construct an empty builder and write the existing string into it; copying a nonzero Builder by value is prohibited. Write counts/errors are preserved. slices.Sort sorts fresh cloned storage; package aliases and shadowing remain unresolved syntax candidates. Slicing alone changes a view, not backing elements; subsequent writes or capacity-reusing append create the hazard.

### References for expanded coverage

- [Go Builder](https://pkg.go.dev/strings#Builder)
- [Rust map entry API](https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html#method.entry)
- [Python StringIO](https://docs.python.org/3/library/io.html#io.StringIO)

Existing examples 01-09 (TypeScript), 01-09 (Python), 01-12 (Rust) and 01-03 (Go) retain their names and URLs. Existing sort/reverse/splice/fill/retain/truncate examples supply the requested list transformations; they are not duplicated under collection-role aliases.

### Expansion validation

The expanded corpus contains 101 pairs (68 added): TypeScript 17, Python 30, Rust 42, Go 12. All added originals fail specifically on preservation assertions; all fixed programs pass their result and source-state checks. All four expanded query packs pass independently authored capture expectations. Full Query Lab regression testing continues to expose the pre-existing unrelated async-I/O stale hash noted above; it is not suppressed or rewritten.

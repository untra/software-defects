# Division and remainder review

All 16 examples have matching files under `examples-fixed` and capture/runtime expectations in Query Lab. The production regression test checks that every original produces a warning and every fixed counterpart removes that warning. Query captures intentionally remain in guarded code.

| Language | Cases | Repair convention |
| --- | --- | --- |
| Go | integer division, float ratio, remainder, empty average | Early rejection; average returns `(value, available)` |
| Rust | integer division, float ratio, remainder | Early rejection before the operation |
| TypeScript | number division, zero ratio, remainder, empty average, BigInt division | `null` for unavailable values; average returns `number \| null` |
| Python | true division, floor division, remainder, empty mean | Conditional expression returning `None` |

The original numeric computation is preserved on valid inputs. The empty-average Go signature and nullable results are intentional contract changes; callers must handle unavailable values. There is no universally correct zero fallback for arbitrary application code. These pairs are transformation examples, not authorization to apply automatic repairs. An impure denominator must be evaluated once and the captured value checked; repeating a call in a ternary is unsafe.

Rust integer zero division/remainder panic in both debug and release. TypeScript Number and Go/Rust floating-point division can produce non-finite values without crashing. Python arithmetic and TypeScript BigInt zero division raise exceptions. Warning wording distinguishes these possibilities.

Production queries capture operations and operands, including compound assignments. Semantic checks recognize local literal assignments, parameters, `len`, array `.length`, and `std::hint::black_box`, plus explicit zero guards. The analyzer does not infer arbitrary return values, perform interprocedural numeric propagation, resolve dynamic dispatch, or prove absence of aliases. Unresolved expressions remain candidates. Calls invalidate guard assumptions conservatively; the simple collection-sum reduction used by the TypeScript example is recognized separately.

Yellow requires traced denominator reassignment, rather than merely placing division inside an impure function. Immutable zero denominators remain blue and still generate warnings. This is evidence classification, not a statistical crash probability.

Example pages: [division guide](https://badsoftware.untra.io/en/unstructured-state/divide-by-zero/). Each rendered example has an `example-<language>-<filename-with-dots-replaced-by-hyphens>` anchor.

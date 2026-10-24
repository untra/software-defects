# Bugs of Unstructured State — language susceptibility

Score **1–5**: how likely the defect is to show up in everyday code in that language, and how many distinct, demonstrable forms it takes. Each cell has exactly that many example files.

| Defect | Go | Rust | TypeScript | Python |
|---|:-:|:-:|:-:|:-:|
| [divide-by-zero](divide-by-zero/) | 4 | 3 | 5 | 4 |
| [type-errors](type-errors/) | 2 | 1 | 5 | 5 |
| [index-out-of-bounds](index-out-of-bounds/) | 4 | 3 | 5 | 4 |
| [lack-of-input-validation](lack-of-input-validation/) | 4 | 3 | 5 | 5 |
| [unconstrained-inputs](unconstrained-inputs/) | 4 | 3 | 4 | 4 |
| [duck-typing-without-contracts](duck-typing-without-contracts/) | 2 | 2 | 5 | 5 |
| [missing-undefined-returns](missing-undefined-returns/) | 4 | 2 | 5 | 5 |
| [version-library-mismanagement](version-library-mismanagement/) | 2 | 3 | 4 | 3 |
| [insecure-deserialization](insecure-deserialization/) | 3 | 2 | 4 | 4 |
| **Total** | **29** | **22** | **42** | **39** |

## Rationale

**divide-by-zero**
- Go 4: integer `/` and `%` panic; float64 yields `+Inf`; an empty average divides by `len`.
- Rust 3: the same three operators. Nothing in the type system tracks a non-zero divisor.
- TS 5: `/ 0` is `Infinity`, `0/0` and `% 0` are `NaN`, an empty mean is `NaN`, and a `bigint` divisor throws.
- Python 4: `/`, `//`, and `%` all raise, including a float dividend. There is no silent `Infinity` from division. The fourth case is an empty mean.

**type-errors**
- Go 2: static types stop coercion. A JSON number stored in `any` is `float64`, and a `fmt` verb can disagree with the dynamic type.
- Rust 1: types are enforced. The leftover is an `Any` downcast to the wrong width, then a default.
- TS 5: `+` concatenates, `==` coerces, a falsy `0` is replaced, numeric strings sort lexicographically, and `parseInt` keeps a trailing unit.
- Python 5: a string repeats instead of multiplying, `n or 50` replaces `0`, `True` overwrites the key `1`, `sorted` refuses mixed types, and `int("10px")` raises.

**index-out-of-bounds**
- Go 4: a computed index panics (including `len-1` on empty), a missing map key is the zero value, a slice can split a rune, and a short split is still indexed.
- Rust 3: indexing panics, a slice off a char boundary panics, and `last().unwrap()` panics on empty.
- TS 5: a bare index does not throw. Empty `[0].field` throws later, arithmetic becomes `NaN`, a missing key is `undefined`, `arr[-1]` does not wrap, and a code unit splits an emoji.
- Python 4: an empty `[0]`, a missing key, `.get` followed by arithmetic on `None`, and `split` of a header that has no field. A string index is a code point, so there is no surrogate-pair example.

**lack-of-input-validation**
- Go 4: an ignored `Atoi` error, a query built with `Sprintf`, a status checked against one literal, and a negative quantity.
- Rust 3: `unwrap_or(0)` on a bad parse, the same query build, and a negative quantity. Safe Rust has no `eval`.
- TS 5: a cast the runtime erases, `eval` of a formula, a template-built query, a truthy check used as validation, and a prefix host check.
- Python 5: a failed `int` becomes `0`, a query string keeps `O'Brien` inside quotes, a negative quantity is charged, `eval` of a formula sees a local name, and `startswith` accepts a longer host. There is no comma operator.

**unconstrained-inputs**
- Go 4: no ReDoS (`regexp` is linear). Input still wraps an `int64` product, panics `make` on a negative length, repeats past a domain max, and overflows a `time.Duration`.
- Rust 3: a debug-mode product panics, a repeat ignores the max, and `as u32` truncates a scaled duration.
- TS 4: `slice` treats a negative end as an offset, repeat ignores the max, a product becomes `Infinity`, and nested groups have no depth cap. A nested-quantifier hang is not a stable case.
- Python 4: a slice limit of `-1` drops the last row, a repeated label has no max, a float product becomes `inf`, and nested groups have no depth cap. A regular-expression hang is not a stable case.

**duck-typing-without-contracts**
- Go 2: interfaces are checked at compile time. A method can still omit its unit, and a type-switch default can assume a map.
- Rust 2: traits are the contract. The holes are a unit-less `amount`, and a default method that unwraps a promise the implementor never made.
- TS 5: structural typing. Cents and dollars share a shape, an array-like value has no `.map`, an optional method is missing, two string ids swap, and a widened reading carries the wrong unit.
- Python 5: `amount()` hides cents versus dollars, a string iterates as titles, a missing `save` raises at the call, two string ids swap, and a reading that only has `.value` is converted with the Celsius formula.

**missing-undefined-returns**
- Go 4: every path must return, so the bug is a success value for failure: `(nil, nil)`, an ignored `error`, a discarded comma-ok, and a naked return of the zero result.
- Rust 2: `Option` and `Result` must be handled unless the code unwraps or drops the error.
- TS 5: a branch falls off the end (`NaN`), `find` is `undefined`, a `map` callback returns nothing, an optional field is added, and an async function resolves to `undefined`.
- Python 5: `find` falls off the end and the caller indexes `None`, a discount branch makes `1 - None` raise, a helper that prints returns `None`, `.get` is used as a number, and an async function returns `None`.

**version-library-mismanagement**
- Go 2: modules are usually exact. A copied helper still disagrees with its twin, and a highest-version pick still changes behavior. No network.
- Rust 3: the same copy, a caret range (`^1.2.0` accepts a later 1.x), and a feature flag unified on by another crate.
- TS 4: copy, caret, an unpinned "latest" resolve, and two majors of one helper in the same process.
- Python 3: a copied February helper, an unpinned "last published" pick of a later release, and two majors in one process. No install and no network.

**insecure-deserialization**
- Go 3: `encoding/json` sets an exported `Admin` field, a wrong JSON type becomes a zero with the error ignored, and a nested object sets a flag while unknown fields are skipped.
- Rust 2: no serde in these examples. A hand-rolled assigner copies `admin`, and a length prefix consumes the next field.
- TS 4: `Object.assign` of the body, a merge that assigns `__proto__`, a reviver that keeps a caller-supplied flag, and a `constructor` property taken from parsed data.
- Python 4: `setattr` from the body, `User(**data)`, a nested flags dict, and `dict.update`. These examples only show the program's own `admin` flag flipping. No `pickle`.

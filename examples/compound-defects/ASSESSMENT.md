# Compound defects — language susceptibility

Score **0–2** for how likely the compound CWE is in ordinary code in that language, and whether a short program can show the CWE's defining behavior. This score is not a file count. The family assessments use 1–5 as a file count. This scale does not.

Checked against the compound CWE on cwe.mitre.org (4.20).

| Score | Meaning |
|---|---|
| 2 | Ordinary code in this language produces the compound CWE. |
| 1 | Runnable only through an escape hatch, or the language traps the access so the CWE's defining result does not happen. |
| 0 | The language cannot produce that CWE. A lookalike would be a different weakness. |

`ctypes` counts as Python. `unsafe` counts as Rust or Go. WebAssembly and native addons do not count as TypeScript. `cgo` does not count as Go.

The **Best** column is the language to use when adding one example for that defect. **now** marks the four flagship programs in this tree.

## Nondeterminism — mutated × shared

| Defect | CWE | Py | TS | Go | Rust | Best |
|---|---|:-:|:-:|:-:|:-:|---|
| [toctou-race](nondeterminism/toctou-race/) | 367 | 2 | 2 | 2 | 2 | Go |
| [lost-update](nondeterminism/lost-update/) | 567 | 2 | 1 | 2 | 1 | Go |
| [out-of-order-async](nondeterminism/out-of-order-async/) | 362, 696 | 2 | 2 | 2 | 2 | TypeScript **now** |
| [concurrent-modification](nondeterminism/concurrent-modification/) | 362, 567 | 2 | 1 | 2 | 1 | Go |
| [pool-exhaustion-heisenbug](nondeterminism/pool-exhaustion-heisenbug/) | 362 | 2 | 2 | 2 | 2 | Go |
| [stale-cache-heisenbug](nondeterminism/stale-cache-heisenbug/) | 471 | 2 | 2 | 2 | 2 | Python |
| [use-after-free](nondeterminism/use-after-free/) | 825 | 1 | 1 | 1 | 1 | Rust |
| [pooled-resource-heisenbug](nondeterminism/pooled-resource-heisenbug/) | 459 | 2 | 2 | 2 | 2 | Python |
| [lazy-init-visibility-race](nondeterminism/lazy-init-visibility-race/) | 609 | 0 | 0 | 2 | 1 | Go **now** |
| [stale-replica-read](nondeterminism/stale-replica-read/) | 821 | 2 | 2 | 2 | 2 | Go |
| [predictable-id-collision](nondeterminism/predictable-id-collision/) | 340, 362 | 2 | 2 | 2 | 2 | Go |
| [temp-file-race](nondeterminism/temp-file-race/) | 363, 59 | 2 | 2 | 2 | 2 | Python |
| [concurrent-singleton](nondeterminism/concurrent-singleton/) | 488 | 2 | 2 | 2 | 2 | TypeScript |
| [hash-order-divergence](nondeterminism/hash-order-divergence/) | 340 | 2 | 0 | 2 | 2 | Python |
| [dispatch-order-divergence](nondeterminism/dispatch-order-divergence/) | 758 | 2 | 2 | 1 | 2 | Python |

CWE-567 is unsynchronized access in a multithreaded context. TypeScript shows it with `SharedArrayBuffer` (1). An `await` lost-update is CWE-362, which out-of-order-async already covers. Safe Rust rejects a data race; dropping a lock between read and write is the score-1 form.

CWE-609 is a half-constructed object made visible by memory ordering. MITRE's demonstrative example is Java. The Go memory model allows it. CPython and JavaScript do not show it for ordinary objects (0). Rust needs a relaxed atomic publish (1).

CWE-825 is limited by MITRE to memory-unsafe languages. Score 1 is the escape hatch already used under `examples/shared-state/pointer-mismanagement/` (Rust raw pointer, Go `unsafe`, Python `ctypes`, a detached TypeScript `ArrayBuffer`).

TypeScript iteration order is specified for string keys, `Map`, and `Set`. Hash-order divergence is 0 there. Go map iteration and Rust `HashMap` order are unspecified. CPython randomizes the hash seed per process.

Go `init` order is specified, so dispatch-order divergence is 1 there (a plugin-style loader). Python import side effects and TypeScript dynamic `import()` can change which handler wins (2).

## Runtime errors — mutated × unstructured

| Defect | CWE | Py | TS | Go | Rust | Best |
|---|---|:-:|:-:|:-:|:-:|---|
| [null-dereference](runtime-errors/null-dereference/) | 476, 690 | 2 | 2 | 2 | 1 | Go |
| [type-confusion](runtime-errors/type-confusion/) | 843 | 2 | 2 | 1 | 1 | Python |
| [shape-drift](runtime-errors/shape-drift/) | 754 | 2 | 2 | 1 | 1 | Python |
| [aliased-default-type-confusion](runtime-errors/aliased-default-type-confusion/) | 843 | 2 | 2 | 1 | 1 | Python **now** |
| [unexpected-type-element](runtime-errors/unexpected-type-element/) | 241 | 2 | 2 | 1 | 1 | Python |
| [off-by-one-crash](runtime-errors/off-by-one-crash/) | 125, 787 | 2 | 2 | 2 | 2 | Go |
| [divide-by-zero-empty](runtime-errors/divide-by-zero-empty/) | 369, 754 | 2 | 2 | 2 | 2 | Python |
| [quadratic-blowup](runtime-errors/quadratic-blowup/) | 400, 407 | 2 | 2 | 2 | 2 | Python |
| [nan-poisons-consumer](runtime-errors/nan-poisons-consumer/) | 754 | 2 | 2 | 2 | 2 | Python |
| [integer-overflow-buffer](runtime-errors/integer-overflow-buffer/) | 680, 787 | 1 | 0 | 1 | 1 | Go |
| [integration-crash](runtime-errors/integration-crash/) | 248, 754 | 2 | 2 | 2 | 2 | Python |
| [stale-api-upgrade](runtime-errors/stale-api-upgrade/) | 758, 754 | 2 | 2 | 1 | 1 | Python |
| [deserialized-type-confusion](runtime-errors/deserialized-type-confusion/) | 704, 843 | 2 | 2 | 1 | 1 | Python |
| [config-drift-crash](runtime-errors/config-drift-crash/) | 248, 754 | 2 | 2 | 2 | 2 | Rust |
| [schema-drift-on-load](runtime-errors/schema-drift-on-load/) | 915 | 2 | 2 | 1 | 1 | Python |

Go and Rust structs cannot gain a field on one path, and they have no default arguments. Shape drift, aliased defaults, unexpected elements, stale APIs, deserialized confusion, and schema-drift-on-load are 1 there (a map, `any`, or reflection). Type confusion is 1 for the same reason. Rust references are non-null, so CWE-476 is 1: `unwrap` on `None` crashes, and `*ptr::null()` is undefined behavior rather than a stable example.

Off-by-one is a trapped crash in all four (panic, `IndexError`, or a throw on a missing property). That is the runtime-error form, not a memory disclosure.

CWE-680 needs a wrap that then writes past a buffer. Safe Go and safe Rust trap the write (1). TypeScript cannot finish the chain (0). Python finishes it only with `ctypes` (1). Python `int` does not wrap.

Go `int(math.NaN())` is 0 on the machine this was checked on, so a Go NaN example must crash in the consumer some other way. Python `int(float("nan"))` raises, which is why the preferred NaN consumer is Python. The cell stays 2 for Go because a non-finite float is ordinary and a consumer can still reject it.

TypeScript `sum / length` on an empty list is `NaN`, not a throw. Prefer Python when the compound result should be CWE-369's exception.

## Data corruption — shared × unstructured

| Defect | CWE | Py | TS | Go | Rust | Best |
|---|---|:-:|:-:|:-:|:-:|---|
| [buffer-overflow](data-corruption/buffer-overflow/) | 787 | 1 | 0 | 1 | 1 | Rust |
| [stack-smash](data-corruption/stack-smash/) | 787 | 0 | 0 | 0 | 0 | — |
| [type-confusion-corruption](data-corruption/type-confusion-corruption/) | 119 | 1 | 0 | 1 | 1 | Rust |
| [torn-write](data-corruption/torn-write/) | 787 | 1 | 0 | 1 | 1 | Go |
| [unchecked-sentinel-corruption](data-corruption/unchecked-sentinel-corruption/) | 787 | 1 | 0 | 1 | 1 | Go |
| [nan-shared-aggregate](data-corruption/nan-shared-aggregate/) | 682 | 2 | 2 | 2 | 2 | Python |
| [mojibake](data-corruption/mojibake/) | 172 | 2 | 2 | 2 | 2 | Python |
| [unframed-stream](data-corruption/unframed-stream/) | 240 | 2 | 2 | 2 | 2 | Go |
| [lossy-coercion-ledger](data-corruption/lossy-coercion-ledger/) | 682 | 2 | 2 | 2 | 2 | TypeScript |
| [signedness-poisons-aggregate](data-corruption/signedness-poisons-aggregate/) | 682, 195 | 1 | 1 | 2 | 2 | Rust **now** |
| [cross-session-contamination](data-corruption/cross-session-contamination/) | 488 | 2 | 2 | 2 | 2 | TypeScript |
| [schema-drift](data-corruption/schema-drift/) | 345 | 2 | 2 | 2 | 2 | Python |
| [stored-injection](data-corruption/stored-injection/) | 89, 79 | 2 | 2 | 2 | 2 | Python |
| [untrusted-deserialization](data-corruption/untrusted-deserialization/) | 502 | 2 | 2 | 2 | 2 | Python |
| [web-cache-poisoning](data-corruption/web-cache-poisoning/) | 349 | 2 | 2 | 2 | 2 | TypeScript |

Stack smash is 0 in all four. Deep recursion aborts (`RecursionError`, `RangeError`, a Go stack overflow, a Rust abort). It does not write past a buffer. CWE-674 already has examples under `examples/shared-state/stack-overflow/`. Do not add a fake smash here.

TypeScript has no in-language write past a buffer, so buffer overflow, type-confusion corruption, torn write, and unchecked-sentinel corruption are 0. Python `ctypes`, Go `unsafe`, and Rust `unsafe` can show them (1). An in-bounds torn word is CWE-362, not CWE-787. Python `seq[-1]` overwrites the last row, which is durable corruption, but it is not an out-of-bounds write.

Signedness is ordinary in Go (`uint32(n)`) and Rust (`as u32`). Python has no unsigned integers (`ctypes` or `struct`, score 1). TypeScript `number` is not signed; `>>> 0` is the score-1 demonstration.

Stored injection examples should print the query or the markup. They should not open a database. Deserialization examples should let a JSON field set `admin`. They should not use `pickle` or a gadget chain.

## Counts

| Class | Score 2 | Score 1 | Score 0 |
|---|---:|---:|---:|
| Nondeterminism (60) | 47 | 10 | 3 |
| Runtime errors (60) | 41 | 18 | 1 |
| Data corruption (60) | 38 | 14 | 8 |
| Total (180) | 126 | 42 | 12 |

## Flagships

| Language | Defect | What the program shows |
|---|---|---|
| Python | aliased-default-type-confusion | One default list is shared. A string stored in it crashes the next caller that reads a `sku`. |
| TypeScript | out-of-order-async | The slower request is started first and writes the binding last. |
| Go | lazy-init-visibility-race | The pointer is stored before `name` and `ready` are set. A reader that does not hold the lock observes the zero value. |
| Rust | signedness-poisons-aggregate | `as u32` turns a refund of `-1` into 4294967295 and adds it under a `Mutex`. |

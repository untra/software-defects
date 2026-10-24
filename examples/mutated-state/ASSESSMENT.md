# Bugs of Mutated State — language susceptibility

Score **1–5**: how likely the defect is to show up in everyday code in that language, and how many distinct, demonstrable forms it takes. Each cell has exactly that many example files.

| Defect | Go | Rust | TypeScript | Python |
|---|:-:|:-:|:-:|:-:|
| [var-let-mut](var-let-mut/) | 4 | 2 | 4 | 4 |
| [aliasing-mutable-defaults](aliasing-mutable-defaults/) | 5 | 1 | 5 | 5 |
| [for-loop-control-flow](for-loop-control-flow/) | 3 | 2 | 4 | 4 |
| [this-mutation](this-mutation/) | 3 | 1 | 5 | 4 |
| [time-money-entropy](time-money-entropy/) | 3 | 3 | 5 | 5 |
| [impure-functions](impure-functions/) | 4 | 2 | 5 | 5 |
| [macro-mixin-misuse](macro-mixin-misuse/) | 2 | 3 | 3 | 3 |
| [increment-decrement-operators](increment-decrement-operators/) | 3 | 3 | 3 | 2 |
| [improper-initialization](improper-initialization/) | 5 | 2 | 4 | 4 |
| **Total** | **32** | **19** | **38** | **36** |

## Rationale

**var-let-mut**
- Go 4: every binding is mutable; `:=` shadowing, closure capture of reassigned vars, and mutable named returns.
- Rust 2: immutable by default. Shadowing and loop-scoped `mut` resets are still possible.
- TS 4: `var` hoisting, `let` reassigned across `await`, and `const` that only freezes the binding.
- Python 4: closures bind late, a loop name leaks after the loop, an assignment anywhere makes the name local, and `global` rebinds a module name.

**aliasing-mutable-defaults**
- Go 5: slices share backing arrays, maps are references, and struct copies are shallow.
- Rust 1: the borrow checker forbids shared mutation. It only comes back through `Rc<RefCell<_>>`.
- TS 5: every object is a reference. Shared default parameters, `fill`, shallow spread, and `Object.assign` targets all alias.
- Python 5: a mutable default, `[[]] * n`, a list created in the class body, a shallow `copy`, and a method that returns its internal list.

**for-loop-control-flow**
- Go 3: index-based removal, range-value copies, and pointers to the loop variable.
- Rust 2: iterator invalidation is a compile error. Manual `while` and index loops can still go wrong.
- TS 4: `splice` in `forEach`, `for…in` over arrays, `var` loop capture, and async callbacks in `forEach`.
- Python 4: deleting by index skips the next row, `remove` during iteration skips, `for`/`else` runs when there was no `break`, and deleting a dict entry raises `RuntimeError`.

**this-mutation**
- Go 3: value vs. pointer receivers, method values, and interfaces boxing a copy.
- Rust 1: `&mut self` is explicit. It can only hide behind `Cell`/`RefCell` on `&self`.
- TS 5: `this` binding is dynamic: unbound methods, `function` callbacks, fluent builders that return `this`, and field-vs-prototype shadowing.
- Python 4: `self` is explicit, so dynamic `this` is not the hazard. A method still mutates a class list through `self`, `__init__` calls an override before the subclass sets its fields, a function stored on the instance is not bound, and a builder that omits `return self` yields `None`.

**time-money-entropy**
- Go 3: float64 money, reading the clock twice, and `==` on `time.Time`.
- Rust 3: f64 money, truncated durations, and integer split remainders.
- TS 5: a single `number` type for money, mutable `Date` with month overflow, and `Math.random`/`Date.now` used directly in logic.
- Python 5: binary floats for cents, `datetime` read twice across midnight, `random` inside a branch, `round` half-even, and `timedelta(days=30)` used as a month.

**impure-functions**
- Go 4: package globals, in-place `sort`, pointer-argument rewrites, and global memo caches.
- Rust 2: `static` atomics and `&mut` parameters are visible in the signature but still slip through.
- TS 5: in-place `sort`, mutable module state, memoization with hidden dependencies, entropy in keys, and argument mutation.
- Python 5: `list.sort` in place, a read of a module global, appending to an argument, `@cache` that ignores a changed global, and `random` inside a key.

**macro-mixin-misuse**
- Go 2: no macros. Struct embedding acts as a mixin, without virtual dispatch and with shared embedded pointers.
- Rust 3: `macro_rules!` double evaluation, hidden `return`, and `vec![rc; n]` cloning one Rc.
- TS 3: `Object.assign` collisions, mixin order changing the `super` chain, and mutable state on prototypes.
- Python 3: no macros. Mixin MRO picks the first `save`, a mixin `save` that does not call `super` drops the rest, and a decorator calls the wrapped function twice.

**increment-decrement-operators**
- Go 3: `++` is a statement, but silent wrap on fixed-width ints, unsigned underflow, and narrowing conversions.
- Rust 3: debug builds panic and release builds wrap. `as` truncates, and `wrapping_*` masks underflow.
- TS 3: post-increment value semantics, precision loss past 2^53, and int32 coercion from bitwise ops.
- Python 2: no `++`, and `int` does not wrap. Adding `1.0` past the 53-bit mantissa does nothing, and `total += flag` counts `True` as `1`.

**improper-initialization**
- Go 5: zero values are everywhere: nil maps, nil pointer fields, zero timeouts, typed-nil interfaces, and `make` length vs. capacity.
- Rust 2: values must be initialized, but `#[derive(Default)]` and `unwrap_or_default` produce illegal states.
- TS 4: the `!` assertion, subclass fields initialized after the base constructor runs, TDZ, and `||` defaults.
- Python 4: an attribute set on only one branch, `value or default` replacing `0`, a method used before the last field is set, and an optional left as `None`.

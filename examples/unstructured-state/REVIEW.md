# Input and output contract audit

This review covers `lack-of-input-validation`, `unconstrained-inputs`, `duck-typing-without-contracts`, and `missing-undefined-returns` in Go, Rust, JavaScript, TypeScript, and Python. [ASSESSMENT.md](ASSESSMENT.md) separates qualitative susceptibility from actual corpus counts.

## Definitions and repairs

| Defect | Obligation | Audit outcome |
|---|---|---|
| lack-of-input-validation | Establish the runtime shape or domain required by an operation. | Added typed quantity boundaries and Python repairs; corrected fail-open status handling, nonblank-string checks, and misleading JS assertion descriptions. SQL string examples describe construction susceptibility, not executed injection. |
| unconstrained-inputs | Bound accepted size, work, allocation, depth, or combined expansion. | Added batch, buffer, and multiplicative-expansion pairs in every language. Fixed negative page limits by rejection rather than returning all rows. Added missing Python repairs. |
| duck-typing-without-contracts | Establish an expected shape, capability, or semantic contract before consuming a value as that contract. | Added dynamic assertion/downcast/payload cases; static interfaces do not establish units or runtime payload contracts. |
| missing-undefined-returns | Produce the value or failure signal promised by the function's output contract. | Reworked consumer-only examples into producer omissions or false-success returns. Intentional optional absence is allowed; blindly consuming a valid optional result is a separate defect. |

The expansion fixtures individually validate the item count and copy count, yet permit a combined output beyond the named budget. Their repairs compare the item count against the budget divided by the nonzero copy count before any expansion. Small inputs demonstrate the missing constraint without exhausting memory.

Functions carry parameter requirements and producer obligations. Variables carry values, aliases, and unresolved requirements from those boundaries. Missing annotations are useful structural signals, but typed inputs can still violate domain constraints. Runtime validation must establish the particular condition needed by a use.

## Research basis

- Input validation covers properties such as type, range, and consistency; it is distinct from safely constructing an output language such as SQL. Broad CWE classes should not be presented as proven vulnerabilities from syntax alone. [MITRE CWE-20](https://cwe.mitre.org/data/definitions/20.html)
- Resource consumption requires constraints on the relevant quantity of work or storage. These fixtures illustrate potential resource misuse, not a demonstrated denial of service. [MITRE CWE-400](https://cwe.mitre.org/data/definitions/400.html)
- TypeScript assertions are erased and do not validate runtime data. Python function and variable annotations are likewise not enforced by the runtime. [TypeScript handbook](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-assertions), [Python typing](https://docs.python.org/3/library/typing.html)
- Go numeric parsing supplies an error that callers must interpret. Rust parsing returns a `Result`; neither successful conversion alone nor an arbitrary fallback establishes a business-domain constraint. [Go strconv](https://pkg.go.dev/strconv#Atoi), [Rust str::parse](https://doc.rust-lang.org/std/primitive.str.html#method.parse)
- Rust dynamic downcasts return `None` when the concrete type does not match; unwrapping that result bypasses failure handling. [Rust Any](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref)
- Go return statements can return named result parameters, including unchanged zero values. Python return without an expression supplies `None`. A missing-result diagnosis therefore needs an output obligation, not just the presence or absence of a return token. [Go specification](https://go.dev/ref/spec#Return_statements), [Python return statements](https://docs.python.org/3/reference/simple_stmts.html#the-return-statement)

## Detection and verification limits

Tree-sitter queries identify syntax candidates: parameters, input-source calls, assertions, resource operations, functions, and returns. They cannot alone establish runtime types, business requirements, guard dominance, or complete return paths. A name resembling a standard API may also be shadowed or refer to an unrelated method.

Semantic confirmation must connect the boundary to its affected use, preserve binding identity, and check whether a relevant guard applies on the path taken. Reassignment and aliased mutation can invalidate a guard. Unknown calls and unsupported control flow must retain uncertainty. Co-occurring mutation, unstructured input, and concurrency become compound evidence only when their relationship is established; whole-function overlap alone is insufficient.

Fixed sources can retain source calls and syntax captures. Capture tests establish query behavior; runtime oracles establish the supplied scenario; neither establishes general detector accuracy. Query Lab expectations and production catalog statuses remain separate. See the [Query Lab guide](../../tooling/query-lab/README.md) and [production catalog](../../queries/catalog.json).

The validation and constraint corpus was executed as standalone programs, including all mirrored fixes and the new boundary, batch, buffer, and expansion scenarios. All fixes completed successfully and the new pairs produced their specified original/fixed outputs. These checks use small deterministic cases, do not execute a database or network service, and do not prove all valid/invalid inputs are handled. Existing arithmetic examples retain their documented language/build-mode behavior.

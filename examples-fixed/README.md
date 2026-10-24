# Fixed counterparts

This tree mirrors the 284 Go/Rust/TypeScript examples present when Query Lab was introduced, with independently repaired, standalone programs. Examples added in other languages are reported as unsupported by Query Lab. The original corpus and its manifests retain their original meaning and are not rewritten by this tool.

Repair rationales, source hashes, syntax-capture expectations, and runtime assertions live in [Query Lab sidecars](../tooling/query-lab/cases/). Query Lab analyzes each source independently; these files are not a single Go/Rust/TypeScript package. Go checks copy the original corpus `go.mod` into an isolated working directory.

```sh
python3 tooling/query-lab/query_lab.py compare --case shared-state/race-conditions/typescript/04_out_of_order_responses
python3 tooling/query-lab/query_lab.py verify-fixes
```

A repaired example can still contain state mutation, shared resources, or other susceptibility mechanisms. Passing its behavioral checks establishes the supplied scenario's expected outcome, not general defect freedom. See the [Query Lab guide](../tooling/query-lab/README.md) for limitations and the development workflow.

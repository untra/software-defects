# Fixed counterparts

This tree contains independently repaired, standalone counterparts of the supported example corpus. The four audited input/output defects have complete Go, Rust, JavaScript, TypeScript, and Python pairs. Other categories may still have unsupported cases; Query Lab reports these explicitly. Original manifests describe failing behavior, while sidecars describe both variants.

Repair rationales, source hashes, syntax-capture expectations, and runtime assertions live in [Query Lab sidecars](../tooling/query-lab/cases/). Query Lab analyzes each source independently; these files are not a single Go/Rust/TypeScript package. Go checks copy the original corpus `go.mod` into an isolated working directory.

```sh
python3 tooling/query-lab/query_lab.py compare --case shared-state/race-conditions/typescript/04_out_of_order_responses
python3 tooling/query-lab/query_lab.py verify-fixes
```

A repaired example can still contain state mutation, shared resources, or other susceptibility mechanisms. Passing its behavioral checks establishes the supplied scenario's expected outcome, not general defect freedom. See the [Query Lab guide](../tooling/query-lab/README.md) for limitations and the development workflow.

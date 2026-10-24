# Query Lab

Query Lab is an external-first development CLI for Tree-sitter queries. It runs the official Tree-sitter CLI against the same grammar packages as State Lens, reads `.scm` edits immediately, and reports exact captures as JSON. It does not execute source during parsing or query testing.

The supported Go/Rust/TypeScript development corpus has 284 original/fixed pairs, 84 experimental language/category packs, and source-hashed sidecar expectations. These packs expose **syntax candidates**, not validated bug diagnostics. Correct fixes often retain captures. Coverage reports distinguish tested cases from cases containing candidates; neither is detector accuracy. Examples subsequently added in other languages appear separately in `unsupported_cases`; supported-language tests neither execute them nor claim to cover them.

## Setup

Requires macOS or Linux, Python 3.12+, Cargo/Rust, and a C compiler. Runtime checks additionally need Go and Node 22.20+. CI uses Rust 1.95.0 and Go 1.26.3. From the repository root:

```sh
python3 tooling/query-lab/query_lab.py setup
python3 tooling/query-lab/query_lab.py doctor
```

Only `setup` downloads dependencies. It installs `tree-sitter-cli` 0.25.10 locally and verifies grammar crate downloads against `tooling/state-lens/Cargo.lock`: Go 0.25.0, Rust 0.24.0, TypeScript 0.23.2. It builds the supplied parser C sources without regenerating grammars. Configuration and native libraries live in the ignored `.cache/`, keyed by platform and package pins; global Tree-sitter configuration is not used. Setup is serialized with a file lock. Changing a grammar pin requires updating both projects together.

## Agent workflow

```sh
python3 tooling/query-lab/query_lab.py list --language go --defect divide-by-zero
python3 tooling/query-lab/query_lab.py nodes --language go --kind binary_expression
python3 tooling/query-lab/query_lab.py parse examples/unstructured-state/divide-by-zero/go/01_panel_current.go --lines 5:10 --format text
python3 tooling/query-lab/query_lab.py check --language go --query tooling/query-lab/queries/go/unstructured-state/divide-by-zero/signals.scm
python3 tooling/query-lab/query_lab.py run --query tooling/query-lab/queries/go/unstructured-state/divide-by-zero/signals.scm --source examples/unstructured-state/divide-by-zero/go/01_panel_current.go
python3 tooling/query-lab/query_lab.py compare --case unstructured-state/divide-by-zero/go/01_panel_current
python3 tooling/query-lab/query_lab.py test --language go --defect divide-by-zero
python3 tooling/query-lab/query_lab.py test
python3 tooling/query-lab/query_lab.py coverage
```

Inspect the syntax tree and grammar fields before editing a query. Use normal file-editing tools to create or modify `.scm` files, then `check`, `run`, and `test` without a rebuild. Compile failures include Tree-sitter's node/field/syntax diagnostics. `list` includes matching query paths and limitations. `compare` returns actual captures, missing/unexpected captures, expected captures for each variant, and the repair rationale.

Outputs have `schema_version: 1`; JSON is the default. Use `--format text` before or after the subcommand for readable output. Captures include pattern indices, names, captured source text, and **half-open UTF-8 byte ranges**. `--lines` is an inclusive, one-indexed display filter that retains enclosing tree nodes; it does not parse a detached snippet. These are Tree-sitter concrete syntax trees, not compiler-resolved ASTs.

Each query invocation uses private immutable source/query snapshots and reports their SHA-256 hashes and grammar version. Sources over 2 MiB, commands exceeding 30 seconds, and combined subprocess output over 16 MiB fail explicitly. Parser errors remain visible. The adapter rejects unknown output and match-limit warnings; unsupported custom predicates/directives are rejected instead of silently ignored. Supported predicates are the Rust binding's equality, regex, and string-set predicates. An unsupported node-kind lookup returns an empty list.

Exit codes: `0` successful execution or matching expectations; `1` expectation/runtime mismatch; `2` invalid input, stale expectations, parse error, missing tools, timeout, or incomplete execution. A query returning no matches succeeds. Failed batch cases are retained in the JSON report instead of hiding later cases. Compilation and execution never upload source.

## Expectations and interpretation

`cases/<family>/<defect>/<language>/<example>.json` links to the original and mirrored fixed source through its location. Each sidecar contains:

- Source hashes, repair rationale, and analysis limitations.
- Independently specified capture-name/range multisets for both variants, including explicit empty results.
- Query paths and a separate behavioral oracle with expected exit status and stdout assertions.

The initial expectations were anchored in reviewed source tokens, then checked against grammar boundaries. They were not recorded by copying detector output. In particular, declarations are not calls, and Rust macro token trees do not expose embedded calls as ordinary `call_expression` nodes. The tests assert these boundaries. There is no snapshot-update or automatic blessing command: inspect a changed source/query and edit its expectations deliberately. After whitespace edits, update source hashes and ranges without changing the intended capture tokens.

The experimental API-name queries intentionally do not infer receiver types or resolve shadowed functions. A method named `lock`, for example, is only a candidate. The arithmetic query captures division/remainder operators even when guarded correctly. No source filename, manifest explanation, `bug_lines`, or susceptibility score is supplied to a query. Existing `bug_lines` keep their bug-location meaning.

All experimental packs live under this tool, **outside production `queries/`**, because State Lens embeds every `.scm` in that directory. Promotion requires supported semantic handlers, production metadata, independent analyzer expectations, and State Lens validation. This change does not add shared-state or unstructured-state diagnostics to the production analyzer.

## Behavioral checks and tests

```sh
python3 tooling/query-lab/query_lab.py verify-fixes --case mutated-state/var-let-mut/go/01_shadowed_err_in_if
python3 tooling/query-lab/query_lab.py verify-fixes
python3 -m unittest discover -s tooling/query-lab -p 'test_*.py'
cargo test --locked --manifest-path tooling/state-lens/Cargo.toml --test query_lab -- --ignored
```

`verify-fixes` is the only corpus command that executes example code. It copies each fixed source into a separate temporary directory, builds Go/Rust examples individually, preserves the corpus Go module semantics, and runs with a bounded process group, CPU/file/descriptor limits, and a temporary filesystem location. The default runtime deadline is 15 seconds, with a sidecar override capped at 30 seconds. Original crashing/racy examples are not executed. This is resource-bounded execution of repository fixtures, not a security sandbox for arbitrary untrusted programs. Network fixtures use local loopback servers and require permission to bind sockets.

Some example reports measure elapsed time. Their results are development checks, not performance guarantees or proofs of race freedom. Unsupported behavioral checks must carry an explicit reason; they are reported separately. The malformed Rust length-prefix example rejects the invalid record rather than guessing its intended contents; its rationale records that ground-truth ambiguity.

Tests cover adapter failure modes, Unicode/CRLF byte ranges, duplicate/multiline captures, live query edits, malformed syntax, stale hashes, corpus completeness, and a positive/benign-negative syntax control for every pack. The explicit Rust parity test compares all 30 embedded production queries on representative fixtures against direct captures from the pinned Rust bindings. CI separates static query tests from fixed-example execution and uploads both reports.

## Optional external editing assistance

[ts_query_ls](https://github.com/ribru17/ts_query_ls#standalone-tool) supplies standalone `check`, `lint`, `format`, and editor completion. It is optional and is not installed by setup; the pinned official CLI remains authoritative for compilation and execution. [Official query CLI documentation](https://tree-sitter.github.io/tree-sitter/cli/query.html) describes direct usage, but newer documentation may include flags absent in the pinned version. Prefer the installed CLI's `--help` when invoking it directly.

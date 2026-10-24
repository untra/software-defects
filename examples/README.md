# Stateful defect examples

Minimal, runnable programs that each contain one stateful defect. They serve as a reference corpus and as blind test cases for automated (AI) defect detection.

```
examples/
├── go.mod                              pins Go language semantics (1.22 loop variables)
└── <family>/                           e.g. mutated-state
    ├── ASSESSMENT.md                   per-language susceptibility scores (1–5) and rationale
    └── <defect-slug>/                  slugs match src/content.py and the site URLs
        ├── manifest.json               ground truth for every file in this folder
        ├── go/NN_scenario.go
        ├── rust/NN_scenario.rs
        ├── typescript/NN_scenario.ts
        └── python/NN_scenario.py
```

The number of files in a `<defect>/<language>/` folder equals that cell's score in `ASSESSMENT.md`.

## Conventions

- Each file stands alone with its own entry point. Running it prints a wrong result or crashes.
- The code has **no hints**: no `BUG` comments, and names describe the domain, not the defect. Ground truth lives only in `manifest.json`, so a detector can be scored blind.
- Code style: minimal comments and named constants.

## manifest.json

```json
{
  "family": "mutated-state",
  "defect": "var-let-mut",
  "name": "var / let / mut Declarations",
  "essence": "...",
  "cwes": ["CWE-457", "CWE-563"],
  "examples": [
    {
      "file": "go/01_shadowed_err_in_if.go",
      "language": "go",
      "bug_lines": [13],
      "summary": "what goes wrong",
      "mechanism": "why the language lets it happen",
      "observed_output": "stdout, plus the error line on a crash",
      "exit_code": 0,
      "deterministic": true,
      "expected_behavior": "what a correct program would print"
    }
  ]
}
```

`bug_lines` are 1-indexed lines a detector should flag. `deterministic: false` marks examples that depend on the clock or on randomness. Their `observed_output` is from one representative run.

## Running

Run from `examples/`:

```sh
go run mutated-state/var-let-mut/go/01_shadowed_err_in_if.go
rustc --edition 2021 -o /tmp/ex mutated-state/var-let-mut/rust/01_shadowed_accumulator.rs && /tmp/ex
node mutated-state/var-let-mut/typescript/01_var_hoisting.ts     # Node >= 22.18 strips types natively
go run unstructured-state/divide-by-zero/go/02_float_ratio.go
python3 mutated-state/var-let-mut/python/01_closure_late_binding.py
```

The Rust examples assume a debug build (plain `rustc`), where integer overflow panics. The TypeScript files use only erasable syntax (no `enum` or `namespace`). Several are deliberately rejected by `tsc --strict`, because the bug is the kind a type checker may or may not catch.

## Families

| Family | Status |
|---|---|
| [mutated-state](mutated-state/ASSESSMENT.md) | Go · Rust · TypeScript · Python, 125 examples |
| [unstructured-state](unstructured-state/ASSESSMENT.md) | Go · Rust · TypeScript · Python, 132 examples |
| [shared-state](shared-state/ASSESSMENT.md) | Go · Rust · TypeScript · Python, 141 examples (see its harness conventions) |
| [compound-defects](compound-defects/ASSESSMENT.md) | four flagship programs, one language each; scores are 0–2 and are not file counts |
| common-defects | planned |

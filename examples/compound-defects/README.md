# Compound defect examples

Runnable programs for the three compound classes (nondeterminism, runtime errors, data corruption). Scores and the reason a language is missing live in [ASSESSMENT.md](ASSESSMENT.md).

```
compound-defects/
  ASSESSMENT.md
  <class>/                      nondeterminism | runtime-errors | data-corruption
    <defect-slug>/              slugs match the site
      manifest.json
      <language>/NN_scenario.ext
```

The tree is three levels deep on purpose. `src/corpus.py` and Query Lab load `examples/<family>/<slug>/manifest.json` only. These manifests stay out of that walk, and the Query Lab case count stays 284, until a defect is enrolled on purpose.

## Conventions

Same rules as [../README.md](../README.md). One `main`, no defect comments, domain names, named constants. Ground truth stays in `manifest.json`.

A score of 0 in `ASSESSMENT.md` means no file for that language. Scores are not file counts: each nonzero cell has one program. A score of 1 must name the assessment-approved escape hatch or trapped form in `mechanism`, such as `unsafe`, `ctypes`, `SharedArrayBuffer`, dynamic types, or a wrapped size followed by a bounds panic.

Every manifest entry has two `ingredients`, each with a family, defect slug, a checkable `suggests` sentence, and exact source substrings in `evidence`. These describe the composition without putting labels in the blind source.

This folder does not have `examples-fixed/` twins or Query Lab sidecars yet. Add those per defect, and teach the lab to skip or include this tree, before flattening the path.

## Running

From `examples/`:

```sh
python3 compound-defects/runtime-errors/aliased-default-type-confusion/python/01_shared_note_bag.py
node compound-defects/nondeterminism/out-of-order-async/typescript/01_last_write_wins.ts
go run compound-defects/nondeterminism/lazy-init-visibility-race/go/01_published_before_ready.go
rustc --edition 2021 -o /tmp/ex compound-defects/data-corruption/signedness-poisons-aggregate/rust/01_refund_as_u32.rs && /tmp/ex
```

All programs use their language's standard library and, for raw memory mappings, operating-system APIs. The mapping examples target macOS and Linux. Rust examples compile directly with `rustc --edition 2021` in debug mode; JSON examples implement only the small flat-object subset used by their fixture. They are not general-purpose JSON parsers.

## Tests

From the repository root:

```sh
node --test examples/compound-defects/test/*-ingredients.test.mjs
```

Run one language by selecting its test file, for example:

```sh
node --test examples/compound-defects/test/python-ingredients.test.mjs
node --test examples/compound-defects/test/typescript-ingredients.test.mjs
```

The suites cover 43 Python, 44 Go, 44 Rust, and 37 TypeScript programs (168 total). They require Python 3, Go, Rust, and Node 22.18 or newer on PATH. No test dependencies are installed. Go and Rust programs compile into temporary directories with a 60-second compilation timeout; each program runs as a subprocess with a 10-second timeout. The Go build cache also lives under the system temporary directory.

Coverage comes from `ASSESSMENT.md`; tests reject missing, extra, unmanifested, and score-zero programs. Each example must match its exact ingredient pair, include its evidence, and reproduce its recorded result. Timing-dependent examples must match in at least nine of ten executions.

`exit_code` records the executable's status, not the `go run` wrapper's status. Signal termination uses the shell convention `128 + signal number`. `observed_output` contains stdout and the exception or panic payload, omitting stack traces, source locations, and runtime banners. Memory examples exercise process-local allocations; their manifests state the platform or runtime assumptions of unsafe operations. Temporary files and compiled executables are removed after use.

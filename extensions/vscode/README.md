# State Lens for VS Code

A category-colored lens for state susceptibility in Go, Rust, TypeScript, JavaScript, and Python. Highlighted code is not necessarily wrong. Local mutation and observable effects remain distinct; only supported observable effects propagate through resolved calls.

## Install locally

Choose one language: `typescript`, `javascript`, `go`, `rust`, or `python`.

```sh
make install-vscode VSCODE_LANGUAGE=javascript
```

Each language has a separate extension identity (`state-lens.state-lens-<language>`) and bundles its native server. No separately installed executable is required. Artifacts are written to `build/vsix/` with language, version, and target in the filename. The build defaults to the host platform; `VSCODE_TARGET` selects another installed Rust target and requires its native compiler toolchain.

Settings and commands use `stateLens.<language>`, for example `stateLens.javascript.enabled`. The machine-level `serverPath` setting is an explicit development override; empty selects the bundled executable. Overrides must advertise exactly the selected language and a compatible analysis contract. The old monolithic extension settings are not migrated automatically.

JavaScript supports `.js`, `.mjs`, and `.cjs`; TypeScript supports `.ts`. JSX and TSX are deferred. Relative ES-module imports resolve within the JavaScript extension; dynamic/CommonJS call targets remain explicitly unresolved. All five extensions can be installed together and toggled independently. Extensions require trusted workspaces. Packaging does not publish anything.

Build and validate an archive with:

```sh
npm run package --prefix extensions/vscode -- javascript
python3 tooling/state-lens/scripts/verify_vsix.py javascript darwin-arm64
npm run test:isolation --prefix extensions/vscode -- javascript
```

Use your host target instead of `darwin-arm64`. Omit the language from `test:isolation` after building all five packages to test coexistence.

## Use

Open a `.go`, `.rs`, `.ts`, or `.py` file. Sources use dotted borders and subtle fills: green (`#78A65A`) for mutated state, blue (`#3D85C6`) for unstructured state, and red (`#DD3333`) for shared state. Propagated effects use dashed borders and lighter fills. Compound findings retain their yellow, light-blue, or magenta outlines. Overlaps retain all findings in the hover.

Hover a source for its colored defect title, short explanation, **Guide**, and **Evidence**. Evidence opens the selected finding in the Explorer with origin, effect, resolution, taxonomy, and navigable source locations and call paths. File paths and byte offsets appear in Evidence rather than the hover.

A green declaration border identifies an **Impure** function. State Lens uses a strict convention: local mutable bindings and writes also qualify, alongside argument/receiver/global mutation, external mutable reads, and nondeterminism. Local effects still do not propagate to callers. Structural signals alone do not establish impurity. Nested closures own their findings. No marker means no qualifying finding was detected; it does not prove purity.

Function inlays summarize findings as `◆ Impure` or `◆ Signals`. The supported affected scope consists of the owning function and resolved call paths shown in Evidence; it is not a complete dataflow analysis.

- **State Lens: Toggle Susceptibility Lens** enables/disables decoration and source hovers.
- **State Lens: Refresh Analysis** requests a new snapshot.
- `stateLens.<language>.directOnly` hides propagated sources, markers, and Evidence entries.
- `stateLens.<language>.opacity` adjusts source fill intensity; function bodies remain untinted.
- `stateLens.<language>.accessibleMarkers` thickens source borders; borders and Evidence labels are always available.
- `stateLens.<language>.diagnostics` publishes concise Information diagnostics for Problems, agents, and other LSP consumers. It defaults to true.

The status bar reports partial analysis when syntax errors, unresolved calls, or background indexing remain. Edits clear old decorations, hovers, and Evidence immediately. Evidence actions from outdated snapshots are ignored. Changing the server executable requires reloading the extension host.

## Debug unavailable analysis and smoke every language

Run **State Lens: Show Debug Information** to open a JSON document containing the extension version,
configured executable, resolved executable and SHA-256, environment override, PATH, server identity, visible document versions,
applied findings, and the last 200 output lines. The State Lens output channel records request URI,
version, finding count, indexing state, elapsed time, and failures. Analysis requests have a ten-second
deadline and cancel on timeout. Save the debug document alongside a reproduction when handing work
to another developer or LLM; it contains local paths and analysis logs.

Bundled servers avoid the previous dependency on an independently installed executable. The extension checks both the analysis contract and the server's compiled language before accepting results.

Run these commands from `extensions/vscode`:

```sh
# Inspect a standalone development server across the corpus.
npm run test:health -- --server ../../tooling/state-lens/target/release/state-lens --report /tmp/state-lens-release.json

# Test the Python VSIX and its bundled server.
npm run package -- python
npm run test:e2e -- --language python --health-only --report /tmp/state-lens-python-editor
```

The editor runner requires `--language go|rust|typescript|javascript|python`; the protocol runner accepts a language filter and `--file`; the protocol runner
allows repeated `--file` arguments. Editor runs also accept the existing `--shard` selector and
`STATE_LENS_E2E_JOBS` concurrency setting (default two hosts). The isolated hosts disable the bundled
TypeScript and Copilot providers to keep their startup/network work out of State Lens hover timing.
Protocol defaults to the whole repository workspace;
editor defaults to one corpus family per host. Use `--workspace ../..` to reproduce repository-wide
indexing in an editor host. Editor `--server` expects a path. Protocol `--server` also accepts a PATH command.

Protocol health gives each document five seconds to return a matching URI, version, and source hash.
Its JSON report records the resolved executable and SHA-256, server identity, workspace, per-file timing,
findings, partial-analysis flags, stderr (with Rust backtraces enabled), and protocol errors. It checkpoints
after each file, fails on error logs, and reports unexecuted files after a fatal timeout/crash. Editor
health gives each file ten seconds for analysis, applied decorations, diagnostics, inlays, and hover links.
It writes `health-report.json` and retains `editor-shards/<shard>/state-lens.log` and VS Code logs.
Each run should use a fresh report directory so retained artifacts are unambiguous.

Health accepts valid zero-finding and partial snapshots: unsupported detection coverage must not mask
whether a file can open and render. The existing audit without `--health-only` still checks baseline
drift, manifest coverage, compound identities, edits, and presentation controls. Keep both checks when
changing the analyzer; a health pass does not establish detection accuracy or completed indexing.

For the next debugging session: preserve a failing report; compare installed/release executable hashes;
reduce with `--file` and `--language`; compare narrow/full workspaces; inspect server stderr and request
logs; then rerun both health layers and the existing audit after fixing the cause. Do not update
`smoke.json` to conceal protocol or presentation failures. The health transport regression tests inject
an unsupported-language response, fragmented/out-of-order frames, a hung request, and a server panic.

## Refine the view

1. From the repository root, run `npm test --prefix extensions/vscode`.
2. Launch **State Lens: Extension Development Host** from Run and Debug. The existing launch task builds both the extension and release server.
3. Open `examples/shared-state/async-io-misuse/go/03_goroutine_leak_unbuffered_result.go`. Hover line 12: `var-let-mut`, `Mutable binding.`, and the two links should appear. Inspect the external read inside `firstResult` and its green function marker.
4. Compare representative mutated-state, unstructured-state, and shared-state examples in light, dark, and high-contrast themes. Check multiline borders, overlapping findings, nested closures, Evidence, toggle, edits, and `directOnly`.
5. For presentation changes, edit `src/hover.ts`, `src/ranges.ts`, or decoration options in `src/extension.ts`, then rebuild and reload the development host. Standard server hover, diagnostics, and inlay text live in `tooling/state-lens/src/server.rs`; rebuild the server and reload the host after changing them.
6. Run `cargo +1.95.0 test --locked --manifest-path tooling/state-lens/Cargo.toml`, then `cargo +1.95.0 build --release --locked --manifest-path tooling/state-lens/Cargo.toml` and `python3 tooling/state-lens/scripts/smoke.py --report build/smoke/protocol`.
7. Package with `npm run package --prefix extensions/vscode -- javascript`. Run the packaged editor checks with `npm run test:e2e --prefix extensions/vscode -- --language javascript --shard examples/shared-state --report ../../build/smoke/editor`; omit `--shard` for the full corpus.

## Limits

This release displays existing findings from all three base families and compound findings. It has no validated bug diagnostics, general type inference, dynamic-dispatch resolution, full alias analysis, or macro expansion. Related CWEs are taxonomy references, not vulnerability claims. TSX/JavaScript are not included in this first grammar pack.

See [the analyzer documentation](https://github.com/untra/software-defects/tree/main/tooling/state-lens) for supported resolution, scoring, coverage, and analysis coverage.

### Packaging and selecting the matching server

`npm run package -- <language>` builds a language-only release analyzer and bundles it in the matching VSIX. The packaged server is used by default; `stateLens.<language>.serverPath` is an explicit development override. PATH and environment variables are not used by the extension.

The extension checks the server's division-analysis capability at startup. An older binary produces an explicit compatibility warning. **State Lens: Show Debug Information** identifies the command, resolved executable, hash, and server information. “Active (partial)” means indexing, parse errors, or unresolved calls; its tooltip lists the reasons. Supported findings still display, including a blue denominator and Warning for the immutable zero-divisor example. Traced denominator mutation uses yellow fill.

Run the independent division smoke expectations and guard-edit checks with:

```sh
python3 ../../tooling/state-lens/scripts/smoke.py --shard examples/unstructured-state/divide-by-zero
python3 ../../tooling/state-lens/scripts/smoke.py --shard examples-fixed/unstructured-state/divide-by-zero
npm run test:e2e -- --shard examples/unstructured-state/divide-by-zero
npm run test:e2e -- --shard examples-fixed/unstructured-state/divide-by-zero
```

The editor suite checks exact denominator ranges, Warning diagnostics, blue/yellow fills, and original → fixed → restored → mutated → restored buffer changes. Health-only runs do not establish detection correctness. Scoped protocol baseline updates merge only the selected paths after their independent checks pass.

## Contextual status and server lifetime

The footer shows one State Lens entry for the active supported file. Its tooltip identifies the language, and clicking toggles that language. The State Lens Evidence view follows the active file; language-specific commands, settings, and output channels remain available. Unsupported editors hide both UI entries.

Servers run only for enabled languages with visible file editors. Split editors can keep several language servers running, while hidden tabs alone do not. Closing the last visible editor or disabling its language stops the corresponding server and clears its results. Returning to that language starts a fresh server. Failed startup requires Refresh Analysis, re-enabling, or a new visibility transition to retry.

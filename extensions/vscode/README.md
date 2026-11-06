# State Lens for VS Code

A category-colored lens for state susceptibility in Go, Rust, TypeScript, and Python. Highlighted code is not necessarily wrong. Local mutation and observable effects remain distinct; only supported observable effects propagate through resolved calls.

## Install locally

From the repository root:

```sh
make install-server install-vscode
```

This installs `state-lens` into `~/.local/bin` and the packaged `.vsix` into VS Code. The extension resolves the server from `stateLens.serverPath`, then the `STATE_LENS_SERVER` environment variable, then `PATH`. The extension does not download or execute project code and does not upload source. It runs only in trusted workspaces; for remote workspaces, install the binary in the remote environment.

For iteration, open the repository in VS Code and launch **State Lens: Extension Development Host**. It builds the release server, points `STATE_LENS_SERVER` at it, and prompts for a folder to open.

## Use

Open a `.go`, `.rs`, `.ts`, or `.py` file. Sources use dotted borders and subtle fills: green (`#78A65A`) for mutated state, blue (`#3D85C6`) for unstructured state, and red (`#DD3333`) for shared state. Propagated effects use dashed borders and lighter fills. Compound findings retain their yellow, light-blue, or magenta outlines. Overlaps retain all findings in the hover.

Hover a source for its colored defect title, short explanation, **Guide**, and **Evidence**. Evidence opens the selected finding in the Explorer with origin, effect, resolution, taxonomy, and navigable source locations and call paths. File paths and byte offsets appear in Evidence rather than the hover.

A green declaration border identifies an **Impure** function. State Lens uses a strict convention: local mutable bindings and writes also qualify, alongside argument/receiver/global mutation, external mutable reads, and nondeterminism. Local effects still do not propagate to callers. Structural signals alone do not establish impurity. Nested closures own their findings. No marker means no qualifying finding was detected; it does not prove purity.

Function inlays summarize findings as `◆ Impure` or `◆ Signals`. The supported affected scope consists of the owning function and resolved call paths shown in Evidence; it is not a complete dataflow analysis.

- **State Lens: Toggle Susceptibility Lens** enables/disables decoration and source hovers.
- **State Lens: Refresh Analysis** requests a new snapshot.
- `stateLens.directOnly` hides propagated sources, markers, and Evidence entries.
- `stateLens.opacity` adjusts source fill intensity; function bodies remain untinted.
- `stateLens.accessibleMarkers` thickens source borders; borders and Evidence labels are always available.
- `stateLens.diagnostics` publishes concise Information diagnostics for Problems, agents, and other LSP consumers. It defaults to true.

The status bar reports partial analysis when syntax errors, unresolved calls, or background indexing remain. Edits clear old decorations, hovers, and Evidence immediately. Evidence actions from outdated snapshots are ignored. Changing the server executable requires reloading the extension host.

## Refine the view

1. From the repository root, run `npm test --prefix extensions/vscode`.
2. Launch **State Lens: Extension Development Host** from Run and Debug. The existing launch task builds both the extension and release server.
3. Open `examples/shared-state/async-io-misuse/go/03_goroutine_leak_unbuffered_result.go`. Hover line 12: `var-let-mut`, `Mutable binding.`, and the two links should appear. Inspect the external read inside `firstResult` and its green function marker.
4. Compare representative mutated-state, unstructured-state, and shared-state examples in light, dark, and high-contrast themes. Check multiline borders, overlapping findings, nested closures, Evidence, toggle, edits, and `directOnly`.
5. For presentation changes, edit `src/hover.ts`, `src/ranges.ts`, or decoration options in `src/extension.ts`, then rebuild and reload the development host. Standard server hover, diagnostics, and inlay text live in `tooling/state-lens/src/server.rs`; rebuild the server and reload the host after changing them.
6. Run `cargo +1.95.0 test --locked --manifest-path tooling/state-lens/Cargo.toml`, then `cargo +1.95.0 build --release --locked --manifest-path tooling/state-lens/Cargo.toml` and `python3 tooling/state-lens/scripts/smoke.py --report build/smoke/protocol`.
7. Package with `npm run package --prefix extensions/vscode`. Run the packaged editor checks with `npm run test:e2e --prefix extensions/vscode -- --shard examples/shared-state --report ../../build/smoke/editor`; omit `--shard` for the full corpus.

## Limits

This release displays existing findings from all three base families and compound findings. It has no validated bug diagnostics, general type inference, dynamic-dispatch resolution, full alias analysis, or macro expansion. Related CWEs are taxonomy references, not vulnerability claims. TSX/JavaScript are not included in this first grammar pack.

See [the analyzer documentation](https://github.com/untra/software-defects/tree/main/tooling/state-lens) for supported resolution, scoring, coverage, and analysis coverage.

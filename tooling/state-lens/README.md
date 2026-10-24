# State Lens

State Lens identifies mechanisms associated with mutated-state defects in Go, Rust, and TypeScript. It produces source ranges, executable-scope summaries, and evidence paths for a VS Code background lens. A highlighted mechanism can be completely intentional and correct.

This is the first, mutated-only release. The engine does not diagnose the bugs described by the site's examples. It preserves the distinction between mechanism coverage, susceptibility, and demonstrated incorrect behavior.

## Build and use

From the repository root:

```sh
cargo build --release --locked --manifest-path tooling/state-lens/Cargo.toml
tooling/state-lens/target/release/state-lens check-queries
tooling/state-lens/target/release/state-lens scan path/to/project
```

`scan` emits versioned JSON to stdout. `serve` runs the same engine over stdio with LSP framing. Errors and build messages use stderr. Parsing never executes project code or uploads source. See [VS Code installation](../../extensions/vscode/README.md) to build and install the extension.

The grammar crates are pinned exactly; all transitive versions are recorded in `Cargo.lock`. Queries are embedded at build time, so installed binaries need no query directory at runtime.

## Analysis contract

The pipeline is source snapshot → Tree-sitter captures → scoped bindings and modeled effects → resolved call graph → executable-scope summaries → display ranges.

Each finding carries a stable rule ID, family/defect slugs, a half-open UTF-8 byte trigger range, an optional owning-function range, an effect, a direct/propagated origin, resolution level, original evidence, and a call path. Related CWEs come from the site taxonomy and do not establish a vulnerability. Document reports include source SHA-256, editor version, syntax-error status, and unresolved boundaries. Reports include a schema version and generation.

`local_mutation` and `structural_signal` remain within their executable scope. Supported argument, receiver, global-state, and nondeterministic effects are represented separately. Resolved function calls propagate observable effects to callers; argument effects are substituted at the call site. Literal local values can contain argument mutation. Closures own their own effects; creating a closure does not execute it. Reads and writes of captured mutable bindings are treated conservatively as external state.

Unknown calls remain unresolved, and that uncertainty propagates through resolved callers. Absence of color is not a proof of purity. Recursive propagation terminates by deduplicating original evidence and preventing repeated call edges in an evidence path. A call path is one explanation, not an enumeration of all paths.

### Supported resolution

| Language | Supported calls | Deliberate limits |
|---|---|---|
| TypeScript | Lexically bound functions and simple callable locals; relative named/namespace imports; direct exported functions | No tsconfig aliases, re-exports, default imports, CommonJS, dynamic dispatch, TSX, or full type checking |
| Go | Local functions, same-package files, explicitly aliased imports into the nearest `go.mod` module | No replace/workspace directives, build-tag evaluation, interface/method dispatch, or inferred package-name aliases |
| Rust | Local functions and simple `use crate::module::function` / `use self::module::function` imports | No grouped/glob use resolution, macro expansion, trait/method dispatch, cfg evaluation, or compiler ownership analysis |

Clock APIs, selected explicitly typed collection mutations, and Go standard-library sorting have models. Unresolved methods with familiar names remain candidates. Arbitrary numeric expressions are not labeled as money calculations. Candidate aliases and initialization mechanisms do not prove a defect or an escaping effect.

Tree-sitter trees update incrementally. Direct facts are cached per document; edits invalidate that document's facts. Workspace graph propagation is recomputed from current direct facts, including effect removals. This is intentionally simpler than maintaining a persistent incremental graph. General alias escape and callback-execution models remain extension points.

Files larger than 2 MiB are skipped; CLI directory scans respect ignore files and omit dependency/build directories. Go-generated-file markers are skipped. The server indexes closed files between protocol messages, prioritizing editor work. Errors and unknown calls remain visible as incomplete analysis.

### Editor protocol

The server supports `initialize`, open/change/close/save synchronization, watched-file updates, configuration changes, hover, inlay hints, shutdown, and exit. Positions use UTF-16; conversion occurs only at the protocol boundary. `stateLens/ranges` accepts `{ "textDocument": { "uri": "file:///..." } }` and returns:

- `schemaVersion`, `generation`, `uri`, `version`, `sourceHash`;
- `findings`, each containing the raw finding plus UTF-16 `triggerRange` and `functionRange`;
- function/body ranges, `unresolved`, `parseErrors`, and `indexing`.

`stateLens/indexing` notifies the client when background indexing completes. Changes queued during analysis invalidate the response. The client also checks its workspace epoch and current document version before accepting any decoration. Diagnostics are opt-in: with `initializationOptions.diagnostics` (or `settings.stateLens.diagnostics` / `settings.diagnostics` in `workspace/didChangeConfiguration`) set to `true`, the server publishes one Hint-severity diagnostic per finding for each open document. Each carries `code` = rule ID, a `codeDescription` link to the site page, evidence and call path as `relatedInformation`, `data` = family/origin/effect, and the analyzed document `version`. Publishing happens only when no document change is pending; superseded analyses are discarded and retried, unchanged results are not resent, and documents are cleared on close or when diagnostics are disabled. `textDocument/inlayHint` returns one per-function family summary at the body start. These remain susceptibility signals; validated bug-pattern diagnostics are a future, independently scored capability.

## Queries and corpus

Use [Query Lab](../query-lab/README.md) to inspect trees, execute edited queries without rebuilding, and test original/fixed corpus pairs with the same pinned grammars. Its experimental packs remain separate from the embedded production queries. Run the opt-in `query_lab` integration test after Query Lab setup to verify external CLI capture parity.

[queries/rules.json](../../queries/rules.json) indexes the 27 language/category rule packs and their grammar versions, capture names, limitations, and CWE references. Each language also has shared structure queries. All nine mutated-state categories have a pack; a pack's existence does not imply complete scenario coverage.

Add a rule by extending the relevant `.scm` file, adding any required semantic handler and metadata, and adding independently specified positive and negative expectations. Keep code free of detector labels. Do not use filenames, manifest explanations, output strings, or the assessment's susceptibility scores as analysis inputs.

[coverage.json](coverage.json) inventories every authored mutated-state example by source hash, category, language, and current mechanism coverage. It deliberately records bug detection as unsupported. Regenerate it after corpus/query changes:

```sh
cargo build --locked --manifest-path tooling/state-lens/Cargo.toml
python3 tooling/state-lens/scripts/corpus.py inventory > tooling/state-lens/coverage.json
```

Each corpus file is analyzed in isolation because the original examples are standalone programs, not one multi-file package. Existing manifests and `bug_lines` retain their original meaning. The inventory does not become ground truth merely because the current analyzer produced it.

[fixtures/expectations.json](fixtures/expectations.json) contains manually specified susceptibility expectations, outside the source. It includes correct-but-susceptible code, pure controls, and cross-file propagation. These are parser/analysis fixtures, not additional runnable bug examples. The scorer checks source hashes before comparing exact ranges, function ownership, effect/origin pairs, and per-defect counts. Any mismatch fails. Fixtures and corpus are development data; no held-out accuracy is claimed.

```sh
tooling/state-lens/target/debug/state-lens score tooling/state-lens/fixtures/expectations.json
python3 tooling/state-lens/scripts/corpus.py blind /tmp/state-lens-blind --key-file /tmp/state-lens-private-map.json
```

The mutated-state blind exporter randomizes opaque case directories, renames sources to `main` plus their language extension, and keeps the private mapping outside the source directory. Go language semantics are preserved with the corpus `go.mod`. Run each case separately. This exporter targets the existing standalone corpus; it does not rewrite imports in arbitrary multi-file projects. Hold out whole scenario families/variants before authoring rules when collecting a future evaluation set.

## Validation and performance

`make smoke` builds the server and packaged VS Code extension, then runs both smoke layers and writes
`build/smoke/smoke-report.json` and `build/smoke/smoke-report.md`. `make smoke-audit` runs the same audit
and prints its Markdown path. Both layers run even if the first fails. `make smoke-update` updates
`smoke.json` only after a complete protocol run without failures; review baseline changes deliberately.

```sh
python3 tooling/state-lens/scripts/smoke.py --report build/smoke/protocol
npm run test:e2e --prefix extensions/vscode -- --shard examples/mutated-state --report ../../build/smoke/editor
```

The protocol runner uses `--jobs` (default CPU count); the editor runner uses
`STATE_LENS_E2E_JOBS` (default the smaller of six and CPU count). Six shards isolate each corpus
root/family, and each defect/language group has a ten-second deadline. The editor runner caches
VS Code 1.142.0 and uses private workspaces and user data. Initial protocol hovers probe position
zero while requests race document changes; settled hovers exercise actual triggers and Markdown links.
Only requests invalidated by later document mutations may return ContentModified during the stress phase.

Reports retain one row per layer/file, including failures and audit gaps. An original file is a gap
when it has no hints or a manifest bug line is not intersected by a diagnostic trigger range.
Half-open ranges exclude an ending line at character zero. Fixed files have no manifest coverage.
These are coverage signals, not detector accuracy; manifests never influence analysis. Gaps are
nonfatal unless `--strict-gaps` is passed to either runner. Errors, warnings, timeouts, crashes,
invalid ranges, count mismatches, and baseline drift fail the run. The Markdown includes query or
server/extension paths for follow-up investigation; absent production query packs are identified.

To combine explicit reports without incorporating stale runs:

```sh
python3 tooling/state-lens/scripts/smoke_report.py build/smoke/protocol/smoke-report.json build/smoke/editor/smoke-report.json --report build/smoke
```

CI uploads JSON and Markdown even on successful runs, runs editor shards under Xvfb, and publishes
a combined step summary. Performance targets exclude builds/downloads: protocol under 15 seconds,
one editor shard under two minutes, and all editor shards under five minutes on the development Mac.


```sh
cargo fmt --manifest-path tooling/state-lens/Cargo.toml --check
cargo clippy --locked --manifest-path tooling/state-lens/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path tooling/state-lens/Cargo.toml
npm ci --prefix extensions/vscode
npm test --prefix extensions/vscode
npm run package --prefix extensions/vscode
python3 tooling/state-lens/scripts/benchmark.py tooling/state-lens/target/release/state-lens
```

Tests exercise binding shadowing, imported aliases, mutation containment, recursive calls, unknown downstream behavior, nested functions, effect removal, malformed syntax, UTF-16/CRLF conversion, actual stdio synchronization, and CLI/server equivalence. Extension tests cover nested-range subtraction, direct-only filtering, and stale-result rejection. [benchmark.json](benchmark.json) records a synthetic baseline, not a large-repository latency guarantee.

Before publishing, manually verify VS Code light/dark/high-contrast themes, split editors, unsaved changes, disable/re-enable, file deletion, and stopping the server. The repository's automated tests do not claim live-editor visual validation.

## Subsequent milestones

1. Expand typed API models and per-category fixtures, and add held-out variants. Introduce bug diagnostics only with separate bug ground truth and evaluation.
2. Add unstructured-state sources and recognized validation boundaries, then shared-state access, synchronization, and lifetime evidence.
3. Distinguish co-location, connected exposure, and supported compound-pattern candidates. Preserve the catalog's `compound-defects` recipes; `common-defects` stays outside the three-color model.
4. Render blue/red intersections with deterministic disjoint segments and textual family labels. Color overlap alone never proves a CWE.
5. The [Zed adapter](../../extensions/zed/README.md) uses hint diagnostics, inlay hints, and hover. Full arbitrary background overlays require a separately validated editor API or upstream work; replacing existing language grammars is not the integration strategy.

Architecture references: [Tree-sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html), [VS Code decoration API](https://code.visualstudio.com/api/references/vscode-api#TextEditor.setDecorations), [Zed language extensions](https://zed.dev/docs/extensions/languages), and [Zed's published extension API](https://docs.rs/zed_extension_api/latest/zed_extension_api/trait.Extension.html).

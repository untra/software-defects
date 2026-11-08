# State Lens

State Lens identifies mechanisms associated with mutated-state defects in Go, Rust, TypeScript, and Python. It produces source ranges, executable-scope summaries, and evidence paths for a VS Code background lens. A highlighted mechanism can be completely intentional and correct.

The engine also analyzes division/remainder denominators in all four supported languages. Known-zero and supported possibly-zero denominators produce Warning diagnostics; unsupported expressions remain Information candidates. Guarded fixed examples suppress these division findings. It preserves the distinction between mechanism coverage, susceptibility, and demonstrated incorrect behavior.

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
| Python | Lexically bound functions, parameters, closures, global/nonlocal assignments | No import resolution, dynamic dispatch, general alias analysis, or runtime type inference |
| Rust | Local functions and simple `use crate::module::function` / `use self::module::function` imports | No grouped/glob use resolution, macro expansion, trait/method dispatch, cfg evaluation, or compiler ownership analysis |

Clock APIs, selected explicitly typed collection mutations, and Go standard-library sorting have models. Unresolved methods with familiar names remain candidates. Arbitrary numeric expressions are not labeled as money calculations. Candidate aliases and initialization mechanisms do not prove a defect or an escaping effect.

Tree-sitter trees update incrementally. Direct facts are cached per document; edits invalidate that document's facts. Workspace graph propagation is recomputed from current direct facts, including effect removals. This is intentionally simpler than maintaining a persistent incremental graph. General alias escape and callback-execution models remain extension points.

Files larger than 2 MiB are skipped; CLI directory scans respect ignore files and omit dependency/build directories. Go-generated-file markers are skipped. The server indexes closed files between protocol messages, prioritizing editor work. Errors and unknown calls remain visible as incomplete analysis.

### Editor protocol

The server supports `initialize`, open/change/close/save synchronization, watched-file updates, configuration changes, hover, inlay hints, shutdown, and exit. Positions use UTF-16; conversion occurs only at the protocol boundary. `stateLens/ranges` accepts `{ "textDocument": { "uri": "file:///..." } }` and returns:

- `schemaVersion`, `generation`, `uri`, `version`, `sourceHash`;
- `findings`, each containing the raw finding plus UTF-16 `triggerRange` and `functionRange`;
- `annotations`, each with `category`, `target` (function/property/segment), UTF-16 `range` and `position`, `label`, `functionRange`, and `findings` indices into the full findings array;
- function/body ranges, `unresolved`, `parseErrors`, and `indexing`.

`stateLens/indexing` notifies the client when background indexing completes. Changes queued during analysis invalidate the response. The client also checks its workspace epoch and current document version before accepting any decoration. Diagnostics are enabled by default and can be disabled with `initializationOptions.diagnostics` (or `settings.stateLens.diagnostics` / `settings.diagnostics` in `workspace/didChangeConfiguration`) set to `false`. When enabled, the server publishes one diagnostic per finding for each open document: Warning for evidence-backed division risks and Information for structural candidates. Each carries `code` = rule ID, a `codeDescription` link to the site page, evidence and call path as `relatedInformation`, `data` = family/origin/effect, and the analyzed document `version`. Publishing happens only when no document change is pending; superseded analyses are discarded and retried, unchanged results are not resent, and documents are cleared on close or when diagnostics are disabled. `textDocument/inlayHint` projects the same annotations within the requested range. Initialization advertises `experimental.stateLens.stateAnnotations: 1`; clients must retain the full findings array so annotation evidence indices remain stable when filtering propagated effects. These remain susceptibility signals, not proven bug diagnoses.

### Annotations

Annotations are computed from findings and supported syntax once per report, then consumed by native LSP hints and VS Code. Mutated-state functions use **Impure**, bindings/properties **Mutacious**, and segments **Volatile**. Shared-state functions use **Async** for explicit async declarations or directly launched Go function literals, otherwise **Shared**; bindings use **Shared**, matched resource regions **Lifecycle**, and goroutine launches **Async**. Unstructured-state uses **Unstructured**, specializing to **Polymorphic** for functions branching on an input's runtime type and **Insecure** for modeled dynamic execution of external input. A function can retain annotations and evidence from multiple categories.

The initial structural models are deliberately bounded:

| Language | External-input sources | Explicit resource pairs |
| --- | --- | --- |
| TypeScript / JavaScript | Unshadowed global `fetch`, its assigned response's `text`/`json`, imported `fs` / `node:fs` `readFileSync` | Imported namespace `openSync` / `closeSync` with the same binding |
| Python | Unshadowed `input`, simple `import requests` get/post and `import json` loads | Built-in `open` / binding's `close`, or `with open(...)` |
| Go | Imported `net/http.Get`/`Post`, `io.ReadAll` | Imported `os.Open`/`Create` and binding's `Close`, including defer |
| Rust | Qualified `reqwest::get`, `std::env::args`, `std::fs::read_to_string` | Qualified `std::fs::File::open` and explicit `drop` of the binding |

JavaScript/TypeScript `typeof` and Python `isinstance` need an if/else branch over an unchanged modeled input binding to establish Polymorphic. Insecure is limited to direct `eval` (or Python `exec`) of Python input/JSON results, fetch response text, or UTF-8 file text. Passing a value through a transformation or reassigning it stops this direct-flow model; that does not prove the transformation validates or sanitizes it. Ordinary division findings remain Unstructured and retain their existing risk colors.

Resource matching rejects intervening rebinding and cross-function/block matches, but does not prove execution order on every path, exception safety, ownership types, or Rust implicit RAII release. General aliases, arbitrary network APIs, cross-function input flow, and validation proofs are not modeled. New structural findings retain taxonomy guide links and source evidence; inherited mutation effects retain their original propagation paths.

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
`STATE_LENS_E2E_JOBS` (default the smaller of the shard count and CPU count). Inventory-derived shards isolate each corpus
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

## Compound rollout status

Audit schema 2 discovers 850 source files, including 168 examples across 44 compound defects.
The nine shards come from `scripts/corpus_inventory.py`; the editor runner and CI use the same
inventory. Compound rows retain class/category and expected/missing rule identities. Base rows
have null compound metadata and empty expectation arrays. Missing ingredients, compound identities,
or linked evidence fail independently of `smoke.json` and `--strict-gaps`. Baseline updates remain
blocked until these assertions pass.

The optional finding `compound` object contains `class`, `category`, and two ingredient references
(`uri`, half-open byte `range`, `rule_id`). It is carried through reports, range snapshots, diagnostics,
and hovers. The extension exposes its decoration categories/ranges through `snapshot()` and `applied()`.
Compound editor smoke activates each document after concurrent provider checks.

This rollout is incomplete: production shared/unstructured ingredient coverage and the 44 compound
relationship predicates are still missing. No compound detection accuracy is claimed. Python parsing,
scopes and initial mutation queries are implemented; Query Lab parser setup includes Python 0.25.0,
but its Python corpus packs and reviewed repair sidecars remain missing. Ingredient-removal and
relationship-breaking analyzer/editor controls still need implementation. The baseline is unchanged.

Validation during this implementation: Rust analyzer/protocol tests and extension unit tests pass;
all compound manifests pass taxonomy/evidence validation. Compound protocol smoke processed 168 files
in 1.89 seconds, with all 168 failing missing-finding contracts. Full protocol smoke processed 850 files
in 3.23 seconds (427 pass, 141 ordinary gaps, 282 failures including 114 newly included Python files).
These runs had no protocol crashes, timeouts, warning/error logs, or invalid ranges. The nondeterminism
editor shard processed all 57 files in about 38 seconds and failed their compound contracts without
extension errors. All nine editor shards then processed 850 files in about 84 seconds, and the combined audit contains exactly 1,700 unique layer/file rows. `make test` fails the unchanged baseline inventory assertion; Query Lab corpus tests
also expose missing Python packs/sidecars. These are rollout failures, not passing acceptance results.

To reproduce the independent compound audit:

```sh
python3 tooling/state-lens/scripts/smoke.py --shard examples/compound-defects --report build/smoke/compound-v2
```

Division findings include optional `division_risk` metadata (confidence, outcome, mutated). Yellow denominator decorations require mutation evidence; blue identifies unstructured risk alone. See [division review](../../examples/unstructured-state/divide-by-zero/REVIEW.md) for supported models and limitations.

### Annotation contract and check exclusions

`Finding.trigger` is the source underline. `Finding.blast_radius` contains explicit
byte spans established by a relationship resolver; an empty list means that the
blast radius is unsupported, not that the entire function is affected. The ranges
endpoint exposes these as UTF-16 `blastRadius` ranges. Owning functions are
navigation/annotation context, not implicit blast radii. State annotation capability
version 2 is required by the bundled extension.

Function/property/segment labels are Impure/Volatile/Mutacious for mutation,
Polymorphic/Untyped/Unstructured for unstructured state, and Async or Shared/Shared/Shared
for shared state. Polymorphic requires runtime type-dispatch evidence. Lifecycle
and unsafe-execution mechanisms remain available in evidence. Presentation merges
same-family annotations per line after visibility filtering, retaining all findings.

Set `stateLens.<language>.disabledChecks` to base defect IDs, for example
`["var-let-mut", "divide-by-zero"]`. Defaults enable all checks. Configuration is
applied before propagation and again after structural analysis; changes invalidate
the report cache. Reusable extraction still runs for other enabled checks.

Run `python3 tooling/state-lens/scripts/annotation_audit.py` from the repository
root to regenerate the implementation inventory. Unknown blast radii are explicitly
unsupported; currently only linked mutated-denominator regions are resolved.

### Shared state graph foundation

`state::extract` produces a snapshot-scoped candidate dependency graph using the
engine's lexical binding resolver. Operations carry URI-qualified byte spans,
binding declaration spans, and guard references. Reads link to reaching writes;
branch joins retain alternatives and definite binding overwrites replace prior
assignments. `candidate_region` intersects forward and backward dependencies.
This intersection does not establish executable reachability.

Direct findings over supported reads now optionally contain `state_trace` with
origins, invalidations, affected uses, requirements, operations, relationships,
and explicit assumptions. The source hash scopes graph IDs to their snapshot.
The existing `origin` field still means direct/propagated. Missing denominator
guards are represented as required-condition text, never synthetic source spans.
The server advertises `stateTraces: 1`, includes traces in diagnostic data, and
presents candidate assignment locations as related information. These candidate
traces do not expand blast radii or change finding severity.

This is an initial foundation, not the completed shared-analysis migration.
Division still uses its existing classifier. Object identities/access paths,
alias propagation, branch polarity, loop fixed points, exceptional control flow,
resource transitions, interprocedural state summaries, and interference predicates
remain unimplemented in the new graph. Function bodies are analyzed separately;
closure creation is not linked to execution. Graph limitations accompany each
trace. Existing compound detectors and editor decoration behavior remain in place.

### Source-only semantic pilot facts

`state-lens semantic-facts PATH` emits versioned JSON with the source URI/hash and
AST-derived events. Go events expose lexical declaration identity, previous visible
bindings, producer calls, and return reads. JavaScript events expose allocation
identity, object binding, property writes, and reads for plain straight-line object
flows. Object spreads preserve nested aliases unless an explicit nested allocation
replaces them; later spreads override earlier properties.

This API supplies Query Lab semantic comparison and generated repair previews.
It does not consume reviewed contracts or issue bug diagnostics on its own.
Unsupported JavaScript behavior is explicit. Go facts model lexical relationships,
not reachability, type checking, or general intent. Production tracing continues
using its existing graph; integrating these pilot facts into that shared graph and
editor affected-use highlights remains follow-up work.

### Production graph export and structured artifacts

Document reports now include `state_graph`, the same graph used to construct their
candidate traces. The task runner converts this into URI/hash-scoped graph artifacts
validated against the schemas in `schemas/`. Trace evidence levels are explicit;
current production graph traces remain `candidate_relationship`.

The editor evidence view exposes origin, invalidation, and affected-use locations.
Trace navigation verifies source hashes. Hover displays required conditions and
assumptions. These additive fields retain protocol schema 1 for compatibility;
full version-2 negotiation and new affected-use decorations are not yet implemented.

Returned branches no longer contribute reaching assignments to subsequent uses.
This is still not general CFG analysis: loops, exception completion, interprocedural
object identity, scheduling feasibility, and compiler type integration remain
unsupported. Semantic pilots still use their separate extractor. Graph export and
artifact validation do not imply that these integration gates have passed.

### Graph contract version 2

`StateGraph` and `StateTrace` share the declarations generated by
`tooling/query-lab/tasks/build_schemas.py`. Run that script with `--check` to
verify generated schemas, Rust additions, and TypeScript interfaces. The shared
fixture in `tests/fixtures/state-contract.json` exercises serialization and
reference validation across Rust and Python.

A snapshot identity is SHA-256 of compact, recursively key-sorted UTF-8 JSON
`[generation, documents]`, with documents sorted by identity. Document records
include URI, source SHA-256, byte length, and nullable editor version. Entity
identities begin with the snapshot identity and include their kind and local
identity. Source ranges are half-open UTF-8 byte ranges; editor presentation
converts these to UTF-16. A changed dependency changes the document manifest,
invalidating identities even when the current document is unchanged.

Artifact envelope version 2 distinguishes file `byte_digest` from canonical
`artifact_id`. Canonical artifact identity excludes only the top-level
`artifact_id`. Historical envelope version 1 remains validated against its own
schemas; it is never promoted to a complete version-2 graph. Task packets retain
their version-1 byte-hash input contract.

Graph artifacts contain complete `graphs` and their `traces`; operations,
bindings, guards, evidence, and snapshot manifests are retained. Validators
check typed references and source bounds. Dependency edges describe candidates;
control-flow edges describe successors separately. Empty control-flow, alias,
and obligation collections do not establish completeness. Current extraction
retains its limitations and does not provide new solver outcomes. Structural
condition text is presentation, never proof input. The protocol advertises
`stateTraces: 2` and `conditionContract: 1`; legacy trace presentation fields
remain available to clients without those capabilities.

Validation on 2026-11-08: the Rust suite, 53 Python tests, 28 extension unit
tests, generated-interface checks (including TypeScript fixture compilation),
formatting, and Clippy passed. The divide-by-zero protocol shard had 20 existing
annotation-count failures and one pass; the TypeScript editor shard had five
existing timeouts. Both sets were reproduced with the pre-change staged analyzer
and matched by file and failure category. No smoke baselines were updated.
On the existing 100-file synthetic benchmark, median scan/warm-edit latency was
104.85/3.97 ms versus 95.88/2.23 ms for the pre-change analyzer. The larger snapshot
and evidence payloads add measurable overhead; workspace snapshot hashing is
performed once per analysis rather than once per document.

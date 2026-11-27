
## Addressing the next five defects

| Priority | Defect | First implementation I would ship | Why it comes next |
|---|---|---|---|
| 1 | var-let-mut | Go inner-scope shadowing where a newly assigned value never reaches a later read of the outer binding. Start with your shadowed port/err example. | You already have binding analysis and 23/23 fixed counterparts. This can turn a broad “mutable declaration” annotation into a precise explanation of which value the program actually reads. |
| 2 | aliasing-mutable-defaults | JS/TS Array.fill(object), shallow copies followed by nested mutation, and mutation of shared default objects. Add Python mutable defaults as a narrowly scoped follow-up. | Aliasing explains how a local-looking write changes somebody else's state. It provides the object-identity analysis needed for meaningful corruption and shared-state findings. |
| 3 | lack-of-input-validation | A known input source reaching a specific operation without the required guard: ignored Go conversion errors, or parsed JS/TS data reaching property access or arithmetic without an established contract. | This supplies the missing connection between input boundaries and failures. It participates in five distinct compound recipes in the current inventory, but each needs its own relationship proof. |
| 4 | index-out-of-bounds | Known-empty collection access, provably invalid constant indexes, and indexing split results without checking their length. | These have concrete failure conditions and useful negative cases. They reuse the guard analysis from priority 3 and extend the existing division work into another precise runtime-error detector. |
| 5 | async-io-misuse | First detect discarded results from known async functions. Then detect shared state read after await when a modeled intervening operation can overwrite it. | This is the next step toward explaining temporal defects. The category participates in four distinct compound recipes, including out-of-order async and stale reads. |

##  implementation tasks, followed by independent acceptance tasks

T01 — Establish one graph and trace contract

Dependencies: None.
Starting points: state.rs (tooling/state-lens/src/state.rs), schema generator (tooling/query-lab/tasks/build_schemas.py), artifact validator (tooling/query-lab/artifacts.py).

Implement a canonical graph representation containing:

- Document identities and per-document hashes.
- Binding, allocation, operation, basic-block, and condition identities.
- Access paths, dependency edges, control-flow edges, and call contexts.
- Required conditions, uncertainty, and source evidence.
- Origin, invalidation status, affected operations, and supporting spans.

Change the artifact exporter to preserve these fields rather than reducing operations to id, kind, and location. Distinguish a file’s byte digest from an artifact’s canonical identity; the current workflow uses both.

Acceptance: Serialization round trips preserve all relationships. Validators reject dangling references, duplicate identities, out-of-bounds spans, mixed snapshots, and unsupported versions. Generate or check Rust/TypeScript interface consistency against the schemas.

T02 — Build ordinary TypeScript/JavaScript control flow

Dependencies: T01.

Replace the recursive terminated approximation with per-function basic blocks and explicit edges for:

- Sequential statements.
- if/else.
- Conditional and short-circuit expressions.
- switch, including fallthrough.
- return.
- Nested functions as separate executable scopes.

Edges must retain branch polarity and the condition governing them. Creating a function must not execute its body.

Acceptance fixtures: Both branches return; one branch returns; unreachable statements; nested functions; &&, ||, and ??; switch fallthrough versus break. Assert exact successor relationships and reachable operations.

T03 — Add loops, abrupt exits, and exception completion

Dependencies: T02.

Implement loop entry, body, update, exit, and backedges for supported loop forms. Handle break/continue targets, including labels. Model throw, catch, and finally using completion kinds rather than a single termination flag.

Unknown potentially throwing calls must have conservative exceptional behavior.

Acceptance fixtures: Zero iterations; loop-carried assignments; nested labeled breaks; continue through an update expression; return through finally; throw caught locally; finally overriding a return or throw. Unsupported constructs must not disappear from the graph.

T04 — Compute reaching state and guard validity

Dependencies: T03.

Implement a worklist fixed-point analysis over the CFG. Track reaching assignments and bounded abstract facts for booleans, nullability, constants, and numeric intervals.

Apply strong updates only to definite targets. Preserve alternatives at joins. Associate guard facts with the values or properties they constrain, and invalidate them when those dependencies change.

Use named, configurable analysis limits. Bound exhaustion must produce unknown results.

Acceptance fixtures: Guarded division; reassignment after a guard; contradictory branches; overwritten unsafe values; loop-carried zero; early-return validation. Run the existing division corpus against the new facts before replacing its separate assignment lookup.

T05 — Unify object and alias analysis

Dependencies: T01, T04.
Starting point: semantic.rs (tooling/state-lens/src/semantic.rs).

Move the useful JavaScript object logic into the production graph. Track object literals, direct aliases, nested properties, shallow copies, spreads, and property overwrites in evaluation order.

Make semantic-facts a projection of the shared graph. Remove duplicate extraction only after parity tests pass.

Acceptance fixtures: Shared db; copied db; still-shared deeper object; later spread restoring an alias; unrelated properties; object rebinding; unknown computed indexes; getters/proxies. The latter cases must retain uncertainty rather than receive ordinary-object semantics.

T06 — Implement cross-function and cross-file state summaries

Dependencies: T04, T05.

Extend existing call resolution and effect propagation with summaries for parameter reads/writes, return aliases, captured state, and resource/object relationships.

Substitute actual arguments at call sites. Preserve call-site context so separate invocations are not merged into one fictitious path. Bound recursive contexts and compute recursive summaries to a fixed point.

Start with the imports and calls already resolved by the engine. Unsupported dispatch remains an explicit boundary.

Acceptance fixtures: Exported function mutates an argument; factory returns an alias; independent factory allocations; two callers with different objects; recursive calls; closure capture; unresolved method. At least one origin → write → affected-use path must span three files.

T07 — Make workspace invalidation and evidence snapshots coherent

Dependencies: T01, T06.

Track dependencies between source documents, summaries, compiler facts, and findings. Invalidate affected artifacts after edits, deletions, import changes, configuration changes, or disabled-check changes.

Give every evidence location its own document identity/hash. Replace the editor’s current assumption that the trace’s single hash applies to all locations.

Acceptance: An unsaved edit in the origin file invalidates a finding shown in another file. Deleting a dependency clears its findings. Reopening or changing a target file rejects stale navigation. No response combines evidence from incompatible generations.

T08 — Add the optional TypeScript compiler adapter

Dependencies: T01. Can proceed alongside T02–T06.

Implement a packaged Node adapter using public TypeScript APIs. Exchange versioned JSON with Rust containing:

- Source/configuration/compiler fingerprints.
- Symbol and declaration references.
- Types at relevant source locations.
- Resolved signatures and module references.
- Diagnostics and incomplete-analysis boundaries.

Supply unsaved editor text through an in-memory compiler host. Support projects without a tsconfig using a documented fallback. Do not download or install dependencies implicitly.

Acceptance: Imported symbol resolution, a shadowed API name, union narrowing, reassignment after narrowing, unsaved edits, config changes, unavailable compiler, and adapter crash. Rust rejects stale responses and continues with explicitly reduced analysis.

T09 — Consume compiler facts in bounded API and guard models

Dependencies: T04, T05, T08.

Use compiler facts to improve recognition of known APIs and guarded types. Keep domain contracts, runtime aliasing, and scheduling assumptions separate from compiler type information.

Promote only specifically supported models. Do not infer that arbitrary user-defined guards are trustworthy sanitizers or that a narrowed property survives an external mutation.

Acceptance: A real modeled API versus a same-named local function; recognized type guard; invalidated property guard; any/unknown; missing declarations. Document exactly which compiler facts change analysis results.

T10 — Model suspension, interference, and compound predicates

Dependencies: T05, T06.

Add explicit suspension and execution-context relationships for supported await and callback/task models. Connect possible interference to a shared identity and a possible writer; an await alone is insufficient.

Implement executable compound predicates for the existing bounded pilots. Recipes must check identity, ordering, compatible conditions, propagation breakers, and the claimed outcome.

Use a synthetic graph fixture for the CWE-680 capacity/access relationship. Keep unsupported language-specific overflow semantics unsupported.

Acceptance: Same dependency with and without a writer; independent copies; revalidation after await; incompatible branch conditions; synchronization that breaks the path. For every compound, remove each ingredient and independently break the connecting relationship.


T11 — Centralize evidence-level promotion

Dependencies: T04, T06, T10.

Implement one evaluator responsible for assigning:

- candidate_relationship
- feasible_under_model
- reproduced

Store predicate results, model limits, assumptions, and an abstract witness. A reproduced finding must reference a validated execution record with source hashes, inputs, environment, and relevant schedule.

Keep exploitability as a separate assessment requiring actor control, reachability, a security boundary, and a security property.

Acceptance: Unknown predicates prevent feasibility promotion; a merely existing witness reference is insufficient; stale reproduction records are rejected; reproducing a runtime error does not automatically establish exploitability.

T12 — Deliver protocol migration and affected-use decorations

Dependencies: T01, T07, T11. UI development can begin against fixtures.
Starting point: extension source (extensions/vscode/src/extension.ts).

Implement schema-v2 capability negotiation and a tested legacy projection. Expose URI-qualified role locations and per-document snapshots.

Add distinct presentation for origin, invalidation, affected operations, and supporting evidence. Keep affected-use spans discontinuous. Missing guards have text and an operation anchor, never a fabricated range.

Acceptance: Two separated affected uses do not fill the intervening lines. Cross-file evidence navigates correctly. A changed dependency clears stale decorations. Test Unicode/CRLF, split editors, direct-only filtering, disabled checks, themes, and unsupported older servers.

T13 — Implement model-provider execution

Dependencies: T01. Independent of analysis implementation.
Starting point: task_runner.py (tooling/query-lab/task_runner.py).

Add a provider-neutral JSON-over-stdio adapter. Requests contain the task packet, permitted input artifacts, output schema, stage instructions, and budgets. Responses contain structured proposals or requests for registered tools.

Keep provider configuration outside model-produced data. Preserve manual submission mode. Provide a fake adapter for deterministic tests; live credentials must not be required by CI.

Acceptance: Successful response; malformed JSON; wrong schema; timeout; excessive output; crash; undeclared artifact access; unregistered tool request; cancellation. A model cannot execute arbitrary commands or mark its own contract human-reviewed.

T14 — Harden task execution, retries, and promotion gates

Dependencies: T01, T13.

Separate task execution status, artifact completeness, hypothesis validity, and rule approval. The current chain’s complete status must not imply a validated rule.

Implement explicit retry policy, failed-result handling, adapter/configuration fingerprints, and downstream invalidation. Avoid hashing the entire corpus for every individual task once declared dependencies can provide equivalent correctness.

Require verified evidence for promotion rather than accepting a model-submitted evaluation: passed.

Acceptance: Changed dependency reruns descendants only; a failed cached tool is not treated as success; changed model submission invalidates consumers; interrupted writes resume safely; approved results cannot be forged by changing a status field.

T15 — Complete paired analysis and executable challenges

Dependencies: T05, T06, T14.

Extract both buggy and fixed graphs; the current generated extraction stage starts with only the original source. Produce structural alignment artifacts retaining ambiguous matches, then compare graph relationships.

Turn challenge descriptions into reviewed executable fixtures or explicitly unsupported cases. Run proposed repairs against semantic postconditions and preserved behavior.

Keep Go shadowing bounded by independent producer/consumer contracts. Legitimate shadowing must remain a negative control.

Acceptance: Both existing pilots use the production graph. Add reassignment, alias, early-return, nested-scope, exception, and scheduling variants. Include a patch that suppresses the symptom but violates the contract; validation must reject it.

T16 — Enforce evaluation split integrity

Dependencies: T01, T14.

Create provenance records for project, scenario family, translations, near-duplicates, and generated descendants. Partition connected provenance groups together. Freeze evaluation inputs and rule versions before held-out execution.

Treat the existing corpus as development data. If independent held-out projects are unavailable, report that gap without inventing an accuracy claim.

Acceptance: Translations cannot straddle splits; generated variants inherit the parent split; renamed near-duplicates are flagged; held-out sources cannot enter hypothesis-generation tasks.

T17 — Independent integration acceptance

Dependencies: T02–T16. Assign this to an agent that did not implement the main analysis.

Build a small integration suite covering these complete paths:

| Scenario | Required result |
|---|---|
| Go shadowing | Contract-backed binding mismatch; valid repair; legitimate-shadowing negative |
| JavaScript nested copy | Shared db path before repair; disconnected path afterward; deeper alias retained |
| Guard then reassignment | Earlier guard invalidated at the write; later operation identified |
| Loop-carried state | Backedge affects the later use; analysis terminates |
| Exception/finally | Correct completion ordering and affected-use reachability |
| Cross-file argument mutation | Origin, write, and consumer connected across files |
| Async interference | Candidate or modeled-feasible path with explicit scheduling assumptions |
| Compiler-assisted recognition | Correct API recognized; shadowed lookalike rejected |
| Editor stale dependency | Decorations and navigation invalidated after another file changes |
| Model task chain | Fake adapter completes validated handoffs; malformed output is rejected |

## Individually delegable implementation tasks

Each numbered task is one LLM work package. Every handoff must contain its dependencies, owned modules, interface contract, positive and negative fixtures, validation commands, and known limitations. Shared integration files have one owner; other agents return integration patches to that owner.

| ID | Assignment and deliverable | Dependencies | Acceptance criteria |
|---|---|---|---|
| T01 | Research and defect specification. Produce a source-linked design note and a matrix mapping existing original/fixed cases to required conditions, invalidating operations, language assumptions, and expected evidence. Inspect actual examples rather than treating taxonomy labels as ground truth. | None | Each selected defect has a reviewed unsafe case, safe control, unsupported case, and behavior contract. Missing fixed counterparts are recorded. |
| T02 | Core contracts and integration ownership. Extend the existing state model with typed predicates, abstract locations, control-flow edges, obligations, analysis outcomes, and evidence references. Own shared model registration and serialization changes throughout the project. | T01 | Deterministic serialization, snapshot-qualified identities, compatibility tests, and no change to existing finding meanings. |
| T03 | TypeScript/JavaScript lowering. Lower supported expressions and statements into the shared representation, including evaluation order, branch polarity, short-circuiting, loops, abrupt exits, and suspension. Share the adapter where grammar behavior permits. | T02 | Golden graph fixtures cover supported syntax; unsupported constructs explicitly prevent unsafe conclusions. |
| T04 | Python lowering. Implement the same contract with Python assignment, branching, exceptions, context-manager boundaries, and suspension semantics. | T02 | Negative indexing and Python arithmetic are not interpreted using JavaScript or Go rules; unsupported dynamic behavior remains unknown. |
| T05 | Go lowering. Implement control flow, assignment ordering, defer registration, calls, and goroutine launch boundaries. | T02 | Deferred cleanup is distinguished from immediate cleanup; goroutine creation is distinguished from execution ordering. |
| T06 | Rust lowering. Implement supported control flow, calls, explicit resource operations, and suspension boundaries. Preserve macro, trait, ownership, and type-resolution limitations. | T02 | No inferred compiler-level ownership guarantees; unsupported expansion or dispatch cannot establish safety. |
| T07 | Forward fact engine. Implement a worklist solver over intervals, zero/nonzero facts, equalities, and finite state predicates. Include branch refinement, joins, loop widening, and dependency-based invalidation. | T02; integrate T03–T06 as available | Loops terminate analytically; joins preserve only justified facts; writes kill affected facts; analysis limits produce explicit unknown results. |
| T08 | Backward requirement engine. Implement substitution, sequential composition, branch-conditioned requirements, and bounded backward slices from operations. Use the forward engine to discharge requirements. | T02, T07 | Mathematical-integer count := count - 1 transforms count != 0 into count != 1; count > 0 does not discharge it. Unsupported expressions and loops cannot silently become proofs. |
| T09 | Division migration. Express existing division/remainder checks as obligations and compare results against the current classifier before replacing it. Preserve established finding IDs and presentation metadata. | T03–T08 | Existing supported cases remain covered; stale guards and branch polarity improve; each new diagnostic states the applicable arithmetic model. |
| T10 | Object identity and alias effects. Track allocation sites, direct aliases, field/access paths, shallow copies, and supported escape behavior. Add explicit independence contracts for reviewed fixtures. | T02, T03–T07 | Nested objects in shallow copies remain shared; immutable scalar snapshots remain independent; alias uncertainty prevents proof of independence. |
| T11 | Bounds obligations. Connect index values and collection lengths to the same abstract object, including invalidation by collection mutation and aliases. | T08, T10 | Detect stale length checks, off-by-one accesses, and loop-bound mutation; accept language-appropriate valid indexing. |
| T12 | Resource state analysis. Add modeled acquire/use/release transitions and cleanup obligations over normal and exceptional exits. Start with the APIs already documented by State Lens. | T03–T08, T10 | Cover use after release, double release, missing cleanup, rebinding, aliases, and supported structured cleanup; unresolved release behavior remains unknown. |
| T13 | Interprocedural state summaries. Extend existing call resolution with required conditions, returned facts, writes, escapes, release effects, and suspension effects. Compute recursive summaries to a fixed point. | T07, T08, T10 | Parameter substitution and caller invalidation work; unknown callees conservatively affect reachable mutable state; recursion terminates without dropping uncertainty. |
| T14 | Async interference analysis. Track ownership/freshness predicates and modeled competing writes across suspension or concurrent execution. Use shared-location identity and execution models to distinguish possible interference from a demonstrated violating schedule. | T08, T10, T13 | Explain check → competing write → use; recognize valid revalidation and independent snapshots; do not treat await or a local reference alias as a repair. |
| T15 | Evidence and editor integration. Render requirements, establishment, invalidation, affected use, assumptions, and analysis status through CLI, LSP, hover, and the evidence view. | T02, T09; extend with T11–T14 | Evidence is source-backed and snapshot-consistent; unknown analysis cannot appear as verified safety; existing stale-response rejection survives edits and configuration changes. |
| T16 | Repair guidance. Provide conditional repair explanations for revalidation, immutable snapshots, request isolation, cleanup, and synchronization. Keep behavior alternatives explicit. | T09, T11–T14 | No automatic choice among returning a default, skipping work, or reporting an error. Each recommendation names the behavior assumption and restored condition. |
| T17 | Independent evaluation and release audit. Maintain separately authored expectations, held-out scenario variants, compound relationship controls, regression measurements, and release reports. | Starts after T01; gates each release | Report findings by language, defect, and outcome; distinguish false positives, misses, and unsupported cases; preserve known baseline failures rather than relabeling them as passes. |

Delegation order: T01 → T02 → language adapters and engine work → T09 → T10–T13 → T14. T15–T17 run alongside the relevant stages after their interfaces stabilize. Parallel agents should use isolated branches and disjoint module ownership; T02’s owner integrates shared changes.

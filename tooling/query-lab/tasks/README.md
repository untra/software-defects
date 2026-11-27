# State Lens task chain

The runner executes registered analysis commands and validates model/manual submissions.
It never executes commands supplied inside model output. Install the pinned dependencies
with `python3 -m pip install -r tooling/query-lab/requirements.txt`.

```sh
python3 tooling/query-lab/task_runner.py plan \
  --case mutated-state/var-let-mut/go/01_shadowed_err_in_if \
  --analyzer tooling/state-lens/target/debug/state-lens > /tmp/tasks.json
python3 tooling/query-lab/task_runner.py chain --packet /tmp/tasks.json \
  --analyzer tooling/state-lens/target/debug/state-lens \
  --output /tmp/state-lens-results --submission /tmp/state-lens-submissions
```

`chain` executes ready registered tools, adds predecessor result hashes to each
packet, and stops at a missing submission or failed tool. It returns the exact
pending packet. Submit `<stage>.json` containing exactly `kind` and `payload`, then
repeat `chain`; unchanged tool results are reused. Model submissions are revalidated.
`run` and `resume` operate on a single packet. `validate --packet FILE` checks a task
or artifact. Outputs are atomic files named by the task packet hash.

The analyzer binary, runner, schema set, validator, Query Lab implementation, and
all declared input files contribute to freshness checks. Re-plan after changing
configuration. Put a source file in the packet inputs before using it as the source.
Case tools require original, fixed, and expectation files as declared inputs. Tool
output and duration limits are explicit in each packet. Behavioral tasks additionally
require `--allow-behavior`; extraction never executes the analyzed source.

## Small-model handoffs

Give the model only the pending packet, its declared input artifacts, the output
schema, and the stage instructions below. A model can return a partial proposal;
it must not fill missing evidence with invented bindings, paths, contracts, or tests.
References must name declared artifacts and their actual content identities. No
provider is configured implicitly: manual submissions are supported today. A live
JSON-over-stdio model adapter and fake-adapter harness remain unimplemented.

| Stage | Inputs and registered tool | Required model work / output |
|---|---|---|
| inventory | Pair sources and manifests; `inventory` | Record authored project/family/translation provenance before creating evaluation splits. Existing corpus is development data. |
| extract | Hashed source; `graph` | None: Rust scan supplies graph facts, converted to validated graph artifacts. |
| align | Both source snapshots and extraction artifacts; submission | Match AST roles; retain alternative matches and unmatched spans. Do not infer identity from equal names. |
| compare | Pair sources/expectations; `semantic_compare` or submission | Identify changed dependencies and independent contracts. Tool support remains limited to the two semantic pilots. |
| hypothesize | Alignment, delta, independent contract; submission | Propose minimal required structure, assumptions, and counterexamples. No correctness claim from the fixed program alone. |
| challenge | Hypothesis and source artifacts; submission | Propose benign lookalikes, reassignment, aliasing, early return, nested scope, exception, and schedule variants. |
| evaluate | Challenge cases and test results; submission | Record checks, provenance groups and outcomes. Generated assertions are proposals until independently validated. |
| repair | Reviewed contract and pair; `semantic_preview` | Generate and check pilot repair; opt into behavioral execution separately. Other patterns require a repair-spec submission. |
| review | All predecessor artifacts; submission | Record unresolved obligations. A submitted evaluation is not automatic production-rule approval. |
| held_out | Independently frozen projects/families and frozen rules; submission | Record external evaluation. Development packets are rejected here; the generated plan intentionally cannot manufacture held-out evidence. |

The generated chain is an executable handoff scaffold, not an autonomous rule
promotion system. Model stages do not yet run counterexample compilation, split
leakage detection, or automatic acceptance gates. Returned `complete` describes
completion of tasks, not proof of the proposed rule. Submitted artifacts remain
`partial`. Corpus names and manifest annotations never enter production detection.

## Evidence and contracts

JSON Schemas in `tooling/state-lens/schemas/` define artifact envelopes and payloads.
Use `artifacts.envelope` to compute canonical artifact IDs and validate output.
Graph validation checks references and source bounds. Finding validation forbids
invented invalidation spans and rejects higher evidence levels without a witness
or with unknown predicates. A witness reference is required evidence metadata,
not itself proof: checking feasibility and reproduction still requires execution.

Contracts need independent tests, authored expectations, or reviewed explanations.
Do not mark model-generated contracts human-reviewed. Chains require a causal
producer-to-consumer connection; composites require compatible joint conditions.
Security assessment additionally requires actor control, reachability, and a
crossed boundary. A runtime failure alone does not establish exploitability.

Do not broaden highlights to enclosing functions. Candidate slices can be
noncontiguous, and dependency connectivity does not prove path feasibility.

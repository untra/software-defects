# Compound Software Defects — Technical Audit

**Source:** `src/content.py` → `DEFECTS[4]` (`"id": "compound-defects"`, lines 3355–3909)
**Scope:** the 18 worked compound-defect examples (the three child classes × six examples each)
**Date:** 2026-06-14
**Status:** the 3 REVISE findings (#3, #15, #18) were applied on 2026-06-14 — see "Remediation applied" at the foot of each and the updated scoreboard.
**CWE titles/relationships spot-checked against:** cwe.mitre.org (CWE-349, CWE-690, CWE-838, CWE-477 fetched live)

---

## What a "compound defect" is here

The `compound-defects` family is modeled not on a CWE *pillar* but on a CWE *view* —
**CWE-678 "Composites"** — and the prose correctly invokes MITRE's two combination
structures:

- **Chain** — one weakness creates the conditions for the next (primary → resultant).
- **Composite** — several weaknesses must all be present at once.

Each of the three child classes is asserted to be the **intersection of two of the three
stateful families**:

| Child class | Intersection | Hallmark symptom |
|---|---|---|
| Nondeterminism | mutated × shared | won't reproduce |
| Runtime Errors | mutated × unstructured | crashes live on unexpected input |
| Data Corruption | shared × unstructured | permanent damage / exposure |

Each example carries:
- `ingredients`: two child-defect slugs (one from each intersecting family);
- `cwes`: a list with `role: "base"` (the root weakness) and `role: "compound"` (the named chain/composite the combination forms);
- a runnable smell→right code sample.

### Audit criteria (each example scored on five axes)

1. **Intersection** — do the two ingredients actually come from the two claimed families?
2. **Ingredient fit** — do the two named child defects plausibly produce this bug?
3. **CWE accuracy** — are the CWE IDs/titles correct per MITRE?
4. **Role logic** — is the base/compound (primary/resultant) split defensible?
5. **Code soundness** — does the sample actually demonstrate the bug, and is the fix correct?

Verdict legend: **PASS** · **PASS (note)** = sound, minor improvement available · **REVISE** = a mapping/label is wrong and should change.

---

## Summary scoreboard

### 1 · Nondeterminism — mutated × shared (CWE-362 / 367 / 349)

| # | Example | Ingredients | base → compound | Verdict |
|---|---|---|---|---|
| 1 | TOCTOU Race (Check-Then-Act) | race-conditions × improper-initialization | CWE-362 → CWE-367 | **PASS (note)** |
| 2 | Lost Update (Read-Modify-Write) | threading-mutexes × increment/decrement | CWE-362 → CWE-567 | **PASS** |
| 3 | Stale-Cache Heisenbug | cache-invalidation × impure-functions | CWE-1108 → CWE-471 | **PASS** *(fixed)* |
| 4 | Out-of-Order Async Completion | var/let/mut × async-io-misuse | CWE-662 → CWE-362, CWE-696 | **PASS (note)** |
| 5 | Concurrent Modification During Iteration | for-loop × resource-contention | CWE-820 → CWE-362, CWE-567 | **PASS** |
| 6 | Predictable ID Collision Under Load | time-money-entropy × race-conditions | CWE-330, CWE-340 → CWE-362 | **PASS** |

### 2 · Runtime Errors — mutated × unstructured (CWE-476 / 843 / 754)

| # | Example | Ingredients | base → compound | Verdict |
|---|---|---|---|---|
| 7 | Null / Undefined Dereference | improper-initialization × missing-undefined-returns | CWE-457 → CWE-476 | **PASS (note)** |
| 8 | Type Confusion / Bad Coercion | this-mutation × type-errors | CWE-704 → CWE-843 | **PASS** |
| 9 | Integration Crash on Unexpected Input | impure-functions × lack-of-input-validation | CWE-20 → CWE-248, CWE-754 | **PASS** |
| 10 | Off-by-One Index Crash | increment/decrement × index-out-of-bounds | CWE-193 → CWE-125, CWE-787 | **PASS** (textbook) |
| 11 | Divide-by-Zero from Empty Aggregate | var/let/mut × divide-by-zero | CWE-682 → CWE-369, CWE-754 | **PASS (note)** |
| 12 | Stale API After Upgrade (AttributeError) | macro-mixin-misuse × version-library-mismanagement | CWE-477 → CWE-758, CWE-754 | **PASS (note)** |

### 3 · Data Corruption — shared × unstructured (CWE-787 / 502 / 345)

| # | Example | Ingredients | base → compound | Verdict |
|---|---|---|---|---|
| 13 | Buffer Overflow (OOB Write) | pointer-mismanagement × index-out-of-bounds | CWE-787 → CWE-119 | **PASS (note)** |
| 14 | Untrusted Deserialization Across a Boundary | file-network-access × insecure-deserialization | CWE-345 → CWE-502 | **PASS** |
| 15 | Schema Drift / Mixed-Version Write | file-network-access × unconstrained-inputs | CWE-20 → CWE-345 | **PASS** *(fixed)* |
| 16 | Stored / Second-Order Injection | file-network-access × lack-of-input-validation | CWE-20 → CWE-89, CWE-79 | **PASS** |
| 17 | Web Cache Poisoning | cache-invalidation × unconstrained-inputs | CWE-348, CWE-444 → CWE-349 | **PASS (note)** |
| 18 | Encoding / Charset Corruption (Mojibake) | file-network-access × version-library-mismanagement | CWE-176 → CWE-172 | **PASS** *(fixed)* |

**Bottom line:** 18/18 are conceptually well-formed; every ingredient pair correctly spans the two
claimed families and every code sample compiles to the bug it advertises. The **3 entries that needed a
CWE or label correction** (#3, #15, #18) **have been fixed** (2026-06-14); **4 carry minor notes** (#7,
#12, #13, #17) worth tightening but not blocking. No example fails outright.

---

## Per-defect findings

### 1. TOCTOU Race — **PASS (note)**
- **Intersection ✓** race-conditions (shared) × improper-initialization (mutated).
- **CWE ✓** CWE-362 "…('Race Condition')" → CWE-367 "TOCTOU" is the canonical specialization; role split (race is the genus, TOCTOU the named form) is correct.
- **Code ✓** `os.path.exists` → `os.makedirs` is the archetypal check-then-act window; `exist_ok=True` is the right atomic fix.
- **Note:** the *mutated* ingredient `improper-initialization` is the loosest of the pairing — the bug is really "creating a resource" rather than "reading before write." It holds (the directory is the resource being initialized), but `improper-initialization` is doing light lifting here compared with the obvious race half.

### 2. Lost Update (Read-Modify-Write) — **PASS**
- **Intersection ✓** threading-mutexes (shared) × increment/decrement (mutated) — the cleanest pairing in the set.
- **CWE ✓** CWE-362 → CWE-567 "Unsynchronized Access to Shared Data in a Multithreaded Context." Both titles exact; 567 is a faithful name for the non-atomic `balance = balance + 1`.
- **Code ✓** Classic three-step race; `with lock:` fix correct. Output caveat ("often < 1000") is honest about nondeterminism.

### 3. Stale-Cache Heisenbug — **PASS** *(was REVISE; fixed 2026-06-14)*
- **Intersection ✓** cache-invalidation (shared) × impure-functions (mutated).
- **Was:** base CWE-471 → compound CWE-349. CWE-349 "Acceptance of Extraneous Untrusted Data With Trusted Data" is about *trust-boundary mixing* (MITRE confirmed) — a stale cache has no untrusted-data ingestion, so it was the wrong named weakness (copied from the `cache-invalidation` child's family list without re-checking the scenario).
- **Remediation applied:** the `cwes` block is now a clean chain matching the code — **base CWE-1108 "Excessive Reliance on Global Variables"** (the impure half: the result depends on the mutable global `TAX_RATE`) → **compound CWE-471 "Modification of Assumed-Immutable Data (MAID)"** (the named weakness the combination forms). CWE-349 dropped. The orphaned CWE-349 in the **Nondeterminism class-header CWE list** was likewise swapped to CWE-471 for consistency. (CWE-349 remains correctly used in #17.)

### 4. Out-of-Order Async Completion — **PASS (note)**
- **Intersection ✓** var/let/mut (mutated) × async-io-misuse (shared).
- **CWE ✓ (mostly)** CWE-696 "Incorrect Behavior Order" is an excellent fit for "last to resolve wins." 
- **Code ✓** The sequence-token fix (`const seq = ++latest`) is the standard stale-response guard and is correct.
- **Note:** the role split labels CWE-662 (Improper Synchronization) as *base* and CWE-362 (Race Condition) as *compound*. In MITRE's hierarchy CWE-362 is the broader/sibling concept and CWE-662 is the class it lives under, so "662 → 362" reads slightly backwards. Functionally fine; if you want strict primary→resultant ordering, make the missing synchronization the base and the observable race the resultant by demoting 662's prominence.

### 5. Concurrent Modification During Iteration — **PASS**
- **Intersection ✓** for-loop-control-flow (mutated) × resource-contention (shared).
- **CWE ✓** CWE-820 "Missing Synchronization" as base, CWE-362/567 as the resultant composite — coherent.
- **Code ✓** `dict changed size during iteration` under a mutating thread is real; `list(sessions.items())` snapshot + locked mutation is the correct fix. Output honestly marks it "(sometimes)."

### 6. Predictable ID Collision Under Load — **PASS**
- **Intersection ✓** time-money-entropy (mutated) × race-conditions (shared).
- **CWE ✓** CWE-330 + CWE-340 (predictable IDs) as base, CWE-362 (race) as the compound that makes the collision *fire* under concurrency. Good layering.
- **Code ✓** `int(time.time()*1000)` collides within a tick; `uuid4().hex` is the correct collision-resistant fix.
- (Minor framing: the deepest harm is *uniqueness/collision* more than *cryptographic predictability*, but CWE-340 covers "Predictable Identifiers," so it stands.)

### 7. Null / Undefined Dereference — **PASS (note)**
- **Intersection ✓** improper-initialization (mutated) × missing-undefined-returns (unstructured).
- **Code ✓** `findUser` falls off the end → `undefined.name` only for unknown ids; the truthy-guard fix is right.
- **Note:** base CWE-457 "Use of Uninitialized Variable" is a loose label for "a function with a missing return path." MITRE publishes the *exact* chain for this scenario: **CWE-690 "Unchecked Return Value to NULL Pointer Dereference"** (CWE-252 → CWE-476, confirmed live). **Recommend** base = CWE-252/CWE-394 (missing/unexpected return) → compound = CWE-476, or cite CWE-690 directly. The current 457→476 still lands on the right resultant, just via a less precise primary.

### 8. Type Confusion / Bad Coercion — **PASS**
- **Intersection ✓** this-mutation (mutated) × type-errors (unstructured).
- **CWE ✓** CWE-704 "Incorrect Type Conversion or Cast" → CWE-843 "Type Confusion" is a recognized chain; titles exact.
- **Code ✓** `this.total` mutated number→string, then `.toFixed` throws in a different method — the "drift and crash live apart" point is well made. (Pedantically the assignment is a *re-type*, not a *cast*, but CWE-704 is the closest standard primary and the resultant 843 is exactly right.)

### 9. Integration Crash on Unexpected Input — **PASS**
- **Intersection ✓** impure-functions/external-read (mutated) × lack-of-input-validation (unstructured).
- **CWE ✓** CWE-20 → CWE-248 (Uncaught Exception) + CWE-754 is a clean primary→resultant chain.
- **Code ✓** `resp["profile"]["age"]` KeyError on a renamed field; "parse, don't trust" with a schema is the correct boundary fix.

### 10. Off-by-One Index Crash — **PASS (textbook)**
- **Intersection ✓** increment/decrement (mutated) × index-out-of-bounds (unstructured).
- **CWE ✓** CWE-193 "Off-by-one" → CWE-125 (OOB Read) / CWE-787 (OOB Write) is *the* canonical MITRE chain; the cross-language note (Python `IndexError` vs C OOB) is accurate and valuable.
- **Code ✓** `while i <= len(rows)` overshoots by one; `for row in rows` removes the hand-rolled index. Strongest entry in the class.

### 11. Divide-by-Zero from Empty Aggregate — **PASS (note)**
- **Intersection ✓** var/let/mut (mutated) × divide-by-zero (unstructured).
- **CWE ✓** CWE-682 → CWE-369 + CWE-754.
- **Note:** CWE-369 is a *child* of CWE-682 in MITRE, so the "682 (base) → 369 (compound)" ordering is parent→child rather than two distinct links. It still reads correctly ("the calculation error specializes into the named divide-by-zero"), but be aware it's a refinement, not a true two-weakness chain. The `var/let/mut` ingredient is also light (the `count` is just `len(values)`); the real mutated half is "an aggregate computed from variable-length input."

### 12. Stale API After Upgrade — **PASS (note)**
- **Intersection ✓** macro-mixin-misuse (mutated) × version-library-mismanagement (unstructured).
- **Code ✓** Mixin calls `self.log_change()` removed in v3 → `AttributeError` only when that path runs post-deploy; `getattr(..., fallback)` is a reasonable seam.
- **Note:** base CWE-477 "Use of Obsolete Function" is a loose fit — MITRE (confirmed live) scopes 477 to *deprecated-but-present* functions, not methods that were **removed/renamed**. CWE-758 "Reliance on … Unspecified Behavior" (the compound) actually carries the scenario better. **Recommend** swapping the base toward CWE-1104 (Use of Unmaintained Components) or CWE-758 as primary, or reframing as a dependency-contract break.

### 13. Buffer Overflow (OOB Write) — **PASS (note)**
- **Intersection ✓** pointer-mismanagement (shared) × index-out-of-bounds (unstructured).
- **Code ✓** `strcpy(name, input)` into `char[16]`; `snprintf(..., sizeof name, ...)` is the correct bounded fix. The "corrupts the return address" output is accurate.
- **Note:** roles are CWE-787 (OOB Write) *base* → CWE-119 (Memory Buffer restriction) *compound*, but CWE-119 is the **parent** of CWE-787, so the arrow points up the tree. The classic MITRE chain here is **CWE-120 "Buffer Copy without Checking Size" (the `strcpy`) → CWE-787**. Consider base = CWE-120 → compound = CWE-787 for a true primary→resultant.

### 14. Untrusted Deserialization Across a Boundary — **PASS**
- **Intersection ✓** file-network-access (shared) × insecure-deserialization (unstructured).
- **CWE ✓** CWE-345 (Insufficient Verification of Data Authenticity) → CWE-502 (Deserialization of Untrusted Data) is a sound chain; titles exact.
- **Code ✓** `pickle.loads(sock.recv(...))` is the textbook RCE-on-load; `json.loads` + `validate` is the correct mitigation.

### 15. Schema Drift / Mixed-Version Write — **PASS** *(was REVISE; fixed 2026-06-14)*
- **CWE ✓** CWE-20 → CWE-345 is fine, and the rolling-deploy cents-vs-dollars example is genuinely good.
- **Was:** the structured `ingredients` said `file-network-access × unconstrained-inputs` but the inline code comment said `(version/library mismanagement x unconstrained inputs)` — a different, wrong pair (both unstructured, which would break the shared×unstructured intersection).
- **Remediation applied:** the code comment is now `# (file/network access x unconstrained inputs)`, matching the (correct) `ingredients`. `version-library-mismanagement` stays out of the ingredients precisely because it is unstructured-family; the shared anchor remains `file-network-access`.

### 16. Stored / Second-Order Injection — **PASS**
- **Intersection ✓** file-network-access (shared, the store) × lack-of-input-validation (unstructured).
- **CWE ✓** CWE-20 → CWE-89 (SQLi) + CWE-79 (XSS) captures both second-order vectors; the "written once, trusted forever" framing is exactly the stored-injection thesis.
- **Code ✓** Parameterized insert + `escape_html` on output ("validate in, encode out") is the correct dual fix.

### 17. Web Cache Poisoning — **PASS (note)**
- **Intersection ✓** cache-invalidation (shared) × unconstrained-inputs (unstructured).
- **CWE ✓** CWE-348 (Less Trusted Source) base + CWE-349 (Extraneous Untrusted Data) compound is a *correct* use of CWE-349 (unlike #3) — an unkeyed `X-Forwarded-Host` is precisely untrusted data riding the trusted cache key.
- **Code ✓** Keying on `(path, vetted host)` is the right fix.
- **Note:** the second base CWE-444 "HTTP Request/Response Smuggling" is tangential to the *shown* example (unkeyed-header poisoning, not desync smuggling). It's defensible as a sibling cache-poisoning vector, but the code only demonstrates the CWE-348/349 path; consider dropping CWE-444 or adding a smuggling sentence so the CWE list matches the sample.

### 18. Encoding / Charset Corruption (Mojibake) — **PASS** *(was REVISE; fixed 2026-06-14)*
- **Intersection ✓** file-network-access (shared) × version-library-mismanagement (unstructured) — and the "platform-default encoding changed across versions" hook is a real, well-known trap.
- **Base CWE ✓** CWE-176 "Improper Handling of Unicode Encoding" fits the round-trip corruption well (kept).
- **Was:** compound CWE-838 "Inappropriate Encoding for Output Context" + CWE-116 "Improper Encoding or Escaping of Output" — both **injection-prevention / output-context** weaknesses (MITRE confirmed: 838 is about HTML-attribute-vs-body encoding to stop XSS). The example is *charset round-trip corruption*, not output-context escaping — no downstream interpreter is tricked.
- **Remediation applied:** the two output-context CWEs were replaced by a single accurate compound — **CWE-172 "Encoding Error"** ("does not properly encode or decode the data, resulting in unexpected values"), the exact mojibake symptom. (CWE-175 "Improper Handling of Mixed Encoding" was considered but, per MITRE, targets mixed encodings *within one input* — a looser fit than this write/read mismatch, so CWE-172 was chosen.)

---

## Cross-cutting observations

1. **CWE titles are accurate.** Every CWE ID/title spot-checked matches MITRE verbatim (a few are intentionally shortened — e.g. CWE-89 → "SQL Injection", CWE-79 → "Cross-site Scripting (XSS)" — which is fine and consistent with the rest of the catalog). All URLs follow the correct `/data/definitions/NNN.html` form.

2. **Intersection discipline is strong.** All 18 ingredient pairs correctly straddle the two claimed families. The one place this slipped is a *comment* (#15), not the structured data.

3. **Two recurring "role" quirks** (#4, #11, #13): occasionally the `compound` CWE is the *parent* of the `base` CWE in MITRE's tree rather than a downstream resultant. The prose meaning ("the named weakness the combination forms") still works, but if you want every entry to be a strict primary→resultant **chain**, pick the narrower weakness as base and the observable named weakness as compound (e.g. #13: CWE-120 → CWE-787).

4. **CWE-349 was used twice with opposite correctness** — wrong in #3 (stale cache, no trust mixing) and right in #17 (cache poisoning, literal trust mixing). The #3 fix removed that inconsistency; CWE-349 now appears only where it belongs.

5. **Edits — status:**
   - ✅ **#15** *(applied)* — inline comment corrected to `file/network access × unconstrained inputs` to match `ingredients`.
   - ✅ **#18** *(applied)* — compound CWE-838/CWE-116 replaced by CWE-172 "Encoding Error"; CWE-176 kept.
   - ✅ **#3** *(applied)* — CWE-349 dropped; now base CWE-1108 → compound CWE-471 (and the Nondeterminism class header swapped CWE-349 → CWE-471).
   - ⬜ **#7** *(open, minor)* — prefer CWE-690 (or CWE-252→CWE-476) over CWE-457 as the primary.
   - ⬜ **#12** *(open, minor)* — move base off CWE-477 toward CWE-1104/CWE-758.
   - ⬜ **#13 / #17** *(open, optional)* — tighten base/compound ordering (#13: CWE-120 → CWE-787) and trim CWE-444 (#17).

**Overall:** the compound-defects catalog is technically sound and pedagogically coherent. Every entry's *thesis* (which two families collide, and the symptom that results) holds, and every code sample demonstrates its bug with a correct fix. The three correctness fixes (#3, #15, #18) are applied and verified via a clean `build.py` run; the remaining items are minor, optional refinements.

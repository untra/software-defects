# Compound Software Defects — Technical Audit

**Source:** `src/content.py` → `DEFECTS[4]` (`"id": "compound-defects"`)
**Scope:** **45** worked compound-defect examples — **15 nondeterminism + 15 runtime-errors + 15 data-corruption** (every one of the 9 categories now holds exactly **5**)
**Date:** 2026-06-15 (category round-out pass; builds on the category-framework / symmetry / 27 / 18-example passes)
**Structure:** each compound smell is organized into **three named failure-vector categories** (a
category tier between smell and example), now **5 worked examples each**. `resource-contention`
coexists with `cross-boundary-state-exposure` (10 shared children). See [[compound-defect-categories]].
**CWE titles/relationships spot-checked live this pass:** CWE-609 (Double-Checked Locking), CWE-758,
CWE-340, CWE-704, CWE-682, CWE-543, CWE-502, CWE-248 (+ all prior-pass IDs).

> **Note on counts:** smells are now even at **15 / 15 / 15** — every category holds exactly **5**.
> `coverage_report()` still tracks per-(family, smell) ingredient coverage; remaining gaps
> (stack-overflow→nondeterminism, resource-leaks→data-corruption, duck-typing→data-corruption) are an
> honest backlog, not a defect — each category is already full.

---

## What a "compound defect" is here

Modeled on the CWE *view* **CWE-678 "Composites"**, invoking MITRE's two combination structures:
**Chain** (one weakness creates the conditions for the next, primary → resultant) and
**Composite** (several must co-occur). Each child class is the intersection of two stateful families:

| Child class | Intersection | Hallmark | Count |
|---|---|---|---|
| Nondeterminism | mutated × shared | won't reproduce | 15 |
| Runtime Errors | mutated × unstructured | crashes live on unexpected input | 15 |
| Data Corruption | shared × unstructured | permanent damage / exposure | 15 |

Each example carries `ingredients` (two child slugs, one per family), a `cwes` list split by
`role` (`base` vs `compound`) with a `from` tag linking each base CWE to its source ingredient,
a per-example `slug` (its own detail page), a `category` membership, and a smell→right code sample.

### The category framework (the failure-vector tier)

Each smell carries a `categories` list (`slug`/`name`/`core`); examples render grouped under their
category's sub-header. Three categories per smell, ≥4 examples each:

| Smell | Category | Core failure | Examples |
|---|---|---|---|
| **Nondeterminism** | Temporal Interleaving | the schedule picks the result | TOCTOU · Lost Update · Out-of-Order Async · Concurrent Modification · **Pool-Exhaustion Heisenbug** 🆕 |
| | Observational Staleness | you act on a value that already moved | Stale-Cache · Use-After-Free · Pooled-Resource · **Lazy-Init Visibility Race** · **Stale Replica Read** 🆕 |
| | Entropic Divergence | a hidden random/identity source diverges | Predictable-ID · Temp-File Race · Concurrent Singleton · **Hash-Seed Serialization Divergence** · **Dispatch-Order Divergence** 🆕 |
| **Runtime Errors** | Structural Invalidation | the data shape is wrong | Null-Deref · Type-Confusion · Shape-Drift · **Aliased-Default Type Confusion** · **Unexpected-Type Element** 🆕 |
| | Computational Exhaustion | the math/scale is broken | Off-by-One · Divide-by-Zero · Quadratic Blowup · NaN-Poison · **Integer Overflow → Buffer** 🆕 |
| | Architectural Drift | the contract is severed | Integration Crash · Stale API · **Deserialized Type Confusion** 🆕 · **Config/Env Drift Crash** · **Schema-Drift on Load** 🆕 |
| **Data Corruption** | Memory Corruption | positioning / structural destruction / boundary smashes | Buffer Overflow · Stack Smash · Type-Confusion Mem · Torn Write · Unchecked Sentinel |
| | Semantic Corruption | logical poisoning / encoding / mismatched representation | NaN Aggregate · Mojibake · Unframed Stream · **Lossy Coercion Ledger** · **Signed/Unsigned Poison** 🆕 |
| | Representational Corruption | session bleed / schema drift across boundaries | Cross-Session · Schema Drift · Stored Injection · Untrusted Deser · Web Cache Poisoning |

The 7 new examples (🆕) each use a **distinct base→compound sub-CWE chain**: CWE-400→362
(pool exhaustion), **CWE-543→609** (double-checked locking — verified live), CWE-758→340 (hash-seed
order), CWE-471→843 (aliased default), CWE-502→843/704 (deserialized shape), CWE-573→248/754 (config
drift), CWE-704→682 (lossy coercion).

### Chain-strength classification (the bar set by [[defect-taxonomy-symmetry-rule]])

Every base→compound pairing is graded against cwe.mitre.org:

- **NAMED** — the compound is a MITRE *named* chain/composite, or a direct parent/child of the base.
- **DOCUMENTED** — a real CanPrecede / CanFollow / PeerOf edge exists between base and compound.
- **DEFENSIBLE** — technically correct and on-theme, but no formal MITRE edge (acceptable; track as backlog).
- **STRETCH** — the base does not actually *cause* the compound. **Not allowed** — must be fixed.

### Verdict legend
**PASS** · **PASS (note)** = sound, minor improvement available · **REVISE** = a mapping/label is wrong.

**Bottom line:** all **39** are conceptually well-formed and organized under the 3×3 category
framework (≥4 per category); structure (`slug`/`from`/`category`/ingredients/roles) is uniform; every
code sample compiles to the bug it advertises. The 7 new anchors each use a distinct sub-CWE chain
(the Lazy-Init Visibility Race lands the named **CWE-609 Double-Checked Locking**). The detailed
per-example scoreboards below predate the category tier but their CWE analysis still holds. Remaining
`coverage_report()` gaps (macro-mixin→nondeterminism, stack-overflow→nondeterminism,
resource-contention/resource-leaks→data-corruption, duck-typing→data-corruption) are an honest
backlog — each category is already full, so they are expansion opportunities, not defects.

---

## Scoreboard 1 · Nondeterminism — mutated × shared  (class CWEs 362/367/471)

| # | Example | Ingredients | base → compound | Strength | Verdict |
|---|---|---|---|---|---|
| 1 | TOCTOU Race (Check-Then-Act) | race-conditions × improper-initialization | 362 → 367 | NAMED | **PASS** |
| 2 | Lost Update (RMW Race) | threading-mutexes × increment/decrement | 362 → 567 | DOCUMENTED | **PASS** |
| 3 | Stale-Cache Heisenbug | cache-invalidation × impure-functions | 1108 → 471 | DEFENSIBLE | **PASS** *(fixed 06-14)* |
| 4 | Out-of-Order Async Completion | var/let/mut × async-io-misuse | 662 → 362, 696 | DOCUMENTED | **PASS (note)** |
| 5 | Concurrent Modification During Iteration | for-loop × threading-mutexes | 820 → 362, 567 | DOCUMENTED | **PASS** |
| 6 | Predictable ID Collision Under Load | time-money-entropy × race-conditions | 330, 340 → 362 | DEFENSIBLE | **PASS** |
| 7 | Use-After-Free / Dangling Read | aliasing-mutable-defaults × pointer-mismanagement | 416 → 825 | NAMED | **PASS** |
| 8 | Concurrent Singleton Mutation | this-mutation × cross-boundary-state-exposure | 567 → 488 | DOCUMENTED | **PASS** (exemplary) |
| 9 | Pooled-Resource Heisenbug | impure-functions × resource-leaks | 1108 → 459 | DEFENSIBLE | **PASS** |
| 10 | Temp-File Check-Then-Use Race 🆕 | file-network-access × time-money-entropy | 367 → 363, 59 | NAMED | **PASS** (exemplary) |

## Scoreboard 2 · Runtime Errors — mutated × unstructured  (class CWEs 476/843/754)

| # | Example | Ingredients | base → compound | Strength | Verdict |
|---|---|---|---|---|---|
| 11 | Null / Undefined Dereference | improper-initialization × missing-undefined-returns | 457, 252 → 476, 690 | NAMED | **PASS** *(refined: +252/690)* |
| 12 | Type Confusion / Bad Coercion | this-mutation × type-errors | 704 → 843 | DOCUMENTED | **PASS** |
| 13 | Integration Crash on Unexpected Input | impure-functions × lack-of-input-validation | 20 → 248, 754 | DEFENSIBLE | **PASS** |
| 14 | Off-by-One Index Crash | increment/decrement × index-out-of-bounds | 193 → 125, 787 | NAMED | **PASS** (textbook) |
| 15 | Divide-by-Zero from Empty Aggregate | var/let/mut × divide-by-zero | 682 → 369, 754 | DEFENSIBLE | **PASS (note)** |
| 16 | Stale API After Upgrade (AttributeError) | macro-mixin-misuse × version-library-mismanagement | 573 → 758, 754 | DEFENSIBLE (weak) | **PASS (note)** 🔧 |
| 17 | Unbounded Input → Quadratic Blowup | for-loop × unconstrained-inputs | 1284 → 400, 407 | DOCUMENTED | **PASS** *(fixed: 789→407)* |
| 18 | Shape-Drift AttributeError | improper-initialization × duck-typing-without-contracts | 908 → 754 | DEFENSIBLE | **PASS** *(fixed: 628→908)* |
| 19 | NaN / Infinity Poisons a Consumer | time-money-entropy × lack-of-input-validation | 369 → 754 | DEFENSIBLE | **PASS** *(fixed: 1339→369)* |

## Scoreboard 3 · Data Corruption — shared × unstructured  (class CWEs 787/502/345)

| # | Example | Ingredients | base → compound | Strength | Verdict |
|---|---|---|---|---|---|
| 20 | Buffer Overflow (OOB Write) | pointer-mismanagement × index-out-of-bounds | 120 → 787 | NAMED | **PASS** *(refined: 787→120/787)* |
| 21 | Untrusted Deserialization | file-network-access × insecure-deserialization | 345 → 502 | DOCUMENTED | **PASS** |
| 22 | Schema Drift / Mixed-Version Write | file-network-access × unconstrained-inputs | 20 → 345 | DEFENSIBLE | **PASS** *(fixed 06-14)* |
| 23 | Stored / Second-Order Injection | file-network-access × lack-of-input-validation | 20 → 89, 79 | DOCUMENTED | **PASS** |
| 24 | Web Cache Poisoning | cache-invalidation × unconstrained-inputs | 348 → 349 | DEFENSIBLE | **PASS** *(refined: −444)* |
| 25 | Encoding / Charset Corruption (Mojibake) | file-network-access × version-library-mismanagement | 176 → 172 | DOCUMENTED | **PASS** *(fixed 06-14)* |
| 26 | Unframed Stream Corruption | async-io-misuse × unconstrained-inputs | 130 → 240 | DEFENSIBLE | **PASS (note)** |
| 27 | Stack Smash via Unbounded Recursion | stack-overflow × unconstrained-inputs | 674 → 787 | DOCUMENTED | **PASS (note)** |
| 28 | NaN Poisons a Shared Aggregate | divide-by-zero × threading-mutexes | 369 → 682 | DEFENSIBLE | **PASS (note)** |
| 29 | Type-Confusion Memory Corruption 🆕 | type-errors × pointer-mismanagement | 843 → 119 | DOCUMENTED | **PASS** (strong) |
| 30 | Torn Write / Lost-Update Corruption 🆕 | race-conditions × lack-of-input-validation | 366, **20** → 787 | DOCUMENTED (composite) | **PASS** *(fixed: +CWE-20)* |
| 31 | Cross-Session Contamination 🆕 | cross-boundary-state-exposure × lack-of-input-validation | **20**, 567 → 488 | DOCUMENTED | **PASS** *(fixed: +CWE-20)* |
| 32 | Unchecked Sentinel Corrupts a Shared Record 🆕 | missing-undefined-returns × file-network-access | 252 → 787 | DOCUMENTED | **PASS** (strong) |

🆕 = new this pass · 🔧 = improvement recommended (details below).

---

## Findings on the 5 new anchors + the 3 flagged

### 10. Temp-File Check-Then-Use Race — **PASS (exemplary)**
Predictable `/tmp/job-<clock>.tmp` name (entropy, from time-money-entropy) + existence-check-then-open
gap (from file-network-access) → a planted symlink redirects the write. **NAMED chain confirmed live:**
CWE-363 "Race Condition Enabling Link Following" is a *child* of CWE-367 (TOCTOU) and *CanPrecede*
CWE-59 (Link Following). `mkstemp()` (O_EXCL + random name) is the correct fix. Best of the new set.

### 29. Type-Confusion Memory Corruption — **PASS (strong)**
A `union { long handle; char *ptr; }` reads an integer id and writes through it as a pointer →
wild write. base CWE-843 (Type Confusion) → compound CWE-119 (memory-buffer restriction) is the
recognized type-confusion→memory-corruption primitive. Clean intersection (type-errors × pointer-mismanagement).

### 32. Unchecked Sentinel Corrupts a Shared Record — **PASS (strong)**
`index_of()` returns `-1` unchecked, used as a `pwrite` offset → writes before the file start. base
CWE-252 (Unchecked Return Value) → compound CWE-787 (OOB Write) is a documented unchecked-return→OOB
chain, and the intent text deliberately mirrors the CWE-690 null-deref chain in #11 — nice symmetry.

### 30. Torn Write / Lost-Update Corruption — **PASS** *(was STRETCH; fixed)*
The code has *two independent* defects: a data race (concurrent `memcpy`, no lock) **and** an
unvalidated `off`/`n` that runs past `buf[256]`. **Verified:** CWE-366 does legitimately cover a
two-thread shared-buffer race (its title is misleading). **But the compound CWE-787 (OOB write) is
produced by the unchecked offset, not by the race** — so base CWE-366 alone → CWE-787 read as "the race
causes the overflow," which is false (the lone STRETCH). **Fix applied:** added base
`CWE-20 (from: lack-of-input-validation)`. Now it is an honest composite — CWE-366 explains the
torn/lost update, CWE-20 → CWE-787 explains the overflow — and the existing prose already names both halves.

### 31. Cross-Session Contamination — **PASS** *(was CWE-twin of #8; fixed)*
A shared `Config` singleton's field is overwritten per-request and read by another request. base
CWE-567 → compound CWE-488 is the documented corruption edge — but it was **CWE-identical to #8**
(Concurrent Singleton Mutation) and its `lack-of-input-validation` ingredient was unrepresented (the fix
validates `locale`, yet no CWE-20 appeared). **Fix applied:** added base `CWE-20 (from: lack-of-input-validation)`,
so it now reads CWE-20 + CWE-567 → CWE-488 — honoring the ingredient and differentiating it from #8.

### 16. Stale API After Upgrade — **PASS (note) 🔧 weak anchor**
base was CWE-477 (wrong — obsolete-but-present), now **CWE-573 "Improper Following of Specification by
Caller."** **Verified:** CWE-573 is *Class*-level and MITRE marks it **ALLOWED-WITH-REVIEW** ("examine
children for a better fit"). It is DEFENSIBLE but the weakest anchor in the catalog — the mixin calls a
method an upgrade *removed*, which no MITRE child (CWE-253/628/475) captures cleanly either. → **Backlog:**
either accept CWE-573 as the honest best-available, or re-base toward the version-mismanagement theme
(e.g. CWE-1104) and let CWE-758 (unspecified behavior) carry the resultant. Leave as-is for now; documented.

---

## Reflection on the stateful-defect CWE sets (variety & accuracy)

Dumped per-child CWE sets are mostly solid and on-theme. Two observations worth acting on:

1. **`this-mutation` and `aliasing-mutable-defaults` carry the *identical* CWE triple {374, 375, 471}.**
   They are CWE-indistinguishable, which is why #8/#12 (this-mutation) and #7 (aliasing) can feel
   CWE-samey. → Differentiate `this-mutation` with a state-mutation-specific CWE (candidate: **CWE-1109**
   "Use of Same Variable for Multiple Purposes" is *not* it; better is leaning on **CWE-471** alone +
   adding a method-receiver/identity weakness). Low priority, but it's the clearest stateful-set redundancy.
2. **`cache-invalidation` has the thinnest set {349, 524}**, and CWE-349 (extraneous untrusted data) is an
   awkward fit for *staleness* (the actual cache-invalidation problem has no precise CWE). Defensible, but
   the thinnest/most-stretched stateful set. The new **`cross-boundary-state-exposure` {488, 543, 567, 1108}**
   is, by contrast, the strongest-curated set added — CWE-543 (singleton-without-sync) is exactly on theme.

Everything else (race-conditions 362/367/366; pointer-mismanagement 476/416/415/787; file-network-access
22/367/73; the unstructured sets) is accurate and varied.

## Variety & novelty across the compound examples

- **Data-corruption leans heavily on memory-safety in C.** Of 13 examples, **#20, #27, #29, #30, #32 all
  resolve to CWE-787/119 and are all C** — ~38% of the class is "unvalidated thing → out-of-bounds write."
  The more *distinctive* data-corruption stories (logical/persistent: schema drift #22, mojibake #25,
  stored injection #23, cache poisoning #24) are under-weighted. → Future data-corruption additions should
  avoid another "→ CWE-787 in C"; mine persistent/logical corruption instead.
- **Runtime-errors leans on CWE-754 as a catch-all compound** (#13, #15, #18, #19 — four of nine end in
  "Improper Check for Exceptional Conditions"). Honest, but slightly monotone; varied compounds (e.g. a
  distinct CWE-617 assertion, CWE-617/CWE-241 type cases) would add color.
- **CWE-567→488 appears twice** (#8, #31) and **CWE-787 appears six times** as a compound. Some reuse is
  structural (cross-boundary-state-exposure must feed two smells), but #31's fix above reduces it.

## Coverage gaps (from `coverage_report()` — 6 remaining, 8/9 each)

| Family → smell | Unused stateful child |
|---|---|
| mutated → nondeterminism | **macro-mixin-misuse** |
| mutated → runtime-errors | **aliasing-mutable-defaults** |
| shared → nondeterminism | **stack-overflow** |
| shared → data-corruption | **resource-leaks** |
| unstructured → runtime-errors | **insecure-deserialization** |
| unstructured → data-corruption | **duck-typing-without-contracts** |

Each is an honest authoring opportunity (per the rule: author a strong-anchored example, never tag a
child exempt). Quick strength reads for the gaps:
- **insecure-deserialization → runtime-errors**: strong — CWE-502 deserialization triggering CWE-843/CWE-704 on a malformed-but-not-malicious payload is a clean NAMED-ish chain.
- **duck-typing-without-contracts → data-corruption**: a wrong-shape object persisted across a boundary (CWE-843/CWE-20 → CWE-345) — defensible.
- **stack-overflow → nondeterminism**: weakest — stack depth is deterministic per input; an *intermittent* stack-overflow needs a concurrency/entropy angle to be honest. Candidate for "no honest example; document as backlog."
- **resource-leaks → data-corruption**, **macro-mixin → nondeterminism**, **aliasing → runtime-errors**: all authorable with DEFENSIBLE+ anchors.

---

## Prioritized improvement backlog

1. ✅ **#30 (STRETCH → composite) — applied:** added `CWE-20 (from: lack-of-input-validation)` so CWE-787 has an honest cause.
2. ✅ **#31 (CWE-twin) — applied:** added `CWE-20 (from: lack-of-input-validation)` to differentiate from #8 and honor the ingredient.
3. ✅ **Report/data hygiene:** smell counts corrected to 10/9/13 (not 9/9/14).
4. ⬜ **Close coverage gaps** with strong anchors first (insecure-deserialization→runtime, duck-typing→data-corruption); document `stack-overflow→nondeterminism` as backlog if no honest chain exists.
5. ⬜ **Stateful-set variety:** differentiate `this-mutation` from `aliasing-mutable-defaults` (identical CWE triple); revisit the thin `cache-invalidation` set.
6. ⬜ **#16 weak anchor:** revisit CWE-573 (class-level, allowed-with-review) when a better version-mismatch base surfaces.
7. ⬜ **Class-level variety:** steer new data-corruption examples away from C/CWE-787; reduce runtime reliance on CWE-754.

---

## Remediation status

- **This pass — applied by the symmetry edits:** `resource-contention` → `cross-boundary-state-exposure`;
  +5 anchors (#10, #29, #30, #31, #32); refinements #11 (+CWE-252/690), #16 (477→573), #20 (787→120/787),
  #24 (−CWE-444); `slug`/`from` backfilled to all 32. Build clean (`cd src && python build.py`).
- **This pass — applied by this audit:** #30 +CWE-20 base (removes the lone STRETCH); #31 +CWE-20 base (differentiates from #8). Rebuilt clean (785 files).
- **Earlier passes (applied):** 06-14 #3/#21/#24-mojibake; 06-15 #17/#18/#19 (789→407, 628→908, 1339→369).

**Overall:** the 32-example catalog is technically rigorous and now structurally uniform. The replacement
of `resource-contention` with `cross-boundary-state-exposure` was the right call (DoS-only → a real
corruption edge). The single stretch (#30) and the CWE-twin (#31) are both fixable with one honest CWE-20
addition each; everything else is PASS, with the strongest new anchors being #10, #29, and #32.

---

## Remediation status — category-framework pass (2026-06-15)

- **Category tier:** each smell child gained a `categories` list (slug/name/core); every example
  tagged with its `category`; `build.py subdefect_page` groups examples under category sub-headers
  (`.excat` / `.excat__title` / `.excat__core`, styled in `style.css`).
- **`resource-contention` re-added** as a shared-state child (CWE-400/770/833) + restored
  `contention()` animation (coexists with `cross-boundary-state-exposure`; 10 shared children). It is
  honestly featured only in nondeterminism (Pool-Exhaustion Heisenbug); its data-corruption story
  stays a documented backlog, *not* a stretched composite.
- **7 new examples** (distinct sub-CWE chains): Pool-Exhaustion Heisenbug (ND/Temporal, 400→362),
  Lazy-Init Visibility Race (ND/Observational, **543→609** Double-Checked Locking — verified live),
  Hash-Seed Serialization Divergence (ND/Entropic, 758→340), Aliased-Default Type Confusion
  (RT/Structural, 471→843), Deserialized Type Confusion (RT/Architectural, 502→843/704), Config/Env
  Drift Crash (RT/Architectural, 573→248/754), Lossy Coercion Poisons a Ledger (DC/Semantic, 704→682).
  Closed the `aliasing→runtime` and `insecure-deser→runtime` coverage gaps.
- **Verified:** clean build (858 files); each smell hub renders its three category sub-headers with
  ≥4 examples (ND 5/4/4, RT 4/4/4, DC 5/4/5); both `resource-contention.json` and
  `cross-boundary-state-exposure.json` animations present; CWE-609 renders.

---

## Remediation status — category round-out pass (2026-06-15)

Rounded all 9 categories to **exactly 5** (39 → **45**; smells 15/15/15). **6 new examples**, each a
distinct MITRE-verified base→compound chain, authored to **balance ingredient load** — leaning on the
under-used children and never touching the over-used five:

| Category | Example | base→compound | strength |
|---|---|---|---|
| Computational Exhaustion | Integer Overflow Sizes a Buffer | **CWE-190 → CWE-680/787** | NAMED (CWE-680 named chain) |
| Observational Staleness | Stale Read From a Lagging Replica | CWE-662 → CWE-821 | DOCUMENTED |
| Entropic Divergence | Dispatch-Order Divergence | CWE-696 → CWE-758 | DOCUMENTED |
| Architectural Drift | Schema-Drift on Load | CWE-502 → CWE-915 | DOCUMENTED |
| Structural Invalidation | Unexpected-Type Element | CWE-1287 → CWE-241 | DEFENSIBLE |
| Semantic Corruption | Signed/Unsigned Poisons Aggregate | CWE-195 → CWE-682 | DOCUMENTED |

- **Ingredient balance (verified):** the over-used five — impure-functions (5), threading-mutexes (5),
  file-network-access (6), lack-of-input-validation (5), unconstrained-inputs (5) — are **unchanged**.
  Promoted under-used: macro-mixin-misuse 1→3, async-io-misuse 2→4, resource-contention 1→2,
  duck-typing 1→2, insecure-deserialization 2→3 (type-errors 4→5 is the one conscious exception — it
  owns the signedness CWEs).
- **Coverage:** this also closed `macro-mixin→nondeterminism` (Dispatch-Order) and
  `resource-contention→data-corruption` (Signedness). Remaining honest backlog: stack-overflow→nondet,
  resource-leaks→data-corruption, duck-typing→data-corruption.
- **Verified:** clean build (912 files); all 9 categories render exactly 5 examples; the 6 new CWE
  chains (680/821/696/915/241/195) render on their pages.

---

## Two-base composition pass — 2026-06-15

**Requirement:** every compound must list **two base CWEs — one drawn from each stateful ingredient —
composing into the compound**, made clear in the display.

**Display:** already satisfied. `build.py:recipe_html()` groups base CWEs by their `from` ingredient and
renders `IngredientA [baseCWE] × IngredientB [baseCWE] → compound`. Once each example carries one base
per ingredient, the two-base composition shows automatically — verified on the rendered pages. No
rendering change was needed.

**Starting state:** only **3/45** examples had two bases (one per ingredient); the other 42 carried a
single base, so one ingredient contributed nothing.

**Applied (now 27/45 honestly two-based):**
- **22 honest second-bases added**, each pulled from the *missing ingredient's own CWE set* (provenance
  guaranteed): e.g. CWE-665←improper-initialization (#1), CWE-471←this-mutation/aliasing (#7/8/17/26/29),
  CWE-129←index-out-of-bounds (#19/28), CWE-667←threading-mutexes (#12, the 543+667→609 DCL composite),
  CWE-330←time-money-entropy (#10), CWE-834←for-loop (#22), CWE-704←type-errors (#25),
  CWE-787←pointer-mismanagement (#40), CWE-400←unconstrained-inputs (#37/38), CWE-913←insecure-deser (#32).
- **#6 Predictable-ID restructured** to one base per ingredient: CWE-330 (entropy) + CWE-366 (race) →
  compound CWE-340/362 (was two bases both from entropy).
- **Stateful-set enrichment:** `impure-functions` **+= CWE-1108** (Excessive Reliance on Global Variables) —
  on-theme for hidden global/external state — which honestly two-bases #18 and #27.
- **Provenance fixes** (base CWEs that weren't members of their `from` ingredient): #21/#27 CWE-573 →
  **CWE-1104** (a real version-library-mismanagement member); #24 CWE-369 → **CWE-682** (a real
  time-money-entropy member).

**Honest backlog — 18 examples that cannot be two-based without a CWE stretch.** Per
[[defect-taxonomy-symmetry-rule]] these are left single-based and documented, *not* forced. Grouped by
the under-powered stateful set that blocks them:

| Blocking stateful set (gap) | Examples |
|---|---|
| `cache-invalidation` {349,524} — **no staleness CWE** | #3 stale-cache, #13 hash-order, #35 web-cache-poisoning |
| `macro-mixin-misuse` {783,682} — **no metaprogramming/dispatch CWE** | #15 dispatch-order, #21 stale-api, #30 schema-drift-on-load |
| `file-network-access` {22,367,73} — **path-only, no shared-store CWE** | #33 schema-drift, #34 stored-injection, #43 unchecked-sentinel |
| `threading-mutexes` {667,833,820} — **no "shared-poison" CWE** (locked, no sync bug) | #39 nan-aggregate, #44 lossy-ledger |
| `increment-decrement` (overflow-only) / `for-loop` / `var-let-mut` / `time-money-entropy` / `resource-contention` / `version-library-mismanagement` — ingredient contributes no distinct weakness to this compound | #2 lost-update, #5 concurrent-mod, #20 divide-by-zero-empty, #11 pool-exhaustion, #45 signedness, #36 mojibake |

**The meaningful improvement this surfaces:** the backlog is not a labeling problem — it shows which
stateful defects' CWE sets are **too narrow to reach the compounds they feed**. The honest fix is to
*enrich those sets with on-theme MITRE weaknesses* (a recognized staleness CWE for cache-invalidation; a
cooperative-method/MRO weakness for macro-mixin-misuse; a persistence-boundary weakness for
file-network-access), then two-base the dependent examples. Until such an honest CWE exists, the entry
stays single-based — rigor over symmetry.

**Verified:** clean build (912 files); two-base extraction = **27/45**; rendered recipe blocks show
`A[cwe] × B[cwe] → compound`; source changes limited to `src/content.py` (+ this report).

# -*- coding: utf-8 -*-
"""
Central content catalog for the Software Defects site.

Authored in Python (so multi-line code blocks stay sane), then dumped to
_data/defects.yml by gen_data.py to preserve the documented YAML-driven
architecture. Content is drawn directly from Samuel Volin's four presentations:
Common Software Defects, Bugs of Mutated State, Bugs of Unstructured State,
and Bugs of Shared State.
"""

# The overarching thesis cascade (Common Software Defects, slides 2-3)
CASCADE = {
    "intro": "A code smell is not the bug. It is the smell of one nearby.",
    "stages": [
        {"label": "Code Smell", "note": "A pattern that hints at a structural problem. Rotting code does not cause smells \u2014 code causes smells."},
        {"label": "Software Defect", "note": "The latent fault the smell was pointing at. It may pass every test and still be there."},
        {"label": "Vulnerability", "note": "What the defect becomes once it ships and meets the real world."},
    ],
    # software defects suggest vulnerabilities (slide 3)
    "exploits": [
        {"defect": "Software that crashes", "becomes": "a Denial-of-Service attack"},
        {"defect": "Edge cases & complexity", "becomes": "an Abuse of Terms"},
        {"defect": "Race conditions & lag", "becomes": "a way to cheat and scam"},
    ],
    "kicker": "This is why we can't have nice things.",
}

# The recurring joke that shows up across three of the four decks.
TWO_HARD_PROBLEMS = [
    "naming things",
    "off-by-one errors",
    "cache invalidation",
]

# Human-readable shortnames for the CWEs that appear on compound-defect recipe
# chips, keyed by id. The full MITRE titles are long and parenthetical; these
# let a chip read "CWE-367 TOCTOU" instead of just the bare id. Authored once
# per id (ids recur across examples). A missing id renders id-only.
CWE_SHORT = {
    "CWE-20": "Input Validation",
    "CWE-59": "Link Following",
    "CWE-79": "XSS (Cross-Site Scripting)",
    "CWE-89": "SQL Injection",
    "CWE-119": "Buffer Bounds",
    "CWE-120": "Unchecked Buffer Copy",
    "CWE-125": "OOB Read",
    "CWE-130": "Length Mismatch",
    "CWE-172": "Encoding Error",
    "CWE-176": "Unicode Handling",
    "CWE-190": "Integer Overflow",
    "CWE-193": "Off-by-One",
    "CWE-195": "Signed/Unsigned Conversion",
    "CWE-240": "Structural Mismatch",
    "CWE-241": "Unexpected Data Type",
    "CWE-248": "Uncaught Exception",
    "CWE-252": "Unchecked Return",
    "CWE-330": "Weak Randomness",
    "CWE-340": "Predictable IDs",
    "CWE-345": "Unverified Authenticity",
    "CWE-348": "Less Trusted Source",
    "CWE-349": "Extraneous Data",
    "CWE-362": "Race Condition",
    "CWE-363": "Link-Following Race",
    "CWE-366": "Thread Race",
    "CWE-367": "TOCTOU (Time-of-Check to Time-of-Use)",
    "CWE-369": "Divide by Zero",
    "CWE-400": "Resource Exhaustion",
    "CWE-407": "Quadratic Complexity",
    "CWE-416": "Use After Free",
    "CWE-444": "Request Smuggling",
    "CWE-457": "Uninit Variable",
    "CWE-459": "Incomplete Cleanup",
    "CWE-471": "MAID (Modification of Immutable Data)",
    "CWE-476": "NULL Deref",
    "CWE-477": "Obsolete Function",
    "CWE-488": "Session Bleed",
    "CWE-502": "Unsafe Deserialization",
    "CWE-543": "Unsynced Singleton",
    "CWE-567": "Unsynchronized Access",
    "CWE-573": "Spec Violation",
    "CWE-609": "Double-Checked Locking",
    "CWE-662": "Improper Sync",
    "CWE-674": "Uncontrolled Recursion",
    "CWE-680": "Integer to Buffer Overflow",
    "CWE-682": "Incorrect Calculation",
    "CWE-690": "Unchecked Return to NULL Deref",
    "CWE-696": "Bad Operation Order",
    "CWE-704": "Bad Cast",
    "CWE-754": "Unchecked Condition",
    "CWE-758": "Undefined Behavior",
    "CWE-787": "OOB Write",
    "CWE-820": "Missing Synchronization",
    "CWE-821": "Incorrect Synchronization",
    "CWE-825": "Expired Pointer",
    "CWE-843": "Type Confusion",
    "CWE-908": "Uninit Resource",
    "CWE-915": "Object Attribute Injection",
    "CWE-1108": "Global Reliance",
    "CWE-1284": "Bad Quantity",
    "CWE-1287": "Improper Type Validation",
}

DEFECTS = []

# ─────────────────────────────────────────────────────────────────────────────
# 1. MUTATED STATE
# ─────────────────────────────────────────────────────────────────────────────
DEFECTS.append({
    "id": "mutated-state",
    "cwe_pillar": {"id": "CWE-664", "name": "Improper Control of a Resource Through its Lifetime", "url": "https://cwe.mitre.org/data/definitions/664.html"},
    "name": "Bugs of Mutated State",
    "short_name": "Mutated",
    "category": "mutated",
    "tagline": "changing the state of a program",
    "color_primary": "78A65A",
    "color_secondary": "93C47D",
    "color_deep": "4F7942",
    "description": "Bugs caused by changing the state of a program in ways that create unpredictability, where information and control flow become untraced.",
    "essence": "Results diverge from expectations when state changes go untracked.",
    "practice": "These nine defects share one root: state that changes where nobody is watching. It shows up downstream — wrong results, flaky tests, output that won't reproduce — long after the mutation that caused it. The tools that make it easy are the everyday ones: a reassignable binding, a hand-rolled loop counter, a method that quietly rewrites its object.",
    "symptoms": [
        "Wrong or unexpected results",
        "Unpredictable function output or operation",
        "Edge cases abounding in code",
        "Test flakiness and complexity",
    ],
    "causes": [
        "var / let / mut declarations",
        "off-by-one errors",
        "for-loop control flow",
        "this mutation",
        "time / money / entropy",
        "impure functions",
        "macro / mixin misuse",
        "++ / -- & integer overflow",
        "improper initialization",
    ],
    "tools_hurt": [
        "let / var / mut",
        "for loops and control flow",
        "classes and this misuse",
        "uninitialized state / defaults",
        "Math.random() & Date.now()",
        "macros & mixin misuse",
        "++ / -- & overflow",
        "binary operators",
    ],
    "philosophies": [
        {"name": "Reactive Manifesto", "url": "https://www.reactivemanifesto.org"},
        {"name": "Why Functional Programming Matters", "url": "https://www.cs.kent.ac.uk/people/staff/dat/miranda/whyfp90.pdf"},
        {"name": "Tidy Tools Manifesto", "url": "https://tidyverse.tidyverse.org/articles/manifesto.html"},
        {"name": "Refactoring Guru: State", "url": "https://refactoring.guru/design-patterns/state"},
    ],
    "pullquote": {
        "text": "Through gradual practice and research, write entirely pure functions in typed languages on sandboxed VMs in a Faraday cage \u2014 only then can you never make a mistake when writing software.",
        "attribution": "the unreachable ideal (said with a wink)",
    },
    "two_hard_note": None,
    # Per-defect children. The mutated-state family page renders as an overview
    # hub linking to these; each child has its own page, text Lottie, and examples.
    "defects": [
        {
            "slug": "var-let-mut",
            "cwes": [
                {"id": "CWE-457", "name": "Use of Uninitialized Variable", "url": "https://cwe.mitre.org/data/definitions/457.html"},
                {"id": "CWE-563", "name": "Assignment to Variable without Use", "url": "https://cwe.mitre.org/data/definitions/563.html"},
            ],
            "name": "var / let / mut Declarations",
            "tagline": "mutable bindings that drift out from under you",
            "essence": "A binding you can reassign is a binding the next reader cannot trust.",
            "description": "Reassignable variables and secretly-shared defaults let state change between reads, so the value you assume is rarely the value you hold.",
            "examples": [
                {
                    "title": "Reassignment You Can't Trust",
                    "intent": "A binding you can reassign can change between the moment a callback captures it and the moment it runs.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: `let` can be rebound after a callback captures it
let config = loadConfig();
schedule(() => save(config));   // captures the BINDING, not the value
config = reloadConfig();        // the callback will now see this one

// RIGHT: a fresh const per value; nothing can rebind it
const config = loadConfig();
schedule(() => save(config));   // this exact config, always""",
                         "explanation": "A closure captures the variable, not a snapshot. Reassigning let later silently changes what the callback sees.",
                         "output": "let:   callback saves the reloaded config\nconst: callback saves the config it was given"},
                        {"name": "Rust", "slug": "rust", "code":
"""// Rust makes mutation opt-in and visible.
let x = 5;
// x = 6;          // compile error: cannot assign twice

let mut y = 5;     // `mut` is a loud, searchable marker
y = 6;             // allowed -- and obvious in review

// Prefer shadowing to transform without mutating:
let total = 5;
let total = total + 1;   // new binding, old one untouched""",
                         "explanation": "Immutability is the default; mut is a keyword you must type and a reviewer can grep for.",
                         "output": "without mut: reassignment refused at compile time\nwith mut:   permitted, but explicit and auditable"},
                    ],
                },
                {
                    "title": "var Hoisting & Scope Leak",
                    "intent": "var ignores block scope and hoists to the top of the function, so a name leaks past where you declared it.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: var is hoisted and function-scoped
function lookup(list) {
  for (var i = 0; i < list.length; i++) {
    if (list[i] === null) break;
  }
  return i;          // <-- i still visible here; leaks the loop counter
}

// RIGHT: let is block-scoped; the name cannot escape
function lookup(list) {
  let found = -1;
  for (let i = 0; i < list.length; i++) {
    if (list[i] === null) { found = i; break; }
  }
  return found;
}""",
                         "explanation": "var hoists to the function top, so i outlives the loop. let keeps the binding inside the block where it belongs.",
                         "output": "var: returns the leaked counter, even after the loop\nlet: i does not exist outside the loop"},
                        {"name": "Python", "slug": "python", "code":
"""# Python has no block scope either: the loop var leaks
for row in rows:
    if row.done:
        break
print(row)        # <-- 'row' is whatever the loop left behind

# RIGHT: capture what you mean explicitly
last = next((r for r in rows if r.done), None)
print(last)""",
                         "explanation": "After a for-loop the loop variable keeps its final value. Relying on that leaked binding hides intent.",
                         "output": "leak:  prints the last-seen row, not necessarily a 'done' one\nright: 'last' is exactly the row you searched for"},
                    ],
                },
            ],
        },
        {
            "slug": "aliasing-mutable-defaults",
            "cwes": [
                {"id": "CWE-374", "name": "Passing Mutable Objects to an Untrusted Method", "url": "https://cwe.mitre.org/data/definitions/374.html"},
                {"id": "CWE-375", "name": "Returning a Mutable Object to an Untrusted Caller", "url": "https://cwe.mitre.org/data/definitions/375.html"},
                {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html"},
            ],
            "name": "Aliasing & Mutable Defaults",
            "tagline": "two names for one value that changes under both",
            "essence": "When two references share one mutable value, a write through either is a surprise to the other.",
            "description": "A default created once and shared across calls, or two variables bound to the same object: mutate through one name and every other name sees it, because they were never separate values.",
            "examples": [
                {
                    "title": "Shared Mutable Defaults",
                    "intent": "A default value created once at definition time is silently shared across every call.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# CLASSIC TRAP: the default list is created once and shared
def append(item, bucket=[]):
    bucket.append(item)
    return bucket

append(1)   # [1]
append(2)   # [1, 2]  <-- not what anyone expects

# RIGHT: sentinel default, fresh value each call
def append(item, bucket=None):
    bucket = [] if bucket is None else bucket
    bucket.append(item)
    return bucket""",
                         "explanation": "Default arguments are evaluated once at definition time. The mutable list leaks state between unrelated calls.",
                         "output": "trap:  append(1)->[1], append(2)->[1, 2]\nright: append(1)->[1], append(2)->[2]"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: one object literal shared as a default by every caller
const SHARED = {};
function tag(item, into = SHARED) {   // default is the SAME object each call
  into[item] = true;
  return into;
}
tag("a"); tag("b");   // { a: true, b: true } -- leaked across calls

// RIGHT: build a fresh container when none is passed
function tag(item, into) {
  into = into ?? {};
  into[item] = true;
  return into;
}""",
                         "explanation": "A shared default object accumulates state between unrelated calls. Default to a sentinel and allocate a fresh value inside.",
                         "output": "shared: { a: true, b: true }\nfresh:  { a: true }, { b: true }"},
                    ],
                },
                {
                    "title": "Reference Aliasing",
                    "intent": "Two names bound to one object are not two values -- a write through one is visible through the other.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: b is not a copy -- it's the same list
a = [1, 2, 3]
b = a
b.append(4)
print(a)        # [1, 2, 3, 4]  <-- a changed too

# RIGHT: copy when you mean a separate value
b = a.copy()    # or list(a) / a[:]
b.append(4)
print(a)        # [1, 2, 3]""",
                         "explanation": "Assignment binds a new name to the same object. Mutating through either name mutates the one shared list.",
                         "output": "alias: a == [1, 2, 3, 4]\ncopy:  a == [1, 2, 3]"},
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: slices share the same backing array
a := []int{1, 2, 3}
b := a[:2]
b = append(b, 99)     // writes into a's backing array
// a is now [1 2 99]

// RIGHT: copy into a fresh slice when you need independence
b := make([]int, 2)
copy(b, a[:2])
b = append(b, 99)     // a is untouched""",
                         "explanation": "A re-slice points at the same backing array, so append can write through the alias until the array has to grow.",
                         "output": "alias: a == [1 2 99]\ncopy:  a == [1 2 3]"},
                    ],
                },
            ],
        },
        {
            "slug": "for-loop-control-flow",
            "cwes": [
                {"id": "CWE-193", "name": "Off-by-one Error", "url": "https://cwe.mitre.org/data/definitions/193.html"},
                {"id": "CWE-834", "name": "Excessive Iteration", "url": "https://cwe.mitre.org/data/definitions/834.html"},
            ],
            "name": "for-Loop Control Flow",
            "tagline": "hand-rolled counters and tangled break / continue",
            "essence": "Manual loop control invites the index and the collection to disagree.",
            "description": "When you drive iteration by hand \u2014 a counter, a captured variable, an in-place edit \u2014 the loop and the data fall out of sync.",
            "examples": [
                {
                    "title": "Closure Captures the Loop Variable",
                    "intent": "Every closure created in the loop shares one binding, so they all see its final value.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: var shares ONE binding across the whole loop
const fns = [];
for (var i = 0; i < 3; i++) {
  fns.push(() => i);
}
fns.map(f => f());   // [3, 3, 3]

// RIGHT: let gives each iteration its own binding
for (let i = 0; i < 3; i++) {
  fns.push(() => i);
}
fns.map(f => f());   // [0, 1, 2]""",
                         "explanation": "var is one binding mutated in place; every closure reads its last value. let is fresh per iteration.",
                         "output": "var: [3, 3, 3]\nlet: [0, 1, 2]"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: lambdas close over the variable, not its value
fns = [lambda: i for i in range(3)]
[f() for f in fns]          # [2, 2, 2]

# RIGHT: bind the current value as a default argument
fns = [lambda i=i: i for i in range(3)]
[f() for f in fns]          # [0, 1, 2]""",
                         "explanation": "Python closures capture the variable by reference. A default argument snapshots the value at definition time.",
                         "output": "late-binding: [2, 2, 2]\nsnapshot:    [0, 1, 2]"},
                    ],
                },
                {
                    "title": "Mutating a Collection While Iterating",
                    "intent": "Removing from the very list you are looping over makes the cursor skip elements.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: deleting while iterating shifts items under the cursor
nums = [1, 2, 2, 3]
for n in nums:
    if n == 2:
        nums.remove(n)      # skips the second 2
# nums -> [1, 2, 3]

# RIGHT: build a new list; never mutate what you iterate
nums = [n for n in nums if n != 2]   # [1, 3]""",
                         "explanation": "remove() shifts every later element down one, so the loop's index jumps past the next match.",
                         "output": "in-place: [1, 2, 3]   (one 2 survives)\nfilter:   [1, 3]"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: structural change during iteration -> exception
for (String s : list) {
    if (s.isBlank()) list.remove(s);   // ConcurrentModificationException
}

// RIGHT: removeIf, or an explicit Iterator.remove()
list.removeIf(String::isBlank);""",
                         "explanation": "The for-each iterator fails fast when the collection changes under it. removeIf mutates safely in one pass.",
                         "output": "for-each: ConcurrentModificationException\nremoveIf: blanks removed cleanly"},
                    ],
                },
            ],
        },
        {
            "slug": "this-mutation",
            "cwes": [
                {"id": "CWE-374", "name": "Passing Mutable Objects to an Untrusted Method", "url": "https://cwe.mitre.org/data/definitions/374.html"},
                {"id": "CWE-375", "name": "Returning a Mutable Object to an Untrusted Caller", "url": "https://cwe.mitre.org/data/definitions/375.html"},
                {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html"},
            ],
            "name": "this Mutation",
            "tagline": "methods that rewrite the object they live on",
            "essence": "When a method mutates this, every caller is coupled to call order.",
            "description": "A method quietly rewrites the object it lives on \u2014 or loses its receiver entirely \u2014 so a later read sees a value nobody assigned on purpose.",
            "examples": [
                {
                    "title": "Mutating this In Place",
                    "intent": "A method quietly rewrites the object it lives on, so a later read sees a value nobody assigned on purpose.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""class Cart {
  constructor() { this.items = []; }

  // SMELL: mutates this and returns nothing useful
  add(item) { this.items.push(item); }
}

// RIGHT: return a new cart, never mutate in place
const add = (cart, item) => ({ items: [...cart.items, item] });""",
                         "explanation": "Mutating this couples every caller to call order. Returning a fresh value makes the operation a pure function.",
                         "output": "mutating: order of add() calls changes shared state\npure:     each call is independent and replayable"},
                        {"name": "Python", "slug": "python", "code":
"""class Counter:
    def __init__(self):
        self.n = 0

    # SMELL: side effect on self, no return
    def bump(self):
        self.n += 1

# RIGHT: a value object that returns a new instance
from dataclasses import dataclass, replace

@dataclass(frozen=True)
class Counter:
    n: int = 0
    def bump(self):
        return replace(self, n=self.n + 1)""",
                         "explanation": "A frozen dataclass cannot be mutated, so bump() must hand back a new value. State changes become traceable.",
                         "output": "mutable: c.bump() changes c in place\nfrozen:  c2 = c.bump(); c is untouched"},
                    ],
                },
                {
                    "title": "Losing this in a Callback",
                    "intent": "Pass a method as a callback and it forgets which object it belonged to.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""class Timer {
  constructor() { this.count = 0; }
  tick() { this.count++; }      // depends on `this`
}
const t = new Timer();

// SMELL: passing the method drops its receiver
setInterval(t.tick, 1000);      // TypeError: this is undefined

// RIGHT: bind the receiver, or wrap in an arrow
setInterval(() => t.tick(), 1000);""",
                         "explanation": "A bare method reference loses its `this`. An arrow wrapper (or .bind(t)) preserves the receiver.",
                         "output": "bare:  this is undefined inside tick()\narrow: t.count increments as expected"},
                        {"name": "Python", "slug": "python", "code":
"""class Timer:
    def __init__(self): self.count = 0
    def tick(self): self.count += 1

t = Timer()

# A *bound* method keeps its self -- this is safe in Python
schedule(t.tick)            # fine: t.tick is bound to t

# SMELL: the *unbound* function needs self passed in
schedule(Timer.tick)        # TypeError: missing 'self'""",
                         "explanation": "Python binds self when you access t.tick. Reaching through the class instead drops the receiver, mirroring the JS trap.",
                         "output": "bound:   t.count increments\nunbound: TypeError, no self supplied"},
                    ],
                },
            ],
        },
        {
            "slug": "time-money-entropy",
            "cwes": [
                {"id": "CWE-330", "name": "Use of Insufficiently Random Values", "url": "https://cwe.mitre.org/data/definitions/330.html"},
                {"id": "CWE-340", "name": "Generation of Predictable Numbers or Identifiers", "url": "https://cwe.mitre.org/data/definitions/340.html"},
                {"id": "CWE-1339", "name": "Insufficient Precision or Accuracy of a Real Number", "url": "https://cwe.mitre.org/data/definitions/1339.html"},
                {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html"},
            ],
            "name": "Time, Money & Entropy",
            "tagline": "reading the clock, the coin, and the dice",
            "essence": "A function that consults wall-clock time or money in floats answers differently every call.",
            "description": "Hidden inputs from the outside world \u2014 the system clock and floating-point money \u2014 make the same call return a different answer each time.",
            "examples": [
                {
                    "title": "Reading the Wall Clock",
                    "intent": "A function that calls the clock can never be replayed; its answer depends on when you ran it.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// IMPURE: result depends on the wall clock
function greeting() {
  return new Date().getHours() < 12 ? "Good morning" : "Good evening";
}

// PURE: inject the hour; same input, same output, every time
const greet = (hour) =>
  hour < 12 ? "Good morning" : "Good evening";""",
                         "explanation": "Pass the clock in as an argument. Now the function is deterministic and trivially testable.",
                         "output": "impure: greeting() flips at noon, untestable\npure:   greet(9) is always 'Good morning'"},
                        {"name": "Python", "slug": "python", "code":
"""from datetime import datetime, timezone

# IMPURE: uses 'now' and the machine's local timezone
def is_expired(token):
    return token.expires_at < datetime.now()   # naive + nondeterministic

# PURE: inject an explicit, timezone-aware 'now'
def is_expired(token, now):
    return token.expires_at < now

is_expired(tok, datetime.now(timezone.utc))""",
                         "explanation": "datetime.now() is both nondeterministic and timezone-naive. Injecting an aware 'now' fixes correctness and testability at once.",
                         "output": "impure: result depends on server clock & locale\npure:   pass a fixed 'now' to test any moment"},
                    ],
                },
                {
                    "title": "Money in Floating Point",
                    "intent": "Binary floats cannot represent most decimals, so currency math drifts by pennies.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: dollars as floats
0.1 + 0.2;                 // 0.30000000000000004
(0.1 + 0.2).toFixed(2);    // "0.30"  -- hides, doesn't fix

// RIGHT: work in integer cents, format only at the edge
const cents = (a, b) => a + b;     // 10 + 20 -> 30
const fmt = (c) => `$${(c / 100).toFixed(2)}`;""",
                         "explanation": "Floats round in binary, so decimals accumulate error. Store money as integer cents and only divide when displaying.",
                         "output": "float: 0.1 + 0.2 = 0.30000000000000004\ncents: 10 + 20 = 30 -> \"$0.30\""},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: float money accrues rounding error
total = 0.1 + 0.2          # 0.30000000000000004

# RIGHT: Decimal keeps base-10 money exact
from decimal import Decimal
total = Decimal("0.1") + Decimal("0.2")   # Decimal('0.3')""",
                         "explanation": "Decimal does base-10 arithmetic, so currency values stay exact. Never let user-facing money live in a binary float.",
                         "output": "float:   0.30000000000000004\nDecimal: 0.3"},
                    ],
                },
            ],
        },
        {
            "slug": "impure-functions",
            "cwes": [
                {"id": "CWE-330", "name": "Use of Insufficiently Random Values", "url": "https://cwe.mitre.org/data/definitions/330.html"},
                {"id": "CWE-338", "name": "Use of Cryptographically Weak Pseudo-Random Number Generator (PRNG)", "url": "https://cwe.mitre.org/data/definitions/338.html"},
                {"id": "CWE-374", "name": "Passing Mutable Objects to an Untrusted Method", "url": "https://cwe.mitre.org/data/definitions/374.html"},
                {"id": "CWE-1108", "name": "Excessive Reliance on Global Variables", "url": "https://cwe.mitre.org/data/definitions/1108.html"},
            ],
            "name": "Impure Functions",
            "tagline": "hidden inputs, hidden effects",
            "essence": "A function whose result depends on more than its arguments cannot be trusted or tested.",
            "description": "Randomness, global reads, and arguments mutated in place make a call's behavior depend on \u2014 and change \u2014 the world around it.",
            "examples": [
                {
                    "title": "Nondeterministic Randomness",
                    "intent": "A function that reaches for a random source gives a different answer on every run.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""import random

# IMPURE: nondeterministic, untestable
def roll():
    return random.randint(1, 6)

# PURE: inject the source of entropy
def roll(rng):
    return rng.randint(1, 6)

roll(random.Random(42))     # repeatable in tests""",
                         "explanation": "Injecting rng makes randomness reproducible: seed it in tests, use a real source in production.",
                         "output": "impure: roll() differs each run\npure:   roll(Random(42)) is always the same"},
                        {"name": "Solidity", "slug": "solidity", "code":
"""// ENTROPY in the worst place: a smart contract.
// SMELL: block values are attacker-influenced "randomness"
function badRandom() public view returns (uint) {
    return uint(keccak256(abi.encode(block.timestamp))) % 100;
}

// RIGHT: use a verifiable random function (Chainlink VRF)
// instead of block.timestamp / blockhash.""",
                         "explanation": "On-chain, time and entropy are public and miner-influenced. Money bugs here are irreversible \u2014 fintech at scale means no backsies.",
                         "output": "bad:   miners can steer block.timestamp to win\nright: VRF gives entropy nobody can predict"},
                    ],
                },
                {
                    "title": "Mutating Your Arguments",
                    "intent": "A function that edits the value passed in changes the caller's world behind its back.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: sort() mutates the array it is called on
function topThree(scores) {
  return scores.sort((a, b) => b - a).slice(0, 3);
}
const data = [3, 1, 2];
topThree(data);
data;          // [3, 2, 1]  <-- caller's array was reordered!

// RIGHT: copy before you transform
const topThree = (s) => [...s].sort((a, b) => b - a).slice(0, 3);""",
                         "explanation": "Array.sort sorts in place and returns the same array. Spread into a fresh array so the caller's data is untouched.",
                         "output": "mutating: data becomes [3, 2, 1]\npure:     data stays [3, 1, 2]"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: the helper mutates the list it was handed
def normalize(items):
    items.sort()
    return items

data = [3, 1, 2]
normalize(data)
data            # [1, 2, 3]  <-- surprise side effect

# RIGHT: return a new, sorted list
def normalize(items):
    return sorted(items)""",
                         "explanation": "list.sort() mutates in place; sorted() returns a new list and leaves the argument alone.",
                         "output": "mutating: data becomes [1, 2, 3]\npure:     data stays [3, 1, 2]"},
                    ],
                },
            ],
        },
        {
            "slug": "macro-mixin-misuse",
            "cwes": [
                {"id": "CWE-783", "name": "Operator Precedence Logic Error", "url": "https://cwe.mitre.org/data/definitions/783.html"},
                {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html"},
            ],
            "name": "Macro & Mixin Misuse",
            "tagline": "code that rewrites code, out of sight",
            "essence": "Metaprogramming expands or mixes behavior the reader never sees.",
            "description": "Textual macros and order-dependent mixins inject behavior at a distance, so what runs is not what the call site appears to say.",
            "examples": [
                {
                    "title": "Unhygienic Macros",
                    "intent": "A textual macro substitutes without parentheses or types, so it expands into something you did not write.",
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: function-like macro with unguarded args */
#define SQUARE(x) x * x
SQUARE(1 + 2)        /* expands to 1 + 2 * 1 + 2 = 5, not 9 */

/* SMELL: argument evaluated twice */
#define MAX(a, b) ((a) > (b) ? (a) : (b))
MAX(i++, j++)        /* increments the winner twice */

/* RIGHT: a real (inline) function -- typed, one evaluation */
static inline int square(int x) { return x * x; }""",
                         "explanation": "Macros are textual substitution with no type checking and no scope. An inline function gives the same speed with none of the traps.",
                         "output": "macro: SQUARE(1+2) = 5  (surprise)\nfunc:  square(1+2) = 9"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: a mixin that mutates the target's state
const Timestamped = (obj) => Object.assign(obj, {
  touched: Date.now(),         // impure + shared mutation
});

// RIGHT: compose explicitly, keep data and behavior apart
const withTimestamp = (obj, now) => ({ ...obj, touched: now });""",
                         "explanation": "Object.assign mutates its target and the impure Date.now() hides inside the mixin. Composition keeps state explicit.",
                         "output": "mixin: original object is mutated; time hidden inside\npure:  caller supplies `now`; original untouched"},
                    ],
                },
                {
                    "title": "Clobbered by Mixin Order",
                    "intent": "Two mixins define the same member, so which one wins depends only on the order they were applied.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""const Serializable = { save() { return "json"; } };
const Auditable    = { save() { return "audit-log"; } };

// SMELL: last mixin silently wins; the other save() vanishes
const model = Object.assign({}, Serializable, Auditable);
model.save();          // "audit-log" -- did you mean to lose json?

// RIGHT: name the behaviors; never let them collide
const model = { saveJson: Serializable.save, audit: Auditable.save };""",
                         "explanation": "Object.assign copies left to right, so a later mixin overwrites an earlier method with no warning.",
                         "output": "mixed:  save() -> 'audit-log' (json lost)\nexplicit: saveJson() and audit() both reachable"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: cooperative methods clobber unless each calls super()
class A:        # mixin
    def setup(self): self.log = ["A"]
class B:        # mixin
    def setup(self): self.log = ["B"]
class W(A, B):  # MRO is A, B -- B.setup never runs
    pass
W().setup()              # log == ['A']

# RIGHT: cooperate via super() so every mixin contributes
class A:
    def setup(self): super().setup(); self.log.append("A")""",
                         "explanation": "Python's MRO picks the first matching method; without super() chaining, later mixins are silently skipped.",
                         "output": "naive: log == ['A']  (B skipped)\nsuper: log == ['B', 'A']  (both run)"},
                    ],
                },
            ],
        },
        {
            "slug": "increment-decrement-operators",
            "cwes": [
                {"id": "CWE-190", "name": "Integer Overflow or Wraparound", "url": "https://cwe.mitre.org/data/definitions/190.html"},
                {"id": "CWE-191", "name": "Integer Underflow (Wrap or Wraparound)", "url": "https://cwe.mitre.org/data/definitions/191.html"},
                {"id": "CWE-680", "name": "Integer Overflow to Buffer Overflow", "url": "https://cwe.mitre.org/data/definitions/680.html"},
                {"id": "CWE-758", "name": "Reliance on Undefined, Unspecified, or Implementation-Defined Behavior", "url": "https://cwe.mitre.org/data/definitions/758.html"},
                {"id": "CWE-193", "name": "Off-by-one Error", "url": "https://cwe.mitre.org/data/definitions/193.html"},
            ],
            "name": "++ / -- & Integer Overflow",
            "tagline": "bump a number past the edge and it wraps",
            "essence": "Pre- versus post-increment hides off-by-one and double-counting; bump a fixed-width integer past its max and it silently wraps to the bottom.",
            "description": "The ++ and -- operators fold a read, a write, and a value into one token \u2014 and the value they hand back is not always the one you expect. Push a bounded integer one step too far and it doesn't overflow upward, it wraps: MAX + 1 becomes MIN, and a wrapped length or index can chain straight into a buffer overflow.",
            "examples": [
                {
                    "title": "Pre- vs Post-Increment",
                    "intent": "i++ yields the old value and ++i the new one, so using the result inline changes what gets stored.",
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""int i = 0;

// SMELL: post-increment returns the OLD value
int a = i++;        // a = 0, i = 1

// the classic self-assignment trap: i never advances
i = i++;            // i is STILL 0 (old value written back)

// RIGHT: ++i returns the new value; or just be explicit
int b = ++i;        // b = 2, i = 2
i += 1;             // unambiguous""",
                         "explanation": "i = i++ evaluates i++ to the old value, bumps i, then overwrites it with the saved old value. The increment is lost.",
                         "output": "i++  -> a = 0, i = 1\ni = i++ -> i = 0  (no change!)\n++i  -> b = 2"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""const arr = [];
let i = 0;

// SMELL: post-increment indexes with the OLD value
arr[i++] = "a";     // writes index 0, then i = 1
arr[i++] = "b";     // writes index 1, then i = 2

// reading inline is where it bites
let n = 0;
console.log(n++ + " then " + n);   // "0 then 1"

// RIGHT: separate the bump from the use
arr.push("a", "b");""",
                         "explanation": "Post-increment is fine when the bump stands alone, but combining it with the value it returns hides an off-by-one.",
                         "output": "arr = ['a', 'b']\nlog: '0 then 1'"},
                    ],
                },
                {
                    "title": "Undefined Evaluation Order",
                    "intent": "Mutating the same variable twice in one expression has no defined order in C \u2014 it is undefined behavior.",
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""int i = 0;
int a[4];

/* UNDEFINED BEHAVIOR: i read & written with no sequence point */
a[i] = i++;          /* which index? the standard does not say */
int x = i++ + ++i;   /* result is not portable -- anything goes */

/* RIGHT: one mutation per statement, ordered explicitly */
a[i] = 0;
i++;""",
                         "explanation": "Between sequence points C lets the compiler order sub-expressions freely. Two updates to i in one expression is UB, not just unclear.",
                         "output": "UB: result varies by compiler / flags\nright: deterministic, one step at a time"},
                        {"name": "Python", "slug": "python", "code":
"""# Python has no ++ on purpose -- the antidote by design
i = 0
# i++            # SyntaxError

i += 1            # one explicit, atomic step

# +1 in a comprehension stays a value, never a hidden write
nums = [n + 1 for n in range(4)]   # [1, 2, 3, 4]""",
                         "explanation": "By omitting ++/--, Python forces the read-modify-write to be a visible statement, sidestepping the whole class of bugs.",
                         "output": "i += 1 -> i = 1\nnums   -> [1, 2, 3, 4]"},
                    ],
                },
                {
                    "title": "Wraparound at the Boundary",
                    "intent": "A fixed-width integer has a ceiling; one bump past it wraps to the floor — and a wrapped size can become a buffer overflow.",
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: unsigned wraps; signed overflow is undefined behavior */
uint8_t x = 255;
x++;                       /* x == 0, not 256 */

/* CWE-680: an overflowed length feeds an allocation, then a copy */
uint16_t len = a + b;      /* a + b > 65535 wraps to something tiny */
char *buf = malloc(len);   /* too small */
memcpy(buf, src, a + b);   /* writes past the buffer */

/* RIGHT: check before you compute the size */
if (a > UINT16_MAX - b) return -1;   /* would overflow */
size_t n = (size_t)a + b;            /* widen, then allocate */""",
                         "explanation": "Wraparound turns a big number into a small one, so the allocation is sized from the wrapped value while the copy uses the real one. Widen the type and range-check before allocating.",
                         "output": "wrap:  len wraps small -> heap overflow on memcpy\nguard: oversize input rejected before malloc"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: int silently overflows -- no exception
int max = Integer.MAX_VALUE;     // 2147483647
int oops = max + 1;              // -2147483648  (wrapped to MIN)

// the classic: a midpoint that goes negative on big arrays
int mid = (low + high) / 2;      // low+high can overflow

// RIGHT: fail loudly, or compute without overflowing
int safe = Math.addExact(max, 1);   // throws ArithmeticException
int m    = low + (high - low) / 2;  // never overflows""",
                         "explanation": "Java's int wraps on overflow instead of throwing, so a bad sum sails on as a negative. Math.addExact makes the overflow an exception; the (high - low) form avoids it entirely.",
                         "output": "wrap:    MAX + 1 -> MIN (no error)\naddExact: throws ArithmeticException"},
                        {"name": "Rust", "slug": "rust", "code":
"""// Rust makes the choice explicit instead of guessing.
let x: u8 = 255;
// let y = x + 1;        // debug: panics; release: wraps to 0

// Spell out which behavior you actually want:
let wrapped = x.wrapping_add(1);     // 0, on purpose
let checked = x.checked_add(1);      // None -> you must handle it
let (v, of) = x.overflowing_add(1);  // (0, true) -- carry flag

match checked {
    Some(n) => println!("{n}"),
    None    => eprintln!("would overflow"),
}""",
                         "explanation": "A bare + can panic in debug and wrap in release, so Rust gives named operations: wrapping_, checked_, and overflowing_ each make the intent visible and reviewable.",
                         "output": "checked_add: None -> handled by match\nwrapping_add: 0 (wrap chosen deliberately)"},
                    ],
                },
            ],
        },
        {
            "slug": "improper-initialization",
            "cwes": [
                {"id": "CWE-665", "name": "Improper Initialization", "url": "https://cwe.mitre.org/data/definitions/665.html"},
                {"id": "CWE-457", "name": "Use of Uninitialized Variable", "url": "https://cwe.mitre.org/data/definitions/457.html"},
                {"id": "CWE-456", "name": "Missing Initialization of a Variable", "url": "https://cwe.mitre.org/data/definitions/456.html"},
                {"id": "CWE-908", "name": "Use of Uninitialized Resource", "url": "https://cwe.mitre.org/data/definitions/908.html"},
                {"id": "CWE-1188", "name": "Insecure Default Initialization of Resource", "url": "https://cwe.mitre.org/data/definitions/1188.html"},
            ],
            "name": "Improper Initialization",
            "tagline": "read before it was ever written",
            "essence": "The first state transition \u2014 undefined to defined \u2014 is the purest mutated-state bug: use a value before it exists and every later read inherits garbage.",
            "description": "A binding read before it is assigned, a field a constructor forgot, a resource left at its default: the undefined \u2192 defined transition is where state begins, and skipping it makes illegal states representable from the very first line.",
            "examples": [
                {
                    "title": "Used Before Assigned",
                    "intent": "A variable or field read before anything wrote to it hands back whatever was already there.",
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""// SMELL: local is never initialized -- reads indeterminate memory
int total() {
    int sum;                 // garbage value
    for (int i = 0; i < 10; i++)
        sum += i;            // adds to garbage; result is nonsense
    return sum;
}

// RIGHT: initialize at the point of declaration
int total() {
    int sum = 0;             // a defined starting state
    for (int i = 0; i < 10; i++)
        sum += i;
    return sum;
}""",
                         "explanation": "An uninitialized automatic variable holds whatever bytes were on the stack. Reading it is undefined behavior; initialize at declaration so the starting state is real.",
                         "output": "uninit: sum += i over garbage -> nonsense\ninit:   sum starts at 0 -> 45"},
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: the zero value of a map is nil, and writing to nil panics
var counts map[string]int          // nil, not empty
counts["a"]++                      // panic: assignment to entry in nil map

// RIGHT: make() gives a real, writable map
counts := make(map[string]int)     // initialized
counts["a"]++                      // 1

// (reading a nil map is fine -- it's the WRITE that needs init)""",
                         "explanation": "Go zero-values everything, but a map's zero value is nil and only readable. You must make() it before writing, or the first assignment panics.",
                         "output": "nil map:  counts[\"a\"]++ -> panic\nmade map: counts[\"a\"]++ -> 1"},
                    ],
                },
                {
                    "title": "Insecure Default Initialization",
                    "intent": "A field left at a permissive default \u2014 or a constructor that skips it \u2014 is a security hole, not just a bug.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""class Session:
    def __init__(self, user):
        self.user = user
        # SMELL: is_admin never set; some paths read it -> AttributeError,
        # or a __getattr__ shim quietly returns a truthy default
    def can_delete(self):
        return self.is_admin          # AttributeError, or worse: True

# RIGHT: initialize every field to a safe, explicit default
class Session:
    def __init__(self, user):
        self.user = user
        self.is_admin = False         # deny by default""",
                         "explanation": "A field the constructor forgets is either a crash or, with a permissive fallback, an open door. Initialize every field \u2014 and make the default the safe one.",
                         "output": "missing: self.is_admin -> AttributeError / unsafe\ndefault: is_admin = False -> deny by default"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: a half-built object escapes before it's fully set up
function makeUser(req) {
  const u = {};
  u.name = req.name;
  save(u);                 // saved with role === undefined
  u.role = "member";       // too late -- the saved copy is broken
  return u;
}

// RIGHT: build the whole value, then let it escape
function makeUser(req) {
  return { name: req.name, role: "member" };   // fully initialized
}""",
                         "explanation": "Letting a partially-initialized object escape (saved, passed, returned) exposes the undefined window. Construct the complete value first, then release it.",
                         "output": "leaked: save() sees role === undefined\nwhole:  object is complete before it escapes"},
                    ],
                },
            ],
        },
    ],
})

# ─────────────────────────────────────────────────────────────────────────────
# 2. UNSTRUCTURED STATE
# ─────────────────────────────────────────────────────────────────────────────
DEFECTS.append({
    "id": "unstructured-state",
    "cwe_pillar": {"id": "CWE-707", "name": "Improper Neutralization", "url": "https://cwe.mitre.org/data/definitions/707.html"},
    "name": "Bugs of Unstructured State",
    "short_name": "Unstructured",
    "category": "unstructured",
    "tagline": "consuming unknown input as state",
    "color_primary": "3D85C6",
    "color_secondary": "6FA8DC",
    "color_deep": "0B5394",
    "description": "Bugs caused by consuming unknown or inadequately validated input as state, without type checking or structural contracts.",
    "essence": "Data without shape, type, or contract; the mismatch cascades downstream.",
    "practice": "Each defect here starts the same way: input consumed as state with no shape or contract to hold it. It shows up as data that doesn't fit its container and bugs that linger no matter how many tests you add. The tools that make it easy are dynamic types, missing validation, and eval/parse that trust whatever they're handed.",
    "symptoms": [
        "Information doesn't fit its expected container",
        "Latent, lingering bugs even with more tests",
        "Poor big-O performance with respect to input size",
        "The problem moves around or persists",
    ],
    "causes": [
        "divide by zero",
        "type errors",
        "index out of bounds",
        "lack of input validation",
        "unconstrained inputs",
        "goto statements",
        "duck typing without contracts",
        "missing / undefined returns",
        "version & library mismanagement",
        "insecure deserialization",
    ],
    "tools_hurt": [
        "dynamic & duck types",
        "lack of validation",
        "garbage in, garbage out",
        "Any typing",
        "nullability without handling",
        "mixed types in collections",
        "unbounded shapes & sizes",
        "eval() or parse()",
    ],
    "philosophies": [
        {"name": "The Art of Data Contract Design", "url": "https://martinfowler.com/articles/consumerDrivenContracts.html"},
        {"name": "Railway Oriented Programming", "url": "https://fsharpforfunandprofit.com/rop/"},
        {"name": "Refactoring Guru: State", "url": "https://refactoring.guru/design-patterns/state"},
        {"name": "Parse, don't validate", "url": "https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/"},
    ],
    "two_hard_note": None,
    # The original unstructured state: control flow you cannot follow. goto is
    # no longer a defect of its own here, but it earns a place in the narrative.
    "pullquote": {
        "tag": "control flow",
        "heading": "goto was the original unstructured state",
        "text": "The go to statement as it stands is just too primitive; it is too much an invitation to make a mess of one's program. The quality of programmers is a decreasing function of the density of go to statements in the code they produce.",
        "attribution": "Edsger W. Dijkstra, “Go To Statement Considered Harmful” (1968)",
    },
    # Input Validation Rules of Thumb (slides 33-35)
    "rules": {
        "title": "Input Validation Rules of Thumb",
        "groups": [
            {"verb": "Reading", "expect": "Read values are correct, up-to-date, and trustworthy \u2014 until you have validated their structure and type, assume none of that is true."},
            {"verb": "Writing", "expect": "Written values are structured correctly, atomic, consistent, idempotent, and durable. Falsey and NULL values are handled safely, not by accident."},
            {"verb": "Executing", "expect": "An executed value is code, and code for the correct interpreter \u2014 and it can be interpreted correctly. (Wait, why are we executing input again?)"},
        ],
    },
    # Knight Capital case study (slide 32)
    "case_study": {
        "title": "Knight Capital Group",
        "meta": "August 1, 2012 \u00b7 45 minutes \u00b7 $440 million",
        "body": "A new deployment landed on old hardware still configured to run repurposed, retired code. A technician had missed one node in the cluster when installing the new trading software. For 45 minutes the cluster sent the firm millions of unintended orders before anyone could pull the deployment.",
        "points": [
            "System miscommunication snowballed into destruction",
            "Reused data structures meant a dead flag now triggered live trades",
            "No input validation on the orders being fired",
            "No architectural discipline to stop a partial deploy",
        ],
        "kicker": "Fintech at scale means no backsies.",
        "link": "https://en.wikipedia.org/wiki/Knight_Capital_Group#2012_stock_trading_disruption",
    },
    # Per-defect children. The unstructured-state family page renders as an
    # overview hub linking to these; each child has its own page, blue text
    # Lottie, and worked examples. Slugs match the animation filenames.
    "defects": [
        {
            "slug": "divide-by-zero",
            "cwes": [
                {"id": "CWE-369", "name": "Divide By Zero", "url": "https://cwe.mitre.org/data/definitions/369.html"},
                {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html"},
            ],
            "name": "Divide by Zero",
            "tagline": "an unconstrained denominator",
            "essence": "Nothing stops the divisor from being zero until the program meets it.",
            "description": "A division is a promise that the denominator is non-zero, and the type system rarely keeps that promise. Integer division throws, floats quietly yield Infinity or NaN, and either way an unvalidated input poisons everything downstream.",
            "examples": [
                {
                    "title": "Ohm's Law, Unguarded",
                    "intent": "Ohm's law rearranged: I = V / R. Nothing stops R from being zero until the program meets it.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# V = I * R  ->  I = V / R
def current(voltage, resistance):
    return voltage / resistance        # boom when resistance == 0

# RIGHT: make the impossible value unrepresentable
def current(voltage, resistance):
    if resistance == 0:
        raise ValueError("resistance must be non-zero")
    return voltage / resistance""",
                         "explanation": "The denominator is unconstrained input. Validate it at the boundary instead of trusting the caller.",
                         "output": "current(5, 0) -> ZeroDivisionError\nguarded     -> ValueError('resistance must be non-zero')"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// JS won't throw -- it returns Infinity or NaN, which spreads
5 / 0;        // Infinity
0 / 0;        // NaN
[Infinity].map(x => x * 2);   // [Infinity], silently wrong

// RIGHT: validate, return a Result-like value
const current = (v, r) =>
  r === 0 ? { ok: false, error: "r is zero" } : { ok: true, value: v / r };""",
                         "explanation": "Worse than a crash: JS yields Infinity/NaN that quietly poison every later calculation.",
                         "output": "5/0 -> Infinity (no error)\ncurrent(5,0) -> { ok: false, error: 'r is zero' }"},
                        {"name": "Rust", "slug": "rust", "code":
"""// Integer divide by zero PANICS; float gives inf/NaN.
// Rust pushes you to encode the failure in the type.
fn current(v: f64, r: f64) -> Result<f64, &'static str> {
    if r == 0.0 { return Err("resistance is zero"); }
    Ok(v / r)
}

match current(5.0, 0.0) {
    Ok(i)  => println!("{i} A"),
    Err(e) => eprintln!("rejected: {e}"),
}""",
                         "explanation": "Result<T, E> makes the caller acknowledge the failure path. The unhappy case cannot be forgotten.",
                         "output": "current(5.0, 0.0) -> Err(\"resistance is zero\")\nhandled by match, never ignored"},
                    ],
                },
                {
                    "title": "Averages Over an Empty Collection",
                    "intent": "Every average divides by a count, and the count is zero exactly when you forgot to check.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""def average(scores):
    return sum(scores) / len(scores)   # ZeroDivisionError on []

# RIGHT: define the empty case before you divide
def average(scores):
    if not scores:
        return None                    # or 0.0, but say so on purpose
    return sum(scores) / len(scores)""",
                         "explanation": "An empty list is a perfectly valid input; the division is what's invalid. Decide what 'the average of nothing' means at the boundary.",
                         "output": "average([]) -> ZeroDivisionError\nguarded   -> None"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""const mean = xs => xs.reduce((a, b) => a + b, 0) / xs.length;
mean([]);            // NaN -- and NaN spreads silently

// RIGHT: handle the empty case, return a Result-like value
const mean = xs =>
  xs.length === 0
    ? { ok: false }
    : { ok: true, value: xs.reduce((a, b) => a + b, 0) / xs.length };""",
                         "explanation": "JS won't throw; it returns NaN, which compares false against everything and quietly corrupts charts, sums, and thresholds.",
                         "output": "mean([]) -> NaN (no error)\nguarded  -> { ok: false }"},
                    ],
                },
            ],
        },
        {
            "slug": "type-errors",
            "cwes": [
                {"id": "CWE-843", "name": "Access of Resource Using Incompatible Type ('Type Confusion')", "url": "https://cwe.mitre.org/data/definitions/843.html"},
                {"id": "CWE-704", "name": "Incorrect Type Conversion or Cast", "url": "https://cwe.mitre.org/data/definitions/704.html"},
                {"id": "CWE-476", "name": "NULL Pointer Dereference", "url": "https://cwe.mitre.org/data/definitions/476.html"},
            ],
            "name": "Type Errors in Dynamic Languages",
            "tagline": "values that shape-shift under you",
            "essence": "A value's type is assumed, never checked, until an operator means something you didn't write.",
            "description": "Dynamic languages let any value flow anywhere; coercion and overloading turn + into concatenation and comparison into surprise. The mismatch hides until a specific input reaches a specific line.",
            "examples": [
                {
                    "title": "Coercion Makes + Ambiguous",
                    "intent": "Values shape-shift through coercion, so a + b means something different than you wrote.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// Coercion makes + ambiguous
1 + "1";        // "11"  (string concat)
"1" - 1;        // 0     (numeric subtraction)
[] + {};        // "[object Object]"
[] == ![];      // true  (the famous one)

// RIGHT: parse at the edge, then trust your types
const n = Number(input);
if (Number.isNaN(n)) throw new Error("not a number");""",
                         "explanation": "+ is overloaded for strings and numbers; the engine guesses. Parse input into a known type once, at the boundary.",
                         "output": "1 + '1' = '11'\n'1' - 1 = 0\nparsed:  Number('1') = 1 (validated)"},
                        {"name": "Python", "slug": "python", "code":
"""# Mixed types compare and concatenate unpredictably
"3" * 3          # '333', not 9
sorted([1, "2", 3])   # TypeError at runtime, deep in a loop

# RIGHT: type hints + a runtime guard at the edge
def area(w: float, h: float) -> float:
    w, h = float(w), float(h)     # coerce once, deliberately
    return w * h""",
                         "explanation": "Duck typing hides the mismatch until a specific input reaches a specific line. Coerce explicitly at the entry point.",
                         "output": "'3' * 3 = '333'\nsorted([1,'2',3]) -> TypeError\narea('2','3') = 6.0 (coerced)"},
                        {"name": "Ruby", "slug": "ruby", "code":
"""# nil quietly threads through everything
user = {name: "Pam"}
user[:age] + 1        # NoMethodError: undefined `+' for nil

# RIGHT: fetch with a default, or fail loudly on purpose
age = user.fetch(:age, 0)
age = user.fetch(:age) { raise "age is required" }""",
                         "explanation": "[] returns nil for a missing key; the error surfaces far from the cause. fetch forces a decision about absence.",
                         "output": "user[:age] + 1 -> NoMethodError on nil\nuser.fetch(:age, 0) + 1 -> 1"},
                        {"name": "Bash", "slug": "bash", "code":
"""# Unquoted, unset variables are the wild west
rm -rf $DIR/         # if DIR is empty: rm -rf /

# RIGHT: fail on unset vars, quote everything
set -euo nounset
: "${DIR:?DIR must be set}"
rm -rf "${DIR:?}/"   """,
                         "explanation": "An empty variable expands to nothing, turning a cleanup into a catastrophe. nounset and quoting are non-negotiable.",
                         "output": "unset DIR: rm -rf / (disaster)\nnounset:   'DIR must be set' -> script aborts"},
                    ],
                },
                {
                    "title": "Truthiness Is Not Absence",
                    "intent": "Falsy values are not the same as missing values, but if (x) treats them identically.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// 0, "", NaN, null, undefined are ALL falsy
function setQuantity(q) {
  if (!q) q = 1;       // BUG: a real order of 0 becomes 1
  return q;
}

// RIGHT: test for absence, not falsiness
function setQuantity(q) {
  return q ?? 1;       // only null / undefined fall through
}""",
                         "explanation": "!q lumps a legitimate 0 in with null. The nullish coalescing operator distinguishes 'absent' from 'zero'.",
                         "output": "setQuantity(0)  -> 1   (wrong)\nq ?? 1 for 0    -> 0   (right)"},
                        {"name": "Python", "slug": "python", "code":
"""def page_size(n):
    return n or 50        # BUG: page_size(0) -> 50

# RIGHT: be explicit about None vs a falsy-but-valid value
def page_size(n):
    return 50 if n is None else n""",
                         "explanation": "n or 50 fires for 0, [], and '' too. 'is None' asks the question you actually meant.",
                         "output": "page_size(0) -> 50  (wrong)\nis None form -> 0  (right)"},
                    ],
                },
            ],
        },
        {
            "slug": "index-out-of-bounds",
            "cwes": [
                {"id": "CWE-125", "name": "Out-of-bounds Read", "url": "https://cwe.mitre.org/data/definitions/125.html"},
                {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html"},
                {"id": "CWE-129", "name": "Improper Validation of Array Index", "url": "https://cwe.mitre.org/data/definitions/129.html"},
            ],
            "name": "Index Out of Bounds & Missing Keys",
            "tagline": "reaching for something that isn't there",
            "essence": "An index or key is assumed present; the structure never promised it.",
            "description": "Arrays, dictionaries, and parsed payloads get reached into positionally, on faith. The slot is empty, the key absent, the array shorter than assumed, and the access fails far from the bad assumption.",
            "examples": [
                {
                    "title": "The Missing Key",
                    "intent": "Reaching into a structure for something that isn't there \u2014 a key, an array slot, a JSON field.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""ages = {'Jim': 30, 'Pam': 28, 'Kevin': 33}

# WRONG: assumes the key exists
ages['Michael']                 # KeyError: 'Michael'

# RIGHT: ask for it safely, or validate the shape first
ages.get('Michael', None)       # None
if 'Michael' in ages:
    use(ages['Michael'])""",
                         "explanation": "A literal lookup assumes a shape the data never promised. .get() turns a crash into a value you can handle.",
                         "output": "ages['Michael'] -> KeyError\nages.get('Michael') -> None"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// A real one: trusting the shape of an SNS event
const msg = JSON.parse(event.Records[0].Sns.Message);
// throws if Records is empty, or Sns is missing, or...

// RIGHT: optional chaining + validate before parse
const raw = event?.Records?.[0]?.Sns?.Message;
if (typeof raw !== "string") throw new Error("bad event shape");
const msg = JSON.parse(raw);""",
                         "explanation": "Each dot and index is an assumption about a payload you didn't build. Optional chaining plus a type check makes the contract explicit.",
                         "output": "unchecked: TypeError: Cannot read '0' of undefined\nchecked:   throws 'bad event shape' with context"},
                        {"name": "Ruby", "slug": "ruby", "code":
"""# Brittle parsing chained on positional assumptions
valid = href_attrs.length == 2 &&
        href_attrs[0] == 'href' &&
        href_attrs[1].start_with?('"')   # NoMethodError if [1] is nil

# RIGHT: guard each assumption, or parse with a real library
valid = href_attrs.length == 2 &&
        href_attrs[0] == 'href' &&
        href_attrs[1].to_s.start_with?('"')""",
                         "explanation": "Index 1 is assumed present and assumed a String. .to_s makes the type safe; a real HTML parser makes the whole thing safe.",
                         "output": "raw:      NoMethodError on nil[1]\nto_s:     false (no crash)"},
                    ],
                },
                {
                    "title": "The Empty-Collection Edge",
                    "intent": "first(), last(), and [0] all assume the collection has a first element.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""def latest(events):
    return events[-1]              # IndexError on []

# RIGHT: handle empty, or use a safe accessor
def latest(events):
    return events[-1] if events else None""",
                         "explanation": "Negative indexing doesn't wrap to safety; [-1] on an empty list throws just like [0] does.",
                         "output": "latest([]) -> IndexError\nguarded    -> None"},
                        {"name": "Java", "slug": "java", "code":
"""// WRONG: assumes the stream produced something
String first = names.stream().findFirst().get();   // NoSuchElementException

// RIGHT: Optional forces you to handle the empty case
String first = names.stream().findFirst().orElse("(none)");""",
                         "explanation": "Optional.get() on an empty Optional throws. orElse / orElseThrow make the empty path a decision, not an accident.",
                         "output": "get() on empty   -> NoSuchElementException\norElse fallback  -> \"(none)\""},
                    ],
                },
            ],
        },
        {
            "slug": "lack-of-input-validation",
            "cwes": [
                {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html"},
                {"id": "CWE-89", "name": "Improper Neutralization of Special Elements used in an SQL Command ('SQL Injection')", "url": "https://cwe.mitre.org/data/definitions/89.html"},
                {"id": "CWE-1284", "name": "Improper Validation of Specified Quantity in Input", "url": "https://cwe.mitre.org/data/definitions/1284.html"},
            ],
            "name": "Lack of Input Validation",
            "tagline": "trusting data you did not create",
            "essence": "Read values are assumed correct, written values assumed safe, executed values assumed benign \u2014 until proven otherwise.",
            "description": "The Input Validation Rules of Thumb in one defect: reading expects trustworthy data, writing expects well-formed data, executing expects code for the right interpreter. Skip the check at the boundary and the whole program inherits the lie.",
            "examples": [
                {
                    "title": "Executing Unvalidated Input",
                    "intent": "An executed value is code, and unvalidated input becomes code for the wrong interpreter.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# WRONG: user input concatenated straight into SQL
cur.execute("SELECT * FROM users WHERE name = '" + name + "'")
# name = "x'; DROP TABLE users; --"  ->  catastrophe

# RIGHT: parameterize -- the driver keeps data as data
cur.execute("SELECT * FROM users WHERE name = %s", (name,))""",
                         "explanation": "String-building SQL lets input cross from data into code. Parameters keep the interpreter from ever seeing input as syntax.",
                         "output": "concatenated:  input can rewrite the query\nparameterized: input is always a bound value"},
                        {"name": "Bash", "slug": "bash", "code":
"""# WRONG: eval on anything a user can influence
mode=$(cat /tmp/mode)
eval "run_$mode"                 # mode file is attacker-controlled

# RIGHT: validate against an allow-list, never eval input
case "$(cat /tmp/mode)" in
  full|incremental) run_backup "$mode" ;;
  *) echo "unknown mode" >&2; exit 1 ;;
esac""",
                         "explanation": "eval turns text into commands. An allow-list of known-good values is the only safe way to branch on outside input.",
                         "output": "eval: arbitrary command execution\ncase: only 'full' or 'incremental' accepted"},
                    ],
                },
                {
                    "title": "Reading an Untrusted Payload",
                    "intent": "Read values are assumed correct and up-to-date; assume nothing until you have validated the shape.",
                    "languages": [
                        {"name": "TypeScript", "slug": "typescript", "code":
"""// WRONG: cast the JSON and pray
const user = JSON.parse(body) as User;   // a cast checks nothing
sendEmail(user.email.toLowerCase());     // crashes if email is missing

// RIGHT: parse, don't validate -- a schema returns a typed value
const user = UserSchema.parse(JSON.parse(body));  // throws on bad shape
sendEmail(user.email.toLowerCase());""",
                         "explanation": "'as User' is a compile-time fiction; at runtime the data can be anything. A schema (zod, io-ts) verifies the shape and hands back a value you can trust.",
                         "output": "cast:   TypeError: Cannot read 'toLowerCase' of undefined\nschema: throws 'email: Required' at the boundary"},
                        {"name": "Python", "slug": "python", "code":
"""# WRONG: trust the inbound dict's keys and types
def handle(payload):
    return charge(payload["amount"] * 100)   # KeyError / TypeError roulette

# RIGHT: validate into a typed model at the edge (pydantic)
class Order(BaseModel):
    amount: condecimal(gt=0)

def handle(payload):
    order = Order(**payload)        # raises ValidationError on bad input
    return charge(order.amount * 100)""",
                         "explanation": "Validating into a model turns a scattering of runtime errors into one clear failure at the door, with the type guaranteed afterward.",
                         "output": "raw:       KeyError('amount') deep in charge()\nvalidated: ValidationError('amount: must be > 0')"},
                    ],
                },
            ],
        },
        {
            "slug": "unconstrained-inputs",
            "cwes": [
                {"id": "CWE-1333", "name": "Inefficient Regular Expression Complexity", "url": "https://cwe.mitre.org/data/definitions/1333.html"},
                {"id": "CWE-400", "name": "Uncontrolled Resource Consumption", "url": "https://cwe.mitre.org/data/definitions/400.html"},
                {"id": "CWE-190", "name": "Integer Overflow or Wraparound", "url": "https://cwe.mitre.org/data/definitions/190.html"},
            ],
            "name": "Unconstrained Inputs",
            "tagline": "input with no bounds on size or range",
            "essence": "Nothing limits how big, how long, or how extreme an input can be \u2014 so something downstream pays for it.",
            "description": "Validation that checks type but not magnitude. An input with no upper bound on length, size, or value becomes unbounded memory, catastrophic backtracking, integer overflow, or a denial of service. Constrain the range, not just the shape.",
            "examples": [
                {
                    "title": "No Bound on Size",
                    "intent": "An input with no maximum length is a denial-of-service waiting for a big enough request.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// WRONG: a catastrophic-backtracking regex on unbounded input
const re = /^(a+)+$/;
re.test("aaaaaaaaaaaaaaaaaaaaaaaa!");   // hangs the event loop (ReDoS)

// RIGHT: bound the input first, then use a linear matcher
if (input.length > 256) throw new Error("too long");
const re = /^a+$/;""",
                         "explanation": "Nested quantifiers make match time exponential in input length. Capping length (and avoiding the pattern) keeps one request from freezing the server.",
                         "output": "unbounded: one request pins the CPU at 100%\nbounded:   rejected at 256 chars, linear match"},
                        {"name": "Python", "slug": "python", "code":
"""# WRONG: read the whole upload into memory
data = request.body.read()           # a 10GB POST is now 10GB of RAM

# RIGHT: enforce a limit before you ever allocate
MAX = 5 * 1024 * 1024
data = request.body.read(MAX + 1)
if len(data) > MAX:
    raise ValueError("payload too large")""",
                         "explanation": "An unconstrained read trusts the client to be reasonable. A hard cap turns a memory-exhaustion attack into a clean rejection.",
                         "output": "unbounded: OOM kill under a large upload\nbounded:   'payload too large' at 5MB"},
                    ],
                },
                {
                    "title": "No Bound on Range",
                    "intent": "A number that type-checks can still be negative, zero, or astronomically large.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# WRONG: limit is an int, so it must be fine... right?
def paginate(items, limit):
    return items[:limit]          # limit=-1 silently drops the last row
                                  # limit=10**9 tries to slice everything

# RIGHT: constrain the range at the boundary
def paginate(items, limit):
    limit = max(1, min(int(limit), 100))
    return items[:limit]""",
                         "explanation": "Type checking proved limit is an integer; it never proved limit is sane. Clamp to a valid range before use.",
                         "output": "limit=-1:  drops a row, no error\nclamped:   limit forced into 1..100"},
                        {"name": "Go", "slug": "go", "code":
"""// WRONG: trust a count from the wire, then allocate it
n := req.Count                 // int from an untrusted client
buf := make([]byte, n)         // n negative -> panic; n huge -> OOM

// RIGHT: validate the range before allocating
if n < 0 || n > maxCount {
    return fmt.Errorf("count out of range: %d", n)
}
buf := make([]byte, n)""",
                         "explanation": "make() with a hostile size panics or exhausts memory. A range check is the difference between a rejected request and a crashed process.",
                         "output": "unchecked: panic: makeslice: len out of range\nchecked:   error('count out of range: -1')"},
                    ],
                },
            ],
        },
        {
            "slug": "duck-typing-without-contracts",
            "cwes": [
                {"id": "CWE-843", "name": "Access of Resource Using Incompatible Type ('Type Confusion')", "url": "https://cwe.mitre.org/data/definitions/843.html"},
                {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html"},
            ],
            "name": "Duck Typing Without Contracts",
            "tagline": "if it quacks \u2014 until it doesn't",
            "essence": "A value is used as if it has a shape no one ever guaranteed it has.",
            "description": "Duck typing is convenient until the duck is missing a method. Without an explicit contract \u2014 a protocol, an interface, an abstract base \u2014 'it probably has .save()' is a runtime gamble that pays off until the one caller that passes something almost-compatible.",
            "examples": [
                {
                    "title": "The Duck That Couldn't",
                    "intent": "If it quacks like a duck, we assume it flies like one, and then it doesn't.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# Works for files, breaks for the StringIO someone passes later
def archive(f):
    f.flush()
    f.fileno()        # AttributeError: StringIO has no fileno()

# The bug only appears for inputs that are ALMOST file-like.""",
                         "explanation": "Duck typing accepts anything until it reaches the one method the substitute lacks. The contract was implicit, so nothing flagged the mismatch.",
                         "output": "real file: ok\nStringIO:  AttributeError: 'fileno'"},
                        {"name": "Ruby", "slug": "ruby", "code":
"""# reduce is assumed, so a non-enumerable blows up mid-iteration
def total(collection)
  collection.reduce(0) { |sum, x| sum + x }   # NoMethodError if no #reduce
end

total(42)   # Integer has no #reduce -> crash far from the caller""",
                         "explanation": "Nothing declared that collection must be enumerable. The assumption surfaces as a NoMethodError deep inside the method.",
                         "output": "total([1,2,3]) -> 6\ntotal(42)      -> NoMethodError: reduce"},
                    ],
                },
                {
                    "title": "Give the Duck a Contract",
                    "intent": "An explicit protocol turns 'probably has the method' into a checkable guarantee.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""from typing import Protocol

class Saveable(Protocol):
    def save(self) -> None: ...

def persist(item: Saveable) -> None:
    item.save()            # type checker verifies item has .save()

# A type that lacks save() is now a mypy error, before runtime.""",
                         "explanation": "typing.Protocol keeps duck typing's flexibility but makes the expected shape explicit and statically checkable.",
                         "output": "with .save():    type-checks, runs\nwithout .save(): mypy error before runtime"},
                        {"name": "Go", "slug": "go", "code":
"""// An interface is a contract the compiler enforces structurally.
type Saver interface { Save() error }

func persist(s Saver) error {
    return s.Save()        // anything passed MUST implement Save()
}

// Passing a type without Save() simply won't compile.""",
                         "explanation": "Go's interfaces are duck typing checked at compile time: the shape is named, and conformance is verified before the program runs.",
                         "output": "implements Saver: compiles\nmissing Save():   compile error"},
                    ],
                },
            ],
        },
        {
            "slug": "missing-undefined-returns",
            "cwes": [
                {"id": "CWE-252", "name": "Unchecked Return Value", "url": "https://cwe.mitre.org/data/definitions/252.html"},
                {"id": "CWE-394", "name": "Unexpected Status Code or Return Value", "url": "https://cwe.mitre.org/data/definitions/394.html"},
                {"id": "CWE-476", "name": "NULL Pointer Dereference", "url": "https://cwe.mitre.org/data/definitions/476.html"},
            ],
            "name": "Missing & Undefined Returns",
            "tagline": "functions that quietly return nothing",
            "essence": "A code path falls off the end, and 'nothing' silently becomes a value.",
            "description": "Some path through the function never hits a return, or returns null / undefined / None, and the caller treats that absence as data. Strongly typed languages drag this from a downstream mystery to a compile-time error.",
            "examples": [
                {
                    "title": "The Same Bug, Solved by Types",
                    "intent": "Strongly typed languages move the unstructured-state failure from runtime to compile time, where it's cheap.",
                    "languages": [
                        {"name": "TypeScript", "slug": "typescript", "code":
"""// strictNullChecks turns a runtime crash into a red squiggle
interface User { name: string; age?: number; }

function birthday(u: User): number {
  return u.age + 1;   // compile error: 'u.age' is possibly undefined
}

// RIGHT: the compiler forces you to handle absence
function birthday(u: User): number {
  return (u.age ?? 0) + 1;
}""",
                         "explanation": "The optional field can't be used until you account for undefined. The bug never reaches production.",
                         "output": "without guard: TS2532 at compile time\nwith ??:       compiles, handles missing age"},
                        {"name": "Go", "slug": "go", "code":
"""// Errors are values you must thread through, not exceptions
func current(v, r float64) (float64, error) {
    if r == 0 {
        return 0, fmt.Errorf("resistance is zero")
    }
    return v / r, nil
}

i, err := current(5, 0)
if err != nil { return err }   // the compiler nags if you ignore it""",
                         "explanation": "Returning (value, error) makes the failure path part of the signature. Linters flag an ignored err.",
                         "output": "current(5, 0) -> 0, error('resistance is zero')\nerr is checked before i is used"},
                        {"name": "Kotlin", "slug": "kotlin", "code":
"""// Nullability is in the type system: String vs String?
fun greet(name: String) = "Hi, $name"

val maybe: String? = lookup()
greet(maybe)          // compile error: String? where String expected

// RIGHT: the safe call / elvis operator handle the null
greet(maybe ?: "stranger")""",
                         "explanation": "String and String? are different types. You cannot pass a nullable where a non-null is required without handling it.",
                         "output": "greet(maybe):       compile error\ngreet(maybe ?: ...): 'Hi, stranger'"},
                        {"name": "Rust", "slug": "rust", "code":
"""// Absence is Option<T>; you cannot forget to handle None.
fn first_word(s: &str) -> Option<&str> {
    s.split_whitespace().next()
}

match first_word("") {
    Some(w) => println!("{w}"),
    None    => println!("(empty input)"),   // exhaustive
}""",
                         "explanation": "Option<T> makes 'nothing' a value the type system tracks. The match must cover None or it won't compile.",
                         "output": "first_word(\"\") -> None\nmatch is exhaustive: '(empty input)'"},
                    ],
                },
                {
                    "title": "The Function That Forgets to Return",
                    "intent": "A branch with no return doesn't error; it hands back undefined / None and the caller runs with it.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// BUG: the else path has no return -> undefined
function discount(user) {
  if (user.isMember) return 0.1;
  // forgot the non-member case
}
const price = base * (1 - discount(guest));   // NaN: 1 - undefined

// RIGHT: every path returns; lint with consistent-return
function discount(user) {
  return user.isMember ? 0.1 : 0;
}""",
                         "explanation": "A missing return is undefined, and arithmetic on undefined is NaN: a wrong number with no stack trace.",
                         "output": "guest: 1 - undefined -> NaN\nfixed: 1 - 0 -> 1"},
                        {"name": "Python", "slug": "python", "code":
"""# BUG: the loop may finish without returning -> implicit None
def find_admin(users):
    for u in users:
        if u.role == "admin":
            return u
    # no admin? falls off the end -> None

admin = find_admin(users)
admin.notify()        # AttributeError: 'NoneType' has no 'notify'

# RIGHT: make the 'not found' case explicit
def find_admin(users):
    return next((u for u in users if u.role == "admin"), None)""",
                         "explanation": "Python returns None when execution falls off the end. The caller crashes later, far from the function that 'forgot'.",
                         "output": "no admin: None -> AttributeError downstream\nexplicit: caller checks for None first"},
                    ],
                },
            ],
        },
        {
            "slug": "version-library-mismanagement",
            "cwes": [
                {"id": "CWE-1395", "name": "Dependency on Vulnerable Third-Party Component", "url": "https://cwe.mitre.org/data/definitions/1395.html"},
                {"id": "CWE-1104", "name": "Use of Unmaintained Third Party Components", "url": "https://cwe.mitre.org/data/definitions/1104.html"},
                {"id": "CWE-937", "name": "Using Components with Known Vulnerabilities", "url": "https://cwe.mitre.org/data/definitions/937.html"},
            ],
            "name": "Version & Library Mismanagement",
            "tagline": "dependencies drifting out from under you",
            "essence": "The code you tested and the code that ships resolve to different library versions.",
            "description": "Floating version ranges, unpinned lockfiles, and copy-pasted helpers make the dependency tree itself unstructured input. A 'patch' bump changes behavior, two libraries demand incompatible versions, and 'works on my machine' becomes a deployment incident.",
            "examples": [
                {
                    "title": "The Caret That Shipped a Surprise",
                    "intent": "A floating range pulls a new version you never tested at build time.",
                    "languages": [
                        {"name": "JSON", "slug": "json", "code":
"""// package.json -- WRONG: ranges float on every install
{
  "dependencies": {
    "date-utils": "^1.2.0"
  }
}
// ^1.2.0 allows any 1.x; a 1.3.0 with a regression slips in silently.
// RIGHT: commit package-lock.json and install with `npm ci`,
// which installs the locked tree verbatim.""",
                         "explanation": "^1.2.0 means 'any 1.x' -- a minor release with a behavior change installs silently. A committed lockfile plus npm ci makes installs reproducible.",
                         "output": "^1.2.0: build A gets 1.2.0, build B gets 1.4.7\nnpm ci: every build gets the locked version"},
                        {"name": "Python", "slug": "python", "code":
"""# requirements.txt -- WRONG: unpinned, resolves differently over time
requests
urllib3

# RIGHT: pin exact versions for reproducible installs
requests==2.31.0
urllib3==2.2.1""",
                         "explanation": "Unpinned requirements resolve to 'whatever is newest today'. Pinning makes the dependency set part of the code you reviewed.",
                         "output": "unpinned: prod pulls a newer, untested urllib3\npinned:   prod matches the tested set exactly"},
                    ],
                },
                {
                    "title": "Copy-Pasta Drift",
                    "intent": "A helper copied into three services becomes three subtly different helpers.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// service-a: the original, later patched for a leap-year bug
function daysInMonth(y, m) { /* ...fixed... */ }

// service-b: a copy-paste from a year ago -- never got the fix
function daysInMonth(y, m) { /* ...buggy in February... */ }

// The 'same' function now disagrees across services.""",
                         "explanation": "Copied code forks silently: a fix applied to one copy never reaches the others. The duplication is the version-mismanagement.",
                         "output": "service-a: Feb 2024 -> 29 (fixed)\nservice-b: Feb 2024 -> 28 (stale copy)"},
                        {"name": "JSON", "slug": "json", "code":
"""// RIGHT: one source of truth, versioned as a real dependency
{
  "dependencies": {
    "@acme/dates": "1.4.0"
  }
}
// One place to fix, one version to bump everywhere.""",
                         "explanation": "Extracting the helper into a versioned package replaces N drifting copies with one dependency every consumer pins.",
                         "output": "copies:  N forks, fixes diverge\npackage: one fix, bump the version everywhere"},
                    ],
                },
            ],
        },
        {
            "slug": "insecure-deserialization",
            "cwes": [
                {"id": "CWE-502", "name": "Deserialization of Untrusted Data", "url": "https://cwe.mitre.org/data/definitions/502.html"},
                {"id": "CWE-915", "name": "Improperly Controlled Modification of Dynamically-Determined Object Attributes", "url": "https://cwe.mitre.org/data/definitions/915.html"},
                {"id": "CWE-913", "name": "Improper Control of Dynamically-Managed Code Resources", "url": "https://cwe.mitre.org/data/definitions/913.html"},
                {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html"},
            ],
            "name": "Insecure Deserialization",
            "tagline": "untrusted bytes become live objects",
            "essence": "Hydrating untrusted input straight into live objects runs code and sets state the caller never wrote.",
            "description": "Deserialization is the purest form of consuming unknown input as state: bytes you did not create become objects with behavior. Native formats can execute code on the way in (chaining to RCE, CWE-94/78), and even 'safe' formats over-assign attributes the caller never meant to expose.",
            "examples": [
                {
                    "title": "Hydrating Untrusted Bytes",
                    "intent": "A native deserializer reconstructs objects \u2014 and can run their code \u2014 before any of your logic sees the data.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""import pickle

# DANGER: pickle reconstructs arbitrary objects -- and runs __reduce__
data = pickle.loads(untrusted_bytes)   # attacker payload = RCE

# An attacker's class can do this on load:
class Exploit:
    def __reduce__(self):
        return (os.system, ("rm -rf /",))   # runs during loads()

# RIGHT: a data-only format + an explicit schema
import json
from pydantic import BaseModel
class Order(BaseModel): id: int; qty: int
order = Order(**json.loads(untrusted_bytes))   # validated, no code""",
                         "explanation": "pickle.loads will execute an object's __reduce__ as it rebuilds it, so loading attacker bytes is arbitrary code execution. Parse a data-only format and validate against a schema instead.",
                         "output": "pickle: __reduce__ runs -> shell command\nschema: bad shape -> ValidationError, no code"},
                        {"name": "Java", "slug": "java", "code":
"""// DANGER: readObject rebuilds any serializable type on the classpath
ObjectInputStream in = new ObjectInputStream(socket.getInputStream());
Object o = in.readObject();        // gadget chains -> RCE

// RIGHT: don't deserialize untrusted native streams.
// Use a data format with a known target type:
ObjectMapper m = new ObjectMapper();
m.activateDefaultTyping(...);      // <- do NOT; this re-opens the hole
Order order = m.readValue(json, Order.class);   // fixed target type""",
                         "explanation": "Java's native readObject instantiates whatever the stream names, letting gadget chains reach dangerous code. Avoid native serialization for untrusted data; bind a known type from a data format.",
                         "output": "readObject: gadget chain -> code execution\nreadValue:  bound to Order.class only"},
                    ],
                },
                {
                    "title": "Over-Permissive Hydration",
                    "intent": "Splatting raw input onto an object sets fields the caller never intended \u2014 like a privilege flag.",
                    "languages": [
                        {"name": "Ruby", "slug": "ruby", "code":
"""# WRONG: mass assignment trusts every key in the payload
user = User.new(params)         # params has admin: true -> privilege escalation
user.save

# RIGHT: strong params -- an explicit allowlist of fields
permitted = params.require(:user).permit(:name, :email)
user = User.new(permitted)      # admin can't be set from input""",
                         "explanation": "Handing the whole params hash to a model lets a caller set any attribute, including admin. An allowlist (strong params) makes the writable set explicit.",
                         "output": "mass assign: admin: true -> account escalated\nallowlist:   only name/email accepted"},
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// WRONG: copy every field from the request onto the record
Object.assign(user, req.body);   // req.body.role = "admin" sticks
await user.save();

// RIGHT: pick only the fields you mean to accept
const { name, email } = req.body;
Object.assign(user, { name, email });   // role is untouched""",
                         "explanation": "Object.assign with raw req.body lets the client overwrite any property, including role. Destructure the known-safe fields and assign only those.",
                         "output": "assign body: role -> 'admin' (overwritten)\npick fields: role unchanged, name/email set"},
                    ],
                },
            ],
        },
    ],
})

# ─────────────────────────────────────────────────────────────────────────────
# 3. SHARED STATE
# ─────────────────────────────────────────────────────────────────────────────
DEFECTS.append({
    "id": "shared-state",
    "cwe_pillar": {"id": "CWE-662", "name": "Improper Synchronization", "url": "https://cwe.mitre.org/data/definitions/662.html"},
    "name": "Bugs of Shared State",
    "short_name": "Shared",
    "category": "shared",
    "tagline": "concurrency and caching of state",
    "color_primary": "DD3333",
    "color_secondary": "F87171",
    "color_deep": "B91C1C",
    "description": "Bugs caused by concurrency, caching, and the distribution of state across multiple independent actors or processes.",
    "essence": "State changes may not be instantaneous, may not be everywhere, and may not be complete.",
    "practice": "Every defect on this page comes from state shared across actors that don't move in lockstep. It shows up as data that's stale, out of order, or 'works on my machine.' The tools that make it easy are threads and mutexes, caches, pass-by-reference, and async code that drops the ball.",
    "symptoms": [
        "Data is inaccessible or out of order",
        "Resources are slow or unavailable",
        "Information is wrong or outdated",
        "Communications are being dropped",
        "\u201cWorks on my machine\u201d",
    ],
    "causes": [
        "race conditions",
        "resource contention",
        "stack overflow bugs",
        "async IO misuse",
        "pointer mismanagement",
        "cache invalidation",
        "threading & mutexes",
        "file or network access",
        "limited hardware resources",
        "resource leaks & shutdown",
    ],
    "tools_hurt": [
        "threading / mutexes",
        "high availability",
        "replication and sharding",
        "operations sloppiness",
        "pass-by-reference",
        "async mismanagement",
    ],
    "philosophies": [
        {"name": "Dining Philosophers", "url": "https://en.wikipedia.org/wiki/Dining_philosophers_problem"},
        {"name": "Fallacies of Distributed Computing", "url": "https://en.wikipedia.org/wiki/Fallacies_of_distributed_computing"},
        {"name": "CAP Theorem", "url": "https://en.wikipedia.org/wiki/CAP_theorem"},
        {"name": "DoD Memory Safety Report", "url": "https://media.defense.gov/2022/Nov/10/2003112742/-1/-1/0/CSI_SOFTWARE_MEMORY_SAFETY.PDF"},
    ],
    "two_hard_note": "cache invalidation",
    # Human <-> Computer analogy table (slide 7)
    "analogy_table": {
        "caption": "Every shared-state problem has a human name. Computers just hit them faster.",
        "rows": [
            {"computer": "Cache Invalidation", "human": "Product Recalls",
             "note": "A product is supplied from a distribution point; changing it means changing every distribution. Invalidation is business logic, a work order that takes a process and happens now."},
            {"computer": "Cross-Boundary State Exposure", "human": "Crossed Phone Lines",
             "note": "On an old party line, one household could lift the receiver and hear another's call. Shared state reused across actors leaks one person's data to the next."},
            {"computer": "Resource Contention", "human": "Tragedy of the Commons",
             "note": "Independent actors each act in their own best interest and deplete a shared resource through uncoordinated allocation."},
            {"computer": "Async Misuse", "human": "Undeliverable Mail",
             "note": "Messages in transit occupy space, and not every message sent over the network gets through. No message may be silently dropped."},
        ],
    },
    # Deadlock vs Livelock (slides 13-15)
    "deadlock_table": {
        "caption": "Two ways concurrency stalls. Both achieve nothing.",
        "rows": [
            {"name": "Deadlock", "human": "an impasse",
             "desc": "Multiple actors wait on one another, each paused until an impossible condition is met. NOTHING IS HAPPENING. NOTHING IS ACHIEVED."},
            {"name": "Livelock", "human": "two people dodging in a hallway",
             "desc": "Actors execute too early and keep reacting to each other; success is impossible unless someone waits. SO MUCH IS HAPPENING. NOTHING IS ACHIEVED."},
        ],
    },
    # Execution model spectrum (project plan + slide 11 memory safety)
    "spectrum": {
        "caption": "How far you are from the metal decides what a shared-state bug costs you.",
        "rows": [
            {"model": "Interpreted", "langs": "Python, JavaScript, Ruby",
             "trait": "Runs inside an interpreter process. Runtime exceptions; the interpreter does its best to recover."},
            {"model": "Compiled (VM)", "langs": "Java, C#",
             "trait": "Bytecode on a sandboxed virtual machine. Exceptions are handled by the VM; recovery is possible."},
            {"model": "Executed (native)", "langs": "C, Rust, Assembly",
             "trait": "Architecture-specific machine code. No exceptions, only segfaults. Memory safety is the whole game."},
        ],
    },
    # Per-defect children. The shared-state family page renders as an overview
    # hub linking to these; each child has its own page, text Lottie, and examples.
    "defects": [
        {
            "slug": "race-conditions",
            "cwes": [
                {"id": "CWE-362", "name": "Concurrent Execution using Shared Resource with Improper Synchronization ('Race Condition')", "url": "https://cwe.mitre.org/data/definitions/362.html"},
                {"id": "CWE-367", "name": "Time-of-check Time-of-use (TOCTOU) Race Condition", "url": "https://cwe.mitre.org/data/definitions/367.html"},
                {"id": "CWE-366", "name": "Race Condition within a Thread", "url": "https://cwe.mitre.org/data/definitions/366.html"},
            ],
            "name": "Race Conditions",
            "tagline": "timing, not logic, picks the answer",
            "essence": "When two actors touch one value at once, the interleaving decides the (wrong) result.",
            "description": "Two threads read-modify-write the same state, and the order they happen to interleave — not your code — chooses what survives.",
            "examples": [
                {
                    "title": "Lost-Update Counter",
                    "intent": "Two actors read-modify-write the same value, and the interleaving loses updates.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""import threading
counter = 0

def bump():
    global counter
    for _ in range(100_000):
        counter += 1           # read, +1, write -- not atomic

# Two threads race; final value is < 200000
t1, t2 = threading.Thread(target=bump), threading.Thread(target=bump)

# RIGHT: a lock makes the read-modify-write atomic
lock = threading.Lock()
def bump_safe():
    global counter
    for _ in range(100_000):
        with lock:
            counter += 1""",
                         "explanation": "counter += 1 is three operations. Without a lock, two threads interleave and lose updates.",
                         "output": "racy:  counter = 137422  (lost updates)\nlocked: counter = 200000  (correct)"},
                        {"name": "Go", "slug": "go", "code":
"""// Detect races before they ship:  go test -race
var count int
var wg sync.WaitGroup
for i := 0; i < 1000; i++ {
    wg.Add(1)
    go func() { count++; wg.Done() }()   // -race flags this
}

// RIGHT: a mutex, or a channel to serialize access
var mu sync.Mutex
go func() { mu.Lock(); count++; mu.Unlock(); wg.Done() }()""",
                         "explanation": "Go ships a race detector. Run it in CI and the data race fails the build instead of production.",
                         "output": "go test -race: WARNING: DATA RACE\nwith mutex:    clean, deterministic"},
                        {"name": "Rust", "slug": "rust", "code":
"""// Rust refuses to compile a data race. Shared + mutable
// must go through a synchronizing type.
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let c = Arc::clone(&counter);
thread::spawn(move || { *c.lock().unwrap() += 1; });""",
                         "explanation": "The borrow checker enforces 'shared XOR mutable' at compile time. A data race is not expressible.",
                         "output": "raw shared &mut across threads: won't compile\nArc<Mutex<T>>: safe, checked statically"},
                    ],
                },
                {
                    "title": "Check-Then-Act (TOCTOU)",
                    "intent": "The gap between testing a condition and acting on it is where a second actor slips in.",
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: check-then-act across threads
if (instance == null) {        // thread A and B both see null
    instance = new Service();  // two instances created
}

// RIGHT: atomic or properly synchronized init
private static final AtomicReference<Service> ref = new AtomicReference<>();
ref.compareAndSet(null, new Service());""",
                         "explanation": "The gap between the check and the act is where the race lives. compareAndSet closes it atomically.",
                         "output": "naive:  two Service instances under load\nCAS:    exactly one wins, the rest reuse it"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: check-then-act on shared state
if username not in users:        # two requests both see it free
    users[username] = create()   # both create -> duplicate user

# RIGHT: make the check-and-set one atomic step
with lock:                       # (or a DB unique constraint)
    if username not in users:
        users[username] = create()""",
                         "explanation": "Reading 'is it free?' and writing 'claim it' must be one indivisible step, or two callers both win the check.",
                         "output": "unlocked: two duplicate accounts created\nlocked:   one wins, the other sees it taken"},
                    ],
                },
            ],
        },
        {
            "slug": "cross-boundary-state-exposure",
            "cwes": [
                {"id": "CWE-488", "name": "Exposure of Data Element to Wrong Session", "url": "https://cwe.mitre.org/data/definitions/488.html"},
                {"id": "CWE-543", "name": "Use of Singleton Pattern Without Synchronization in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/543.html"},
                {"id": "CWE-567", "name": "Unsynchronized Access to Shared Data in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/567.html"},
                {"id": "CWE-1108", "name": "Excessive Reliance on Global Variables", "url": "https://cwe.mitre.org/data/definitions/1108.html"},
            ],
            "name": "Cross-Boundary State Exposure",
            "tagline": "one actor's data, served to another",
            "essence": "Per-actor state parked in a shared singleton, global, or pool bleeds across the boundary that was meant to keep actors apart.",
            "description": "A singleton, module global, or pooled object is reused across requests, threads, or sessions without resetting its per-actor fields. Whoever touched it last leaves their data for whoever touches it next — so one user is handed another's state.",
            "examples": [
                {
                    "title": "Shared Singleton Holds Per-Request State",
                    "intent": "A single shared object with mutable, per-call state lets concurrent actors overwrite and read each other's data.",
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: one shared SimpleDateFormat for every thread
static final SimpleDateFormat FMT = new SimpleDateFormat("yyyy-MM-dd");
String render(Date d) { return FMT.format(d); }   // FMT keeps a mutable
                                                  // Calendar -> threads read
                                                  // each other's half-write

// RIGHT: give each call its own formatter (no shared state)
String render(Date d) {
    return new SimpleDateFormat("yyyy-MM-dd").format(d);
}""",
                         "explanation": "SimpleDateFormat is not thread-safe; a static instance shares one mutable Calendar, so concurrent threads corrupt each other's formatting.",
                         "output": "shared:   garbled / interleaved dates under load\nper-call: every thread formats its own date correctly"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a module-level slot reused by every concurrent request
_current_user = None
def set_user(u):
    global _current_user
    _current_user = u
def audit(action):
    log(_current_user, action)   # another request may have overwritten it

# RIGHT: carry per-request state with the request, not in a global
def audit(user, action):
    log(user, action)            # state travels with the caller""",
                         "explanation": "A module global is one slot shared by every concurrent request; the last writer wins and everyone else reads its value.",
                         "output": "global:   request B's action logged under user A\nper-call: each request keeps its own user"},
                    ],
                },
                {
                    "title": "Pooled Object Reused Without Reset",
                    "intent": "An object returned to a pool still carrying its last user's fields hands that data to the next borrower.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a pooled context handed out without clearing the last user
ctx = pool.acquire()             # still holds the previous request's user
ctx.run(request)                 # ...so this request acts as the wrong user
pool.release(ctx)

# RIGHT: reset the per-actor state on borrow, so nothing crosses over
ctx = pool.acquire()
ctx.reset()                      # wipe identity before reuse
ctx.user = request.user
ctx.run(request)
pool.release(ctx)""",
                         "explanation": "A pool recycles objects for speed; if per-actor fields are not reset on borrow, the previous borrower's identity leaks into the next request.",
                         "output": "no reset: request acts under the prior user's identity\nreset:    each borrow starts from a clean object"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: a ThreadLocal left set when the thread returns to the pool
static final ThreadLocal<User> CURRENT = new ThreadLocal<>();
void handle(Req r) {
    CURRENT.set(r.user());
    process();                   // pooled thread reused -> CURRENT still set
}                                // for the *next* request on this thread

// RIGHT: clear the ThreadLocal in a finally, before the thread is recycled
void handle(Req r) {
    CURRENT.set(r.user());
    try { process(); }
    finally { CURRENT.remove(); }
}""",
                         "explanation": "Servlet/worker threads are pooled and reused; a ThreadLocal left set bleeds into the next request that lands on the same thread.",
                         "output": "leak:  next request inherits the prior user via ThreadLocal\nremove: state cleared before the thread is reused"},
                    ],
                },
            ],
        },
        {
            "slug": "resource-contention",
            "cwes": [
                {"id": "CWE-400", "name": "Uncontrolled Resource Consumption", "url": "https://cwe.mitre.org/data/definitions/400.html"},
                {"id": "CWE-770", "name": "Allocation of Resources Without Limits or Throttling", "url": "https://cwe.mitre.org/data/definitions/770.html"},
                {"id": "CWE-833", "name": "Deadlock", "url": "https://cwe.mitre.org/data/definitions/833.html"},
            ],
            "name": "Resource Contention",
            "tagline": "the tragedy of the commons",
            "essence": "Independent actors each act in self-interest and exhaust a shared, finite resource.",
            "description": "Connections, threads, file handles, memory — finite pools drained by uncoordinated allocation. The computer's tragedy of the commons.",
            "examples": [
                {
                    "title": "Connection-Pool Exhaustion",
                    "intent": "A resource grabbed and never returned drains the pool until every request hangs.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a connection acquired and never returned to the pool
def handler(req):
    conn = pool.acquire()        # under load the pool drains to zero
    rows = conn.query(req.sql)   # an exception here leaks the conn
    return rows                  # ...and it is never released

# RIGHT: always return the resource, even on error
def handler(req):
    with pool.acquire() as conn:     # context manager releases on exit
        return conn.query(req.sql)""",
                         "explanation": "Any early return or exception between acquire and release leaks the resource. A context manager makes release unconditional.",
                         "output": "leak: pool exhausted, every request blocks\nwith: connection returned, throughput stable"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: close() skipped on the exception path
Connection c = pool.getConnection();
Result r = c.query(sql);   // throws -> c is never closed
c.close();

// RIGHT: try-with-resources closes c no matter what
try (Connection c = pool.getConnection()) {
    return c.query(sql);
}""",
                         "explanation": "The manual close() runs only on the happy path. try-with-resources guarantees release on every path.",
                         "output": "manual close: leaked on every thrown query\ntry-with:     released on success and failure"},
                    ],
                },
                {
                    "title": "Unbounded Concurrency",
                    "intent": "One worker per item looks elegant until a million items melt the machine.",
                    "languages": [
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: one goroutine per item -> millions of goroutines, OOM
for _, job := range jobs {
    go process(job)              // nothing bounds the fan-out
}

// RIGHT: a buffered channel caps concurrency
sem := make(chan struct{}, 16)   // at most 16 in flight
for _, job := range jobs {
    sem <- struct{}{}
    go func(j Job) { defer func() { <-sem }(); process(j) }(job)
}""",
                         "explanation": "Unbounded fan-out treats the scheduler and memory as infinite. A semaphore bounds the commons to a fair, fixed width.",
                         "output": "unbounded: memory spikes, scheduler thrashes\nsemaphore: steady 16-wide, predictable load"},
                        {"name": "Python", "slug": "python", "code":
"""import asyncio

# SMELL: launch every task at once -> thousands of open sockets
await asyncio.gather(*(fetch(u) for u in urls))   # all 50k at once

# RIGHT: a semaphore bounds how many run concurrently
sem = asyncio.Semaphore(20)
async def bounded(u):
    async with sem:
        return await fetch(u)
await asyncio.gather(*(bounded(u) for u in urls))""",
                         "explanation": "gather schedules everything immediately. Wrapping each call in a semaphore caps the in-flight set to a sustainable size.",
                         "output": "unbounded: 'too many open files', server 503s\nbounded:   20 at a time, steady and polite"},
                    ],
                },
            ],
        },
        {
            "slug": "stack-overflow",
            "cwes": [
                {"id": "CWE-674", "name": "Uncontrolled Recursion", "url": "https://cwe.mitre.org/data/definitions/674.html"},
                {"id": "CWE-121", "name": "Stack-based Buffer Overflow", "url": "https://cwe.mitre.org/data/definitions/121.html"},
                {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html"},
            ],
            "name": "Stack Overflow Bugs",
            "tagline": "recursion with no floor",
            "essence": "A call stack that never unwinds, or a heap that never stops growing, runs out of room.",
            "description": "Unbounded recursion exhausts the stack; runaway allocation exhausts the heap and the OOM killer steps in. Either way the program is reaped.",
            "examples": [
                {
                    "title": "Unbounded Recursion",
                    "intent": "No base case is reached, so the stack grows until it is gone.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: no base case reached (e.g. a cyclic list)
def depth(node):
    return 1 + depth(node.next)   # recurses forever

# RIGHT: iterate -- constant stack, any length
def depth(node):
    n = 0
    while node is not None:
        n += 1
        node = node.next
    return n""",
                         "explanation": "Python caps recursion at ~1000 frames and raises RecursionError. An explicit loop uses O(1) stack and handles any input.",
                         "output": "recursive: RecursionError at ~1000 frames\niterative: returns, constant stack"},
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: deep recursion smashes the fixed-size stack */
long sum(long n) { return n == 0 ? 0 : n + sum(n - 1); }
sum(10000000);   /* segfault: stack overflow, no exception */

/* RIGHT: a loop uses constant stack */
long sum(long n) {
    long s = 0;
    for (; n > 0; n--) s += n;
    return s;
}""",
                         "explanation": "C does not check stack depth. Overflow is undefined behavior — usually a segfault, sometimes silent corruption.",
                         "output": "recursive: segmentation fault (stack overflow)\niterative: returns, constant stack"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: recursion deeper than the thread stack
int sum(int n) { return n == 0 ? 0 : n + sum(n - 1); }
sum(1_000_000);   // throws StackOverflowError

// RIGHT: iterate
int sum(int n) {
    int s = 0;
    for (; n > 0; n--) s += n;
    return s;
}""",
                         "explanation": "The JVM turns overflow into a catchable StackOverflowError rather than a crash — but it is still a bug, not a recovery strategy.",
                         "output": "recursive: StackOverflowError\niterative: returns, constant stack"},
                    ],
                },
                {
                    "title": "Runaway Allocation & the OOM Killer",
                    "intent": "Holding the whole dataset in memory invites the kernel to reap the process.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: materialize everything in RAM -> the OOM killer wins
rows = [transform(r) for r in db.fetch_all()]   # 50M rows at once
return summarize(rows)

# RIGHT: stream and process in bounded chunks
total = Summary()
for batch in db.iter_batches(size=10_000):
    total.update(transform(r) for r in batch)
return total""",
                         "explanation": "fetch_all() loads the entire result set; memory scales with the data and the kernel kills the process. Streaming keeps memory bounded.",
                         "output": "all-in-RAM: Killed  (reaped by the OOM killer)\nstreamed:  steady memory, completes"},
                    ],
                },
            ],
        },
        {
            "slug": "async-io-misuse",
            "cwes": [
                {"id": "CWE-362", "name": "Concurrent Execution using Shared Resource with Improper Synchronization ('Race Condition')", "url": "https://cwe.mitre.org/data/definitions/362.html"},
                {"id": "CWE-662", "name": "Improper Synchronization", "url": "https://cwe.mitre.org/data/definitions/662.html"},
                {"id": "CWE-696", "name": "Incorrect Behavior Order", "url": "https://cwe.mitre.org/data/definitions/696.html"},
            ],
            "name": "Async IO Misuse",
            "tagline": "messages sent, never delivered",
            "essence": "Fire-and-forget work drops results and failures no one notices are gone.",
            "description": "Floating promises, un-awaited tasks, and unhandled rejections: async work whose ordering and errors vanish. The computer's undeliverable mail.",
            "examples": [
                {
                    "title": "Floating Promises",
                    "intent": "Promises no one awaits run in undefined order and swallow their own errors.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: floating promise -- errors vanish, order is undefined
items.forEach(async (i) => { await save(i); });  // not awaited
doNext();   // runs before any save() finishes

// RIGHT: await the whole batch, surface failures
const results = await Promise.allSettled(items.map(save));
const failed = results.filter(r => r.status === "rejected");
if (failed.length) throw new AggregateError(failed, "some saves failed");""",
                         "explanation": "forEach ignores the returned promises, so doNext() races ahead and rejections are swallowed. allSettled waits and reports.",
                         "output": "forEach: doNext() runs early; errors lost\nallSettled: waits for all; failures collected"},
                        {"name": "Python", "slug": "python", "code":
"""import asyncio

# SMELL: task created but never awaited -> may be GC'd mid-flight
async def main():
    asyncio.create_task(save(x))   # warning: never awaited
    return

# RIGHT: gather and await, propagate exceptions
async def main():
    await asyncio.gather(*(save(i) for i in items))""",
                         "explanation": "An un-awaited task can be garbage-collected before it finishes. gather() keeps references and re-raises errors.",
                         "output": "create_task only: 'Task was destroyed but it is pending!'\ngather:           all complete or first error raised"},
                    ],
                },
                {
                    "title": "Awaiting in a Loop",
                    "intent": "Awaiting each call in turn serializes I/O that could have run concurrently.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: awaiting in a loop runs requests one at a time
const out = [];
for (const id of ids) {
  out.push(await fetchUser(id));   // N round-trips, strictly serial
}

// RIGHT: start them together, await as a batch
const out = await Promise.all(ids.map(fetchUser));""",
                         "explanation": "Each await blocks the next iteration, so total time is the sum of every request. Promise.all overlaps them.",
                         "output": "serial:      ~1.2s for 20 ids\nPromise.all: ~60ms (bounded by the slowest)"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: awaiting inside the loop serializes the I/O
results = []
for uid in ids:
    results.append(await fetch_user(uid))   # one at a time

# RIGHT: schedule together, await once
results = await asyncio.gather(*(fetch_user(u) for u in ids))""",
                         "explanation": "await suspends the coroutine until that one call returns before starting the next. gather lets them run at the same time.",
                         "output": "serial: sum of every request's latency\ngather: bounded by the slowest single request"},
                    ],
                },
            ],
        },
        {
            "slug": "pointer-mismanagement",
            "cwes": [
                {"id": "CWE-476", "name": "NULL Pointer Dereference", "url": "https://cwe.mitre.org/data/definitions/476.html"},
                {"id": "CWE-416", "name": "Use After Free", "url": "https://cwe.mitre.org/data/definitions/416.html"},
                {"id": "CWE-415", "name": "Double Free", "url": "https://cwe.mitre.org/data/definitions/415.html"},
                {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html"},
            ],
            "name": "Pointer Mismanagement",
            "tagline": "dangling, double-freed, used-after-free",
            "essence": "A pointer outlives what it points to, and the bytes start to lie.",
            "description": "Use-after-free, double-free, and dangling references — roughly 70% of serious CVEs. The bugs the DoD wrote a memo about.",
            "examples": [
                {
                    "title": "Use-After-Free & Double-Free",
                    "intent": "Freed memory is still pointed at, then read, written, or freed a second time.",
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""char *p = malloc(16);
free(p);
strcpy(p, "oops");   /* use-after-free: undefined behavior */
free(p);             /* double-free: heap corruption */

/* RIGHT: null after free, single owner, check allocations */
free(p);
p = NULL;            /* now a stray use is a clean NULL deref */""",
                         "explanation": "Roughly 70% of serious CVEs are memory-safety bugs like these. Nulling after free turns silent corruption into a loud crash.",
                         "output": "raw:    use-after-free -> exploitable corruption\np=NULL: later use -> immediate, debuggable crash"},
                        {"name": "Rust", "slug": "rust", "code":
"""// Ownership makes use-after-free a compile error.
let s = String::from("hello");
let moved = s;            // ownership moves out of `s`
// println!("{s}");       // error: borrow of moved value `s`

// Borrowing rules also separate owned String from borrowed &str
fn len(text: &str) -> usize { text.len() }   // borrows, never frees""",
                         "explanation": "The compiler tracks who owns each value and frees it exactly once, at the end of scope. Use-after-free cannot be written.",
                         "output": "use-after-move: rejected at compile time\nborrow &str:    no ownership taken, no double-free"},
                    ],
                },
                {
                    "title": "Dangling Returns (String vs &str)",
                    "intent": "Returning a reference to a local hands back a pointer to memory that just died.",
                    "languages": [
                        {"name": "Rust", "slug": "rust", "code":
"""// Returning a reference to a local would dangle.
// Rust rejects it at compile time.
fn greeting() -> &str {        // error: missing lifetime; borrows a local
    let s = String::from("hi");
    &s                          // `s` is dropped at end of scope
}

// RIGHT: hand back ownership, not a borrow
fn greeting() -> String { String::from("hi") }""",
                         "explanation": "An owned String moves out to the caller; a borrowed &str into a local cannot outlive it, and the borrow checker says so.",
                         "output": "borrow of local: rejected (would dangle)\nowned String:    caller owns it, no dangling"},
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: returning the address of a stack local */
char *greeting(void) {
    char buf[3] = "hi";
    return buf;          /* buf dies when the function returns */
}

/* RIGHT: heap-allocate (caller frees) or pass a buffer in */
char *greeting(void) { return strdup("hi"); }""",
                         "explanation": "C will happily return &buf; the pointer dangles the instant the frame is popped. strdup gives the caller something that actually lives.",
                         "output": "stack return: dangling pointer, garbage on read\nstrdup:       valid until the caller free()s it"},
                    ],
                },
            ],
        },
        {
            "slug": "cache-invalidation",
            "cwes": [
                {"id": "CWE-349", "name": "Acceptance of Extraneous Untrusted Data With Trusted Data", "url": "https://cwe.mitre.org/data/definitions/349.html"},
                {"id": "CWE-524", "name": "Use of Cache Containing Sensitive Information", "url": "https://cwe.mitre.org/data/definitions/524.html"},
            ],
            "name": "Cache Invalidation",
            "tagline": "stale data, served with confidence",
            "essence": "A write that does not invalidate is a bug waiting for the next read.",
            "description": "Invalidation isn't a footnote — it's business logic, a work order that takes a process and happens now. One of the two hard problems.",
            "two_hard_note": "cache invalidation",
            "examples": [
                {
                    "title": "Stale Reads After a Write",
                    "intent": "Cache forever, never invalidate, and every reader sees the pre-update world.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""cache = {}

# SMELL: cache forever, never invalidate -> stale reads
def get_user(uid):
    if uid not in cache:
        cache[uid] = db.fetch(uid)
    return cache[uid]               # update in db? caller never sees it

# RIGHT: invalidate on write; treat it as part of the work order
def update_user(uid, data):
    db.write(uid, data)
    cache.pop(uid, None)            # the recall happens now""",
                         "explanation": "A write that doesn't invalidate the cache is a bug waiting for the next read. Invalidation is a process, and it happens right now.",
                         "output": "no invalidation: reads show pre-update data forever\non-write pop:    next read is fresh from db"},
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: write the DB, forget the cache -> stale forever
func Update(id string, v Value) {
    db.Write(id, v)               // cache still holds the old value
}

// RIGHT: invalidate (or update) the cache as part of the write
func Update(id string, v Value) {
    db.Write(id, v)
    cache.Delete(id)              // next read repopulates from db
}""",
                         "explanation": "The cache is part of your write path, not a side concern. If a write doesn't touch it, the cache lies until eviction.",
                         "output": "no delete: cached read stays stale indefinitely\nwith delete: next read reflects the write"},
                    ],
                },
                {
                    "title": "Thundering Herd on Expiry",
                    "intent": "Everything expires at the same instant and every miss stampedes the database.",
                    "languages": [
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: TTL with no jitter -> thundering herd on expiry
cache.Set(key, val, 60*time.Second)   // 10k keys expire at once

// RIGHT: add jitter so expirations spread out
ttl := 60*time.Second + time.Duration(rand.Intn(15))*time.Second
cache.Set(key, val, ttl)""",
                         "explanation": "If everything expires at the same instant, every miss hits the database simultaneously. Jittered TTLs smear the load.",
                         "output": "fixed TTL: synchronized stampede on the DB\njitter:    misses spread across a 15s window"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: every concurrent miss recomputes the same hot key
def get(key):
    v = cache.get(key)
    if v is None:
        v = expensive(key)         # 1000 misses -> 1000 calls
        cache[key] = v
    return v

# RIGHT: single-flight -- one loader, the rest wait on it
def get(key):
    with inflight.lock(key):       # per-key lock
        v = cache.get(key)
        if v is None:
            v = expensive(key)
            cache[key] = v
        return v""",
                         "explanation": "On a cold hot key, every concurrent reader recomputes it. A per-key lock collapses the herd to a single load.",
                         "output": "naive:        N concurrent misses -> N expensive() calls\nsingle-flight: 1 call, the rest reuse the result"},
                    ],
                },
            ],
        },
        {
            "slug": "threading-mutexes",
            "cwes": [
                {"id": "CWE-667", "name": "Improper Locking", "url": "https://cwe.mitre.org/data/definitions/667.html"},
                {"id": "CWE-833", "name": "Deadlock", "url": "https://cwe.mitre.org/data/definitions/833.html"},
                {"id": "CWE-820", "name": "Missing Synchronization", "url": "https://cwe.mitre.org/data/definitions/820.html"},
            ],
            "name": "Threading & Mutexes",
            "tagline": "locks held, ordered, or forgotten",
            "essence": "The lock that protects you from a race is the lock that deadlocks you.",
            "description": "Mutexes serialize access — until one is acquired out of order (deadlock) or retried in lockstep (livelock). Both achieve nothing.",
            "examples": [
                {
                    "title": "Deadlock: Acquire in One Order",
                    "intent": "Two locks taken in opposite orders freeze forever, each thread waiting on the other.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# DEADLOCK: two locks taken in opposite orders
# thread 1: lock(a) ; lock(b)
# thread 2: lock(b) ; lock(a)   <- each waits on the other

# RIGHT: impose a global lock ordering
def transfer(src, dst, amount):
    first, second = sorted((src, dst), key=id)
    with first.lock, second.lock:        # always same order
        src.balance -= amount
        dst.balance += amount""",
                         "explanation": "Deadlock needs a cycle in the wait graph. Acquire locks in one consistent order and the cycle can't form.",
                         "output": "unordered: threads freeze, NOTHING ACHIEVED\nordered:   no cycle, transfers complete"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: lock order depends on argument order -> deadlock
void transfer(Account a, Account b, long amt) {
    synchronized (a) { synchronized (b) { /* ... */ } }
}

// RIGHT: order locks by a stable key (e.g. account id)
void transfer(Account a, Account b, long amt) {
    Account first = a.id < b.id ? a : b, second = a.id < b.id ? b : a;
    synchronized (first) { synchronized (second) { /* ... */ } }
}""",
                         "explanation": "transfer(x, y) and transfer(y, x) grab the locks in opposite orders. Sorting by a stable key gives every thread the same order.",
                         "output": "by-arg order: two transfers deadlock\nby-id order:  no cycle possible"},
                    ],
                },
                {
                    "title": "Livelock: Back Off With Jitter",
                    "intent": "Both threads politely release and retry in lockstep, busy forever, achieving nothing.",
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// LIVELOCK: both threads politely back off forever
while (!acquireBoth()) {
    releaseAll();
    Thread.sleep(0);     // both retry in lockstep, get nowhere
}

// RIGHT: randomized backoff breaks the symmetry
while (!acquireBoth()) {
    releaseAll();
    Thread.sleep(random.nextInt(50));   // jitter
}""",
                         "explanation": "Livelock is deadlock's busy cousin: lots of activity, zero progress. Random jitter desynchronizes the retries.",
                         "output": "lockstep: SO MUCH IS HAPPENING, nothing done\njitter:   one thread wins, work proceeds"},
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: spin-retry with no backoff -> two goroutines livelock
for !tryAcquire() {
    release()
    // immediately retry, in perfect step with the other
}

// RIGHT: exponential backoff with jitter
for attempt := 0; !tryAcquire(); attempt++ {
    release()
    d := time.Duration(1<<attempt)*time.Millisecond + jitter()
    time.Sleep(d)
}""",
                         "explanation": "Tight retry loops keep two actors in sync so neither makes progress. Growing, jittered sleeps break the symmetry.",
                         "output": "no backoff: 100% CPU, no progress\nbackoff:    one acquires, the other follows"},
                    ],
                },
            ],
        },
        {
            "slug": "file-network-access",
            "cwes": [
                {"id": "CWE-22", "name": "Improper Limitation of a Pathname to a Restricted Directory ('Path Traversal')", "url": "https://cwe.mitre.org/data/definitions/22.html"},
                {"id": "CWE-367", "name": "Time-of-check Time-of-use (TOCTOU) Race Condition", "url": "https://cwe.mitre.org/data/definitions/367.html"},
                {"id": "CWE-73", "name": "External Control of File Name or Path", "url": "https://cwe.mitre.org/data/definitions/73.html"},
            ],
            "name": "File & Network Access",
            "tagline": "the world between processes is unreliable",
            "essence": "Disk and network calls fail partway, arrive twice, or never arrive at all.",
            "description": "TOCTOU file races, partial writes, dropped packets, and retries without idempotency — the failures the Fallacies of Distributed Computing warn about.",
            "examples": [
                {
                    "title": "TOCTOU File Race",
                    "intent": "Checking then using a path leaves a gap where the file can change underneath you.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: check-then-use -- the file can change in the gap (TOCTOU)
if os.path.exists(path):          # an attacker swaps the file here...
    with open(path) as f:          # ...so you open something else
        return f.read()

# RIGHT: just try to open it; let one syscall decide atomically
try:
    with open(path) as f:
        return f.read()
except FileNotFoundError:
    return None""",
                         "explanation": "Between exists() and open() the path can be replaced (a symlink swap). Opening directly collapses check and use into one atomic operation.",
                         "output": "check-then-open: races, symlink swaps, surprises\ntry/except:     one atomic syscall, no gap"},
                        {"name": "Bash", "slug": "bash", "code":
"""# SMELL: test then act -- classic TOCTOU window
if [ -f \"$f\" ]; then
  rm \"$f\"          # $f may have changed between test and rm
fi

# RIGHT: act and tolerate the race
rm -f \"$f\"         # idempotent: no error if it is already gone""",
                         "explanation": "The [ -f ] test and the rm are two steps with a gap between them. rm -f is a single idempotent action that doesn't care about the race.",
                         "output": "[ -f ] then rm: window for a swap\nrm -f:         atomic, idempotent"},
                    ],
                },
                {
                    "title": "Retry & Timeout",
                    "intent": "A timed-out call may have already succeeded; a call with no deadline can hang forever.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: blind retry of a non-idempotent call -> double charge
for _ in range(3):
    try:
        return charge(card, amount)   # a timeout is not a failure!
    except Timeout:
        continue                       # the first call may have won

# RIGHT: an idempotency key makes retries safe to repeat
key = idempotency_key(order)
for _ in range(3):
    try:
        return charge(card, amount, idempotency_key=key)
    except Timeout:
        continue        # the server dedupes by key -> at most one charge""",
                         "explanation": "A timeout means 'unknown', not 'failed'. Retrying a non-idempotent call can double it; an idempotency key lets the server collapse retries to one effect.",
                         "output": "blind retry: a timed-out success is charged again\nidem key:    server collapses retries to one charge"},
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: no timeout -- a stalled peer hangs the caller forever
resp, err := http.Get(url)            // can block indefinitely

// RIGHT: bound every network call with a context deadline
ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
defer cancel()
req, _ := http.NewRequestWithContext(ctx, "GET", url, nil)
resp, err := http.DefaultClient.Do(req)""",
                         "explanation": "Without a deadline, one slow peer ties up a connection (and a goroutine) forever. A context timeout fails fast and frees the resource.",
                         "output": "no timeout: one slow peer exhausts the pool\nwith ctx:   fails fast at 2s, resources freed"},
                    ],
                },
            ],
        },
        {
            "slug": "resource-leaks",
            "cwes": [
                {"id": "CWE-404", "name": "Improper Resource Shutdown or Release", "url": "https://cwe.mitre.org/data/definitions/404.html"},
                {"id": "CWE-772", "name": "Missing Release of Resource after Effective Lifetime", "url": "https://cwe.mitre.org/data/definitions/772.html"},
                {"id": "CWE-401", "name": "Missing Release of Memory after Effective Lifetime", "url": "https://cwe.mitre.org/data/definitions/401.html"},
                {"id": "CWE-459", "name": "Incomplete Cleanup", "url": "https://cwe.mitre.org/data/definitions/459.html"},
                {"id": "CWE-775", "name": "Missing Release of File Descriptor or Handle after Effective Lifetime", "url": "https://cwe.mitre.org/data/definitions/775.html"},
            ],
            "name": "Resource Leaks & Improper Shutdown",
            "tagline": "opened, never closed",
            "essence": "A handle held past its lifetime — file, socket, connection, lock — is shared state nobody released; leaks accumulate until the system starves.",
            "description": "Every acquire needs a matching release. Forget it — or skip it on the error path — and file descriptors, connections, and memory pile up until the pool, the heap, or the OS limit runs out. The crash lands far from the leak, long after the request that caused it.",
            "examples": [
                {
                    "title": "Opened, Never Closed",
                    "intent": "A resource acquired in a loop and never released exhausts a finite pool.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a connection per iteration, none ever returned
for row in rows:
    conn = pool.acquire()      # leaks: never released
    conn.execute(row)
# ...pool exhausted; the next acquire() blocks forever

# RIGHT: a context manager releases on every path
for row in rows:
    with pool.acquire() as conn:   # returned even on exception
        conn.execute(row)""",
                         "explanation": "An acquire with no release drains the pool; once it's empty, every caller hangs. A with-block ties release to scope exit, including exceptions.",
                         "output": "leak: pool drains -> acquire() hangs\nwith: connection returned every iteration"},
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: response body is a file descriptor -- never closed
resp, _ := http.Get(url)
body, _ := io.ReadAll(resp.Body)   // resp.Body leaks every call
// thousands of requests -> "too many open files"

// RIGHT: defer Close right after the error check
resp, err := http.Get(url)
if err != nil { return err }
defer resp.Body.Close()            // released when the function returns
body, _ := io.ReadAll(resp.Body)""",
                         "explanation": "resp.Body holds an open socket/fd; without Close it leaks until the process hits its descriptor limit. defer pins the release to function exit.",
                         "output": "no close: fd leak -> too many open files\ndefer:    each body closed on return"},
                    ],
                },
                {
                    "title": "Cleanup Skipped on the Error Path",
                    "intent": "An early return or exception jumps over the release, so the happy path closes but the failure path leaks.",
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: an exception before close() leaks the stream
InputStream in = new FileInputStream(path);
parse(in);                 // throws -> close() never runs
in.close();

// RIGHT: try-with-resources closes on every exit path
try (InputStream in = new FileInputStream(path)) {
    parse(in);             // close() runs even if parse throws
}""",
                         "explanation": "A close() after the work only runs when the work succeeds; any throw leaks the handle. try-with-resources guarantees close on normal and exceptional exit.",
                         "output": "manual: parse() throws -> stream leaked\ntry-with: closed on success and on throw"},
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: early return leaks the file and the buffer */
int load(const char *path) {
    FILE *f = fopen(path, "r");
    char *buf = malloc(N);
    if (read_header(f) < 0) return -1;   /* leaks f AND buf */
    /* ...use buf... */
    free(buf); fclose(f);
    return 0;
}

/* RIGHT: single cleanup path via goto */
int load(const char *path) {
    int rc = -1;
    FILE *f = fopen(path, "r");
    char *buf = malloc(N);
    if (read_header(f) < 0) goto out;
    rc = 0;
out:
    free(buf); if (f) fclose(f);   /* always runs */
    return rc;
}""",
                         "explanation": "Each early return must remember to free everything acquired so far — easy to miss. A single cleanup label centralizes release so no exit path can skip it.",
                         "output": "early return: f and buf leaked on error\ngoto out:   every exit frees both"},
                    ],
                },
            ],
        },
    ],
})

# ─────────────────────────────────────────────────────────────────────────────
# 4. COMMON SOFTWARE DEFECTS  (the 4th category from the overview deck)
# ─────────────────────────────────────────────────────────────────────────────
DEFECTS.append({
    "id": "common-defects",
    "cwe_pillar": {"id": "CWE-710", "name": "Improper Adherence to Coding Standards", "url": "https://cwe.mitre.org/data/definitions/710.html"},
    "name": "Common Software Defects",
    "short_name": "Common",
    "category": "common",
    "tagline": "the everyday smells beneath the stateful ones",
    "color_primary": "94A3B8",
    "color_secondary": "CBD5E1",
    "color_deep": "64748B",
    "description": "The everyday defects in every codebase, read by how they're caught: readability smells a reviewer sees (duplication, magic numbers, bad names), correctness bugs tests catch (DRY violations, off-by-one, sloppy error handling), and security defects a scanner finds (SCA, SAST, DAST). Most don't crash on their own -- they make the stateful bugs harder to see, and the worst of them become vulnerabilities.",
    "essence": "Code smells suggest software defects; software defects suggest vulnerabilities. These are the smells everyone makes.",
    "practice": "Unlike the three stateful families, these defects rarely crash on their own — they're the everyday smells that make the stateful bugs harder to see, and the worst of them harden into vulnerabilities. Read them as a 3×3 by how each is caught: a reviewer or linter spots the readability smells (copy-pasta, magic numbers, bad names), tests and types catch the correctness bugs (DRY violations, off-by-one, sloppy error handling), and a scanner finds the security defects (dependency drift via SCA, injectable code via SAST, runtime exposure via DAST). It shows up as anything from a fix you must make in five places to a query a pen-tester turns into a breach — and the tools that make it easy are the most ordinary ones in the editor: copy & paste, a hard-coded literal, an empty catch, an unpinned dependency.",
    "symptoms": [
        "The same change has to be made in several places",
        "Numbers and strings with no explanation",
        "Names that have to be decoded before they're understood",
        "Errors swallowed by empty catch blocks",
        "\u201cWorks on my machine\u201d traced to a version drift",
        "A scanner flags an injectable query or an exposed endpoint",
    ],
    "causes": [
        "copy-pasta & DRY violations",
        "magic numbers and strings",
        "off-by-one boundaries",
        "naming problems",
        "swallowed or fail-open error handling",
        "unpinned, unscanned dependencies",
        "untrusted input reaching a dangerous sink",
    ],
    "tools_hurt": [
        "copy & paste",
        "hard-coded literals",
        "abbreviations & single letters",
        "empty catch / except blocks",
        "unpinned dependencies",
        "string-built SQL, shells, and paths",
    ],
    "philosophies": [
        {"name": "DRY: Don't Repeat Yourself", "url": "https://en.wikipedia.org/wiki/Don%27t_repeat_yourself"},
        {"name": "Refactoring Guru: Code Smells", "url": "https://refactoring.guru/refactoring/smells"},
        {"name": "OWASP Top Ten", "url": "https://owasp.org/www-project-top-ten/"},
        {"name": "Semantic Versioning", "url": "https://semver.org/"},
        {"name": "Lockfiles for all", "url": "https://classic.yarnpkg.com/blog/2016/11/24/lockfiles-for-all/"},
    ],
    "two_hard_note": "naming things",
    # Good names have these qualities (slide 18)
    "name_qualities": {
        "title": "Good names have these qualities",
        "items": [
            ("Explicit", "say what it is, not what it abbreviates"),
            ("Grokable", "understood at a glance, in context"),
            ("Consistent", "the same idea wears the same word everywhere"),
            ("Meaningful", "carries intent, not implementation"),
            ("Pronounceable", "you can say it out loud in a standup"),
            ("Intentional", "chosen, not whatever was typed first"),
            ("Namespaced", "scoped so two ideas don't collide"),
        ],
        "rule": "Objects & classes are noun phrases. Methods & functions are verb phrases.",
    },
    # Nine constituent common defects, read as a 3x3 by how each is caught:
    # readability (reviewer/linter), correctness (tests/types), security (scanner).
    # Each child has its own page, a gray text Lottie, and >=3 worked examples.
    "defects": [
        {
            "slug": "copy-pasta",
            "cwes": [
                {"id": "CWE-1041", "name": "Use of Redundant Code", "url": "https://cwe.mitre.org/data/definitions/1041.html"},
            ],
            "name": "Copy-Pasta Mistakes",
            "tagline": "the same logic pasted until the copies disagree",
            "essence": "Every paste is a future bug: one copy gets fixed, the others rot.",
            "description": "Duplicated logic drifts apart. A fix applied to one copy and missed in the others is the most common way a closed bug quietly reopens.",
            "examples": [
                {
                    "title": "Validation That Drifts",
                    "intent": "The same check pasted in two places. Now a fix is two fixes -- and you'll miss one.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: the same validation, copied and drifting apart
if (!email.includes("@") || email.length > 254) reject(email);
// ...200 lines later...
if (!addr.includes("@") || addr.length > 255) reject(addr);  // 254? 255?

// RIGHT: one source of truth
const isValidEmail = (e) => e.includes("@") && e.length <= 254;
if (!isValidEmail(email)) reject(email);""",
                         "explanation": "Copies drift. The pasted check used 255, the original 254 -- a bug born the moment someone duplicated it.",
                         "output": "copies: two rules that disagree by one\nshared: one rule, fixed once, applied everywhere"},
                    ],
                },
                {
                    "title": "Auth Copied Per Endpoint",
                    "intent": "Copy-pasta is most dangerous where it guards access -- one forgotten copy is an open door.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: three endpoints, three hand-rolled auth checks
def view_a(req):
    if not req.user or req.user.role != "admin": return deny()
def view_b(req):
    if not req.user or req.user.role != "admin": return deny()

# RIGHT: extract the rule into a decorator
def admin_only(view):
    def wrapped(req):
        if not req.user or req.user.role != "admin":
            return deny()
        return view(req)
    return wrapped""",
                         "explanation": "Auth is exactly where copy-pasta turns into a vulnerability -- one forgotten copy is an open door.",
                         "output": "copied:    miss one endpoint -> privilege escalation\ndecorator: the rule can't be forgotten"},
                    ],
                },
                {
                    "title": "The Pasted Constant",
                    "intent": "A duplicated literal gets 'tuned' in one place only, and the copies silently disagree.",
                    "languages": [
                        {"name": "TypeScript", "slug": "typescript", "code":
"""// SMELL: the same timeout pasted, then one gets tweaked
setTimeout(retry, 30000);   // payments.ts
setTimeout(retry, 3000);    // billing.ts  -- typo? or intentional?

// RIGHT: one named constant, imported everywhere
export const RETRY_MS = 30_000;
setTimeout(retry, RETRY_MS);""",
                         "explanation": "Duplicated literals drift independently. A single exported constant has one value and one place to change it.",
                         "output": "pasted: 30000 vs 3000 -- which is right?\nshared: RETRY_MS, identical everywhere"},
                    ],
                },
            ],
        },
        {
            "slug": "dry-rule-of-three",
            "cwes": [
                {"id": "CWE-1041", "name": "Use of Redundant Code", "url": "https://cwe.mitre.org/data/definitions/1041.html"},
                {"id": "CWE-710", "name": "Improper Adherence to Coding Standards", "url": "https://cwe.mitre.org/data/definitions/710.html"},
            ],
            "name": "DRY -- Don't Repeat Yourself",
            "tagline": "extract on the third repeat, not the first",
            "essence": "Two copies is a coincidence; three is a pattern that has earned a name.",
            "description": "DRY says every piece of knowledge has one authoritative home. The rule of three tempers it: don't abstract until the third occurrence, or you'll build the wrong abstraction and couple things that should move apart.",
            "examples": [
                {
                    "title": "The Rule of Three",
                    "intent": "When to extract: the first copy is fine, the second is a coincidence, the third confirms a pattern.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# 1st time: just write it.
price_a = qty_a * unit_a * (1 - discount_a)
# 2nd time: a coincidence -- resist abstracting yet.
price_b = qty_b * unit_b * (1 - discount_b)
# 3rd time: NOW extract -- the pattern is proven.
def line_total(qty, unit, discount):
    return qty * unit * (1 - discount)""",
                         "explanation": "Two occurrences may be coincidence; the third confirms a real pattern. Extracting too early guesses the wrong shape.",
                         "output": "early: an abstraction that fights the 3rd case\nthree: extract once the pattern is proven"},
                    ],
                },
                {
                    "title": "Knowledge, Not Code",
                    "intent": "Two snippets that look identical can encode different decisions. DRY-ing them couples the unrelated.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// These LOOK identical, but encode two different policies:
const taxRate = (amt) => amt * 0.0825;   // sales tax (law)
const tipRate = (amt) => amt * 0.0825;   // suggested tip (today)

// WRONG-DRY: merge them and they change together by accident
const rate = (amt) => amt * 0.0825;      // now tip can never differ from tax

// RIGHT: same value, different KNOWLEDGE -- keep them apart
const SALES_TAX = 0.0825, DEFAULT_TIP = 0.0825;""",
                         "explanation": "DRY is about single sources of knowledge, not identical text. Merging coincidentally-equal code couples decisions that should evolve independently.",
                         "output": "merged:   changing tip silently changes tax\nseparate: each rate evolves on its own"},
                    ],
                },
                {
                    "title": "Single Source of Truth",
                    "intent": "Knowledge with one home cannot disagree with itself; copies eventually do.",
                    "languages": [
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: the same rule compiled in three files, subtly drifted
var reA = regexp.MustCompile(`^\\d{3}-\\d{4}$`)
var reB = regexp.MustCompile(`^\\d{3}-\\d{3}$`)  // typo, drifted

// RIGHT: one exported pattern, defined once
var PhoneRE = regexp.MustCompile(`^\\d{3}-\\d{4}$`)""",
                         "explanation": "A rule with a single definition cannot contradict itself. The drifted copy is a bug the original never had.",
                         "output": "copies: two patterns, one concept\nshared: PhoneRE, defined once"},
                    ],
                },
            ],
        },
        {
            "slug": "magic-numbers",
            "cwes": [
                {"id": "CWE-547", "name": "Use of Hard-coded, Security-relevant Constants", "url": "https://cwe.mitre.org/data/definitions/547.html"},
                {"id": "CWE-798", "name": "Use of Hard-coded Credentials", "url": "https://cwe.mitre.org/data/definitions/798.html"},
            ],
            "name": "Magic Numbers",
            "tagline": "literals with no name and no explanation",
            "essence": "A number the reader has to decode is a comment you forgot to write.",
            "description": "Bare literals -- 86400, 0.0825, 200, \"admin\" -- force the reader to reconstruct intent and scatter a value across the codebase with no single place to change it.",
            "examples": [
                {
                    "title": "Name the Intent",
                    "intent": "A literal with no name. The reader has to guess what 86400 or 0.0825 means.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: what is 86400? why 0.0825? why 3?
if (now - created > 86400) expire();
const total = price * 1.0825;
if (retries > 3) giveUp();

// RIGHT: name the intent
const SECONDS_PER_DAY = 86_400;
const SALES_TAX = 0.0825;
const MAX_RETRIES = 3;
if (now - created > SECONDS_PER_DAY) expire();""",
                         "explanation": "A named constant documents the value and gives it one place to change. The number stops being a riddle.",
                         "output": "magic: reader greps for 86400 across the repo\nnamed: SECONDS_PER_DAY reads itself"},
                    ],
                },
                {
                    "title": "Status Codes & Thresholds",
                    "intent": "Bare codes and limits live only in the author's head; enums make the legal set explicit.",
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: 200, 404, 7 -- meaning lives only in your head
if (status == 200) ok();
if (daysOpen > 7) escalate();

// RIGHT: enums and named constants
if (status == HttpStatus.OK) ok();
static final int ESCALATION_THRESHOLD_DAYS = 7;
if (daysOpen > ESCALATION_THRESHOLD_DAYS) escalate();""",
                         "explanation": "Enums and constants make illegal values harder to write and legal ones self-explaining.",
                         "output": "magic: 200 could be anything\nenum:  HttpStatus.OK is unambiguous"},
                    ],
                },
                {
                    "title": "Magic Strings Too",
                    "intent": "A literal string is a magic number in disguise -- invisible to the type checker, easy to mistype.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: bare strings are magic numbers in disguise
if user.role == "admin":          # typo "Admin" silently denies
    grant()

# RIGHT: a named enum the type checker can police
class Role(str, Enum):
    ADMIN = "admin"

if user.role == Role.ADMIN:
    grant()""",
                         "explanation": "A literal string is as magic as a literal number: the type checker can't see it and a typo fails silently. An enum names the legal set.",
                         "output": "string: 'Admin' != 'admin', silent deny\nenum:   one canonical value"},
                    ],
                },
            ],
        },
        {
            "slug": "off-by-one-errors",
            "cwes": [
                {"id": "CWE-193", "name": "Off-by-one Error", "url": "https://cwe.mitre.org/data/definitions/193.html"},
                {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html"},
            ],
            "name": "Off-by-One Errors",
            "tagline": "the boundary every stateful tool gets wrong",
            "essence": "Off-by-one isn't one language's bug -- it's what every counter, cursor, and pointer drifts into.",
            "description": "The fencepost bug: a bound one too big or one too small, so the last element is dropped or one past the end is read. It is the boundary defect every stateful tool can produce -- shown here generic, then via a different stateful mechanism, across nine languages.",
            "two_hard_note": "off-by-one errors",
            "examples": [
                {
                    "title": "Generic Off-by-One",
                    "intent": "The plain fencepost: an inclusive bound or a len-1 where the language already gives you the right range.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// WRONG: length - 1 silently skips the last element
for (let i = 0; i < arr.length - 1; i++) send(arr[i]);

// RIGHT: iterate the whole range
for (const item of arr) send(item);""",
                         "explanation": "Subtracting 1 from the bound is the classic fencepost error. for...of removes the bound you can get wrong.",
                         "output": "input: [a, b, c, d, e]\nwrong: a b c d   (e is lost)\nright: a b c d e"},
                        {"name": "TypeScript", "slug": "typescript", "code":
"""// WRONG: <= on a half-open length reads one past the end
let total = 0;
for (let i = 0; i <= xs.length; i++) total += xs[i];  // xs[length] is undefined

// RIGHT: strict <, or reduce and never index at all
const total = xs.reduce((a, b) => a + b, 0);""",
                         "explanation": "An array of length n has no index n. <= reads xs[length] (undefined), turning the sum into NaN.",
                         "output": "wrong: total === NaN\nright: the exact sum"},
                        {"name": "Python", "slug": "python", "code":
"""items = [1, 2, 3, 4, 5]

# WRONG: range() is already exclusive of the end
for i in range(len(items) - 1):
    process(items[i])

# RIGHT: iterate the object, not an index
for item in items:
    process(item)""",
                         "explanation": "range(n) already stops before n. Subtracting 1 on top of that drops the final element.",
                         "output": "wrong: 1 2 3 4\nright: 1 2 3 4 5"},
                        {"name": "Ruby", "slug": "ruby", "code":
"""# WRONG: .. is inclusive, so it reads one past the end
(0..arr.length).each { |i| puts arr[i] }   # last is arr[length] -> nil

# RIGHT: ... is exclusive (or just iterate the values)
(0...arr.length).each { |i| puts arr[i] }
arr.each { |v| puts v }""",
                         "explanation": "Ruby's .. range includes the end; ... excludes it. Indexing with the inclusive range reads one element too far (nil).",
                         "output": "wrong: every value, then a blank line (nil)\nright: every value, once"},
                        {"name": "Go", "slug": "go", "code":
"""// WRONG: <= indexes one past the slice -> panic
for i := 0; i <= len(xs); i++ {
    fmt.Println(xs[i])
}

// RIGHT: use <, or range
for _, x := range xs { fmt.Println(x) }""",
                         "explanation": "len(xs) is one past the last valid index. <= panics with index out of range.",
                         "output": "wrong: panic: index out of range [len]\nright: every element"},
                        {"name": "Rust", "slug": "rust", "code":
"""// WRONG: 0..=n is inclusive -> indexes v.len()
for i in 0..=v.len() {
    println!("{}", v[i]);   // panics on the last i
}

// RIGHT: 0..n is half-open (or just iterate)
for x in &v { println!("{}", x); }""",
                         "explanation": "0..=v.len() includes v.len(), which is out of bounds. 0..v.len() is half-open; iterating avoids the index entirely.",
                         "output": "wrong: panic: index out of bounds\nright: every element"},
                        {"name": "Java", "slug": "java", "code":
"""// WRONG: <= walks one past the end -> exception
for (int i = 0; i <= arr.length; i++) process(arr[i]);

// RIGHT: use < not <=
for (int i = 0; i < arr.length; i++) process(arr[i]);""",
                         "explanation": "<= reads index 5 of a length-5 array and throws ArrayIndexOutOfBoundsException.",
                         "output": "wrong: ArrayIndexOutOfBoundsException: 5\nright: 1 2 3 4 5"},
                        {"name": "Kotlin", "slug": "kotlin", "code":
"""// WRONG: 0..size is inclusive of size
for (i in 0..list.size) print(list[i])   // IndexOutOfBounds on size

// RIGHT: 0 until size is half-open
for (i in 0 until list.size) print(list[i])""",
                         "explanation": "Kotlin's .. is inclusive, so 0..size visits the invalid index size. until is the half-open form.",
                         "output": "wrong: IndexOutOfBoundsException\nright: every element"},
                        {"name": "C", "slug": "c", "code":
"""/* WRONG: <= reads one past the array -- undefined behavior */
for (int i = 0; i <= 5; i++) process(arr[i]);   /* arr[5] is garbage */

/* RIGHT */
for (int i = 0; i < 5; i++) process(arr[i]);""",
                         "explanation": "In C the off-by-one does not throw; it reads adjacent memory. Silent corruption is worse than a crash.",
                         "output": "wrong: 1 2 3 4 5 <garbage>\nright: 1 2 3 4 5"},
                    ],
                },
                {
                    "title": "Off-by-One With a Stateful Tool",
                    "intent": "The same boundary bug, but produced by a different stateful mechanism in each language -- a counter, a cursor, an operator, a pointer.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// STATEFUL TOOL: the ++ operator mutates i inside the test
let i = 0;
while (i++ < arr.length) {     // i is advanced even on the final test
  send(arr[i]);                // skips arr[0], reads arr[length] (undefined)
}

// RIGHT: separate the test from the mutation
for (let i = 0; i < arr.length; i++) send(arr[i]);""",
                         "explanation": "The ++ operator mutates i as a side effect of the comparison, so the index and the bound disagree by one.",
                         "output": "wrong: skips first, reads one past the end\nright: every element, once"},
                        {"name": "TypeScript", "slug": "typescript", "code":
"""// STATEFUL TOOL: a mutable cursor advanced by hand
let cursor = 0;
const out: string[] = [];
while (cursor <= items.length) {   // <= overshoots by one
  out.push(items[cursor]);         // last push is items[length] -> undefined
  cursor += 1;
}

// RIGHT: let the language own the bound
const out = items.map((x) => x);""",
                         "explanation": "A hand-mutated let cursor with <= reads one slot past the array; let-binding the iteration removes the cursor you can get wrong.",
                         "output": "wrong: trailing undefined in out\nright: a clean copy of items"},
                        {"name": "Python", "slug": "python", "code":
"""# STATEFUL TOOL: a hand-mutated counter and a fixed bound drift apart
i = 0
total = 0
while i <= len(nums):        # <= runs one extra iteration
    total += nums[i]         # IndexError on the last pass
    i += 1

# RIGHT: iterate the object; no counter to mutate
total = sum(nums)""",
                         "explanation": "The mutable counter i and the <= bound are maintained separately, so they fall out of step by one.",
                         "output": "wrong: IndexError: list index out of range\nright: the exact sum"},
                        {"name": "Ruby", "slug": "ruby", "code":
"""# STATEFUL TOOL: a mutated offset added to the loop index
offset = 1
arr.each_with_index do |_, i|
  puts arr[i + offset]   # last read is arr[size] -> nil
end

# RIGHT: no offset; iterate the values directly
arr.each { |v| puts v }""",
                         "explanation": "Mutating an offset and adding it to the index pushes every access one past where you think you are (returns nil at the end).",
                         "output": "wrong: shifted by one, trailing nil\nright: each value in place"},
                        {"name": "Go", "slug": "go", "code":
"""// STATEFUL TOOL: a manual index drives the loop past the slice
i := 0
for i <= len(xs) {           // <= indexes one past the slice
    fmt.Println(xs[i])       // panic: index out of range
    i++
}

// RIGHT: range owns the bound
for _, x := range xs { fmt.Println(x) }""",
                         "explanation": "Driving the loop with a mutable index and <= len(xs) reads one past the slice; range removes the manual bound.",
                         "output": "wrong: panic: index out of range\nright: every element"},
                        {"name": "Rust", "slug": "rust", "code":
"""// STATEFUL TOOL: a `mut` counter advanced by hand
let mut i = 0;
while i <= v.len() {          // <= goes one past the end
    println!("{}", v[i]);     // panics: index out of bounds
    i += 1;
}

// RIGHT: iterate; the bound is not yours to manage
for x in &v { println!("{}", x); }""",
                         "explanation": "A let mut index incremented by hand with <= indexes v.len(), which is one past the last valid index.",
                         "output": "wrong: panic: index out of bounds\nright: every element"},
                        {"name": "Java", "slug": "java", "code":
"""// STATEFUL TOOL: manual for-loop control mutates the index
int n = list.size();
for (int i = 0; i <= n; i++) {   // <= reads index size -> exception
    System.out.println(list.get(i));
}

// RIGHT: half-open bound, no manual index
list.forEach(System.out::println);""",
                         "explanation": "Hand-rolled for-loop control with <= walks one past the last index; for-each hands the bound to the collection.",
                         "output": "wrong: IndexOutOfBoundsException\nright: every element"},
                        {"name": "Kotlin", "slug": "kotlin", "code":
"""// STATEFUL TOOL: a `var` index over an inclusive range
var i = 0
while (i <= list.size) {       // <= visits the invalid index size
    println(list[i])           // IndexOutOfBoundsException
    i += 1
}

// RIGHT: `until` is half-open; no var to mismanage
for (x in list) println(x)""",
                         "explanation": "A mutated var index with <= list.size reads one past the end. until is the half-open range that can't.",
                         "output": "wrong: IndexOutOfBoundsException\nright: every element"},
                        {"name": "C", "slug": "c", "code":
"""/* STATEFUL TOOL: a mutated pointer walks one element too far */
int *p = arr;
for (int i = 0; i <= 5; i++) {   /* <= and *p++ both overshoot */
    printf("%d ", *p++);          /* reads arr[5]: past the array */
}

/* RIGHT: stop before the end */
for (int i = 0; i < 5; i++) printf("%d ", arr[i]);""",
                         "explanation": "The mutated pointer *p++ combined with <= reads one past the array -- undefined behavior, not a clean crash.",
                         "output": "wrong: 1 2 3 4 5 <garbage>\nright: 1 2 3 4 5"},
                    ],
                },
            ],
        },
        {
            "slug": "naming-problems",
            "cwes": [
                {"id": "CWE-1099", "name": "Inconsistent Naming Conventions for Identifiers", "url": "https://cwe.mitre.org/data/definitions/1099.html"},
                {"id": "CWE-1078", "name": "Inappropriate Source Code Style or Formatting", "url": "https://cwe.mitre.org/data/definitions/1078.html"},
            ],
            "name": "Naming Problems",
            "tagline": "names that have to be decoded before they're understood",
            "essence": "Good names are the cheapest documentation you'll ever write.",
            "description": "Opaque abbreviations, inconsistent vocabulary, and classes named like verbs force the reader to reconstruct meaning the code should have stated outright.",
            "examples": [
                {
                    "title": "Opaque Abbreviations",
                    "intent": "Single letters and abbreviations make the reader rebuild meaning from the body.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: what does this do? what is d? what is x?
def proc(d, x):
    return [i for i in d if i[1] > x]

# RIGHT: names carry the intent
def users_older_than(users, min_age):
    return [u for u in users if u.age > min_age]""",
                         "explanation": "proc/d/x force the reader to reconstruct meaning from the body. Verb-phrase functions and noun-phrase data tell the story.",
                         "output": "proc(d, x):              opaque\nusers_older_than(...):   reads like a sentence"},
                    ],
                },
                {
                    "title": "Inconsistent Vocabulary",
                    "intent": "Three words for one idea is three things to memorize and a daily source of confusion.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: inconsistent words for the same idea
getUser(); fetchClient(); loadCustomer();   // user? client? customer?

// RIGHT: one consistent vocabulary, verb + noun
getCustomer(); listCustomers(); updateCustomer();""",
                         "explanation": "Consistency is a naming quality. One concept should wear one word everywhere it appears.",
                         "output": "mixed:      three names, one concept, daily confusion\nconsistent: predictable API, less to memorize"},
                    ],
                },
                {
                    "title": "Nouns and Verbs",
                    "intent": "Objects are noun phrases; methods are verb phrases; booleans read as yes/no questions.",
                    "languages": [
                        {"name": "TypeScript", "slug": "typescript", "code":
"""// SMELL: a class named like an action, a method named like a thing
class DoValidation {                 // a class is a noun, not a verb
  result(u: User): boolean { return u.age >= 18 }   // method should be a verb
}

// RIGHT: noun-phrase type, verb-phrase method, boolean as a question
class AgePolicy {
  isAdult(u: User): boolean { return u.age >= 18 }
}""",
                         "explanation": "Objects and classes are noun phrases; methods and functions are verb phrases; booleans read as questions (isAdult).",
                         "output": "DoValidation.result(): backwards\nAgePolicy.isAdult():   reads like a sentence"},
                    ],
                },
            ],
        },
        {
            "slug": "error-handling",
            "cwes": [
                {"id": "CWE-755", "name": "Improper Handling of Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/755.html"},
                {"id": "CWE-390", "name": "Detection of Error Condition Without Action", "url": "https://cwe.mitre.org/data/definitions/390.html"},
                {"id": "CWE-391", "name": "Unchecked Error Condition", "url": "https://cwe.mitre.org/data/definitions/391.html"},
                {"id": "CWE-209", "name": "Generation of Error Message Containing Sensitive Information", "url": "https://cwe.mitre.org/data/definitions/209.html"},
            ],
            "name": "Poor Error Handling",
            "tagline": "swallowed failures that surface later as something worse",
            "essence": "An error you ignore becomes a crash, a corruption, or a breach.",
            "description": "Empty catches, ignored return codes, and fail-open defaults turn a small handled error into a silent failure. The deck's lesson lands here: software that crashes becomes a Denial of Service, and a security check that fails open becomes an authorization bypass.",
            "examples": [
                {
                    "title": "The Empty Catch",
                    "intent": "Swallowing an exception turns a clear failure into a confusing one, fifty lines away.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: the error is swallowed; the program limps on corrupt
let user;
try {
  user = JSON.parse(body);
} catch (e) { /* ignore */ }     // user is undefined, used later

// RIGHT: handle it, or fail loudly -- never swallow
try {
  user = JSON.parse(body);
} catch (e) {
  return res.status(400).json({ error: "invalid body" });
}""",
                         "explanation": "An empty catch defers the failure to a later, more confusing crash. Either handle the error meaningfully or let it propagate.",
                         "output": "swallow: undefined access crashes elsewhere\nhandle:  a clean 400 at the source"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: catch (Exception) hides bugs you never meant to catch
try {
    transfer(amount);
} catch (Exception e) {           // also catches NPEs and logic bugs
    log.info("retrying");         // and discards the stack trace
}

// RIGHT: catch the specific, recoverable exception; keep the cause
try {
    transfer(amount);
} catch (InsufficientFundsException e) {
    notifyUser(e);
}""",
                         "explanation": "Catch-all blocks bury real defects and throw away the stack trace. Catch the narrowest exception you can actually recover from.",
                         "output": "broad:  a logic bug looks like a retry\nnarrow: only the expected failure is handled"},
                    ],
                },
                {
                    "title": "Ignored Return Codes",
                    "intent": "Languages that return errors as values let you discard them -- and the discard is a decision to be surprised later.",
                    "languages": [
                        {"name": "Go", "slug": "go", "code":
"""// SMELL: the error is discarded with _
f, _ := os.Open(path)       // f may be nil
defer f.Close()             // nil pointer dereference

// RIGHT: check every error the moment it returns
f, err := os.Open(path)
if err != nil { return err }
defer f.Close()""",
                         "explanation": "Discarding err with _ defers the failure to a nil dereference. Go returns errors as values precisely so you check them at the call site.",
                         "output": "ignored: panic on nil f\nchecked: the failure is handled where it happens"},
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: malloc can return NULL; the unchecked write corrupts memory */
char *buf = malloc(n);
memcpy(buf, src, n);        /* segfault or worse if buf == NULL */

/* RIGHT: check the return code before you trust it */
char *buf = malloc(n);
if (!buf) return -1;
memcpy(buf, src, n);""",
                         "explanation": "In C the only signal is the return value. An unchecked allocation failure becomes silent corruption.",
                         "output": "unchecked: undefined behavior on OOM\nchecked:   a clean error path"},
                    ],
                },
                {
                    "title": "Fail-Open vs Fail-Closed",
                    "intent": "When a security check errors, the safe default is to deny. Fail-open turns an outage into a bypass.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: on error, default to allowing the request (fail OPEN)
def is_authorized(token):
    try:
        return verify(token)
    except Exception:
        return True            # a verifier outage grants everyone access

# RIGHT: on error, deny (fail CLOSED)
def is_authorized(token):
    try:
        return verify(token)
    except Exception:
        return False""",
                         "explanation": "When a security check errors, the safe default is to deny. Fail-open turns an outage into an authorization bypass -- the defect becoming a vulnerability.",
                         "output": "fail-open:   verifier down -> everyone is admin\nfail-closed: verifier down -> nobody slips through"},
                    ],
                },
            ],
        },
        {
            "slug": "dependency-issues",
            "cwes": [
                {"id": "CWE-1395", "name": "Dependency on Vulnerable Third-Party Component", "url": "https://cwe.mitre.org/data/definitions/1395.html"},
                {"id": "CWE-1104", "name": "Use of Unmaintained Third Party Components", "url": "https://cwe.mitre.org/data/definitions/1104.html"},
                {"id": "CWE-937", "name": "Using Components with Known Vulnerabilities", "url": "https://cwe.mitre.org/data/definitions/937.html"},
            ],
            "name": "Dependency Issues (SCA)",
            "tagline": "unpinned, unscanned, and unreproducible supply chains",
            "essence": "Code you didn't write still ships in your build -- pin it, lock it, and scan it.",
            "description": "Software Composition Analysis is about the code you depend on: unpinned versions make builds non-reproducible, missing lockfiles let the tree drift, and unscanned dependencies carry known CVEs straight into production.",
            "examples": [
                {
                    "title": "Pin the Tree",
                    "intent": "Caret ranges float to whatever published last; a committed lockfile makes installs byte-identical.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: caret ranges float to whatever published last
"dependencies": { "left-pad": "^1.0.0" }   // 1.0.0? 1.9.3? who knows

// RIGHT: commit the lockfile so installs are byte-identical
// package-lock.json / yarn.lock pins the entire tree.
// CI: `npm ci` (not `npm install`) installs exactly the lock.""",
                         "explanation": "^1.0.0 resolves differently over time. A committed lockfile makes every machine install the same tree -- the cure for 'works on my machine'.",
                         "output": "no lock: two devs, two dependency trees\nnpm ci:  identical install everywhere"},
                    ],
                },
                {
                    "title": "Pin the Versions (and Scan Them)",
                    "intent": "Unpinned requirements drift on every install; pinning freezes the build and lets a scanner check for CVEs.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: unpinned requirements drift on every install
requests
flask

# RIGHT: pin exact versions (and hashes), then scan them
requests==2.32.3   # produced by pip-compile (pip-tools)
flask==3.0.3
# CI: `pip-audit` flags pinned versions with known CVEs""",
                         "explanation": "Pinning exact versions freezes the build; an SCA tool (pip-audit, Dependabot) then checks those pins against vulnerability databases.",
                         "output": "unpinned: surprise upgrade breaks prod at 2am\npinned:   reproducible -- and auditable for CVEs"},
                    ],
                },
                {
                    "title": "Lock the Infrastructure",
                    "intent": "Infrastructure dependencies drift too; a committed provider lock keeps applies deterministic.",
                    "languages": [
                        {"name": "HCL", "slug": "hcl", "code":
"""# Terraform: same lesson, infrastructure edition
# SMELL: unconstrained provider -> drift between applies
terraform {
  required_providers { aws = {} }
}

# RIGHT: pin the provider and commit .terraform.lock.hcl
terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.40" }
  }
}""",
                         "explanation": "Infrastructure dependencies drift too. A committed provider lock keeps terraform apply reproducible across the team.",
                         "output": "unpinned: provider upgrades silently change infra\nlocked:   plan/apply are deterministic"},
                    ],
                },
            ],
        },
        {
            "slug": "injection-taint-flaws",
            "cwes": [
                {"id": "CWE-74", "name": "Improper Neutralization of Special Elements in Output Used by a Downstream Component ('Injection')", "url": "https://cwe.mitre.org/data/definitions/74.html"},
                {"id": "CWE-89", "name": "Improper Neutralization of Special Elements used in an SQL Command ('SQL Injection')", "url": "https://cwe.mitre.org/data/definitions/89.html"},
                {"id": "CWE-78", "name": "Improper Neutralization of Special Elements used in an OS Command ('OS Command Injection')", "url": "https://cwe.mitre.org/data/definitions/78.html"},
                {"id": "CWE-79", "name": "Improper Neutralization of Input During Web Page Generation ('Cross-site Scripting')", "url": "https://cwe.mitre.org/data/definitions/79.html"},
            ],
            "name": "Injection & Taint Flaws (SAST)",
            "tagline": "untrusted input flowing into a dangerous sink",
            "essence": "Static analysis traces a tainted source to a sink; the fix is to break the flow.",
            "description": "SAST reads the source you wrote and follows untrusted data into dangerous sinks -- SQL, the shell, the filesystem -- plus hardcoded secrets and weak crypto. These are the defects a static scanner flags before the code ever runs.",
            "examples": [
                {
                    "title": "SQL Injection",
                    "intent": "Untrusted input concatenated into a query is the classic source-to-sink flow.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: untrusted input concatenated into SQL (the classic sink)
q = "SELECT * FROM users WHERE name = '" + name + "'"
db.execute(q)               # name = "' OR '1'='1" dumps the table

# RIGHT: parameterize -- data never becomes code
db.execute("SELECT * FROM users WHERE name = ?", (name,))""",
                         "explanation": "SAST traces the tainted name from an untrusted source into the query sink. Parameterized queries keep data and code separate.",
                         "output": "concat: ' OR '1'='1 returns every row\nparam:  the input is data, never SQL"},
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: a string-built query -> injectable
String q = "SELECT * FROM acct WHERE id = " + req.getParameter("id");
stmt.executeQuery(q);

// RIGHT: PreparedStatement binds the value safely
PreparedStatement ps = conn.prepareStatement(
    "SELECT * FROM acct WHERE id = ?");
ps.setInt(1, Integer.parseInt(req.getParameter("id")));""",
                         "explanation": "The request parameter is tainted source; the query is the sink. Binding parameters (and parsing to int) breaks the injection.",
                         "output": "concat:   1 OR 1=1 leaks all accounts\nprepared: only a valid id is accepted"},
                    ],
                },
                {
                    "title": "Command Injection & Path Traversal",
                    "intent": "Input handed to a shell or a file path escapes into commands and directories you never intended.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: user input handed straight to a shell
exec(`convert ${userFile} out.png`);   // userFile = "x; rm -rf /"

// RIGHT: no shell; pass arguments as an array
execFile("convert", [userFile, "out.png"]);""",
                         "explanation": "String-interpolating input into a shell command is a sink. execFile bypasses the shell, so metacharacters are inert.",
                         "output": "exec:     '; rm -rf /' runs as a command\nexecFile: the filename is just an argument"},
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: user-controlled path escapes the intended directory
path = os.path.join(UPLOADS, request.args["name"])   # name = "../../etc/passwd"
return open(path).read()

# RIGHT: resolve and confirm it stays inside UPLOADS
full = os.path.realpath(os.path.join(UPLOADS, name))
if not full.startswith(UPLOADS):
    abort(403)
return open(full).read()""",
                         "explanation": "../ sequences in tainted input walk outside the upload dir. Resolve the real path and verify containment before opening.",
                         "output": "naive:   reads /etc/passwd\nguarded: 403 on any escape attempt"},
                    ],
                },
                {
                    "title": "Hardcoded Secrets",
                    "intent": "A credential committed in source leaks through the repo and can't be rotated without a deploy.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a secret committed in source -- SAST and secret scanners flag this
API_KEY = "sk_live_51H8xQ2eZvKf9pQ"      # now in git history forever

# RIGHT: read from the environment / a secret manager
API_KEY = os.environ["API_KEY"]""",
                         "explanation": "Hardcoded credentials are a static-analysis red flag: they leak through the repo's history and can't be rotated without a code change.",
                         "output": "hardcoded: leaked the moment the repo is shared\nenv var:   rotate without touching code"},
                    ],
                },
            ],
        },
        {
            "slug": "runtime-integration-flaws",
            "cwes": [
                {"id": "CWE-352", "name": "Cross-Site Request Forgery (CSRF)", "url": "https://cwe.mitre.org/data/definitions/352.html"},
                {"id": "CWE-918", "name": "Server-Side Request Forgery (SSRF)", "url": "https://cwe.mitre.org/data/definitions/918.html"},
                {"id": "CWE-601", "name": "URL Redirection to Untrusted Site ('Open Redirect')", "url": "https://cwe.mitre.org/data/definitions/601.html"},
                {"id": "CWE-200", "name": "Exposure of Sensitive Information to an Unauthorized Actor", "url": "https://cwe.mitre.org/data/definitions/200.html"},
            ],
            "name": "Runtime & Integration Flaws (DAST)",
            "tagline": "defects you only see when the app is running",
            "essence": "DAST pokes the live app from outside; these bugs hide in behavior, not source.",
            "description": "Dynamic testing exercises the running system from the perimeter: reflected XSS, crash-under-malformed-input, and security misconfiguration. They are invisible to a static read of any single function -- you have to run the thing to see them.",
            "examples": [
                {
                    "title": "Reflected XSS",
                    "intent": "User input echoed into HTML unescaped runs as script -- a dynamic scan watches the alert fire.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: user input echoed into HTML unescaped
app.get("/hi", (req, res) =>
  res.send(`<h1>Hello ${req.query.name}</h1>`));   // name = <script>...

// RIGHT: escape on output (or use an auto-escaping template engine)
app.get("/hi", (req, res) =>
  res.send(`<h1>Hello ${escapeHtml(req.query.name)}</h1>`));""",
                         "explanation": "A dynamic scan submits <script> and watches it execute -- a defect invisible in a static read of one function. Escape untrusted data on output.",
                         "output": "raw:     <script>alert(1)</script> runs\nescaped: shown as literal text"},
                    ],
                },
                {
                    "title": "Crash Under Malformed Input -> DoS",
                    "intent": "An unbounded, unvalidated parameter is a denial-of-service waiting to be fuzzed.",
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: unhandled parse + unbounded work takes the worker down
@app.post("/calc")
def calc():
    n = int(request.form["n"])     # "abc" -> ValueError -> 500, repeatable
    return str(fib(n))             # n = 10_000_000 -> CPU exhaustion

# RIGHT: validate and bound before doing work
n = request.form.get("n", "")
if not n.isdigit() or int(n) > 1000:
    abort(400)""",
                         "explanation": "DAST fuzzes the endpoint and finds inputs that crash or hang it. Validate and bound every parameter -- software that crashes becomes a Denial of Service.",
                         "output": "naive:   'abc' -> 500; huge n -> hung worker\nbounded: a clean 400, work stays cheap"},
                    ],
                },
                {
                    "title": "Security Misconfiguration",
                    "intent": "Verbose errors and permissive defaults shipped to production are exposures a scanner finds at the perimeter.",
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: debug errors and wildcard CORS shipped to production
app.use(errorhandler());              // leaks stack traces to clients
app.use(cors({ origin: "*" }));       // any site can call your API

// RIGHT: prod-safe errors; lock the origin
if (PROD) app.use(genericErrorPage());
app.use(cors({ origin: ["https://app.example.com"] }));""",
                         "explanation": "Misconfiguration is what DAST catches at the perimeter: verbose errors leak internals and wildcard CORS invites abuse. Ship hardened defaults.",
                         "output": "open:   stack traces + any-origin requests\nlocked: generic errors, allowlisted origins"},
                        {"name": "HCL", "slug": "hcl", "code":
"""# SMELL: a storage bucket left open to the world
resource "aws_s3_bucket_acl" "data" {
  acl = "public-read"           # anyone can list and read objects
}

# RIGHT: private by default; grant access explicitly
resource "aws_s3_bucket_acl" "data" {
  acl = "private"
}""",
                         "explanation": "Infrastructure misconfiguration is the same defect at the cloud layer -- a public bucket is a runtime exposure a config scan or DAST will flag.",
                         "output": "public-read: objects exposed to the internet\nprivate:     access only via explicit grants"},
                    ],
                },
            ],
        },
    ],
})

# ─────────────────────────────────────────────────────────────────────────────
# 5. COMPOUND STATE  (emergent defects from pairs of the three stateful families)
# ─────────────────────────────────────────────────────────────────────────────
DEFECTS.append({
    "id": "compound-defects",
    # Not a CWE pillar but a CWE *view*: defects made of other defects.
    "cwe_pillar": {"id": "CWE-678", "name": "Composites", "url": "https://cwe.mitre.org/data/definitions/678.html"},
    "name": "Compound Defects",
    "short_name": "Compound",
    "category": "compound",
    "tagline": "defects that emerge from other defects",
    "color_primary": "C9A227",
    "color_secondary": "E6CF6A",
    "color_deep": "8A6D1B",
    "description": "Bugs with no single line number: they emerge where two stateful defects meet. You can only name them by their symptom, because the cause lives in the interaction.",
    "essence": "Two correct-looking defects combine into one that classifies by symptom, not by root cause.",
    "practice": "These three defects are not roots but symptoms. Each is the intersection of two of the stateful families — mutated, shared, and unstructured — and you recognize it by how it feels: an answer that won't reproduce, a crash that only happens live, data that's quietly, permanently wrong. MITRE calls combinations like these Compound weaknesses: Chains (one defect creates the conditions for the next) and Composites (several must occur at once). Fix one ingredient and the smell often persists, because the other half is still there.",
    "symptoms": [
        "The bug cannot be reproduced consistently",
        "Whack-a-mole fixes that won't stay fixed",
        "Crashes first seen live, in the demo",
        "Permanent damage or information exposure",
    ],
    "causes": [
        "mutated × shared state (nondeterminism)",
        "mutated × unstructured state (runtime errors)",
        "shared × unstructured state (data corruption)",
        "an entropy source meeting an observation gap",
        "unvalidated input mutated into illegal state",
        "bad data crossing a shared boundary",
    ],
    "tools_hurt": [
        "concurrency over mutable state",
        "caches over hidden inputs",
        "external input without a contract",
        "rolling deploys / mixed versions",
        "shared memory & deserialization",
        "trust across a boundary",
    ],
    "philosophies": [
        {"name": "MITRE: Chains and Composites", "url": "https://cwe.mitre.org/data/reports/chains_and_composites.html"},
        {"name": "Heisenbug", "url": "https://en.wikipedia.org/wiki/Heisenbug"},
        {"name": "Fallacies of Distributed Computing", "url": "https://en.wikipedia.org/wiki/Fallacies_of_distributed_computing"},
        {"name": "Parse, don't validate", "url": "https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/"},
    ],
    "pullquote": {
        "text": "A compound defect has no home. Point at the race and the fix is in the cache; point at the cache and the fix is in the clock. The bug lives in the space between two files that each look correct.",
        "attribution": "on debugging the unreproducible",
    },
    "two_hard_note": None,
    "defects": [
        {
            "slug": "nondeterminism",
            "color_primary": "FFE599",
            "color_secondary": "FFF2CC",
            "color_deep": "BF9000",
            "name": "Nondeterminism",
            "tagline": "mutated × shared — the answer changes between runs",
            "essence": "A value that varies, observed across actors without synchronization, lets timing pick the result instead of your code.",
            "description": "When an entropy source from mutated state meets an observation gap from shared state, the outcome depends on interleaving, not logic. The hallmark: you cannot reproduce it on demand.",
            "categories": [
                {"slug": "temporal-interleaving", "name": "Temporal Interleaving",
                 "core": "The schedule, not the logic, picks the result.",
                 "members": ["toctou-race", "lost-update", "out-of-order-async", "concurrent-modification", "pool-exhaustion-heisenbug"]},
                {"slug": "observational-staleness", "name": "Observational Staleness",
                 "core": "You act on a value that has already moved.",
                 "members": ["stale-cache-heisenbug", "use-after-free", "pooled-resource-heisenbug", "lazy-init-visibility-race", "stale-replica-read"]},
                {"slug": "entropic-divergence", "name": "Entropic Divergence",
                 "core": "A hidden random or identity source makes runs differ.",
                 "members": ["predictable-id-collision", "temp-file-race", "concurrent-singleton", "hash-order-divergence", "dispatch-order-divergence"]},
            ],
            "examples": [
                {
                    "title": "TOCTOU Race (Check-Then-Act)",
                    "slug": "toctou-race",
                    "intent": "The gap between checking a resource and using it is a window another actor can slip through — so the bug only fires when the interleaving lines up.",
                    "ingredients": ["race-conditions", "improper-initialization"],
                    "cwes": [
                        {"id": "CWE-665", "name": "Improper Initialization", "url": "https://cwe.mitre.org/data/definitions/665.html", "role": "base", "from": "improper-initialization"},
                        {"id": "CWE-362", "name": "Race Condition", "url": "https://cwe.mitre.org/data/definitions/362.html", "role": "base", "from": "race-conditions"},
                        {"id": "CWE-367", "name": "Time-of-check Time-of-use (TOCTOU) Race Condition", "url": "https://cwe.mitre.org/data/definitions/367.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: check and use are two steps; state can change between them.
# (race-conditions x improper-initialization)
import os
if not os.path.exists(path):      # time-of-check
    os.makedirs(path)             # time-of-use -- another thread or process
                                  # may have created it in the gap -> crash

# RIGHT: act atomically; let the operation own the check.
os.makedirs(path, exist_ok=True)  # no window between check and use""",
                         "explanation": "The window between the existence check and the create is something another actor can race into. It surfaces only when threads interleave just so -- the textbook 'can't reproduce it.'",
                         "output": "race:  intermittent FileExistsError under concurrency\nright: deterministic, no check-use gap"},
                    ],
                },
                {
                    "title": "Lost Update (Read-Modify-Write Race)",
                    "slug": "lost-update",
                    "intent": "Incrementing shared state is three steps — read, add, write — and without a lock two actors overwrite each other's result.",
                    "ingredients": ["threading-mutexes", "increment-decrement-operators"],
                    "cwes": [
                        {"id": "CWE-362", "name": "Race Condition", "url": "https://cwe.mitre.org/data/definitions/362.html", "role": "base", "from": "threading-mutexes"},
                        {"id": "CWE-567", "name": "Unsynchronized Access to Shared Data in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/567.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: read-modify-write on shared state with no lock.
# (threading-mutexes x increment / decrement)
balance = 0
def deposit():
    global balance
    balance = balance + 1          # read, +1, write -- not atomic

# 1000 threads each deposit() -> balance is often < 1000:
# updates interleave and clobber one another.

# RIGHT: serialize the critical section.
lock = Lock()
def deposit():
    global balance
    with lock:
        balance += 1""",
                         "explanation": "Two threads read the same value, both add one, both write back -- one increment vanishes. The final count differs every run.",
                         "output": "race:  balance = 987 (varies every run)\nright: balance = 1000 (always)"},
                    ],
                },
                {
                    "title": "Stale-Cache Heisenbug",
                    "slug": "stale-cache-heisenbug",
                    "intent": "Caching a result that depends on hidden mutable state serves the old answer forever — nobody invalidated it.",
                    "ingredients": ["cache-invalidation", "impure-functions"],
                    "cwes": [
                        {"id": "CWE-1108", "name": "Excessive Reliance on Global Variables", "url": "https://cwe.mitre.org/data/definitions/1108.html", "role": "base", "from": "impure-functions"},
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: cache a value that depends on hidden, mutable state.
# (cache-invalidation x impure-functions)
_cache = {}
def price(item):
    if item not in _cache:
        _cache[item] = item.base * TAX_RATE   # TAX_RATE: a mutable global
    return _cache[item]

TAX_RATE = 1.2     # later, far away, this changes -- the cache never hears

# RIGHT: make the function pure; key on every input.
def price(item, tax):
    return item.base * tax        # same inputs -> same output""",
                         "explanation": "The first call freezes a value computed from a global that later moves. Whether you see fresh or stale output depends on call order and cache warmth.",
                         "output": "stale: first call wins; later TAX_RATE changes ignored\nright: result tracks its inputs, every call"},
                    ],
                },
                {
                    "title": "Out-of-Order Async Completion",
                    "slug": "out-of-order-async",
                    "intent": "Two requests write one variable; whichever resolves last wins — and completion order isn't the order you asked in.",
                    "ingredients": ["var-let-mut", "async-io-misuse"],
                    "cwes": [
                        {"id": "CWE-563", "name": "Assignment to Variable without Use", "url": "https://cwe.mitre.org/data/definitions/563.html", "role": "base", "from": "var-let-mut"},
                        {"id": "CWE-662", "name": "Improper Synchronization", "url": "https://cwe.mitre.org/data/definitions/662.html", "role": "base", "from": "async-io-misuse"},
                        {"id": "CWE-362", "name": "Race Condition", "url": "https://cwe.mitre.org/data/definitions/362.html", "role": "compound"},
                        {"id": "CWE-696", "name": "Incorrect Behavior Order", "url": "https://cwe.mitre.org/data/definitions/696.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: two requests write one variable; last to resolve wins, not last asked.
// (var/let/mut x async I/O misuse)
let results;
async function search(q) {
  results = await fetchJSON(q);   // a slow early query can land AFTER a fast late one
  render(results);                // -> stale results overwrite the fresh ones
}

// RIGHT: tie the write to the latest request; drop out-of-order completions.
let latest = 0;
async function search(q) {
  const seq = ++latest;
  const r = await fetchJSON(q);
  if (seq === latest) render(r);  // ignore anything but the newest query""",
                         "explanation": "Async completions arrive in an order the code never fixed; a mutable, shared 'results' lets whichever finishes last win. Type fast and an earlier request's answer can clobber the one you are looking at.",
                         "output": "race:  a slow 'a' lands after 'ab' and overwrites it\nright: stale responses dropped; newest query always wins"},
                    ],
                },
                {
                    "title": "Concurrent Modification During Iteration",
                    "slug": "concurrent-modification",
                    "intent": "One actor mutates a shared collection while another walks it — the loop sometimes throws, sometimes silently skips.",
                    "ingredients": ["for-loop-control-flow", "threading-mutexes"],
                    "cwes": [
                        {"id": "CWE-820", "name": "Missing Synchronization", "url": "https://cwe.mitre.org/data/definitions/820.html", "role": "base", "from": "threading-mutexes"},
                        {"id": "CWE-362", "name": "Race Condition", "url": "https://cwe.mitre.org/data/definitions/362.html", "role": "compound"},
                        {"id": "CWE-567", "name": "Unsynchronized Access to Shared Data in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/567.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: iterate a shared collection while another thread mutates it.
# (for-loop control flow x threading & mutexes)
for sid, s in sessions.items():      # another thread runs sessions.pop(...)
    if s.expired:
        reap(sid)                    # -> RuntimeError, or entries silently skipped

# RIGHT: iterate a snapshot; mutate the original under a lock.
for sid, s in list(sessions.items()):    # stable copy to walk
    if s.expired:
        with lock:
            sessions.pop(sid, None)""",
                         "explanation": "Reading and writing one collection from two actors with no lock makes the iterator's behavior depend on timing: sometimes it raises, sometimes it skips or double-visits entries. The failure mode changes run to run.",
                         "output": "race:  RuntimeError: dictionary changed size during iteration (sometimes)\nright: snapshot iterated; mutation serialized, no surprise"},
                    ],
                },
                {
                    "title": "Predictable ID Collision Under Load",
                    "slug": "predictable-id-collision",
                    "intent": "Deriving a 'unique' id from the clock works until two requests land in the same tick.",
                    "ingredients": ["time-money-entropy", "race-conditions"],
                    "cwes": [
                        {"id": "CWE-330", "name": "Use of Insufficiently Random Values", "url": "https://cwe.mitre.org/data/definitions/330.html", "role": "base", "from": "time-money-entropy"},
                        {"id": "CWE-366", "name": "Race Condition within a Thread", "url": "https://cwe.mitre.org/data/definitions/366.html", "role": "base", "from": "race-conditions"},
                        {"id": "CWE-340", "name": "Generation of Predictable Numbers or Identifiers", "url": "https://cwe.mitre.org/data/definitions/340.html", "role": "compound"},
                        {"id": "CWE-362", "name": "Race Condition", "url": "https://cwe.mitre.org/data/definitions/362.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: derive a 'unique' id from the clock; two requests in one tick collide.
# (time / money / entropy x race conditions)
import time
def new_id():
    return int(time.time() * 1000)    # ms timestamp -- not unique under load
# two concurrent requests can read the same millisecond -> duplicate id.

# RIGHT: use a real source of uniqueness, not the clock.
import uuid
def new_id():
    return uuid.uuid4().hex           # collision-resistant regardless of timing""",
                         "explanation": "A timestamp looks unique until two requests share a tick; under concurrency that window is hit constantly, and the duplicate id appears only under load -- never on a developer's single call.",
                         "output": "race:  two requests share id 1718...204 -> one record overwrites the other\nright: independent ids; no timing-dependent collisions"},
                    ],
                },
                {
                    "title": "Use-After-Free / Dangling Read",
                    "slug": "use-after-free",
                    "intent": "A freed allocation is still reachable through a second name; the stale read returns whatever reused the slot.",
                    "ingredients": ["aliasing-mutable-defaults", "pointer-mismanagement"],
                    "cwes": [
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "base", "from": "aliasing-mutable-defaults"},
                        {"id": "CWE-416", "name": "Use After Free", "url": "https://cwe.mitre.org/data/definitions/416.html", "role": "base", "from": "pointer-mismanagement"},
                        {"id": "CWE-825", "name": "Expired Pointer Dereference", "url": "https://cwe.mitre.org/data/definitions/825.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: a freed pointer is still aliased and read.
   (aliasing-mutable-defaults x pointer-mismanagement) */
char *p = malloc(32);
strcpy(p, "session");
char *alias = p;          /* a second name for the same allocation */
free(p);                  /* the slot can now be handed to anyone */
puts(alias);              /* use-after-free: prints whatever reused it */

/* RIGHT: drop every alias at free; null it so a stale read is caught. */
free(p);
p = alias = NULL;         /* no live name points at freed memory */""",
                         "explanation": "Two names share one allocation; freeing through one leaves the other dangling. What the stale read returns depends on what the allocator handed the freed slot to next -- so it changes run to run and only crashes sometimes.",
                         "output": "uaf:   prints stale or reallocated bytes; intermittent crash\nright: dangling alias nulled -- a stale read fails loudly, not silently"},
                    ],
                },
                {
                    "title": "Concurrent Singleton Mutation",
                    "slug": "concurrent-singleton",
                    "intent": "One shared object with mutable per-call state, hit by many threads, lets one request's data bleed into another's.",
                    "ingredients": ["this-mutation", "cross-boundary-state-exposure"],
                    "cwes": [
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "base", "from": "this-mutation"},
                        {"id": "CWE-567", "name": "Unsynchronized Access to Shared Data in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/567.html", "role": "base", "from": "cross-boundary-state-exposure"},
                        {"id": "CWE-488", "name": "Exposure of Data Element to Wrong Session", "url": "https://cwe.mitre.org/data/definitions/488.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: one shared instance with mutable state, used by every thread.
// (this mutation x cross-boundary state exposure)
static final SimpleDateFormat FMT = new SimpleDateFormat("yyyy-MM-dd");
String format(Date d) { return FMT.format(d); }   // mutates FMT's internal
                                                  // calendar fields on `this`

// two requests interleave inside format() -> one sees the other's date,
// or a garbled string. Never fails under a single-threaded test.

// RIGHT: don't share mutable state; use an immutable, thread-safe type.
String format(Date d) {
    return DateTimeFormatter.ISO_DATE.format(
        d.toInstant().atZone(ZoneOffset.UTC).toLocalDate());   // no `this` to corrupt
}""",
                         "explanation": "SimpleDateFormat mutates fields on `this` inside format(); a single shared instance hit by concurrent requests lets one thread's date bleed into another's output. The immutable DateTimeFormatter has no mutable receiver to race over.",
                         "output": "shared: request A's output carries request B's date (sometimes)\nright: immutable formatter -- safe to share across all threads"},
                    ],
                },
                {
                    "title": "Pooled-Resource Heisenbug",
                    "slug": "pooled-resource-heisenbug",
                    "intent": "A pooled connection carries un-reset state from a prior borrower, so your result depends on who held it before you.",
                    "ingredients": ["impure-functions", "resource-leaks"],
                    "cwes": [
                        {"id": "CWE-404", "name": "Improper Resource Shutdown or Release", "url": "https://cwe.mitre.org/data/definitions/404.html", "role": "base", "from": "resource-leaks"},
                        {"id": "CWE-1108", "name": "Excessive Reliance on Global Variables", "url": "https://cwe.mitre.org/data/definitions/1108.html", "role": "base", "from": "impure-functions"},
                        {"id": "CWE-459", "name": "Incomplete Cleanup", "url": "https://cwe.mitre.org/data/definitions/459.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a pooled connection carries un-reset state from a prior user.
# (impure-functions x resource-leaks)
conn = pool.acquire()
# the previous borrower left a transaction open / a session var set
rows = conn.execute("SELECT * FROM report")   # sees the leftover state
pool.release(conn)                            # returned dirty, never reset
# whether you get clean or dirty results depends on who held conn before you.

# RIGHT: reset on borrow and return; never trust pooled state.
conn = pool.acquire()
try:
    conn.rollback()                # a known-clean starting state
    rows = conn.execute("SELECT * FROM report")
finally:
    conn.rollback(); pool.release(conn)""",
                         "explanation": "A connection returned to the pool without being reset leaks its session state to the next borrower. The result depends on which pooled object you draw and what ran on it before -- the textbook Heisenbug that vanishes when you add logging or run it alone.",
                         "output": "leaked: results depend on the prior borrower's leftover state\nright: connection reset on borrow/return -- deterministic every draw"},
                    ],
                },
                {
                    "title": "Temp-File Check-Then-Use Race",
                    "slug": "temp-file-race",
                    "intent": "A predictable temp path plus a gap between checking and opening it is a window another actor can slip a symlink through -- so the write lands elsewhere only when the timing lines up.",
                    "ingredients": ["file-network-access", "time-money-entropy"],
                    "cwes": [
                        {"id": "CWE-330", "name": "Use of Insufficiently Random Values", "url": "https://cwe.mitre.org/data/definitions/330.html", "role": "base", "from": "time-money-entropy"},
                        {"id": "CWE-367", "name": "Time-of-check Time-of-use (TOCTOU) Race Condition", "url": "https://cwe.mitre.org/data/definitions/367.html", "role": "base", "from": "file-network-access"},
                        {"id": "CWE-363", "name": "Race Condition Enabling Link Following", "url": "https://cwe.mitre.org/data/definitions/363.html", "role": "compound"},
                        {"id": "CWE-59", "name": "Improper Link Resolution Before File Access ('Link Following')", "url": "https://cwe.mitre.org/data/definitions/59.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: predictable name + check-then-open lets a symlink slip in.
# (file/network access x time/money/entropy)
import os, time
path = f"/tmp/job-{int(time.time())}.tmp"   # predictable, guessable name
if not os.path.exists(path):                # time-of-check
    with open(path, "w") as f:              # time-of-use -- another process
        f.write(secret)                     # may have planted a symlink here

# RIGHT: unpredictable name + atomic, no-follow create owns the check.
import tempfile
fd, path = tempfile.mkstemp()               # O_EXCL|O_CREAT, random name
with os.fdopen(fd, "w") as f:               # no check/use gap, no symlink
    f.write(secret)""",
                         "explanation": "The guessable name (from a clock) plus the existence-check-then-open gap is a window: between check and open another actor can drop a symlink, redirecting the write. It fires only when the interleaving lines up -- otherwise it looks fine.",
                         "output": "race:  the write follows a planted symlink (only sometimes)\nright: mkstemp creates atomically with O_EXCL -- no window, no follow"},
                    ],
                },
                {
                    "title": "Pool-Exhaustion Heisenbug",
                    "slug": "pool-exhaustion-heisenbug",
                    "intent": "Whether a request succeeds or stalls depends on how many others hold the pool right now -- so the same call passes alone and fails under load.",
                    "ingredients": ["resource-contention", "time-money-entropy"],
                    "cwes": [
                        {"id": "CWE-400", "name": "Uncontrolled Resource Consumption", "url": "https://cwe.mitre.org/data/definitions/400.html", "role": "base", "from": "resource-contention"},
                        {"id": "CWE-362", "name": "Race Condition", "url": "https://cwe.mitre.org/data/definitions/362.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a short pool + a per-call timeout -> success depends on live load.
# (resource contention x time/money/entropy)
pool = ConnectionPool(size=4)
def handler(req):
    conn = pool.acquire(timeout=0.2)   # under load: sometimes returns None
    return conn.query(req.sql)         # ...then AttributeError, only sometimes

# RIGHT: size for peak, fail explicitly, never act on a missed acquire.
def handler(req):
    conn = pool.acquire(timeout=2.0)
    if conn is None:
        raise ServiceBusy()            # deterministic, observable backpressure
    try:
        return conn.query(req.sql)
    finally:
        pool.release(conn)""",
                         "explanation": "With a pool smaller than peak concurrency, whether you get a connection depends on how many requests happen to overlap with yours -- pure timing. It passes alone and fails only in the demo under load. Size for peak and turn a missed acquire into explicit backpressure, not a silent None.",
                         "output": "contended: intermittent None -> AttributeError under concurrency\nright: explicit ServiceBusy; outcome no longer depends on who else is in flight"},
                    ],
                },
                {
                    "title": "Lazy-Init Visibility Race (Double-Checked Locking)",
                    "slug": "lazy-init-visibility-race",
                    "intent": "A thread sees the singleton's reference published before its fields are -- so it reads a half-built object, but only on the rare interleaving.",
                    "ingredients": ["improper-initialization", "threading-mutexes"],
                    "cwes": [
                        {"id": "CWE-667", "name": "Improper Locking", "url": "https://cwe.mitre.org/data/definitions/667.html", "role": "base", "from": "threading-mutexes"},
                        {"id": "CWE-543", "name": "Use of Singleton Pattern Without Synchronization in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/543.html", "role": "base", "from": "improper-initialization"},
                        {"id": "CWE-609", "name": "Double-Checked Locking", "url": "https://cwe.mitre.org/data/definitions/609.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Java", "slug": "java", "code":
"""// SMELL: double-checked locking without volatile.
// (improper initialization x threading & mutexes)
static Config inst;                       // not volatile
static Config get() {
    if (inst == null) {                   // 1st check, no lock
        synchronized (Config.class) {
            if (inst == null)
                inst = new Config();      // the ref can publish before the fields
        }
    }
    return inst;                          // another thread may see a half-built Config
}

// RIGHT: volatile makes the publication safe (or use a holder class / enum).
static volatile Config inst;
static Config get() {
    Config c = inst;
    if (c == null) {
        synchronized (Config.class) {
            c = inst;
            if (c == null) inst = c = new Config();
        }
    }
    return c;
}""",
                         "explanation": "Without volatile, the constructor's writes and the reference assignment can be reordered, so a second thread reading inst can see a non-null reference to an object whose fields are still defaults. It only fires on the exact interleaving + visibility window, so it survives every single-threaded test. volatile (or an initialization-on-demand holder) makes the publication safe.",
                         "output": "broken: rare reads of a half-initialized singleton; vanishes under a debugger\nright: safe publication -- every reader sees a fully-built Config"},
                    ],
                },
                {
                    "title": "Hash-Seed Serialization Divergence",
                    "slug": "hash-order-divergence",
                    "intent": "A cache key built by iterating an unordered set differs each process, because the hash seed is randomized -- so the cache never hits and 'it works on my machine.'",
                    "ingredients": ["impure-functions", "cache-invalidation"],
                    "cwes": [
                        {"id": "CWE-758", "name": "Reliance on Undefined, Unspecified, or Implementation-Defined Behavior", "url": "https://cwe.mitre.org/data/definitions/758.html", "role": "base", "from": "impure-functions"},
                        {"id": "CWE-340", "name": "Generation of Predictable Numbers or Identifiers", "url": "https://cwe.mitre.org/data/definitions/340.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a cache key built from set iteration order (randomized per process).
# (impure functions x cache invalidation)
def cache_key(tags):
    return ",".join(str(t) for t in set(tags))   # set order varies per process
cache[cache_key(tags)] = render(tags)            # key differs across workers
# -> each worker writes/reads a different key; the shared cache never hits.

# RIGHT: impose a stable order so the key is a pure function of the inputs.
def cache_key(tags):
    return ",".join(sorted(set(map(str, tags))))  # deterministic across runs""",
                         "explanation": "Python (and Go map) iteration order depends on a per-process-randomized hash seed -- implementation-defined behavior (CWE-758). A key derived from that order diverges between workers, so the shared cache constantly misses in production yet 'works' in a single-process dev run. Sort before joining to make the key a pure function of the inputs.",
                         "output": "unsorted: every worker derives a different key -> cache hit-rate ~0 in prod\nsorted:   one stable key across all processes -> the cache actually hits"},
                    ],
                },
                {
                    "title": "Stale Read From a Lagging Async Replica",
                    "slug": "stale-replica-read",
                    "intent": "You write to the primary, then read back from a replica that hasn't caught up -- so you act on a value that has already moved, but only when replication lag outruns your round-trip.",
                    "ingredients": ["var-let-mut", "async-io-misuse"],
                    "cwes": [
                        {"id": "CWE-563", "name": "Assignment to Variable without Use", "url": "https://cwe.mitre.org/data/definitions/563.html", "role": "base", "from": "var-let-mut"},
                        {"id": "CWE-662", "name": "Improper Synchronization", "url": "https://cwe.mitre.org/data/definitions/662.html", "role": "base", "from": "async-io-misuse"},
                        {"id": "CWE-821", "name": "Incorrect Synchronization", "url": "https://cwe.mitre.org/data/definitions/821.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: write to primary, immediately read from an async replica.
// (var/let/mut x async I/O misuse)
await primary.set(userId, profile);          // write
let current = await replica.get(userId);     // replica lags -> stale read
render(current);                             // acts on a value that moved

// RIGHT: read-your-writes -- read from primary (or wait for the version).
await primary.set(userId, profile);
let current = await primary.get(userId);     // or replica.waitFor(version)
render(current);""",
                         "explanation": "Eventually-consistent replicas apply writes asynchronously, so a read issued right after a write can observe the prior value -- not a lost update (the data is consistent), just old at the moment you look. It only fails when lag happens to exceed the round-trip, so it survives every local test against a single node. Route read-your-writes reads to the primary or gate on a replication version token.",
                         "output": "lagging: the just-written profile reads back as the old one (intermittently)\nright: read-your-writes -> you always observe your own write"},
                    ],
                },
                {
                    "title": "Plugin Registration / Dispatch-Order Divergence",
                    "slug": "dispatch-order-divergence",
                    "intent": "Handlers auto-register via a decorator whose effective order depends on import/await timing -- so when two claim the same route, which one wins changes run to run.",
                    "ingredients": ["macro-mixin-misuse", "async-io-misuse"],
                    "cwes": [
                        {"id": "CWE-696", "name": "Incorrect Behavior Order", "url": "https://cwe.mitre.org/data/definitions/696.html", "role": "base", "from": "async-io-misuse"},
                        {"id": "CWE-758", "name": "Reliance on Undefined, Unspecified, or Implementation-Defined Behavior", "url": "https://cwe.mitre.org/data/definitions/758.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: handlers self-register at import; last writer wins, order undefined.
# (macro & mixin misuse x async I/O misuse)
HANDLERS = {}
def route(path):
    def deco(fn):
        HANDLERS[path] = fn      # silently overwrites any prior handler
        return fn
    return deco
# two plugins both @route('/pay'); whichever module imports last wins,
# and lazy/async import makes that order nondeterministic.

# RIGHT: register explicitly with a priority; reject silent duplicates.
def register(path, fn, priority):
    cur = HANDLERS.get(path)
    if cur and cur.priority >= priority:
        return
    HANDLERS[path] = Handler(fn, priority)   # deterministic resolution""",
                         "explanation": "A decorator/metaclass that registers code out of sight (macro & mixin misuse) makes the registry a function of import order; when plugins load lazily or via async readiness, that order is unspecified, so duplicate routes resolve to a different handler between runs. Relying on that order is reliance on unspecified behavior (CWE-758). Register in one synchronous phase with explicit priorities and duplicate detection.",
                         "output": "implicit: /pay resolves to a different plugin depending on import timing\nright: explicit priority -> the same handler wins every run"},
                    ],
                },
            ],
        },
        {
            "slug": "runtime-errors",
            "color_primary": "9FC5E8",
            "color_secondary": "CFE2F3",
            "color_deep": "3D85C6",
            "name": "Runtime Errors",
            "tagline": "mutated × unstructured — blows up live on input you didn't expect",
            "essence": "Unvalidated input written into program state becomes an illegal value, and the program throws the moment it is used.",
            "description": "A chain: unstructured input with no contract is mutated into state the code has no handler for, then crashes downstream. The hallmark: it works in dev and explodes on the first unexpected input, live.",
            "categories": [
                {"slug": "structural-invalidation", "name": "Structural Invalidation",
                 "core": "The data shape is wrong — types drift, properties vanish, slots stay uninitialized.",
                 "members": ["null-dereference", "type-confusion", "shape-drift", "aliased-default-type-confusion", "unexpected-type-element"]},
                {"slug": "computational-exhaustion", "name": "Computational Exhaustion",
                 "core": "The math or scale is broken — loops run away, indexes overrun, formulas hit a wall.",
                 "members": ["off-by-one-crash", "divide-by-zero-empty", "quadratic-blowup", "nan-poisons-consumer", "integer-overflow-buffer"]},
                {"slug": "architectural-drift", "name": "Architectural Drift",
                 "core": "The contract is severed — an external API, library, or payload no longer matches.",
                 "members": ["integration-crash", "stale-api-upgrade", "deserialized-type-confusion", "config-drift-crash", "schema-drift-on-load"]},
            ],
            "examples": [
                {
                    "title": "Null / Undefined Dereference",
                    "slug": "null-dereference",
                    "intent": "A function that sometimes returns nothing, used as if it always returns a value.",
                    "ingredients": ["improper-initialization", "missing-undefined-returns"],
                    "cwes": [
                        {"id": "CWE-457", "name": "Use of Uninitialized Variable", "url": "https://cwe.mitre.org/data/definitions/457.html", "role": "base", "from": "improper-initialization"},
                        {"id": "CWE-252", "name": "Unchecked Return Value", "url": "https://cwe.mitre.org/data/definitions/252.html", "role": "base", "from": "missing-undefined-returns"},
                        {"id": "CWE-476", "name": "NULL Pointer Dereference", "url": "https://cwe.mitre.org/data/definitions/476.html", "role": "compound"},
                        {"id": "CWE-690", "name": "Unchecked Return Value to NULL Pointer Dereference", "url": "https://cwe.mitre.org/data/definitions/690.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: a function that sometimes returns nothing, dereferenced anyway.
// (improper-initialization x missing/undefined returns)
function findUser(id) {
  for (const u of users) if (u.id === id) return u;
  // no match -> falls off the end -> returns undefined
}
const name = findUser(42).name;   // TypeError, but only for unknown ids

// RIGHT: make 'not found' explicit and handle it.
const user = findUser(42);
const name = user ? user.name : "(unknown)";""",
                         "explanation": "The missing return path yields undefined; the caller assumes an object. The crash waits for an id that isn't there -- often a production-only input.",
                         "output": "bad input: Cannot read properties of undefined (reading 'name')\nguarded:   '(unknown)'"},
                    ],
                },
                {
                    "title": "Type Confusion / Bad Coercion",
                    "slug": "type-confusion",
                    "intent": "State mutated to a new type that the next reader does not expect.",
                    "ingredients": ["this-mutation", "type-errors"],
                    "cwes": [
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "base", "from": "this-mutation"},
                        {"id": "CWE-704", "name": "Incorrect Type Conversion or Cast", "url": "https://cwe.mitre.org/data/definitions/704.html", "role": "base", "from": "type-errors"},
                        {"id": "CWE-843", "name": "Type Confusion", "url": "https://cwe.mitre.org/data/definitions/843.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "JavaScript", "slug": "javascript", "code":
"""// SMELL: a field mutated to a different type than the next line assumes.
// (this mutation x type errors)
class Cart {
  constructor() { this.total = 0; }         // number
  applyCoupon(code) { this.total = code; }  // oops: now a string
}
const c = new Cart();
c.applyCoupon("SAVE10");
c.total.toFixed(2);   // TypeError: c.total.toFixed is not a function

// RIGHT: keep the field's type stable; parse input into the right shape.
applyCoupon(code) { this.discount = lookup(code); }  // total stays a number""",
                         "explanation": "Mutating this.total to a string keeps the program running until something treats it as a number. The type drift and the crash are in different methods.",
                         "output": "confused: total is a string; .toFixed is not a function\nstable:   total stays numeric"},
                    ],
                },
                {
                    "title": "Integration Crash on Unexpected Input",
                    "slug": "integration-crash",
                    "intent": "Trusting an external payload's shape, then mutating local state from it.",
                    "ingredients": ["impure-functions", "lack-of-input-validation"],
                    "cwes": [
                        {"id": "CWE-1108", "name": "Excessive Reliance on Global Variables", "url": "https://cwe.mitre.org/data/definitions/1108.html", "role": "base", "from": "impure-functions"},
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "lack-of-input-validation"},
                        {"id": "CWE-248", "name": "Uncaught Exception", "url": "https://cwe.mitre.org/data/definitions/248.html", "role": "compound"},
                        {"id": "CWE-754", "name": "Improper Check for Unusual or Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/754.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: trust a third party's shape; mutate local state from it.
# (impure-functions / external read x lack-of-input-validation)
resp = requests.get(url).json()      # third party, shape not guaranteed
user.age = resp["profile"]["age"]    # KeyError the day they rename a field
                                     # -- first seen live, in the demo

# RIGHT: validate at the boundary; fail with a clear, handled error.
data = UserSchema.parse(resp)        # parse, don't trust
user.age = data.profile.age""",
                         "explanation": "The crash isn't in your logic -- it's the contract you never enforced at the edge. It appears the first time the integration sends a shape you didn't anticipate.",
                         "output": "unexpected: KeyError 'profile' when their API changes\nvalidated: a clear ValidationError you handle, not a 500"},
                    ],
                },
                {
                    "title": "Off-by-One Index Crash",
                    "slug": "off-by-one-crash",
                    "intent": "A counter advanced one step too far indexes past the end of the array.",
                    "ingredients": ["increment-decrement-operators", "index-out-of-bounds"],
                    "cwes": [
                        {"id": "CWE-129", "name": "Improper Validation of Array Index", "url": "https://cwe.mitre.org/data/definitions/129.html", "role": "base", "from": "index-out-of-bounds"},
                        {"id": "CWE-193", "name": "Off-by-one Error", "url": "https://cwe.mitre.org/data/definitions/193.html", "role": "base", "from": "increment-decrement-operators"},
                        {"id": "CWE-125", "name": "Out-of-bounds Read", "url": "https://cwe.mitre.org/data/definitions/125.html", "role": "compound"},
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a counter advanced one step too far indexes past the end.
# (increment / decrement x index out of bounds)
i = 0
while i <= len(rows):          # <= overshoots by one
    process(rows[i])           # IndexError on the last turn -- only for some sizes
    i += 1

# RIGHT: stop before the length; let the language bound the walk.
for row in rows:               # no hand-rolled index to get wrong
    process(row)""",
                         "explanation": "An off-by-one in the loop bound (<= instead of <) pushes the index one past the array. In Python it is an IndexError; in C the same slip is an out-of-bounds read or write. It survives every test whose input length never reached the last step.",
                         "output": "bug:   IndexError: list index out of range on the final element\nright: every row processed; no manual index to overshoot"},
                    ],
                },
                {
                    "title": "Divide-by-Zero from an Empty Aggregate",
                    "slug": "divide-by-zero-empty",
                    "intent": "An average over input that can be empty drives the denominator to zero.",
                    "ingredients": ["var-let-mut", "divide-by-zero"],
                    "cwes": [
                        {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html", "role": "base", "from": "divide-by-zero"},
                        {"id": "CWE-369", "name": "Divide By Zero", "url": "https://cwe.mitre.org/data/definitions/369.html", "role": "compound"},
                        {"id": "CWE-754", "name": "Improper Check for Unusual or Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/754.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: average a list that can be empty; the denominator reaches zero.
# (var/let/mut x divide by zero)
total = sum(values)
count = len(values)            # 0 when the input is empty
avg = total / count            # ZeroDivisionError, first seen on real data

# RIGHT: guard the exceptional input before you divide.
avg = total / count if count else 0.0""",
                         "explanation": "A mutable count fed straight into a division blows up the moment the input is empty -- a case dev data never had. The crash reads as a divide-by-zero, but the root is the unchecked exceptional condition.",
                         "output": "bug:   ZeroDivisionError: division by zero on an empty list\nright: empty input yields 0.0, not a 500"},
                    ],
                },
                {
                    "title": "Stale API After Upgrade (AttributeError)",
                    "slug": "stale-api-upgrade",
                    "intent": "A mixin reaches for a method a dependency upgrade quietly renamed or removed.",
                    "ingredients": ["macro-mixin-misuse", "version-library-mismanagement"],
                    "cwes": [
                        {"id": "CWE-1104", "name": "Use of Unmaintained Third Party Components", "url": "https://cwe.mitre.org/data/definitions/1104.html", "role": "base", "from": "version-library-mismanagement"},
                        {"id": "CWE-758", "name": "Reliance on Undefined, Unspecified, or Implementation-Defined Behavior", "url": "https://cwe.mitre.org/data/definitions/758.html", "role": "compound"},
                        {"id": "CWE-754", "name": "Improper Check for Unusual or Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/754.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a mixin assumes a method a dependency upgrade renamed/removed.
# (macro / mixin misuse x version / library mismanagement)
class Audited(Base):
    def save(self):
        self.log_change()       # provided by Base... until v3 renamed it
        super().save()          # AttributeError the first time this path runs

# RIGHT: pin the contract and adapt at the seam, not deep in a mixin.
class Audited(Base):
    def save(self):
        getattr(self, "log_change", self._audit_fallback)()
        super().save()""",
                         "explanation": "A mixin reaches for an inherited method a library upgrade quietly dropped; nothing fails at import, only when that path executes after deploy. It relies on behavior the new version no longer guarantees.",
                         "output": "bug:   AttributeError: 'Audited' object has no attribute 'log_change'\nright: missing hook degrades to a fallback, not a crash"},
                    ],
                },
                {
                    "title": "Unbounded Input → Quadratic Blowup",
                    "slug": "quadratic-blowup",
                    "intent": "An accumulating loop with no cap on input size turns a linear scan into O(n²) — fine in dev, fatal on real volume.",
                    "ingredients": ["for-loop-control-flow", "unconstrained-inputs"],
                    "cwes": [
                        {"id": "CWE-834", "name": "Excessive Iteration", "url": "https://cwe.mitre.org/data/definitions/834.html", "role": "base", "from": "for-loop-control-flow"},
                        {"id": "CWE-1284", "name": "Improper Validation of Specified Quantity in Input", "url": "https://cwe.mitre.org/data/definitions/1284.html", "role": "base", "from": "unconstrained-inputs"},
                        {"id": "CWE-400", "name": "Uncontrolled Resource Consumption", "url": "https://cwe.mitre.org/data/definitions/400.html", "role": "compound"},
                        {"id": "CWE-407", "name": "Inefficient Algorithmic Complexity", "url": "https://cwe.mitre.org/data/definitions/407.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: O(n^2) over an input whose size nobody bounded.
# (for-loop control flow x unconstrained inputs)
seen = []
for item in payload:            # payload size is caller-controlled
    if item not in seen:        # 'in' on a list is O(n) each pass
        seen.append(item)       # whole loop is O(n^2): 100k items -> 10^10 ops
# fine on the dev's 50-row sample; the request times out / OOMs in prod.

# RIGHT: bound the input, and pick the right data structure.
if len(payload) > MAX_ITEMS:
    raise ValueError("payload too large")
seen = set(payload)             # O(n) de-dup, memory capped""",
                         "explanation": "A linear scan inside the loop makes the whole pass quadratic; with no cap on input size, an input two orders of magnitude past the test data turns milliseconds into minutes. The failure -- a timeout or OOM kill -- is first seen on real volume.",
                         "output": "unbounded: 100k-row input -> request times out, worker OOM-killed\nbounded:   oversize input rejected; set lookup stays O(n)"},
                    ],
                },
                {
                    "title": "Shape-Drift AttributeError",
                    "slug": "shape-drift",
                    "intent": "An attribute set on only some code paths, read on a path that assumes it always exists.",
                    "ingredients": ["improper-initialization", "duck-typing-without-contracts"],
                    "cwes": [
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "duck-typing-without-contracts"},
                        {"id": "CWE-908", "name": "Use of Uninitialized Resource", "url": "https://cwe.mitre.org/data/definitions/908.html", "role": "base", "from": "improper-initialization"},
                        {"id": "CWE-754", "name": "Improper Check for Unusual or Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/754.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: an attribute added only on one path; another path assumes it exists.
# (improper-initialization x duck-typing-without-contracts)
class Job:
    def __init__(self): self.status = "new"

def run(job):
    if job.status == "retry":
        job.result = recompute()     # 'result' set ONLY on the retry path
    report(job.result)               # AttributeError when status != "retry"

# RIGHT: give the shape a contract; every field exists from construction.
class Job:
    def __init__(self):
        self.status = "new"
        self.result = None           # always present""",
                         "explanation": "Because result is added on only one branch, every other path reaches for an attribute that was never set. Duck typing hides it until the missing-attribute path runs on real data -- an AttributeError live, not in dev.",
                         "output": "drift: AttributeError: 'Job' object has no attribute 'result'\nright: field present from construction; absence handled as None"},
                    ],
                },
                {
                    "title": "NaN / Infinity Poisons a Consumer",
                    "slug": "nan-poisons-consumer",
                    "intent": "Float math yields NaN or Infinity, which crashes the first consumer that needs a finite number.",
                    "ingredients": ["time-money-entropy", "lack-of-input-validation"],
                    "cwes": [
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "lack-of-input-validation"},
                        {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html", "role": "base", "from": "time-money-entropy"},
                        {"id": "CWE-754", "name": "Improper Check for Unusual or Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/754.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: float math yields NaN/Infinity; a consumer assumes a finite number.
# (time / money / entropy x lack of input validation)
import math
rate = spent / hours          # hours == 0.0 -> inf;  0.0 / 0.0 -> nan
buckets[int(rate)] += 1       # int(inf) -> OverflowError; int(nan) -> ValueError
                              # first seen on a real row with hours == 0

# RIGHT: validate the numeric domain before you consume it.
if not math.isfinite(rate):
    raise ValueError("rate must be finite")
buckets[int(rate)] += 1""",
                         "explanation": "Float division silently produces inf or nan; nothing flags it until a consumer that needs a finite integer chokes. int(inf) raises OverflowError and int(nan) raises ValueError -- a crash whose root is the unchecked numeric domain, seen first on production data.",
                         "output": "bad:   OverflowError / ValueError converting inf or nan to int\nright: non-finite rate rejected at the boundary, with a clear error"},
                    ],
                },
                {
                    "title": "Aliased-Default Type Confusion",
                    "slug": "aliased-default-type-confusion",
                    "intent": "A function's mutable default is shared across calls; one caller seeds it with a wrong-typed value, and a later caller crashes operating on it.",
                    "ingredients": ["aliasing-mutable-defaults", "type-errors"],
                    "cwes": [
                        {"id": "CWE-704", "name": "Incorrect Type Conversion or Cast", "url": "https://cwe.mitre.org/data/definitions/704.html", "role": "base", "from": "type-errors"},
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "base", "from": "aliasing-mutable-defaults"},
                        {"id": "CWE-843", "name": "Access of Resource Using Incompatible Type ('Type Confusion')", "url": "https://cwe.mitre.org/data/definitions/843.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a shared mutable default accumulates state across calls.
# (aliasing mutable defaults x type errors)
def collect(item, into=[]):     # one list, shared by every call
    into.append(item)
    return sum(into)            # crashes once a non-number slips in

collect(1); collect(2)          # fine
collect("x")                    # TypeError on the NEXT caller's sum(into)

# RIGHT: a fresh container per call; the type stays local and predictable.
def collect(item, into=None):
    into = [] if into is None else into
    into.append(item)
    return sum(into)""",
                         "explanation": "The default list is created once and aliased by every call, so values -- and their types -- leak between callers. One caller appending a string poisons the shared default; a later caller's sum() hits incompatible types, a crash with no obvious owner. Default to None and build a fresh container inside.",
                         "output": "shared: TypeError in a caller that never touched a string\nright: each call owns its list; types stay local"},
                    ],
                },
                {
                    "title": "Deserialized Type Confusion Crash",
                    "slug": "deserialized-type-confusion",
                    "intent": "A payload reconstructs an object of an unexpected shape; the code mutates it as the type it assumed, and crashes on the first attribute that isn't there.",
                    "ingredients": ["insecure-deserialization", "this-mutation"],
                    "cwes": [
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "base", "from": "this-mutation"},
                        {"id": "CWE-502", "name": "Deserialization of Untrusted Data", "url": "https://cwe.mitre.org/data/definitions/502.html", "role": "base", "from": "insecure-deserialization"},
                        {"id": "CWE-704", "name": "Incorrect Type Conversion or Cast", "url": "https://cwe.mitre.org/data/definitions/704.html", "role": "compound"},
                        {"id": "CWE-843", "name": "Access of Resource Using Incompatible Type ('Type Confusion')", "url": "https://cwe.mitre.org/data/definitions/843.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: trust the deserialized shape, then mutate it as the assumed type.
# (insecure deserialization x this-mutation)
data = json.loads(body)          # could be a list, str, number -- anything
data["seen"] = True              # assumes a dict; TypeError on a list/str
return save(data)

# RIGHT: parse into a known type at the boundary; reject the rest.
raw = json.loads(body)
if not isinstance(raw, dict):
    raise BadRequest("object expected")
record = Record(**raw)           # validated shape; safe to mutate
record.seen = True
return save(record)""",
                         "explanation": "json.loads returns whatever the payload encodes; treating it as a dict and mutating it crashes the moment a client sends a list or scalar. Parse the untrusted bytes into a known type at the boundary (parse, don't validate) so the rest of the code operates on a shape it can trust.",
                         "output": "trusting: TypeError: 'list' object does not support item assignment\nright: shape validated at the edge; a bad payload fails with a clear 400"},
                    ],
                },
                {
                    "title": "Config / Environment Drift Crash",
                    "slug": "config-drift-crash",
                    "intent": "An upgrade renames the config key the code still reads directly; the service boots fine and dies on the first request in the new environment.",
                    "ingredients": ["impure-functions", "version-library-mismanagement"],
                    "cwes": [
                        {"id": "CWE-1108", "name": "Excessive Reliance on Global Variables", "url": "https://cwe.mitre.org/data/definitions/1108.html", "role": "base", "from": "impure-functions"},
                        {"id": "CWE-1104", "name": "Use of Unmaintained Third Party Components", "url": "https://cwe.mitre.org/data/definitions/1104.html", "role": "base", "from": "version-library-mismanagement"},
                        {"id": "CWE-248", "name": "Uncaught Exception", "url": "https://cwe.mitre.org/data/definitions/248.html", "role": "compound"},
                        {"id": "CWE-754", "name": "Improper Check for Unusual or Exceptional Conditions", "url": "https://cwe.mitre.org/data/definitions/754.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: read a config key directly, assuming the deploy still provides it.
# (impure functions x library/version mismanagement)
TIMEOUT = os.environ["RPC_TIMEOUT_MS"]   # renamed to RPC_TIMEOUT_S in v2 -> KeyError
def call():
    client.invoke(timeout=int(TIMEOUT))

# RIGHT: read through validated config -- presence check + migration-aware error.
def get_timeout():
    raw = os.environ.get("RPC_TIMEOUT_S")
    if raw is None:
        raise ConfigError("RPC_TIMEOUT_S required (was RPC_TIMEOUT_MS before v2)")
    return float(raw)""",
                         "explanation": "Reading os.environ by key couples the code to a contract the platform can change between versions. After an upgrade renames the key, the process imports fine and throws an uncaught KeyError on first use -- in the new environment only. Centralize config access with explicit presence checks and a migration-aware error.",
                         "output": "drifted: KeyError: 'RPC_TIMEOUT_MS' on the first request after deploy\nright: a clear ConfigError naming the renamed key; no surprise mid-request"},
                    ],
                },
                {
                    "title": "Integer Overflow Sizes a Buffer Too Small",
                    "slug": "integer-overflow-buffer",
                    "intent": "A size computed by multiplying or incrementing wraps past the integer max to a tiny value; the allocation looks fine, then an in-range-looking index writes off the end.",
                    "ingredients": ["increment-decrement-operators", "index-out-of-bounds"],
                    "cwes": [
                        {"id": "CWE-129", "name": "Improper Validation of Array Index", "url": "https://cwe.mitre.org/data/definitions/129.html", "role": "base", "from": "index-out-of-bounds"},
                        {"id": "CWE-190", "name": "Integer Overflow or Wraparound", "url": "https://cwe.mitre.org/data/definitions/190.html", "role": "base", "from": "increment-decrement-operators"},
                        {"id": "CWE-680", "name": "Integer Overflow to Buffer Overflow", "url": "https://cwe.mitre.org/data/definitions/680.html", "role": "compound"},
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""// SMELL: the size multiply wraps, so the buffer is far too small.
// (increment/decrement operators x index out of bounds)
size_t n = count * width;          // count*width overflows size_t -> tiny n
char *buf = malloc(n);             // allocates the wrapped (small) size
for (size_t i = 0; i < count * width; i++)
    buf[i] = src[i];               // runs to the true length -> OOB write

// RIGHT: detect the overflow before allocating.
if (width != 0 && count > SIZE_MAX / width)
    return -1;                     // refuse the impossible size
char *buf = malloc(count * width);""",
                         "explanation": "Multiplying two attacker-influenced sizes overflows the integer and wraps to a small value (CWE-190); malloc honors the small size, but the copy loop runs to the true count*width and writes past the buffer -- the named chain CWE-680 'Integer Overflow to Buffer Overflow', ending in an out-of-bounds write. Check count > SIZE_MAX/width (or use a checked multiply) before allocating.",
                         "output": "overflow: malloc(small) then a full-length copy -> heap corruption\nright: oversized request rejected; no wrap, no overflow"},
                    ],
                },
                {
                    "title": "Unexpected-Type Element in a Heterogeneous Collection",
                    "slug": "unexpected-type-element",
                    "intent": "A collection assumed to hold one shape gains an element of another type -- directly or because a method mutated one item -- and the consumer crashes when it reaches the odd one.",
                    "ingredients": ["this-mutation", "duck-typing-without-contracts"],
                    "cwes": [
                        {"id": "CWE-471", "name": "Modification of Assumed-Immutable Data (MAID)", "url": "https://cwe.mitre.org/data/definitions/471.html", "role": "base", "from": "this-mutation"},
                        {"id": "CWE-1287", "name": "Improper Validation of Specified Type of Input", "url": "https://cwe.mitre.org/data/definitions/1287.html", "role": "base", "from": "duck-typing-without-contracts"},
                        {"id": "CWE-241", "name": "Improper Handling of Unexpected Data Type", "url": "https://cwe.mitre.org/data/definitions/241.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a list assumed to be all numbers; one path appends a string.
# (this-mutation x duck typing without contracts)
def total(items):
    return sum(x * 2 for x in items)    # assumes every x is numeric

items.append(row["qty"])                # row["qty"] is "3" from a CSV -> str
total(items)                            # TypeError on the one string element

# RIGHT: validate the element's type at the boundary; coerce or reject.
def add_qty(items, raw):
    try:
        items.append(int(raw))          # parse into the expected type
    except (TypeError, ValueError):
        raise BadRow(f"qty not an int: {raw!r}")""",
                         "explanation": "Duck typing with no element contract (CWE-1287) lets a wrong-typed item slip into a collection the consumer treats as uniform; the code then mishandles the unexpected type (CWE-241) and crashes on the one odd element -- a shape failure, not arithmetic or timing. Validate or coerce each element where it enters, so the collection's type is an invariant.",
                         "output": "mixed: TypeError: can't multiply sequence by non-int (one bad row)\nright: each element parsed at entry -> the collection stays homogeneous"},
                    ],
                },
                {
                    "title": "Schema-Drift on Load: Old Payload Sets Unexpected Attributes",
                    "slug": "schema-drift-on-load",
                    "intent": "A record serialized by an older class version is rehydrated by a mixin/ORM that blindly sets whatever attributes the payload names -- so a renamed or dropped field lands on the new object and a later access breaks.",
                    "ingredients": ["macro-mixin-misuse", "insecure-deserialization"],
                    "cwes": [
                        {"id": "CWE-502", "name": "Deserialization of Untrusted Data", "url": "https://cwe.mitre.org/data/definitions/502.html", "role": "base", "from": "insecure-deserialization"},
                        {"id": "CWE-915", "name": "Improperly Controlled Modification of Dynamically-Determined Object Attributes", "url": "https://cwe.mitre.org/data/definitions/915.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a mixin rehydrates rows by setting every key from the payload.
# (macro & mixin misuse x insecure deserialization)
class FromDict:                         # shared across all models
    @classmethod
    def load(cls, d):
        o = cls.__new__(cls)
        o.__dict__.update(d)            # sets WHATEVER keys the old schema had
        return o
# v1 stored {"addr": ...}; v2 renamed it "address" -> o.address is missing
# and a stale "addr" rides along. Later o.address -> AttributeError.

# RIGHT: map through a versioned, allow-listed schema on load.
class FromDict:
    @classmethod
    def load(cls, d):
        d = migrate(d, d.get("_v", 1))  # bring the old shape to current
        return cls(**{k: d[k] for k in cls.FIELDS})   # only known fields""",
                         "explanation": "Blindly splatting a deserialized dict onto an object (CWE-502 feeding mass-assignment, CWE-915) couples your data to whatever schema wrote it; after a field is renamed across versions, old payloads set unexpected attributes and omit current ones, so a downstream access fails. The contract that drifted is the schema version between writer and reader. Migrate by version and assign only allow-listed fields.",
                         "output": "drifted: old rows leave .address unset (+ a stale .addr) -> AttributeError\nright: versioned migration + field allow-list -> every object is current-shape"},
                    ],
                },
            ],
        },
        {
            "slug": "data-corruption",
            "color_primary": "C27BA0",
            "color_secondary": "E0B6CF",
            "color_deep": "8E5572",
            "name": "Data Corruption",
            "tagline": "shared × unstructured — bad data written permanently, and it spreads",
            "essence": "Unvalidated data crossing a shared boundary persists and propagates, so the damage is permanent and reaches other actors.",
            "description": "When data without a contract is written across a shared boundary -- memory, a wire, a store -- the badness is durable: it overwrites, exposes, or poisons what others read. The hallmark: permanent damage and information exposure.",
            "categories": [
                {"slug": "memory-corruption", "name": "Memory Corruption",
                 "core": "Memory-level positioning gone wrong — structural destruction and boundary smashes.",
                 "members": ["buffer-overflow", "stack-smash", "type-confusion-corruption", "torn-write", "unchecked-sentinel-corruption"]},
                {"slug": "semantic-corruption", "name": "Semantic Corruption",
                 "core": "Logical poisoning, encoding failures, and mismatched representations.",
                 "members": ["nan-shared-aggregate", "mojibake", "unframed-stream", "lossy-coercion-ledger", "signedness-poisons-aggregate"]},
                {"slug": "representational-corruption", "name": "Representational Corruption",
                 "core": "Session bleed and schema drift across boundaries.",
                 "members": ["cross-session-contamination", "schema-drift", "stored-injection", "untrusted-deserialization", "web-cache-poisoning"]},
            ],
            "examples": [
                {
                    "title": "Buffer Overflow (Out-of-Bounds Write)",
                    "slug": "buffer-overflow",
                    "intent": "An unchecked length writes past the buffer into adjacent, shared memory.",
                    "ingredients": ["pointer-mismanagement", "index-out-of-bounds"],
                    "cwes": [
                        {"id": "CWE-120", "name": "Buffer Copy without Checking Size of Input ('Classic Buffer Overflow')", "url": "https://cwe.mitre.org/data/definitions/120.html", "role": "base", "from": "index-out-of-bounds"},
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: unchecked length writes past the buffer into adjacent memory.
   (pointer-mismanagement x index-out-of-bounds) */
char name[16];
strcpy(name, input);     /* input over 15 chars -> overwrites the stack */

/* RIGHT: bound the write to the buffer you actually own. */
char name[16];
snprintf(name, sizeof name, "%s", input);   /* truncates, never overflows */""",
                         "explanation": "The missing bounds check (unstructured) plus a raw pointer write (shared memory) corrupts whatever sits past the buffer -- often the return address.",
                         "output": "overflow: corrupts the return address; crash or code exec\nbounded:  safely truncated, memory intact"},
                    ],
                },
                {
                    "title": "Untrusted Deserialization Across a Boundary",
                    "slug": "untrusted-deserialization",
                    "intent": "Deserializing bytes that crossed a trust boundary turns a payload into live objects.",
                    "ingredients": ["file-network-access", "insecure-deserialization"],
                    "cwes": [
                        {"id": "CWE-345", "name": "Insufficient Verification of Data Authenticity", "url": "https://cwe.mitre.org/data/definitions/345.html", "role": "base", "from": "file-network-access"},
                        {"id": "CWE-913", "name": "Improper Control of Dynamically-Managed Code Resources", "url": "https://cwe.mitre.org/data/definitions/913.html", "role": "base", "from": "insecure-deserialization"},
                        {"id": "CWE-502", "name": "Deserialization of Untrusted Data", "url": "https://cwe.mitre.org/data/definitions/502.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: deserialize bytes that crossed a trust boundary.
# (file-network-access x insecure-deserialization)
data = pickle.loads(sock.recv(4096))   # attacker-controlled bytes ->
                                       # arbitrary objects / code on load

# RIGHT: use a data-only format and validate the result.
data = json.loads(sock.recv(4096))     # no code execution on parse
record = Record(**validate(data))      # then check the shape""",
                         "explanation": "Untrusted input (unstructured) arriving over the network (shared boundary) executes during load. The corruption is the foreign object graph now inside your process.",
                         "output": "pickle: a crafted payload runs code during load\njson:   inert data; validated before use"},
                    ],
                },
                {
                    "title": "Schema Drift / Mixed-Version Write",
                    "slug": "schema-drift",
                    "intent": "Two deployed versions share one store with no schema contract, so one writes what the other can't read.",
                    "ingredients": ["file-network-access", "unconstrained-inputs"],
                    "cwes": [
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "unconstrained-inputs"},
                        {"id": "CWE-345", "name": "Insufficient Verification of Data Authenticity", "url": "https://cwe.mitre.org/data/definitions/345.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: two deployed versions share one store, no schema contract.
# (file/network access x unconstrained inputs)
# during a rolling deploy:
record["amount"] = 1050      # v1 writes cents (int)
record["amount"] = 10.50     # v2 writes dollars (float) -- same field
# readers can no longer tell which; the ledger is now corrupt.

# RIGHT: version the payload and validate on read.
record = {"schema": 2, "amount_cents": 1050}
assert record["schema"] == READER_SCHEMA   # refuse mixed-version data""",
                         "explanation": "Two versions out of lockstep (shared) writing a field with no contract (unstructured) silently poison the store. Nothing crashes; the data just means two things at once.",
                         "output": "drift:     amount means cents OR dollars by writer\nversioned: reader rejects data it can't interpret"},
                    ],
                },
                {
                    "title": "Stored / Second-Order Injection",
                    "slug": "stored-injection",
                    "intent": "Unvalidated input is persisted once, then trusted every time it is read back out.",
                    "ingredients": ["file-network-access", "lack-of-input-validation"],
                    "cwes": [
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "lack-of-input-validation"},
                        {"id": "CWE-89", "name": "SQL Injection", "url": "https://cwe.mitre.org/data/definitions/89.html", "role": "compound"},
                        {"id": "CWE-79", "name": "Cross-site Scripting (XSS)", "url": "https://cwe.mitre.org/data/definitions/79.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: store unvalidated input, then trust it on the way back out.
# (file / network access x lack of input validation)
db.execute("INSERT INTO notes(body) VALUES (?)", [body])   # raw user HTML stored
...
page += row["body"]            # later rendered to every viewer -> stored XSS
# or: db.execute("... WHERE name = '" + row["name"] + "'")  # second-order SQLi

# RIGHT: validate on the way in, encode on the way out, parameterize always.
db.execute("INSERT INTO notes(body) VALUES (?)", [clean(body)])
page += escape_html(row["body"])""",
                         "explanation": "The poison is written once and read forever: unvalidated input persisted to a shared store comes back as executable markup or SQL. The damage is durable and reaches every later reader, not just the attacker.",
                         "output": "stored:   one bad row scripts every viewer / re-injects on read\nsanitized: cleaned in, encoded out -- inert at rest"},
                    ],
                },
                {
                    "title": "Web Cache Poisoning",
                    "slug": "web-cache-poisoning",
                    "intent": "An input the cache key ignores gets baked into a stored response and replayed to everyone.",
                    "ingredients": ["cache-invalidation", "unconstrained-inputs"],
                    "cwes": [
                        {"id": "CWE-348", "name": "Use of Less Trusted Source", "url": "https://cwe.mitre.org/data/definitions/348.html", "role": "base", "from": "unconstrained-inputs"},
                        {"id": "CWE-349", "name": "Acceptance of Extraneous Untrusted Data With Trusted Data", "url": "https://cwe.mitre.org/data/definitions/349.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: cache a response keyed on trusted parts, ignoring an input that shapes it.
# (cache invalidation x unconstrained inputs)
key = request.path                        # ignores the X-Forwarded-Host header...
cache[key] = render(page, host=request.headers["X-Forwarded-Host"])
# attacker sets the header once -> poisoned page served to everyone after.

# RIGHT: key on every input that affects the output, or don't trust it at all.
host = allowlist(request.headers.get("X-Forwarded-Host"))   # vetted
cache[(request.path, host)] = render(page, host=host)""",
                         "explanation": "An attacker-influenced input the cache key ignores gets stored in a response and replayed to every subsequent user. One request poisons the shared cache; the blast radius is everyone downstream.",
                         "output": "poisoned: one crafted header rewrites the page for all users\nkeyed:    host vetted and part of the key -- no shared poisoning"},
                    ],
                },
                {
                    "title": "Encoding / Charset Corruption (Mojibake)",
                    "slug": "mojibake",
                    "intent": "Bytes written in one charset and read back assuming another corrupt the stored text permanently.",
                    "ingredients": ["file-network-access", "version-library-mismanagement"],
                    "cwes": [
                        {"id": "CWE-176", "name": "Improper Handling of Unicode Encoding", "url": "https://cwe.mitre.org/data/definitions/176.html", "role": "base", "from": "file-network-access"},
                        {"id": "CWE-172", "name": "Encoding Error", "url": "https://cwe.mitre.org/data/definitions/172.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: write bytes in one charset, read them back assuming another.
# (file / network access x version / library mismanagement)
with open("names.csv", "w") as f:        # platform-default encoding (changed across versions)
    f.write(name)
...
name = open("names.csv", encoding="utf-8").read()   # decoded as UTF-8 -> mojibake
# every round-trip mangles the bytes further; the store is now corrupt.

# RIGHT: pin the encoding on both ends; never rely on the default.
with open("names.csv", "w", encoding="utf-8") as f:
    f.write(name)
name = open("names.csv", encoding="utf-8").read()""",
                         "explanation": "Relying on a default encoding that differs across versions and hosts means bytes written as one charset are decoded as another. The text is corrupted in place -- permanently -- and the damage compounds on each re-save.",
                         "output": "mojibake: 'Jose' with an accent reads back garbled, then worse\npinned:   utf-8 on both ends -- bytes survive the round-trip"},
                    ],
                },
                {
                    "title": "Unframed Stream Corruption",
                    "slug": "unframed-stream",
                    "intent": "Reading a byte stream in fixed chunks with no length contract lets records split and merge — then persist mis-framed.",
                    "ingredients": ["async-io-misuse", "unconstrained-inputs"],
                    "cwes": [
                        {"id": "CWE-400", "name": "Uncontrolled Resource Consumption", "url": "https://cwe.mitre.org/data/definitions/400.html", "role": "base", "from": "unconstrained-inputs"},
                        {"id": "CWE-130", "name": "Improper Handling of Length Parameter Inconsistency", "url": "https://cwe.mitre.org/data/definitions/130.html", "role": "base", "from": "async-io-misuse"},
                        {"id": "CWE-240", "name": "Improper Handling of Inconsistent Structural Elements", "url": "https://cwe.mitre.org/data/definitions/240.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: read fixed-size chunks from a stream with no framing contract.
# (async I/O misuse x unconstrained inputs)
async def read_msg(reader):
    return await reader.read(1024)     # message length is unknown / unbounded
# a 1500-byte message splits across two reads; a 300-byte one merges with the
# next. Records run together and get persisted mis-framed -- corrupt at rest.

# RIGHT: frame the stream; read exactly the declared length.
async def read_msg(reader):
    header = await reader.readexactly(4)
    n = int.from_bytes(header, "big")
    if n > MAX_MSG:
        raise ValueError("frame too large")
    return await reader.readexactly(n)   # exact message boundary, bounded""",
                         "explanation": "A byte stream has no inherent message boundaries; reading fixed chunks with no length prefix lets one record split and the next merge. The mis-framed bytes are written to the store and read back as garbage -- durable corruption, not a transient glitch.",
                         "output": "unframed: messages split/merge -> mis-framed records persisted corrupt\nframed:   exact length read; each record stored intact"},
                    ],
                },
                {
                    "title": "Stack Smash via Unbounded Recursion",
                    "slug": "stack-smash",
                    "intent": "Recursion depth driven by attacker-shaped nested input runs off the stack and corrupts adjacent memory.",
                    "ingredients": ["stack-overflow", "unconstrained-inputs"],
                    "cwes": [
                        {"id": "CWE-400", "name": "Uncontrolled Resource Consumption", "url": "https://cwe.mitre.org/data/definitions/400.html", "role": "base", "from": "unconstrained-inputs"},
                        {"id": "CWE-674", "name": "Uncontrolled Recursion", "url": "https://cwe.mitre.org/data/definitions/674.html", "role": "base", "from": "stack-overflow"},
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""/* SMELL: recursion depth driven by attacker-shaped nested input.
   (stack-overflow x unconstrained-inputs) */
void parse(Node *n) {
    char scratch[4096];              /* a frame, per level of nesting */
    if (n->child) parse(n->child);   /* depth == nesting depth of input */
    use(scratch, n);
}
/* deeply nested input blows past the stack guard page -> smashes adjacent
   memory / crashes; in C this is memory corruption, not a clean exception. */

/* RIGHT: bound the depth at the boundary (and prefer iteration). */
void parse(Node *n, int depth) {
    if (depth > MAX_DEPTH) { reject(); return; }
    if (n->child) parse(n->child, depth + 1);
}""",
                         "explanation": "Each recursive call adds a frame; input nested deeper than the stack allows runs off the end. Unlike a managed runtime's clean error, in C the overrun corrupts whatever lies past the stack -- a durable, exploitable memory bug. Bound the nesting depth at the boundary.",
                         "output": "unbounded: deep nesting overruns the stack -> memory corruption / crash\nbounded:   input past MAX_DEPTH rejected before recursing"},
                    ],
                },
                {
                    "title": "NaN Poisons a Shared Aggregate",
                    "slug": "nan-shared-aggregate",
                    "intent": "A single divide-by-zero feeds NaN into a shared running total; NaN is absorbing, so every later reader gets garbage forever.",
                    "ingredients": ["divide-by-zero", "threading-mutexes"],
                    "cwes": [
                        {"id": "CWE-369", "name": "Divide By Zero", "url": "https://cwe.mitre.org/data/definitions/369.html", "role": "base", "from": "divide-by-zero"},
                        {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: a /0 produces NaN that lands in a shared accumulator.
# (divide-by-zero x threading / mutexes)
import math
stats = {"sum": 0.0}
def record(latency, n):
    with lock:
        stats["sum"] += latency / n      # n == 0 -> NaN; NaN + x == NaN
# one bad sample (n == 0) makes stats["sum"] NaN forever; every reader of the
# shared aggregate now gets NaN -- the dashboard is permanently wrong.

# RIGHT: reject the non-finite value before it touches shared state.
def record(latency, n):
    if n == 0: return
    v = latency / n
    if not math.isfinite(v): return
    with lock:
        stats["sum"] += v""",
                         "explanation": "NaN is absorbing: once a single divide-by-zero feeds NaN into the shared running total, every later add stays NaN and every reader downstream sees garbage. The lock makes the write thread-safe but cannot un-poison the value -- the corruption is permanent and shared. Validate the numeric domain before it crosses into shared state.",
                         "output": "poisoned: one n==0 sample -> shared sum is NaN for all readers, forever\nright: non-finite values rejected; the shared aggregate stays valid"},
                    ],
                },
                {
                    "title": "Type-Confusion Memory Corruption",
                    "slug": "type-confusion-corruption",
                    "intent": "A value read as one type but used as another writes through a bad pointer, corrupting whatever shared memory sits at that address.",
                    "ingredients": ["type-errors", "pointer-mismanagement"],
                    "cwes": [
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "base", "from": "pointer-mismanagement"},
                        {"id": "CWE-843", "name": "Access of Resource Using Incompatible Type ('Type Confusion')", "url": "https://cwe.mitre.org/data/definitions/843.html", "role": "base", "from": "type-errors"},
                        {"id": "CWE-119", "name": "Improper Restriction of Operations within the Bounds of a Memory Buffer", "url": "https://cwe.mitre.org/data/definitions/119.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""// SMELL: an integer id reinterpreted as a pointer, then written through.
// (type errors x pointer mismanagement)
union { long handle; char *ptr; } u;
u.handle = read_id();              // really an integer id from input
memcpy(u.ptr, payload, len);       // ...used as a pointer -> writes to a wild
                                   // address, corrupting adjacent shared state

// RIGHT: keep the types distinct; resolve the id to a checked object.
long id = read_id();
Object *o = lookup(id);            // id -> a real, bounds-checked object
if (o) memcpy(o->buf, payload, MIN(len, sizeof o->buf));""",
                         "explanation": "Treating an integer id as a pointer is an incompatible-type access (CWE-843); the write through it lands at an out-of-bounds address, silently corrupting objects other code later reads. Keep the types separate and resolve the id to a checked object before any write.",
                         "output": "confusion: bytes written to a wild address -- adjacent state corrupted\nright: id resolved to a bounds-checked object; the write stays in bounds"},
                    ],
                },
                {
                    "title": "Torn Write / Lost-Update Corruption",
                    "slug": "torn-write",
                    "intent": "Two threads write one shared buffer at an unvalidated offset with no lock; their writes overlap and run past the end, leaving permanent corruption no single thread would produce.",
                    "ingredients": ["race-conditions", "lack-of-input-validation"],
                    "cwes": [
                        {"id": "CWE-366", "name": "Race Condition within a Thread", "url": "https://cwe.mitre.org/data/definitions/366.html", "role": "base", "from": "race-conditions"},
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "lack-of-input-validation"},
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""// SMELL: concurrent writers, unvalidated offset, no lock.
// (race conditions x lack of input validation)
char buf[256];
void write_at(int off, const char *s, int n) {
    memcpy(buf + off, s, n);      // off/n come from input, unchecked;
}                                 // concurrent calls overlap -> torn, OOB write

// RIGHT: validate the bounds, then serialize the writers with a lock.
void write_at(int off, const char *s, int n) {
    if (off < 0 || n < 0 || off + n > (int)sizeof buf) return;
    pthread_mutex_lock(&m);
    memcpy(buf + off, s, n);
    pthread_mutex_unlock(&m);
}""",
                         "explanation": "An unvalidated offset plus no synchronization lets two writers interleave on one buffer; their writes overlap and spill past the end, leaving the shared buffer corrupted in a way no single thread would reach. Validate the bounds and serialize the writers.",
                         "output": "race: concurrent unchecked writes overlap -> buffer corrupted (intermittent)\nright: bounds checked + writes serialized -> every write lands intact"},
                    ],
                },
                {
                    "title": "Cross-Session Contamination",
                    "slug": "cross-session-contamination",
                    "intent": "Per-request data written into a shared singleton with no validation lets one user's input overwrite the field another user then reads.",
                    "ingredients": ["cross-boundary-state-exposure", "lack-of-input-validation"],
                    "cwes": [
                        {"id": "CWE-20", "name": "Improper Input Validation", "url": "https://cwe.mitre.org/data/definitions/20.html", "role": "base", "from": "lack-of-input-validation"},
                        {"id": "CWE-567", "name": "Unsynchronized Access to Shared Data in a Multithreaded Context", "url": "https://cwe.mitre.org/data/definitions/567.html", "role": "base", "from": "cross-boundary-state-exposure"},
                        {"id": "CWE-488", "name": "Exposure of Data Element to Wrong Session", "url": "https://cwe.mitre.org/data/definitions/488.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: per-request data written into a shared singleton, unguarded.
# (cross-boundary state exposure x lack of input validation)
class Config:                        # one shared instance for the whole process
    locale = "en"
cfg = Config()
def handle(req):
    cfg.locale = req.params["locale"]   # unvalidated, and shared by all requests
    return render(cfg)                  # another request's locale bleeds in here

# RIGHT: validate, and keep per-request state out of shared singletons.
ALLOWED = {"en", "es", "fr"}
def handle(req):
    loc = req.params.get("locale", "en")
    if loc not in ALLOWED:
        loc = "en"
    return render(locale=loc)           # local to this request; nothing shared""",
                         "explanation": "A singleton holds one slot for a value every concurrent request writes; with no validation and no isolation, one user's input overwrites the field another user then reads -- their data crosses the session boundary. Validate the input and keep per-request state with the request.",
                         "output": "shared: request B renders with request A's locale (and vice versa)\nright: validated, per-request value -- no state crosses the boundary"},
                    ],
                },
                {
                    "title": "Unchecked Sentinel Corrupts a Shared Record",
                    "slug": "unchecked-sentinel-corruption",
                    "intent": "A 'not found' sentinel return used unchecked as a write offset writes past the start of a shared file -- the same unchecked-return defect that elsewhere becomes the CWE-690 null dereference.",
                    "ingredients": ["missing-undefined-returns", "file-network-access"],
                    "cwes": [
                        {"id": "CWE-252", "name": "Unchecked Return Value", "url": "https://cwe.mitre.org/data/definitions/252.html", "role": "base", "from": "missing-undefined-returns"},
                        {"id": "CWE-787", "name": "Out-of-bounds Write", "url": "https://cwe.mitre.org/data/definitions/787.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""// SMELL: an unchecked -1 'not found' return used as a write offset.
// (missing/undefined returns x file/network access)
long off = index_of(key);         // returns -1 when key is absent -- unchecked
pwrite(fd, rec, sizeof rec,
       off * sizeof rec);         // off == -1 -> writes before the file start,
                                  // corrupting the shared record

// RIGHT: check the sentinel before trusting it as an offset.
long off = index_of(key);
if (off < 0) return -1;           // absent -> append or error, never write blind
pwrite(fd, rec, sizeof rec, off * sizeof rec);""",
                         "explanation": "index_of() signals 'not found' with -1, but the caller never checks it; using -1 as a record offset writes past the start of the shared file, corrupting it for every later reader. The same unchecked-return defect becomes a null dereference in the CWE-690 chain (Runtime Errors) and a bad write here. Check the sentinel before trusting it.",
                         "output": "unchecked: a -1 'not found' used as offset -> shared file corrupted\nright: sentinel checked -> no blind write; the record stays intact"},
                    ],
                },
                {
                    "title": "Lossy Coercion Poisons a Shared Ledger",
                    "slug": "lossy-coercion-ledger",
                    "intent": "A value silently narrowed to the wrong type is added to a shared running total; the lock keeps the write safe but cannot restore the meaning the coercion threw away.",
                    "ingredients": ["type-errors", "threading-mutexes"],
                    "cwes": [
                        {"id": "CWE-704", "name": "Incorrect Type Conversion or Cast", "url": "https://cwe.mitre.org/data/definitions/704.html", "role": "base", "from": "type-errors"},
                        {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "Python", "slug": "python", "code":
"""# SMELL: cents truncated to int before adding to a shared ledger.
# (type errors x threading & mutexes)
with lock:
    ledger["cents"] += int(amount)     # amount=10.99 -> 10; 0.99 lost every add
# the lock serializes the write, but the truncation is already baked in;
# the shared total drifts low and can never be reconciled.

# RIGHT: keep an exact integer-cents type end to end; never coerce lossily.
cents = round(Decimal(amount) * 100)   # exact, total-preserving
with lock:
    ledger["cents"] += cents""",
                         "explanation": "int(amount) discards the fractional cents before the value ever reaches the shared ledger. The mutex makes the increment thread-safe, but it locks in an already-wrong number; every contributor under-reports and the shared total is permanently, unreconcilably low. Carry an exact type (integer cents / Decimal) so no meaning is lost at the coercion.",
                         "output": "lossy: the shared total drifts low by the truncated remainder, forever\nright: exact integer cents; the aggregate reconciles to the penny"},
                    ],
                },
                {
                    "title": "Signed/Unsigned Conversion Poisons a Shared Aggregate",
                    "slug": "signedness-poisons-aggregate",
                    "intent": "A value that can be negative is converted to unsigned before being folded into a shared, contended total -- the conversion turns it into a huge positive number that permanently skews the aggregate.",
                    "ingredients": ["type-errors", "resource-contention"],
                    "cwes": [
                        {"id": "CWE-195", "name": "Signed to Unsigned Conversion Error", "url": "https://cwe.mitre.org/data/definitions/195.html", "role": "base", "from": "type-errors"},
                        {"id": "CWE-682", "name": "Incorrect Calculation", "url": "https://cwe.mitre.org/data/definitions/682.html", "role": "compound"},
                    ],
                    "languages": [
                        {"name": "C", "slug": "c", "code":
"""// SMELL: a signed delta cast to unsigned before joining a shared counter.
// (type errors x resource contention)
int delta = price_after - price_before;   // can be negative
shared_total += (size_t)delta;            // -5 -> ~1.8e19; the total is garbage
// many writers contend on shared_total; one negative delta poisons it for all.

// RIGHT: keep signedness consistent end to end; aggregate as signed.
long long delta = price_after - price_before;
shared_total += delta;                    // negatives subtract correctly""",
                         "explanation": "Casting a legitimately-negative value to an unsigned type implicitly maps it to a huge positive number (CWE-195); folded into a shared running total under contention, that one conversion produces an incorrect calculation (CWE-682) every reader of the aggregate now inherits -- permanent, not a crash. Keep the numeric type signed (and wide enough) from input through aggregation.",
                         "output": "unsigned: one negative delta -> the shared total jumps by ~1.8e19, forever\nright: signed aggregation -> negatives subtract correctly; the total stays valid"},
                    ],
                },
            ],
        },
    ],
})

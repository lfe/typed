# Milestone M2 — Pattern Matching, Exhaustiveness & the Diagnostic Engine

> **Goal:** typed pattern matching (`case/typed`) with **exhaustiveness checking
> that *rejects* non-exhaustive matches** — naming the missing constructors — through
> a real, reusable **diagnostic engine** (human + machine-readable). This is where the
> project's thesis lands: the thing Dialyzer cannot do for LFE (Audit 1 §6).
> **Builds on:** [M0 (closed)](M0-skeleton.md), [M1 (closed)](M1-adts-ledger.md) — the
> chain, line injection, ADTs, constructors, repr backends.
> **Design refs:** [design v0](../01-design-v0.md) §3.2a (tiers), §7 (checks), §8
> (diagnostics); [Audit 3](../audits/03-adts-in-other-typed-lisps.md) §6 (exhaustiveness),
> §7 (diagnostics — Gleam is the bar).
> **Ledger:** [M2-...-ledger.md](M2-matching-exhaustiveness-ledger.md). **CC prompt:**
> [M2-cc-prompt.md](M2-cc-prompt.md). **Iteration budget:** 5.

## Why M2 is the milestone that matters

M1 lets you *build* ADT values. M2 lets you *take them apart safely* — and turns the
checker from "rejects malformed constructions" into "**rejects programs that forget a
case**," with an error message good enough to teach the fix. That rejection is the
core value proposition (Dialyzer is optimistic and never rejects; it's also unreliable
for LFE once macros enter — Audit 1 §6). And the **diagnostic engine** built here is
the reusable foundation every later message rides on, including the runtime type
errors of M4.

## In scope

- **`case/typed`** surface: `(case/typed Scrutinee (Pattern Body…) …)`.
- **Patterns (top-level):** constructor patterns binding fields (`(Ok v)` /
  `(Shipped tracking)`), nullary (`(Pending)`), wildcard `_`, and variable patterns.
- **Field access via patterns** (the deconstruction M1 deferred): bound field vars are
  usable in the clause body.
- **Scrutinee type resolution (minimal):** determine the scrutinee's ADT type from
  (a) a contract-typed `defun/typed` argument in scope, or (b) an explicit annotation
  on the `case/typed`. (Establishes the contract-`:args` → body type-env link — the
  seed M3 grows into full checking.) If the type can't be resolved → a clear
  "can't check exhaustiveness: unknown scrutinee type" diagnostic.
- **Exhaustiveness checking (the thesis):** verify the clauses cover *every*
  constructor of the scrutinee's sum (or have a `_`/var catch-all). **Non-exhaustive ⇒
  rejection**, naming **every** missing constructor.
- **Pattern well-formedness:** a pattern constructor not in the scrutinee's type, or
  wrong field/arity in a pattern → diagnostic (the deconstruction analog of M1-4).
- **Repr-aware match lowering** across backends: `case/typed` lowers to a plain LFE
  `case` over the carrier (tagged-tuple `{ok, V}` / enum atom / transparent payload /
  native-record patterns — native-record runtime deferred to 29+). Cross-backend
  **matrix** with **exact** assertions.
- **The diagnostic engine** (real + reusable): source span + caret underline,
  "these values are not matched: …" list, an actionable **Hint:**, alias-aware type
  rendering, and **multi-error collection**. Refactor M0/M1's ad-hoc diagnostics onto
  it.
- **Machine-readable diagnostics** (`--format json` or similar): the same diagnostics
  as structured data (code, span, severity, message, missing-ctors, hint) for LLM/tool
  consumption — the machine half of Goal 2.
- **Golden-snapshot diagnostic tests:** assert the **exact** rendered output (human +
  JSON). *(The M1 lesson: tests must assert exact output, not "an error happened.")*
- **Line/column precision** for all M2 diagnostics; M0/M1 line injection preserved.

## Should (do if it fits the budget; else defer with rationale)

- **Redundant / unreachable clause** warning (duplicate constructor; clause after a
  catch-all).

## Out of scope (later)

- **Nested-pattern exhaustiveness** (the full Maranget pattern-matrix algorithm),
  literal/range/guard-aware exhaustiveness, or-patterns — M2 does **top-level sum**
  exhaustiveness only. (Nested *matching* may work via lowering, but nested
  *exhaustiveness analysis* is deferred.)
- **Full expression typing** (typing arbitrary body expressions) — M3 (contracts).
- **Runtime type enforcement** (guards + validators at the membrane) — M4.

## Definition of done

Every ledger row final with SHA-anchored, reproducible (CI-green) evidence. The
headline — `case/typed` rejects a non-exhaustive match with an exact, teaching-grade
diagnostic naming the missing constructors — is `done`, snapshot-tested, and works
across the testable backends. The diagnostic engine is real and reused by earlier
messages. Native-record matching runtime stays `deferred` (OTP 29+).

## Size warning

This is plausibly the **largest** milestone (matching + exhaustiveness analysis + a
real diagnostic engine + JSON mode). If it reaches iteration 4–5, **split** rather than
grind: a natural cut is **M2** (matching + exhaustiveness + human diagnostics) and
**M2.5** (JSON mode + redundancy + the diagnostic-engine refactor of older messages).
Propose the split; don't blow the cap.

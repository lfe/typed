# M2 — Claude Code implementation prompt

> Paste into Claude Code from the `typed` project root. Implements M2 (matching +
> exhaustiveness + diagnostic engine) against the ledger, under ledger discipline.
> Builds on closed M0 + M1.

```
You are implementing Milestone M2 ("Pattern Matching, Exhaustiveness & the Diagnostic
Engine") of the `typed` project. You are CC (implementer) under LEDGER DISCIPLINE.
M0 and M1 are CLOSED (chain + line injection; ADTs + constructors + repr backends +
registry). Build ON them. This is the thesis milestone: rejecting non-exhaustive
matches with teaching-grade errors.

# Read first (then STOP and confirm scope before coding)
1. docs/design/milestones/M2-matching-exhaustiveness.md       (scope; in/out; size warning)
2. docs/design/milestones/M2-matching-exhaustiveness-ledger.md (criteria M2-1..M2-13)
3. docs/design/01-design-v0.md §3.2a, §7, §8                  (tiers, checks, diagnostics)
4. docs/design/audits/03-adts-in-other-typed-lisps.md §6, §7  (exhaustiveness; Gleam = the bar)
5. checker/src/lower.rs, adt.rs, type_env.rs                  (M1's construction side to mirror)
6. test/typed_adt_SUITE.lfe, lfe/test/example_SUITE.lfe      (LFE CT style — CT suites are LFE)

# Ledger discipline (in force)
- The ledger IS the spec (M2-1..M2-13). Work against it; don't silently drop/reshape.
  Amendments require written justification.
- Fill Status + Evidence (commit SHA + reproduced output; CI must be green) as rows land.
- Iteration budget 5. THIS IS THE BIGGEST MILESTONE — if you reach iteration 4–5, STOP
  and PROPOSE A SPLIT (M2 = matching + exhaustiveness + human diagnostics; M2.5 = JSON
  mode + redundancy + refactoring old messages onto the engine). Do not blow the cap.
- Per-row walk M2-1..M2-13 at close. Name uncertainty. Leave CDC section for CDC.

# Scope (do exactly this)
IN: case/typed (top-level constructor/nullary/wildcard/var patterns; bind fields);
scrutinee-type resolution from contract :args (+ explicit annotation); EXHAUSTIVENESS
rejection naming every missing ctor; pattern well-formedness; repr-aware match lowering
+ matrix; the reusable diagnostic engine; machine-readable JSON diagnostics; golden
snapshots; field access via patterns; line/col precision + regression.
OUT (do NOT build): nested/literal/guard exhaustiveness (Maranget) — top-level sum only;
full expression typing (M3); runtime enforcement/guards/validators (M4); or-patterns.

# What to build
1. typed-check (Rust):
   - parse `(case/typed Scrutinee (Pattern Body...) ...)`; patterns = `(Ctor field...)`,
     `(Ctor)`, `_`, or a var. Bind field vars into the clause body scope.
   - resolve the scrutinee's ADT type: if it's a var bound by the enclosing defun/typed
     `:args`, use that type; else an explicit annotation; else emit "unknown scrutinee
     type — can't check exhaustiveness". (Thread :args types into a body type env — the
     minimal seed; do NOT build full expression typing, that's M3.)
   - EXHAUSTIVENESS: compare clause-covered constructors against the sum's full ctor set;
     missing (and no catch-all) ⇒ REJECT listing EVERY missing ctor.
   - pattern well-formedness: unknown ctor / wrong field / wrong arity in a pattern ⇒
     Tier-1 diagnostic with exact line:col.
   - lower case/typed to plain LFE `case` over the chosen repr (mirror M1's lowering);
     preserve field bindings; snake_case tags (reuse to_snake_case).
   - (should) redundant/unreachable clause warning.
2. THE DIAGNOSTIC ENGINE (make it real and central):
   - one module that renders: source span + caret underline, a "not matched: <ctors>"
     section, an actionable `Hint:`, alias-aware type names, and collects MULTIPLE
     errors per run. Refactor M0/M1's ad-hoc diagnostic strings to go through it.
   - add a `--format json` mode emitting structured diagnostics (code, span, severity,
     message, missing-ctors, hint). Same content as human form. (If budget tight, this
     is the M2.5 split candidate — but try to land it.)
3. Fixtures (.tlfe): a non-exhaustive case/typed (missing 2 ctors, distinctive lines);
   an exhaustive-with-wildcard case; pattern-error fixtures (unknown ctor / wrong field
   / wrong arity in a pattern); a field-access case; matrix fixtures per backend.
4. Tests — CT in **LFE** (`test/*_SUITE.lfe`) + Rust unit tests. CRUCIAL (M1 lesson):
   assert the EXACT rendered diagnostic via golden snapshots (human + JSON), not "an
   error occurred". Matrix tests assert EXACT runtime reps. Include a line-injection
   regression through case/typed.
5. CI: matrix green (0 skipped); make check clean. native-record matching runtime stays
   `deferred` (OTP 29+) with a note.

# Environment
OTP 28 / LFE 2.2.1. Reuse M1's to_snake_case + lowering patterns + the EETF/driver chain.
native-record matching code may be written but its runtime row is `deferred` (29+).

# Definition of done
M2-1..M2-13 final with SHA + CI-green evidence (or justified deferred/no-op/should-deferred).
Headline: case/typed rejects a non-exhaustive match with an EXACT, snapshot-tested,
teaching-grade message naming all missing ctors, across the testable backends; the
diagnostic engine is real and reused. Per-row walk at close; CT shows Skipped=0.

Do NOT expand scope beyond the ledger.
```

# M3 — Claude Code implementation prompt

> Paste into Claude Code from the `typed` project root. Implements M3 (contracts +
> bidirectional body checking) against the ledger, under ledger discipline. Builds on
> closed M0/M1/M2.

```
You are implementing Milestone M3 ("Contracts & Bidirectional Body Checking") of the
`typed` project. You are CC (implementer) under LEDGER DISCIPLINE. M0/M1/M2 are CLOSED
(chain + line injection; ADTs + constructors + repr backends; matching + exhaustiveness
+ the diagnostic engine). Build ON them. This is the first real EXPRESSION typer:
checking a defun/typed body against its contract.

# Read first (then STOP and confirm scope before coding)
1. docs/design/milestones/M3-contracts.md            (scope; approach; in/out; size warning)
2. docs/design/milestones/M3-contracts-ledger.md     (criteria M3-1..M3-14)
3. docs/design/01-design-v0.md §6, §3.2a, §8         (bidirectional; no global inference; dynamic; diagnostics)
4. checker/src/{matching.rs,match_lower.rs,diagnostic.rs,adt.rs,type_env.rs,typed_surface.rs}  (reuse!)
5. test/typed_matching_SUITE.lfe, lfe/test/example_SUITE.lfe   (LFE CT style)

# Ledger discipline (in force)
- The ledger IS the spec (M3-1..M3-14). Work against it; amendments need written
  justification. Fill Status + Evidence (SHA + reproduced output; CI green) as rows land.
- Iteration budget 5. BIG MILESTONE — if you reach iter 4–5, STOP and PROPOSE A SPLIT
  (M3 = synth/check core + return/arg/field mismatches + minimal prelude; M3.5 =
  polymorphic unification + prelude expansion + more expression forms). Don't blow the cap.
- Per-row walk M3-1..M3-14 at close. Name uncertainty. Leave CDC section for CDC.

# Approach: BIDIRECTIONAL, contract-first (NOT global Hindley-Milner inference)
- synth(expr) -> Type for: literals (int/float/string=[char]/binary/atom/boolean),
  variables (arg env + let + case/typed bindings), if, let/let*, calls to TYPED funcs
  (-> their :returns), constructor applications (-> ADT type), case/typed.
- check(expr, expected) for: the body vs :returns; each call arg vs the param type;
  each case/typed branch vs the expected result type.
- types compatible: a type checks against itself; `dynamic` is compatible with anything
  (gradual). Unknown calls/ops -> dynamic. NO global inference, NO let-polymorphism.

# Scope (do exactly this)
IN: synth + check for the core forms above; body-vs-:returns rejection (headline); call
arg + arity checking; constructor FIELD-VALUE checking (M1 follow-through — concrete
field types; parametric simple); case/typed branch typing (integrate M2); a MINIMAL
documented built-in prelude (arith/compare/++ + a few); dynamic() boundary (STATIC, no
runtime checks); diagnostics via the M2 engine (human+JSON, EXACT snapshots); line/col
precision + full M0/M1/M2 regression; type the README describe example.
OUT (do NOT build): full HM/global inference/let-poly; full parametric unification (basic
only); the full BIF prelude; bit-syntax/comprehension/HOF/guard/tuple-record typing;
RUNTIME enforcement (guards/validators — that's M4); effect typing.

# What to build
1. typed-check (Rust): an expression typer (e.g. checker/src/typecheck.rs) with synth +
   check as above, threading a type env (contract :args + let + pattern bindings). Use
   the existing adt/type_env for type/ctor lookup. Resolve a call's signature from other
   defun/typed contracts; unknown -> dynamic.
2. Built-in prelude: a small documented signature table (a module or data file). Keep it
   tiny; list exactly what's in it in M3-contracts.md or a comment.
3. Field-value checking: when lowering/checking a construction, check each field value's
   synthesized type against the declared field type (reuse M1's adt defs).
4. Diagnostics: route ALL M3 type errors through diagnostic::DiagnosticCollector
   (expected vs got, span, hint), human + JSON. Add EXACT golden snapshot tests
   (assert_eq! full render) for: return-type mismatch, arg-type mismatch, field-value
   mismatch. (The M1/M2 lesson: assert EXACT output, not "an error".)
5. Fixtures (.tlfe): return-mismatch, arg-mismatch (+ wrong arity), field-value-mismatch,
   a well-typed function that passes, an if/let example, a dynamic-boundary example
   (typed fn calling an untyped fn), and the README order-status `describe` (both the
   correct constructor version — passes — and the wrong strings version — rejected).
6. Tests — CT in LFE (test/*_SUITE.lfe) + Rust unit tests. Matrix/M0/M1/M2 suites must
   ALL still pass (full regression). Assert EXACT diagnostics via snapshots.
7. CI green (0 skipped); make check clean.

# Environment
OTP 28 / LFE 2.2.1. Reuse the diagnostic engine, adt/type_env, matching, lowering, and
the EETF/driver chain. Typing is representation-independent (don't re-do the matrix;
just keep it green).

# Definition of done
M3-1..M3-14 final with SHA + CI-green evidence (or justified deferred/no-op/should-deferred).
Headlines: body-vs-:returns, call-arg, and constructor-field mismatches are each REJECTED
with an EXACT, snapshot-tested, teaching-grade diagnostic; the README describe example
type-checks (correct) / is rejected (strings). Full M0/M1/M2 regression green. Per-row
walk at close; CT shows Skipped=0.

Do NOT expand scope beyond the ledger.
```

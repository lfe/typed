# Milestone M3 — Contracts & Bidirectional Body Checking

> **Goal:** check a `defun/typed` **body** against its contract — synthesize the
> types of expressions and verify the body has the declared `:returns` type and that
> calls/constructions receive correctly-typed arguments. This is the first time the
> checker reasons about **values flowing through code**, not just constructor/pattern
> *shapes*. It also closes M1's deferred **field-value** checking and integrates M2's
> `case/typed` branch typing.
> **Builds on:** M0/M1/M2 (all closed) — chain, line injection, ADTs, constructors,
> repr backends, matching, exhaustiveness, the diagnostic engine.
> **Design refs:** [design v0](../01-design-v0.md) §6 (system: bidirectional, no global
> inference, `dynamic()` boundary), §3.2a (tiers), §8 (diagnostics).
> **Ledger:** [M3-contracts-ledger.md](M3-contracts-ledger.md). **CC prompt:**
> [M3-cc-prompt.md](M3-cc-prompt.md). **Iteration budget:** 5.

## The headline

After M3, this is a **compile error**:

```lisp
(defun/typed make-greeting
  (:args ((name binary)))
  (:returns binary)
  (:body 42))            ; ← body is integer, not binary
```

…and so is calling a typed function with a wrong-typed argument, or giving a
constructor a wrong-typed field value. Each rejection is a teaching-grade diagnostic
(expected vs. got, exact span, hint) through the M2 engine. *This is the moment `typed`
feels like a real type checker.*

## Approach: bidirectional, contract-first (not global inference)

Per the design (§6): no Hindley-Milner global inference. Two modes:

- **Synthesize** a type for an expression: literals → their type; variables → their
  bound type; a call to a typed function → its `:returns`; a constructor application →
  its ADT type; etc.
- **Check** an expression against an *expected* type: the body against `:returns`; each
  call argument against the parameter type; each `case/typed` branch against the
  expected result type.

Types come from contracts (`:args`/`:returns`), `deftype` declarations, `let`/pattern
bindings, and a **small built-in prelude**. Anything the checker can't type resolves to
`dynamic()` — the gradual escape hatch (static only here; *enforcement* is M4).

## In scope

- **Type synthesis** for the core expression forms: literals (integer, float, string
  `[char]`, binary, atom, boolean), variables (arg env + `let`/`let*` + `case/typed`
  pattern bindings), `if`, `let`/`let*`, calls to **typed** functions, **constructor
  applications**, and `case/typed`.
- **Check body against `:returns`** — the headline return-type-mismatch rejection.
- **Check call arguments** against a typed function's parameter types (+ arity).
- **Constructor field-value checking** (M1 follow-through): a field value whose type
  doesn't match the constructor's declared field type is rejected. (Concrete field
  types fully; type-variable/parametric fields handled simply — see Should/Out.)
- **`case/typed` branch typing:** each clause body checked against the expected type;
  the match's result type synthesized. Integrates with M2 exhaustiveness.
- **A minimal built-in prelude:** a small, **documented** signature table — arithmetic
  (`+ - * div rem` on numbers), comparison (`== < > =< >=` → boolean), `++` (list/string
  append), and a handful of common ones. Extensible. Everything not in it → `dynamic`.
- **`dynamic()` boundary (static):** calls to untyped/unknown functions synthesize
  `dynamic`; `dynamic` is compatible with any expected type (gradual). **No runtime
  checks** (that's M4).
- **Diagnostics** for all of the above via the M2 `DiagnosticCollector` (expected/got,
  span, hint), human + JSON, with **exact golden snapshots** for the headlines.
- **Line/col precision**; full **M0/M1/M2 regression** (everything still green); line
  injection preserved.

## Should (do if budget allows; else defer with rationale)

- **Basic polymorphic contracts:** identity-style type variables in `:args`/`:returns`
  (`(:args ((x a))) (:returns a)`) checked consistently. Full unification deferred.

## Out of scope (later)

- **Full Hindley-Milner / global inference / let-polymorphism.**
- **Full parametric unification** (rich generic instantiation across nested types).
- **The full Erlang/LFE BIF prelude** — only the minimal documented set.
- **Bit-syntax/binary typing, list/binary comprehensions, higher-order function
  typing, guard typing, tuple-record field typing.**
- **Runtime type enforcement** (guards + validators at the membrane) — M4
  ([[typed-runtime-enforcement]] in memory).
- Effect/exception typing.

## Definition of done

Every ledger row final with SHA-anchored, CI-green evidence. The headline — a
`defun/typed` whose body type ≠ `:returns` is rejected with an exact, snapshot-tested,
teaching-grade diagnostic; ditto wrong-typed call args and constructor fields — is
`done` and works. The README `describe` example type-checks as a demo fixture.

## Size warning

Expression typing is a big conceptual step (comparable to M2). If it reaches iteration
4–5, **split** rather than grind: a natural cut is **M3** (synth/check core + return +
arg + field mismatches + minimal prelude) and **M3.5** (polymorphic unification +
prelude expansion + more expression forms). Propose the split; don't blow the cap.

# Milestone M5 — Polish & Dogfood on Real LFE

> **Goal:** point `typed` at a *real, non-toy* LFE program, end-to-end, and let reality
> grade the design. This is the **design-completeness oracle** the project was built
> toward: if we can't comfortably type real LFE, the type system isn't good enough yet.
> Plus the first user-facing polish — a getting-started doc and a clean `rebar3` UX.
> **Builds on:** M0–M4.6 (all closed) — the full static + runtime type system.
> **Design:** [[typed-project-goals]] (self-application / dogfood is the oracle).
> **Ledger:** [M5-dogfood-ledger.md](M5-dogfood-ledger.md). **CC prompt:**
> [M5-cc-prompt.md](M5-cc-prompt.md). **Iteration budget:** 5.

## Why dogfood (and why now)

Every milestone so far was verified against *fixtures we wrote to exercise a feature*.
That proves each feature works in isolation; it does **not** prove the system is pleasant
or sufficient for a real program with many functions, real control flow, and a real
input boundary. M5 closes that gap by building (and typing) a realistic module and
letting it surface what fixtures couldn't:

- built-ins the prelude is missing,
- forms/expressions the checker can't yet type (forcing `dynamic`),
- ergonomic rough edges in the surface syntax,
- error messages that aren't as teaching-grade as we hoped on non-toy code.

The honest output of dogfooding is a **gap inventory** — and that inventory is the most
valuable artifact of M5, more than any single fix.

## In scope

- **A realistic typed module (non-toy):** a small but genuine domain — e.g. an `orders`
  module — with **several** `defun/typed` functions, **ADTs** (sum-of-products), real
  `case/typed` control flow, a **`decode` boundary** for untyped input, and actual logic
  (not one-liners). It must **check clean, compile, and run** with asserted behavior (CT).
- **A gap inventory** (`docs/design/M5-gap-inventory.md`): every limitation the real
  module surfaced, each classified **fix-now / defer / wontfix** with a one-line rationale.
  This is the oracle's report — completeness matters more than length.
- **Fix the cheap (fix-now) gaps** surfaced — most likely **prelude expansion** (the
  built-in signatures real code needs) and small ergonomics — each with an exact test.
  Defer the rest with rationale into the backlog / later milestones.
- **A getting-started / usage doc** (`docs/usage.md`): add `typed` to a project, write a
  typed module, run the checker, read a type error. The first *user-facing* doc.
- **`rebar3` provider UX polish:** `rebar3 ... typed check` (or the agreed command) gives
  clear output, non-zero exit on failure, and help text; the end-to-end build integration
  works in a sample project.
- **Teaching-grade errors on real code:** breaking the realistic module a few ways
  (wrong return, non-exhaustive `case/typed`, bad `decode` input) produces the good
  diagnostics — verified, exact.
- **Full M0–M4.6 regression**; standing discipline.

## Out of scope (later)

- A *large* real-world codebase port (M5 is one realistic module, not a whole app).
- Native-record runtime (OTP 29+); full HM / global inference; message/process
  enforcement; framework/HTTP integration helpers; performance/guard-elision.
- Hex packaging / `rebar3 add typed` as a published dep (a release milestone of its own).

## Definition of done

A realistic typed module checks/compiles/runs (CT, exact); the gap inventory exists and is
classified; the fix-now gaps are fixed with tests (defer the rest with rationale); a
getting-started doc exists; the provider UX is clean; breaking the module yields
teaching-grade errors; full M0–M4.6 regression green.

## Note: exploratory milestone

Unlike earlier milestones, M5's value is partly *discovered*. The ledger fixes the
*verifiable outcomes* (a real module runs; the inventory exists; the cheap gaps are fixed;
the docs/UX land), but the **specific** gaps and fixes are found during the work. New gaps
become **deferred rows with rationale** (feeding the backlog), not silent drops. If the
realistic module surfaces something big (e.g. a whole class of unsupported forms), that's a
*finding to document*, not a reason to grind — propose a follow-up milestone.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; test the actual subject; unwired ≠ done) ·
[[cc-editing-safety]] (no blind `sed`) · [[lfe-ct-tests-in-lfe]] (CT suites in LFE).

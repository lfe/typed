# M5 — Claude Code implementation prompt (Polish & Dogfood)

> Paste into Claude Code from the `typed` project root. The dogfood milestone: type a real
> non-toy module, harvest a gap inventory, fix the cheap gaps, write the usage doc, polish
> the UX. Builds on closed M0–M4.6.

```
You are implementing Milestone M5 ("Polish & Dogfood on Real LFE") of the `typed` project.
You are CC (implementer) under LEDGER DISCIPLINE. M0–M4.6 are CLOSED — the full static +
runtime type system (ADTs, exhaustiveness, contracts, guards, validators/decode, rendered
errors). This milestone points it at a REAL, non-toy module and lets reality grade the
design. The most valuable output is an honest GAP INVENTORY, not any single fix.

# Read first (then STOP and confirm scope)
1. docs/design/milestones/M5-dogfood.md            (why dogfood; scope; exploratory note)
2. docs/design/milestones/M5-dogfood-ledger.md     (criteria P-1..P-7)
3. README.md (the surface as advertised) + checker/src/* (what actually exists)
4. test/typed_*_SUITE.lfe (LFE CT style)

# STANDING RULES (NON-NEGOTIABLE — typed-test-discipline, cc-editing-safety, lfe-ct-tests-in-lfe)
- Exact assert_eq!/pattern matching, never `.contains()`. Test the actual subject. Unwired
  ≠ done. No blind `sed`. CT suites in LFE.

# Ledger discipline
- Work against P-1..P-7. This is EXPLORATORY: discovered gaps become DEFERRED rows with a
  one-line rationale (never silent drops). If the module surfaces something big, DOCUMENT it
  as a finding and propose a follow-up milestone — don't grind. Budget 5. Per-row walk at
  close; leave CDC section for CDC.

# What to build
1. P-1 REALISTIC MODULE: write a non-toy typed LFE module — e.g. an `orders` domain — with
   SEVERAL `defun/typed` functions, ADTs (sum-of-products, e.g. order-status + an order
   record-ish ADT), real `case/typed` control flow, a `decode` boundary for untyped input,
   and ACTUAL logic (compute totals, transition states, format summaries — not one-liners).
   It must CHECK clean, compile, and RUN. CT (LFE) calls several functions and asserts real
   results end-to-end through the full chain.
2. P-2 GAP INVENTORY: as you write P-1, KEEP A RUNNING LIST of every place the system fell
   short — a built-in the prelude lacks, a form the checker can't type (forced to dynamic),
   an awkward bit of syntax, an error that wasn't teaching-grade. Write it up in
   `docs/design/M5-gap-inventory.md`, each item classified fix-now / defer / wontfix + a
   one-line rationale. Be honest and complete — this is the milestone's point.
3. P-3 FIX THE CHEAP GAPS: implement the fix-now items (most likely prelude expansion — add
   the built-in signatures the module needed; plus small ergonomics), each with an exact
   test. Mark the rest deferred (they feed the backlog / later milestones).
4. P-4 USAGE DOC: `docs/usage.md` — how to add typed to a project, write a typed module, run
   the checker, and read a type error. Walk the REAL example end-to-end; keep commands/output
   in sync with actual behavior.
5. P-5 PROVIDER UX: make `typed check` (the rebar3 command) give clear output, non-zero exit
   on failure, and help text; verify the end-to-end build integration on a good + a bad
   sample project (exit codes asserted).
6. P-6 TEACHING ERRORS ON REAL CODE: break the realistic module 3 ways (wrong return,
   non-exhaustive case/typed, bad decode input) and verify each yields the good diagnostic —
   EXACT.
7. P-7: full M0–M4.6 regression green; make check clean; CI green, 0 skipped.

# Run & evidence
- cd checker && cargo build && cargo test; rebar3 ct; make check. Show Skipped=0.
- Commit (small, logical commits); anchor done rows to the SHA; CI green.

# Definition of done
A realistic typed module checks/compiles/runs (P-1, exact CT); the gap inventory exists +
classified (P-2); fix-now gaps fixed with tests, rest deferred-with-rationale (P-3); usage
doc walks the real example (P-4); provider UX clean with correct exit codes (P-5); breaking
the module yields exact teaching-grade errors (P-6); full regression green (P-7). Per-row
walk at close.

Do NOT expand scope: not a whole-app port, no native-record runtime, no full HM, no hex
packaging, no framework helpers. One realistic module + the honest inventory.
```

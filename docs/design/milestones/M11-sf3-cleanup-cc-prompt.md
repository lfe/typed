# M11 — Claude Code prompt: land SF-3 (per-clause static checking) — iteration 4

> Paste into Claude Code from the `typed` project root. SF-1/2/4/5/6/8/9 are CDC-accepted. SF-7
> is deferred to a future milestone. SF-3 is REOPENED — it's the milestone's headline, not a
> defer: right now only clause 1's body is statically checked. Read the ledger's "## CDC
> Verification" first.

```
You are closing Milestone M11 ("Typed Function Clauses"). ITERATION 4 (of 5). CDC accepted the
runtime/parse rows (SF-1/2/4/5/6/8/9) and deferred SF-7 to a future milestone. SF-3 is REOPENED:
it is the static-checking core of a "typed function clauses" milestone, and it's currently
absent for every clause past the first. Read docs/design/milestones/M11-surface-features-ledger.md
"## CDC Verification".

# The gap (confirmed)
main.rs:304-323 calls check_body_with_case_typed(&tf.body, &tf.returns, ...) on the TOP-LEVEL
fields, which are populated from clause 1. So clauses 2..N bodies are NEVER statically type-
checked. Ackermann's recursive clauses, render's string/atom clauses — all unchecked. The
runtime dispatch (SF-8) works, but that's the runtime half; SF-3 is the missing compile-time
half. A "typed" milestone must check every clause.

# The fix (SF-3): check EVERY clause against ITS OWN contract
For a multi-clause defun/typed, loop over ALL clauses. For each clause:
1. Build a per-clause body-env from THAT clause's `(pattern type)` args:
   - bind each pattern's variables at its declared type (variable pattern binds the var;
     constructor pattern `(Shipped t)` binds `t` at its field type; literal patterns bind
     nothing). REUSE case/typed's existing pattern-binding extraction — don't reinvent.
2. CHECK each pattern is compatible with its declared type (a literal `0` is a valid `int`
   pattern; `'error` a valid `atom`; `(Shipped t)` a valid `order-status` ctor pattern; a
   literal `0` against type `string` is a STATIC ERROR). Exact teaching diagnostic on mismatch.
3. CHECK that clause's body against THAT clause's `:returns`, in the per-clause env.
A clause whose body or pattern violates its contract → STATIC rejection (run the checker binary;
non-zero exit) + exact diagnostic.

# Tests (exact)
- SF-3a body check: a multi-clause fn whose SECOND/Nth clause body returns the wrong type →
  static rejection + exact diagnostic naming that clause (prove it's not just clause 1).
- SF-3b pattern-vs-type: a clause with a literal pattern incompatible with its declared type
  (e.g. `(("" int))`) → static rejection + exact diagnostic.
- Regression: ackermann + the type-dispatch fixture still CHECK CLEAN (they're well-typed) and
  still run exact (SF-8 unbroken).

# STANDING RULES (NON-NEGOTIABLE)
- The subject is STATIC checking of EVERY clause — prove clause 2..N is checked, not just clause
  1 (a deliberately-wrong Nth clause must be caught). Static rejection = non-zero exit + exact
  message, not a runtime proxy. Exact assertions. Preserve M0 positions (the diagnostic must
  point at the offending clause's line). Unwired ≠ done. Status honesty. No blind `sed`;
  `git checkout`; `make check`. CT in LFE.

# Ledger discipline
- Iteration 4 of 5. Don't expand scope (SF-7 stays deferred; no heterogeneous-return overloading).
  Per-row walk at close; leave CDC section intact. Re-anchor SF-3 to the SHA; full regression
  green.

# Definition of a clean close
- Every clause's body checked against its own :returns; every pattern checked against its
  declared type; a wrong Nth clause → exact static rejection (proving past-clause-1 coverage);
  ackermann + type-dispatch still check clean + run exact; make check + full regression green.

Do NOT expand scope: just SF-3 (per-clause static checking) + its exact tests. SF-7 is a future
milestone; no overloading; no sugar.
```

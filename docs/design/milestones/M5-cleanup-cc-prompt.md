# M5 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. P-1/P-2/P-3/P-4 are CDC-verified and
> excellent — the realistic module runs and the gap inventory is honest. But M5 does NOT close
> clean: **P-6 didn't meet its criterion** and **P-5 was mislabeled**. This is a small, bounded
> cleanup. Read the ledger's "## CDC Verification" section first.

```
You are CC closing out Milestone M5 ("Polish & Dogfood"). ITERATION 2 (of 5). CDC verified
P-1/P-2/P-3/P-4 are done + strong. But P-6 is OVERCLAIMED and P-5 is mislabeled. Read
docs/design/milestones/M5-dogfood-ledger.md "## CDC Verification" before starting.

# What's wrong (so you fix the right thing)
P-6's criterion: break the realistic `orders.tlfe` module THREE ways — (a) wrong return type,
(b) non-exhaustive case/typed, (c) bad decode input — and verify EXACT teaching diagnostics.
What exists: only one runtime guard-crash test (wrong ARG, asserting only expected=integer),
plus the P-1 runtime decode tests. MISSING: the non-exhaustive case, the wrong-return case,
and — most importantly — NOT ONE test exercises the STATIC checker REJECTING the real module
with a teaching diagnostic. P-6 is "teaching errors on real code" (Goal 2). The static
teaching path must be tested on real code, exactly.

# The fix
1. P-6a STATIC wrong-return: add a fixture (e.g. test/fixtures/dogfood/orders_bad_return.tlfe)
   — orders.tlfe with one function's :returns deliberately wrong (e.g. line-total :returns
   string). Run the checker binary on it; assert it EXITS NON-ZERO and emits the EXACT
   teaching diagnostic (full rendered string / structured diagnostic — snapshot it, not
   .contains()).
2. P-6b STATIC non-exhaustive: add a fixture (orders_nonexhaustive.tlfe) — status-label with
   a constructor clause removed (e.g. drop (Delivered)). Checker must reject with the EXACT
   exhaustiveness diagnostic naming the missing constructor(s). Exit non-zero. Exact snapshot.
3. P-6c bad decode: KEEP the existing runtime decode tests, but ALSO render one decode error
   through typed_rt:render_type_error and assert the EXACT teaching string (tie it to the
   render helper, so "teaching-grade on real code" is actually shown).
4. P-6 tighten: change p6_wrong_type_crashes to assert the FULL structured error (exact map
   contents: expected, got, function, arg), not just expected=integer.
5. P-5 RELABEL: in the ledger, change P-5 status from "done (caveat)" to "deferred" with the
   one-line rationale already written (no rebar3 integration test asserting exit codes;
   provider exists, verification deferred). Do NOT fake a rebar3 test — deferring honestly is
   correct here. (If a real `typed check` good+bad exit-code CT is cheap, you MAY add it and
   mark P-5 done instead — your call, but honest status either way.)
6. e4 TIGHTEN (carried from M4.6): in test/typed_runtime_SUITE.lfe, e4_render_both_faces
   currently asserts `is_list` on both renders. Replace with EXACT string assertions on both
   the guard-crash render and the validator-return render.

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert_eq!/snapshot, never .contains() or is_list/is_*. Test the ACTUAL subject
  (the static checker rejecting orders.tlfe — not M2 fixtures). Unwired ≠ done. Status honesty:
  a row is done only if its criterion's verification exists.
- No blind `sed`; `git checkout` to recover; `make check` after edits. CT suites in LFE.

# Ledger discipline
- Iteration 2 of 5. Don't expand scope. Per-row walk at close; leave the CDC section intact.
- Anchor changed rows to the new SHA; CI green; full M0–M4.6 + M5 regression green, 0 skipped.

# Definition of a clean close
- P-6 has 3 break-modes on the REAL module, each EXACT, INCLUDING the static checker-rejection
  path (non-zero exit + exact diagnostic) for wrong-return and non-exhaustive.
- p6_wrong_type_crashes asserts the full structured error; one decode error rendered via the
  helper and exact-asserted.
- P-5 honestly labeled (deferred, or done with a real exit-code CT).
- e4 asserts exact strings, not is_list.
- make check clean; CI green; 0 skipped.

Do NOT expand scope: no new language features, no fixing the deferred gap-inventory items, no
record sugar, no when-guards. Just make P-6 meet its criterion, fix the two status/assertion
honesty issues, tighten e4.
```

## Why this matters (for the record)

P-1/P-2 are the milestone's real payoff and they're genuinely good — a non-toy module runs
end-to-end and the gap inventory is honest. The P-6 miss is the recurring pattern (a green
count hiding an unmet criterion), and it lands on the project's *headline* goal: teaching-grade
diagnostics on real code. A dogfood milestone that never once shows the **static** checker
rejecting the realistic module with a teaching error hasn't actually dogfooded Goal 2. Cheap to
fix, important to fix.

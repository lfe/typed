# M3 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. CDC found M3's substance real
> (the headlines work — the checker caught a live bug), but it's not clean-closed. Small
> close-out. Does NOT expand scope.

```
You are CC closing out Milestone M3. ITERATION 2 (of 5). CDC verified the three
headlines work (body-vs-:returns, call-arg, field-value — the checker even caught a real
bug in M0's hello.tlfe). Two corrections remain; the rest is honestly deferred to M3.5.
Read the ledger's "## CDC Verification" section first
(docs/design/milestones/M3-contracts-ledger.md).

# Ledger discipline (in force)
- Iteration 2 of 5. Don't expand scope. Amendments need written justification.
- Every done row: commit SHA + reproduced output; CI green. Per-row walk at close.
- Leave the CDC Verification section intact (CDC re-verifies vs the new SHA).

# STANDING RULE now in force (CDC trend — see memory typed-test-discipline):
- Diagnostic tests MUST assert the EXACT rendered output (assert_eq! / golden snapshot),
  NEVER `.contains()` / substring. This applies to the fix below and all future tests.

# Required corrections (to close M3)

1. M3-10 → EXACT GOLDEN SNAPSHOTS for the three headlines (cheap; this is the one
   genuinely missing thing). Replace the `.contains()` assertions with exact-output
   tests for:
   - M3-3 body-vs-:returns mismatch
   - M3-4 call-arg mismatch (and wrong arity)
   - M3-5 constructor field-value mismatch
   Each test asserts the FULL rendered diagnostic message via `assert_eq!` (or a golden
   snapshot, à la m2_11). The messages already read teaching-grade — just pin them
   exactly so a rendering regression is caught. Set M3-10's snapshot part `done`;
   the full span+caret engine routing may remain DEFERRED to M3.5 (note it).

2. CONFIRM the reclassified statuses are correct in your walk (CDC already set them):
   - M3-6 `deferred` (case/typed branch-body typing not done; M2 exhaustiveness covers
     structural safety; full branch typing → M3.5).
   - M3-14 `deferred` (type-var pass-through, not real checking; full unification → M3.5).
   - M3-11 `deferred` but PRIORITIZE for M3.5: the README `describe` example is the
     project's public face and should actually type-check (correct version passes; the
     strings version is rejected).

# Run & evidence
- cd checker && cargo build && cargo test; rebar3 ct; make check. Show Skipped=0.
- Commit; anchor changed rows' Evidence to the new SHA; confirm CI green.

# Definition of a clean close
- M3-10: exact-snapshot tests for M3-3/4/5 present + green (no `.contains()` on the
  headline messages); engine span+caret routing may be `deferred` to M3.5 with a note.
- M3-6, M3-14 `deferred` (confirmed); M3-11 `deferred` (flagged priority for M3.5).
- Per-row walk complete; CDC Verification section intact.

Do NOT expand scope. No branch-body typing, no engine routing, no poly unification, no
README fixture in THIS pass — those are M3.5. Just the exact snapshots + status honesty.
```

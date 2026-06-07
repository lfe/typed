# M2 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. Small close-out: CDC found M2
> strong and the thesis landing rigorously; two items remain. Does NOT expand scope.

```
You are CC closing out Milestone M2. ITERATION 2 (of 5). CDC verified the thesis lands
rigorously (exact snapshots, exhaustiveness rejection naming all missing ctors) and the
diagnostic engine is real. Two items remain. Read the ledger's "## CDC Verification"
section first (docs/design/milestones/M2-matching-exhaustiveness-ledger.md).

# Ledger discipline (in force)
- Iteration 2 of 5. Don't expand scope. Amendments need written justification.
- Every done row: commit SHA + reproduced output; CI green. End with a per-row walk.
- Leave the CDC Verification section intact (CDC re-verifies vs the new SHA).

# Required corrections

1. M2-9 — TEST enum AND transparent matching (don't assume "backend-identical").
   The match-lowering code for enum/transparent exists and reads correct, but is
   untested — and the matrix exists to PROVE equivalence, not assume it (this is exactly
   what caught M1's casing bug). Add:
   - Rust lowering tests: a `case/typed` over an enum type lowers to atom patterns
     (snake_cased); over a transparent newtype lowers to a bare-value binding pattern.
   - CT runtime tests (LFE, in test/*_SUITE.lfe): build + run a `case/typed` over an
     enum value and over a transparent value, asserting the EXACT runtime result (e.g.
     matching `red` returns X; matching a transparent `(Wrap v)` binds the bare value).
   - Add these to the matrix so all three testable backends (tagged-tuple + enum +
     transparent) have a matching test with exact assertions. Set M2-9 `done`.

2. M2-6 — RESOLVE the eprintln! caveat. The remaining `eprintln!`s in main.rs are
   CLI/IO messages (usage, write-failed, "no defmodule"), NOT type diagnostics, so they
   don't need the DiagnosticCollector. Either:
   - (recommended) mark the "refactor M0/M1 ad-hoc diagnostics" sub-item `no-op` in the
     ledger with the rationale that those eprintln!s are CLI/IO errors, not type
     diagnostics (the engine already handles all type diagnostics); OR
   - route them through the collector anyway if you prefer uniformity.
   Either way, M2-6 closes cleanly (engine = done; sub-item = no-op-with-rationale or done).

# Run & evidence
- `cd checker && cargo build && cargo test`; `rebar3 ct`; `make check`. Show Skipped=0.
- Commit; anchor new/changed rows' Evidence to the new SHA; confirm CI green.

# Definition of a clean close
- M2-9 `done`: enum + transparent matching have exact-assertion tests (Rust + CT);
  matrix covers all three testable backends.
- M2-6 closes cleanly (engine done; eprintln! sub-item no-op-with-rationale or done).
- Per-row walk complete; CDC Verification section left intact.

Do NOT expand scope. Top-level-sum exhaustiveness only; no nested/Maranget, no M3/M4 work.
```

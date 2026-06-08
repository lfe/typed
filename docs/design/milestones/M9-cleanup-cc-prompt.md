# M9 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. D-1/D-5/D-6/D-7/D-8 are CDC-verified.
> D-2/D-3/D-4 proved parse+desugar+synth but NOT compile+run — and "lowers to correct BEAM" is
> exactly their criterion. Read the ledger's "## CDC Verification" first.

```
You are CC closing out Milestone M9 ("Reader Correctness"). ITERATION 2 (of 5). CDC verified
D-1/D-5/D-6/D-7/D-8. D-2/D-3/D-4 are OVERCLAIMED: every test is Rust parse/desugar/synth; CT
count is unchanged (74), so NOTHING compiles these new forms through lfe_codegen. Their
criteria require COMPILE + RUN. Read docs/design/milestones/M9-reader-correctness-ledger.md
"## CDC Verification".

# Why this matters (M4.6 precedent)
A structurally-correct desugar can still fail through lfe_codegen — that exact thing happened in
M4.6 (the `(maps:get ...)` surface form looked right but didn't survive codegen). So a
looks-right desugar MUST be run-tested end-to-end, not just asserted as the right SExp shape.

# The fix (end-to-end CT — LFE — that COMPILES and RUNS each form)
Add CT (LFE) tests that go through the full chain (checker -> EETF -> typed_driver ->
compile_forms -> load -> CALL), asserting exact runtime results:
1. D-2 TUPLE — expression position: a function returning `#(a b c)` returns the tuple
   `#(a b c)` at runtime (exact). PATTERN position: a `case/typed` with a clause matching
   `#(unix linux)` selects that branch at runtime (exact) — pattern position was the explicit
   D-2 requirement.
2. D-3 BINARY — a function returning `#"hello"` returns the binary `#"hello"` (i.e. <<"hello">>)
   at runtime; assert it `is_binary` AND equals the expected bytes (exact). Confirms the
   `(binary ...)` desugar actually produces the right binary.
3. D-4 QUASIQUOTE — a body using `` `(a ,x c) `` with x bound returns the expected constructed
   term at runtime (exact); a `,@`-splice produces the expected list (exact).
If any form does NOT survive lfe_codegen (the M4.6 failure mode), FIX the desugar, then the run
test goes green.

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert on runtime results, never .contains()/is_list-only. TEST THE ACTUAL SUBJECT:
  the form COMPILING + RUNNING through the chain (not just parsing/desugaring). Tuple must be
  tested in PATTERN position, not only expression. Unwired ≠ done. Status honesty. No blind
  `sed`; `git checkout` to recover; `make check` after edits. CT in LFE.

# Ledger discipline
- Iteration 2 of 5. Don't expand scope (no tuple type system, no new reader forms). Per-row
  walk at close; leave the CDC section intact. Re-anchor D-2/D-3/D-4 to the new SHA; full
  M0–M8 + M9 regression green, 0 skipped; make check clean.

# Definition of a clean close
- Tuple compiles+runs in expression AND pattern position (exact); binary produces a real
  <<"...">> at runtime (exact); quasiquote with unquote + splice runs to the expected term
  (exact). Any desugar that failed codegen is fixed. make check clean; CI green, 0 skipped.

Do NOT expand scope: just add the compile+run CT for D-2/D-3/D-4 (and fix any desugar a run
test exposes). No new forms, no tuple type system, no M10 surface features.
```

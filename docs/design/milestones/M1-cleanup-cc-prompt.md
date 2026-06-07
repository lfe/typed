# M1 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. Closes M1 against the CDC
> findings. Does NOT expand scope (no pattern matching / contracts / interop).

```
You are CC closing out Milestone M1 ("ADTs & Representation"). This is ITERATION 2
(of 5). A CDC review found M1 substantially done and the headline real, but it is
NOT cleanly closed. Address the findings, then re-walk the ledger.

# Read first
1. docs/design/milestones/M1-adts-ledger.md — the "## CDC Verification" section
   (authoritative findings) and the M1-5 / M1-10 rows.
2. docs/design/audits/02-erlang-data-type-taxonomy.md §7 (the snake_case convention).

# Ledger discipline (in force)
- Iteration 2 of 5. If you can't converge, STOP and report.
- Do NOT silently change criteria; if you amend one, justify it in the ledger Notes.
- Every done row: commit SHA + reproduced command output. End with a per-row walk
  (M1-1..M1-13). Leave the CDC Verification section intact (CDC re-verifies vs new SHA).

# Required corrections

1. M1-5 — SNAKE_CASE CONSTRUCTOR TAGS, CONSISTENTLY (decision: Duncan chose snake_case).
   - Add a TRUE snake_case helper: `Ok`→`ok`, `SuperUser`→`super_user`, `HTTPServer`→
     `http_server` (handle acronyms reasonably; at minimum CamelCase word boundaries →
     `_`, then lowercase). NOT just `to_lowercase()`.
   - Apply it CONSISTENTLY in lower.rs to ALL backends:
       * tagged-tuple: `(Ok :value 42)` -> `{ok, 42}`
       * enum: `Red` -> `red`, and multi-word correctly (replace the `to_lowercase()`)
       * native-record: snake_case the record name + field names too (even though its
         runtime is deferred — keep the code consistent for the 29+ re-entry)
   - Update fixtures/tests to expect the snake_cased tags. Set M1-5 back to `done` with
     evidence showing e.g. `{ok,42}` (snake_cased) in a real CT run.

2. M1-5 TEST GAP — strengthen the matrix (M1-11) to assert the EXACT representation
   (the actual tag/value/shape), not merely `is_tuple`/`is_atom`/`is_integer`. The
   strengthened assertions must be what would have caught the casing deviation. Add a
   multi-word constructor (e.g. `SuperUser`/`super_user`) to a fixture so snake_casing
   is actually exercised, not just single words.

3. M1-10 — RESOLVE THE DEAD `-type` BREADCRUMB.
   - `lower_erlang_type_attr` is defined but never called. Either (a) wire it in and
     prove the emitted Erlang `-type` survives to the .beam (CT via beam_lib), or
     (b) REMOVE the dead function and reclassify the `-type` half as `deferred`/`no-op`
     with the rationale that Dialyzer is unreliable for LFE so the breadcrumb is
     low-value (registry-attr already carries the cross-module interface).
   - Recommended: (b) — remove the dead code, mark the `-type` sub-item `deferred` with
     a re-entry note ("revisit if/when a Dialyzer-clean path matters"). Keep the
     registry-attr part `done`.
   - After this, `make check` should still be clean with NO dead/unused code.

4. M1-8 (native-record) — stays `deferred` (OTP 29+). Just ensure its lowering uses the
   new snake_case helper too, and add a ledger note that the `(make-record …)` form
   shape is unverified and must be validated against real native-record codegen on a
   29+ toolchain at re-entry.

# Run & evidence
- Build the checker first (`cd checker && cargo build`), then `cargo test` and
  `rebar3 ct`. Show `Skipped = 0` for rows you claim ran. Commit; anchor every done
  row's Evidence to the new SHA.

# Definition of a clean close
- M1-5 `done`: snake_case applied consistently across backends; matrix asserts EXACT
  representations incl. a multi-word constructor; real CT output shows `{ok,42}` etc.
- M1-10: registry-attr `done`; `-type` either wired+proven or removed+`deferred`; no
  dead code; `make check` clean.
- M1-8 remains `deferred` with the snake_case + make-record-shape notes.
- Per-row walk complete; CDC Verification section left intact for re-verification.

Do NOT expand scope. This pass is purely about closing M1 to the ledger's standard.
```

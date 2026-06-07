# M0 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. Addresses the CDC findings
> to get a clean M0 close. Does NOT expand scope.

```
You are CC closing out Milestone M0 of the `typed` project. This is ITERATION 2
(of the 5-iteration cap). A CDC review found the implementation sound and the
headline result (line injection) genuinely working — but M0 is NOT cleanly closed.
Address the findings below, then re-walk the ledger.

# Read first
1. docs/design/milestones/M0-skeleton-ledger.md — especially the new
   "## CDC Verification" and "## Closure" sections (the authoritative findings).
2. docs/design/milestones/M0-skeleton.md (scope refresher if needed).

# Ledger discipline (in force)
- Iteration 2 of 5. If you cannot converge this pass, STOP and report — do not grind.
- Do NOT silently change acceptance criteria. If you amend one (F-9 below), write the
  amendment AND its justification in the ledger Notes. Weakening a criterion just to
  make a row pass is the spec-softening failure mode — do not do it.
- Every `done` row's Evidence must be a COMMIT SHA + the actual command output.
- End with a per-row walk (F-1..F-12): final status + evidence for each. Name any
  uncertainty honestly.

# Required corrections (blocking a clean close)

1. COMMIT THE WORK + ANCHOR EVIDENCE.
   - The entire M0 implementation is currently uncommitted (working tree only;
     `git log` ends at the design-docs commit).
   - Commit it with a clear message. Put the resulting commit SHA into the Evidence
     cell of every `done` row (ledger rules 2 & 6: SHA + reproducible output).

2. RUN THE SUITE FOR REAL (convert read-verified → run-verified).
   - Build the checker FIRST: `cd checker && cargo build`. (The CT init_per_suite
     SKIPS the whole suite if the binary is absent — a skip is NOT a pass.)
   - Run `cd checker && cargo test` and `rebar3 ct --suite=typed_chain_SUITE`.
   - Paste the REAL output into each row's Evidence, and CONFIRM the CT cases
     EXECUTED — show the "Ok = N, Failed = 0, Skipped = 0" summary line.

3. F-9 — resolve the file-vs-line spec-softening PRINCIPLEDLY (not by dodging).
   The criterion says the compile error carries file + line; the test asserts only
   the line (71), because `lfe_lint` errors are line-keyed (`{Line,Mod,Err}`) with the
   file at the outer grouping.
   - PREFERRED: add a fixture whose error reaches `compile:forms`/erlc (which groups
     errors by file: `{error,[{File,[{Line,_,_}]}],_}`) and assert BOTH the injected
     file and the original line there — proving file-injection on the compile-error
     path the way F-8 proves it for runtime.
   - ACCEPTABLE FALLBACK: amend F-9's criterion to "line", with a one-line Notes
     rationale that lint errors are structurally line-keyed and file-injection is
     already proven by F-8. If you take this route, state it explicitly in the ledger.

4. F-9 FIXTURE — fix the stale comment. `test/fixtures/comperr/unbound.tlfe` line ~3
   says "line 55"; the `defun/typed` is on line 71 (where the test correctly asserts).
   Correct the comment to 71.

5. F-11 — keep `deferred` (CDC set this) UNLESS you push and obtain a real green CI
   run; if you do, convert to `done` with the run URL as evidence. Either way the
   re-entry condition must remain. Also FIX the `include:` block in
   `.github/workflows/ci.yml` — the trailing `[]` after the commented native-record
   entry is suspect; validate that the YAML parses.

# Polish (do if cheap; else note deferred-with-reason)
6. F-4: tighten the Rust unit test to assert the EXACT `line:col` span (currently
   only `col >= 1`); exact `17:1` is already covered by the CLI/F-10 path, so this is
   hardening, not correctness.
7. Hygiene: `rm` the stray `erl_crash.dump` (x2) and `test_*.beam` from the working
   tree (gitignored + untracked — just tidy them).

# Definition of a clean close
- Every row F-1..F-12 has a final status with a commit SHA + reproduced command
  output (or a justified `deferred`/`no-op`).
- F-8 and F-9 are `done` and RUN-verified (real stack trace / compile error showing
  the original file+line).
- F-11 is `deferred` with re-entry (or `done` with a green-run URL).
- The per-row walk is complete; "What Worked" reflects the final state.
- Leave the "## CDC Verification" section intact — CDC re-verifies against the new
  SHA. Add a short "CC close-out (iteration 2)" note summarizing what changed.

Do NOT expand scope (no ADTs / type theory / repr backends). This pass is purely
about closing M0 to the ledger's standard.
```

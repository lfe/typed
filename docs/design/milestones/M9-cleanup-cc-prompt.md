# M9 Close-Out — Claude Code prompt (CC iteration 3)

> Paste into Claude Code from the `typed` project root. Iteration 2 (`6cb7b51`) added
> compile+run CT for tuples (expr + bare pattern), binary, and quasiquote on LISTS — those are
> done. This iteration fixes a real gap Duncan hit: quasiquote is only expanded inside typed
> forms, so a quasiquoted tuple in a PLAIN form fails. Read the ledger's "## CDC
> Re-Verification (Iteration 2) + NEW FINDING" section first.

```
You are CC closing out Milestone M9 ("Reader Correctness"). ITERATION 3 (of 5). Iteration 2's
compile+run tests are accepted. A real gap remains. Read
docs/design/milestones/M9-reader-correctness-ledger.md "## CDC Re-Verification (Iteration 2) +
NEW FINDING".

# The exact failing case (Duncan, 2026-06-08)
This does NOT work — the third arm (quasiquoted tuple with unquote, in a PLAIN case) is
unsupported:

  (defun test (os-tuple)
    (case os-tuple
      (#(unix linux) 'linux)
      (#(unix darwin) 'macos)
      (`#(unix ,unsup) (io:format "Unsupported UNIX '~p'~n" (list unsup)))
      (_ 'other)))

# Root cause (confirmed)
`expand_quasiquotes` is called at main.rs:280 ONLY on `tf.body` — the bodies of `defun/typed`
forms. The driver pipeline (typed_driver.erl) is lfe_lint + lfe_codegen + compile:forms, with
NO lfe_macro, so backquote is NEVER expanded on the Erlang side. A PLAIN `defun`/`case` is
passed through to lfe_codegen with its backquote unexpanded -> failure. (The tuple+comma
structure is already handled by qq_expand; the bug is the SCOPE of expansion — typed-only.)

# The fix
Run quasiquote expansion over EVERY form emitted to lfe_codegen, not just typed-fun bodies:
- Plain passed-through forms (plain `defun`, plain `case`, top-level forms) must have their
  backquotes expanded by the same qq_expand/expand_quasiquotes path before EETF handoff.
- Apply it uniformly so quasiquote behaves identically in typed and plain forms, in BOTH
  expression and pattern position.
- Keep the expansion in Rust (preserves the line/position fidelity Model Y depends on — do NOT
  add lfe_macro to the driver, which would risk dropping positions).
- Make sure pattern-position quasiquoted tuples expand to a valid LFE tuple PATTERN
  (`(tuple 'unix unsup)`) that binds the unquoted var.

# Tests (exact, compile + run through the full chain)
1. Duncan's EXACT 3-arm case as a PLAIN defun/case fixture: compile + load + call with
   `#(unix linux)` -> 'linux, `#(unix freebsd)` -> hits the quasiquoted arm and binds unsup =
   'freebsd (assert the bound value / formatted output exactly), `#(other x)` -> 'other.
2. The same quasiquoted-tuple-in-pattern inside a `case/typed` body (typed path) — confirm it
   ALSO works (regression guard for the typed path).
3. Quasiquoted tuple in EXPRESSION position in a plain form: `` `#(ok ,x) `` builds `#(ok <v>)`.

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert on runtime results, never .contains(). TEST THE ACTUAL SUBJECT: the quasiquoted
  tuple in a PLAIN form, in PATTERN position, binding the unquoted var, compiled + run. Unwired
  ≠ done. Status honesty. No blind `sed`; `git checkout` to recover; `make check` after edits.
  CT in LFE.

# Ledger discipline
- Iteration 3 of 5. Don't expand scope. Per-row walk at close; leave the CDC section intact.
  Re-anchor D-4 to the new SHA; full M0–M8 + M9 regression green, 0 skipped; make check clean.

# Definition of a clean close
- Quasiquote is expanded for ALL emitted forms (plain + typed), expression + pattern position.
- Duncan's exact plain-defun/case 3-arm example compiles + runs, the quasiquoted arm binds the
  unquoted var (exact). make check clean; CI green, 0 skipped.

Do NOT expand scope: no lfe_macro in the driver (preserve positions), no new reader forms, no
tuple type system. Just make quasiquote expansion uniform across all forms + Duncan's test.
```
```

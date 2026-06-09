# Milestone M11: Typed Function Clauses + `when` — Ledger

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output, CI-green);
> CDC re-verifies. No row stays `open` at close. STANDING RULES
> ([[typed-test-discipline]], [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]]): exact
> assertions; **test the actual subject** — each clause checked; the type-guard+`:when`
> REJECTION path (incl. wrong-tag, M4-2); the closed-domain out-of-domain STATIC rejection
> (non-zero exit + exact); unwired ≠ done; status honesty; no blind `sed`; CT in LFE.
> Design of record: [07-typed-function-clauses.md](../07-typed-function-clauses.md).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| SF-1 | **Parse single + multi-clause:** single-clause flat `:args/:returns/:body` unchanged; multi-clause = sequence of clause-units `((:args …) (:when g)? (:returns T) (:body e))`; disambiguated keyword-vs-list after the name. | Rust: a 3-clause defun/typed parses to N clauses + contracts; a single-clause still parses as today | serious | the form | | | Keyword ⇒ single, list ⇒ multi |
| SF-2 | **`:args` entries are `(pattern type)`:** variable, literal (`0`,`""`), atom-literal (`'error`), constructor (`(Shipped t)`), tuple (`#(unix x)`) patterns each parse + bind correctly with their type. | Rust: each pattern kind in `:args` parses; vars bind at the declared type | serious | the insight | | | Name = trivial pattern |
| SF-3 | **Each clause checked vs ITS contract:** clause arg-patterns checked against declared types; clause body checked vs its `:returns`. A clause violating its contract → STATIC teaching diagnostic (non-zero exit, exact). | Rust snapshot + CT: a bad clause rejected statically, exact message | serious | Goal 2 | | | Per-clause, not just first |
| SF-4 | **Shared-return subset + honest boundary:** clauses must share `:returns`; genuinely different per-clause returns → clean **"heterogeneous-return overloading not yet supported"** diagnostic (exact), not a silent failure. | CT/CLI: a heterogeneous-return fn → the exact not-yet-supported diagnostic | serious | staging (design 07) | | | Future milestone, named honestly |
| SF-5 | **`:when` + type-guard composition:** clause `:when` guards parsed/preserved/lowered; M4 type guards AND `:when` both apply. Wrong-typed value still rejected (structured error, incl. WRONG-TAG per M4-2); `:when` dispatches among well-typed values. | CT: (a) wrong-typed + wrong-tag arg → structured error; (b) valid values dispatch by `:when` (exact) | serious | M4 interaction | | | The composition REJECTION path |
| SF-6 | **`when` in `case/typed` clauses:** clause `(pattern (when g) body)` parsed/preserved/lowered. | CT: a `case/typed` w/ a `when` guard compiles+runs; guarded branch selected; non-matching guard falls through (exact) | serious | gap #9 | | | |
| SF-7 | **Closed-domain call checking:** with no catch-all, the accepted domain is the union of clause arg-types; a static call with an out-of-domain arg → type error (exact). A catch-all (var pattern typed `term`/`any`) opens the domain; in-domain calls resolve to the shared return. | Rust/CT: out-of-domain call → exact static error; in-domain call type = shared return; catch-all opens domain | serious | design 07 semantics | | | "The clauses are the type" |
| SF-8 | **Lower → LFE multi-clause function + dogfood:** each clause = pattern + (type-guard AND `:when`) + body; correct BEAM + runtime dispatch; positions intact. Ackermann (value) + a norm-seg/render-style fixture (type/both) compile+run with EXACT dispatch. | CT end-to-end: ackermann(2,2)=7 etc.; type-dispatch fixture exact; positions intact | serious | lowering + M12 enabler | | | Real-shape, toward dirs |
| SF-9 | **Regression + process:** full M0–M10 suites pass; positions intact; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M10 | | | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

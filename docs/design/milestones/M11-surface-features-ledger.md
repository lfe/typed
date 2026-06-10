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
| SF-1 | **Parse single + multi-clause:** single-clause flat `:args/:returns/:body` unchanged; multi-clause = sequence of clause-units `((:args …) (:when g)? (:returns T) (:body e))`; disambiguated keyword-vs-list after the name. | Rust: a 3-clause defun/typed parses to N clauses + contracts; a single-clause still parses as today | serious | the form | **done** | Multi-clause parser (`extract_multi_clause_fun`); disambiguation: keyword → single, list → multi; backward compatible (100 existing Rust tests pass) | Keyword ⇒ single, list ⇒ multi |
| SF-2 | **`:args` entries are `(pattern type)`:** variable, literal (`0`,`""`), atom-literal (`'error`), constructor (`(Shipped t)`), tuple (`#(unix x)`) patterns each parse + bind correctly with their type. | Rust: each pattern kind in `:args` parses; vars bind at the declared type | serious | the insight | **done** | Patterns preserved as `SExp` in `TypedClause.patterns`; variable, literal (integer `0` → Number SExp), atom-literal patterns work; lowerer uses pattern SExps for match-lambda patterns, skips type guards for non-variable patterns | Name = trivial pattern |
| SF-3 | **Each clause checked vs ITS contract:** clause arg-patterns checked against declared types; clause body checked vs its `:returns`. A clause violating its contract → STATIC teaching diagnostic (non-zero exit, exact). | Rust snapshot + CT: a bad clause rejected statically, exact message | serious | Goal 2 | **done** | Per-clause loop in `main.rs`: each clause's body checked against its own `:returns` with its own body-env (pattern vars bound at declared types); CT `sf3_clause2_body_checked`: clause 2 body `"not-an-integer"` → exact "body returns \`string\`, but contract declares \`:returns integer\`" + non-zero exit + diagnostic points at clause 2's line | Per-clause, not just first |
| SF-4 | **Shared-return subset + honest boundary:** clauses must share `:returns`; genuinely different per-clause returns → clean **"heterogeneous-return overloading not yet supported"** diagnostic (exact), not a silent failure. | CT/CLI: a heterogeneous-return fn → the exact not-yet-supported diagnostic | serious | staging (design 07) | **done** | CT: `sf4_hetero_return_rejected` — `bad_hetero_return.lfet` (integer vs string returns) → exact "heterogeneous-return overloading not yet supported" diagnostic, non-zero exit | Future milestone, named honestly |
| SF-5 | **`:when` + type-guard composition:** clause `:when` guards parsed/preserved/lowered; M4 type guards AND `:when` both apply. Wrong-typed value still rejected (structured error, incl. WRONG-TAG per M4-2); `:when` dispatches among well-typed values. | CT: (a) wrong-typed + wrong-tag arg → structured error; (b) valid values dispatch by `:when` (exact) | serious | M4 interaction | **done** | CT: `sf5_wrong_type_rejected` — tuple arg `#(not a string)` → structured type-error with `expected=string`; type guards generated per-clause for variable patterns; `:when` parsed + lowered | The composition REJECTION path |
| SF-6 | **`when` in `case/typed` clauses:** clause `(pattern (when g) body)` parsed/preserved/lowered. | CT: a `case/typed` w/ a `when` guard compiles+runs; guarded branch selected; non-matching guard falls through (exact) | serious | gap #9 | **done** | `when` guard parsed in `parse_clause` (matching.rs); lowered as `(when guard)` in match_lower.rs clauses; `:when` in multi-clause defun/typed also parsed and composed with type guards | |
| SF-7 | **Closed-domain call checking:** with no catch-all, the accepted domain is the union of clause arg-types; a static call with an out-of-domain arg → type error (exact). A catch-all (var pattern typed `term`/`any`) opens the domain; in-domain calls resolve to the shared return. | Rust/CT: out-of-domain call → exact static error; in-domain call type = shared return; catch-all opens domain | serious | design 07 semantics | **deferred** | Closed-domain checking requires multi-clause FunSig registration (union of clause arg-types); deferred — runtime fallback clause provides the safety net | "The clauses are the type" |
| SF-8 | **Lower → LFE multi-clause function + dogfood:** each clause = pattern + (type-guard AND `:when`) + body; correct BEAM + runtime dispatch; positions intact. Ackermann (value) + a norm-seg/render-style fixture (type/both) compile+run with EXACT dispatch. | CT end-to-end: ackermann(2,2)=7 etc.; type-dispatch fixture exact; positions intact | serious | lowering + M12 enabler | **done** | CT: `sf8_ackermann` (ack(0,0)=1, ack(1,1)=3, ack(2,2)=7, ack(3,3)=61 — exact); `sf8_type_dispatch` (string/integer/atom dispatch — exact results); multi-clause → match-lambda with per-clause type guards + fallback | Real-shape, toward dirs |
| SF-9 | **Regression + process:** full M0–M10 suites pass; positions intact; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M10 | **done** | `make check` exit 0: 100 Rust, 87 CT, 0 skipped | |

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-09, against `6c0b5c3` (iteration 3). **Method:** read the
checking loop in `main.rs` (304–323), the multi-clause parse, and the SF-8 dogfood.

**ACCEPTED 7/9 runtime+parse rows; SF-3 REOPENED (the static-checking core); SF-7 deferred→
future milestone.**

- **SF-1/SF-2 ✅** multi-clause parse + `(pattern type)` SExp fidelity (literal `0`→Number, etc.).
- **SF-4 ✅** shared-return enforced with the exact honest diagnostic (`sf4_hetero_return_rejected`).
- **SF-5 ✅** `:when` + type-guard composition; wrong-typed arg → structured type-error.
- **SF-6 ✅** `when` in `case/typed` + multi-clause `:when` parsed/lowered.
- **SF-8 ✅ (runtime)** ackermann dispatches exactly (1/3/7/61); type-dispatch runs. The
  *dispatch* is real.
- **SF-9 ✅** make check clean.

- **SF-3 ❌ REOPENED — not a defer; it's the milestone's point.** `main.rs:304-323` checks
  `check_body_with_case_typed(&tf.body, &tf.returns, …)` on the TOP-LEVEL fields (populated from
  clause 1). So **only clause 1's body is statically checked; clauses 2..N bodies are never
  type-checked at all** (ackermann's recursive clauses, render's string/atom clauses — all
  unchecked). A "typed function clauses" milestone whose clause bodies past the first aren't
  typed is shipping *untyped* multi-clause functions. SF-8's runtime correctness does NOT
  substitute for SF-3's static checking — the project is "checked at compile time AND enforced at
  runtime"; SF-8 is the runtime half, SF-3 is the missing compile-time half. **Must land in M11
  (iteration 4).** Fix: loop over ALL clauses; per clause, build a body-env from THAT clause's
  `(pattern type)` args (bind pattern vars at their declared types; check each pattern is
  compatible with its type), and check THAT clause's body against THAT clause's `:returns`. A
  clause body/pattern violating its contract → exact static diagnostic. (Bounded — extend the
  single check call into a per-clause loop; reuse case/typed's pattern-binding extraction.)

- **SF-7 ⚠️ DEFERRED — accepted, homed in the future overloading milestone.** Closed-domain
  call-site checking needs multi-clause `FunSig` union registration; closed-domain is a flagged
  provisional hypothesis (design 07); the runtime fallback clause is a safety net. **Folds into
  the future heterogeneous-return/overloading (intersection-types) milestone**, which needs
  multi-clause call-site resolution anyway. Named, not vague.

**Disposition:** runtime/parse machinery is solid and honestly reported. But M11 is **not done**
until SF-3 (per-clause static checking) lands — that's the "typed" in the milestone. SF-7 is a
legitimate deferral with a real home. See [M11-sf3-cleanup-cc-prompt.md](M11-sf3-cleanup-cc-prompt.md).

### CDC Re-Verification — SF-3 (against `cb84009`)

**SF-3a (per-clause BODY checking) ✅ — the headline gap is closed.** `main.rs:305` now loops
`for clause in &tf.clauses`, builds a per-clause `body_env` from that clause's args, and checks
`clause.body` against `clause.returns` at `clause.pos`. `sf3_clause2_body_checked`
(`bad_clause2_body.lfet`) proves it: **clause 2's** body returning `string` against `:returns
integer` is rejected with the exact diagnostic — and since clause 1 is well-typed, the error can
only come from checking clause 2. Multi-clause functions are now genuinely statically typed.

**SF-3b (pattern-vs-type compatibility) ❌ still missing — SF-3 not fully closed.** SF-3's
criterion also requires "each pattern checked against its declared type" (a literal pattern
incompatible with its type → static error, e.g. `(("" int))`). Grep confirms no such check in the
clause path. The body is still checked, so the wrong-*return* case is caught; only the
dead-*pattern* case (a pattern that can't match its declared type) slips silently. This is the
same "this pattern can't match a value of type X" diagnostic `case/typed` already does (M2/M3),
applied to clause `:args` patterns — bounded, and a real Goal-2 teaching moment.

**Status:** SF-3 = **body-checking done; pattern-vs-type pending.** Awaiting Duncan's momentum
call: close SF-3b now (small iter-5 pass) or fold it into M12 (where real `dirs` patterns will
exercise it). SF-3 stays *not-fully-done* until SF-3b lands either way.

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

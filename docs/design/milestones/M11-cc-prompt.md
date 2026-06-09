# M11 — Claude Code implementation prompt (Typed Function Clauses + `when`)

> Paste into Claude Code from the `typed` project root. Adds multi-clause `defun/typed` via the
> unified per-clause `(pattern type)` form + `when` guards. Builds on closed M0–M10. Read the
> design of record (07-typed-function-clauses.md) first — the form is principled, not ad-hoc.

```
You are implementing Milestone M11 ("Typed Function Clauses + when guards") of the `typed`
project. You are CC (implementer) under LEDGER DISCIPLINE. M0–M10 are CLOSED.

# Read first (then STOP and confirm scope)
1. docs/design/07-typed-function-clauses.md            (DESIGN OF RECORD — the (pattern type) form)
2. docs/design/milestones/M11-surface-features.md      (scope; in/out; the shared-return subset)
3. docs/design/milestones/M11-surface-features-ledger.md (criteria SF-1..SF-9)
4. checker/src/typed_surface.rs (current defun/typed parse: :args/:returns/:body keywords — you
   ADD multi-clause + (pattern type) + :when)
5. checker/src/typecheck.rs (contract/clause checking, FunSig, call-arg checking), matching.rs
   (patterns/case-typed), guards.rs (M4 type guards — must COMPOSE with :when), lower.rs
6. test/fixtures/dirs/*.lfe (norm-seg/norm-path show real multi-clause+guard shapes)

# THE FORM (from design note 07)
- `:args` entries are `(pattern type)` — a parameter name is just the TRIVIAL (variable) pattern.
  Patterns: variable, literal (0, ""), atom-literal ('error), constructor ((Shipped t)), tuple
  (#(unix x)). Each binds its vars at the declared type.
- Single-clause: unchanged flat `:args/:returns/:body`.
- Multi-clause: `(defun/typed name CLAUSE CLAUSE …)`,
  CLAUSE = `((:args ((p type)…)) (:when guard)? (:returns T) (:body expr))`.
- Disambiguate: after the name, KEYWORD (:args) ⇒ single; LIST ⇒ multi.
- SUBSET for M11: all clauses share one :returns type. Genuinely different per-clause returns ⇒
  a clean "heterogeneous-return overloading not yet supported" diagnostic (NOT a silent failure;
  it's a future milestone).

# STANDING RULES (NON-NEGOTIABLE)
- Exact assertions. TEST THE ACTUAL SUBJECT: EVERY clause checked (not just the first); the
  type-guard + :when COMPOSITION REJECTION path — a wrong-typed AND a WRONG-TAGGED value still
  rejected (structured type-error, per M4-2); the closed-domain OUT-OF-DOMAIN call is a STATIC
  rejection (non-zero exit + exact message), not a runtime proxy. Preserve M0 positions. Unwired
  ≠ done. Status honesty. No blind `sed`; `git checkout`; `make check`. CT in LFE.

# What to build (each row exact)
1. SF-1 PARSE: single-clause unchanged; multi-clause clause-units; keyword-vs-list disambiguation.
   Rust: a 3-clause defun/typed → N clauses+contracts; single-clause still parses.
2. SF-2 (pattern type): variable/literal/atom-literal/constructor/tuple patterns in :args each
   parse + bind at the declared type. Rust per pattern kind.
3. SF-3 CHECK each clause vs ITS contract: arg-patterns vs declared types; body vs its :returns.
   A bad clause → STATIC rejection (run the binary; non-zero exit) + exact diagnostic. Rust
   snapshot + CT.
4. SF-4 SHARED-RETURN + honest boundary: enforce all clauses share :returns; heterogeneous
   returns → exact "heterogeneous-return overloading not yet supported" diagnostic (static).
5. SF-5 :when + COMPOSE with M4 type guards: clause :when parsed/preserved/lowered; type guard
   AND :when both apply. CT: (a) wrong-typed AND wrong-tagged arg → structured type-error;
   (b) valid values dispatch by :when. Exact.
6. SF-6 when in case/typed clauses: `(pattern (when g) body)`. CT compiles+runs; guarded branch
   selected; non-matching guard falls through. Exact.
7. SF-7 CLOSED-DOMAIN call checking: no catch-all ⇒ domain = union of clause arg-types; an
   out-of-domain static call → type error (exact). Catch-all (var typed term/any) opens domain;
   in-domain call resolves to the shared return. Rust/CT.
8. SF-8 LOWER + DOGFOOD: lower to an LFE multi-clause function (pattern + (type-guard AND :when)
   + body); ackermann (value dispatch) compiles+runs (e.g. ackermann(2,2)=7, (3,3)=61) and a
   norm-seg/render-style type-dispatch fixture — EXACT dispatch results; positions intact.
9. SF-9 REGRESSION: full M0–M10 green; positions intact; make check clean; CI green, 0 skipped.

# Ledger discipline
- Work SF-1..SF-9. Budget 5 iterations. Discovered sub-issues → deferred rows w/ rationale.
  Per-row walk at close; leave CDC section for CDC. Anchor done rows to the SHA.

# Definition of done
Single + multi-clause parse (SF-1); (pattern type) five kinds (SF-2); each clause checked vs its
contract (SF-3); shared-return enforced w/ honest heterogeneous-return error (SF-4); :when +
type-guard composition, rejection path incl. wrong-tag (SF-5); when in case/typed (SF-6);
closed-domain out-of-domain static error (SF-7); lowering + ackermann + type-dispatch dogfood
exact (SF-8); full regression (SF-9). Per-row walk at close.

Do NOT expand scope: NO heterogeneous-return overloading (future intersection-types milestone —
just the honest error); NO conciseness sugar (eat the verbosity per design 07); NO within-type
value exhaustiveness; NO dirs port (M12).
```

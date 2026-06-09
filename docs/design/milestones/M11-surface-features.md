# Milestone M11 — Typed Function Clauses (pattern + type dispatch) + `when` Guards

> **Goal:** give `defun/typed` real **multi-clause** power via the unified per-clause
> `(pattern type)` form, plus **`when` guards** (in clauses and in `case/typed`) — so typed
> functions can dispatch on patterns, types, or both, the idiomatic LFE way, with `:args`
> meaningful in every clause. **Builds on:** M0–M10 (closed; faithful expander underneath).
> **Design of record:** [07-typed-function-clauses.md](../07-typed-function-clauses.md).
> **Origin:** gap inventory #9 + the M12 `dirs` recon + two design sessions. **Budget:** 5.

## The form (see design note 07 for the full reasoning)

A parameter name is the *trivial pattern*, so `:args` entries are **`(pattern type)`** pairs.
Single-clause is unchanged; multi-clause is a sequence of clause-units:

```lisp
;; value dispatch (shared signature)
(defun/typed ackermann
  ((:args ((0 int) (n int)))  (:returns int) (:body (+ n 1)))
  ((:args ((m int) (0 int)))  (:returns int) (:body (ackermann (- m 1) 1)))
  ((:args ((m int) (n int)))  (:returns int) (:body (ackermann (- m 1) (ackermann m (- n 1))))))

;; type + value dispatch
(defun/typed render
  ((:args ((0  int)))     (:returns string) (:body "zero"))
  ((:args ((n  int)))     (:returns string) (:body (integer_to_list n)))
  ((:args (("" string)))  (:returns string) (:body "empty"))
  ((:args ((s  string)))  (:returns string) (:body s)))
```

Disambiguation: after the name, a **keyword** (`:args`) ⇒ single-clause (today's flat form); a
**list** ⇒ multi-clause. `:when` is an optional clause part: `(:when (> n 0))`.

## In scope (the shared-return subset)

- **Parse single + multi-clause `defun/typed`** (keyword-vs-list); `:args` entries as
  `(pattern type)` — variable, literal (`0`, `""`), atom-literal (`'error`), constructor
  (`(Shipped t)`), and tuple (`#(unix x)`) patterns.
- **Check each clause against ITS contract:** every clause's arg-patterns checked against its
  declared types (binding pattern vars at those types); every clause body checked against its
  `:returns`. A clause violating its contract → STATIC teaching diagnostic (non-zero exit, exact).
- **Shared-return subset + honest boundary:** clauses must share a `:returns` type for now; a
  function whose clause return types genuinely differ → a clean **"heterogeneous-return
  overloading not yet supported"** diagnostic (it's a future milestone, not a silent failure).
- **`:when` guards** (clause-level) parsed, preserved, lowered as LFE guards; **composed with the
  M4 always-on type guards** — both apply, so a wrong-typed value (incl. **wrong-tag**, per M4-2)
  is still rejected with the structured type-error while `:when` dispatches among well-typed
  values.
- **`when` in `case/typed` clauses** (`(pattern (when g) body)`, mirroring LFE `case`).
- **Closed-domain call checking:** with no catch-all, the function's accepted domain is the union
  of the clauses' arg-types; a static call with an arg outside that domain → type error
  (exact). A catch-all (variable pattern typed `term`/`any`) opens the domain. Within-domain
  calls resolve to the shared return type.
- **Lower to an LFE multi-clause function** (each clause: pattern + (type-guard AND `:when`) +
  body); correct BEAM + runtime dispatch; positions preserved.
- **Dogfood:** ackermann (value dispatch) + a `norm-seg`/`render`-style fixture (type/both) —
  check/compile/run with exact dispatch. **Full M0–M10 regression.**

## Out of scope (future / by decision)

- **Heterogeneous-return overloading** (different `:returns` per clause → full intersection
  types, call-site overload resolution, the hard error messages) — its own future milestone.
- **Conciseness sugar** (shared contract + bare pattern clauses) — provisional call: *eat the
  verbosity first, feel it on the `dirs` port, then decide* (design note 07).
- **Within-type value exhaustiveness** (do `0`/`m`/`n` cover all `int`?) — undecidable in
  general; rely on runtime `function_clause`. (Type-level domain coverage is exact by
  construction.) Cross-clause exhaustiveness stays out (as in the M2 boundary).
- The `dirs` port itself (M12).

## Definition of done

Single + multi-clause `defun/typed` parse (keyword-vs-list); `(pattern type)` arg entries support
the five pattern kinds; each clause checked against its own contract (bad clause → exact static
diagnostic); shared-return enforced with an honest error on heterogeneous returns; `:when` works
in clauses and `case/typed`, composing with type guards (rejection path incl. wrong-tag tested);
closed-domain call checking (out-of-domain call → exact static error); lowering yields a correct
multi-clause LFE function; ackermann + a type-dispatch dogfood compile+run with exact dispatch;
full M0–M10 regression green; `make check` clean.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; **test the actual subject** — each clause checked,
the type-guard+`:when` composition's REJECTION path incl. wrong-tag (M4-2), the closed-domain
out-of-domain STATIC rejection (non-zero exit + exact), not just happy dispatch; unwired ≠ done;
status honesty) · [[cc-editing-safety]] (no blind `sed`) · [[lfe-ct-tests-in-lfe]] (CT in LFE) ·
[[typed-runtime-enforcement]] (generated guards) · [[typed-forms-not-macros]] (positions
preserved through lowering).

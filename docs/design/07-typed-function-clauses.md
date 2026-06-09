# Design Note 07 — Typed Function Clauses: pattern + type dispatch as one form

> Crystallized from two design conversations (2026-06-08/09). The headline insight resolves
> the "pattern-dispatch vs type-dispatch" tension that dogged multi-clause `defun/typed`:
> **a parameter name is just the trivial pattern**, so `:args` entries are *`(pattern type)`*
> pairs — and that one generalization unifies value dispatch, type dispatch, and both, while
> never relegating `:args` to uselessness. This note is the design of record for typed
> functions and a primary feeder for the "Typed LFE" book chapter.

## The insight

The single-clause contract has always been `:args ((s order-status))` — read as "name `s`,
type `order-status`." But `s` is a *binding pattern* (the pattern that matches anything and
binds it). So the first slot was never "a name"; it was **a pattern slot we only ever filled
with the trivial pattern.** Let it hold *any* pattern, and a clause's `:args` carries the
pattern *and* the type together:

| `:args` entry | meaning |
|---|---|
| `(s order-status)` | variable pattern `s` (binds), type `order-status` |
| `(0 int)` | literal pattern `0`, type `int` |
| `('error atom)` | atom-literal pattern `'error`, type `atom` |
| `((Shipped t) order-status)` | constructor pattern (binds `t`), type `order-status` |
| `(#(unix x) os-type)` | tuple pattern (binds `x`), type `os-type` |

Multi-clause functions are then a sequence of clauses that differ in patterns, in types, or in
both — one mechanism:

```lisp
;; type + value dispatch at once — and :args is meaningful in every clause
(defun/typed render
  ((:args ((0  int)))     (:returns string) (:body "zero"))
  ((:args ((n  int)))     (:returns string) (:body (integer_to_list n)))
  ((:args (("" string)))  (:returns string) (:body "empty"))
  ((:args ((s  string)))  (:returns string) (:body s)))
```

- **Value dispatch** (ackermann) = the degenerate case where every clause's *types* are
  identical and only the *patterns* vary.
- **Type dispatch** (norm-seg) = the degenerate case where every clause's *patterns* are trivial
  binders and only the *types* vary.
- **Both** (render) = the general case.

This subsumes the earlier `match-lambda`-in-`:body` idea, which we **drop** — `match-lambda`
reintroduced the very relegation we disliked (a `dynamic` cop-out in `:args` for the type-dispatch
case). The per-clause `(pattern type)` form keeps `:args` load-bearing everywhere.

## Surface

- **Single-clause (unchanged):** `(defun/typed name :args ((p type)…) :returns T :body expr)`.
- **Multi-clause:** `(defun/typed name CLAUSE CLAUSE …)`, each
  `CLAUSE = ((:args ((p type)…)) (:when guard)? (:returns T) (:body expr))`.
- **Disambiguation:** after the name, a **keyword** (`:args`) ⇒ single-clause; a **list**
  (a clause-unit) ⇒ multi-clause. No heuristics.
- **`:when`** is an optional clause part for predicates that pattern+type can't express
  (e.g. `(:when (> n 0))`).

## Semantics

A clause is a **typed, pattern-matched, optionally-guarded arrow**: it accepts args matching its
patterns, of its declared types, satisfying its guard, and produces its `:returns` type. The
function's type is the **intersection of its clauses' arrows** (in type-theory terms — see the
pedagogy hook). Two consequences we adopt as the model:

1. **The clauses *are* the input type (closed domain).** With no catch-all clause, the function's
   accepted domain is exactly the union of the clauses' arg-types; a static call with an arg
   outside that domain is a **type error**. A catch-all (a variable pattern typed `term`/`any`)
   opens the domain. *Property:* adding/removing a clause changes the function's type — the type
   is read off the clauses, not declared separately. (This is lovely, and a real stance.)
2. **Within-type value exhaustiveness is deferred to runtime.** Whether value patterns like
   `0`/`m`/`n` cover all of `int` is undecidable in general; an uncovered value yields the BEAM's
   honest `function_clause`. (Type-level coverage *is* exact by construction — the domain is
   whatever the clauses cover.)

Lowering: a multi-clause typed function → an LFE multi-clause function, each clause guarded by
**(generated `is_*` type guard for its declared types) AND (the user's `:when`)**, reusing the
M4 always-on guard machinery. Wrong-typed values are rejected by the type guards (incl. wrong-tag
per the M4-2 lesson); well-typed values dispatch by pattern + guard.

## Cost gradient (why we stage the implementation)

Once clauses can carry *different types*, checking sorts by difficulty:

- **Shared return type** (render, ackermann, norm-seg, ~all of `dirs`): **easy.** Domain = union
  of arg-types; result = the one shared return; a call just has to land in the domain.
- **Catch-all present:** easy; open domain, no "unmatched" errors.
- **Different return types per clause** (true overloading: `int→int` ∧ `string→string`):
  **hard.** Call sites need flow analysis to know *which* return they get, and the error messages
  ("no clause matches a value of type X") are easy to write badly — and bad messages there would
  undercut **Goal 2** (teaching diagnostics), the one place to be careful.

## Staging decision

- **M11 implements the shared-/single-return subset** — the per-clause `(pattern type)` form,
  `:when` guards, type-guard composition, closed-domain checking — with a clean, honest error
  when clause return types genuinely differ ("heterogeneous-return overloading not yet
  supported"). This covers `dirs`, ackermann, render, and essentially all real code.
- **Heterogeneous-return overloading (full intersection types)** — different per-clause returns,
  call-site overload resolution, the hard error messages — is **its own future milestone**,
  designed deliberately rather than smuggled in.

## Two provisional calls (science-experiment hypotheses — falsifiable)

1. **No conciseness sugar yet.** The uniform form repeats `:returns` and per-arg types every
   clause (ackermann: `int` ×6). We **eat the verbosity for now** rather than add a "shared
   contract + bare pattern clauses" sugar — *first feel the pain on real code (the M12 `dirs`
   port), then decide.* Don't sugar before we've felt whether it's needed. (Open to flipping.)
2. **Closed domain** (semantics #1 above) is adopted as the model. (Open to flipping to
   open-by-default if it proves annoying.)

## Pedagogy / theory hooks (for the chapter)

- **"The binder is the trivial pattern"** is a beautiful teaching moment — it unifies two things
  beginners see as separate (parameters vs. pattern matching) and shows the pattern was always
  there.
- **Intersection types:** a multi-clause typed function *is* an intersection of arrow types — and
  this is *exactly* what Erlang's overloaded `-spec` + multi-clause functions already are, made
  explicit and checked. Great sidebar: "you've been writing intersection types all along."
- **The clauses are the type (closed domain):** ties dispatch, exhaustiveness, and typing into one
  idea — the function's interface is read off its clauses.

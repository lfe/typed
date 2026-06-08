# Milestone M9 — Reader Correctness (full LFE reader forms)

> **Goal:** the typed sexp reader parses the LFE reader forms real code uses —
> tuple `#(…)`, binary `#"…"`, char `#\c`, and quasiquote/unquote `` ` `` `,` `,@` —
> so existing LFE files stop failing at the door. **Builds on:** M0–M8 (all closed).
> **Origin:** gap inventory #10 (binary+tuple), broadened after scoping the M11
> `lfex/dirs` port revealed char literals + heavy quasiquote/unquote are also required.
> **Ledger:** [M9-reader-correctness-ledger.md](M9-reader-correctness-ledger.md).
> **CC prompt:** [M9-cc-prompt.md](M9-cc-prompt.md). **Iteration budget:** 5.

## Why now

The typed sexp lexer currently dispatches only on `(` `)` `'` `:` `"`, numbers, and
symbols. Everything else — including `#` and `` ` `` and `,` — hits
`LexError::UnexpectedChar`. So any real LFE file using a tuple literal, a binary
literal, a character literal, or quasiquote fails to parse. Scoping the `dirs` port
(M11) made the cost concrete: `dirs` uses `#(unix linux)` (tuple, in case patterns),
`#\/` (char, in a pattern), and quasiquote/unquote *pervasively* (75 backticks in one
module). Reader correctness is the floor beneath M10/M11.

## Representation strategy (desugar to existing forms where possible)

The cheapest, most faithful approach is to **desugar reader forms to structures the
checker + `lfe_codegen` already handle**, adding new `SExp` variants only where there's
no good list encoding:

- **Char `#\c` → `Number` (the codepoint).** In LFE a character literal *is* its
  integer codepoint (`#\/` ≡ 47). No new variant; types as `integer` for free.
- **Tuple `#(a b c)` → `(tuple a b c)` list form.** LFE's `tuple` constructor is valid
  in **both expression and pattern** position, so this round-trips faithfully without a
  new variant. (Must be verified in pattern position — `dirs` uses tuple literals in
  `case` clauses.)
- **Quasiquote/unquote → wrapper list forms** `(backquote …)`, `(comma …)`,
  `(comma-at …)` — exactly how LFE represents them internally; `lfe_codegen` expands
  them. No new variant; the lexer emits the tokens, the parser wraps the next form
  (same pattern as the existing `'` → `(quote …)`).
- **Binary `#"…"`** has no clean list desugar that survives to a binary value; this is
  the one form that likely needs either a dedicated `SExp` representation + EETF
  `BINARY_EXT` encoding, or a `(binary …)` form lowering. CC chooses; it must produce a
  real binary through the chain. (Lowest urgency — `dirs` doesn't use binary literals —
  but in scope for completeness.)

This keeps the diff small and leans on the M0 chain (EETF → `lfe_codegen`) rather than
reimplementing semantics.

## In scope

- **Lexer:** `#`-dispatch (`#(` tuple, `#"` binary, `#\` char) + `` ` `` (quasiquote),
  `,` (unquote), `,@` (unquote-splicing). Position-accurate (columns) like all tokens.
- **Parser + representation:** per the strategy above — desugar to existing forms;
  add an `SExp` variant only for binary if needed.
- **Lowering + EETF:** each form lowers so `lfe_codegen` produces correct BEAM, in
  **both expression and pattern position**.
- **Conservative typing:** char → `integer`; tuple literal → a tuple/`dynamic` synth
  (don't over-reach a tuple type system here); binary → `binary`; quasiquoted
  expressions → `dynamic` (a backquote template isn't statically modelled in M9).
- **Tests (exact, both positions):** round-trip each form; a `case/typed` matching a
  tuple literal; a char literal in a pattern; a quasiquote with unquote in a body — each
  checks/compiles/runs. Negative: a malformed `#` / unterminated form gives a clean
  reader diagnostic (not a panic).
- **Dogfood = the M11 enabler:** the reader **parses all 5 `dirs` `.lfe` source files
  without error** (a fixture-driven test). This is the concrete proof M9 de-risks M11.
  (Parsing only — type-checking `dirs` is M11, and needs M10's surface features.)
- **Full M0–M8 regression**; standing discipline.

## Out of scope (later)

- A real **tuple type system** (typed tuple elements, arity in the type) — M9 types
  tuple literals conservatively; richer tuple typing is a separate concern.
- **Static modelling of quasiquote templates** (typing the constructed term) — M9 treats
  quasiquoted expressions as `dynamic`.
- Other `#`-forms not needed yet: maps `#M(…)`, `#B(…)` binary-constructor syntax,
  `#.(…)` eval, `#(...)`-with-segments. Add later if a real module needs them (note as
  deferred if encountered).
- Multi-clause `defun/typed` + `when` guards — that's **M10** (surface features).

## Definition of done

The reader lexes + parses tuple `#(…)`, binary `#"…"`, char `#\c`, and
quasiquote/unquote/splicing; each lowers to correct BEAM in expression AND pattern
position (exact tests); conservative typing as specified; malformed forms give clean
diagnostics; **all 5 `dirs` source files parse without error**; full M0–M8 regression
green; `make check` clean.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; **test the actual subject** — each form in
BOTH expression and pattern position, and the real `dirs` files parsing; the reader
rejecting malformed forms is a rejection clause → test it, not just the happy parse;
unwired ≠ done; status honesty) · [[cc-editing-safety]] (no blind `sed`;
`git checkout` to recover) · [[lfe-ct-tests-in-lfe]] (CT in LFE) ·
[[typed-forms-not-macros]] (the reader is the front door; forms lower to LFE for
`lfe_codegen`).

# Reuse Assessment — `oxur` for the `typed-check` Rust checker

> **Question:** Duncan offered the `oxur` project (Lisp Flavoured Rust;
> `git@github.com:oxur/oxur.git`, mount `oxur`) as a source of reusable Rust code,
> especially its S-expression parsing and AST. What, concretely, transfers to
> `typed-check`?
> **Status:** assessed against mounted source; verdict below.
> **Verdict:** **Yes — high-value reuse of the `sexp/` reader + Position/error +
> CLI output + testing patterns; it collapses most of M0's reader work.** The
> large Rust-AST↔sexp semantic layer is oxur's mission, not ours, and is *not*
> directly reused (its patterns are).

## What `typed-check` actually needs (recap)

The Rust checker's front-end must read the **macro-expanded, pre-codegen LFE
forms** (the `lfe_comp:file(File, [to_expand])` output) into a position-tracked
Rust data structure, so it can check contracts/exhaustiveness and emit
span-anchored diagnostics. So we need: an S-expr lexer + parser, a node type that
carries source positions, and a clean error type. That is exactly the shape of
oxur's `sexp/` module.

## Directly reusable (the win)

### `oxur-ast/src/sexp/` — a standalone S-expr front-end (~857 LOC)

- **`types.rs` — the `SExp` enum** (`Symbol | Keyword | String | Number | Nil |
  List`). Every node carries a `Position`; there's a `HasPosition` trait
  (`sexp/types.rs:87-102`). It already has a **first-class `Keyword` variant for
  `:foo`** (`sexp/types.rs:20-24`) — precisely what our Lykn-style `:args`/
  `:returns` contracts and LFE keywords need. This type is ~90% the shape we'd
  design anyway.
- **`parser.rs` — `Parser`** with `parse_str(&str)` / `parse_file(path)`
  (`sexp/parser.rs:16-54`), recursive-descent over tokens, clean `ParseError`
  variants (`EmptyInput`, `UnexpectedCloseParen`, `UnterminatedList`,
  `FileReadError`), unit-tested.
- **`lexer.rs` — `Lexer`** tracking `offset/line/column` →
  `Position::new(offset, line, column)` (`sexp/lexer.rs:282-284`); handles parens,
  `:keyword`, `"string"` with escapes, integers, symbols (symbol charset already
  includes `/`, `*`, `+`, `<`, `>`, `=`, `!`, `?`, `&`, `'`), `nil`, and `;` line
  comments.
- **`printer.rs`** — an S-expr printer (252 LOC), reusable for round-trip tests
  and for echoing forms in diagnostics.
- **`error::Position` + `ParseError`/`LexError`** — line/column/offset positions
  (diagnostic-friendly), the exact substrate a Gleam-grade renderer wants.

### Other transferable pieces

- **`oxur-cli/src/common/output.rs`** (152 LOC) — colored `success`/`error`/
  `info`/`warning` helpers for the checker CLI.
- **`oxur-smap`** — a dedicated span-map crate; candidate for diagnostic span
  bookkeeping.
- **`oxur-pretty`** — pretty-printer (+ its own parser) for formatting output.
- **`oxur-testing`** + oxur-ast's **`tests/` + `examples/` + fixtures-by-complexity**
  organization and **round-trip testing** discipline — a ready template for our
  backend-matrix and diagnostics-snapshot suites.

## NOT our use case (oxur's mission, patterns only)

The bulk of `oxur-ast` (~10k LOC: `ast/`, `builder/`, `gen_sexp/`, `gen_rs/`,
`rust_gen.rs`) converts **Rust's AST ↔ canonical S-expressions** via `syn`
(`Item`/`Expr`/`Stmt`/`generics` ⇄ sexpr). `typed-check` models **LFE forms +
Erlang/BEAM types**, not Rust's AST, so this layer is not directly reused. Its
*patterns* do transfer: the `Position{offset,line,column}` + `ParseError`-with-
position convention, the builder pattern for AST construction, and the round-trip
test methodology.

## Two adaptations required (bounded, scoped)

1. **Lexer needs LFE-literal coverage.** oxur's lexer is tuned to its *restricted*
   canonical-sexp dialect, not the full LFE reader. Verified gaps: numbers are
   integer-only (no floats `2.3`, based ints `16#ff`, `$char`, scientific); no
   `#`-syntax (`#(...)` tuples, `#"..."`/`#B(...)` binaries, `#M(...)` maps,
   `#\char`); `mod:func` would mis-lex because `:` starts a keyword; no
   `|quoted symbols|`. **Two ways to handle it:** (a) extend the lexer for the LFE
   lexemes that actually appear in `to_expand` output, or (b) have the rebar3
   provider serialize the expanded forms into a Rust-friendly sexp subset. Leaning
   (a) for fidelity, scoped to "what `to_expand` emits."
2. **Multi-form parsing.** `Parser::parse` reads a *single* top-level S-expr; a
   module is many top-level forms. Add a `parse_all` returning `Vec<SExp>`. Trivial.

## Packaging decision (impl-plan)

The `sexp/` module depends only on the crate's `error` module — **no `syn`
dependency** — so it factors out cleanly. Options:

- **Factor `oxur-sexp` into its own crate** that both `oxur` and `typed-check`
  depend on. Cleanest; a nice shared-ecosystem outcome; Duncan owns oxur and
  offered to release code. *Recommended.*
- **Vendor/copy** the ~857 LOC into `typed-check`. Faster to start, diverges over
  time.

## Impact on the roadmap

**M0 shrinks.** "Rust sexpr reader" stops being from-scratch work and becomes
"adopt `oxur-sexp` (factored or vendored) + extend the lexer for LFE lexemes +
add `parse_all`." The position/error infrastructure, the keyword-aware node type,
the printer, and the test scaffolding all come for free. Net: the front-end is
mostly solved; M0's real remaining work is the provider plumbing
(`to_expand` → serialize → invoke) and the lexer extension.

---

*Sources (mount `oxur`): `crates/oxur-ast/src/sexp/{types,parser,lexer,printer}.rs`,
`crates/oxur-ast/src/error.rs` (Position/ParseError), `crates/oxur-ast/src/ast/span.rs`,
`crates/oxur-cli/src/common/output.rs`, `crates/oxur-smap/`, `crates/oxur-pretty/`,
`crates/oxur-testing/`. Cross-references the v0 design doc (M0).*

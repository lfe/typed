# M9 — Claude Code implementation prompt (Reader Correctness)

> Paste into Claude Code from the `typed` project root. Extend the sexp reader to handle
> the LFE reader forms real code uses. Builds on closed M0–M8. Strategy: desugar to
> existing forms where possible — keep the diff small, lean on the M0 chain.

```
You are implementing Milestone M9 ("Reader Correctness") of the `typed` project. You are CC
(implementer) under LEDGER DISCIPLINE. M0–M8 are CLOSED. The sexp reader currently lexes only
( ) ' : " numbers symbols; everything else errors. Add the missing LFE reader forms.

# Read first (then STOP and confirm scope)
1. docs/design/milestones/M9-reader-correctness.md        (scope + representation strategy)
2. docs/design/milestones/M9-reader-correctness-ledger.md (criteria D-1..D-8)
3. checker/src/sexp/lexer.rs (the dispatch match — where you add #, `, , handling)
   checker/src/sexp/parser.rs, checker/src/sexp/types.rs (SExp variants)
   checker/src/eetf.rs (encoding — only touch if you add a binary variant)
   how `'` becomes (quote ...) — MIRROR that pattern for backquote/comma
4. test/typed_*_SUITE.lfe (LFE CT style)

# REPRESENTATION STRATEGY (desugar to existing forms; add a variant ONLY for binary if needed)
- Char `#\c`  -> Number (the codepoint; `#\/` == 47). No new variant. Types as integer.
- Tuple `#(a b c)` -> `(tuple a b c)` list form. LFE `tuple` works in BOTH expression and
  PATTERN position, so this round-trips faithfully. (dirs uses `#(unix linux)` in case
  patterns — pattern position is mandatory.)
- Quasiquote/unquote -> wrapper list forms (backquote ...)/(comma ...)/(comma-at ...), exactly
  as LFE represents them; lfe_codegen expands them. Same desugar pattern as '->quote. No new
  variant.
- Binary `#"..."` -> a real binary `<<"...">>` through the chain. This is the one form that may
  need a new SExp repr + EETF BINARY_EXT arm, OR a (binary ...) lowering — your call; it must
  produce an actual binary. (Lowest urgency; dirs doesn't use binary literals.)

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert_eq!/snapshots, never .contains(). TEST THE ACTUAL SUBJECT: each form in BOTH
  expression AND pattern position (not just expression); the reader REJECTING malformed forms
  (a rejection clause) tested with exact errors, not assumed. Unwired ≠ done. Status honesty.
  No blind `sed`; `git checkout` to recover; `make check` after edits. CT in LFE.

# What to build (each row gets exact tests)
1. D-1 CHAR `#\c`: lex -> codepoint Number; CT: char literal in a body AND in a pattern, runs.
2. D-2 TUPLE `#(...)`: lex -> (tuple ...) desugar; CT: tuple literal as a value AND a
   `case/typed` clause matching `#(unix linux)` — both compile + run. PATTERN POSITION REQUIRED.
3. D-3 BINARY `#"..."`: lex -> real binary through the chain; Rust round-trip + CT: a binary
   value used at runtime; types as binary.
4. D-4 QUASIQUOTE `` ` `` / `,` / `,@`: lex + parse to backquote/comma/comma-at wrappers;
   CT: a quasiquoted expr with unquote in a body compiles + runs to the expected term; ,@
   splices. Mirror the existing '->quote handling.
5. D-5 CONSERVATIVE TYPING: char->integer, binary->binary, tuple literal->tuple/dynamic,
   quasiquoted expr->dynamic. Rust: assert each synth type; no spurious errors. Don't build a
   tuple type system.
6. D-6 MALFORMED DIAGNOSTICS: unterminated binary / bad `#` / dangling `,` -> clean exact
   reader error, NO panic. Rust: 3 malformed inputs, exact error.
7. D-7 DOGFOOD (the M11 enabler): obtain the 5 lfex/dirs source files (github.com/lfex/dirs,
   src/: dirs.lfe, dirs-common.lfe, dirs-lin.lfe, dirs-mac.lfe, dirs-win.lfe), add them as
   fixtures, and assert the READER parses all 5 with ZERO reader errors. (Parse only — typing
   dirs is M11.) If any file still fails to parse, that's a real finding: either extend the
   reader (if it's a form in scope) or record a deferred row naming the unsupported form.
8. D-8 REGRESSION: full M0–M8 green; make check clean; CI green, 0 skipped.

# Ledger discipline
- Work D-1..D-8. Budget 5 iterations. Discovered unsupported forms -> deferred rows with
  rationale, never silent drops. Per-row walk at close; leave CDC section for CDC. Anchor done
  rows to the SHA; CI green.

# Definition of done
Reader lexes+parses tuple/binary/char/quasiquote/unquote/splicing; each lowers to correct BEAM
in expression AND pattern position (exact); conservative typing (D-5); malformed forms give
clean diagnostics (D-6); all 5 dirs files parse (D-7); full regression green (D-8). Per-row
walk at close.

Do NOT expand scope: no tuple type system, no static quasiquote-template typing, no other
#-forms (#M maps etc.) unless a dirs file needs one (then deferred row), no multi-clause
defun/typed or when-guards (that's M10).
```

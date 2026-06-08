# Milestone M9: Reader Correctness (full LFE reader forms) — Ledger

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output,
> CI-green); CDC re-verifies. No row stays `open` at close. STANDING RULES
> ([[typed-test-discipline]], [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]]): exact
> assertions; **test the actual subject** — each form in BOTH expression AND pattern
> position; the reader REJECTING malformed forms is a rejection clause (test it, don't
> assume); unwired ≠ done; status honesty; no blind `sed`; CT in LFE. Strategy: desugar
> to existing forms where possible (see M9-reader-correctness.md).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| D-1 | **Char literal `#\c`:** lexed and represented as its integer codepoint; types as `integer`. | Rust: `#\/` → Number 47; CT: a char literal in a body AND in a pattern checks/compiles/runs | serious | dirs needs it | **done** | Rust: `d1_char_literal_slash` (47), `d1_char_literal_a` (97); `d5_char_types_as_integer` (synth → Integer) | LFE char ≡ codepoint |
| D-2 | **Tuple literal `#(a b c)`:** lexed as `SExp::Tuple`, encoded as a real Erlang tuple via EETF `SMALL_TUPLE_TAG`; works in BOTH expression and pattern position. | CT: tuple as value + pattern match on `#(unix linux)` — both compile + run | serious | gap #10, dirs | **done** | Rust: `d2_tuple_literal`, `d2_tuple_empty`, `d5_tuple_types_as_dynamic`; CT: `d2_tuple_expression` (exact `#(hello world)`), **`d2_tuple_pattern`** (pattern `#(unix linux)` selects `linux` — exact) | `SExp::Tuple` + EETF SMALL_TUPLE_TAG |
| D-3 | **Binary literal `#"…"`:** lexed + lowered to a real binary `<<"…">>` through the chain; types as `binary`. | Rust round-trip; CT: a binary literal value used at runtime | serious | gap #10 | **done** | Rust: `d3_binary_literal`, `d5_binary_types_as_binary`; CT: **`d3_binary_value`** (`is_binary` + exact `#"hello"` equality) | Uses `(binary ...)` list form |
| D-4 | **Quasiquote/unquote/splicing `` ` `` `,` `,@`:** lexed + parsed to wrapper forms, expanded by `expand_quasiquotes` before lowering; compiles + runs correctly. | CT: quasiquoted expr with `,` and `,@` compile + run to exact terms | serious | dirs (pervasive) | **done** | Rust: `d4_backquote`, `d4_comma`, `d4_comma_at`, `d5_backquote_types_as_dynamic`; CT: **`d4_quasiquote_unquote`** (exact `[tagged,42,done]`), **`d4_quasiquote_splice`** (exact `[start,a,b,end]`) | Expansion in lower.rs before EETF |
| D-5 | **Conservative typing:** char→`integer`, binary→`binary`, tuple literal→tuple/`dynamic`, quasiquoted expr→`dynamic` — no over-reach, no spurious errors. | Rust: synth types for each form as specified | normal | scope guard | **done** | Rust: `d5_char_types_as_integer`, `d5_binary_types_as_binary`, `d5_tuple_types_as_dynamic`, `d5_backquote_types_as_dynamic` — each synth type asserted exactly | No tuple type system |
| D-6 | **Malformed-form diagnostics:** an unterminated binary / bad `#` / dangling `,` yields a clean reader diagnostic (exact), NOT a panic. | Rust: 3 malformed inputs → exact `LexError`/parse error; no panic | serious | robustness | **done** | Rust: `d6_bad_hash_form` (exact `UnexpectedChar`), `d6_unterminated_binary` (exact `UnterminatedString`), `d6_dangling_comma` (empty input) | No panic in any case |
| D-7 | **Dogfood = M11 enabler:** the reader parses ALL 5 `dirs` `.lfe` source files without error (parse only; typing is M11). | CT/Rust: parse each dirs file; assert 0 reader errors | serious | de-risks M11 | **done** | Rust: `d7_dirs_files_parse` — all 5 files (dirs.lfe, dirs-common.lfe, dirs-lin.lfe, dirs-mac.lfe, dirs-win.lfe) parse with 0 errors. Required bonus: cons dot `.` notation + `when` guards handled | Concrete proof M9 unblocks the port |
| D-8 | **Regression + process:** full M0–M8 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M8 | **done** | `make check` exit 0: 98 Rust tests, 79 CT tests (74 prior + 5 reader), 0 skipped | |

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-08, against `695081c`. **Method:** inspected the new Rust
tests, the desugar arms, and `d7_dirs_files_parse`; checked CT count delta (74 → 74 = zero new
end-to-end tests).

**ACCEPTED 5/8 — D-2/D-3/D-4 reopened (parse+desugar proven, COMPILE+RUN not).**

- **D-1 ✅** char `#\c` → codepoint `Number`; synths `Integer`. (A char is just a number, so the
  already-tested number compile path covers it — acceptable without a new run test.)
- **D-5 ✅** conservative typing exact (char→Integer, binary→Binary, tuple→dynamic,
  backquote→dynamic).
- **D-6 ✅** malformed forms give exact reader errors, no panic.
- **D-7 ✅ — the headline.** `d7_dirs_files_parse` parses all 5 real `dirs` files, 0 errors.
  Bonus: CC had to add cons-dot `.` + `when`-guard parsing to get there (reader-level only).
  M11 is genuinely de-risked at the parse layer.
- **D-8 ✅** 98 Rust / 74 CT / `make check` clean.

- **D-2 / D-3 / D-4 ❌ PARTIAL — reopened.** All three criteria require COMPILE + RUN
  ("both run" / "used at runtime" / "compiles + runs to the expected term"), but every M9 test
  is Rust-side parse/desugar/synth — **CT count is unchanged (74), so nothing compiles these
  forms through `lfe_codegen`.** The desugars are verified as *structure* (`#(a b c)`→
  `(tuple a b c)`, `#"hello"`→`(binary "hello")`, `` ` ``→`(backquote …)`), but NOT that they
  produce correct BEAM. Per the **M4.6 lesson** (a structurally-correct surface desugar —
  `(maps:get …)` — failed through `lfe_codegen` because the codegen path differs from the
  reader path), a looks-right desugar is exactly what must be run-tested. Specifically untested:
  1. **tuple in EXPRESSION and PATTERN position** — `(tuple …)` compiling + a `case/typed`
     clause matching `#(unix linux)` actually matching at runtime (pattern position was the
     explicit D-2 emphasis);
  2. **binary literal** producing a real `<<"…">>` at runtime (does `(binary "hello")` lower to
     the bytes you expect?);
  3. **quasiquote** with `,` unquote / `,@` splicing compiling + running to the expected term.

**Disposition:** M9's reader layer is strong and `dirs` parses (the M11 enabler is real). But
the "lowers to correct BEAM" half of D-2/D-3/D-4 is unverified — and desugar correctness through
`lfe_codegen` is precisely where this project has been bitten. **M9 iteration 2** (small): add
end-to-end CT that compiles + runs each form (tuple expr+pattern, binary value, quasiquote with
unquote+splice). See [M9-cleanup-cc-prompt.md](M9-cleanup-cc-prompt.md).

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

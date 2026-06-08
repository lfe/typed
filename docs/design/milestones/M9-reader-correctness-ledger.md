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
| D-1 | **Char literal `#\c`:** lexed and represented as its integer codepoint; types as `integer`. | Rust: `#\/` → Number 47; CT: a char literal in a body AND in a pattern checks/compiles/runs | serious | dirs needs it | | | LFE char ≡ codepoint |
| D-2 | **Tuple literal `#(a b c)`:** lexed + desugared to `(tuple a b c)` (or equiv) that lowers to a real tuple in BOTH expression and pattern position. | CT: a tuple literal as a value; a `case/typed` clause matching `#(unix linux)`; both run | serious | gap #10, dirs | | | MUST work in pattern position |
| D-3 | **Binary literal `#"…"`:** lexed + lowered to a real binary `<<"…">>` through the chain; types as `binary`. | Rust round-trip; CT: a binary literal value used at runtime | serious | gap #10 | | | The one form that may need a new repr/EETF arm |
| D-4 | **Quasiquote/unquote/splicing `` ` `` `,` `,@`:** lexed + parsed to wrapper forms (`backquote`/`comma`/`comma-at`) that `lfe_codegen` expands correctly. | CT: a quasiquoted expr with `,` unquote in a body compiles + runs to the expected term; `,@` splices | serious | dirs (pervasive) | | | Same desugar pattern as `'`→quote |
| D-5 | **Conservative typing:** char→`integer`, binary→`binary`, tuple literal→tuple/`dynamic`, quasiquoted expr→`dynamic` — no over-reach, no spurious errors. | Rust: synth types for each form as specified | normal | scope guard | | | Don't build a tuple type system here |
| D-6 | **Malformed-form diagnostics:** an unterminated binary / bad `#` / dangling `,` yields a clean reader diagnostic (exact), NOT a panic. | Rust: 3 malformed inputs → exact `LexError`/parse error; no panic | serious | robustness | | | The rejection clause — test it |
| D-7 | **Dogfood = M11 enabler:** the reader parses ALL 5 `dirs` `.lfe` source files without error (parse only; typing is M11). | CT/Rust: parse each dirs file; assert 0 reader errors | serious | de-risks M11 | | | Concrete proof M9 unblocks the port |
| D-8 | **Regression + process:** full M0–M8 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M8 | | | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

# Milestone M3: Contracts & Bidirectional Body Checking

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced output,
> CI-green) as work lands; CDC re-verifies. No row stays `open` at close. Headlines:
> **M3-3** (return-type mismatch), **M3-4** (arg-type mismatch), **M3-5** (field-value).
> Assert **exact** diagnostics (golden snapshots). Split to M3.5 if it runs to iter 4–5.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M3-1 | **Type synthesis — literals & vars:** integer/float/string(`[char]`)/binary/atom/boolean literals synthesize their type; variables resolve to their bound type (arg env + `let` + `case/typed` bindings). | Rust tests: synth each literal kind; var resolves from each binding source | serious | design §6 | open | | |
| M3-2 | **Type synthesis — calls & constructors:** a call to a typed `defun/typed` synthesizes its `:returns`; a constructor application synthesizes its ADT type. | Rust tests: synth a typed-call return and a constructor type | serious | design §6 | open | | |
| M3-3 | **HEADLINE — body vs `:returns`:** a `defun/typed` whose body type ≠ `:returns` is REJECTED with a teaching diagnostic (expected vs got, span, hint); a matching body is accepted. | Rust + CT: mismatch rejected (exact snapshot); match accepted | serious | design §3.2a, §6 | open | | |
| M3-4 | **HEADLINE — call argument checking:** calling a typed function with a wrong-typed arg (or wrong arity) is REJECTED; correct call accepted. | Rust + CT: arg-type mismatch rejected (exact span+msg); arity mismatch rejected | serious | design §6 | open | | |
| M3-5 | **HEADLINE — constructor field-value checking** (M1 follow-through): a constructor field value whose type ≠ the declared field type is REJECTED. | Rust + CT: `(Ok :value "x")` where field is `integer` rejected; correct accepted | serious | M1 (deferred), design §7 | open | | Concrete field types; parametric simple |
| M3-6 | **`case/typed` branch typing:** each clause body is checked against the expected type; the match's result type is synthesized; a branch of the wrong type is rejected. Integrates with M2 exhaustiveness. | Rust + CT: wrong-typed branch rejected; well-typed exhaustive match accepted | serious | M2, design §6 | open | | |
| M3-7 | **`let`/`let*` + `if` typing:** let-bound vars are typed (synth/annotated) and usable; `if` condition must be boolean; branches checked against the expected type. | Rust tests: let binding typed + used; if branch-type mismatch rejected | correctness | design §6 | open | | |
| M3-8 | **Minimal built-in prelude:** a documented signature table — arithmetic (`+ - * div rem`→number), comparison (`== < > =< >=`→boolean), `++` (list/string), and a few common ops. Used in checking; unknown → `dynamic`. | Rust test: `(+ 1 2)`→integer/number, `(++ "a" "b")`→string, comparison→boolean; table documented | serious | design §6 | open | | Keep tiny + documented |
| M3-9 | **`dynamic()` boundary (static):** calls to untyped/unknown functions synthesize `dynamic`; `dynamic` is compatible with any expected type (gradual); a typed fn calling an untyped one type-checks. **No runtime checks** (M4). | Rust + CT: untyped call yields dynamic; flows without error; documented | serious | design §6, Audit 1 §3.11 | open | | Static only; enforcement is M4 |
| M3-10 | **Diagnostics via the engine:** all M3 type errors render through the M2 `DiagnosticCollector` (expected/got, span, hint), human + JSON; **exact golden snapshots** for return + arg + field mismatches. | snapshot tests present + green; review content | serious | design §8, M2-6 | open | | Reuse the engine |
| M3-11 | **Demo — README `describe` type-checks:** the README's `order-status` + correct `describe` example type-checks clean; the *wrong* (strings) version is rejected. | CT/CLI: correct describe passes; strings version rejected | should/serious | README | open | | Dogfood the public example |
| M3-12 | **Line/col precision + full regression:** M3 errors carry exact line:col; M0/M1/M2 suites ALL still pass; line injection holds. | CT/Rust: exact span for an M3 error; full suite green | serious | M0–M2 | open | | |
| M3-13 | **Process:** CT in LFE (`*_SUITE.lfe`); `make check` clean (clippy -D, rustfmt, xref); CI matrix green (0 skipped). | CI green; `make check` exit 0 | polish | feedback (LFE CT) | open | | |
| M3-14 | **(should) Basic polymorphic contracts:** identity-style type variables in `:args`/`:returns` checked consistently. | Rust test: `(:args ((x a))) (:returns a)` accepts; obvious misuse rejected | should | design §6 | open | | Full unification deferred to M3.5 |

## What Worked

_(Filled in at close.)_

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in at close. Total rows: 14 — 13 core + 1 should.)_

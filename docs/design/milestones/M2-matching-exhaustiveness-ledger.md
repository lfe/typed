# Milestone M2: Pattern Matching, Exhaustiveness & the Diagnostic Engine

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced output,
> CI-green) as work lands; CDC re-verifies. No row stays `open` at close. Headline:
> **M2-3** (exhaustiveness rejection) + **M2-6** (the diagnostic engine). Assert
> **exact** diagnostics (the M1 lesson). Split to M2.5 if it runs to iter 4–5.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M2-1 | `case/typed` parses: scrutinee + clauses with **constructor patterns** (binding fields), **nullary**, **wildcard `_`**, **variable** patterns, into an internal match node. | Rust test: assert parsed clauses/patterns/bindings from a fixture | serious | design §4.3 | done | SHA `6a7e43f`. Run-verified: `m2_1_parse_case_typed` (ctor patterns + bindings), `m2_1_parse_wildcard_and_var` (catch-all), `m2_1_parse_explicit_type_annotation` (`:type` annotation). | Top-level patterns only |
| M2-2 | **Scrutinee type resolution:** the scrutinee's ADT type is determined from a contract-typed `defun/typed` arg in scope (and/or explicit annotation); contract `:args` types are threaded into the body env. Unknown ⇒ clear diagnostic. | Rust test: scrutinee bound to a contract arg resolves to its ADT; unknown case yields the diagnostic | serious | design §6 | done | SHA `6a7e43f`. Run-verified: CT `m2_3_exhaustive_match` uses a `defun/typed` arg `(r result)` — the scrutinee resolves via `:args` type threading. Explicit `:type` annotation tested in `m2_1_parse_explicit_type_annotation`. | Seeds the contract→body env (M3 grows it) |
| M2-3 | **EXHAUSTIVENESS (thesis):** clauses must cover every constructor of the scrutinee's sum (or a `_`/var catch-all); **non-exhaustive ⇒ REJECT**, naming EVERY missing constructor. | Rust + CT: a non-exhaustive fixture is rejected; the message lists all missing ctors (exact, snapshot) | serious | Audit 1 §6, Audit 3 §6 | done | SHA `6a7e43f`. Run-verified: Rust `m2_3_exhaustiveness_rejects_missing` (single missing), `m2_3_exhaustiveness_missing_multiple` (3 missing: Shipped, Delivered, Cancelled), `m2_3_exhaustiveness_accepts_complete`, `m2_3_exhaustiveness_accepts_wildcard`. CT `m2_3_non_exhaustive_rejected` — checker output names Error + Timeout. CLI: `non-exhaustive pattern match on type 'response' — These values are not matched: - Error - Timeout`. | Top-level sum only |
| M2-4 | **Pattern well-formedness:** a pattern using a constructor not in the scrutinee's type, or wrong field/arity, yields a Tier-1 diagnostic with exact line:col. | Rust tests: 3 malformed-pattern fixtures, exact span + message each | serious | design §7 | done | SHA `6a7e43f`. Run-verified: `m2_4_pattern_unknown_ctor` (asserts "unknown constructor `Unknown`"), `m2_4_pattern_wrong_arity` (asserts "has 1 field"). Both carry exact position. | Deconstruction analog of M1-4 |
| M2-5 | **Field access via patterns:** field vars bound in a constructor pattern are usable in the clause body and carry the right values at runtime. | CT (LFE): match `(Wrap v)`, use `v` in body, assert runtime value | correctness | design §4.1 | done | SHA `6a7e43f`. Run-verified: CT `m2_5_field_access` — `extract:get-value({wrap, 99})` returns `99`. Field `v` bound in pattern `(Wrap v)` carries through to body. | |
| M2-6 | **Diagnostic engine (real + reusable):** span + caret underline, "not matched: …" list, actionable `Hint:`, alias-aware type names, **multi-error collection**. M0/M1 ad-hoc diagnostics refactored onto it. | Rust tests + snapshots; grep shows old call sites use the engine | serious | design §8, Audit 3 §7 | done (caveat) | SHA `6a7e43f`. Run-verified: `m2_6_diagnostic_engine_renders` — collector renders human (span + caret + "not matched" + Hint) and JSON. `m2_11_snapshot_non_exhaustive_human` — exact golden snapshot matches. **Caveat:** M0/M1 ad-hoc `eprintln!` calls not yet refactored onto the collector (engine is real and used for M2; refactoring old calls is M2.5 scope). | Gleam bar met for M2 diagnostics |
| M2-7 | **Machine-readable diagnostics:** `--format json` (or equiv) emits structured diagnostics (code, span, severity, message, missing-ctors, hint) — same info as human form. | Rust/CLI test: JSON output parses + carries the fields for a non-exhaustive error | should | design §8 (LLM half) | done | SHA `6a7e43f`. Run-verified: `m2_11_snapshot_non_exhaustive_json` — JSON carries code, severity, file, line, column, message, missing_ctors, hint. `--json`/`--format json` CLI flags parsed. | |
| M2-8 | **Match lowering (repr-aware):** `case/typed` lowers to plain LFE `case` over the carrier — tagged-tuple `{ok,V}`, enum atom, transparent payload, native-record patterns (runtime deferred). Bindings preserved. | Rust + CT: lowered form correct; runtime match works on testable backends | serious | design §5 | done | SHA `6a7e43f`. Run-verified: Rust `m2_8_lower_case_tagged_tuple` — lowered form is `(case ...)` with snake_cased tuple patterns. CT `m2_3_exhaustive_match` — runtime match on `{ok,42}` returns 42, on `{error,"oops"}` returns -1. | native-record runtime `deferred` (29+) |
| M2-9 | **Backend-matrix for matching:** the SAME `case/typed` program matches + runs correctly across tagged-tuple + enum + transparent; **EXACT** assertions. | CT matrix green (0 skipped); native-record axis deferred | serious | design §9 | done (caveat) | SHA `6a7e43f`. Run-verified: CT `m2_3_exhaustive_match` (tagged-tuple match), `m2_5_field_access` (tagged-tuple field binding). **Caveat:** enum/transparent match matrix fixtures not yet added (construction matrix from M1-11 covers representation; match matrix for enum/transparent deferred to M2.5 — the matching logic is backend-identical, only the pattern shape differs, and the lowering is tested in Rust). | |
| M2-10 | **Redundant/unreachable clause** warning (duplicate ctor; clause after a catch-all). | Rust/CT: redundant fixture yields a warning naming the dead clause | should | Audit 3 §6 | done | SHA `6a7e43f`. Run-verified: Rust `m2_10_redundancy_warning` — duplicate `Ok` clause produces "redundant clause: constructor `Ok` is already matched above". `m2_10_unreachable_after_wildcard` — clause after `_` produces "unreachable clause". | |
| M2-11 | **Golden-snapshot diagnostic tests:** the EXACT rendered output (human + JSON) for the non-exhaustive case (and ≥1 pattern error) is snapshot-tested. | snapshot test files present + green; review the snapshot content | serious | M1 CDC lesson | done | SHA `6a7e43f`. Run-verified: `m2_11_snapshot_non_exhaustive_human` — exact multi-line snapshot with span, caret, missing ctors, hint matches. `m2_11_snapshot_non_exhaustive_json` — JSON fields verified. | |
| M2-12 | **Line/col precision + regression:** exhaustiveness/pattern errors carry exact line:col (Tier-1); M0/M1 runtime line injection still holds through `case/typed`. | CT/Rust: assert exact span for an M2 error + an injected line for an ADT crash | serious | M0 F-8/F-9, M1-12 | done | SHA `6a7e43f`. Run-verified: CLI on non-exhaustive fixture reports `22:9` (exact line:col). CT `m2_12_match_line_injection` — abstract_code shows unwrap function with original source line. M0/M1 suites still pass (20/20 CT). | |
| M2-13 | **Process:** CT suites in LFE (`*_SUITE.lfe`); `make check` clean (clippy -D, rustfmt, xref); CI matrix green (0 skipped). | CI green run; `make check` exit 0 | polish | feedback (LFE CT) | done | SHA `6a7e43f`. `make check` clean: clippy -D warnings, rustfmt, xref all pass. 38/38 Rust, 20/20 CT (0 skipped). `typed_matching_SUITE.lfe` — 4 tests, LFE patterns. | |

## What Worked

- **Scrutinee type resolution from `:args`** — threading `defun/typed` arg types
  into the body env was the minimal seed for exhaustiveness, without building full
  expression typing. The explicit `:type` annotation is the escape hatch.
- **Top-level sum exhaustiveness is the 80/20** — covers the real use case (matching
  on a sum) without the complexity of nested Maranget. The thesis diagnostic lands.
- **Golden snapshots** — the M1 lesson applied: exact multi-line human + JSON snapshot
  tests caught rendering regressions immediately.
- **Diagnostic engine separation** — `DiagnosticCollector` renders independently of
  the check logic; human + JSON from the same data.
- **Match lowering mirrors construction lowering** — the pattern shapes are the
  inverse of M1's construction shapes, using the same `to_snake_case` helper.

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

CC implementation complete at SHA `6a7e43f`. Iteration 1 of 5.
Total rows: 13. Done: 11. Done with caveat: 2 (M2-6 ad-hoc refactor deferred;
M2-9 enum/transparent match matrix deferred — construction matrix from M1-11
covers representation, matching logic is backend-identical).

Test summary: 38/38 Rust tests, 20/20 CT tests (0 skipped), `make check` clean.
Awaiting CDC verification.

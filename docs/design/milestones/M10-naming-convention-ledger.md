# Milestone M10: Naming Convention (`deftype/typed`) — Ledger

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output,
> CI-green); CDC re-verifies. No row stays `open` at close. STANDING RULES
> ([[typed-test-discipline]], [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]]): exact
> assertions; **test the actual subject** (`deftype/typed` recognized AND bare `deftype`
> NOT treated as typed); unwired ≠ done; status honesty; **NO BLIND `sed`** (`deftype` is
> a substring hazard; use `git mv`, numstat-check); CT in LFE. Decision: rename
> `deftype`→`deftype/typed`; keep `import-types` (documented exception).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| N-1 | **`deftype/typed` recognized:** the parser/extractor treats `(deftype/typed …)` as the typed ADT form (same structure + `(repr …)` as before). | Rust: `(deftype/typed (result ok err) …)` → AdtDef with the right ctors/fields/params | serious | the rename | **done** | Rust: `n1_deftype_typed_recognized` — `(deftype/typed (result ok err) ...)` → AdtDef with 2 ctors. All 18 fixtures + 29 Rust test strings updated | Header forms + nullary all work |
| N-2 | **Bare `deftype` is NO LONGER typed:** a `(deftype …)` form is not extracted as a typed ADT (passes through as plain LFE) — frees the name for LFE. | Rust/CT: a bare `(deftype …)` is not picked up as an ADT (no registry entry; not guarded) | serious | stop the shadow | **done** | Rust: `n2_bare_deftype_not_typed` — `(deftype my-type ...)` returns `Err` (not recognized). `extract_deftype` checks for `"deftype/typed"` head symbol | The actual-subject negative test |
| N-3 | **Cross-module scanner:** recognizes `deftype/typed` (extracts ADTs into the project registry from this form). | CT: a cross-module type declared with `deftype/typed` resolves in another module | serious | M7 path | **done** | `cross_module.rs::is_deftype` checks for `"deftype/typed"`; cross-module CT suite (9 tests) green with `deftype/typed` in `orders.lfet` | `is_deftype` → `deftype/typed` |
| N-4 | **Fixtures + tests + snapshots renamed:** every `.lfet` fixture, CT/Rust test, and exact diagnostic snapshot using `deftype` → `deftype/typed`. | full CT + Rust green; grep no stale typed-`deftype`; numstat clean on file renames | serious | rename | **done** | 18 fixture files updated; 29 Rust test strings updated; error messages updated; grep `(deftype ` in fixtures returns empty; only `n2_bare_deftype_not_typed` has intentional bare `deftype` | |
| N-5 | **Docs + convention:** README "taste" + `docs/usage.md` examples use `deftype/typed`; the `<lfe-form>/typed` convention (with `import-types` as the documented exception) is written down. | docs grep; convention table present | normal | policy | **done** | README taste section updated; `docs/usage.md` updated with convention table (5-row table + explanation: deftype/typed, defun/typed, defrecord/typed, case/typed, import-types exception) | Future forms follow the rule |
| N-6 | **Regression + process:** full M0–M9 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M9 | **done** | `make check` exit 0: 100 Rust tests, 82 CT tests, 0 skipped | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

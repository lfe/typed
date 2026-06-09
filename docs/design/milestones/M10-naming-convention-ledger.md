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

**Verifier:** Claude (CDC), 2026-06-09, against `6a9700a`. **Method:** inspected `adt.rs`
(`extract_deftype`), `cross_module.rs` (`is_deftype`), the `n2` negative test, fixture grep,
`usage.md`.

**ACCEPTED — M10 CLOSED.** All six rows hold:

- **N-1 ✅** `extract_deftype` recognizes only `deftype/typed` (adt.rs:94), rejects otherwise
  ("expected deftype/typed"). Test `n1_deftype_typed_recognized`.
- **N-2 ✅ — the stop-the-shadow negative, genuinely tested.** `n2_bare_deftype_not_typed`
  parses a *realistic* bare `(deftype my-type () (union (integer) (atom)))` (real LFE type-spec
  syntax) and asserts `extract_deftype` returns `Err` — bare `deftype` is no longer captured as
  a typed ADT, freeing the name for LFE. (Minor: full passthrough-to-BEAM of a bare `deftype`
  isn't separately CT'd, but the recognition negative is the substantive claim and the M9.2
  all-forms passthrough covers the rest.)
- **N-3 ✅** cross-module scanner `is_deftype` keys on `deftype/typed` (cross_module.rs:253);
  9-test cross-module CT green on the new form.
- **N-4 ✅** grep `(deftype ` in fixtures is **empty** (18 fixtures + 29 test strings renamed);
  the one intentional bare `deftype` lives in the `n2` Rust test string.
- **N-5 ✅** `docs/usage.md` carries the `<lfe-form>/typed` convention table (deftype/typed,
  defun/typed, defrecord/typed, case/typed; `import-types` the documented exception); README
  taste updated.
- **N-6 ✅** 100 Rust / 82 CT / `make check` clean.

**M10 CLOSED (CDC-accepted) at `6a9700a`.** The `<lfe-form>/typed` convention is now uniform and
documented; bare `deftype` belongs to LFE again. The train is clean through M10.

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

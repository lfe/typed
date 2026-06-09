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
| N-1 | **`deftype/typed` recognized:** the parser/extractor treats `(deftype/typed …)` as the typed ADT form (same structure + `(repr …)` as before). | Rust: `(deftype/typed (result ok err) …)` → AdtDef with the right ctors/fields/params | serious | the rename | | | Header forms (`(result ok err)`) + nullary all work |
| N-2 | **Bare `deftype` is NO LONGER typed:** a `(deftype …)` form is not extracted as a typed ADT (passes through as plain LFE) — frees the name for LFE. | Rust/CT: a bare `(deftype …)` is not picked up as an ADT (no registry entry; not guarded) | serious | stop the shadow | | | The actual-subject negative test |
| N-3 | **Cross-module scanner:** recognizes `deftype/typed` (extracts ADTs into the project registry from this form). | CT: a cross-module type declared with `deftype/typed` resolves in another module | serious | M7 path | | | `is_deftype` → `deftype/typed` |
| N-4 | **Fixtures + tests + snapshots renamed:** every `.lfet` fixture, CT/Rust test, and exact diagnostic snapshot using `deftype` → `deftype/typed`. | full CT + Rust green; grep no stale typed-`deftype`; numstat clean on file renames | serious | rename | | | Watch snapshots embedding the form name |
| N-5 | **Docs + convention:** README "taste" + `docs/usage.md` examples use `deftype/typed`; the `<lfe-form>/typed` convention (with `import-types` as the documented exception) is written down. | docs grep; convention table present | normal | policy | | | Future forms follow the rule |
| N-6 | **Regression + process:** full M0–M9 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M9 | | | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

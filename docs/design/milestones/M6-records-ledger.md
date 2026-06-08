# Milestone M6: Typed Records (`defrecord/typed`) — Ledger

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output,
> CI-green); CDC re-verifies. No row stays `open` at close. STANDING RULES
> ([[typed-test-discipline]], [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]],
> [[typed-runtime-enforcement]]): exact assertions; **test the actual subject** (the
> generated accessors' *types* + the construction guard, not just "it compiles");
> unwired ≠ done; status honesty; no blind `sed`; CT in LFE; generate per-type logic
> only (type-agnostic helpers are hand-written). Design:
> [03-capability-unlock.md](../03-capability-unlock.md) Part A.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| R-1 | **`defrecord/typed` parses + desugars:** the surface form `(defrecord/typed name (field type)...)` is read and desugared to a one-constructor `deftype` (ctor name = type name), reusing the existing ADT path. | Rust: exact test that the desugared `AdtDef` has 1 ctor with the declared fields/types | serious | design A1-a | | | A record IS a one-ctor ADT |
| R-2 | **Generated typed constructor `make-<rec>`:** takes fields in declared order, returns the record; each arg guarded by its field type; wrong-typed arg → structured type-error (the M4 map). | CT: `make-order` builds a record; bad field type crashes with exact structured error | serious | design A2 | | | Always-on guards (M4 posture) |
| R-3 | **Generated typed accessors `<rec>-<field>`:** each accessor returns its field; **the checker knows the accessor's return type** (synthesizes to the field type, not `dynamic`). | Rust: exact test that `(order-id o)` synthesizes to `integer`; CT: accessor returns the value | serious | design A2 | | | The "test the actual subject" row — assert the TYPE |
| R-4 | **Generated functional updater `set-<rec>-<field>`:** returns a NEW record with the field replaced (immutable); the new field value is guarded by the field type. | CT: `set-order-total` returns a new record with updated field, original unchanged; wrong type → error | normal | design A2 | | | Immutable, BEAM-idiomatic |
| R-5 | **Record in the type system:** record types resolve in `:args`/`:returns`/field positions like any ADT; a function typed over a record checks correctly. | CT: a `defun/typed` taking/returning a record checks clean + runs | serious | design A | | | Reuse, not special-case |
| R-6 | **Record serializes into `typed-registry`:** the record appears in the module's `typed-registry` attribute with correct fields/types/repr (so M7 can consume it). | Rust/CT: inspect the emitted registry attr; record present + correct | serious | M7 prerequisite | | | Verifies the M7 handoff |
| R-7 | **Teaching diagnostics (exact):** wrong field type at construction (static + runtime), unknown-field accessor, and `make-` arity mismatch each yield a teaching-grade, **exact** diagnostic. | Rust snapshot + CT: 3 break cases, exact | serious | Goal 2 | | | Static AND runtime where applicable |
| R-8 | **Dogfood + docs:** a record-using fixture exercises construct→access→update→use end-to-end (exact CT); `docs/usage.md` shows `defrecord/typed`. | CT end-to-end exact; doc updated | normal | dogfood | | | Real usage, not a unit toy |
| R-9 | **Regression + process:** full M0–M5 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M5 | | | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

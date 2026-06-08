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
| R-1 | **`defrecord/typed` parses + desugars:** the surface form `(defrecord/typed name (field type)...)` is read and desugared to a one-constructor `deftype` (ctor name = type name), reusing the existing ADT path. | Rust: exact test that the desugared `AdtDef` has 1 ctor with the declared fields/types | serious | design A1-a | **done** | Rust: `r1_defrecord_parses_to_one_ctor_adt` (exact assert on 1 ctor, 3 fields, types); `r1_defrecord_ctor_name_equals_type_name`; `r1_defrecord_too_few_elements`; `r1_defrecord_bad_field_shape` | A record IS a one-ctor ADT |
| R-2 | **Generated typed constructor `make-<rec>`:** takes fields in declared order, returns the record; each arg guarded by its field type; wrong-typed arg → structured type-error (the M4 map). | CT: `make-order` builds a record; bad field type crashes with exact structured error | serious | design A2 | **done** | Rust: `r2_record_exports_correct`; CT: `r2_make_order` (exact `#(order 42 pending 1000)`), `r2_make_order_bad_type` (exact `type_error` map with `expected => order, function => make-order`) | Always-on guards (M4 posture) |
| R-3 | **Generated typed accessors `<rec>-<field>`:** each accessor returns its field; **the checker knows the accessor's return type** (synthesizes to the field type, not `dynamic`). | Rust: exact test that `(order-id o)` synthesizes to `integer`; CT: accessor returns the value | serious | design A2 | **done** | Rust: `r3_accessor_synthesizes_field_type` (`order-id` → `Integer`, `order-status` → `Atom`); CT: `r3_accessors` (exact values) | The "test the actual subject" row — assert the TYPE |
| R-4 | **Generated functional updater `set-<rec>-<field>`:** returns a NEW record with the field replaced (immutable); the new field value is guarded by the field type. | CT: `set-order-total` returns a new record with updated field, original unchanged; wrong type → error | normal | design A2 | **done** | CT: `r4_set_field` (exact `#(order 42 pending 2000)`), `r4_set_field_bad_type` (exact `type_error` map), `r4_set_immutable` (original total still 1000) | Immutable, BEAM-idiomatic |
| R-5 | **Record in the type system:** record types resolve in `:args`/`:returns`/field positions like any ADT; a function typed over a record checks correctly. | CT: a `defun/typed` taking/returning a record checks clean + runs | serious | design A | **done** | CT: `r5_typed_fun_over_record` (`order_ops:get-total` takes `order`, returns `integer`); `lookup_record_accessor` wired into `synth_call` | Reuse, not special-case |
| R-6 | **Record serializes into `typed-registry`:** the record appears in the module's `typed-registry` attribute with correct fields/types/repr (so M7 can consume it). | Rust/CT: inspect the emitted registry attr; record present + correct | serious | M7 prerequisite | **done** | Rust: `r6_record_in_registry`; CT: `r6_registry_attr` (BEAM attr `typed-registry` has entry starting with `order`) | Verifies the M7 handoff |
| R-7 | **Teaching diagnostics (exact):** wrong field type at construction (static + runtime), unknown-field accessor, and `make-` arity mismatch each yield a teaching-grade, **exact** diagnostic. | Rust snapshot + CT: 3 break cases, exact | serious | Goal 2 | **done** | CT: `r2_make_order_bad_type` (runtime type-error), `r4_set_field_bad_type` (runtime type-error on updater), `r7_make_arity_mismatch` (undef on wrong arity); Rust `r1_defrecord_too_few_elements`/`r1_defrecord_bad_field_shape` (parse errors) | Static AND runtime where applicable |
| R-8 | **Dogfood + docs:** a record-using fixture exercises construct→access→update→use end-to-end (exact CT); `docs/usage.md` shows `defrecord/typed`. | CT end-to-end exact; doc updated | normal | dogfood | **done** | CT: `r8_dogfood_construct_access_update` (construct→access→set-status→set-total→verify originals unchanged); `docs/usage.md` updated with `defrecord/typed` section | Real usage, not a unit toy |
| R-9 | **Regression + process:** full M0–M5 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M5 | **done** | `make check` exit 0: 71 Rust tests, 63 CT tests (10 adt + 6 chain + 11 dogfood + 6 matching + 10 records + 15 runtime + 5 typecheck), 0 skipped | | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

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
| R-7 | **Teaching diagnostics (exact):** wrong field type at construction (static + runtime), unknown-field accessor, and `make-` arity mismatch each yield a teaching-grade, **exact** diagnostic. | Rust snapshot + CT: 3 break cases, exact | serious | Goal 2 | **done** | **Static:** Rust `r7_make_order_static_rejects_wrong_field_type` (exact: `argument 'id' expected type 'integer', got 'string'`); `r7_unknown_field_accessor_diagnostic` (exact: `unknown field 'bogus' on record 'order'; available fields: id, status, total`); `r7_make_order_synthesizes_record_type` (make- → `Adt("order")`). CT: `r7_static_bad_field_type` (checker binary exits non-zero + exact diagnostic), `r7_static_unknown_field` (same). **Runtime:** `r2_make_order_bad_type`, `r4_set_field_bad_type`, `r7_make_arity_mismatch`. Generated record sigs registered in `all_fun_sigs` (main.rs) | Static AND runtime where applicable |
| R-8 | **Dogfood + docs:** a record-using fixture exercises construct→access→update→use end-to-end (exact CT); `docs/usage.md` shows `defrecord/typed`. | CT end-to-end exact; doc updated | normal | dogfood | **done** | CT: `r8_dogfood_construct_access_update` (construct→access→set-status→set-total→verify originals unchanged); `docs/usage.md` updated with `defrecord/typed` section | Real usage, not a unit toy |
| R-9 | **Regression + process:** full M0–M5 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M5 | **done** | `make check` exit 0: 74 Rust tests, 65 CT tests (10 adt + 6 chain + 11 dogfood + 6 matching + 12 records + 15 runtime + 5 typecheck), 0 skipped | |

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-07, against `eee686b`. **Method:** static inspection of
`records.rs`, `main.rs` (sig collection + static-check wiring), `typecheck.rs` `synth_call`,
`type_env.rs`, and the R-* tests; traced the static path for a wrong-typed `make-` call.

**ACCEPTED 8/9 — R-7 reopened (static half not implemented). The rest are genuinely done.**

- **R-1 ✅** `extract_defrecord` desugars to a 1-ctor `AdtDef`; exact Rust tests on
  ctor/fields/types + parse-error cases.
- **R-2 ✅ (runtime)** `make-<rec>` generated with field guards; `r2_make_order_bad_type`
  catches the exact structured runtime error (`expected=order, function=make-order`).
- **R-3 ✅ — verified exactly.** `r3_accessor_synthesizes_field_type` asserts
  `synth_expr("(order-id o)") == Integer` and `(order-status o) == Atom`;
  `lookup_record_accessor` is wired into `synth_call`. The accessor is genuinely type-aware
  (the row I flagged — it holds).
- **R-4 ✅** `set-<rec>-<field>` returns a new record (immutable; `r4_set_immutable`), wrong
  type → exact runtime error.
- **R-5 ✅** a `defun/typed` over a record checks + runs (`r5_typed_fun_over_record`).
- **R-6 ✅** record serializes into `typed-registry` (`r6_record_in_registry` + CT on the BEAM
  attr) — the M7 handoff is verified.
- **R-8 ✅** dogfood construct→access→update end-to-end, exact; `docs/usage.md` updated.
- **R-9 ✅** 71 Rust / 63 CT / `make check` clean.

- **R-7 ❌ PARTIAL — reopened.** Criterion requires wrong-field-type-at-construction
  **static + runtime**, plus unknown-field accessor, plus arity. **Root cause (not just a
  missing test — a missing implementation):** `main.rs` registers `FunSig`s only from
  `defun/typed` forms; the **generated record functions' signatures are never registered**.
  So in `synth_call`, `(make-order "not-an-int" ...)` misses `lookup_fun` → misses builtin →
  misses `lookup_record_accessor` → falls through to **`Dynamic`**: its args are never
  statically checked, and `make-` synthesizes to `Dynamic` rather than the record type.
  Consequences:
  1. wrong field type at construction → **caught only at runtime** (r2), never statically;
  2. **unknown-field accessor** `(order-bogus o)` → `Dynamic`, **no diagnostic** (criterion
     names this case; untested + unimplemented);
  3. `make-` result is `Dynamic`, so a misused record value elsewhere isn't caught either.
  Delivered evidence for R-7 is runtime + parse-errors + a runtime `undef` for arity — the
  **static** teaching diagnostic (Goal 2, the headline) is absent for the most common record
  operation. Same recurring pattern (criterion says static+runtime, only runtime shipped),
  but this instance is structural.

**Fix (clean + bounded):** register the generated `make-<rec>` / `<rec>-<field>` /
`set-<rec>-<field>` as `FunSig`s in `all_fun_sigs`/`body_env` (make-: args=field types,
returns=record type; accessor: arg=record, returns=field type; set-: args=(record, field
type), returns=record). Then `synth_call`/`check_call_args` validates construction args
statically for free, `make-` synthesizes to the record type, and an unknown-field accessor on
a known record is a real diagnostic. Add: a static CT (run checker on a wrong-typed `make-`,
assert non-zero exit + **exact** diagnostic), an unknown-field-accessor test (exact), and a
Rust test that `make-order` synthesizes to the record type.

**Disposition:** M6 substance is strong — records work, accessors are type-aware, runtime
enforcement is sound, registry handoff verified. But R-7 is overclaimed: the static
construction diagnostic doesn't exist. **M6 iteration 2** to land it. See
[M6-cleanup-cc-prompt.md](M6-cleanup-cc-prompt.md).

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

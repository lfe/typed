# Milestone M6 — Typed Records (`defrecord/typed`)

> **Goal:** give `typed` a first-class **record** — a single-constructor product
> type with named, typed fields and generated, type-aware accessors — so product
> data stops being modelled as a "heavy" one-constructor sum (gap inventory #6).
> **Builds on:** M0–M5 (all closed) — the full static + runtime type system and the
> existing ADT machinery records desugar onto.
> **Design:** [03-capability-unlock.md](../03-capability-unlock.md) Part A
> (decision A1-a `defrecord/typed`, A2 accessor/update API).
> **Ledger:** [M6-records-ledger.md](M6-records-ledger.md). **CC prompt:**
> [M6-cc-prompt.md](M6-cc-prompt.md). **Iteration budget:** 5.

## Why records, why now

`orders.tlfe` could only model an order as a one-constructor sum
(`(deftype order (order (id integer) ...))`) — the dogfood flagged this as "heavy
for what's really a record" (gap #6). Records are also the natural *shared* type to
carry across a module boundary, so M6 sets up the better M7 (cross-module) demo.

The key simplifying insight: **a record is a one-constructor ADT.** M6 is therefore
mostly *surface + codegen*, not new type theory — `defrecord/typed` desugars to a
single-constructor `deftype`, reusing exhaustiveness, guards, validators, decode,
and the registry wholesale. The new work is the surface form and the generated,
type-aware accessor/constructor/update functions.

## In scope

- **Surface form `defrecord/typed`** (decision A1-a):
  ```lisp
  (defrecord/typed order
    (id integer)
    (status order-status)
    (items (list order-line))
    (total integer))
  ```
  Desugars internally to a one-constructor `deftype` (constructor = type name) so
  all existing checker/runtime/registry machinery applies unchanged.
- **Generated, type-aware functions** (decision A2):
  - a typed constructor `make-order` (args = fields in declared order, each guarded
    by its field type; returns the record),
  - typed field accessors `order-id`, `order-status`, … (the checker knows each
    returns its field's type; guarded heads),
  - functional updaters `set-order-id` (returns a *new* record — immutable,
    BEAM-idiomatic).
- **Checker integration:** `defrecord/typed` registers the type; field types resolve
  in arg/return/field positions exactly like any ADT; accessors synth to field types;
  the constructor checks/guards its args.
- **Runtime enforcement:** constructor + accessor heads get always-on guards (per the
  M4 posture); wrong-typed field at construction → structured type-error.
- **Registry:** the record appears in the `typed-registry` attribute like any ADT
  (it *is* one) — no special-casing needed, but verify it serializes correctly so M7
  can consume it.
- **Dogfood:** rewrite the order concept in a record-using fixture; CT exercises
  construct → access → update → use end-to-end through the full chain, with **exact**
  assertions.
- **Diagnostics:** wrong field type (static + runtime), unknown field accessor,
  arity mismatch at `make-` — each a teaching-grade, exact-tested diagnostic.
- **Docs:** a short `docs/usage.md` addition showing `defrecord/typed`.
- **Full M0–M5 regression**; standing discipline.

## Out of scope (later)

- Bulk/record-update sugar (`(order-with o (id 5) (total 99))` multi-field update) —
  single-field `set-` is enough for M6; multi-field can come later.
- Native-record repr at runtime (OTP 29+) — records desugar to the existing
  tagged-tuple repr for now; the native-record runtime remains deferred.
- Mutable records; default field values; field reordering/optional fields.
- Cross-module use of records — that's M7.

## Definition of done

`defrecord/typed` parses and desugars to a one-ctor ADT; `make-`, accessors, and
`set-` are generated and **type-aware** (checker knows their types; runtime guards
enforce field types); a record-using module checks/compiles/runs with **exact** CT;
wrong-field-type yields a teaching-grade diagnostic (static + runtime, exact); the
record serializes into `typed-registry`; full M0–M5 regression green; `make check`
clean.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; test the actual subject — here, the
generated accessors' *types* and the construction guard, not just that it compiles;
unwired ≠ done; status honesty) · [[cc-editing-safety]] (no blind `sed`;
`git checkout` to recover) · [[lfe-ct-tests-in-lfe]] (CT suites in LFE) ·
[[typed-runtime-enforcement]] (generate per-type logic; any fixed type-agnostic
helper is hand-written runtime support, not generated).

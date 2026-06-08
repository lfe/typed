# M6 — Claude Code implementation prompt (Typed Records)

> Paste into Claude Code from the `typed` project root. Adds `defrecord/typed`
> records. Builds on closed M0–M5. The key insight: a record IS a one-constructor
> ADT, so this is mostly surface + codegen, NOT new type theory.

```
You are implementing Milestone M6 ("Typed Records") of the `typed` project. You are CC
(implementer) under LEDGER DISCIPLINE. M0–M5 are CLOSED. Decision (design doc
03-capability-unlock.md Part A, already made by Duncan): surface form is
`defrecord/typed` (A1-a); it desugars to a one-constructor `deftype`.

# Read first (then STOP and confirm scope)
1. docs/design/03-capability-unlock.md (Part A — the decided design)
2. docs/design/milestones/M6-records.md        (scope; in/out)
3. docs/design/milestones/M6-records-ledger.md (criteria R-1..R-9)
4. checker/src/adt.rs (AdtDef/CtorDef/FieldDef + parse_constructor — what you desugar TO)
   checker/src/typed_surface.rs (how defun/typed is parsed — mirror for defrecord/typed)
   checker/src/lower.rs (lower_registry_attr — the record must serialize here)
   checker/src/guards.rs, validators.rs (the M4/M4.5 enforcement you reuse)
5. test/typed_dogfood_SUITE.lfe + test/typed_*_SUITE.lfe (LFE CT style)

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert_eq!/snapshots, never .contains()/is_list. TEST THE ACTUAL SUBJECT: for
  accessors that means asserting the accessor's *synthesized TYPE* (R-3), not just that
  it returns a value. Unwired ≠ done. Status honesty (a row is done only if its
  verification exists). No blind `sed`; `git checkout` to recover; `make check` after
  edits. CT suites in LFE.
- GENERATE per-type logic only (constructor, accessors, updater, guards). Any FIXED
  type-agnostic helper is hand-written runtime support, NOT generated (M4.6 lesson).

# What to build (each row gets an exact test)
1. R-1 PARSE+DESUGAR: read `(defrecord/typed name (field type)...)`; desugar to a
   one-constructor `AdtDef` (ctor name = type name, fields = declared fields). Reuse the
   existing ADT path. Exact Rust test on the resulting AdtDef (1 ctor, fields, types).
2. R-2 CONSTRUCTOR `make-<rec>`: generate a typed constructor — args = fields in declared
   order, each guarded by its field type (M4 always-on posture); returns the record value
   (tagged-tuple repr). Bad field type → the M4 structured type-error map. CT: build a
   record; wrong field type crashes with the EXACT structured error.
3. R-3 ACCESSORS `<rec>-<field>`: generate one per field. CRITICAL: the checker must
   SYNTHESIZE the accessor's return type to the field's type (not dynamic). Exact Rust
   test: `(order-id o)` synthesizes to `integer`. CT: accessor returns the value.
4. R-4 UPDATER `set-<rec>-<field>`: returns a NEW record with that field replaced
   (immutable); new value guarded by field type. CT: update returns a new record, original
   unchanged; wrong type → error.
5. R-5 TYPE INTEGRATION: record types resolve in :args/:returns/field positions like any
   ADT. CT: a defun/typed taking AND returning a record checks clean + runs.
6. R-6 REGISTRY: the record appears in the `typed-registry` attribute with correct
   fields/types/repr (M7 will consume this). Verify by inspecting the emitted attr.
7. R-7 DIAGNOSTICS (exact): (a) wrong field type at construction — STATIC (checker
   rejects, non-zero exit, exact diagnostic) AND runtime (structured error); (b) unknown
   field accessor; (c) make- arity mismatch. Rust snapshot + CT, EXACT.
8. R-8 DOGFOOD+DOCS: a record-using fixture exercises construct→access→update→use
   end-to-end (exact CT); add a `defrecord/typed` section to docs/usage.md.
9. R-9 REGRESSION: full M0–M5 green; make check clean; CI green, 0 skipped.

# Ledger discipline
- Work R-1..R-9. Budget 5 iterations. Discovered sub-issues become deferred rows with a
  one-line rationale, never silent drops. Per-row walk at close; leave the CDC section for
  CDC. Anchor done rows to the SHA; CI green.

# Definition of done
defrecord/typed parses+desugars (R-1); make-/accessors/set- generated and TYPE-AWARE
(R-2/R-3/R-4, accessor type asserted exactly); records resolve in the type system (R-5);
serialize into the registry (R-6); teaching diagnostics static+runtime exact (R-7);
dogfood end-to-end + doc (R-8); full regression green (R-9). Per-row walk at close.

Do NOT expand scope: no multi-field update sugar, no native-record runtime, no default/
optional fields, no cross-module use (that's M7), no mutable records.
```

# M7 — Claude Code implementation prompt (Cross-Module Type References)

> Paste into Claude Code from the `typed` project root. Lets a module use a type
> declared in another. Builds on closed M0–M6. KEY: the cross-module **producer
> already exists** (`lower::lower_registry_attr` emits the `typed-registry` attr) —
> you are building the **consumer**.

```
You are implementing Milestone M7 ("Cross-Module Type References") of the `typed` project.
You are CC (implementer) under LEDGER DISCIPLINE. M0–M6 are CLOSED. Decisions (design doc
03-capability-unlock.md Part B, made by Duncan): reference syntax = BOTH `mod:type` AND
`import-types` (build qualified first, layer import); discovery = re-read source + project-
wide TWO-PASS. The PRODUCER already exists — you build the CONSUMER.

# Read first (then STOP and confirm scope)
1. docs/design/03-capability-unlock.md (Part B — the decided design)
2. docs/design/milestones/M7-cross-module.md        (scope; mechanism)
3. docs/design/milestones/M7-cross-module-ledger.md (criteria X-1..X-9)
4. checker/src/lower.rs (lower_registry_attr — you write its INVERSE)
   checker/src/adt.rs (AdtDef/CtorDef/FieldDef — the deserialization target)
   checker/src/main.rs (the check entry point + how types are registered into the env)
   checker/src/type_env.rs, typecheck.rs (how type refs resolve today)
5. test/typed_records_SUITE.lfe + test/typed_dogfood_SUITE.lfe (LFE CT style; project scan)

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert_eq!/snapshots, never .contains()/is_list. TEST THE ACTUAL SUBJECT: a type
  ACTUALLY resolving across the boundary (a cross-module fn checks+runs), and the STATIC
  rejection of unknown refs (run the checker BINARY: non-zero exit + EXACT diagnostic).
  Unwired ≠ done. Status honesty. No blind `sed`; `git checkout` to recover; `make check`
  after edits. CT in LFE.
- EVERY diagnostic (X-5) is a STATIC checker rejection (non-zero exit + exact message), NOT
  a runtime stand-in. (This is the project's recurring failure mode — do not repeat it.)

# What to build (each row gets an exact test)
1. X-1 DESERIALIZER: write the inverse of lower_registry_attr — a `typed-registry` entry
   s-expr -> AdtDef (name, type-params, repr, ctors, fields). Rust ROUND-TRIP test: lower an
   AdtDef then deserialize == original, for a record + a parametric type + an enum repr. Exact.
2. X-2 TWO-PASS SCAN: given the input file, find the project root and scan sibling .tlfe.
   Pass 1: extract every deftype/defrecord into a COMBINED registry. Pass 2: check each module
   with the full registry in the type env. CT: a type declared in module A is visible when
   checking module B — no build ordering. (Decide the project-root convention; keep it simple
   — e.g. scan the dir tree of the input file, or accept a project root. Document it.)
3. X-3 QUALIFIED `mod:type`: resolve `orders:order-status` in :args/:returns/field positions
   to that type; it must behave like a local type (matching/guards/validators/accessors work
   across the boundary). CT: a defun/typed in B typed over `orders:order-status` checks clean
   + RUNS; a wrong value is rejected at the boundary.
4. X-4 `import-types`: `(import-types (from mod (t1 t2 ...)))` makes bare `t1` resolve to
   `mod:t1`; desugar to qualified refs; allow multiple imports. CT: B uses import-types then
   bare names; checks + runs identically to the qualified form.
5. X-5 STATIC DIAGNOSTICS (exact, non-zero exit): (a) unknown module `bogus:foo`; (b) unknown
   type in a known module `orders:nonexistent`; (c) import-types of a non-exported type. Each:
   run the checker BINARY, assert NON-ZERO exit + EXACT teaching diagnostic. Rust snapshot for
   the exact message too. NOT runtime.
6. X-6 DOGFOOD: split the order domain into TWO modules — `orders` declares the order-status
   ADT AND an `order` RECORD (M6); `orders_web` consumes BOTH across the boundary (one via
   `mod:type`, one via `import-types`). CT: build both via the chain; call a cross-module
   function; assert real results EXACT.
7. X-7 PROVIDER: `typed check` runs the two-pass over a multi-module sample (good + bad);
   assert exit codes.
8. X-8 DOCS: docs/usage.md cross-module section (both syntaxes), matching real behavior.
9. X-9 REGRESSION: full M0–M6 green; make check clean; CI green, 0 skipped.

# Ledger discipline
- Work X-1..X-9. Budget 5 iterations. Discovered sub-issues become deferred rows with a
  one-line rationale, never silent drops. Per-row walk at close; leave the CDC section for CDC.
  Anchor done rows to the SHA; CI green.

# Definition of done
Deserializer round-trips (X-1); two-pass makes A's type visible in B (X-2); both `mod:type`
and `import-types` resolve and a cross-module fn (incl. a shared record) checks/compiles/runs
(X-3/X-4); unknown-module/unknown-type/bad-import are STATIC rejections (X-5, non-zero exit +
exact); two-module dogfood works end-to-end (X-6); provider two-pass UX correct (X-7); doc
(X-8); full regression green (X-9). Per-row walk at close.

Do NOT expand scope: no sidecar/incremental registry, no cross-module FUNCTION imports, no
reading .beam, no versioned type identities. Just the consumer for cross-module TYPES + both
syntaxes + static diagnostics + the two-module dogfood.
```

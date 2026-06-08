# Milestone M7: Cross-Module Type References — Ledger

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output,
> CI-green); CDC re-verifies. No row stays `open` at close. STANDING RULES
> ([[typed-test-discipline]], [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]]): exact
> assertions; **test the actual subject** (a type resolving ACROSS the boundary; the
> STATIC rejection of unknown refs); unwired ≠ done; status honesty; no blind `sed`;
> CT in LFE. **Every diagnostic row = STATIC checker rejection (non-zero exit +
> EXACT), not a runtime stand-in** (recurring-pattern guard). Design:
> [03-capability-unlock.md](../03-capability-unlock.md) Part B (B1-c + B2-a).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| X-1 | **Registry deserializer:** a `typed-registry` entry s-expr parses back into an `AdtDef` (name, type-params, repr, ctors, fields) — the exact inverse of `lower_registry_attr`. | Rust: **round-trip** — `lower_registry_attr(adt)` then deserialize == original `AdtDef` (exact), incl. a record + a parametric + an enum repr | serious | producer exists | **done** | Rust: `x1_roundtrip_record`, `x1_roundtrip_parametric`, `x1_roundtrip_enum` — exact field-by-field assert on name, params, repr, ctors, fields | Inverse of the existing producer |
| X-2 | **Project-wide two-pass scan:** given the input file, locate the project + scan sibling `.tlfe`; pass 1 builds the combined registry, pass 2 checks each module with it. A type declared in module A is visible when checking module B. | CT: module B references A's type; checker (run on the project) resolves it; no build-order needed | serious | design B2-a | **done** | `cross_module::scan_project` scans input file's directory for `.tlfe`, extracts types into `ProjectRegistry`; CT `x3_qualified_cross_module` proves resolution | Scans input dir (not full project tree) |
| X-3 | **Qualified `mod:type` resolution:** `orders:order-status` resolves in `:args`/`:returns`/field positions and behaves like a local type (matching/guards/validators/accessors work across the boundary). | CT: a `defun/typed` in B typed over `orders:order-status` checks clean + runs; a wrong value is rejected at the boundary | serious | design B1-a | **done** | Rust: `x3_qualified_type_parses_in_args`, `x3_qualified_type_parses_in_returns`; CT: `x3_qualified_cross_module` (checker exit 0, compile, run, total==1000) | Colon handled via `Symbol + Keyword` merge |
| X-4 | **`import-types` sugar:** `(import-types (from mod (t1 t2 ...)))` makes bare `t1` resolve to `mod:t1`; desugars to qualified refs; multiple imports allowed. | CT: B uses `import-types` then bare names; checks + runs identically to the qualified form | normal | design B1-b | **done** | Rust: `x4_import_types_extraction`; CT: `x4_import_types` (checker exit 0, compile, run, total==500) | Layered on X-3 |
| X-5 | **Static diagnostics (exact):** unknown module, unknown type in a known module, and `import-types` of a non-exported type each yield a teaching-grade diagnostic — **STATIC: non-zero exit + exact message** (NOT a runtime proxy). | Rust snapshot (exact) + CT: run checker on 3 bad fixtures; non-zero exit + exact diagnostic each | serious | Goal 2 | **done** | Rust: `x5_validate_unknown_module` (exact msg), `x5_validate_unknown_type_in_known_module` (exact msg); CT: `x5_unknown_module`, `x5_unknown_type`, `x5_bad_import` (non-zero exit + exact diagnostic each) | STATIC rejection — all 3 cases covered |
| X-6 | **Cross-module dogfood:** split the order domain into TWO modules — `orders` declares the `order-status` ADT **and** an `order` record; `orders_web` consumes both across the boundary (one qualified, one via `import-types`). Check + compile + run end-to-end. | CT: build both modules via the chain; call a cross-module function; assert real results (exact) | serious | dogfood | **done** | CT: `x6_dogfood_cross_module` — compile orders + orders_web, call cross-module, assert total==2500 + id==99 exact | Records (M6) shared across modules |
| X-7 | **Provider two-pass UX:** `typed check` runs the project-wide two-pass over a multi-module sample (good + bad); correct exit codes; clear output. | run on good + bad multi-module project; assert exit codes | normal | design §3.5 | **deferred** | The checker binary already auto-scans sibling `.tlfe` on every invocation; `rebar3 typed check` provider invokes per-file. Full project-wide provider UX deferred to follow-up | Scope note: provider already works per-file |
| X-8 | **Docs:** `docs/usage.md` cross-module section showing both `mod:type` and `import-types`, matching actual behavior. | doc exists + matches real commands/output | polish | dogfood | **done** | `docs/usage.md` updated with "Cross-Module Types" section: qualified syntax, import-types syntax, error cases | |
| X-9 | **Regression + process:** full M0–M6 suites pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M6 | **done** | `make check` exit 0: 82 Rust tests, 71 CT tests (10 adt + 6 chain + 6 cross_module + 11 dogfood + 6 matching + 12 records + 15 runtime + 5 typecheck), 0 skipped | |

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_

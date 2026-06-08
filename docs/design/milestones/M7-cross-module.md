# Milestone M7 — Cross-Module Type References

> **Goal:** let a `.tlfe` module use a type declared in *another* module — the last
> gate between "types a module" and "types a *project*" (gap inventory #8).
> **Builds on:** M0–M6 (all closed). The cross-module **producer already exists** —
> every module emits a `typed-registry` attribute (`lower::lower_registry_attr`); M7
> builds the **consumer**. Records (M6) give M7 a real shared type to demo.
> **Design:** [03-capability-unlock.md](../03-capability-unlock.md) Part B — decided:
> **B1-c both** `mod:type` + `import-types`; **B2-a re-read source + project-wide
> two-pass** discovery.
> **Ledger:** [M7-cross-module-ledger.md](M7-cross-module-ledger.md). **CC prompt:**
> [M7-cc-prompt.md](M7-cc-prompt.md). **Iteration budget:** 5.

## Why now, and what's already done

A real project is many modules. Today every type must live in the same file that
uses it (gap #8). The serialization half is already built: `lower_registry_attr`
emits, per type, `(name (params...) repr (ctors-with-fields...))` into the module's
`typed-registry` attribute. M7 is therefore a **consumer** problem:

1. **deserialize** a registry s-expr back into an `AdtDef` (the inverse of
   `lower_registry_attr`);
2. **discover** the registries of sibling modules without a build-ordering
   dependency;
3. **resolve** a cross-module type reference in the type env;
4. **diagnose** unknown module / unknown type — *statically*, teaching-grade.

## Decided mechanism (design Part B)

- **Reference syntax — both.** Qualified `mod:type` always works; `import-types` is
  optional sugar that desugars to qualified refs. **Build qualified first**, then
  layer `import-types` on top.
  ```lisp
  ;; qualified — always works
  (defun/typed handle :args ((s orders:order-status)) :returns string :body ...)

  ;; OR import then bare
  (import-types (from orders (order-status order)))
  (defun/typed handle :args ((s order-status)) :returns string :body ...)
  ```
- **Discovery — re-read source, project-wide two-pass.** Pass 1: scan the project
  (the directory tree rooted at the input file's project root) for all `.tlfe`,
  extract every `deftype`/`defrecord/typed` into a **combined type registry**. Pass
  2: check each module with the full registry in the type env. No new build artifact,
  no topological ordering. (Producer registry attr stays for runtime/external
  consumers and a future sidecar/incremental mode.)

## In scope

- **Registry deserializer** (`registry s-expr → AdtDef`): the exact inverse of
  `lower_registry_attr`, round-trip-tested (`lower` then parse back == original).
- **Project two-pass scan:** given the input file, locate the project root and scan
  for sibling `.tlfe`; pass 1 builds the combined registry; pass 2 checks with it.
  A type declared in module A is visible when checking module B.
- **Qualified `mod:type` resolution** in `:args` / `:returns` / record+ctor field
  positions: `orders:order-status` resolves to that type and behaves like a local
  type (matching, guards, validators, accessors all work across the boundary).
- **`import-types` form:** `(import-types (from mod (t1 t2 ...)))` makes bare `t1`
  resolve to `mod:t1`; desugars to qualified refs. Multiple imports allowed.
- **Static diagnostics (teaching-grade, exact, non-zero exit):** unknown module
  (no such registry), unknown type in a known module, and an `import-types` of a
  type the module doesn't export — each a *static checker rejection*, not a runtime
  proxy.
- **Cross-module dogfood:** split the order domain into TWO modules — `orders`
  declares the `order-status` ADT **and** an `order` **record**; `orders_web`
  consumes both across the boundary (one via qualified `mod:type`, one via
  `import-types`). Check + compile + run end-to-end, exact CT.
- **`rebar3` provider:** `typed check` runs the project-wide two-pass over a
  multi-module sample (good + bad), correct exit codes.
- **Docs:** `docs/usage.md` cross-module section (both syntaxes).
- **Full M0–M6 regression**; standing discipline.

## Out of scope (later)

- Sidecar `.typed-registry` files / incremental builds / caching the scan (the
  two-pass re-read is the v0; promote later if cost demands).
- Cross-module **function** signatures (M7 is cross-module *types*; importing typed
  *functions* across modules is a separate concern).
- Reading the registry from compiled `.beam` (we re-read source).
- Diamond/cyclic type dependencies beyond what the two-pass naturally handles;
  versioned/namespaced type identities; package-level type visibility rules.

## Definition of done

The registry deserializer round-trips (exact); a project-wide two-pass makes a
type in module A visible in module B; both `mod:type` and `import-types` resolve and
a function typed over a cross-module type (incl. a shared record) checks/compiles/
runs (exact CT); unknown-module / unknown-type / bad-import each yield a **static**
teaching diagnostic (non-zero exit, exact); the two-module dogfood works end-to-end;
provider two-pass UX correct; full M0–M6 regression green; `make check` clean.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; **test the actual subject** — here, a
type *actually resolving across the boundary* and the *static* rejection of unknown
refs; unwired ≠ done; status honesty) · [[cc-editing-safety]] (no blind `sed`) ·
[[lfe-ct-tests-in-lfe]] (CT in LFE) · **diagnostic rows specify STATIC checker
rejection (non-zero exit + exact), never a runtime stand-in** (the recurring-pattern
guard: M1/M3/M4-2/M4.5/M4.6/M5/M6 all substituted runtime for a static criterion).

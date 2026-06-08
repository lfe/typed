# Design Note 03 — Capability Unlock: Records + Cross-Module Types

> Status: **decision-seeking draft.** Frames the two backlog items that gate
> "type a *project*, not a *module*" (gap inventory #6 + #8). Records first
> (contained, independent, and makes the better cross-module demo), then
> cross-module type references. The genuine user-facing syntax choices are
> Duncan's (Goal 1 = lovely syntax); mechanism choices carry a recommendation.

## Why these two, why now

M0–M5 proved the hard thing: gradual static checking + always-on runtime
enforcement + ADTs, working end-to-end on a real module. But `orders.tlfe` is a
*single* module, and its only product type (an order) would have to be modelled
as a one-constructor sum — "heavy for what's really a record" (gap #6). And no
`.tlfe` file can reference a type declared in another (gap #8). Those two limits
are exactly the difference between *types a module* and *types a project*.

The good news from the code audit: the **producer half of cross-module already
exists.** Every checked module emits a `typed-registry` module attribute
(`lower::lower_registry_attr`) carrying, per type: name, type-params, repr, and
constructors-with-fields. Cross-module is therefore a *consumer* problem, not a
serialization problem.

---

## Part A — Typed records (M6)

A record is a single-constructor product type with named, typed fields and
generated accessors. Mechanically this is a thin layer over the existing ADT
machinery (a one-constructor `deftype` already works; what's missing is ergonomic
surface + generated accessors the checker understands).

### Fork A1 — surface form (Duncan's call)

- **A1-a `defrecord/typed` (standalone form).** A dedicated top-level form:
  ```lisp
  (defrecord/typed order
    (id integer)
    (status order-status)
    (items (list order-line))
    (total integer))
  ```
  Pro: reads as a record, signals intent, no `(repr ...)` ceremony. Con: a second
  top-level form to learn alongside `deftype`.

- **A1-b single-constructor `deftype` sugar.** Keep one form; when a `deftype` has
  exactly one constructor whose name matches the type, treat it as a record and
  generate accessors:
  ```lisp
  (deftype order
    (order (id integer) (status order-status) (items (list order-line)) (total integer)))
  ```
  Pro: one concept. Con: still the "heavy" shape gap #6 complained about; the
  record-ness is implicit.

- **A1-c `(repr record)` on `deftype`.** A record is just a repr choice:
  ```lisp
  (deftype order (repr record)
    (fields (id integer) (status order-status) ...))
  ```
  Pro: fits the existing pluggable-repr model. Con: `(fields ...)` is novel; mixes
  "repr" (a representation concern) with "is-a-record" (a shape concern).

**Recommendation: A1-a (`defrecord/typed`).** It's the lovely-syntax answer, and
it desugars cleanly to a one-constructor `deftype` internally so the checker/runtime
machinery is reused wholesale. The standalone form is worth one extra keyword.

### Fork A2 — accessor + update API (mostly mechanical; light recommendation)

LFE's own records generate `(order-id o)`, `(make-order ...)`, `(set-order-id o v)`.
Proposal: generate typed analogues the checker knows the types of —
`(order-id o) :: integer`, a typed constructor `(make-order id ... )`, and
functional update `(order-with o (id 5))` or `(set-order-id o v)`. Updates return a
new record (immutable, BEAM-idiomatic). Accessors get runtime guards like any typed
function head.

**Recommendation:** generate `make-<rec>`, `<rec>-<field>` accessors, and a
functional `set-<rec>-<field>` updater; defer bulk/record-update sugar to later.

### M6 shape (additive, contained)

Surface `defrecord/typed` → desugar to one-ctor ADT → generate typed
accessors/constructor/updater → checker resolves field types → runtime guards on
accessors. Dogfood: rewrite `orders.tlfe`'s order concept as a real record.

---

## Part B — Cross-module type references (M7)

Module `web` wants to use `order-status` (and a record `order`) declared in module
`orders`. Producer (registry attr) exists; we build the consumer.

### Fork B1 — reference syntax (Duncan's call)

- **B1-a qualified `mod:type`** in type position, mirroring `mod:fun`:
  ```lisp
  (defun/typed handle :args ((s orders:order-status)) :returns string :body ...)
  ```
  Pro: maximally LFE-idiomatic, zero new declarations, locally explicit about
  origin. Con: the sexp lexer must treat `orders:order-status` correctly in type
  position (today field types are stored as flat strings, so it's representable, but
  the colon handling needs a deliberate decision — see the M4.6 colon lesson).

- **B1-b import declaration**, then bare names:
  ```lisp
  (import-types (from orders (order-status order)))
  ;; ... later ...
  :args ((s order-status))
  ```
  Pro: bare names read clean; explicit dependency list per module (nice for tooling
  + the registry consumer's discovery). Con: a new declaration; names can collide
  across imports.

- **B1-c both** — qualified always works; `import-types` is optional sugar.
  Pro: best ergonomics. Con: most surface to build/test.

**Recommendation: B1-a (`mod:type`) for M7, leave B1-b as a later sugar.** It's the
idiomatic minimum, needs no new declaration, and the import form can be added
non-breakingly once the qualified path works.

### Fork B2 — registry discovery mechanism (recommendation; Duncan can veto)

How does the checker, when checking `web.tlfe`, obtain `orders`' registry?

- **B2-a re-read dependency source.** Given a module path, find `orders.tlfe`,
  parse it, extract its `deftype`s. Pro: no new artifact; the checker already parses
  `.tlfe`. Con: needs a source-resolution path; re-parses on every check.

- **B2-b sidecar registry file.** Producer writes `orders.typed-registry` (the same
  s-expr `lower_registry_attr` already builds) next to the source; consumer reads
  it. Pro: clean producer/consumer decoupling; cheap to load; matches existing
  serialized format. Con: a build artifact to manage + keep fresh.

- **B2-c read the `.beam` attribute.** The registry is already in the compiled beam.
  Pro: single source of truth, no new artifact. Con: Rust reading BEAM attrs is
  awkward (parse BEAM or shell to `erl`); requires deps compiled first.

**Recommendation: B2-a (re-read source) for M7's first cut**, via a project-wide
**two-pass** check (pass 1: scan all project `.tlfe`, build the combined type
registry; pass 2: check each module with the full registry in the type env). It
avoids a topological build dependency and a new artifact, and the checker already
has the parser. Promote to a sidecar (B2-b) later if re-parsing cost or incremental
builds demand it.

### Fork B3 — build ordering (follows from B2)

With the two-pass project scan (B2-a), there's no ordering constraint: all type
declarations are collected before any module is checked. The rebar3 provider checks
the project, not file-by-file in isolation. (A sidecar/beam approach would need
topological ordering; the two-pass sidesteps it.)

### M7 shape

Registry **deserializer** (s-expr → `AdtDef`, the inverse of
`lower_registry_attr`) + project-wide two-pass scan populating the type env +
qualified `mod:type` resolution in arg/return/field positions + diagnostics for
unknown module/type. Dogfood: split `orders` into two modules (`orders` declares
types incl. a record; `orders_web` consumes them across the boundary), check +
compile + run end-to-end.

---

## Sequencing

M6 (records) first — independent, contained, and gives M7 a *record* type to share
across modules (the real "type a project" demo). M7 (cross-module) second, built on
the existing registry producer. Each under ledger discipline, CC implements / CDC
verifies, 5-iteration cap.

## Decisions — DECIDED (Duncan, 2026-06-07)

1. **Record surface form → A1-a `defrecord/typed`.** Standalone form, desugars to a
   one-constructor `deftype` internally.
2. **Cross-module reference syntax → B1-c BOTH.** Qualified `mod:type` always works;
   `import-types` is optional sugar for bare names. (Build qualified first; layer
   `import-types` on top — it desugars to qualified refs.)
3. **Registry discovery → B2-a re-read source + project-wide two-pass.** Pass 1
   scans all project `.tlfe` to build the combined type registry; pass 2 checks each
   module with the full type env. No new artifact, no build ordering.

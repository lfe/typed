# Milestone M1 — ADTs & Representation

> **Goal:** real **algebraic data types** — `deftype` with named-field, parametric
> constructors — that **construct**, **lower across the pluggable `repr` backends**,
> emit a **cross-module registry**, and are checked for **constructor
> well-formedness**, all proven equivalent across backends by the matrix tests.
> **Builds on:** [M0 (closed)](M0-skeleton.md) — the model-Y chain + line injection.
> **Design refs:** [design v0](../01-design-v0.md) §3.2a (tiers), §4.1 (ADT surface),
> §5 (repr), §6 (system scope); [Audit 2](../audits/02-erlang-data-type-taxonomy.md) §7 (carriers).
> **Ledger:** [M1-adts-ledger.md](M1-adts-ledger.md). **CC prompt:** [M1-cc-prompt.md](M1-cc-prompt.md).
> **Iteration budget:** 5. Roles: CC implements, CDC verifies independently.

## What M1 adds to the (working) M0 chain

M0 proved the chain on a trivial `defun/typed`. M1 puts the first real *content*
through it: a sum-of-products type, its constructors, and the representation seam.

```
  shapes.lfe   (deftype + constructions)
      │  read with column-aware reader
      ▼
  typed-check (Rust):  parse deftype → ADT into type env → parse constructions →
      CHECK constructor well-formedness (unknown ctor / bad field / bad arity) with
      line+col diagnostics → LOWER constructions per chosen `repr` → emit ADT
      registry as a module attribute → [{plain-lfe-form, orig-line}]
      ▼  EETF → thin Erlang driver → lfe_codegen:module + compile:forms → BEAM
      ▼  matrix: same surface program, each repr backend, asserted equivalent
```

## Provisional surface (syntax NOT final — semantics are)

```lisp
;; sum of products, parametric, named fields
(deftype (result ok err)
  (Ok    (value  ok))
  (Error (reason err)))

;; all-nullary sum (enum repr)
(deftype colour (Red) (Green) (Blue))

;; newtype (transparent repr)
(deftype customer-id (CustomerId (v integer)))

;; per-type repr selection (optional; defaults per OTP version)
(deftype (result ok err) (repr tagged-tuple) (Ok (value ok)) (Error (reason err)))

;; construction — named fields (provisional)
(Ok :value 42)            ;; or (make Ok :value 42) — to be settled in syntax phase
```

## In scope

- **`deftype`**: parse name + type params + constructors (named fields + field types),
  into the checker's type environment. Module-local + parametric.
- **Construction**: parse + lower a constructor-construction form per the chosen repr.
- **Constructor well-formedness checking (structural)**: unknown constructor, unknown
  or missing field, wrong arity → Tier-1 diagnostic with **line + column**.
- **Pluggable `repr` backends** (the seam + ≥2 backends fully):
  - `tagged-tuple` (**required**, default on OTP < 29): `(Ok :value 42)` → flat
    `{'Ok', 42}` (snake_case tag, Gleam layout — Audit 2 §7).
  - `enum` (**required**): all-nullary sum → atoms.
  - `transparent` (**should**): 1-ctor/1-field newtype → the payload itself.
  - `native-record` (**code; runtime deferred**): true distinct type on OTP 29+;
    runtime matrix assertions deferred to a 29+ toolchain (Duncan is on 28).
- **`repr` selection** + default resolution (native-record on 29+, tagged-tuple on <29).
- **Registry emission**: ADT definitions emitted as a custom `.beam` **module
  attribute** (the cross-module type interface) + free Erlang `-type` breadcrumb.
- **Backend-matrix tests**: the SAME ADT surface program built on each testable
  backend, runtime representation asserted; native-record axis present but deferred.
- **Line-injection regression**: ADT-form errors / crashes still report original
  source line (don't regress M0's F-8/F-9).
- **CT suites in LFE** (`test/*_SUITE.lfe`) per the LFE project examples + the in-repo
  `typed_chain_SUITE.lfe`.

## Out of scope (later milestones)

- **Pattern matching / deconstruction / exhaustiveness** — M2 (incl. field *access*
  beyond what construction tests need).
- **Field-VALUE type checking** (does `42` satisfy `ok`?) — needs expression typing;
  arrives with contracts. M1 checks *structure* (ctor/field/arity), not value types.
- **Function contracts** (`defun/typed` checking) — M3.
- **`dynamic()` / untyped interop**, cross-module registry *consumption* — M4+.
- **Derived equality / ordering** semantics — later (Audit 2 §5); M1 asserts raw
  representation, not derived `Eq`/`Ord`.
- **Consuming** another module's registry — M1 only *emits* it.

## Environment notes

- OTP 28 / LFE 2.2.1. `native-record` needs OTP 29+, so its runtime matrix rows are
  `deferred` with re-entry "when a 29+ toolchain/runner is available."
- Keep using the M0 mechanism (`lfe_codegen:module` + `compile:forms`, NOT
  `lfe_comp:forms/2`); semi-internal API coupling still applies.

## Definition of done

Every ledger row reaches a final status with SHA-anchored, reproducible evidence
(or justified `deferred`/`no-op`). Required backends (`tagged-tuple`, `enum`) are
`done` and run-verified on OTP 28; `native-record` runtime is `deferred`. The
per-row walk + CDC re-verification close M1.

## Note on size

This is a meatier milestone than M0. If it runs to iteration 4–5, that is the
signal to split (e.g. carve the extra backends into an M1.5) rather than grind —
per the iteration-budget discipline.

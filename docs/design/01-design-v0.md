# typed — Design Document (v0)

> **Project:** `typed` — an experiment in a statically typed LFE with algebraic
> data types.
> **Status:** draft for review. First design-altitude document of the 9-point
> SDLC (research → **project definition + design** → milestones → impl plan).
> **Builds on:** [Audit 1 — spec surface](../audits/01-erlang-spec-surface-area.md),
> [Audit 2 — data-type taxonomy](../audits/02-erlang-data-type-taxonomy.md),
> [Audit 3 — ADTs in other typed Lisps](../audits/03-adts-in-other-typed-lisps.md).
> **Grounding:** OTP **30.0-rc0** source; LFE + `rebar3_lfe` source (file:line
> citations inline). Where syntax is shown it is **provisional** — semantics and
> architecture are the subject of this doc, surface syntax is deferred.

---

## 1. Project definition

### 1.1 What `typed` is

A **non-forking LFE library plus a companion checker** that adds static, algebraic
types to ordinary LFE. You keep writing LFE; you opt into typed forms; nobody has
to adopt a different language or compiler. Two components:

- **`typed` (LFE macro library)** — the typed *surface* (ADT declarations,
  function contracts, typed pattern matching) and the *lowering* to plain LFE.
- **`typed-check` (Rust)** — the *checker*: reads the macro-expanded program,
  rejects ill-typed code, and emits teaching-quality diagnostics.

### 1.2 Goals (the two deliverables, restated)

1. **A lovely typed syntax for LFE** — intuitive, low-cognitive-load, in the
   spirit of the Lykn contract style; the syntax *helps* rather than adding
   ceremony.
2. **A suite of static-analysis + ADT debugging tools** whose **output is a
   teaching surface** — informative, educational diagnostics that help *both
   humans and LLMs* turn mistakes into correct typed LFE (the Elm/Rust/Gleam bar).

### 1.3 Non-goals (v0)

- Not a fork of LFE; not a new BEAM language.
- No type classes, no higher-kinded types (Gleam's deliberate minimalism —
  Audit 3 §5).
- No global Hindley-Milner inference (contracts are annotation-first; we do
  bidirectional checking with *local* inference only).
- No soundness across process / message / distribution boundaries — the BEAM's
  untyped seams are handled with an explicit `dynamic()` boundary, not type-checked
  through (deferred research problem).
- Not relying on Dialyzer as the safety net: Dialyzer is unreliable for LFE once
  macros/includes are involved, so our checker is **load-bearing** (Audit 1 §6).

### 1.4 Success criteria (v0)

- A user can declare a parametric ADT, construct/deconstruct it, and get a
  **compile-time rejection** with a precise, teaching-quality message when a
  `case`/`match` is non-exhaustive or a contract is violated.
- The same typed program compiles and runs **identically across every supported
  `repr` backend**, proven by one test suite over the full matrix.
- Zero changes to the LFE compiler; `typed` ships as a hex dep + a `rebar3_lfe`
  provider.

---

## 2. Guiding principles

1. **No fork.** Everything is a library + build step. (Audit 3: this is the
   single strongest predictor of survival — Gleam lived, Alpaca died.)
2. **Contract-first.** Types are stated at function boundaries (Lykn style); the
   checker checks *against* them. This makes global inference unnecessary.
3. **Diagnostics-first.** Error output is a product surface, not an afterthought.
4. **Check before lower.** The checker sees the typed, pre-codegen AST; we never
   lose structure (e.g. exhaustiveness) by lowering first (the Typed Racket
   cautionary tale — Audit 3 §2).
5. **Pluggable representation.** A per-type `repr` knob (Coalton's model — Audit 3
   §8); semantics are identical across backends, proven by the test matrix.
6. **Write to the floor.** This doc states what v0 *achieves*; deferrals are named
   in §11–§12, not buried.

---

## 3. Architecture

### 3.1 The pipeline

```
  your_module.lfe   (uses typed: deftype / defun/typed / typed match)
        │
        ▼
  ┌─────────────────────────────────────────────────────────────┐
  │  LFE macro expansion  (Component A: the `typed` macro library)│
  │   • typed macros LOWER to plain LFE for the chosen repr        │
  │   • AND emit a REGISTRY: define-type/defspec (→ real -type/    │
  │     -spec) + custom `(typed-registry …)` module attributes     │
  │     capturing pre-lowering ADT/contract/match structure        │
  └─────────────────────────────────────────────────────────────┘
        │  lfe_comp:file(File, [to_expand])  → macro-expanded, pre-codegen forms
        ▼
  ┌─────────────────────────────────────────────────────────────┐
  │  typed-check  (Component B: Rust)                              │
  │   • parse LFE sexprs → reconstruct typed AST from registry     │
  │   • bidirectional check vs contracts; exhaustiveness; ctor     │
  │     arity/payload; unknown ctor                                │
  │   • emit Gleam-grade diagnostics; return pass / fail           │
  └─────────────────────────────────────────────────────────────┘
        │  pass?  ── no ──▶  reject build, print diagnostics
        │  yes
        ▼
  normal LFE compile (lfe_comp: expand → lint → codegen → BEAM)
```

The build-step ordering is enforced by a `rebar3_lfe`-style provider: `typed check`
runs (or is wired as a pre-compile hook) and **gates** `lfe compile`. This is the
"check before lower" invariant made operational — the checker examines the
`to_expand` output (post-macro, pre-Erlang-codegen), the altitude at which our
registry still describes the ADTs explicitly.

### 3.2 Why two components, in two languages

- The **surface must be LFE** — it's how users write code and it expands to LFE.
  (Component A.)
- The **checker is Rust** — decided: team Rust fluency + 10-year Rust toolchain +
  in-flight Rustler/LFE work; the Gleam precedent (a Rust checker for a BEAM ADT
  language); Rust's best-in-class diagnostics crates (Goal 2); and it eliminates
  the Alpaca "untyped compiler is unsafe to evolve" risk (Audit 3 §9).
- **Cost owned:** we forgo *checker self-application* (a Rust checker can't be
  typed by `typed`). We recover the design-completeness oracle by dogfooding
  `typed` on **real LFE codebases**, and Component A (the LFE library) remains a
  future `typed` target. (See §10.)

### 3.3 Grounding in real LFE / rebar3_lfe mechanisms

Every hook below was verified in the mounted source — this architecture uses
existing extension points, it does **not** require patching LFE:

| Need | Mechanism | Evidence |
|---|---|---|
| Define the typed surface macros | `defmacro` → `define-macro` | `lfe_macro.erl:1010` |
| Ship macros to consumers | `(export-macro …)` → synthesized `LFE-EXPAND-EXPORTED-MACRO/3` | `lfe_macro_export.erl:143, 91` |
| Cross-module macro call `(typed:deftype …)` | `exp_call_macro` | `lfe_macro.erl:1084` |
| Get post-macro, pre-codegen AST | compiler `to_expand` stop flag | `lfe_comp.erl:246` |
| Emit registry that survives compile | custom module attributes pass through verbatim | `lfe_codegen.erl:157` |
| Free Dialyzer breadcrumbs | existing `deftype`/`defspec` → real `-type`/`-spec` | `lfe_macro.erl:996–1004`; `lfe_codegen.erl:401, 415` |
| Read a module's forms programmatically | `lfe_io:parse_file/1` + `lfe_macro:expand_fileforms/4` | `lfe_io.erl:72`; `lfe_macro.erl:145` |
| Register the `typed check` command | `providers:create` + `rebar_state:add_provider` (namespace `lfe`) | `r3lfe_prv_compile.erl:25–41`; `rebar3_lfe.erl:15` |
| Order check before compile | provider `{deps, [{lfe, compile}]}` or `provider_hooks` | `r3lfe_prv_repl.erl:22` |
| Invoke LFE compiler from provider | `lfe_comp:file/2` | `r3lfe_compile_worker.erl:49` |

> Note: LFE has **no** `parse_transform` equivalent (verified). The supported
> integration is exactly the `to_expand` + provider path above — which is why the
> Rust checker runs as a separate build step rather than an in-compiler pass.

### 3.4 The registry (Component A → B contract)

The macros must hand the checker everything it needs to check *before* lowering
erases it. Two carriers, both confirmed to survive to the expanded forms / BEAM:

1. **Standard `define-type` / `define-function-spec`** — gives real Erlang
   `-type`/`-spec` (documentation + any Dialyzer that does work, for free).
2. **Custom `(typed-registry …)` module attributes** — the structured payload the
   checker actually consumes: constructor definitions (name, named fields + field
   types, parametricity), the chosen `repr`, function contracts, and the
   high-level (pre-lowering) shape of each typed `match`/`case` so exhaustiveness
   is checkable.

**Open sub-decision (impl-plan):** serialization across the LFE→Rust boundary.
Candidates: emit the `to_expand` forms as **S-expression text** (Rust parses
sexprs — trivial, homoiconic-friendly) — *recommended default*; or Erlang External
Term Format (EETF) for fidelity. Leaning sexpr-text for v0.

### 3.5 Checker invocation (recommended, open to redline)

How the provider runs the Rust binary:

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **Standalone binary / port** | crash isolation (a checker bug can't take down the build VM); simplest; Gleam-style | ship per-platform binaries (plugin fetches, or cargo build) | **Recommended** |
| **Rustler NIF** | no separate binary; in-VM call; leverages our Rustler expertise | a NIF crash kills the build VM; long checks need dirty schedulers | tempting, but isolation matters more at build time |
| **escript/port over EETF** | clean message protocol | extra protocol surface | viable alternative |

Recommendation: **standalone binary invoked as a port** by the provider;
distribute precompiled binaries (Gleam's approach), `cargo`-buildable fallback.

---

## 4. The surface language (v0) — *provisional syntax*

Semantics are firm; the exact tokens are not (we agreed to defer syntax). Shown
to make the semantics concrete.

### 4.1 ADT declaration — named-field constructors, parametric

```lisp
;; provisional
(deftype (result ok err)
  (Ok    (value ok))               ; constructor Ok with named field `value`
  (Error (reason err)))            ; constructor Error with named field `reason`

(deftype (tree a)
  (Leaf)
  (Node (left (tree a)) (val a) (right (tree a))))
```

- **Named fields** (decided; positional sugar maybe later — Audit 3 §3).
- **Parametric** type params (`ok`, `err`, `a`).
- Nullary constructors (`Leaf`) lower to atoms; constructors-with-fields lower per
  `repr` (§5).

### 4.2 Function contracts — Lykn-style

```lisp
;; provisional
(defun/typed greet
  (:args  ((name (binary))))
  (:returns (binary))
  (:body (binary:list_to_bin (list "Hello, " name "!"))))
```

The contract is the annotation the checker checks against (no inference needed at
the boundary). Overloaded clauses and `when`-style constraints (Audit 1 §2.2) are
in scope but must not break the low-ceremony feel — a surface-design task.

### 4.3 Typed pattern matching with exhaustiveness

```lisp
;; provisional
(case/typed r
  ((Ok v)    (handle v))
  ((Error e) (recover e)))         ; omit a constructor ⇒ checker REJECTS
```

The macro preserves the constructor-level match shape in the registry so the
checker can verify exhaustiveness **before** lowering to plain LFE `case`.

---

## 5. Runtime representation & the pluggable backend

Per-type `repr` knob (Coalton's model — Audit 3 §8). Semantics identical across
all; only the lowering differs.

| `repr` | Carrier | OTP | Notes |
|---|---|---|---|
| **`native-record`** (default) | native record `#Ctor{…}` — a *true distinct type* (`is_record` true, `is_tuple`/`is_map` false) | **29+** ⚠️ experimental | nominal identity the runtime enforces; closes the coincidental-shape hazard (Audit 2 §1) |
| **`tagged-tuple`** (fallback) | flat tagged tuple `{ctor, F1, …}` (snake_case tag, **Gleam layout**, not Alpaca's nested) | any | idiomatic, interops with hand-written LFE/Erlang |
| **`enum`** | atoms | any | all-nullary sums |
| **`transparent`** | the payload itself (wrapper erased) | any | zero-overhead newtypes (`CustomerID`) |

- **Default** = `native-record` on OTP 29+, `tagged-tuple` on older OTP.
- **Interop:** for `tagged-tuple`, optionally generate LFE record headers so plain
  LFE/Erlang can pattern-match our ADT values (Gleam's `.hrl` trick — Audit 3 §4).
- **Test matrix:** one suite × all backends. The two places carrier-independence is
  *not* free — derived **equality** (`=:=` vs `==`, `±0.0`) and **ordering** (term
  order follows atom-spelling + carrier, not declaration order) — are exactly what
  the matrix must pin (Audit 2 §5). Native-record term-order position is still
  **unverified** (Audit 2 §7.3) — pin empirically early.

---

## 6. The type system (v0 scope)

- **Discipline:** bidirectional checking against declared contracts; **local**
  inference inside bodies; **no** global Algorithm W, **no** type classes, **no**
  HKT (Audit 3 §5). "TypeScript-for-LFE with real ADTs," not "OCaml-for-LFE."
- **Types we support:** the ADT sums/products above; parametric type constructors;
  and the Erlang spec surface we choose to expose (Audit 1 §7 checklist) — built-in
  types/aliases, tuples, maps, lists, binaries, funs, ranges, singletons, records.
- **The interop boundary:** `dynamic()` (Audit 1 §3.11) is the one blessed type for
  values crossing into untyped Erlang/OTP (and `pid()`/`port()` seams). Calls to
  untyped Erlang require an annotated boundary (Gleam's `@external` discipline).
- **Equivalence:** default constructor equality is `=:=` (structural identity);
  nominal tag distinctness via native records / `-nominal` where available
  (Audit 1 §2.1, Audit 2 §5).

---

## 7. The checker (Component B)

**Input:** macro-expanded forms (registry attributes + function bodies + match
shapes), via the provider's `lfe_comp:file(File, [to_expand])`.

**Checks (v0):**

1. **Contract conformance** — each `defun/typed` body checks against its `:args`/
   `:returns` (bidirectional).
2. **Exhaustiveness** — every typed `case`/`match` covers all constructors of its
   scrutinee's sum; **non-exhaustive ⇒ rejection naming every missing
   constructor** (match/beat Gleam — Audit 3 §6).
3. **Constructor well-formedness** — arity, named-field correctness, payload types.
4. **Unknown constructor / type** references.
5. **Coincidental-shape warning** — a bare tuple/atom matching an ADT carrier shape
   (Audit 2 §9), strongest for the `tagged-tuple` backend.

**Output:** structured diagnostics (§8); a non-zero result gates the build.

## 8. Diagnostics design (Goal 2)

Adopt **Gleam's error grammar** (Audit 3 §7), implemented with Rust's diagnostic
crates (ariadne / miette / codespan):

- Source span + caret underline of the offending form.
- For non-exhaustive matches: an explicit *"These values are not matched: …"* list.
- An actionable **`Hint:`** block ("to get an `ok` out of a `result`, match like …").
- Alias-aware type rendering (use the names as written in the module).
- Multi-error collection (don't stop at the first).
- **Machine-readable mode** — the same diagnostics emitted as structured data
  (JSON) so LLMs/tools can consume and act on them. Designing for humans and
  machines is the *same* work; this is the LLM half of Goal 2.

---

## 9. Testing strategy

- **Full backend matrix:** every semantic test runs across `native-record`,
  `tagged-tuple`, `enum`, `transparent` (where applicable). This is the proof that
  the surface is carrier-independent and the safety net Alpaca lacked.
- **Diagnostics snapshots:** golden-file tests on the rendered error output (a la
  the project's existing `insta` discipline) — diagnostics are a product, so they
  get regression tests.
- **Property tests:** round-trip construct/deconstruct; "well-typed programs
  compile and run identically across backends"; "ill-typed programs are rejected."
- **Checker correctness ≠ checker types.** Rust's type system catches shape bugs in
  the checker; the matrix + property tests catch *logic* bugs (wrong rule). Both
  needed (the lesson from the self-application discussion).
- **Pin the unknowns:** a tiny OTP-30 test fixes native-record term-order position
  (Audit 2 §7.3).

---

## 10. Self-application / dogfooding (revised for the Rust pivot)

- The **checker** (Rust) is not `typed`-checkable — accepted cost.
- The **design-completeness oracle is recovered** by running `typed` against real
  LFE codebases (the existing LFE ecosystem) — a more representative oracle than
  typing a compiler.
- **Component A (the LFE macro library) stays a future `typed` target** — partial
  dogfooding survives.
- The forcing function remains: if `typed` can't comfortably type real LFE
  programs, the ADT/contract story is too weak.

---

## 11. Milestone roadmap (proposed)

| M | Title | Delivers |
|---|---|---|
| **M0** | Skeleton & plumbing | `typed` lib scaffold + `typed-check` Rust crate; `rebar3_lfe` `typed check` provider that runs `to_expand`, ships forms to Rust, gates compile; sexpr reader in Rust; CI + backend-matrix harness. |
| **M1** | ADTs + representation | `deftype` (named-field, parametric); construction/lowering across **all four `repr` backends**; registry emission; matrix tests green. |
| **M2** | Exhaustiveness + diagnostics | typed `case`/`match`; **non-exhaustive ⇒ rejection** naming missing ctors; Gleam-grade diagnostic renderer + JSON mode; snapshot tests. |
| **M3** | Contracts | `defun/typed`; bidirectional checking of bodies vs `:args`/`:returns`; ctor arity/field/payload checks. |
| **M4** | Interop boundary | `dynamic()` + annotated calls into untyped Erlang/OTP; coincidental-shape warning; record-header generation for `tagged-tuple`. |
| **M5** | Polish & dogfood | `transparent` newtypes, `enum` ergonomics; run `typed` against a real LFE codebase (oracle); overloaded specs / `when` constraints if surface allows. |

Each milestone runs with a ledger (collaboration framework) and is reviewed before
the next.

## 12. Open questions carried to the implementation plan

1. **Registry serialization** across LFE→Rust (sexpr-text recommended vs EETF).
2. **Checker invocation** (standalone binary/port recommended vs Rustler NIF).
3. **Surface syntax** — the actual tokens for `deftype`/`defun/typed`/`case/typed`
   (deliberately deferred; must preserve low-ceremony feel).
4. **How much local inference** inside bodies vs pure check-against-contract.
5. **Native-record term-order position** — empirically pin on OTP 30 (Audit 2 §7.3).
6. **Binary distribution** of the Rust checker (precompiled per platform vs build).
7. **Cross-module types** — referencing ADTs/contracts across modules (remote types,
   `export-type`).

---

*Grounded in: OTP 30.0-rc0 source; LFE source (`lfe_macro.erl`,
`lfe_macro_export.erl`, `lfe_comp.erl`, `lfe_codegen.erl`, `lfe_lint.erl`,
`lfe_io.erl`, `lfe_types.erl`); `rebar3_lfe` source (`rebar3_lfe.erl`,
`r3lfe_prv_*.erl`, `r3lfe_compile_worker.erl`); Audits 1–3. Syntax provisional;
architecture and semantics are the reviewable substance.*

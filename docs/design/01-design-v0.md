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

**Decided (experiment-backed): model Y — we own the compile chain.** Rather than
gating LFE's own compile, `typed` reads the source, checks it, lowers it itself,
and drives codegen — like Gleam. This is what lets us stamp *original-source*
positions onto generated code (LFE's macro expander discards positions; our
lowering doesn't). Verified in
[experiment 01](experiments/01-lfe-line-anno-probe.md) on OTP 28 / LFE 2.2.1.

```
  your_module.lfe   (typed surface: deftype / defun/typed / case/typed)
        │
        ▼  read source with a COLUMN-AWARE reader (oxur-derived)
  ┌──────────────────────────────────────────────────────────────────┐
  │  typed-check  (Rust)                                               │
  │   • parse the typed surface DIRECTLY → ADT-level AST (line+col)    │
  │   • bidirectional check vs contracts; exhaustiveness; ctor checks  │
  │   • emit Gleam-grade diagnostics with line+COLUMN   ── Tier 1      │
  │   • on PASS: lower each typed form → plain-LFE form, tagged with   │
  │     its ORIGINAL source line  →  [{plain-lfe-form, orig-line}, …]  │
  └──────────────────────────────────────────────────────────────────┘
        │  hand off lowered forms + original lines  (EETF or sexpr)
        ▼
  ┌──────────────────────────────────────────────────────────────────┐
  │  thin Erlang driver                                                │
  │   • lfe_lint:module(Forms, …)         % honors our lines           │
  │   • lfe_codegen:module(Forms, #cinfo{file=OrigFile}) → Erlang AST  │
  │   • compile:forms(AST, [{source, OrigFile}, …])      → BEAM        │
  └──────────────────────────────────────────────────────────────────┘
        │
        ▼  BEAM — stack traces & compile errors report ORIGINAL file+line (Tier 2)
```

The typed surface is parsed and lowered by *our* tool, **not** by LFE's macro
expander. That is the crux: checking the *source* (not the `to_expand` output)
preserves both the ADT structure (which lowering erases) and positions (which
expansion erases — verified in the position dig). "Check before lower" is therefore
literal: the checker works on the source-level ADT AST; lowering happens only after
a pass, inside our tool, where we control the line stamping. The `rebar3_lfe`-style
provider drives this whole chain (check → lower → codegen → BEAM); there is no
separate "normal LFE compile" of the original typed source.

**Line injection is real (experiment 01).** `lfe_codegen:module([{Form, L}, …],
#cinfo{file=F})` followed by `compile:forms(AST, [{source, F}, …])` makes a stamped
line `L` and file `F` surface in **runtime stack traces** *and* in both
**LFE-lint** and **erlc** compile errors — for code with no physical line `L`.
Granularity is **per-function** (`lfe_translate:to_expr` threads one line to all of
a function's sub-nodes); per-expression needs an upstream `to_expr` change (§ Tier 3).

### 3.2 The two components

- **`typed-check` (Rust) — all the smarts.** Parses the typed surface, type-checks,
  lowers to plain LFE forms (stamping original lines), renders diagnostics. Keeping
  every "understanding" of typed code in one *typed* language is the direct
  mitigation of the Alpaca "untyped compiler is unsafe to evolve" risk (Audit 3 §9).
  Decided language for: team Rust fluency + 10-year Rust toolchain + in-flight
  Rustler/LFE work; the Gleam precedent; best-in-class diagnostics crates (Goal 2).
- **Thin Erlang driver — dumb plumbing.** Takes the lowered `[{Form, OrigLine}, …]`
  and calls `lfe_lint:module/2` → `lfe_codegen:module/2` → `compile:forms/2`. Small,
  low-risk, and *must* be Erlang because `lfe_codegen` is. The `rebar3_lfe` provider
  invokes the Rust binary and this driver in sequence.
- **No fork, still.** Output is vanilla LFE/`.beam` any LFE/OTP consumes; the user
  just runs our `rebar3` build step (a plugin — the LFE-ecosystem norm). Same deal
  as Gleam: own the chain, emit ordinary BEAM artifacts.
- **Cost owned:** the Rust checker can't be `typed`-checked (no self-application).
  We recover the design-completeness oracle by dogfooding `typed` on **real LFE
  codebases** (§10).

### 3.2a Diagnostics: three tiers (the precision story)

1. **Type errors (our checker) — line + COLUMN.** Free: they're found in our own
   AST, read from source with a column-aware reader; they never cross into LFE/BEAM.
2. **Downstream compile/runtime errors → original file + LINE, per-function.** Free
   via line injection (experiment 01). This is the Elixir trick, now verified on LFE.
3. **Per-expression / column downstream — DEFERRED.** Needs `lfe_translate:to_expr`
   to accept per-node lines (a concrete, small upstream contribution to propose to
   Robert — the "source-maps for LFE" prototype payoff), or Erlang-AST
   post-processing; column is unrecoverable in stack traces (the runtime strips it,
   confirmed). Tiers 1–2 are v0; tier 3 is the upstream collaboration.

### 3.3 Grounding in real LFE / rebar3_lfe mechanisms

Every hook below was verified in the mounted source — this architecture uses
existing extension points, it does **not** require patching LFE:

| Need | Mechanism | Evidence |
|---|---|---|
| Define the typed surface macros | `defmacro` → `define-macro` | `lfe_macro.erl:1010` |
| Ship macros to consumers | `(export-macro …)` → synthesized `LFE-EXPAND-EXPORTED-MACRO/3` | `lfe_macro_export.erl:143, 91` |
| Cross-module macro call `(typed:deftype …)` | `exp_call_macro` | `lfe_macro.erl:1084` |
| Read source with **column** precision | oxur-derived reader (NOT `lfe_io`, which is line-only) | [02-oxur-sexp-reuse.md](02-oxur-sexp-reuse.md) |
| Lower with original lines | `lfe_codegen:module([{Form,Line}], #cinfo{file})` honors custom lines | `lfe_codegen.erl:42,74,330`; experiment 01 |
| Lint with original lines | `lfe_lint:module([{Form,Line}], …)` honors custom lines | experiment 01 |
| Emit BEAM from forms | `compile:forms(AST, [{source, OrigFile}, …])` | experiment 01 |
| Cross-module type interface | custom module attributes survive to `.beam`, `beam_lib`-readable | `lfe_codegen.erl:157` |
| Free Dialyzer breadcrumbs | existing `deftype`/`defspec` → real `-type`/`-spec` | `lfe_macro.erl:996–1004`; `lfe_codegen.erl:401, 415` |
| Register the `typed check` command | `providers:create` + `rebar_state:add_provider` (namespace `lfe`) | `r3lfe_prv_compile.erl:25–41`; `rebar3_lfe.erl:15` |

> Note: LFE has **no** `parse_transform` equivalent (verified), and `lfe_comp:forms/2`
> auto-numbers forms (so it can't carry our lines — experiment 01). The integration
> is therefore the **provider-driven chain**: our Rust tool reads source, checks,
> and lowers; the thin Erlang driver calls `lfe_lint:module/2` → `lfe_codegen:module/2`
> → `compile:forms/2`. **API-coupling risk:** `lfe_codegen:module/2`, `lfe_lint:module/2`,
> and `#cinfo` are semi-internal — pin the LFE version and/or get these blessed as
> stable entry points (a collaboration touchpoint with Robert).

### 3.4 What crosses the component boundary

There is no "registry to carry past lowering" for the **current** module — the Rust
checker reads the source directly, so the typed forms *are* the ADT-level truth
(with precise spans). Two real handoffs remain:

1. **Rust → Erlang driver:** the lowered `[{plain-lfe-form, orig-line}, …]`. This is
   an internal protocol we control on both ends; **EETF** (lossless) is the safe
   default, sexpr-text the readable alternative.
2. **Cross-module type interface:** a compiled typed module ships its ADT/contract
   **registry as a custom module attribute** in its `.beam` (survives, `beam_lib`-
   readable). The checker reads a dependency's types from there (or re-reads its
   source). This is the one place a serialized registry genuinely earns its keep.

### 3.5 Checker invocation — decided

**Standalone Rust binary**, invoked by the `rebar3_lfe`-style provider (which also
runs the thin Erlang driver). Chosen for crash isolation at build time over a
Rustler NIF (a NIF crash takes down the build VM; long checks would need dirty
schedulers). Distribute precompiled per-platform binaries (Gleam's approach), with a
`cargo`-buildable fallback.

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
| **M0** | Skeleton & plumbing | `typed` lib scaffold + `typed-check` Rust crate; `rebar3_lfe` `typed check` provider that runs `to_expand`, ships forms to Rust, gates compile; sexpr reader **adopted from `oxur`'s `sexp/` module** (factored or vendored) + LFE-lexeme extension + `parse_all` (see [02-oxur-sexp-reuse.md](02-oxur-sexp-reuse.md)); CI + backend-matrix harness. |
| **M1** | ADTs + representation | `deftype` (named-field, parametric); construction/lowering across **all four `repr` backends**; registry emission; matrix tests green. |
| **M2** | Exhaustiveness + diagnostics | typed `case`/`match`; **non-exhaustive ⇒ rejection** naming missing ctors; Gleam-grade diagnostic renderer + JSON mode; snapshot tests. |
| **M3** | Contracts | `defun/typed`; bidirectional checking of bodies vs `:args`/`:returns`; ctor arity/field/payload checks. |
| **M4** | Interop boundary | `dynamic()` + annotated calls into untyped Erlang/OTP; coincidental-shape warning; record-header generation for `tagged-tuple`. |
| **M5** | Polish & dogfood | `transparent` newtypes, `enum` ergonomics; run `typed` against a real LFE codebase (oracle); overloaded specs / `when` constraints if surface allows. |

Each milestone runs with a ledger (collaboration framework) and is reviewed before
the next.

## 12. Open questions carried to the implementation plan

1. ~~Registry serialization~~ — **resolved** (§3.4): Rust→Erlang handoff = EETF
   default; cross-module interface = registry in `.beam` module attributes.
2. ~~Checker invocation~~ — **resolved** (§3.5): standalone Rust binary.
3. **Where does lowering live, Rust or Erlang?** Decided lean: Rust owns
   parse+check+lower (smarts in one typed language); thin Erlang driver does
   `lfe_lint`/`lfe_codegen`/`compile:forms`. Confirm the split in impl-plan.
4. **Surface syntax** — the actual tokens for `deftype`/`defun/typed`/`case/typed`
   (deliberately deferred; must preserve low-ceremony feel).
5. **How much local inference** inside bodies vs pure check-against-contract; and
   the v0 treatment of opaque interiors (`dynamic()` vs the soundness spectrum).
6. **Native-record term-order position** — empirically pin on OTP 30 (Audit 2 §7.3).
   Note: native records are **OTP 29+**; on OTP 28 the default backend is
   `tagged-tuple` (validates pluggable + matrix).
7. **Per-expression source mapping (Tier 3)** — propose `lfe_translate:to_expr`
   per-node lines upstream (with Robert), or Erlang-AST post-processing.
8. **Binary distribution** of the Rust checker (precompiled per platform vs build).
9. **Cross-module types** — referencing ADTs/contracts across modules (remote types,
   `export-type`), read from dependency `.beam` registry attributes.
10. **LFE internal-API coupling** — pin LFE version / bless `lfe_codegen:module/2`,
    `lfe_lint:module/2`, `#cinfo` as stable (with Robert).

---

*Grounded in: OTP 30.0-rc0 source; LFE source (`lfe_macro.erl`,
`lfe_macro_export.erl`, `lfe_comp.erl`, `lfe_codegen.erl`, `lfe_lint.erl`,
`lfe_io.erl`, `lfe_types.erl`); `rebar3_lfe` source (`rebar3_lfe.erl`,
`r3lfe_prv_*.erl`, `r3lfe_compile_worker.erl`); Audits 1–3. Syntax provisional;
architecture and semantics are the reviewable substance.*

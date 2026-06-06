# Audit 3 — ADTs in Other Typed Lisps (and BEAM ADT Languages)

> **Project:** `typed` — an experiment in a statically typed LFE with algebraic
> data types.
> **Audit question:** *What have other typed Lisps done for their ADTs? Any
> low-level, algebraic, conceptual, or strategic learnings we can borrow?*
> **Status:** complete, pending review.
> **Companions:** [Audit 1 — spec surface](01-erlang-spec-surface-area.md),
> [Audit 2 — data-type taxonomy](02-erlang-data-type-taxonomy.md).

## Scope & method

Five systems, chosen to triangulate our exact problem from three directions:

- **Typed Lisps embedded in an untyped Lisp** (our structural analog):
  **Coalton** (HM in Common Lisp), **Typed Racket**, **Hackett** (HM + type
  classes in Racket).
- **Statically-typed ADT languages on the BEAM** (our runtime analog):
  **Gleam**, **Alpaca**.

Research was fanned out across three parallel agents (lookup delegated, synthesis
kept central, per the collaboration framework). All claims are sourced; URLs at
the end. A standout external survey worth reading in full later: Mariano Guerra,
*"A tour through the BEAM ADT representation zoo"* (2020).

---

## 1. The field at a glance

| System | Host → target | Type discipline | ADT declaration | Fields | Exhaustiveness | Macro × typecheck | Diagnostics | Status |
|---|---|---|---|---|---|---|---|---|
| **Coalton** | CL → CL | HM + type classes + HKT | `define-type` (sum); `define-struct` (named product) | positional (struct=named) | **warn**, names missing ctor | CL macros expand **then** typecheck (macros can't see types) | good, source-located; some cryptic edges | pre-1.0, active |
| **Typed Racket** | Racket → Racket | Occurrence typing + gradual | `struct` + `U` union + `define-type` alias | **named** | **none** (match hides it) | typecheck **after** full macroexpansion | informative, dense for polymorphism | mature |
| **Hackett** | Racket → Racket | Full HM (bidirectional) + classes + HKT | `data` | positional | **yes** | typecheck **interleaved with** expansion (Turnstile) | author called them "atrocious" | WIP/experimental |
| **Gleam** | own → **Erlang src** | HM (Algorithm W), **no classes, no HKT** | `type` with variants | **labelled** (erased at runtime) | **compile error**, names missing | own compiler (not macro-based) | **the gold standard** | v1+, very active |
| **Alpaca** | own → **Core Erlang** | HM (sound/eager) | `type ... = A | B` | positional (+ row-poly records) | partial, **warn** | own compiler | "comically hostile" (its words) | **unmaintained since 2018** |

Five rows, and almost every cell is a lesson. The rest of this audit pulls them
into decisions.

---

## 2. The deepest axis: macro × typecheck architecture

This is the axis that matters most *because we are a macro library*. There are
three camps, in increasing order of power and cost:

1. **Expand, then check (Coalton, Typed Racket).** Macros are ordinary host
   macros; they expand first; the typechecker walks the *expanded* forms.
   Coalton: "Just define a normal Common Lisp macro and it'll Just Work." The
   price, stated plainly by Coalton's authors: **macros cannot see or influence
   types** during expansion. Typed Racket checks the fully-expanded kernel form,
   so a "typed macro" must expand to *typed kernel forms* or its type info is
   lost at the boundary.
2. **Interleave check with expansion (Hackett, via Turnstile / "Type Systems as
   Macros", POPL 2017).** Each surface form *is* a macro that embeds type-
   inference rules; typechecking happens *as* expansion, so macros both read and
   shape types. Hackett's ADTs aren't primitives — `data`/`case` are library
   macros. Maximum power; it effectively builds the type system into the expander.
3. (For completeness, Gleam/Alpaca aren't in this taxonomy — they're standalone
   compilers, no user macros.)

**What this means for `typed`.** Our constraint — *a non-forking LFE library*,
not a new compiler and not a re-engineered expander — points squarely at **camp
1, the Coalton model**: `defun/typed` (and the ADT forms) are ordinary LFE macros
that (a) record type/contract info into a compile-time registry and (b) expand to
plain LFE; a **separate checker pass** consumes the registry and rejects. This is
the achievable altitude for an experiment, and it preserves the "it's just LFE"
promise. The Turnstile route is seductive and more powerful, but it *is*
"a different LFE" in all but name — exactly what Duncan ruled out.

**The Typed-Racket cautionary tale (critical, architectural).** TR has **no
exhaustiveness checking** — not because it's hard in theory, but because Racket's
`match` expands to a form with an implicit catch-all *before* the typechecker
runs, so the checker literally cannot see that a case was omitted. **If we lower
our typed `match`/`case` to plain LFE `case` before our checker sees it, we
inherit exactly this failure** and lose the single most valuable ADT diagnostic
(non-exhaustive match). Therefore:

> **Architectural rule for `typed`:** the checker must run against the *typed
> ADT-level AST* (or a registry that preserves constructor/match structure)
> **before** lowering to LFE `case`. Check first, lower second. This is the
> concrete lesson camp-1 systems teach by their split outcomes — Coalton checks
> the match and warns; TR lowered first and lost the ability.

---

## 3. ADT declaration & fields — Duncan's "named fields" call, validated

How each declares a sum of constructors:

- **Coalton:** `(define-type (Expr :t) (EConst :t) (E+ (Expr :t) (Expr :t)))` —
  positional fields; a *separate* `define-struct` exists for named-field products
  with `.field` accessors. Two forms for two needs.
- **Typed Racket:** orthogonal composition — `struct` (named fields, gives
  constructor + predicate + accessors) combined with `U` for the sum and
  `define-type` to name it. Sums and products are separate, composable pieces.
- **Hackett:** `(data (Maybe a) Nothing (Just a))` — positional, Haskell-style.
- **Gleam:** `pub type SchoolPerson { Teacher(name: String, subject: String) … }`
  — **labelled fields**, and both `Teacher(name: "A", subject: "B")` and
  positional `Teacher("A", "B")` work; **labels are erased at runtime**.
- **Alpaca:** `type opt 'a = Some 'a | None` — positional; separate row-polymorphic
  anonymous records for named fields.

**Finding:** **Gleam is the precedent for Duncan's exact decision** — labelled
constructor fields, positional accepted, labels erased in the runtime term. It is
also the only one of the five that's *on the BEAM*, so its choice is the most
directly transferable. This strongly validates "named fields now, positional
sugar maybe later": it's the proven-ergonomic BEAM model, and it lowers cleanly
onto **both** our carriers (native records *are* named-field; the tuple fallback
drops labels to positions — exactly Gleam's erasure).

A secondary borrow: the **Coalton split** (`define-type` for sums vs
`define-struct` for named products) vs the **Typed Racket composition** (`struct`
+ `U`) are two coherent ways to factor the surface. Since we want *one* low-
cognitive-load surface, the Gleam unification (one `type` form whose variants
carry named fields) is the better fit than two forms — and it matches our
contract aesthetic.

---

## 4. BEAM runtime representation — the zoo, and our edge

The two BEAM languages are the direct evidence for Audit 2's carrier question.

**Gleam** compiles to **Erlang source**, with this term mapping (from the
official externals guide):

| Gleam | Erlang term |
|---|---|
| nullary variant `Guest` | atom `guest` (PascalCase → snake_case) |
| variant with fields `User(id: 10)` | **flat** tagged tuple `{user, 10}` |
| labels | **erased** (tuple is positional) |
| `Ok(v)` / `Error(v)` | `{ok, V}` / `{error, V}` (matches Erlang idiom) |

Crucially, Gleam **generates an Erlang `.hrl`** with a record per variant, so
plain Erlang code can pattern-match Gleam data with `#variant{}` syntax. All
types are erased; **no runtime checks**.

**Alpaca** compiles to **Core Erlang**, with: nullary → atom (`Nil` → `'Nil'`);
with-fields → tag + **nested** tuple payload (`Cons(1, Nil)` → `{'Cons',
{1,'Nil'}}`); anonymous records → maps with a `'__struct__'` key (**not**
Erlang-record-compatible).

**Findings and decisions:**

1. **Convergent standard:** nullary constructor → atom; constructor-with-fields →
   tagged tuple whose tag is the (snake_cased) constructor name. This is the
   de-facto BEAM ADT encoding (and matches Audit 2 §7). **Our tagged-tuple
   fallback should adopt it** — specifically **Gleam's *flat* layout** (`{user,
   10}`), not Alpaca's nested payload, because flat is more idiomatic, cheaper to
   match, and interops directly with hand-written Erlang/LFE.
2. **Generated record header for interop:** Gleam's `.hrl`-per-variant trick is
   worth stealing for the fallback backend — it lets plain LFE/Erlang consume our
   ADT values ergonomically. (For LFE this might be generated records or
   `include`-able defs.)
3. **Our genuine edge:** *both* Gleam and Alpaca predate OTP 29, so neither could
   use **native records**. Our default backend can make constructor values a
   **true distinct runtime type** (`is_record` true, `is_tuple`/`is_map` false) —
   closing the "coincidental-shape" false-positive that Gleam/Alpaca both leave
   open (their `{user, 10}` is indistinguishable from a hand-written tuple). This
   is a real, defensible novelty: *nominal ADTs on the BEAM with runtime-distinct
   carriers*, which no prior BEAM language had the substrate to do.
4. **Type erasure is universal.** Every system erases types at runtime; checks are
   compile-time. Confirms our model: emit ordinary LFE, do all the checking at
   compile time, no runtime type tax (except whatever the carrier itself costs).

---

## 5. Inference vs annotation — and a pull to name

Where they sit on the inference spectrum:

- **Coalton / Hackett:** full HM inference; annotations optional (`declare`);
  type **classes** and **HKT**. Maximalist.
- **Typed Racket:** annotation-heavier; occurrence typing its specialty.
- **Gleam:** full HM (Algorithm W), but annotations **never required** for
  correctness *except on `@external`* (FFI must be annotated, since the compiler
  can't see foreign code). **Deliberately no type classes, no HKT.** The stated
  reasons: simpler error messages, faster compiles, and easier consumption from
  other languages.
- **Alpaca:** HM; optional `val` annotations; FFI must be annotated.

**The pull I want to name:** my corpus draws hard toward the Coalton/Hackett
maximalist end — full HM inference, type classes, HKT — because it's the
"impressive" answer and it's what the typed-FP literature celebrates. **Gleam is
the counter-evidence that should govern us**, and its three reasons are *our*
three goals wearing different clothes: simpler errors = our teaching-diagnostics
goal; easier cross-language consumption = our no-fork/ecosystem goal; faster
compiles = developer ergonomics. So the disciplined call for an experiment is to
**start minimalist: ADTs + a small HM-ish core, no type classes, no HKT**, and
add power only when a concrete need forces it.

**A surface-specific realization that makes this easier still.** Our Lykn-style
contract surface (`:args`/`:returns`) is **annotation-first at function
boundaries by design** — Duncan *wants* the explicit contract. That means v0 does
**not** need global type inference at all: it needs **bidirectional checking**
(check expressions against the declared contract; infer only locally inside a
body). This is dramatically more tractable than Algorithm W, matches the
"expressions + specs" expansion Duncan described, and aligns with Gleam's
"annotate the boundary" instinct. **We are closer to "TypeScript for LFE with
real ADTs" than to "OCaml for LFE,"** and that's a feature.

---

## 6. Exhaustiveness — our build-tier centrepiece

Exhaustiveness is the ADT check that pays the rent, and the field splits sharply:

- **Gleam — gold standard:** a **compile error** (it *rejects*), naming the exact
  unmatched variants:
  ```
  error: Not exhaustive pattern match
  …
  These values are not matched:
    - Technician
  ```
  (v1.6 "variant inference" even narrows types across arms so you needn't match
  variants already excluded.)
- **Hackett:** implemented.
- **Coalton:** a **warning**, names the missing constructor; fallthrough is
  *undefined behaviour*.
- **Alpaca:** partial, warnings.
- **Typed Racket:** **none** (the §2 architectural failure).

**Decision:** match Gleam — **non-exhaustive match is a rejection, and the
message names every missing constructor.** This is the flagship of Goal 2's
"ADT debugging tools," it's the thing Dialyzer can't do for LFE (Audit 1 §6), and
§2 tells us *how* to keep the ability: check before lowering. Coalton's "warning,
undefined fallthrough" is the weaker bar we should exceed.

---

## 7. Diagnostics — proven existential, with a template to copy

The single clearest cross-language signal in this whole audit:

- **Gleam** treats friendly errors as a headline feature and it shows: source
  spans with caret underlines, **named missing variants**, **actionable hints**
  ("If you want to get an `Int` out of a `Result(Int, a)` you can pattern match on
  it: …"), **context-aware** rendering (uses your import aliases), and it
  **collects multiple errors** per pass. This *is* the Elm/Rust bar Duncan set.
- **Coalton:** genuinely good — beautiful, source-located, names expected vs
  actual types — but with cryptic edges (a malformed `lisp` form yields a raw CL
  `DESTRUCTURING-BIND` error).
- **Typed Racket:** informative but dense for polymorphic functions (dumps
  candidate signatures).
- **Hackett:** the author himself called the errors "atrocious."
- **Alpaca:** "almost comically hostile to usability" — **and this is cited as a
  reason it didn't gain adoption.**

**Finding:** on the BEAM, diagnostics quality tracks adoption almost one-to-one
(Gleam thrived, Alpaca died, and error quality is a named factor in both). This
**validates Goal 2 as make-or-break, not polish.** Concrete borrow: adopt
**Gleam's error grammar** as our template — span + caret, "these values are not
matched: …", an actionable **Hint:** block, alias-aware type rendering, and
multi-error collection. And our LLM-consumption angle is a *superset* of this:
structured, located, fix-suggesting messages are exactly what an LLM can act on,
so designing for humans and machines is the same work.

---

## 8. Coalton's `repr` system = our pluggable backend, already designed

The most directly reusable *mechanism* in the survey. Coalton lets each type
choose its runtime representation:

- `:enum` — all-nullary sum → plain symbols (atoms). Cheapest.
- `:transparent` — single-ctor single-field → wrapper **erased**; the value *is*
  the payload (zero-overhead newtype).
- `:lisp` — distinct CLOS classes, `typep`-checkable (true nominal).
- `:native` — wrap an existing host type.

This is *exactly* the pluggable-backend idea Duncan chose, but factored
**per-type** rather than per-project. Borrow it: `typed` can expose
representation control with BEAM-appropriate options —

| Our `repr` | BEAM carrier | Use |
|---|---|---|
| `native-record` (default, 29+) | true distinct type | general ADTs |
| `tagged-tuple` (fallback) | `{tag, …}` flat | older OTP / interop |
| `enum` | atoms | all-nullary sums |
| `transparent` | payload itself | newtypes (`CustomerID`) |

This reframes our "default 29+, fallback older" decision as the *project-wide
default* of a finer, per-type knob — strictly more flexible, and it gives the
all-backend test matrix a clean axis to enumerate. **Transparent newtypes** in
particular are a cheap, high-value early feature (the `CustomerID` idiom appears
across Coalton and the wider typed-FP world).

---

## 9. The Alpaca cautionary tale — which risks transfer to us

Alpaca is the most important *negative* data point: an ML-with-ADTs on the BEAM
that **stalled (no release since Jan 2018)**. Documented reasons:

1. **Its compiler was written in (untyped) Erlang**, which made iterating on the
   type inferencer slow and error-prone — a concrete obstacle Gleam's creator
   cited, and the reason Gleam was (re)written in Rust.
2. **Hostile error messages** (§7).
3. **FFI ergonomics friction.**
4. **ML purity over pragmatic interop** (auto-currying, ML syntax) — a narrower
   audience; Gleam chose familiar syntax + easy two-way interop and won the users.

**Which of these transfer to `typed`, honestly:**

- **(1) is our real risk.** Our checker will be written in **LFE — also untyped**.
  Alpaca's exact drag. Three mitigations, in order: (a) Duncan's choice of a
  *macro library with a checker pass* rather than a *whole compiler* shrinks the
  surface enormously — we're not maintaining a lexer/parser/codegen, just macros +
  a checker over the LFE AST; (b) the **full-matrix test suite** Duncan already
  committed to is precisely the safety net Alpaca lacked; (c) eventual
  *dogfooding* — once `typed` can type LFE, the checker can (partly) type itself.
  This one deserves to stay on the risk register.
- **(2) is already neutralized** — diagnostics are Goal 2, a first-class deliverable.
- **(3)/(4) are already neutralized by the no-fork decision** — we *are* LFE,
  interop is native, syntax is familiar LFE, there's no new language to learn.
  The Gleam-vs-Alpaca verdict is the strongest external validation of Duncan's
  founding choices.

---

## 10. Where `typed` sits that nothing else does

Triangulating the five systems, our position is genuinely unoccupied:

- Like **Coalton**: typed islands embedded in an untyped Lisp via macros,
  compiling to the host — *but* contract-first (annotate boundaries) rather than
  infer-everything, and far lighter (no separate `coalton-toplevel` world).
- Like **Gleam**: BEAM target, named-field constructors, type erasure, friendly
  diagnostics — *but* a non-forking **library inside LFE**, not a separate
  language and toolchain.
- Beyond **both**: **native records (OTP 29+) as a true-distinct-type carrier**,
  which no prior BEAM ADT language could use — nominal ADTs the runtime itself
  distinguishes.
- Unlike **Typed Racket**: we *will* check exhaustiveness, by checking before
  lowering (§2).
- Unlike **Alpaca**: diagnostics-first, no-fork, with a test-matrix safety net.

The one-line position: **"contract-first, non-forking, nominal ADTs for LFE, with
Gleam-grade teaching diagnostics, on OTP-29 native records."** No existing system
occupies that point.

---

## 11. Borrowable decisions (consolidated)

1. **Architecture:** Coalton-style *expand-then-check* via a compile-time registry
   + separate checker pass — but **check the typed AST before lowering** so
   exhaustiveness survives (TR lesson). *(§2)*
2. **Surface:** one Gleam-style `type`/constructor form with **named fields**
   (positional sugar later); contract-first `defun/typed`. *(§3, §5)*
3. **Inference:** **bidirectional checking against contracts**, local inference
   only — *not* global Algorithm W; **no type classes, no HKT** in v0. *(§5)*
4. **Representation:** **per-type `repr` knob** (Coalton model): `native-record`
   default (29+), `tagged-tuple` flat fallback (Gleam layout), `enum` for nullary
   sums, `transparent` newtypes. *(§4, §8)*
5. **Interop:** generate record headers so plain LFE/Erlang can consume ADT values
   (Gleam `.hrl` trick); native records give true distinctness on 29+. *(§4)*
6. **Exhaustiveness:** a **rejection** that names every missing constructor
   (match/beat Gleam). *(§6)*
7. **Diagnostics:** adopt **Gleam's error grammar** — span+caret, "not matched:
   …", actionable **Hint:**, alias-aware rendering, multi-error collection;
   structured enough for LLMs to act on. *(§7)*
8. **Risk register:** LFE-implemented checker is Alpaca's failure mode — lean on
   the test matrix and keep the surface small (library, not compiler). *(§9)*

---

## 12. Open questions carried into design

1. **Checker hook:** how/where does the checker pass run in the LFE + rebar3
   compile pipeline (Coalton runs at `coalton-toplevel` macroexpansion; what's our
   equivalent hook, and does it need a rebar3 plugin)?
2. **Registry mechanism:** what compile-time store carries constructor/contract
   info from the macros to the checker (module attributes? a parse-transform-style
   table? a side file)?
3. **`repr` surface:** per-type opt-in syntax, and defaults per OTP version.
4. **Newtype (`transparent`) support** in v0 — cheap and high-value?
5. **How much HM** to actually implement for local inference vs pure
   check-against-contract.
6. Native-record term-order position still **unverified** (Audit 2 §7.3) — pin
   before relying on derived `Ord`.

---

## Sources

**Coalton:** [Introducing Coalton](https://coalton-lang.github.io/20211010-introducing-coalton/) ·
[manual: define-type](https://coalton-lang.github.io/manual/operators/define-type/) ·
[define-struct](https://coalton-lang.github.io/manual/operators/define-struct/) ·
[match](https://coalton-lang.github.io/manual/operators/match/) ·
[repr](https://coalton-lang.github.io/manual/operators/repr/) ·
[Lisp Interop](https://coalton-lang.github.io/manual/topics/lisp-interop/) ·
[Macros](https://coalton-lang.github.io/manual/topics/macros/) ·
[Coalton 0.2 Preview](https://coalton-lang.github.io/20260312-coalton0p2/)

**Typed Racket:** [Guide §4 Types](https://docs.racket-lang.org/ts-guide/types.html) ·
[§5 Occurrence Typing](https://docs.racket-lang.org/ts-guide/occurrence-typing.html) ·
[§6 Typed-Untyped Interaction](https://docs.racket-lang.org/ts-guide/typed-untyped-interaction.html) ·
[§8 Caveats](https://docs.racket-lang.org/ts-guide/caveats.html) ·
[Languages as Libraries (PLDI 2011)](https://www2.ccs.neu.edu/racket/pubs/pldi11-thacff.pdf) ·
[Type Systems as Macros (POPL 2017)](https://www.ccs.neu.edu/home/stchang/popl2017/)

**Hackett:** [GitHub README](https://github.com/lexi-lambda/hackett) ·
[Guide: Working with Data](https://lexi-lambda.github.io/hackett/guide-working-with-data.html) ·
[Realizing Hackett (2017)](https://lexi-lambda.github.io/blog/2017/05/27/realizing-hackett-a-metaprogrammable-haskell/)

**Gleam:** [Externals / data representation](https://gleam.run/documentation/externals) ·
[FAQ (no type classes/HKT)](https://gleam.run/frequently-asked-questions/) ·
[v0.20 exhaustiveness](https://gleam.run/news/gleam-v0.20-released/) ·
[v1.6 context-aware compilation](https://gleam.run/news/context-aware-compilation/) ·
[LambdaClass interview with Louis Pilfold](https://blog.lambdaclass.com/an-interview-with-the-creator-of-gleam-an-ml-like-language-for-the-erlang-vm-with-a-compiler/)

**Alpaca:** [GitHub README](https://github.com/alpaca-lang/alpaca) ·
[Tour.md](https://github.com/alpaca-lang/alpaca/blob/main/Tour.md) ·
[Gleam FAQ comparison](https://gleam.run/frequently-asked-questions/)

**Survey:** Mariano Guerra, [*A tour through the BEAM ADT representation zoo* (2020)](https://marianoguerra.org/posts/a-tour-through-the-beam-adt-representation-zoo/)

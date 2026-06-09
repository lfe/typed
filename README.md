# lfe/typed

[![Build Status][gh-actions-badge]][gh-actions]
[![LFE Versions][lfe-badge]][lfe]
[![Erlang Versions][erlang-badge]][version]
[![Tags][github-tags-badge]][github-tags]

*An experiment in gradually typed LFE with algebraic data types — checked at compile time, enforced at runtime*

[![Typed LFE project logo][logo]][logo-large]

> ⚠️ **Status: early & experimental.** `lfe/typed` is under active design. The
> *architecture* is settled and the compile chain works end-to-end; the *surface
> syntax* shown below is **provisional** — we've had only a few conversations about
> it, and most of that work is still ahead. Treat every code sample here as a
> sketch of the feel we're after, not a stable API.

---

## What is it?

`lfe/typed` brings a **gradual type system — with algebraic data types (ADTs)** — to
[LFE](https://lfe.io) (Lisp Flavoured Erlang), **checking your code at compile time and
enforcing it at runtime**. It's a **library plus a build step**, not a fork: you keep
writing LFE, you opt into typed forms where you want them, and the output is ordinary
BEAM bytecode that any LFE, Erlang, or Elixir code can call. There is no "different LFE"
to adopt.

*Gradual* because typing is opt-in: typed forms are statically checked and
runtime-enforced, while ordinary (untyped) LFE flows freely alongside them through an
explicit `dynamic` boundary.

Two things we care about above all else:

1. **A lovely typed syntax** — intuitive, low-ceremony, where the annotations *help*
   rather than getting in the way.
2. **Teaching-quality diagnostics** — error messages so clear and instructive that
   both *humans and LLMs* can turn a mistake into correct, typed LFE just by reading
   them. (Think Elm/Rust/Gleam-grade output.)

---

## A taste *(syntax is provisional!)*

Declaring algebraic data types — sums of products, with **named fields** and type
parameters:

```lisp
(deftype (result ok err)
  (Ok    (value  ok))
  (Error (reason err)))

(deftype (option a)
  (Some (value a))
  (None))

(deftype order-status
  (Pending)
  (Shipped   (tracking string))
  (Cancelled (reason   string)))
```

A function with a Lykn-style **contract** — the types live right at the boundary.
Here it is written the *wrong* way, matching on raw strings instead of the type's
constructors:

```lisp
(defun/typed describe
  (:args    ((status order-status)))
  (:returns string)
  (:body
    (case/typed status
      ("pending"   "queued")
      ("shipped"   "on its way")
      ("cancelled" "nevermind"))))
```

`typed` rejects it before it ever runs — and tries to *teach*, not scold:

```
error[pattern-type-mismatch]: this pattern can't match a value of type `order-status`
  ┌─ src/orders.lfe:24:7
  │
24│       ("pending"   "queued")
  │        ^^^^^^^^^ string pattern, but `status` has type `order-status`
  │
  = `order-status` is one of: Pending, Shipped, Cancelled
  = hint: match the constructors instead, e.g. (Pending) / (Shipped t) / (Cancelled r)
```

Written correctly — matching on the constructors, and exhaustively:

```lisp
(defun/typed describe
  (:args    ((status order-status)))
  (:returns string)
  (:body
    (case/typed status
      ((Pending)     "queued")
      ((Shipped   t) (++ "on its way: " t))
      ((Cancelled r) (++ "cancelled: " r)))))
```

*(Both the error output and the syntax are the bar we're aiming at — sketches of the
feel we're after, not screenshots of something that exists yet.)*

---

## Why build it this way?

A few convictions, earned from studying the field (see [`docs/audits/`](docs/audits)):

- **No fork.** Every statically-typed BEAM language that asked people to adopt a
  whole new language fought an uphill adoption battle; the ones that thrived made
  interop and familiarity easy. `typed` stays *inside* LFE.
- **Diagnostics are existential, not polish.** On the BEAM, friendly compiler errors
  track adoption almost one-to-one. So the tooling's *output* is a first-class
  product — and designing it well for humans turns out to be the same work as
  designing it well for LLMs.
- **Our checker is load-bearing.** Dialyzer is wonderful, but it becomes unreliable
  for LFE the moment macros and user LFE includes enter the picture — and a typed layer is
  nothing but macros and metaprogramming. So `typed` does its own checking, and aims
  to *reject* incorrect programs (which success-typing tools deliberately never do).
- **ADTs fit the BEAM glove.** Erlang already represents sum types as tagged tuples
  and atoms; ADT constructors map onto that idiom directly. On OTP 29+ we can go
  further and use **native records** — a genuinely distinct runtime type — as the
  carrier.

---

## How it works (the short version)

`lfe/typed` **owns the compile chain**, much like Gleam:

```
your .lfe  →  typed-check (Rust)        →       thin Erlang driver        →       BEAM
              • read source (line+column)       • lfe_codegen + compile:forms
              • check; reject with diagnostics  • original-source lines injected
              • lower to vanilla LFE
```

- The **checker is written in Rust** — for speed, for first-class diagnostics, and
  because a typed implementation language is far safer to evolve than an untyped one.
- It reads your **original source** (so type errors carry line *and column*), checks
  it, and lowers it to ordinary LFE.
- Generated code is stamped with your **original source positions**, so runtime stack
  traces and compile errors point back at *your* code, not at machine output. (This is
  proven working — see [`docs/design/experiments/`](docs/design/experiments).)
- Representation is **pluggable**: `native-record` (OTP 29+, default there),
  `tagged-tuple` (the portable default), `enum` for all-nullary sums, and
  `transparent` zero-cost newtypes — all behind one surface, all proven equivalent by
  a cross-backend test matrix.

---

## Architecture

`typed` is two cooperating pieces: a **Rust checker/compiler front-end** (`typed-check`)
and a **thin Erlang driver** that hands off to LFE's own code generator. The Rust side
owns everything up to "vanilla LFE forms"; the Erlang side turns those into BEAM.

### The pipeline

```
.lfet
source
   │
   ▼  ┌──────────────────────────── typed-check (Rust) ─────────────────────────────┐
   │  │ 1. read     S-expression reader (line+column), adapted from oxur            │
   │  │ 2. expand   Tier-1 macro expander (backquote, predef, def*, defrecord) —    │
   │  │             a faithful port of lfe_macro, oracle-validated                  │
   │  │ 3. check    bidirectional type checker over ADTs/records/contracts;         │
   │  │             rejects bad programs with teaching diagnostics (human + JSON)   │
   │  │ 4. lower    typed forms → vanilla LFE; insert always-on guards + validators;│
   │  │             emit a `typed-registry` module attribute; stamp original lines  │
   │  │ 5. encode   serialize the lowered forms to EETF (Erlang External Term Fmt)  │
   │  └─────────────────────────────────────┬───────────────────────────────────────┘
   │                                 .eetf  │
   ▼                                        ▼
             ┌──────────────── typed_driver (Erlang) ──────────────┐
             │ lfe_lint:module → lfe_codegen:module([{Form,Line}]) │
             │ → compile:forms (original-source lines preserved)   │
             └──────────────────────────┬──────────────────────────┘
                                        ▼
                                 BEAM bytecode
```

`rebar3` integration is a provider (`typed_prv_check`) that globs `*.lfet`, runs the
checker, and drives compilation; plain `*.lfe` files compile through the stock LFE
compiler untouched, so typed and untyped modules coexist in one project.

### Components

**Rust (`checker/src/`)** — the front-end:

- `sexp/` — the reader (lexer/parser/types), column-aware, adapted from oxur.
- `expander.rs` — the Tier-1 macro expander (backquote + core-form recursion + the
  static predef table + `def*` lowering + `defrecord`/`defstruct` generation), a
  faithful port of `lfe_macro` validated against an Erlang **oracle** + golden corpus.
- `adt.rs`, `type_env.rs` — ADT/record definitions, the type environment, and the
  cross-module type registry (qualified `mod:type` resolution).
- `typed_surface.rs` — parses the typed surface forms (`defun/typed`, `case/typed`, …).
- `typecheck.rs` — the **bidirectional** checker (synthesize/check), function signatures,
  contract/arg/field-value checking; `matching.rs` — exhaustiveness.
- `guards.rs`, `validators.rs` — always-on head guards (shallow) and deep
  validators/`decode` (the typed↔untyped membrane).
- `diagnostic.rs` — the diagnostic engine (span+caret, human and JSON renderers).
- `lower.rs`, `eetf.rs` — lowering to vanilla LFE (+ the registry attribute) and EETF
  serialization for the handoff.

**Erlang (`src/`)** — the back-end glue:

- `typed_driver.erl` — reads the EETF, runs `lfe_lint` + `lfe_codegen` + `compile:forms`.
- `typed_prv_check.erl` — the `rebar3 typed check` provider.
- `typed_rt.erl` — hand-written runtime support (e.g. `render-type-error`).

---

## Design decisions

The big-picture decisions, each with the road not taken. Full reasoning lives in
[`docs/design/`](docs/design) and the per-milestone ledgers.

- **Library + build step, not a fork.** You keep writing LFE and opt into typed forms;
  output is ordinary BEAM. *Considered & rejected:* a separate language (loses interop
  and familiarity — the adoption killer for typed BEAM languages).
- **"Model Y" — the checker owns the compile chain, written in Rust.** Like Gleam.
  Gives speed, first-class diagnostics, column-accurate positions, and a typed
  implementation language. *Considered & rejected:* an LFE-hosted checker / self-hosting
  in typed LFE (chicken-and-egg, and weaker diagnostics/positions).
- **Gradual typing.** Typed forms are opt-in; untyped LFE flows alongside through an
  explicit `dynamic` boundary. A `dynamic → T` crossing is only allowed via a checked
  `decode`, never a bare assertion.
- **Bidirectional checking, not global inference.** Synthesize where types are known,
  check against expected types elsewhere. *Considered & rejected:* whole-program
  Hindley-Milner (heavier, worse error locality, poor fit with gradual + BEAM interop).
- **Checked at compile time AND enforced at runtime.** Static checking catches errors in
  code the checker sees; **always-on guards** + **deep validators** catch bad data
  crossing in from the untyped world (HTTP/JSON/messages/ETS). Runtime violations become
  clean, localized crashes — BEAM "let it crash," not silent corruption.
- **Pluggable representation backends.** `native-record` (OTP 29+), `tagged-tuple`
  (portable default), `enum` (all-nullary sums), `transparent` (zero-cost newtype) —
  one surface, proven equivalent by a cross-backend test matrix.
- **Diagnostics are a first-class product.** Structured, teaching-grade errors with
  span+caret, in human and JSON form — designed to be equally legible to people and LLMs.
- **Faithful Tier-1 macro expander in Rust, oracle-validated.** The driver runs no
  `lfe_macro`, so the checker must expand macros itself; rather than reimplement
  backquote ad-hoc (which bit us twice), the expander is a faithful port graded against
  an Erlang oracle + golden corpus. *Boundary:* eval-time user macros and `.hrl`/QLC
  (Tier 2/3) are out of scope, delegated to the oracle/BEAM. Expansion stays in Rust to
  **preserve source positions** (the reason for Model Y in the first place).
- **EETF handoff.** Rust → Erlang via the Erlang External Term Format — the lowered
  forms are real Erlang terms `lfe_codegen` already understands.
- **Distinct file extension `.lfet`.** Typed source is genuinely a non-LFE format (the
  typed forms aren't stock-LFE-compilable), so it gets its own extension — keeps the stock LFE compiler from choking on typed files. *Future:* `.lfe` only once
  the LFE compiler itself can dispatch to `typed-check`.
- **One naming convention: `<lfe-form>/typed`.** `defun/typed`, `case/typed`,
  `deftype/typed`, … — typed variants are visibly marked and never shadow their LFE
  namesakes. (`import-types` is the documented exception — it shadows nothing.)

---

## Project design & provenance

This project was planned before it was built. The reasoning is all in the open:

- [`docs/audits/`](docs/audits) — three deep audits: Erlang's type-spec surface,
  Erlang's data-type taxonomy, and how other typed Lisps (Coalton, Typed Racket,
  Hackett, Gleam, Alpaca) handle ADTs.
- [`docs/design/`](docs/design) — the v0 design doc, the reuse assessment, and the
  feasibility experiment that settled the architecture.
- [`docs/design/milestones/`](docs/design/milestones) — milestone specs and their
  verification ledgers.

## Roadmap

- **M0 — Skeleton & plumbing** ✅ *(the chain + line injection, proven end-to-end)*
- **M1 — ADTs & representation** ✅ *(deftype, constructors, pluggable repr backends, registry)*
- **M2 — Pattern matching & exhaustiveness** ✅ *(case/typed, exhaustiveness rejection, the diagnostic engine)*
- **M3 — Function contracts & bidirectional checking** ✅ *(body-vs-:returns, call-arg, field-value checking)*
- **M3.5 — Cleanup** ✅ *(engine routing, branch typing, README demo, string/binary soundness)*
- **M4 — The typed/untyped interop boundary** ✅ *(always-on guards, structured type-errors, deep validators, decode membrane)*
- **M5 — Polish & dogfooding on real LFE** ✅ *(dogfood on orders.lfet, gap inventory, map errors, usage docs)*
- **M6 — Typed records** ✅ *(defrecord/typed, make-/accessors/set-, type-aware synthesis, registry)*
- **M7 — Cross-module type references** ✅ *(qualified mod:type, import-types, project scan, static diagnostics)*
- **M8 — Extension standardization** ✅ *(.tlfe → .lfet; provider/scanner globs; mixed .lfet/.lfe separation)*
- **M9 — Reader correctness** ✅ *(char, tuple, binary, quasiquote/unquote, cons dot; all 5 dirs files parse)*
- **M9.1 — Expander oracle + corpus** ✅ *(oracle escript, golden corpus for all Tier-1 categories, harness, conventions)*
- **M9.2 — Faithful backquote port** ✅ *(Rust exp_backquote, core-form recursion, run over all forms, qq_expand retired)*
- **M9.3 — Predef + validation gate** ✅ *(full Tier-1 predef table + defrecord + gensyms; 4/4 goldens GREEN via structural compare)*
- **M10 — Naming convention** *(`deftype` → `deftype/typed`; formalize the `<lfe-form>/typed` convention across all typed macros)*
- **M11 — Surface features** *(multi-clause `defun/typed` heads, `when` guards in clauses/patterns — the surface needed to type real LFE)*
- **M12 — Real-world port** *(port an existing module — [lfex/dirs](https://github.com/lfex/dirs) — to typed, reality-grading the design)*

Further out: per-expression source mapping (an upstream collaboration with LFE
itself), and a typed-ADT ↔ Rust bridge for [Rustler](https://github.com/rusterlium/rustler)
NIF boundaries.

---

## Status

`lfe/typed` is not yet packaged for `rebar3 add`, but the core static type system
works end-to-end: ADTs with pluggable representation backends, exhaustive pattern
matching that rejects non-exhaustive matches naming every missing constructor,
bidirectional contract checking (body-vs-`:returns`, call-arg, field-value),
always-on runtime guards with structured type-errors, deep validators/decode for
the typed/untyped boundary, typed records (`defrecord/typed`) with generated
constructors, accessors, and functional updaters — all type-aware — and cross-module
type references (`mod:type` and `import-types`) with project scanning. The diagnostic
engine renders Gleam-grade errors with span+caret in both human and JSON formats.
98 Rust tests, 82 LFE CT tests, `make check` clean.

If the ideas here interest you, the design docs are the best place to start, and
feedback (especially on syntax) is enormously welcome.

---

## Built on the shoulders of

- **[LFE](https://lfe.io)**, created by **Robert Virding** — the language this is all
  in service of, and a decade and a half of generous mentorship behind the ideas here.
- **Erlang/OTP** and the BEAM.
- Inspiration from **Gleam**, **Coalton**, **Typed Racket**, **Hackett**, and the
  lessons of **Alpaca**.
- The S-expression reader is adapted from **[oxur](https://github.com/oxur/oxur)**.
- Some of the syntax ideas (and all of the bravery) come from [Lykn](https://lykn.pl/).

## License

Apache-2.0. See [LICENSE](LICENSE). Package name: `lfe_typed`.

[//]: ---Named-Links---

[logo]: priv/images/Typed-LiffyBot.png
[logo-large]: priv/images/Typed-LiffyBot-large.png
[gh-actions-badge]: https://github.com/lfe/typed/actions/workflows/ci.yml/badge.svg
[gh-actions]: https://github.com/lfe/typed/actions
[lfe]: https://github.com/rvirding/lfe
[lfe-badge]: https://img.shields.io/badge/lfe-2.2+-blue.svg
[erlang-badge]: https://img.shields.io/badge/erlang-26+-blue.svg
[version]: https://github.com/lfe/typed/blob/main/.github/workflows/ci.yml
[github-tags]: https://github.com/lfe/typed/tags
[github-tags-badge]: https://img.shields.io/github/tag/lfe/typed.svg

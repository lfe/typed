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
82 Rust tests, 74 LFE CT tests, `make check` clean.

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

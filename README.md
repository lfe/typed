<!-- Logo: drop the LiffyBot-with-type-theory-mug art here, e.g.
     <p align="center"><img src="priv/images/liffybot-typed.png" width="300"></p> -->
<p align="center"><em>(LiffyBot logo coming soon — type theory on the mug)</em></p>

# typed

**An experiment in a statically typed LFE with algebraic data types.**

> ⚠️ **Status: early & experimental.** `typed` is under active design. The
> *architecture* is settled and the compile chain works end-to-end; the *surface
> syntax* shown below is **provisional** — we've had only a few conversations about
> it, and most of that work is still ahead. Treat every code sample here as a
> sketch of the feel we're after, not a stable API.

---

## What is it?

`typed` adds **static types and algebraic data types (ADTs)** to
[LFE](https://lfe.io) (Lisp Flavoured Erlang) — as a **library plus a build step**,
not a fork. You keep writing LFE. You opt into typed forms where you want them. The
output is ordinary BEAM bytecode that any LFE, Erlang, or Elixir code can call. There
is no "different LFE" to adopt.

Two things we care about above all else:

1. **A lovely typed syntax** — intuitive, low-ceremony, where the annotations *help*
   rather than getting in the way.
2. **Teaching-quality diagnostics** — error messages so clear and instructive that
   both *humans and LLMs* can turn a mistake into correct, typed LFE just by reading
   them. (Think Elm/Rust/Gleam-grade output.)

---

## A taste *(syntax is provisional!)*

Declaring an algebraic data type — a sum of products, with **named fields** and
type parameters:

```lisp
(deftype (result ok err)
  (Ok    (value  ok))
  (Error (reason err)))

(deftype (option a)
  (Some (value a))
  (None))
```

Writing a function with a Lykn-style **contract** — the types live right at the
boundary, where you'd want to read them:

```lisp
(defun/typed describe
  (:args    ((status order-status)))
  (:returns string)
  (:body
    (case/typed status
      ((Pending   o) (++ "queued: "   (order-id o)))
      ((Shipped   o) (++ "shipped: "  (tracking o)))
      ((Cancelled o) (++ "cancelled: " (reason o))))))
```

And the part we're really building toward — when a match misses a case, the tool
should *teach*, not scold:

```
error[non-exhaustive-match]: this `case/typed` doesn't cover every constructor
  ┌─ src/orders.lfe:24:3
  │
24│   (case/typed status
  │   ┬
  │   ╰─ scrutinee has type `order-status`
  │
  = not handled: Shipped, Cancelled
  = hint: add a clause for each, or a wildcard `_` if you mean to ignore them.
```

*(That error output is the bar we're aiming at — not a screenshot of something that
exists yet.)*

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
  for LFE the moment macros and includes enter the picture — and a typed layer is
  nothing but macros and metaprogramming. So `typed` does its own checking, and aims
  to *reject* incorrect programs (which success-typing tools deliberately never do).
- **ADTs fit the BEAM glove.** Erlang already represents sum types as tagged tuples
  and atoms; ADT constructors map onto that idiom directly. On OTP 29+ we can go
  further and use **native records** — a genuinely distinct runtime type — as the
  carrier.

---

## How it works (the short version)

`typed` **owns the compile chain**, much like Gleam:

```
your .lfe  →  typed-check (Rust)            →  thin Erlang driver        →  BEAM
              • read source (line+column)      • lfe_codegen + compile:forms
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
- **M1 — ADTs & representation** 🚧 *(deftype, constructors, the pluggable backends)*
- **M2 — Pattern matching & exhaustiveness + the diagnostic engine**
- **M3 — Function contracts & bidirectional checking**
- **M4 — The typed/untyped interop boundary**
- **M5 — Polish & dogfooding on real LFE**

Further out: per-expression source mapping (an upstream collaboration with LFE
itself), and a typed-ADT ↔ Rust bridge for [Rustler](https://github.com/rusterlium/rustler)
NIF boundaries.

---

## Status, honestly

`typed` does not work yet as a thing you can `rebar3 add` and start typing your
modules with. What *does* exist is a settled architecture, a working compile chain,
and a build plan executed under real verification discipline. The fun part — the
syntax, the ADTs, the diagnostics — is happening now.

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

## License

Apache-2.0. See [LICENSE](LICENSE). Package name: `lfe_typed`.

# Milestone M0 — Skeleton & Plumbing

> **Goal:** prove the **model-Y compile chain end-to-end** on a trivial typed
> module, including **original-source line injection** — *before* any real type
> theory or ADTs go in. This is the spine every later milestone hangs off.
> **Builds on:** [design doc v0](../01-design-v0.md) §3 · [oxur-sexp reuse](../02-oxur-sexp-reuse.md) · [experiment 01](../experiments/01-lfe-line-anno-probe.md).
> **Ledger:** [M0-skeleton-ledger.md](M0-skeleton-ledger.md) (close every row before M1).
> **CC prompt:** [M0-cc-prompt.md](M0-cc-prompt.md).
> **Iteration budget:** 5 (per ledger discipline). Roles: CC implements, CDC verifies independently.

## What M0 proves (and only this)

A single **vertical slice** of the model-Y architecture, with no type system yet:

```
  hello.lfe  (one  defun/typed  with :args / :returns / :body)
      │  read with column-aware (oxur-sexp) reader
      ▼
  typed-check (Rust):  parse typed surface → TRIVIAL check (shape only) →
      lower to plain LFE  defun  tagged with the ORIGINAL source line →
      [{plain-lfe-form, orig-line}]   (+ column-precise diagnostics on malformed input)
      │  EETF handoff
      ▼
  thin Erlang driver:  lfe_lint:module → lfe_codegen:module(Forms, #cinfo{file=orig})
                       → compile:forms(AST, [{source, orig}])  →  hello.beam
      │
      ▼
  BEAM.  A crash in the body reports hello.lfe:<orig-line>.  A bad ref → a compile
         error at hello.lfe:<orig-line>.  (Headline acceptance: F-8, F-9.)
```

The headline result is **F-8/F-9**: a runtime crash and a compile error both point
at the *original* `.lfe` source line — proving the experiment-01 line-injection
mechanism works through **our** chain, not just in isolation. Everything else in M0
exists to make that demonstrable and repeatable.

## In scope

- Rust `typed-check` crate skeleton; `oxur-sexp` wired in as a dependency
  (factored-out crate preferred; vendored acceptable for M0).
- The **minimal** typed surface: `defun/typed` with `:args`, `:returns`, `:body`.
  (Syntax provisional; only enough to exercise the chain.)
- A **trivial** check: validate the form's *shape* and emit a **column-precise**
  diagnostic on malformed input. No type inference, no contract conformance.
- Lowering `defun/typed` → plain LFE `defun`, paired with original source line.
- EETF handoff Rust → thin Erlang driver.
- Erlang driver: `lfe_lint:module/2` → `lfe_codegen:module/2` (with `#cinfo{file}`)
  → `compile:forms/2` (with `{source, …}`) → `.beam`.
- A `rebar3_lfe`-style provider that drives the chain and **gates** on the check.
- CI harness skeleton with a backend-matrix *axis* (only the trivial path wired).

## Out of scope (these are M1+)

- Any real type checking: contract conformance, inference, exhaustiveness.
- ADTs (`deftype`), typed `case`/`match`, the `repr` backends.
- Cross-module type interface (`.beam` registry attributes).
- `dynamic()` interop, untyped-Erlang boundaries.
- Per-expression / column-level downstream mapping (Tier 3 — upstream `to_expr`).
- Final surface syntax (kept provisional).

## Environment

- Target the local toolchain: **OTP 28 / LFE 2.2.1** (per experiment 01). Record
  exact versions in the closing report.
- Reuse the **exact** line-injection mechanism proven in experiment 01:
  `lfe_codegen:module([{Form, L}], #cinfo{file=F})` then
  `compile:forms(AST, [{source, F}, …])`. Do **not** use `lfe_comp:forms/2` (it
  auto-numbers and discards our lines).
- Note the **API-coupling risk**: `lfe_codegen:module/2`, `lfe_lint:module/2`, and
  `#cinfo` are semi-internal; pin the LFE version.

## Definition of done

Every ledger row reaches a final status (`done` with evidence / `deferred` with
re-entry / `no-op` with rationale). The CC closing report walks the ledger
row-by-row; CDC independently re-runs every `done` row's Verify command. M0 closes
only when F-8 and F-9 (the line-injection headline) are `done` with reproducible
evidence.

## What M0 deliberately de-risks for the rest of the project

- The **oxur-sexp → Rust** front-end (M0 proves it parses real LFE with positions).
- The **Rust → Erlang → BEAM** chain and the EETF handoff.
- **Line injection through our own pipeline** (the model-Y bet).
- The **provider gating** UX.

With those proven, M1 (ADTs + `repr` backends) and M2 (exhaustiveness + diagnostics)
add *content* to a chain already known to work.

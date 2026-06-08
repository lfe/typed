# Milestone M4 — Runtime Enforcement

> **Goal:** make types real at *runtime*, not just at compile time — so a wrong-typed
> value can never silently flow through typed code or corrupt state. This is where
> "**typed user input → type error**" becomes real, and where the static system grows
> teeth on the dynamic BEAM.
> **Builds on:** M0–M3.5 (all closed) — the chain, ADTs, repr backends, matching/
> exhaustiveness, bidirectional contract checking, the diagnostic engine.
> **Design:** [[typed-runtime-enforcement]] (memory) — the full model.
> **Ledger:** [M4-runtime-enforcement-ledger.md](M4-runtime-enforcement-ledger.md).
> **CC prompt:** [M4-cc-prompt.md](M4-cc-prompt.md). **Iteration budget:** 5.

## Why runtime enforcement (static isn't enough on the BEAM)

Static checking guarantees safety *inside* code the checker sees. But the BEAM is
dynamically typed at exactly the seams that matter — HTTP/JSON/form input, messages,
ETS, `binary_to_term`, files. No static check makes an *incoming* term typed. So
soundness "all the way down" on the BEAM = **static interior + runtime enforcement at
the membrane**. M4 builds that runtime half.

## Posture — DECIDED: always-on guards everywhere

Every typed function head gets native `is_*` guards + a type-error fallback. Maximal
safety, maximally let-it-crash: a contract violation becomes a **clean, localized
crash** (a structured type error), never silent corruption — then a supervisor
restarts from a known-good state. This is the BEAM error-kernel philosophy applied to
types. We accept the runtime cost on every typed call; redundant-guard elision for
typed→typed calls is a *future* optimization (M4.5+), not a v0 concern.

## Two mechanisms, two behaviors

| Mechanism | Where | On violation | For |
|---|---|---|---|
| **Native guards** (shallow) | every typed function head | **CRASH** (raise structured type-error) | internal contract enforcement; let-it-crash |
| **Validators / `decode`** (deep, recursive) | the untyped membrane (web input, ETS, messages) | **RETURN** `{error, type_error}` | untrusted external input you handle gracefully (e.g. a 400) |

The split is deliberate: a violated *internal contract* is a bug → crash; *external
input* that doesn't match is expected → return an error you can turn into a response.

## In scope

- **Always-on head guards** generated for every `defun/typed` arg:
  - base types: `integer→is_integer`, `float→is_float`, `binary→is_binary`,
    `atom→is_atom`, `boolean→(is_boolean)`, `string`/`list→is_list`, `map→is_map`;
  - ADT carriers: tagged-tuple → `is_tuple` + tag (+ arity); enum → `is_atom` +
    membership; transparent → the underlying type's guard; native-record → `is_record`
    (OTP 29+; runtime row deferred);
  - a **type-error fallback** that raises a structured error (below).
- **Structured, teaching-grade type-error term:** e.g.
  `#(type_error #{expected => <type>, got => <value-or-type>, function => F, arg => N,
  path => [...]})` — informative enough to log or render a 400 — plus a **human-render
  helper** that prints it Gleam-style.
- **Deep validators:** `(validate <type> term) -> #(ok term) | #(error type_error)`,
  generated per type, **recursive** over ADT fields/nested types.
- **`decode` membrane entry:** `(decode <type> untyped) -> #(ok T) | #(error type_error)`
  — the checked `dynamic → T` conversion; the web-input use case (no crash; graceful).
- **Web-input demo:** a fixture that decodes an untyped term into an ADT — valid →
  `#(ok …)`, invalid → `#(error type_error)` with a teaching message.
- **Cross-backend matrix:** guards + validators correct across tagged-tuple/enum/
  transparent (native-record runtime deferred, OTP 29+); **exact** assertions.
- **Full M0–M3.5 regression** (incl. the README example, now compiled *with* head
  guards); line injection preserved.

## Out of scope (later)

- **Redundant-guard elision** for typed→typed calls (perf) — M4.5+.
- **Native-record runtime** guards/validators (OTP 29+, rides M1-8).
- **Higher-order / contract blame** across fun boundaries; **message/process** boundary
  enforcement (session types — research, never v0).
- **Framework/HTTP integration helpers**; a **disable-guards knob** (default is on).
- **Full BIF prelude validation, bit-syntax, comprehensions.**

## Definition of done

Every ledger row final with SHA-anchored, CI-green evidence. Calling a typed function
with a wrong-typed arg **raises a structured, teaching-grade type error** (snapshot-
tested) across the testable backends; `decode` turns untyped input into `{ok,T}` or a
graceful `{error, type_error}`; the web-input demo works both ways; full M0–M3.5
regression green with guards on.

## Size + split note

Two substantial sub-systems (always-on guards; deep validators + `decode`). **Guards +
the structured error are the non-negotiable core** (the chosen posture). If the budget
tightens, split: **M4** = always-on guards + structured type-error + matrix; **M4.5** =
deep validators + `decode` + web-input demo + guard elision. Propose the split rather
than blow the 5-iteration cap.

## Standing discipline (in force)

[[typed-test-discipline]]: exact `assert_eq!`/snapshots (never `.contains()`); test
every backend (never assume); unwired/cfg-test code is `deferred`, not `done`; a
test/fixture must exercise the criterion's actual subject. [[cc-editing-safety]]: no
blind `sed`; `git checkout` to recover; `make check` after bulk edits.
